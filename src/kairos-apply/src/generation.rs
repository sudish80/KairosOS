//! Generation store with atomic symlink swap
use crate::config;
use crate::error::ApplyError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationMetadata {
    pub id: String,
    pub parent_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub checksum: String,
    pub file_count: usize,
}

pub struct GenerationStore {
    config: Arc<RwLock<config::Config>>,
    active_link: PathBuf,
    pending_dir: PathBuf,
    history_dir: PathBuf,
}

impl GenerationStore {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        let cfg = config.read().await;
        let store = Self {
            config,
            active_link: PathBuf::from(&cfg.store.active_link),
            pending_dir: PathBuf::from(&cfg.store.pending_dir),
            history_dir: PathBuf::from(&cfg.store.history_dir),
        };
        fs::create_dir_all(&store.pending_dir).await?;
        fs::create_dir_all(&store.history_dir).await?;
        info!("GenerationStore initialized at {:?}", store.pending_dir);
        Ok(store)
    }

    pub async fn create_generation(
        &self,
        desc: &str,
        files: &[(String, Vec<u8>)],
    ) -> anyhow::Result<String> {
        let id = format!("gen-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S-%3f"));
        let gen_dir = self.pending_dir.join(&id);
        fs::create_dir_all(&gen_dir).await?;

        for (name, data) in files {
            let file_path = gen_dir.join(name);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&file_path, data).await?;
        }

        let checksum = self.compute_checksum(&gen_dir).await?;
        let meta = GenerationMetadata {
            id: id.clone(),
            parent_id: None,
            created_at: chrono::Utc::now(),
            description: desc.to_string(),
            checksum,
            file_count: files.len(),
        };
        fs::write(
            gen_dir.join("gen.json"),
            serde_json::to_string_pretty(&meta)?,
        )
        .await?;

        info!("Creation generation: {} ({} files)", id, files.len());
        Ok(id)
    }

    pub async fn apply_generation(&self, id: &str) -> anyhow::Result<()> {
        let gen_dir = self.pending_dir.join(id);
        if !gen_dir.exists() {
            return Err(ApplyError::Generation(format!("Generation not found: {}", id)).into());
        }

        let temp_link = format!("{}.tmp", self.active_link.display());
        // Symlink to new generation
        #[cfg(unix)]
        {
            let _ = std::fs::remove_file(&temp_link);
            std::os::unix::fs::symlink(&gen_dir, &temp_link)?;
            std::fs::rename(&temp_link, &self.active_link)?;
        }
        #[cfg(not(unix))]
        {
            return Err(anyhow::anyhow!(
                "Symlink-based apply only supported on Unix"
            ));
        }

        // Move to history
        let history_path = self.history_dir.join(id);
        fs::rename(&gen_dir, &history_path).await?;

        info!("Applied generation: {} -> {:?}", id, self.history_dir);
        Ok(())
    }

    pub async fn get_active_id(&self) -> anyhow::Result<Option<String>> {
        if !self.active_link.exists() {
            return Ok(None);
        }
        let target = std::fs::read_link(&self.active_link)?;
        Ok(target.file_name().map(|s| s.to_string_lossy().to_string()))
    }

    pub async fn list_history(&self) -> anyhow::Result<Vec<GenerationMetadata>> {
        let mut reader = fs::read_dir(&self.history_dir).await?;
        let mut result = Vec::new();
        while let Some(entry) = reader.next_entry().await? {
            let meta_path = entry.path().join("gen.json");
            if meta_path.exists() {
                let content = fs::read_to_string(&meta_path).await?;
                if let Ok(meta) = serde_json::from_str::<GenerationMetadata>(&content) {
                    result.push(meta);
                }
            }
        }
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    async fn compute_checksum(&self, dir: &Path) -> anyhow::Result<String> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let mut entries: Vec<_> = Vec::new();

        let mut reader = fs::read_dir(dir).await?;
        while let Some(entry) = reader.next_entry().await? {
            if entry.file_name() != "gen.json" {
                entries.push(entry.path());
            }
        }
        entries.sort();

        for entry in &entries {
            let data = fs::read(entry).await?;
            hasher.update(data);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}
