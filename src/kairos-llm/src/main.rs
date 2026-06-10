use std::path::PathBuf; use clap::Parser; use tracing_subscriber::EnvFilter; use tracing::info;
#[derive(Parser)]
#[command(name = "kairos-llm", version = "1.0.0", about = "Local LLM runtime orchestrator and quantizer daemon")]
struct Cli { #[arg(short, long, default_value = "/etc/kairos/llm.toml")] config: PathBuf, #[arg(long)] infer: Option<String>, #[arg(long)] model: Option<String>, #[arg(long)] scan: bool }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse(); let cfg = kairos_llm::config::Config::load(&cli.config)?;
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new(&cfg.general.log_level)).init();
    info!("kairos-llm v{} starting", env!("CARGO_PKG_VERSION"));
    let state = kairos_llm::AppState::new(cfg).await?;
    if cli.scan { let models = state.model_registry.scan().await?; for m in models { println!("{}", m); } return Ok(()); }
    if let Some(prompt) = cli.infer { let m = cli.model.unwrap_or_else(|| "default".into()); let r = state.runtime.infer(&m, &prompt).await?; println!("{}", r); return Ok(()); }
    let wk = kairos_llm::worker::LlmWorker::new(state.config.clone()); wk.start().await?;
    tokio::signal::ctrl_c().await?; info!("Shutdown signal received"); Ok(())
}
