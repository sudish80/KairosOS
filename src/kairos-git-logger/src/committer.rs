//! Change committer — batches file changes into git commits
use crate::config;
use crate::repo::RepoManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct ChangeCommitter {
    config: Arc<RwLock<config::Config>>,
    repo_manager: Arc<RepoManager>,
}

impl ChangeCommitter {
    pub fn new(config: Arc<RwLock<config::Config>>, repo_manager: Arc<RepoManager>) -> Self {
        Self {
            config,
            repo_manager,
        }
    }

    pub async fn commit_changes(&self, paths: &[PathBuf]) -> anyhow::Result<String> {
        if paths.is_empty() {
            return Ok(String::new());
        }

        // Filter to existing files only
        let mut valid_paths = Vec::new();
        for path in paths {
            if fs::metadata(path).await.is_ok() {
                valid_paths.push(path.clone());
            }
        }

        if valid_paths.is_empty() {
            return Ok(String::new());
        }

        let message = format!(
            "{} Auto-commit: {} file(s) changed",
            self.config.read().await.commit.commit_message_prefix,
            valid_paths.len(),
        );

        let hash = self
            .repo_manager
            .add_and_commit(&valid_paths, &message)
            .await?;
        info!("Committed {} files as {}", valid_paths.len(), hash);
        Ok(hash)
    }
}
