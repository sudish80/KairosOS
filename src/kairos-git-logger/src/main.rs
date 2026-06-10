use clap::Parser;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "kairos-git-logger",
    version = "1.0.0",
    about = "Git-backed /etc version tracker"
)]
struct Cli {
    #[arg(short, long, default_value = "/etc/kairos/git-logger.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = kairos_git_logger::config::Config::load(&cli.config)?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.general.log_level))
        .init();

    info!("kairos-git-logger v{} starting", env!("CARGO_PKG_VERSION"));

    let state = kairos_git_logger::AppState::new(cfg).await?;

    // Start GC worker
    let gc_worker = kairos_git_logger::worker::GitLoggerWorker::new(
        state.config.clone(),
        state.repo_manager.clone(),
    );
    gc_worker.start().await?;

    // Start file watcher
    state.watcher_engine.start().await?;

    Ok(())
}
