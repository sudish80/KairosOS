use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct DrmManager {
    config: Arc<RwLock<config::Config>>,
}
impl DrmManager {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn open(&self) -> anyhow::Result<()> {
        info!("DRM opened: {}", self.config.read().await.drm.card);
        Ok(())
    }
    pub async fn set_mode(&self) -> anyhow::Result<()> {
        info!("DRM mode set");
        Ok(())
    }
}
