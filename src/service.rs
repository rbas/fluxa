use log::{debug, error, info, warn};
use reqwest::Client;
use tokio::time::{self, Duration};

use crate::{
    model::{HealthStatus, MonitoredService},
    notification::Notifier,
    settings::FluxaConfig,
};

async fn send_request(
    client: &Client,
    service: &mut MonitoredService,
    notifier: &Notifier,
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

            let result = notifier.send(&message).await;

            if result.is_err() {
                error!("Problem with PushOver service {:?}", result.err());
            }
        } else {
            let message = format!("{} is unhealthy!", service.url);
            warn!("{}", &message);

            let result = notifier.send(&message).await;

            if result.is_err() {
                error!("Problem with PushOver service {:?}", result.err());
            }
        }
        service.health_status = current_health.clone();
    }

    Ok(current_health)
}

pub async fn monitor_url(
    mut service: MonitoredService,
    notifier: Notifier,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        send_request(&Client::new(), &mut service, &notifier).await?;
        time::sleep(Duration::from_secs(service.interval_seconds)).await;
    }
}

pub fn build_services(
    conf: &FluxaConfig,
) -> Result<Vec<MonitoredService>, Box<dyn std::error::Error + Send + Sync>> {
    let mut services: Vec<MonitoredService> = vec![];

    for service in &conf.services {
        services.push(MonitoredService::try_from(service)?)
    }

    Ok(services)
}
