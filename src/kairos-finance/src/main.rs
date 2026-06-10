use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;
#[derive(Parser)]
#[command(
    name = "kairos-finance",
    version = "1.0.0",
    about = "Market data feed handler and algo trading daemon"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/finance.toml")]
    config: PathBuf,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_finance::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();
    info!("kairos-finance v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_finance::AppState::new(cfg).await?;
    let wk = kairos_finance::worker::FinanceWorker::new(state.config.clone());
    wk.start().await?;
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}
