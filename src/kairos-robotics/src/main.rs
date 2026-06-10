use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)] #[command(name="kairos-robotics", version="1.0.0", about="Robotic control loops and motor drivers daemon")]
struct Cli { #[arg(short, long, default_value="/etc/kairos/robotics.toml")] config: PathBuf }
#[tokio::main] async fn main() -> anyhow::Result<()> {
    let cli=Cli::parse(); let cfg=kairos_robotics::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-robotics v{} starting", env!("CARGO_PKG_VERSION"));
    let state=kairos_robotics::AppState::new(cfg).await?; let wk=kairos_robotics::worker::RoboticsWorker::new(state.config.clone()); wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown"); Ok(())
}
