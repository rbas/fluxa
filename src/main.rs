use std::path::PathBuf;

use clap::{builder::PathBufValueParser, Arg, Command};
use fluxa::http::WebServer;
use fluxa::{
    error::FluxaError, monitoring::MonitoringService, notification::NotificationManager,
    settings::FluxaConfig,
};
use log::info;

#[tokio::main]
async fn main() -> Result<(), FluxaError> {
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

    let http_client = std::sync::Arc::new(reqwest::Client::new());
    let notification_manager =
        std::sync::Arc::new(NotificationManager::from_config(&conf, http_client.clone()));

    let monitoring_service =
        MonitoringService::new(http_client, notification_manager, conf.services)?;

    let web_server = WebServer::new(conf.fluxa.listen);

    info!("🚀 Starting Fluxa with monitoring + web server");

    tokio::select! {
        monitoring_result = monitoring_service.run() => {
            match monitoring_result {
                Ok(_) => {
                    log::warn!("Monitoring service completed unexpectedly");
                    Ok(())
                }
                Err(e) => {
                    log::error!("Monitoring service failed: {}", e);
                    Err(e)
                }
            }
        }

        web_server_result = web_server.run() => {
            match web_server_result {
                Ok(_) => {
                    log::warn!("Web server completed unexpectedly");
                    Ok(())
                }
                Err(e) => {
                    log::error!("Web server failed: {}", e);
                    Err(FluxaError::Http(e))
                }
            }
        }
    }
}
