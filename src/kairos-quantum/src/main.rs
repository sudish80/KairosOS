use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;
#[derive(Parser)]
#[command(
    name = "kairos-quantum",
    version = "1.0.0",
    about = "Quantum gate emulation and simulator daemon"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/quantum.toml")]
    config: PathBuf,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_quantum::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();
    info!("kairos-quantum v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_quantum::AppState::new(cfg).await?;
    let wk = kairos_quantum::worker::QuantumWorker::new(state.config.clone());
    wk.start().await?;
    tokio::signal::ctrl_c().await?;
    info!("Shutdown");
    Ok(())
}
