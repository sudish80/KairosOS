use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct DbWorker {
    config: Arc<RwLock<config::Config>>,
}
impl DbWorker {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("DbWorker started");
        Ok(())
    }
}
