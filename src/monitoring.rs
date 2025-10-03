use log::{debug, error, info, warn};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time;

use crate::error::{FluxaError, ServiceError};
use crate::model::{HealthStatus, MonitoredService};
use crate::notification::NotificationManager;
use crate::settings::{ServiceConfig, ServiceConfigurationError};

#[derive(Debug)]
pub struct ServiceMonitor {
    pub service: MonitoredService,
    pub http_client: Arc<Client>,
    pub notification_manager: Arc<NotificationManager>,
}

impl ServiceMonitor {
    pub fn new(
        service: MonitoredService,
        http_client: Arc<Client>,
        notification_manager: Arc<NotificationManager>,
    ) -> Self {
        Self {
            service,
            http_client,
            notification_manager,
        }
    }

    pub async fn start_monitoring(mut self) -> Result<(), ServiceError> {
        loop {
            self.perform_health_check().await?;
            time::sleep(Duration::from_secs(self.service.interval_seconds)).await;
        }
    }

    async fn perform_health_check(&mut self) -> Result<(), ServiceError> {
        let mut current_health = HealthStatus::Unhealthy;

        // Retry logic moved from service.rs
        for attempt in 0..=self.service.max_retries {
            match self.http_client.get(&self.service.url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        current_health = HealthStatus::Healthy;
                        break;
                    } else {
                        debug!(
                            "Request to {} failed with status: {}",
                            self.service.url,
                            response.status()
                        );
                    }
                }
                Err(_) => {
                    if attempt < self.service.max_retries {
                        debug!(
                            "Attempt {} to {} failed. Retrying in {:?}...",
                            attempt + 1,
                            self.service.url,
                            self.service.retry_interval
                        );
                        time::sleep(self.service.retry_interval).await;
                    } else {
                        debug!(
                            "Max retries ({}) exceeded for {}",
                            self.service.max_retries, self.service.url
                        );
                        current_health = HealthStatus::Unhealthy;
                        break;
                    }
                }
            }
        }

        self.handle_status_change(current_health).await?;

        Ok(())
    }

    /// Handle health status changes and send notifications
    async fn handle_status_change(
        &mut self,
        current_health: HealthStatus,
    ) -> Result<(), ServiceError> {
        if current_health != self.service.health_status {
            if current_health == HealthStatus::Healthy {
                let message = format!("{} is now healthy!", self.service.url);
                info!("{}", &message);

                if let Err(e) = self.notification_manager.send_notification(&message).await {
                    error!("Problem sending notification: {:?}", e);
                }
            } else {
                let message = format!("{} is unhealthy!", self.service.url);
                warn!("{}", &message);

                if let Err(e) = self.notification_manager.send_notification(&message).await {
                    error!("Problem sending notification: {:?}", e);
                }
            }
            self.service.health_status = current_health.clone();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MonitoringService {
    http_client: Arc<reqwest::Client>,
    notification_manager: Arc<NotificationManager>,
    service_monitors: Vec<ServiceMonitor>,
    task_handles: Vec<JoinHandle<Result<(), ServiceError>>>,
}

impl MonitoringService {
    pub fn new(
        http_client: Arc<reqwest::Client>,
        notification_manager: Arc<NotificationManager>,
        service_configs: Vec<crate::settings::ServiceConfig>,
    ) -> Result<Self, FluxaError> {
        debug!(
            "Creating new MonitoringService with {} services",
            service_configs.len()
        );

        let mut service = Self {
            http_client: http_client.clone(),
            notification_manager: notification_manager.clone(),
            service_monitors: Vec::new(),
            task_handles: Vec::new(),
        };

        service.create_services_from_config(service_configs);

        Ok(service)
    }

    pub async fn run(mut self) -> Result<(), FluxaError> {
        self.start_all_monitoring().await?;
        self.wait_for_any_task_completion().await
    }

    async fn start_all_monitoring(&mut self) -> Result<(), FluxaError> {
        if self.service_monitors.is_empty() {
            return Err(FluxaError::Configuration(
                ServiceConfigurationError::ErrorInConfiguration(
                    "No services configured for monitoring".to_string(),
                ),
            ));
        }

        info!(
            "🚀 Starting monitoring for {} services",
            self.service_monitors.len()
        );

        self.task_handles.clear();

        let service_monitors = std::mem::take(&mut self.service_monitors);

        for monitor in service_monitors {
            let service_url = monitor.service.url.clone();
            debug!("Spawning monitoring task for: {}", service_url);

            let handle = tokio::spawn(async move {
                match monitor.start_monitoring().await {
                    Ok(_) => {
                        warn!("Monitoring for {} completed unexpectedly", service_url);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Monitoring failed for {}: {}", service_url, e);
                        Err(e)
                    }
                }
            });

            self.task_handles.push(handle);
        }

        info!(
            "✅ Successfully started {} monitoring tasks",
            self.task_handles.len()
        );
        Ok(())
    }

    async fn wait_for_any_task_completion(&mut self) -> Result<(), FluxaError> {
        if self.task_handles.is_empty() {
            return Err(FluxaError::Configuration(
                ServiceConfigurationError::ErrorInConfiguration(
                    "No running monitoring tasks.".to_string(),
                ),
            ));
        }

        if let Some(handle) = self.task_handles.pop() {
            match handle.await {
                Ok(service_result) => match service_result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(FluxaError::Service(e)),
                },
                Err(e) => Err(FluxaError::TaskJoin(e)),
            }
        } else {
            Err(FluxaError::Configuration(
                ServiceConfigurationError::ErrorInConfiguration(
                    "No monitoring tasks to wait for".to_string(),
                ),
            ))
        }
    }

    fn create_services_from_config(&mut self, service_configs: Vec<ServiceConfig>) {
        info!(
            "Creating {} services from configuration",
            service_configs.len()
        );

        for config in service_configs {
            match MonitoredService::try_from(&config) {
                Ok(monitored_service) => {
                    let monitor = ServiceMonitor::new(
                        monitored_service,
                        self.http_client.clone(),
                        self.notification_manager.clone(),
                    );

                    debug!("Creating service monitor for: {}", monitor.service.url);
                    self.service_monitors.push(monitor);
                }
                Err(e) => {
                    error!("Failed to create service monitor for {}: {}", config.url, e);
                }
            }
        }
    }
}
