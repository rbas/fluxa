use std::{error::Error, fmt, time::Duration};

use reqwest::Url;

use crate::settings::{ServiceConfig, ServiceConfigurationError};

#[derive(Debug, PartialEq, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug)]
pub enum MonitoredServiceError {
    InvalidUrl(String),
}
impl fmt::Display for MonitoredServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitoredServiceError::InvalidUrl(s) => {
                write!(f, "{} is not valid url", s)
            }
        }
    }
}
impl Error for MonitoredServiceError {}

impl From<MonitoredServiceError> for ServiceConfigurationError {
    fn from(error: MonitoredServiceError) -> Self {
        match error {
            MonitoredServiceError::InvalidUrl(s) => {
                Self::ErrorInConfiguration(format!("Please fix the following problem: {}", s))
            }
        }
    }
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
    type Error = ServiceConfigurationError;

    fn try_from(service: &ServiceConfig) -> Result<Self, Self::Error> {
        Ok(Self::new(
            service.url.clone(),
            service.interval_seconds,
            HealthStatus::Healthy,
            service.max_retries,
            Duration::from_secs(service.retry_interval),
        )?)
    }
}
