use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)]
#[command(name = "kairos-fb", version = "1.0.0", about = "Framebuffer canvas and DRM/KMS engine daemon")]
struct Cli { #[arg(short, long, default_value = "/etc/kairos/fb.toml")] config: PathBuf }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse(); let cfg = kairos_fb::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-fb v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_fb::AppState::new(cfg).await?;
    let wk = kairos_fb::worker::FbWorker::new(state.config.clone());
    wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown signal received"); Ok(())
}
