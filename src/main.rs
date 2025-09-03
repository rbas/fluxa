use std::path::PathBuf;

use clap::{builder::PathBufValueParser, Arg, Command};
use fluxa::{
    dashboard::{Dashboard, DashboardEvent},
    error::FluxaError,
    http::spawn_web_server,
    notification::Notifier,
    service::{build_services, monitor_url},
    settings::FluxaConfig,
};
use log::info;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), FluxaError> {
    // Set up logging
    env_logger::init();

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file")
                .value_parser(PathBufValueParser::default()),
        )
        .arg(
            Arg::new("dashboard")
                .short('d')
                .long("dashboard")
                .help("Run in dashboard mode with terminal UI")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let default_file = PathBuf::from("config.local.toml");
    let config_path = matches.get_one("config").unwrap_or(&default_file);
    let dashboard_mode = matches.get_flag("dashboard");

    let conf = FluxaConfig::new(config_path.as_path())?;

    let notifier = Notifier::new(
        conf.pushover_api_key.clone(),
        conf.pushover_user_key.clone(),
    );

    // Configuration for monitoring
    let services = build_services(&conf)?;

    info!("Spawning monitoring");

    // Create dashboard channel if in dashboard mode
    let (dashboard_tx, dashboard_rx) = if dashboard_mode {
        let (tx, rx) = mpsc::unbounded_channel::<DashboardEvent>();
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    // Spawn monitoring tasks
    let mut handles = vec![];
    for service in services {
        let notifier_clone = notifier.clone();
        let dashboard_tx_clone = dashboard_tx.clone();
        let handle = tokio::spawn(async move {
            monitor_url(service, notifier_clone, dashboard_tx_clone).await
            // TODO Handle errors
        });
        handles.push(handle);
    }

    if dashboard_mode {
        // Run dashboard
        if let Some(rx) = dashboard_rx {
            let mut dashboard = Dashboard::new(rx).map_err(|e| FluxaError::Other(e.to_string()))?;
            
            // Spawn web server in background if needed
            let socket_addr = conf.fluxa.listen.clone();
            tokio::spawn(async move {
                if let Err(e) = spawn_web_server(&socket_addr).await {
                    eprintln!("Web server error: {}", e);
                }
            });

            // Run dashboard (this will block until user exits)
            if let Err(e) = dashboard.run().await {
                eprintln!("Dashboard error: {}", e);
            }
        }
    } else {
        // Original headless mode
        // Spawning web server for monitoring that service is alive
        let socket_addr = conf.fluxa.listen.as_str();
        spawn_web_server(socket_addr).await?;

        // Wait for all tasks to complete (they will run indefinitely)
        for handle in handles {
            let _ = handle.await?;
        }
    }

    Ok(())
}
