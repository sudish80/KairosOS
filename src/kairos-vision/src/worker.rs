use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct VisionWorker {
    config: Arc<RwLock<config::Config>>,
}
impl VisionWorker {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self {
        Self { config: c }
    }
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("VisionWorker started");
        Ok(())
    }
}
