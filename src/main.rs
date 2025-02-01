use std::net::SocketAddr;

use axum::{routing::get, Router};
use config::{Config, File};
use fluxa::{notification::pushover_notification, settings::FluxaConfig};
use log::{debug, error, info, warn};
use reqwest::Client;
use tokio::time::{self, Duration};

#[derive(Debug, PartialEq, Clone)]
enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug)]
struct MonitorConfig {
    url: String,
    interval_seconds: u64,
    health_status: HealthStatus,
    max_retries: usize,
    retry_interval: Duration,
}

async fn send_request(
    client: &Client,
    config: &mut MonitorConfig,
    conf: &FluxaConfig,
) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut current_health = HealthStatus::Unhealthy;
    for attempt in 0..=config.max_retries {
        match client.get(&config.url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    current_health = HealthStatus::Healthy;
                    break;
                } else {
                    debug!(
                        "Request to {} failed with status: {}",
                        config.url,
                        response.status()
                    );
                }
            }
            Err(_) => {
                if attempt < config.max_retries {
                    debug!(
                        "Attempt {} to {} failed. Retrying in {:?}...",
                        attempt + 1,
                        config.url,
                        config.retry_interval
                    );
                    time::sleep(config.retry_interval).await;
                } else {
                    debug!(
                        "Max retries ({}) exceeded for {}",
                        config.max_retries, config.url
                    );
                    current_health = HealthStatus::Unhealthy;
                    break;
                }
            }
        }
    }

    if current_health != config.health_status {
        if current_health == HealthStatus::Healthy {
            let message = format!("{} is now healthy!", config.url);
            info!("{}", &message);

            let result =
                pushover_notification(conf.pushover_api_key(), conf.pushover_user_key(), &message)
                    .await;

            if result.is_err() {
                error!("Problem with PushOver service {:?}", result.err());
            }
        } else {
            let message = format!("{} is unhealthy!", config.url);
            warn!("{}", &message);

            let result =
                pushover_notification(conf.pushover_api_key(), conf.pushover_user_key(), &message)
                    .await;

            if result.is_err() {
                error!("Problem with PushOver service {:?}", result.err());
            }
        }
        config.health_status = current_health.clone();
    }

    Ok(current_health)
}

async fn monitor_url(
    mut config: MonitorConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = Config::builder()
        .add_source(File::with_name("config.local.toml"))
        .build()?;

    let conf: FluxaConfig = settings.try_deserialize()?;

    loop {
        send_request(&Client::new(), &mut config, &conf).await?;
        time::sleep(Duration::from_secs(config.interval_seconds)).await;
    }
}

async fn status_handler() -> &'static str {
    "Ok"
}

async fn spawn_webserver() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new().route("/", get(status_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Set up logging
    env_logger::init();

    // Configuration for monitoring
    let configs = vec![MonitorConfig {
        url: "http://localhost:3000".to_string(),
        interval_seconds: 5,
        health_status: HealthStatus::Healthy,
        max_retries: 3,
        retry_interval: Duration::from_secs(1),
    }];

    info!("Spawning monitoring");

    // Spawn monitoring tasks
    let mut handles = vec![];
    for config in configs {
        let handle = tokio::spawn(async move { monitor_url(config).await });
        handles.push(handle);
    }

    spawn_webserver().await?;

    // Wait for all tasks to complete (they will run indefinitely)
    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
