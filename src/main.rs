mod config;
mod error;
mod browser;
mod monitor;
mod autofill;
mod checkout;
mod notification;
mod speed;

use std::sync::Arc;
use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::browser::BrowserEngine;
use crate::monitor::Monitor;
use crate::checkout::Checkout;
use crate::notification::Notifier;
use crate::error::Result;

#[derive(Parser)]
#[command(name = "tiket-war-bot")]
#[command(about = "⚡ Ultra-fast ticket war bot for tiket.com concerts")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// 🔥 Start war mode — monitor & auto-checkout
    War,
    /// 🏁 Run speed benchmark against tiket.com
    Bench,
    /// ✅ Validate config file
    Config,
    /// 🔥 Pre-login & warm up connections
    Warmup,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with_target(false)
        .init();

    print_banner();

    let cli = Cli::parse();

    match cli.command {
        Commands::Config => cmd_config(&cli.config).await?,
        Commands::Bench => cmd_bench(&cli.config).await?,
        Commands::Warmup => cmd_warmup(&cli.config).await?,
        Commands::War => cmd_war(&cli.config).await?,
    }

    Ok(())
}

fn print_banner() {
    println!("{}", r#"
 ████████╗██╗██╗  ██╗███████╗████████╗    ██╗    ██╗ █████╗ ██████╗
 ╚══██╔══╝██║██║ ██╔╝██╔════╝╚══██╔══╝    ██║    ██║██╔══██╗██╔══██╗
    ██║   ██║█████╔╝ █████╗     ██║       ██║ █╗ ██║███████║██████╔╝
    ██║   ██║██╔═██╗ ██╔══╝     ██║       ██║███╗██║██╔══██║██╔══██╗
    ██║   ██║██║  ██╗███████╗   ██║       ╚███╔███╔╝██║  ██║██║  ██║
    ╚═╝   ╚═╝╚═╝  ╚═╝╚══════╝   ╚═╝        ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝
    "#.bright_cyan());
    println!("    {}", "⚡ Ultra-Fast Ticket War Bot — Rust Edition ⚡".bright_yellow().bold());
    println!("    {}\n", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
}

async fn cmd_config(config_path: &str) -> Result<()> {
    println!("{} {}\n", "📄 Validating config:".bright_cyan(), config_path);

    match Config::load(config_path) {
        Ok(config) => {
            println!("{}", "✅ Config is valid!".bright_green());
            println!("\n  {} {}", "Target:".bright_white(), config.target.url);
            println!("  {} {}", "Category:".bright_white(), config.target.ticket_category);
            println!("  {} {}", "Quantity:".bright_white(), config.target.quantity);
            println!("  {} {}", "Buyer:".bright_white(), config.buyer.name);
            println!("  {} {}", "Payment:".bright_white(), config.payment.method);
            println!("  {} {}", "Telegram:".bright_white(), if config.telegram.enabled { "ON" } else { "OFF" });
            println!("  {} {}", "Poll interval:".bright_white(), format!("{}ms", config.monitor.poll_interval_ms));
            println!("  {} {}", "Parallel tabs:".bright_white(), config.speed.parallel_tabs);
            Ok(())
        }
        Err(e) => {
            println!("{} {}", "❌ Config error:".bright_red(), e);
            Err(e)
        }
    }
}

async fn cmd_bench(config_path: &str) -> Result<()> {
    let config = Arc::new(Config::load(config_path)?);
    let engine = BrowserEngine::launch(config).await?;
    let page = engine.new_page().await?;

    let report = speed::run_benchmark(&page).await?;
    report.print();

    engine.close().await?;
    Ok(())
}

async fn cmd_warmup(config_path: &str) -> Result<()> {
    let config = Arc::new(Config::load(config_path)?);
    let engine = BrowserEngine::launch(config).await?;
    let page = engine.new_page().await?;

    engine.warm_up(&page).await?;

    println!("\n{}", "🔥 Warm-up complete!".bright_green());
    println!("{}", "   Browser is open — log in to tiket.com now".bright_yellow());
    println!("{}", "   Session will be saved for war mode".bright_yellow());
    println!("\n{}", "   Press Ctrl+C when done logging in".bright_white());

    // Keep running until user presses Ctrl+C
    tokio::signal::ctrl_c().await.ok();
    println!("\n{}", "👋 Session saved. Ready for war!".bright_green());

    engine.close().await?;
    Ok(())
}

async fn cmd_war(config_path: &str) -> Result<()> {
    let config = Arc::new(Config::load(config_path)?);

    println!("{}", "⚔️  WAR MODE ACTIVATED".bright_red().bold());
    println!("  {} {}", "Target:".bright_white(), config.target.url);
    println!("  {} {}", "Category:".bright_white(), config.target.ticket_category);
    println!("  {} x{}", "Quantity:".bright_white(), config.target.quantity);
    println!("  {} {}", "Payment:".bright_white(), config.payment.method);
    println!("  {} {}", "War time:".bright_white(), config.monitor.start_time);
    println!();

    // Initialize notifier
    let notifier = Arc::new(Notifier::new(&config.telegram));
    notifier.send("⚔️ Tiket War Bot started! Preparing for battle...").await;

    // Launch browser
    let engine = BrowserEngine::launch(config.clone()).await?;
    let page = engine.new_page().await?;

    // Warm up
    if config.speed.pre_connect {
        engine.warm_up(&page).await?;
    }

    // Navigate to target
    engine.navigate_to_target(&page).await?;
    println!("\n{}", "✅ On target page. Waiting for war time...".bright_green());

    // Wait for start time
    let monitor = Monitor::new(config.clone(), notifier.clone());
    monitor.wait_for_start().await?;

    // Refresh page at war time
    BrowserEngine::eval_js(&page, "location.reload()").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Monitor for ticket availability
    monitor.watch(&page).await?;

    // Execute checkout
    let checkout = Checkout::new(config.clone(), notifier.clone());
    checkout.execute(&page).await?;

    // Keep browser open for user to verify
    println!("\n{}", "🏁 War complete! Browser staying open for verification.".bright_cyan());
    println!("{}", "   Press Ctrl+C to exit.".bright_white());
    tokio::signal::ctrl_c().await.ok();

    engine.close().await?;
    Ok(())
}
