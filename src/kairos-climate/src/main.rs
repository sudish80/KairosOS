use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)]
#[command(name = "kairos-climate", version = "1.0.0", about = "Climate model data assimilation engine daemon")]
struct Cli { #[arg(short, long, default_value = "/etc/kairos/climate.toml")] config: PathBuf }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse(); let cfg = kairos_climate::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-climate v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_climate::AppState::new(cfg).await?;
    let wk = kairos_climate::worker::ClimateWorker::new(state.config.clone()); wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown signal received"); Ok(())
}
