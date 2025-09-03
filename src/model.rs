use std::time::{Duration, Instant};

use reqwest::Url;
use thiserror::Error;

use crate::settings::ServiceConfig;

#[derive(Debug, PartialEq, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug, PartialEq, Error)]
pub enum MonitoredServiceError {
    #[error("{0} is not valid url")]
    InvalidUrl(String),
}

#[derive(Debug, Clone)]
pub struct MonitoredService {
    pub url: String,
    pub interval_seconds: u64,
    pub health_status: HealthStatus,
    pub max_retries: usize,
    pub retry_interval: Duration,
    pub last_check: Option<Instant>,
    pub next_check: Option<Instant>,
    pub response_time: Option<Duration>,
    pub error_message: Option<String>,
}

impl MonitoredService {
    pub fn new(
        url: String,
        interval_seconds: u64,
        health_status: HealthStatus,
        max_retries: usize,
        retry_interval: Duration,
    ) -> Result<MonitoredService, MonitoredServiceError> {
        if !is_valid_url(&url) {
            return Err(MonitoredServiceError::InvalidUrl(url));
        }
        let now = Instant::now();
        Ok(Self {
            url,
            interval_seconds,
            health_status,
            max_retries,
            retry_interval,
            last_check: None,
            next_check: Some(now + Duration::from_secs(interval_seconds)),
            response_time: None,
            error_message: None,
        })
    }

    pub fn update_after_check(&mut self, health_status: HealthStatus, response_time: Option<Duration>, error_message: Option<String>) {
        let now = Instant::now();
        self.health_status = health_status;
        self.last_check = Some(now);
        self.next_check = Some(now + Duration::from_secs(self.interval_seconds));
        self.response_time = response_time;
        self.error_message = error_message;
    }
}

fn is_valid_url(input: &str) -> bool {
    Url::parse(input).is_ok()
}

impl TryFrom<&ServiceConfig> for MonitoredService {
    type Error = MonitoredServiceError;

    fn try_from(service: &ServiceConfig) -> Result<Self, Self::Error> {
        Self::new(
            service.url.clone(),
            service.interval_seconds,
            HealthStatus::Healthy,
            service.max_retries,
            Duration::from_secs(service.retry_interval),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_configuration_error_when_url_is_invalid() {
        let config = ServiceConfig {
            url: "".to_string(),
            interval_seconds: 3,
            max_retries: 3,
            retry_interval: 333,
        };

        let actual = MonitoredService::try_from(&config);

        assert!(actual.is_err());
    }
}
