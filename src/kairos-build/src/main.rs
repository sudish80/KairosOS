use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)]
#[command(name = "kairos-build", version = "1.0.0", about = "Buildroot/Yocto compiler and image pipeline daemon")]
struct Cli { #[arg(short, long, default_value = "/etc/kairos/build.toml")] config: PathBuf, #[arg(long)] build: bool }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse(); let cfg = kairos_build::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-build v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_build::AppState::new(cfg).await?;
    if cli.build { state.builder.build().await?; let img = state.image_manager.package_image().await?; println!("Image: {}", img); return Ok(()); }
    let wk = kairos_build::worker::BuildWorker::new(state.config.clone()); wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown signal received"); Ok(())
}
