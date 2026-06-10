use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;
#[derive(Parser)]
#[command(
    name = "kairos-vision",
    version = "1.0.0",
    about = "Real-time vision processing and object detection daemon"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/vision.toml")]
    config: PathBuf,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_vision::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();
    info!("kairos-vision v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_vision::AppState::new(cfg).await?;
    let wk = kairos_vision::worker::VisionWorker::new(state.config.clone());
    wk.start().await?;
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}
