use std::sync::Arc;
use chromiumoxide::{
    Browser, BrowserConfig,
    page::Page,
};
use futures::StreamExt;
use tracing::{info, error};
use crate::config::Config;
use crate::error::{AppError, Result};

pub struct BrowserEngine {
    browser: Browser,
    config: Arc<Config>,
}

impl BrowserEngine {
    pub async fn launch(config: Arc<Config>) -> Result<Self> {
        info!("🚀 Launching Chrome...");

        let mut builder = BrowserConfig::builder()
            .disable_default_args()
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-extensions")
            .arg("--disable-background-networking")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-breakpad")
            .arg("--disable-component-update")
            .arg("--disable-default-apps")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-hang-monitor")
            .arg("--disable-ipc-flooding-protection")
            .arg("--disable-popup-blocking")
            .arg("--disable-prompt-on-repost")
            .arg("--disable-renderer-backgrounding")
            .arg("--disable-sync")
            .arg("--disable-translate")
            .arg("--metrics-recording-only")
            .arg("--no-first-run")
            .arg("--safebrowsing-disable-auto-update")
            .arg("--password-store=basic")
            .arg("--use-mock-keychain")
            .arg("--enable-features=NetworkService,NetworkServiceInProcess")
            .arg("--disable-features=TranslateUI");

        if config.browser.headless {
            builder = builder.arg("--headless=new");
        }

        if let Some(ref chrome_path) = config.browser.chrome_path {
            if !chrome_path.is_empty() {
                builder = builder.chrome_executable(chrome_path);
            }
        }

        if let Some(ref user_data_dir) = config.browser.user_data_dir {
            if !user_data_dir.is_empty() {
                builder = builder.arg(format!("--user-data-dir={}", user_data_dir));
            }
        }

        let browser_config = builder
            .build()
            .map_err(|e| AppError::Browser(format!("Failed to build browser config: {:?}", e)))?;

        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| AppError::Browser(format!("Failed to launch Chrome: {}", e)))?;

        // Spawn the handler in background
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    error!("Browser handler error: {:?}", h);
                    break;
                }
            }
        });

        info!("✅ Chrome launched successfully");

        Ok(Self { browser, config })
    }

    /// Create a new page/tab
    pub async fn new_page(&self) -> Result<Page> {
        let page = self.browser
            .new_page("about:blank")
            .await
            .map_err(|e| AppError::Browser(format!("Failed to create page: {}", e)))?;
        Ok(page)
    }

    /// Navigate a page to URL
    pub async fn navigate(page: &Page, url: &str) -> Result<()> {
        page.goto(url)
            .await
            .map_err(|e| AppError::Browser(format!("Failed to navigate to {}: {}", url, e)))?;
        info!("📄 Navigated to {}", url);
        Ok(())
    }

    /// Execute JavaScript on a page and return result as string
    pub async fn eval_js(page: &Page, js: &str) -> Result<String> {
        let result = page
            .evaluate(js)
            .await
            .map_err(|e| AppError::Browser(format!("JS eval failed: {}", e)))?;
        let value: serde_json::Value = result.into_value()
            .map_err(|e| AppError::Browser(format!("JS result parse failed: {:?}", e)))?;
        Ok(value.to_string())
    }

    /// Execute JavaScript returning bool
    pub async fn eval_js_bool(page: &Page, js: &str) -> Result<bool> {
        let result = page
            .evaluate(js)
            .await
            .map_err(|e| AppError::Browser(format!("JS eval failed: {}", e)))?;
        let value: bool = result.into_value()
            .unwrap_or(false);
        Ok(value)
    }

    /// Click an element by CSS selector
    pub async fn click(page: &Page, selector: &str) -> Result<()> {
        let js = format!(
            r#"(() => {{ const el = document.querySelector('{}'); if(el) {{ el.click(); return true; }} return false; }})()"#,
            selector.replace('\'', "\\'")
        );
        Self::eval_js(page, &js).await?;
        Ok(())
    }

    /// Warm up connection by pre-navigating to tiket.com
    pub async fn warm_up(&self, page: &Page) -> Result<()> {
        info!("🔥 Warming up connection to tiket.com...");
        Self::navigate(page, "https://www.tiket.com").await?;
        // Wait for page to load
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        info!("✅ Connection warmed up");
        Ok(())
    }

    /// Pre-navigate to target URL
    pub async fn navigate_to_target(&self, page: &Page) -> Result<()> {
        info!("🎯 Navigating to target: {}", self.config.target.url);
        Self::navigate(page, &self.config.target.url).await?;
        Ok(())
    }

    /// Get config reference
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Close browser
    pub async fn close(self) -> Result<()> {
        info!("🔒 Closing browser...");
        // Browser drops and closes automatically
        drop(self.browser);
        Ok(())
    }
}
