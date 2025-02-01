use log::debug;
use reqwest::{self, Client};
use serde_json::json;

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
