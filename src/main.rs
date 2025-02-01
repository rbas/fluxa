use core::fmt;
use std::error::Error;
use std::net::SocketAddr;

use axum::{routing::get, Router};
use config::{Config, File};
use fluxa::{
    notification::pushover_notification,
    settings::{FluxaConfig, ServiceConfig},
};
use log::{debug, error, info, warn};
use reqwest::Client;
use tokio::time::{self, Duration};

#[derive(Debug, PartialEq, Clone)]
enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug)]
enum ServiceConfigurationError {
    ErrorInConfiguration(String),
}
impl fmt::Display for ServiceConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceConfigurationError::ErrorInConfiguration(s) => {
                write!(f, "Configuration error in {}", s)
            }
        }
    }
}
impl Error for ServiceConfigurationError {}

#[derive(Debug)]
struct MonitoredService {
    url: String,
    interval_seconds: u64,
    health_status: HealthStatus,
    max_retries: usize,
    retry_interval: Duration,
}

impl TryFrom<&ServiceConfig> for MonitoredService {
    type Error = ServiceConfigurationError;

    fn try_from(service: &ServiceConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            url: service.url.clone(),
            interval_seconds: service.interval_seconds,
            health_status: HealthStatus::Healthy,
            max_retries: service.max_retries,
            retry_interval: Duration::from_secs(service.retry_interval),
        })
    }
}

async fn send_request(
    client: &Client,
    service: &mut MonitoredService,
    conf: &FluxaConfig,
) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut current_health = HealthStatus::Unhealthy;
    for attempt in 0..=service.max_retries {
        match client.get(&service.url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    current_health = HealthStatus::Healthy;
                    break;
                } else {
                    debug!(
                        "Request to {} failed with status: {}",
                        service.url,
                        response.status()
                    );
                }
            }
            Err(_) => {
                if attempt < service.max_retries {
                    debug!(
                        "Attempt {} to {} failed. Retrying in {:?}...",
                        attempt + 1,
                        service.url,
                        service.retry_interval
                    );
                    time::sleep(service.retry_interval).await;
                } else {
                    debug!(
                        "Max retries ({}) exceeded for {}",
                        service.max_retries, service.url
                    );
                    current_health = HealthStatus::Unhealthy;
                    break;
                }
            }
        }
    }

    if current_health != service.health_status {
        if current_health == HealthStatus::Healthy {
            let message = format!("{} is now healthy!", service.url);
            info!("{}", &message);

            let result =
                pushover_notification(conf.pushover_api_key(), conf.pushover_user_key(), &message)
                    .await;

            if result.is_err() {
                error!("Problem with PushOver service {:?}", result.err());
            }
        } else {
            let message = format!("{} is unhealthy!", service.url);
            warn!("{}", &message);

            let result =
                pushover_notification(conf.pushover_api_key(), conf.pushover_user_key(), &message)
                    .await;

            if result.is_err() {
                error!("Problem with PushOver service {:?}", result.err());
            }
        }
        service.health_status = current_health.clone();
    }

    Ok(current_health)
}

async fn monitor_url(
    mut service: MonitoredService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = Config::builder()
        .add_source(File::with_name("config.local.toml"))
        .build()?;

    let conf: FluxaConfig = settings.try_deserialize()?;

    loop {
        send_request(&Client::new(), &mut service, &conf).await?;
        time::sleep(Duration::from_secs(service.interval_seconds)).await;
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

fn build_services() -> Result<Vec<MonitoredService>, Box<dyn std::error::Error + Send + Sync>> {
    let settings = Config::builder()
        .add_source(File::with_name("config.local.toml"))
        .build()?;

    let conf: FluxaConfig = settings.try_deserialize()?;

    let mut services: Vec<MonitoredService> = vec![];

    for service in conf.services() {
        services.push(MonitoredService::try_from(service)?)
    }

    Ok(services)
}

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
        let handle = tokio::spawn(async move { monitor_url(service).await });
        handles.push(handle);
    }

    spawn_webserver().await?;

    // Wait for all tasks to complete (they will run indefinitely)
    for handle in handles {
        let _ = handle.await?;
    }

    Ok(())
}
