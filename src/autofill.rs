use std::sync::Arc;
use std::time::Instant;
use chromiumoxide::page::Page;
use tracing::{info, warn};
use colored::Colorize;

use crate::config::Config;
use crate::browser::BrowserEngine;
use crate::error::{AppError, Result};

pub struct AutoFill {
    config: Arc<Config>,
    /// Pre-built JS string — constructed once at startup for zero runtime overhead
    fill_js: String,
}

impl AutoFill {
    pub fn new(config: Arc<Config>) -> Self {
        let fill_js = config.build_autofill_js();
        info!("📝 AutoFill JS pre-built ({} bytes)", fill_js.len());
        Self { config, fill_js }
    }

    /// Fill all buyer fields in one atomic JS eval — maximum speed
    pub async fn fill_buyer_info(&self, page: &Page) -> Result<()> {
        let start = Instant::now();
        BrowserEngine::eval_js(page, &self.fill_js).await?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("  {} {:.2}ms", "✏️  Buyer info filled in".bright_green(), elapsed);
        info!("✏️ Buyer info filled in {:.2}ms", elapsed);
        Ok(())
    }

    /// Select ticket category
    pub async fn select_ticket(&self, page: &Page) -> Result<()> {
        let start = Instant::now();
        let category = &self.config.target.ticket_category;
        let selector = &self.config.selectors.ticket_category_selector;

        // Try to find and click the matching category
        let js = format!(
            r#"(() => {{
                const containers = document.querySelectorAll('{sel}');
                for (const el of containers) {{
                    if (el.textContent.includes('{cat}')) {{
                        el.click();
                        return true;
                    }}
                }}
                // Fallback: click first available
                if (containers.length > 0) {{
                    containers[0].click();
                    return 'fallback';
                }}
                return false;
            }})()"
            "#,
            sel = selector.replace('\'', "\\'"),
            cat = category.replace('\'', "\\'")
        );

        let result = BrowserEngine::eval_js(page, &js).await?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        if result.contains("false") {
            warn!("⚠️ Could not find ticket category: {}", category);
            println!("  {} {}", "⚠️  Category not found:".bright_yellow(), category);
        } else if result.contains("fallback") {
            println!("  {} {:.2}ms (used fallback)", "🎫 Ticket selected in".bright_yellow(), elapsed);
        } else {
            println!("  {} {:.2}ms", "🎫 Ticket selected in".bright_green(), elapsed);
        }

        info!("🎫 Ticket selection completed in {:.2}ms: {}", elapsed, result);
        Ok(())
    }

    /// Set ticket quantity
    pub async fn set_quantity(&self, page: &Page) -> Result<()> {
        let start = Instant::now();
        let qty = self.config.target.quantity;
        let selector = &self.config.selectors.quantity_input;

        let js = format!(
            r#"(() => {{
                const el = document.querySelector('{sel}');
                if (el) {{
                    if (el.tagName === 'SELECT') {{
                        el.value = '{qty}';
                        el.dispatchEvent(new Event('change', {{bubbles:true}}));
                    }} else {{
                        const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
                        nativeSetter.call(el, '{qty}');
                        el.dispatchEvent(new Event('input', {{bubbles:true}}));
                        el.dispatchEvent(new Event('change', {{bubbles:true}}));
                    }}
                    return true;
                }}
                return false;
            }})()"
            "#,
            sel = selector.replace('\'', "\\'"),
            qty = qty
        );

        BrowserEngine::eval_js(page, &js).await?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("  {} {} in {:.2}ms", "🔢 Quantity set to".bright_green(), qty, elapsed);
        info!("🔢 Quantity set to {} in {:.2}ms", qty, elapsed);
        Ok(())
    }

    /// Select payment method (QRIS or Bank Transfer)
    pub async fn select_payment(&self, page: &Page) -> Result<()> {
        let start = Instant::now();
        let selector = match self.config.payment.method.as_str() {
            "qris" => &self.config.selectors.payment_qris,
            "bank_transfer" => &self.config.selectors.payment_bank,
            _ => return Err(AppError::Checkout(format!("Unknown payment method: {}", self.config.payment.method))),
        };

        let js = format!(
            r#"(() => {{
                const selectors = '{sel}'.split(', ');
                for (const s of selectors) {{
                    const el = document.querySelector(s.trim());
                    if (el) {{
                        el.click();
                        return true;
                    }}
                }}
                return false;
            }})()"
            "#,
            sel = selector.replace('\'', "\\'")
        );

        let result = BrowserEngine::eval_js(page, &js).await?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        if result.contains("false") {
            warn!("⚠️ Payment method selector not found");
            println!("  {} {}", "⚠️  Payment not found:".bright_yellow(), self.config.payment.method);
        } else {
            println!("  {} {} in {:.2}ms", "💳 Payment selected:".bright_green(), self.config.payment.method, elapsed);
        }

        info!("💳 Payment selection completed in {:.2}ms", elapsed);
        Ok(())
    }
}
