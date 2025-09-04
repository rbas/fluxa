use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::settings::ServiceConfig;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug, PartialEq, Error)]
pub enum MonitoredServiceError {
    #[error("{0} is not valid url")]
    InvalidUrl(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub status: HealthStatus,
    pub last_response_time_ms: Option<u64>,
    pub last_check_timestamp: u64,
    pub next_check_timestamp: u64,
    pub last_error: Option<String>,
    pub current_retry_count: usize,
}

impl Default for MonitoringStats {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            status: HealthStatus::Healthy,
            last_response_time_ms: None,
            last_check_timestamp: now,
            next_check_timestamp: now,
            last_error: None,
            current_retry_count: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub url: String,
    pub interval_seconds: u64,
    pub max_retries: usize,
    pub retry_interval_seconds: u64,
    pub stats: MonitoringStats,
}

#[derive(Debug, Clone)]
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

    pub fn to_service_info(&self, stats: MonitoringStats) -> ServiceInfo {
        ServiceInfo {
            url: self.url.clone(),
            interval_seconds: self.interval_seconds,
            max_retries: self.max_retries,
            retry_interval_seconds: self.retry_interval.as_secs(),
            stats,
        }
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
    use serde_json;

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

    #[test]
    fn test_monitoring_stats_serialization() {
        let stats = MonitoringStats {
            status: HealthStatus::Healthy,
            last_response_time_ms: Some(150),
            last_check_timestamp: 1672531200,
            next_check_timestamp: 1672531500,
            last_error: None,
            current_retry_count: 0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: MonitoringStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.status, deserialized.status);
        assert_eq!(stats.last_response_time_ms, deserialized.last_response_time_ms);
        assert_eq!(stats.last_check_timestamp, deserialized.last_check_timestamp);
        assert_eq!(stats.next_check_timestamp, deserialized.next_check_timestamp);
        assert_eq!(stats.last_error, deserialized.last_error);
        assert_eq!(stats.current_retry_count, deserialized.current_retry_count);
    }

    #[test]
    fn test_service_info_serialization() {
        let stats = MonitoringStats::default();
        let service_info = ServiceInfo {
            url: "https://example.com".to_string(),
            interval_seconds: 60,
            max_retries: 3,
            retry_interval_seconds: 5,
            stats,
        };

        let json = serde_json::to_string(&service_info).unwrap();
        let deserialized: ServiceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(service_info.url, deserialized.url);
        assert_eq!(service_info.interval_seconds, deserialized.interval_seconds);
        assert_eq!(service_info.max_retries, deserialized.max_retries);
        assert_eq!(service_info.retry_interval_seconds, deserialized.retry_interval_seconds);
    }

    #[test]
    fn test_monitored_service_to_service_info() {
        let service = MonitoredService::new(
            "https://test.com".to_string(),
            120,
            HealthStatus::Healthy,
            2,
            Duration::from_secs(10),
        ).unwrap();

        let stats = MonitoringStats {
            status: HealthStatus::Unhealthy,
            last_response_time_ms: Some(300),
            last_check_timestamp: 1672531200,
            next_check_timestamp: 1672531320,
            last_error: Some("Timeout".to_string()),
            current_retry_count: 1,
        };

        let service_info = service.to_service_info(stats.clone());

        assert_eq!(service_info.url, "https://test.com");
        assert_eq!(service_info.interval_seconds, 120);
        assert_eq!(service_info.max_retries, 2);
        assert_eq!(service_info.retry_interval_seconds, 10);
        assert_eq!(service_info.stats.status, HealthStatus::Unhealthy);
        assert_eq!(service_info.stats.current_retry_count, 1);
    }
}
