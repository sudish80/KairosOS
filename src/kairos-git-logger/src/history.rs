//! History manager — query git log, filter by path/date, paginate
use crate::config;
use crate::repo::{CommitInfo, RepoManager};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HistoryManager {
    config: Arc<RwLock<config::Config>>,
    repo_manager: Arc<RepoManager>,
}

impl HistoryManager {
    pub fn new(config: Arc<RwLock<config::Config>>, repo_manager: Arc<RepoManager>) -> Self {
        Self {
            config,
            repo_manager,
        }
    }

    pub async fn get_log(&self, count: usize) -> anyhow::Result<Vec<CommitInfo>> {
        self.repo_manager.get_log(count).await
    }

    pub async fn get_timeline(&self) -> anyhow::Result<Vec<CommitTimeline>> {
        let commits = self.get_log(50).await?;
        Ok(commits
            .into_iter()
            .map(|c| CommitTimeline {
                hash: c.hash,
                date: c.date,
                message: c.message,
                files_changed: 0, // In production: git show --stat
            })
            .collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommitTimeline {
    pub hash: String,
    pub date: String,
    pub message: String,
    pub files_changed: usize,
}
