//! Bare git repository management with auto-init and GC via git CLI
use crate::config;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct RepoManager {
    config: Arc<RwLock<config::Config>>,
    pub bare_path: PathBuf,
    pub workdir: PathBuf,
    git_path: PathBuf,
}

impl RepoManager {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        let cfg = config.read().await;
        let bare_path = PathBuf::from(&cfg.repo.bare_path);
        let workdir = PathBuf::from(&cfg.repo.workdir);

        if cfg.repo.auto_init && !bare_path.exists() {
            std::fs::create_dir_all(&bare_path)?;
            let status = Command::new("git")
                .args(["init", "--bare", &bare_path.to_string_lossy()])
                .status()
                .await?;
            if status.success() {
                info!("Initialized bare git repo at {:?}", bare_path);
            }
        }

        Ok(Self {
            config,
            bare_path,
            workdir,
            git_path: bare_path.clone(),
        })
    }

    pub async fn add_and_commit(&self, paths: &[PathBuf], message: &str) -> anyhow::Result<String> {
        let cfg = self.config.read().await;

        // Add files to a temporary working tree
        let temp_clone = std::env::temp_dir().join(format!("kairos-git-{}", std::process::id()));
        if !temp_clone.exists() {
            Command::new("git")
                .args([
                    "clone",
                    &self.bare_path.to_string_lossy(),
                    &temp_clone.to_string_lossy(),
                ])
                .status()
                .await?;
        }

        // Copy files into worktree
        for path in paths {
            let relative = path.strip_prefix(&self.workdir).unwrap_or(path);
            let dest = temp_clone.join(relative);
            if let Some(parent) = dest.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::copy(path, &dest).await?;
        }

        // Git add
        Command::new("git")
            .args(["-C", &temp_clone.to_string_lossy(), "add", "-A"])
            .status()
            .await?;

        // Git commit
        let output = Command::new("git")
            .args([
                "-C",
                &temp_clone.to_string_lossy(),
                "-c",
                format!("user.name={}", cfg.commit.author_name),
                "-c",
                format!("user.email={}", cfg.commit.author_email),
                "commit",
                "-m",
                message,
                "--allow-empty",
            ])
            .output()
            .await?;

        let commit_hash = String::from_utf8_lossy(&output.stdout).to_string();
        let commit_hash = commit_hash.trim().to_string();

        // Push to bare repo
        Command::new("git")
            .args([
                "-C",
                &temp_clone.to_string_lossy(),
                "push",
                "origin",
                "HEAD",
            ])
            .status()
            .await?;

        info!("Committed {} files with message: {}", paths.len(), message);
        Ok(commit_hash)
    }

    pub async fn get_log(&self, max_count: usize) -> anyhow::Result<Vec<CommitInfo>> {
        let output = Command::new("git")
            .args([
                "--git-dir",
                &self.bare_path.to_string_lossy(),
                "log",
                format!("--max-count={}", max_count).as_str(),
                "--format=%H|%ai|%s",
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() >= 3 {
                    Some(CommitInfo {
                        hash: parts[0].to_string(),
                        date: parts[1].to_string(),
                        message: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn diff(&self, old_hash: &str, new_hash: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args([
                "--git-dir",
                &self.bare_path.to_string_lossy(),
                "diff",
                old_hash,
                new_hash,
            ])
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn get_head_hash(&self) -> anyhow::Result<Option<String>> {
        let output = Command::new("git")
            .args([
                "--git-dir",
                &self.bare_path.to_string_lossy(),
                "rev-parse",
                "HEAD",
            ])
            .output()
            .await?;
        if output.status.success() {
            Ok(Some(
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn run_gc(&self) -> anyhow::Result<()> {
        Command::new("git")
            .args([
                "--git-dir",
                &self.bare_path.to_string_lossy(),
                "gc",
                "--auto",
                "--quiet",
            ])
            .status()
            .await?;
        info!("Git GC completed");
        Ok(())
    }

    pub fn workdir(&self) -> &Path {
        &self.workdir
    }
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub date: String,
    pub message: String,
}
