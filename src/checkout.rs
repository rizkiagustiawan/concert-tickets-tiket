use std::sync::Arc;
use std::time::Instant;
use chromiumoxide::page::Page;
use colored::Colorize;
use tokio::time::{sleep, Duration};

use crate::config::Config;
use crate::browser::BrowserEngine;
use crate::autofill::AutoFill;
use crate::notification::Notifier;
use crate::error::Result;

pub struct Checkout {
    config: Arc<Config>,
    autofill: AutoFill,
    notifier: Arc<Notifier>,
}

impl Checkout {
    pub fn new(config: Arc<Config>, notifier: Arc<Notifier>) -> Self {
        let autofill = AutoFill::new(config.clone());
        Self { config, autofill, notifier }
    }

    /// Execute the full checkout flow
    pub async fn execute(&self, page: &Page) -> Result<()> {
        let total_start = Instant::now();
        println!("\n{}", "╔══════════════════════════════════════╗".bright_cyan());
        println!("{}",   "║       ⚡ CHECKOUT SEQUENCE ⚡         ║".bright_cyan());
        println!("{}",   "╚══════════════════════════════════════╝".bright_cyan());

        // Step 1: Click buy button
        self.step_click_buy(page).await?;
        sleep(Duration::from_millis(300)).await;

        // Step 2: Select ticket category
        self.step_select_ticket(page).await?;
        sleep(Duration::from_millis(200)).await;

        // Step 3: Set quantity
        self.step_set_quantity(page).await?;
        sleep(Duration::from_millis(200)).await;

        // Step 4: Fill buyer information
        self.step_fill_buyer(page).await?;
        sleep(Duration::from_millis(200)).await;

        // Step 5: Check for CAPTCHA
        self.step_handle_captcha(page).await?;

        // Step 6: Click submit/proceed
        self.step_submit(page).await?;
        sleep(Duration::from_millis(500)).await;

        // Step 7: Select payment
        self.step_select_payment(page).await?;
        sleep(Duration::from_millis(300)).await;

        // Step 8: Final confirm
        self.step_confirm(page).await?;

        // Step 9: Check result
        let success = self.step_check_result(page).await?;

        let total_elapsed = total_start.elapsed().as_secs_f64() * 1000.0;

        if success {
            println!("\n{}", "╔══════════════════════════════════════╗".bright_green());
            println!("{}",   "║      🎉 CHECKOUT SUCCESSFUL! 🎉     ║".bright_green());
            println!("{}",   "╚══════════════════════════════════════╝".bright_green());
            println!("  {} {:.0}ms", "Total checkout time:".bright_white(), total_elapsed);
            self.notifier.send(&format!("🎉 CHECKOUT SUCCESSFUL! Total: {:.0}ms", total_elapsed)).await;
        } else {
            println!("\n{}", "╔══════════════════════════════════════╗".bright_red());
            println!("{}",   "║      ❌ CHECKOUT MAY HAVE FAILED     ║".bright_red());
            println!("{}",   "╚══════════════════════════════════════╝".bright_red());
            println!("  {}", "Check browser window for details".bright_yellow());
            self.notifier.send("❌ Checkout may have failed — check browser!").await;
        }

        Ok(())
    }

    async fn step_click_buy(&self, page: &Page) -> Result<()> {
        let start = Instant::now();
        println!("\n  {} Clicking buy button...", "[1/8]".bright_cyan());

        let selector = &self.config.selectors.buy_button;
        BrowserEngine::click(page, selector).await?;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("  {} {:.2}ms", "✅ Buy clicked in".bright_green(), elapsed);
        Ok(())
    }

    async fn step_select_ticket(&self, page: &Page) -> Result<()> {
        println!("\n  {} Selecting ticket category...", "[2/8]".bright_cyan());
        self.autofill.select_ticket(page).await
    }

    async fn step_set_quantity(&self, page: &Page) -> Result<()> {
        println!("\n  {} Setting quantity...", "[3/8]".bright_cyan());
        self.autofill.set_quantity(page).await
    }

    async fn step_fill_buyer(&self, page: &Page) -> Result<()> {
        println!("\n  {} Filling buyer information...", "[4/8]".bright_cyan());
        self.autofill.fill_buyer_info(page).await
    }

    async fn step_handle_captcha(&self, page: &Page) -> Result<()> {
        println!("\n  {} Checking for CAPTCHA...", "[5/8]".bright_cyan());

        let captcha_selector = &self.config.selectors.captcha_indicator;
        let js = format!(
            r#"(() => {{
                const selectors = '{sel}'.split(', ');
                for (const s of selectors) {{
                    const el = document.querySelector(s.trim());
                    if (el) return true;
                }}
                return false;
            }})()"
            "#,
            sel = captcha_selector.replace('\'', "\\'")
        );

        let has_captcha = BrowserEngine::eval_js_bool(page, &js).await.unwrap_or(false);

        if has_captcha {
            println!("\n{}", "  ⚠️  CAPTCHA DETECTED!".bright_red().bold());
            println!("  {}", "→ Solve CAPTCHA manually in browser window".bright_yellow());
            println!("  {}", "→ Bot will continue automatically after solve".bright_yellow());
            self.notifier.send("⚠️ CAPTCHA DETECTED! Solve it NOW!").await;

            // Play terminal bell
            print!("\x07");

            // Wait for CAPTCHA to be solved (check every 500ms)
            loop {
                sleep(Duration::from_millis(500)).await;
                let still_captcha = BrowserEngine::eval_js_bool(page, &js).await.unwrap_or(false);
                if !still_captcha {
                    println!("  {}", "✅ CAPTCHA solved! Continuing...".bright_green());
                    break;
                }
            }
        } else {
            println!("  {}", "✅ No CAPTCHA — full speed ahead!".bright_green());
        }

        Ok(())
    }

    async fn step_submit(&self, page: &Page) -> Result<()> {
        let start = Instant::now();
        println!("\n  {} Submitting form...", "[6/8]".bright_cyan());

        let selector = &self.config.selectors.submit_button;
        BrowserEngine::click(page, selector).await?;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("  {} {:.2}ms", "✅ Form submitted in".bright_green(), elapsed);
        Ok(())
    }

    async fn step_select_payment(&self, page: &Page) -> Result<()> {
        println!("\n  {} Selecting payment method...", "[7/8]".bright_cyan());
        self.autofill.select_payment(page).await
    }

    async fn step_confirm(&self, page: &Page) -> Result<()> {
        let start = Instant::now();
        println!("\n  {} Final confirmation...", "[8/8]".bright_cyan());

        // Look for any final confirm/pay button
        let js = r#"(() => {
            const btns = document.querySelectorAll('button, a');
            for (const btn of btns) {
                const text = btn.textContent.toLowerCase();
                if (text.includes('bayar') || text.includes('confirm') || text.includes('pay') || text.includes('proceed')) {
                    btn.click();
                    return true;
                }
            }
            return false;
        })()"#;

        BrowserEngine::eval_js(page, js).await?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("  {} {:.2}ms", "✅ Confirmed in".bright_green(), elapsed);
        Ok(())
    }

    async fn step_check_result(&self, page: &Page) -> Result<bool> {
        // Wait a bit for page to load
        sleep(Duration::from_secs(3)).await;

        let success_selector = &self.config.selectors.success_indicator;
        let js = format!(
            r#"(() => {{
                const selectors = '{sel}'.split(', ');
                for (const s of selectors) {{
                    const el = document.querySelector(s.trim());
                    if (el) return true;
                }}
                // Also check page content
                const body = document.body.innerText.toLowerCase();
                return body.includes('berhasil') || body.includes('success') || body.includes('terima kasih');
            }})()"
            "#,
            sel = success_selector.replace('\'', "\\'")
        );

        let success = BrowserEngine::eval_js_bool(page, &js).await.unwrap_or(false);
        Ok(success)
    }
}
