use std::sync::Arc; use tokio::sync::RwLock; use crate::config;
pub struct FrameCapture { config: Arc<RwLock<config::Config>> }
impl FrameCapture {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn open(&self) -> anyhow::Result<()> { tracing::info!("Opening {}", self.config.read().await.capture.device); Ok(()) }
}
