use clap::Parser;
use tracing::info;
use faber_aeternus::cli::Cli;
use faber_aeternus::tui::run_tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Optionally setup file-based tracing here so iocraft isn't interrupted
    // tracing_subscriber::fmt().with_env_filter("info").init();
    
    // Fallback info print before raw mode
    println!("🔥 faber-aeternus v0.1.0 — the eternal craftsman");
    
    run_tui(cli).await?;
    Ok(())
}