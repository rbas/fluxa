use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::model::{HealthStatus, MonitoringStats};

pub type SharedMonitoringState = Arc<RwLock<MonitoringState>>;

#[derive(Debug, Default)]
pub struct MonitoringState {
    services: HashMap<String, MonitoringStats>,
}

impl MonitoringState {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn update_service_stats(
        &mut self,
        url: String,
        status: HealthStatus,
        response_time_ms: Option<u64>,
        error: Option<String>,
        retry_count: usize,
        next_check_in_seconds: u64,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let stats = MonitoringStats {
            status,
            last_response_time_ms: response_time_ms,
            last_check_timestamp: now,
            next_check_timestamp: now + next_check_in_seconds,
            last_error: error,
            current_retry_count: retry_count,
        };

        self.services.insert(url, stats);
    }

    pub fn get_all_services(&self) -> HashMap<String, MonitoringStats> {
        self.services.clone()
    }

    pub fn get_service(&self, url: &str) -> Option<MonitoringStats> {
        self.services.get(url).cloned()
    }
}

pub async fn update_service_state(
    state: &SharedMonitoringState,
    url: String,
    status: HealthStatus,
    response_time_ms: Option<u64>,
    error: Option<String>,
    retry_count: usize,
    next_check_in_seconds: u64,
) {
    let mut state_guard = state.write().await;
    state_guard.update_service_stats(
        url,
        status,
        response_time_ms,
        error,
        retry_count,
        next_check_in_seconds,
    );
}

pub async fn get_all_service_states(state: &SharedMonitoringState) -> HashMap<String, MonitoringStats> {
    let state_guard = state.read().await;
    state_guard.get_all_services()
}

pub async fn get_service_state(
    state: &SharedMonitoringState,
    url: &str,
) -> Option<MonitoringStats> {
    let state_guard = state.read().await;
    state_guard.get_service(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_state_update_and_retrieval() {
        let state = Arc::new(RwLock::new(MonitoringState::new()));

        // Update service state
        update_service_state(
            &state,
            "https://example.com".to_string(),
            HealthStatus::Healthy,
            Some(250),
            None,
            0,
            300,
        ).await;

        // Retrieve specific service state
        let service_state = get_service_state(&state, "https://example.com").await;
        assert!(service_state.is_some());
        
        let stats = service_state.unwrap();
        assert_eq!(stats.status, HealthStatus::Healthy);
        assert_eq!(stats.last_response_time_ms, Some(250));
        assert_eq!(stats.last_error, None);
        assert_eq!(stats.current_retry_count, 0);

        // Retrieve all service states
        let all_states = get_all_service_states(&state).await;
        assert_eq!(all_states.len(), 1);
        assert!(all_states.contains_key("https://example.com"));
    }

    #[tokio::test]
    async fn test_monitoring_state_update_with_error() {
        let state = Arc::new(RwLock::new(MonitoringState::new()));

        // Update service state with error
        update_service_state(
            &state,
            "https://example.com".to_string(),
            HealthStatus::Unhealthy,
            None,
            Some("HTTP 500 - Internal Server Error".to_string()),
            2,
            60,
        ).await;

        // Retrieve service state
        let service_state = get_service_state(&state, "https://example.com").await;
        assert!(service_state.is_some());
        
        let stats = service_state.unwrap();
        assert_eq!(stats.status, HealthStatus::Unhealthy);
        assert_eq!(stats.last_response_time_ms, None);
        assert_eq!(stats.last_error, Some("HTTP 500 - Internal Server Error".to_string()));
        assert_eq!(stats.current_retry_count, 2);
    }

    #[tokio::test]
    async fn test_monitoring_state_multiple_services() {
        let state = Arc::new(RwLock::new(MonitoringState::new()));

        // Add multiple services
        update_service_state(
            &state,
            "https://service1.com".to_string(),
            HealthStatus::Healthy,
            Some(100),
            None,
            0,
            300,
        ).await;

        update_service_state(
            &state,
            "https://service2.com".to_string(),
            HealthStatus::Unhealthy,
            None,
            Some("Connection timeout".to_string()),
            1,
            60,
        ).await;

        // Retrieve all services
        let all_states = get_all_service_states(&state).await;
        assert_eq!(all_states.len(), 2);
        
        // Check individual services
        let service1 = get_service_state(&state, "https://service1.com").await.unwrap();
        assert_eq!(service1.status, HealthStatus::Healthy);
        assert_eq!(service1.current_retry_count, 0);

        let service2 = get_service_state(&state, "https://service2.com").await.unwrap();
        assert_eq!(service2.status, HealthStatus::Unhealthy);
        assert_eq!(service2.current_retry_count, 1);
    }
}