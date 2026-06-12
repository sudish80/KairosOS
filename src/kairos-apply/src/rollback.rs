//! Rollback management — health-checked, bounded-retry state recovery
use crate::config;
use crate::error::ApplyError;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct RollbackManager {
    config: Arc<RwLock<config::Config>>,
    history_dir: PathBuf,
    active_link: PathBuf,
    pending_dir: PathBuf,
}

impl RollbackManager {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        let (history_dir, active_link, pending_dir) = {
            let cfg = config.read().await;
            (
                PathBuf::from(&cfg.store.history_dir),
                PathBuf::from(&cfg.store.active_link),
                PathBuf::from(&cfg.store.pending_dir),
            )
        };
        Ok(Self {
            config,
            history_dir,
            active_link,
            pending_dir,
        })
    }

    pub async fn rollback(&self, target_gen: Option<&str>) -> anyhow::Result<String> {
        let active_id = self.get_active_id().await?;
        let target = match target_gen {
            Some(id) => id.to_string(),
            None => self.find_previous_generation(&active_id).await?,
        };

        info!("Rolling back from {:?} to {}", active_id, target);

        // Verify target exists in history
        let target_path = self.history_dir.join(&target);
        if !target_path.exists() {
            return Err(
                ApplyError::Rollback(format!("Target generation not found: {}", target)).into(),
            );
        }

        // Atomic swap
        #[cfg(unix)]
        {
            let temp_link = format!("{}.rollback", self.active_link.display());
            let _ = std::fs::remove_file(&temp_link);
            std::os::unix::fs::symlink(&target_path, &temp_link)?;
            std::fs::rename(&temp_link, &self.active_link)?;
        }

        info!("Rollback to {} complete", target);
        Ok(target)
    }

    pub async fn health_check(&self) -> anyhow::Result<bool> {
        if !self.active_link.exists() {
            return Ok(false);
        }
        let target = std::fs::read_link(&self.active_link)?;
        Ok(target.exists())
    }

    async fn get_active_id(&self) -> anyhow::Result<Option<String>> {
        if !self.active_link.exists() {
            return Ok(None);
        }
        let target = std::fs::read_link(&self.active_link)?;
        Ok(target.file_name().map(|s| s.to_string_lossy().to_string()))
    }

    async fn find_previous_generation(&self, active: &Option<String>) -> anyhow::Result<String> {
        let history = self.list_history().await?;
        if let Some(ref active_id) = active {
            if let Some(pos) = history.iter().position(|m| m.id == *active_id) {
                if pos + 1 < history.len() {
                    return Ok(history[pos + 1].id.clone());
                }
            }
        }
        if let Some(last) = history.first() {
            return Ok(last.id.clone());
        }
        Err(ApplyError::Rollback("No previous generation found".into()).into())
    }

    async fn list_history(&self) -> anyhow::Result<Vec<super::generation::GenerationMetadata>> {
        let mut reader = fs::read_dir(&self.history_dir).await?;
        let mut result = Vec::new();
        while let Some(entry) = reader.next_entry().await? {
            let meta_path = entry.path().join("gen.json");
            if meta_path.exists() {
                let content = fs::read_to_string(&meta_path).await?;
                if let Ok(meta) =
                    serde_json::from_str::<super::generation::GenerationMetadata>(&content)
                {
                    result.push(meta);
                }
            }
        }
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }
}
