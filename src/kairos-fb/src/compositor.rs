use crate::canvas::Canvas;
use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct Compositor {
    config: Arc<RwLock<config::Config>>,
    canvas: Arc<Canvas>,
}
impl Compositor {
    pub fn new(config: Arc<RwLock<config::Config>>, canvas: Arc<Canvas>) -> Self {
        Self { config, canvas }
    }
    pub async fn composite(&self) -> anyhow::Result<()> {
        self.canvas.fill(0x1E1E1E).await;
        self.canvas.present().await
    }
}
