use serde::Deserialize;
use std::path::Path;
use crate::error::{AppError, Result};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub target: TargetConfig,
    pub buyer: BuyerConfig,
    pub payment: PaymentConfig,
    pub monitor: MonitorConfig,
    pub browser: BrowserConfig,
    pub telegram: TelegramConfig,
    pub speed: SpeedConfig,
    pub selectors: SelectorsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TargetConfig {
    pub url: String,
    pub ticket_category: String,
    pub quantity: u32,
    pub max_price: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BuyerConfig {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub id_number: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PaymentConfig {
    pub method: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitorConfig {
    pub poll_interval_ms: u64,
    pub start_time: String,
    pub pre_warm_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BrowserConfig {
    pub chrome_path: Option<String>,
    pub headless: bool,
    pub user_data_dir: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpeedConfig {
    pub parallel_tabs: u32,
    pub pre_connect: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SelectorsConfig {
    pub buy_button: String,
    pub ticket_category_selector: String,
    pub quantity_input: String,
    pub name_input: String,
    pub email_input: String,
    pub phone_input: String,
    pub id_input: String,
    pub payment_qris: String,
    pub payment_bank: String,
    pub submit_button: String,
    pub captcha_indicator: String,
    pub success_indicator: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| AppError::Config(format!("Cannot read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Invalid config TOML: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.target.url.is_empty() {
            return Err(AppError::Config("target.url cannot be empty".into()));
        }
        if !self.target.url.contains("tiket.com") {
            return Err(AppError::Config("target.url must be a tiket.com URL".into()));
        }
        if self.target.quantity == 0 {
            return Err(AppError::Config("target.quantity must be > 0".into()));
        }
        if self.buyer.name.is_empty() {
            return Err(AppError::Config("buyer.name cannot be empty".into()));
        }
        if self.buyer.email.is_empty() || !self.buyer.email.contains('@') {
            return Err(AppError::Config("buyer.email must be a valid email".into()));
        }
        if self.buyer.phone.is_empty() {
            return Err(AppError::Config("buyer.phone cannot be empty".into()));
        }
        if self.monitor.poll_interval_ms < 50 {
            return Err(AppError::Config("monitor.poll_interval_ms minimum is 50".into()));
        }
        if self.telegram.enabled {
            if self.telegram.bot_token.is_empty() || self.telegram.bot_token == "YOUR_BOT_TOKEN_HERE" {
                return Err(AppError::Config("telegram.bot_token must be set when telegram is enabled".into()));
            }
            if self.telegram.chat_id.is_empty() || self.telegram.chat_id == "YOUR_CHAT_ID_HERE" {
                return Err(AppError::Config("telegram.chat_id must be set when telegram is enabled".into()));
            }
        }
        let valid_methods = ["qris", "bank_transfer"];
        if !valid_methods.contains(&self.payment.method.as_str()) {
            return Err(AppError::Config(format!(
                "payment.method must be one of: {}",
                valid_methods.join(", ")
            )));
        }
        Ok(())
    }

    /// Pre-build JavaScript for autofill — called once at startup
    pub fn build_autofill_js(&self) -> String {
        format!(
            r#"(() => {{
    const set = (sel, val) => {{
        const el = document.querySelector(sel);
        if(el) {{
            const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
            nativeSetter.call(el, val);
            el.dispatchEvent(new Event('input', {{bubbles:true}}));
            el.dispatchEvent(new Event('change', {{bubbles:true}}));
            el.dispatchEvent(new Event('blur', {{bubbles:true}}));
        }}
    }};
    set('{name_sel}', '{name}');
    set('{email_sel}', '{email}');
    set('{phone_sel}', '{phone}');
    set('{id_sel}', '{id_number}');
}})();"#,
            name_sel = self.selectors.name_input.replace('\'', "\\'"),
            name = self.buyer.name.replace('\'', "\\'"),
            email_sel = self.selectors.email_input.replace('\'', "\\'"),
            email = self.buyer.email.replace('\'', "\\'"),
            phone_sel = self.selectors.phone_input.replace('\'', "\\'"),
            phone = self.buyer.phone.replace('\'', "\\'"),
            id_sel = self.selectors.id_input.replace('\'', "\\'"),
            id_number = self.buyer.id_number.replace('\'', "\\'"),
        )
    }
}
