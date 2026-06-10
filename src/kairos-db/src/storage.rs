use crate::config;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info};
pub struct StorageEngine {
    config: Arc<RwLock<config::Config>>,
    db_path: PathBuf,
}
impl StorageEngine {
    pub async fn new(config: Arc<RwLock<config::Config>>) -> anyhow::Result<Self> {
        let cfg = config.read().await;
        let db_path = PathBuf::from(&cfg.storage.db_path);
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        info!("StorageEngine at {:?}", db_path);
        Ok(Self { config, db_path })
    }
    pub async fn store(&self, key: &str, value: &[u8]) -> anyhow::Result<()> {
        let path = self.db_path.join(key);
        fs::create_dir_all(path.parent().unwrap()).await?;
        fs::write(&path, value).await?;
        Ok(())
    }
    pub async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let path = self.db_path.join(key);
        if path.exists() {
            Ok(Some(fs::read(&path).await?))
        } else {
            Ok(None)
        }
    }
    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        fs::remove_file(self.db_path.join(key)).await?;
        Ok(())
    }
}
