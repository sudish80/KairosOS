//! Diff engine for git log comparisons
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config;
use crate::repo::RepoManager;

pub struct DiffEngine {
    config: Arc<RwLock<config::Config>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffSummary {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl DiffEngine {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }

    pub async fn diff(&self, repo: &RepoManager, from: &str, to: &str) -> anyhow::Result<String> {
        repo.diff(from, to).await
    }

    pub fn summarize_diff(diff: &str) -> DiffSummary {
        let mut files_changed = 0usize;
        let mut insertions = 0usize;
        let mut deletions = 0usize;

        for line in diff.lines() {
            if line.starts_with("diff --git") {
                files_changed += 1;
            } else if line.starts_with('+') && !line.starts_with("+++") {
                insertions += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                deletions += 1;
            }
        }

        DiffSummary { files_changed, insertions, deletions }
    }
}
