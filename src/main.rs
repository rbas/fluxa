use std::path::PathBuf;

use clap::{builder::PathBufValueParser, Arg, Command};
use fluxa::{
    http::spawn_web_server,
    notification::Notifier,
    service::{build_services, monitor_url},
    settings::FluxaConfig,
};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    info!("Spawning monitoring");

    // Spawn monitoring tasks
    let mut handles = vec![];
    for service in services {
        let notifier_clone = notifier.clone();
        let handle = tokio::spawn(async move {
            monitor_url(service, notifier_clone).await
            // TODO Handle errors
        });
        handles.push(handle);
    }

    // Spawning web server for monitoring that service is alive
    let socket_addr = conf.fluxa.listen.as_str();
    spawn_web_server(socket_addr).await?;

    // Wait for all tasks to complete (they will run indefinitely)
    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
