use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;
#[derive(Parser)]
#[command(
    name = "kairos-orchestrator",
    version = "1.0.0",
    about = "Multi-agent task DAG scheduler"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/orchestrator.toml")]
    config: PathBuf,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_orchestrator::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();
    info!(
        "kairos-orchestrator v{} starting",
        env!("CARGO_PKG_VERSION")
    );
    let state = kairos_orchestrator::AppState::new(cfg).await?;
    let wk = kairos_orchestrator::worker::OrchestratorWorker::new(
        state.config.clone(),
        state.task_executor.clone(),
    );
    wk.start().await?;
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}
