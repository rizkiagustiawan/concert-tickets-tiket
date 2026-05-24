use std::sync::Arc;
use std::time::Instant;
use chrono::{DateTime, Utc};
use chromiumoxide::page::Page;
use colored::Colorize;
use tracing::{info, warn};
use tokio::time::{sleep, Duration};

use crate::config::Config;
use crate::browser::BrowserEngine;
use crate::notification::Notifier;
use crate::error::{AppError, Result};

pub struct Monitor {
    config: Arc<Config>,
    notifier: Arc<Notifier>,
}

impl Monitor {
    pub fn new(config: Arc<Config>, notifier: Arc<Notifier>) -> Self {
        Self { config, notifier }
    }

    /// Wait until start_time, showing countdown
    pub async fn wait_for_start(&self) -> Result<()> {
        let start_time = DateTime::parse_from_rfc3339(&self.config.monitor.start_time)
            .map_err(|e| AppError::Monitor(format!("Invalid start_time format: {}", e)))?;

        loop {
            let now = Utc::now().with_timezone(&start_time.timezone());
            let diff = start_time.signed_duration_since(now);

            if diff.num_seconds() <= 0 {
                println!("\n{}", "🚀 WAR TIME! GO GO GO!".bright_red().bold());
                self.notifier.send("🚀 WAR TIME STARTED!").await;
                break;
            }

            let hours = diff.num_hours();
            let mins = diff.num_minutes() % 60;
            let secs = diff.num_seconds() % 60;

            print!("\r{} {:02}:{:02}:{:02} ",
                "⏳ War starts in:".bright_yellow(),
                hours, mins, secs
            );
            use std::io::Write;
            std::io::stdout().flush().ok();

            // Pre-warm phase
            if diff.num_seconds() <= self.config.monitor.pre_warm_seconds as i64 {
                if diff.num_seconds() == self.config.monitor.pre_warm_seconds as i64 {
                    println!("\n{}", "🔥 PRE-WARM PHASE — connections ready!".bright_cyan());
                }
            }

            sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }

    /// Monitor page for ticket availability — returns when tickets detected
    pub async fn watch(&self, page: &Page) -> Result<()> {
        info!("👁️ Monitoring for ticket availability...");
        println!("{}", "👁️  Monitoring... waiting for tickets to appear".bright_cyan());

        let poll_interval = Duration::from_millis(self.config.monitor.poll_interval_ms);
        let buy_selector = &self.config.selectors.buy_button;

        let check_js = format!(
            r#"(() => {{
                const selectors = '{sel}'.split(', ');
                for (const sel of selectors) {{
                    const el = document.querySelector(sel.trim());
                    if (el && !el.disabled && el.offsetParent !== null) {{
                        return true;
                    }}
                }}
                return false;
            }})()"
            "#,
            sel = buy_selector.replace('\'', "\\'")
        );

        let mut attempt = 0u64;
        let start = Instant::now();

        loop {
            attempt += 1;

            match BrowserEngine::eval_js_bool(page, &check_js).await {
                Ok(true) => {
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    println!("\n{} (attempt #{}, {:.0}ms)",
                        "🎫 TICKET DETECTED!".bright_green().bold(),
                        attempt, elapsed
                    );
                    info!("🎫 Ticket detected after {} attempts ({:.0}ms)", attempt, elapsed);
                    self.notifier.send("🎫 TICKET DETECTED! Auto-checkout starting...").await;
                    return Ok(());
                }
                Ok(false) => {
                    // Not available yet — continue polling
                    if attempt % 100 == 0 {
                        print!("\r{} polls: {} | elapsed: {:.1}s ",
                            "👁️ ".bright_cyan(),
                            attempt.to_string().bright_white(),
                            start.elapsed().as_secs_f64()
                        );
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                }
                Err(e) => {
                    warn!("Monitor poll error: {} — retrying", e);
                    // Page might need refresh
                    if attempt % 50 == 0 {
                        info!("🔄 Refreshing page...");
                        let _ = BrowserEngine::eval_js(page, "location.reload()").await;
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            sleep(poll_interval).await;
        }
    }

    /// Auto-refresh page at specified interval
    pub async fn auto_refresh(page: &Page, interval_secs: u64) {
        loop {
            sleep(Duration::from_secs(interval_secs)).await;
            let _ = BrowserEngine::eval_js(page, "location.reload()").await;
        }
    }
}
