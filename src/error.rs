use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Browser error: {0}")]
    Browser(String),

    #[error("Monitor error: {0}")]
    Monitor(String),

    #[error("Checkout error: {0}")]
    Checkout(String),

    #[error("Notification error: {0}")]
    Notification(String),

    #[error("Speed benchmark error: {0}")]
    Speed(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    ChromeError(#[from] chromiumoxide::error::CdpError),
}

pub type Result<T> = std::result::Result<T, AppError>;
