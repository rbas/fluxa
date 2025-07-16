use log::debug;
use reqwest::{self, Client};
use serde_json::json;

use crate::error::NotificationError;

#[derive(Debug, Clone)]
pub struct Notifier {
    api_key: String,
    user_key: String,
}

impl Notifier {
    pub fn new(api_key: String, user_key: String) -> Self {
        Self { api_key, user_key }
    }

    pub async fn send(&self, message: &str) -> Result<(), NotificationError> {
        pushover_notification(&self.api_key, &self.user_key, message).await
    }
}

pub async fn pushover_notification(
    api_key: &str,
    user_key: &str,
    message: &str,
) -> Result<(), NotificationError> {
    let client = Client::new();

    let params = json!({
        "token": api_key,
        "user": user_key,
        "message": message
    });

    let response = client
        .post("https://api.pushover.net/1/messages.json")
        .json(&params)
        .send()
        .await?;

    if response.status().is_success() {
        debug!("Notification sent successfully!");
        Ok(())
    } else {
        let error_text = response.text().await?;
        Err(NotificationError::SendFailed {
            message: error_text,
        })
    }
}
