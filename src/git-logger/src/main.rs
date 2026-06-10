#![deny(unsafe_code)]
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = git_logger::config::Config::default();
    let state = git_logger::AppState::new(cfg).await?;

    // Start inotify watcher
    let watcher = std::sync::Arc::clone(&state.watcher);
    tokio::spawn(async move { watcher.watch_loop().await });

    tracing::info!("git-logger daemon started");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received");
    Ok(())
}
