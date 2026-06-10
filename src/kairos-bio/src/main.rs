use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)] #[command(name="kairos-bio", version="1.0.0", about="DNA/RNA sequence analysis and genomics pipeline daemon")]
struct Cli { #[arg(short, long, default_value="/etc/kairos/bio.toml")] config: PathBuf, #[arg(long)] gc: Option<String> }
#[tokio::main] async fn main() -> anyhow::Result<()> {
    let cli=Cli::parse(); let cfg=kairos_bio::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-bio v{} starting", env!("CARGO_PKG_VERSION"));
    let state=kairos_bio::AppState::new(cfg).await?;
    if let Some(seq) = cli.gc { let gc = state.seq_engine.gc_content(&seq); println!("GC content: {:.2}%", gc * 100.0); return Ok(()); }
    let wk=kairos_bio::worker::BioWorker::new(state.config.clone()); wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown"); Ok(())
}
