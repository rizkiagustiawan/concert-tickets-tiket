use std::time::Instant;
use colored::Colorize;
use tracing::info;
use chromiumoxide::page::Page;
use crate::browser::BrowserEngine;
use crate::error::Result;

pub struct SpeedReport {
    pub connection_latency_ms: f64,
    pub js_eval_latency_ms: f64,
    pub page_load_latency_ms: f64,
    pub click_latency_ms: f64,
}

impl SpeedReport {
    pub fn print(&self) {
        println!("\n{}", "╔══════════════════════════════════════╗".bright_cyan());
        println!("{}", "║     ⚡ SPEED BENCHMARK RESULTS ⚡     ║".bright_cyan());
        println!("{}", "╠══════════════════════════════════════╣".bright_cyan());
        println!("║ {} {:>12.2}ms ║",
            "Connection Latency :".bright_white(),
            self.connection_latency_ms);
        println!("║ {} {:>12.2}ms ║",
            "JS Eval Latency    :".bright_white(),
            self.js_eval_latency_ms);
        println!("║ {} {:>12.2}ms ║",
            "Page Load Latency  :".bright_white(),
            self.page_load_latency_ms);
        println!("║ {} {:>12.2}ms ║",
            "Click Latency      :".bright_white(),
            self.click_latency_ms);
        println!("{}", "╠══════════════════════════════════════╣".bright_cyan());
        let total = self.connection_latency_ms + self.js_eval_latency_ms;
        let rating = if total < 50.0 {
            "🏆 ULTRA FAST".bright_green()
        } else if total < 100.0 {
            "⚡ FAST".bright_yellow()
        } else if total < 200.0 {
            "🐢 MODERATE".yellow()
        } else {
            "💀 SLOW — use VPS!".bright_red()
        };
        println!("║ {} {:>20} ║", "Rating:".bright_white(), rating);
        println!("{}", "╚══════════════════════════════════════╝".bright_cyan());
    }
}

pub async fn run_benchmark(page: &Page) -> Result<SpeedReport> {
    info!("🏁 Running speed benchmark...");

    // 1. Connection latency — navigate to tiket.com
    let start = Instant::now();
    BrowserEngine::navigate(page, "https://www.tiket.com").await?;
    let connection_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Wait for page load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 2. JS eval latency
    let start = Instant::now();
    for _ in 0..10 {
        BrowserEngine::eval_js(page, "document.title").await?;
    }
    let js_eval_latency_ms = (start.elapsed().as_secs_f64() * 1000.0) / 10.0;

    // 3. Page load latency — navigate again
    let start = Instant::now();
    BrowserEngine::navigate(page, "https://www.tiket.com/to-do").await?;
    let page_load_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Wait for page load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 4. Click latency (click on a safe element)
    let start = Instant::now();
    let _ = BrowserEngine::eval_js(page, "(() => { const el = document.querySelector('body'); if(el) el.click(); return true; })()").await;
    let click_latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    let report = SpeedReport {
        connection_latency_ms,
        js_eval_latency_ms,
        page_load_latency_ms,
        click_latency_ms,
    };

    Ok(report)
}

/// Utility: measure and log execution time of an async operation
pub async fn timed<F, T>(label: &str, f: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    let start = Instant::now();
    let result = f.await;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    match &result {
        Ok(_) => info!("⏱️  {} completed in {:.2}ms", label, elapsed),
        Err(e) => info!("⏱️  {} failed in {:.2}ms: {}", label, elapsed, e),
    }
    result
}
