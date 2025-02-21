use std::{error::Error, fmt, path::Path, str::FromStr};

use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

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

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct FluxaConfig {
    pub pushover_api_key: String,
    pub pushover_user_key: String,
    pub services: Vec<ServiceConfig>,
    pub fluxa: Fluxa,
}
impl FluxaConfig {
    pub fn new(path: &Path) -> Result<Self, ServiceConfigurationError> {
        let settings = Config::builder()
            .add_source(File::from(path))
            .build()
            .unwrap();

        Self::build(settings)
    }

    pub(super) fn build(settings: Config) -> Result<Self, ServiceConfigurationError> {
        let result: Result<FluxaConfig, ConfigError> = settings.try_deserialize();
        match result {
            Ok(config) => Ok(config),
            Err(err) => Err(ServiceConfigurationError::from(err)),
        }
    }
}

impl FromStr for FluxaConfig {
    type Err = ServiceConfigurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let settings = Config::builder()
            .add_source(File::from_str(s, FileFormat::Toml))
            .build()
            .unwrap();

        Self::build(settings)
    }
}

#[derive(Debug)]
pub enum ServiceConfigurationError {
    ErrorInConfiguration(String),
}
impl fmt::Display for ServiceConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceConfigurationError::ErrorInConfiguration(s) => {
                write!(f, "Configuration error {}", s)
            }
        }
    }
}
impl Error for ServiceConfigurationError {}

impl From<ConfigError> for ServiceConfigurationError {
    fn from(error: ConfigError) -> Self {
        Self::ErrorInConfiguration(error.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_from_config() {
        let fluxa_configuration = r#"
# Pushover API key
pushover_api_key = "api key"
# Pushover user or group key
pushover_user_key = "key"

[fluxa]
listen = "http://localhost:8080"

[[services]]
# Monitored url
url = "http://localhost:3000"
# How ofter the url will be monitored
interval_seconds = 300
# Determin how many times it will try before the url will be considered as down
max_retries = 3
# How many seconds retry has to wait before next try
retry_interval = 3
        "#;
        let result = fluxa_configuration.parse::<FluxaConfig>();

        match result {
            Ok(config) => {
                assert_eq!(config.services.len(), 1);
                assert_eq!(config.fluxa.listen, "http://localhost:8080");
            }
            Err(e) => {
                panic!("Deserialization failed with error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_configuration_error() {
        let fluxa_configuration = "";
        let result = FluxaConfig::from_str(fluxa_configuration);

        assert!(result.is_err());
    }
}
