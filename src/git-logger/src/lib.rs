#![deny(unsafe_code)]
pub mod config;
pub mod error;
pub mod watcher;
pub mod repo;
pub mod timeline;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub watcher: Arc<watcher::ConfigWatcher>,
    pub repo_engine: Arc<repo::RepoEngine>,
    pub timeline: Arc<timeline::ImmutableTimeline>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let watcher = Arc::new(watcher::ConfigWatcher::new(Arc::clone(&config)));
        let repo_engine = Arc::new(repo::RepoEngine::new(Arc::clone(&config)).await?);
        let tl = timeline::ImmutableTimeline::new(std::path::Path::new("/etc/kairos-timeline"));
        tl.init_repo().await?;
        let timeline = Arc::new(tl);
        info!("git-logger AppState initialized");
        Ok(Self { config, watcher, repo_engine, timeline })
    }
}
