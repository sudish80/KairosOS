use std::sync::Arc; use tokio::sync::RwLock; use tracing::info; use crate::config;
pub struct MeshWorker { config: Arc<RwLock<config::Config>> }
impl MeshWorker {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn start(&self) -> anyhow::Result<()> { info!("MeshWorker started"); Ok(()) }
}
