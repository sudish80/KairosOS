use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)]
#[command(name = "kairos-db", version = "1.0.0", about = "SQLite vector database and memory bus daemon")]
struct Cli { #[arg(short, long, default_value = "/etc/kairos/db.toml")] config: PathBuf }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse(); let cfg = kairos_db::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-db v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_db::AppState::new(cfg).await?;
    let wk = kairos_db::worker::DbWorker::new(state.config.clone());
    wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown signal received"); Ok(())
}
