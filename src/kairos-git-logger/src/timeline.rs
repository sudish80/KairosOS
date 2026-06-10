use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub id: String,
    pub timestamp: f64,
    pub generation: u64,
    pub root_hash: String,
    pub file_count: usize,
    pub total_changes: usize,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub old_mode: Option<String>,
    pub new_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub snapshot: StateSnapshot,
    pub diffs: Vec<FileDiff>,
    pub metadata: HashMap<String, String>,
}

pub struct ImmutableTimeline {
    repo_path: PathBuf,
    snapshots: Arc<RwLock<Vec<TimelineEntry>>>,
    generation: Arc<RwLock<u64>>,
}

impl ImmutableTimeline {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            repo_path: repo_path.to_path_buf(),
            snapshots: Arc::new(RwLock::new(Vec::new())),
            generation: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn init_repo(&self) -> anyhow::Result<()> {
        if !self.repo_path.join(".git").exists() {
            Command::new("git")
                .args(["init", "--initial-branch=kairos"])
                .current_dir(&self.repo_path)
                .status()
                .await?;

            Command::new("git")
                .args(["config", "user.name", "kairos-timeline"])
                .current_dir(&self.repo_path)
                .status()
                .await?;
            Command::new("git")
                .args(["config", "user.email", "timeline@kairos.local"])
                .current_dir(&self.repo_path)
                .status()
                .await?;
            Command::new("git")
                .args(["config", "commit.gpgSign", "false"])
                .current_dir(&self.repo_path)
                .status()
                .await?;

            info!("Initialized immutable timeline at {:?}", self.repo_path);
        }
        Ok(())
    }

    pub async fn snapshot_repo(&self) -> anyhow::Result<TimelineEntry> {
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.repo_path)
            .status()
            .await?;

        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .await?;

        let changes = String::from_utf8_lossy(&status.stdout);
        if changes.trim().is_empty() {
            anyhow::bail!("No changes to snapshot");
        }

        let gen = {
            let mut g = self.generation.write().await;
            *g += 1;
            *g
        };

        let commit_msg = format!("kairos timeline snapshot generation {}", gen);

        Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(&self.repo_path)
            .status()
            .await?;

        let hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .await?;
        let root_hash = String::from_utf8_lossy(&hash_output.stdout).trim().to_string();

        let diff_output = Command::new("git")
            .args(["diff", "--staged", "--stat"])
            .current_dir(&self.repo_path)
            .output()
            .await?;
        let diff_stat = String::from_utf8_lossy(&diff_output.stdout);
        let file_count = diff_stat.lines().count();
        let total_changes: usize = diff_stat.lines()
            .filter_map(|l| l.split('|').nth(1))
            .map(|s| s.trim().chars().filter(|&c| c == '+').count())
            .sum();

        let diffs = self.get_file_diffs().await;

        let snapshot = StateSnapshot {
            id: root_hash.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?.as_secs_f64(),
            generation: gen,
            root_hash,
            file_count,
            total_changes,
            parent_id: None,
        };

        let entry = TimelineEntry {
            snapshot,
            diffs,
            metadata: HashMap::new(),
        };

        self.snapshots.write().await.push(entry.clone());

        Command::new("git")
            .args(["notes", "add", "-m", format!("immutable-generation-{}", gen).as_str()])
            .current_dir(&self.repo_path)
            .status()
            .await?;

        info!("Snapshot generation {}: {} files, {} changes", gen, file_count, total_changes);
        Ok(entry)
    }

    async fn get_file_diffs(&self) -> Vec<FileDiff> {
        let mut diffs = Vec::new();

        let output = Command::new("git")
            .args(["diff", "HEAD~1", "--name-status"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .ok();

        if let Some(output) = output {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if let Some((status, path)) = line.split_once('\t') {
                    let change_type = match status {
                        "A" => ChangeType::Added,
                        "D" => ChangeType::Deleted,
                        "M" => ChangeType::Modified,
                        _ if status.contains("T") => ChangeType::Permission,
                        _ => ChangeType::Modified,
                    };
                    diffs.push(FileDiff {
                        path: PathBuf::from(path),
                        change_type,
                        old_hash: None,
                        new_hash: None,
                        old_mode: None,
                        new_mode: None,
                    });
                }
            }
        }

        diffs
    }

    pub async fn get_timeline(&self) -> Vec<TimelineEntry> {
        self.snapshots.read().await.clone()
    }

    pub async fn get_snapshot(&self, generation: u64) -> Option<TimelineEntry> {
        let snapshots = self.snapshots.read().await;
        snapshots.iter().find(|e| e.snapshot.generation == generation).cloned()
    }

    pub async fn diff_vectors(&self, from_gen: u64, to_gen: u64) -> anyhow::Result<Vec<FileDiff>> {
        let from = self.get_snapshot(from_gen).await;
        let to = self.get_snapshot(to_gen).await;

        match (from, to) {
            (Some(f), Some(t)) => {
                let mut all_diffs: HashMap<String, FileDiff> = HashMap::new();
                for diff in f.diffs {
                    all_diffs.insert(diff.path.to_string_lossy().to_string(), diff);
                }
                for diff in t.diffs {
                    let key = diff.path.to_string_lossy().to_string();
                    all_diffs.insert(key, diff);
                }
                Ok(all_diffs.into_values().collect())
            }
            _ => Err(anyhow::anyhow!("Snapshot not found")),
        }
    }

    pub async fn current_generation(&self) -> u64 {
        *self.generation.read().await
    }

    pub async fn is_repo_clean(&self) -> bool {
        Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().is_empty())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_repo() {
        let tmp = TempDir::new().unwrap();
        let timeline = ImmutableTimeline::new(tmp.path());
        timeline.init_repo().await.unwrap();
        assert!(tmp.path().join(".git").exists());
    }

    #[tokio::test]
    async fn test_snapshot_with_changes() {
        let tmp = TempDir::new().unwrap();
        let timeline = ImmutableTimeline::new(tmp.path());
        timeline.init_repo().await.unwrap();

        std::fs::write(tmp.path().join("test.txt"), b"hello").unwrap();

        let result = timeline.snapshot_repo().await;
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.snapshot.generation, 1);
        assert!(!entry.snapshot.root_hash.is_empty());
    }

    #[tokio::test]
    async fn test_no_changes_returns_error() {
        let tmp = TempDir::new().unwrap();
        let timeline = ImmutableTimeline::new(tmp.path());
        timeline.init_repo().await.unwrap();

        std::fs::write(tmp.path().join("initial.txt"), b"initial").unwrap();
        timeline.snapshot_repo().await.unwrap();

        let result = timeline.snapshot_repo().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeline_with_three_snapshots() {
        let tmp = TempDir::new().unwrap();
        let timeline = ImmutableTimeline::new(tmp.path());
        timeline.init_repo().await.unwrap();

        for i in 1..=3 {
            std::fs::write(tmp.path().join(format!("file{}.txt", i)), format!("content {}", i)).unwrap();
            let entry = timeline.snapshot_repo().await.unwrap();
            assert_eq!(entry.snapshot.generation, i as u64);
        }

        let timeline_entries = timeline.get_timeline().await;
        assert_eq!(timeline_entries.len(), 3);
    }

    #[tokio::test]
    async fn test_diff_vectors() {
        let tmp = TempDir::new().unwrap();
        let timeline = ImmutableTimeline::new(tmp.path());
        timeline.init_repo().await.unwrap();

        std::fs::write(tmp.path().join("a.txt"), b"v1").unwrap();
        timeline.snapshot_repo().await.unwrap();

        std::fs::write(tmp.path().join("a.txt"), b"v2").unwrap();
        std::fs::write(tmp.path().join("b.txt"), b"new").unwrap();
        timeline.snapshot_repo().await.unwrap();

        let diffs = timeline.diff_vectors(1, 2).await;
        assert!(diffs.is_ok());
    }

    #[test]
    fn test_change_type_enum() {
        assert_eq!(ChangeType::Added, ChangeType::Added);
        assert_ne!(ChangeType::Modified, ChangeType::Deleted);
    }
}
