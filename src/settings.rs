use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Clone)]
pub struct FluxaConfig {
    pushover_api_key: String,
    pushover_user_key: String,
}

impl FluxaConfig {
    pub fn pushover_user_key(&self) -> &str {
        &self.pushover_user_key
    }

    pub fn pushover_api_key(&self) -> &str {
        &self.pushover_api_key
    }
}
