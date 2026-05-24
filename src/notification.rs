use tracing::{info, warn};
use crate::config::TelegramConfig;

pub struct Notifier {
    enabled: bool,
    client: reqwest::Client,
    bot_token: String,
    chat_id: String,
}

impl Notifier {
    pub fn new(config: &TelegramConfig) -> Self {
        Self {
            enabled: config.enabled,
            client: reqwest::Client::new(),
            bot_token: config.bot_token.clone(),
            chat_id: config.chat_id.clone(),
        }
    }

    /// Send a notification message. Non-blocking, never fails the caller.
    pub async fn send(&self, message: &str) {
        if !self.enabled {
            return;
        }

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let payload = serde_json::json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML"
        });

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("📨 Telegram notification sent: {}", message);
                } else {
                    warn!("📨 Telegram API error: {}", resp.status());
                }
            }
            Err(e) => {
                warn!("📨 Telegram send failed: {}", e);
            }
        }
    }

    /// Send notification with retry
    pub async fn send_with_retry(&self, message: &str, retries: u32) {
        for attempt in 0..=retries {
            if !self.enabled { return; }

            let url = format!(
                "https://api.telegram.org/bot{}/sendMessage",
                self.bot_token
            );

            let payload = serde_json::json!({
                "chat_id": self.chat_id,
                "text": message,
                "parse_mode": "HTML"
            });

            match self.client.post(&url).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    info!("📨 Telegram sent (attempt {})", attempt + 1);
                    return;
                }
                Ok(resp) => {
                    warn!("📨 Telegram error (attempt {}): {}", attempt + 1, resp.status());
                }
                Err(e) => {
                    warn!("📨 Telegram failed (attempt {}): {}", attempt + 1, e);
                }
            }

            if attempt < retries {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}
