use std::sync::Arc; use tokio::sync::RwLock; use crate::config;
pub struct PackageManager { config: Arc<RwLock<config::Config>> }
impl PackageManager {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn install(&self, _pkg: &str) -> anyhow::Result<()> { tracing::info!("Package install: {}", _pkg); Ok(()) }
    pub async fn remove(&self, _pkg: &str) -> anyhow::Result<()> { Ok(()) }
}
