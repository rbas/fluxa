use log::{debug, info};
use reqwest;
use serde_json;
use std::sync::Arc;

use crate::error::NotificationError;
use crate::settings::FluxaConfig;

pub struct NotificationManager {
    providers: Vec<Arc<dyn NotificationProvider>>,
}

impl std::fmt::Debug for NotificationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NotificationManager {{ providers: {} }}",
            self.providers.len()
        )
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    /// Create new empty notification manager
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn from_config(config: &FluxaConfig, http_client: Arc<reqwest::Client>) -> Self {
        let mut manager = Self::new();

        // Console provider is always enabled (fallback/debugging)
        info!("🖥️  Adding Console notification provider (always enabled)");
        manager.add_provider(Arc::new(ConsoleProvider::new()));

        // Pushover provider (optional - only if API keys are provided and not empty)
        if !config.pushover_api_key.is_empty() && !config.pushover_user_key.is_empty() {
            info!("📱 Adding Pushover notification provider");
            let pushover_provider = PushoverProvider::new(
                config.pushover_api_key.clone(),
                config.pushover_user_key.clone(),
                http_client.clone(),
            );
            manager.add_provider(Arc::new(pushover_provider));
        } else {
            info!("📱 Pushover keys not provided - skipping Pushover notifications (optional)");
        }

        // Telegram provider (optional - only if config section exists)
        if let Some(telegram_config) = &config.telegram {
            info!(
                "🚀 Adding Telegram notification provider (chat_id: {})",
                telegram_config.chat_id
            );
            let telegram_provider = TelegramProvider::new(
                telegram_config.bot_token.clone(),
                telegram_config.chat_id.clone(),
                http_client.clone(),
            );
            manager.add_provider(Arc::new(telegram_provider));
        } else {
            info!("📱 Telegram config not found - skipping Telegram notifications (optional)");
        }

        manager
    }

    /// Send notification to all configured providers
    pub async fn send_notification(&self, message: &str) -> Result<(), NotificationError> {
        if self.providers.is_empty() {
            debug!("No notification providers configured, skipping notification");
            return Ok(());
        }

        debug!(
            "Sending notification to {} providers: '{}'",
            self.providers.len(),
            message
        );

        let mut errors = Vec::new();

        for provider in &self.providers {
            match provider.send_notification(message).await {
                Ok(_) => {
                    debug!("✅ Notification sent via {}", provider.provider_name());
                }
                Err(e) => {
                    debug!("❌ Failed to send via {}: {}", provider.provider_name(), e);
                    errors.push(format!("{}: {}", provider.provider_name(), e));
                }
            }
        }

        if errors.len() == self.providers.len() {
            return Err(NotificationError::SendFailed {
                message: format!("All providers failed: {}", errors.join(", ")),
            });
        }

        Ok(())
    }

    fn add_provider(&mut self, provider: Arc<dyn NotificationProvider>) {
        debug!("Adding notification provider: {}", provider.provider_name());
        self.providers.push(provider);
    }
}

#[async_trait::async_trait]
pub trait NotificationProvider: Send + Sync {
    async fn send_notification(&self, message: &str) -> Result<(), NotificationError>;
    fn provider_name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct PushoverProvider {
    api_key: String,
    user_key: String,
    http_client: Arc<reqwest::Client>,
}

impl PushoverProvider {
    pub fn new(api_key: String, user_key: String, http_client: Arc<reqwest::Client>) -> Self {
        Self {
            api_key,
            user_key,
            http_client,
        }
    }
}

#[async_trait::async_trait]
impl NotificationProvider for PushoverProvider {
    async fn send_notification(&self, message: &str) -> Result<(), NotificationError> {
        let params = serde_json::json!({
            "token": self.api_key,
            "user": self.user_key,
            "message": message
        });

        let response = self
            .http_client
            .post("https://api.pushover.net/1/messages.json")
            .json(&params)
            .send()
            .await
            .map_err(NotificationError::HttpRequest)?;

        if response.status().is_success() {
            debug!("Pushover notification sent successfully!");
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .map_err(NotificationError::HttpRequest)?;
            Err(NotificationError::SendFailed {
                message: format!("Pushover API error: {}", error_text),
            })
        }
    }

    fn provider_name(&self) -> &'static str {
        "Pushover"
    }
}

#[derive(Debug)]
pub struct ConsoleProvider;

impl Default for ConsoleProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl NotificationProvider for ConsoleProvider {
    async fn send_notification(&self, message: &str) -> Result<(), NotificationError> {
        println!("🔔 [CONSOLE NOTIFICATION]: {}", message);
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "Console"
    }
}

#[derive(Debug)]
pub struct TelegramProvider {
    bot_token: String,
    chat_id: String,
    http_client: Arc<reqwest::Client>,
}

impl TelegramProvider {
    pub fn new(bot_token: String, chat_id: String, http_client: Arc<reqwest::Client>) -> Self {
        Self {
            bot_token,
            chat_id,
            http_client,
        }
    }

    fn format_message(&self, message: &str) -> String {
        // Rich HTML formatting for Telegram
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

        if message.contains("unhealthy") || message.contains("down") {
            format!(
                "🚨 <b>Service Alert</b>\n\n\
                 📋 <b>Message:</b> {}\n\
                 ⏰ <b>Time:</b> {}\n\
                 🔧 <i>Fluxa Monitor v{}</i>",
                message,
                timestamp,
                env!("CARGO_PKG_VERSION")
            )
        } else {
            format!(
                "✅ <b>Service Recovery</b>\n\n\
                 📋 <b>Message:</b> {}\n\
                 ⏰ <b>Time:</b> {}\n\
                 🔧 <i>Fluxa Monitor v{}</i>",
                message,
                timestamp,
                env!("CARGO_PKG_VERSION")
            )
        }
    }
}

#[async_trait::async_trait]
impl NotificationProvider for TelegramProvider {
    async fn send_notification(&self, message: &str) -> Result<(), NotificationError> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let formatted_message = self.format_message(message);

        let payload = serde_json::json!({
            "chat_id": self.chat_id,
            "text": formatted_message,
            "parse_mode": "HTML"
        });

        debug!("Sending Telegram notification to chat {}", self.chat_id);

        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::SendFailed {
                message: format!("HTTP request failed: {}", e),
            })?;

        if response.status().is_success() {
            debug!("Telegram notification sent successfully!");
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(NotificationError::SendFailed {
                message: format!("Telegram API error: {}", error_text),
            })
        }
    }

    fn provider_name(&self) -> &'static str {
        "Telegram"
    }
}
