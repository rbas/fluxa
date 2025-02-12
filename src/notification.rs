use log::debug;
use reqwest::{self, Client};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct Notifier {
    api_key: String,
    user_key: String,
}

impl Notifier {
    pub fn new(api_key: String, user_key: String) -> Self {
        Self { api_key, user_key }
    }

    pub async fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        pushover_notification(&self.api_key, &self.user_key, message).await
    }
}

pub async fn pushover_notification(
    api_key: &str,
    user_key: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to send notification: {}", response.text().await?),
        )))
    }
}
