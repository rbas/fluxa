use config::{Config, File};
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

    let config_path = "config.local.toml";

    let settings = Config::builder()
        .add_source(File::with_name(config_path))
        .build()?;

    let conf: FluxaConfig = settings.try_deserialize()?;

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
