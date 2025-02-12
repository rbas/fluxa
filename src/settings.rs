use std::{error::Error, fmt};

use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct FluxaConfig {
    pub pushover_api_key: String,
    pub pushover_user_key: String,
    pub services: Vec<ServiceConfig>,
    pub fluxa: Fluxa,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct ServiceConfig {
    pub url: String,
    pub interval_seconds: u64,
    pub max_retries: usize,
    pub retry_interval: u64,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct Fluxa {
    pub listen: String,
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
