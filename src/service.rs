use log::{debug, error, info, warn};
use reqwest::Client;
use tokio::time::{self, Duration, Instant};
use tokio::sync::mpsc;

use crate::{
    dashboard::DashboardEvent,
    error::ServiceError,
    model::{HealthStatus, MonitoredService},
    notification::Notifier,
    settings::FluxaConfig,
};

async fn send_request(
    client: &Client,
    service: &mut MonitoredService,
    notifier: &Notifier,
    dashboard_tx: Option<&mpsc::UnboundedSender<DashboardEvent>>,
) -> Result<HealthStatus, ServiceError> {
    let mut current_health = HealthStatus::Unhealthy;
    let mut response_time = None;
    let mut error_message = None;
    let previous_health = service.health_status.clone();
    
    for attempt in 0..=service.max_retries {
        let start_time = Instant::now();
        
        match client.get(&service.url).send().await {
            Ok(response) => {
                let elapsed = start_time.elapsed();
                response_time = Some(elapsed);
                
                if response.status().is_success() {
                    current_health = HealthStatus::Healthy;
                    error_message = None;
                    break;
                } else {
                    let error_msg = format!("HTTP {}: {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or("Unknown"));
                    error_message = Some(error_msg.clone());
                    debug!(
                        "Request to {} failed with status: {}",
                        service.url,
                        response.status()
                    );
                }
            }
            Err(e) => {
                error_message = Some(format!("Request failed: {}", e));
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

    // Update service with new information
    service.update_after_check(current_health.clone(), response_time, error_message.clone());

    // Send update to dashboard if channel exists
    if let Some(tx) = dashboard_tx {
        let _ = tx.send(DashboardEvent::ServiceUpdate(service.url.clone(), service.clone()));
    }

    // Check if status changed and send notifications
    if current_health != previous_health {
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
    }

    Ok(current_health)
}

pub async fn monitor_url(
    mut service: MonitoredService,
    notifier: Notifier,
    dashboard_tx: Option<mpsc::UnboundedSender<DashboardEvent>>,
) -> Result<(), ServiceError> {
    loop {
        send_request(&Client::new(), &mut service, &notifier, dashboard_tx.as_ref()).await?;
        time::sleep(Duration::from_secs(service.interval_seconds)).await;
    }
}

pub fn build_services(conf: &FluxaConfig) -> Result<Vec<MonitoredService>, ServiceError> {
    let mut services: Vec<MonitoredService> = vec![];

    for service in &conf.services {
        services.push(MonitoredService::try_from(service)?)
    }

    Ok(services)
}
