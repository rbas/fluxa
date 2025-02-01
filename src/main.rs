use fluxa::{
    http::spawn_web_server,
    service::{build_services, monitor_url},
};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Set up logging
    env_logger::init();

    // Configuration for monitoring
    let services = build_services()?;

    info!("Spawning monitoring");

    // Spawn monitoring tasks
    let mut handles = vec![];
    for service in services {
        let handle = tokio::spawn(async move {
            // TODO Handle errors
            monitor_url(service).await
        });
        handles.push(handle);
    }

    // Spawning web server for monitoring that service is alive
    spawn_web_server().await?;

    // Wait for all tasks to complete (they will run indefinitely)
    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
