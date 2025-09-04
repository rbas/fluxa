use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use clap::{builder::PathBufValueParser, Arg, Command};
use fluxa::{
    error::FluxaError,
    http::{spawn_web_server, AppState},
    notification::Notifier,
    service::{build_services, monitor_url},
    settings::FluxaConfig,
    state::{MonitoringState, SharedMonitoringState},
};
use log::{error, info};

#[cfg(unix)]
use fluxa::http::spawn_unix_socket_server;

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
        .get_matches();

    let default_file = PathBuf::from("config.local.toml");
    let config_path = matches.get_one("config").unwrap_or(&default_file);

    let conf = FluxaConfig::new(config_path.as_path())?;

    let notifier = Notifier::new(
        conf.pushover_api_key.clone(),
        conf.pushover_user_key.clone(),
    );

    // Configuration for monitoring
    let services = build_services(&conf)?;

    // Create shared monitoring state
    let monitoring_state: SharedMonitoringState = Arc::new(RwLock::new(MonitoringState::new()));

    info!("Spawning monitoring");

    // Spawn monitoring tasks
    let mut handles = vec![];
    for service in services.clone() {
        let notifier_clone = notifier.clone();
        let state_clone = monitoring_state.clone();
        let handle = tokio::spawn(async move {
            monitor_url(service, notifier_clone, state_clone).await
            // TODO Handle errors
        });
        handles.push(handle);
    }

    // Create app state for HTTP server
    let app_state = AppState {
        monitoring_state: monitoring_state.clone(),
        services,
    };

    // Spawn Unix socket server if configured
    #[cfg(unix)]
    if let Some(socket_path) = &conf.fluxa.api_socket {
        let unix_app_state = app_state.clone();
        let socket_path = socket_path.clone();
        tokio::spawn(async move {
            if let Err(e) = spawn_unix_socket_server(&socket_path, unix_app_state).await {
                error!("Unix socket server error: {}", e);
            }
        });
    }

    // Spawning web server for monitoring that service is alive
    let socket_addr = conf.fluxa.listen.as_str();
    spawn_web_server(socket_addr, app_state).await?;

    // Wait for all tasks to complete (they will run indefinitely)
    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
