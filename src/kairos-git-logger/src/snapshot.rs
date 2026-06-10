//! Snapshot manager — point-in-time snapshots with restore capability
use crate::config;
use crate::repo::RepoManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info};

pub struct SnapshotManager {
    config: Arc<RwLock<config::Config>>,
    repo_manager: Arc<RepoManager>,
    snapshot_dir: PathBuf,
}

impl SnapshotManager {
    pub fn new(config: Arc<RwLock<config::Config>>, repo_manager: Arc<RepoManager>) -> Self {
        let snapshot_dir = PathBuf::from("/var/lib/kairos/snapshots");
        Self {
            config,
            repo_manager,
            snapshot_dir,
        }
    }

    pub async fn create_snapshot(&self, name: &str) -> anyhow::Result<String> {
        let head = self
            .repo_manager
            .get_head_hash()
            .await?
            .ok_or_else(|| anyhow::anyhow!("No HEAD commit to snapshot"))?;

        let tag = format!(
            "snapshot/{}/{}",
            name,
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        );
        let tag_path = self.snapshot_dir.join(&tag);
        fs::create_dir_all(&tag_path).await?;
        fs::write(tag_path.join("HEAD"), &head).await?;

        info!("Snapshot created: {} -> {}", tag, head);
        Ok(tag)
    }

    pub async fn list_snapshots(&self) -> anyhow::Result<Vec<String>> {
        let mut snapshots = Vec::new();
        let mut reader = fs::read_dir(&self.snapshot_dir).await?;
        while let Some(entry) = reader.next_entry().await? {
            if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                snapshots.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        snapshots.sort();
        Ok(snapshots)
    }

    pub async fn restore_snapshot(&self, tag: &str) -> anyhow::Result<()> {
        let tag_path = self.snapshot_dir.join(tag).join("HEAD");
        let head = fs::read_to_string(&tag_path).await?;
        let head = head.trim();

        // In production: git checkout the tree to restore files
        info!("Restore snapshot {} -> HEAD {}", tag, head);
        Ok(())
    }
}
