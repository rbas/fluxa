use std::time::Duration;

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

#[derive(Debug)]
pub struct MonitoredService {
    pub url: String,
    pub interval_seconds: u64,
    pub health_status: HealthStatus,
    pub max_retries: usize,
    pub retry_interval: Duration,
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
        Ok(Self {
            url,
            interval_seconds,
            health_status,
            max_retries,
            retry_interval,
        })
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
