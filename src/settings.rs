use std::{error::Error, fmt};

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

#[derive(Debug)]
pub enum ServiceConfigurationError {
    ErrorInConfiguration(String),
}
impl fmt::Display for ServiceConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceConfigurationError::ErrorInConfiguration(s) => {
                write!(f, "Configuration error in {}", s)
            }
        }
    }
}
impl Error for ServiceConfigurationError {}
