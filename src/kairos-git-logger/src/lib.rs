//! kairos-git-logger: Git-backed /etc version tracker — production-hardened
#![deny(unsafe_code)]

pub mod config;
pub mod error;
pub mod telemetry;
pub mod worker;
pub mod watcher;
pub mod repo;
pub mod committer;
pub mod diff;
pub mod history;
pub mod snapshot;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::Telemetry>,
    pub repo_manager: Arc<repo::RepoManager>,
    pub watcher_engine: Arc<watcher::FileWatcher>,
    pub committer: Arc<committer::ChangeCommitter>,
    pub diff_engine: Arc<diff::DiffEngine>,
    pub history_manager: Arc<history::HistoryManager>,
    pub snapshot_manager: Arc<snapshot::SnapshotManager>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::Telemetry::new(Arc::clone(&config)));
        let repo_manager = Arc::new(repo::RepoManager::new(Arc::clone(&config)).await?);
        let committer = Arc::new(committer::ChangeCommitter::new(Arc::clone(&config), Arc::clone(&repo_manager)));
        let diff_engine = Arc::new(diff::DiffEngine::new(Arc::clone(&config)));
        let history_manager = Arc::new(history::HistoryManager::new(Arc::clone(&config), Arc::clone(&repo_manager)));
        let snapshot_manager = Arc::new(snapshot::SnapshotManager::new(Arc::clone(&config), Arc::clone(&repo_manager)));
        let watcher_engine = Arc::new(watcher::FileWatcher::new(
            Arc::clone(&config), Arc::clone(&committer), Arc::clone(&telemetry),
        ));

        info!("kairos-git-logger AppState initialized");
        Ok(Self { config, telemetry, repo_manager, watcher_engine, committer, diff_engine, history_manager, snapshot_manager })
    }
}
