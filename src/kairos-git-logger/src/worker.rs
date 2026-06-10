//! Background worker — periodic GC and health checks
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use crate::config;
use crate::repo::RepoManager;

pub struct GitLoggerWorker {
    config: Arc<RwLock<config::Config>>,
    repo_manager: Arc<RepoManager>,
}

impl GitLoggerWorker {
    pub fn new(config: Arc<RwLock<config::Config>>, repo_manager: Arc<RepoManager>) -> Self {
        Self { config, repo_manager }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let gc_interval = self.config.read().await.repo.gc_interval_secs;
        info!("GitLoggerWorker started, GC interval: {}s", gc_interval);

        let repo_manager = Arc::clone(&self.repo_manager);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(gc_interval));
            loop {
                interval.tick().await;
                if let Err(e) = repo_manager.run_gc().await {
                    error!("GC failed: {}", e);
                }
            }
        });

        Ok(())
    }
}
