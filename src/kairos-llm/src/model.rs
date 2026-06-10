use std::collections::HashMap; use std::path::PathBuf; use std::sync::Arc;
use tokio::sync::RwLock; use tokio::fs; use tracing::{info, debug}; use crate::config;
pub struct ModelRegistry { config: Arc<RwLock<config::Config>>, models: Arc<RwLock<HashMap<String, ModelInfo>>> }
#[derive(Debug, Clone)]
pub struct ModelInfo { pub name: String, pub path: PathBuf, pub quant: String, pub size_bytes: u64, pub loaded: bool }
impl ModelRegistry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config, models: Arc::new(RwLock::new(HashMap::new())) } }
    pub async fn scan(&self) -> anyhow::Result<Vec<String>> {
        let dir = self.config.read().await.models.models_dir.clone();
        let mut names = Vec::new(); let mut reader = fs::read_dir(&dir).await?;
        while let Some(entry) = reader.next_entry().await? {
            let p = entry.path();
            if p.extension().map_or(false, |e| e == "gguf") {
                let meta = fs::metadata(&p).await?;
                let name = p.file_stem().unwrap().to_string_lossy().to_string();
                self.models.write().await.insert(name.clone(), ModelInfo {
                    name: name.clone(), path: p, quant: "auto".into(), size_bytes: meta.len(), loaded: false,
                });
                names.push(name);
            }
        }
        info!("Scanned {} models from {}", names.len(), dir); Ok(names)
    }
}
