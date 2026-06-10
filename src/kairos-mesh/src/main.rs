use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)]
#[command(name = "kairos-mesh", version = "1.0.0", about = "WireGuard mesh networking and consensus daemon")]
struct Cli { #[arg(short, long, default_value = "/etc/kairos/mesh.toml")] config: PathBuf, #[arg(long)] up: bool, #[arg(long)] down: bool }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse(); let cfg = kairos_mesh::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-mesh v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_mesh::AppState::new(cfg).await?;
    if cli.up { state.wg_manager.bring_up().await?; return Ok(()); }
    if cli.down { state.wg_manager.bring_down().await?; return Ok(()); }
    state.wg_manager.bring_up().await?;
    state.discovery.start_discovery().await?;
    let wk = kairos_mesh::worker::MeshWorker::new(state.config.clone());
    wk.start().await?;
    tokio::signal::ctrl_c().await?; state.wg_manager.bring_down().await?; info!("Shutdown complete"); Ok(())
}
