use log::{debug, error, info, warn};
use reqwest::Client;
use std::time::Instant;
use tokio::time::{self, Duration};

use crate::{
    error::ServiceError,
    model::{HealthStatus, MonitoredService},
    notification::Notifier,
    settings::FluxaConfig,
    state::{update_service_state, SharedMonitoringState},
};

async fn send_request(
    client: &Client,
    service: &mut MonitoredService,
    notifier: &Notifier,
    state: &SharedMonitoringState,
) -> Result<HealthStatus, ServiceError> {
    let mut current_health = HealthStatus::Unhealthy;
    let mut response_time_ms = None;
    let mut last_error = None;
    let mut retry_count = 0;

    for attempt in 0..=service.max_retries {
        retry_count = attempt;
        let start_time = Instant::now();
        
        match client.get(&service.url).send().await {
            Ok(response) => {
                let elapsed = start_time.elapsed();
                response_time_ms = Some(elapsed.as_millis() as u64);
                
                if response.status().is_success() {
                    current_health = HealthStatus::Healthy;
                    last_error = None;
                    break;
                } else {
                    let error_msg = format!("HTTP {} - {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"));
                    last_error = Some(error_msg.clone());
                    debug!(
                        "Request to {} failed with status: {}",
                        service.url,
                        response.status()
                    );
                }
            }
            Err(e) => {
                let error_msg = format!("Request failed: {}", e);
                last_error = Some(error_msg.clone());
                
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

    // Update shared state
    update_service_state(
        state,
        service.url.clone(),
        current_health.clone(),
        response_time_ms,
        last_error,
        retry_count,
        service.interval_seconds,
    ).await;

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
    state: SharedMonitoringState,
) -> Result<(), ServiceError> {
    loop {
        send_request(&Client::new(), &mut service, &notifier, &state).await?;
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
