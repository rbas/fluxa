use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct FluxaConfig {
    pushover_api_key: String,
    pushover_user_key: String,
    services: Vec<ServiceConfig>,
}

impl FluxaConfig {
    pub fn pushover_user_key(&self) -> &str {
        &self.pushover_user_key
    }

    pub fn pushover_api_key(&self) -> &str {
        &self.pushover_api_key
    }

    pub fn services(&self) -> &[ServiceConfig] {
        &self.services
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct ServiceConfig {
    pub url: String,
    pub interval_seconds: u64,
    pub max_retries: usize,
    pub retry_interval: u64,
}
