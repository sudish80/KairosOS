use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)]
#[command(name="kairos-avionics", version="1.0.0", about="Avionics bus protocols and telemetry standards daemon")]
struct Cli { #[arg(short, long, default_value="/etc/kairos/avionics.toml")] config: PathBuf }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli=Cli::parse(); let cfg=kairos_avionics::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-avionics v{} starting", env!("CARGO_PKG_VERSION"));
    let state=kairos_avionics::AppState::new(cfg).await?;
    let wk=kairos_avionics::worker::AvionicsWorker::new(state.config.clone()); wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown"); Ok(())
}
