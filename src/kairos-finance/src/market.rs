use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct MarketFeed {
    config: Arc<RwLock<config::Config>>,
}
impl MarketFeed {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn connect(&self) -> anyhow::Result<()> {
        info!("Connecting to {}", self.config.read().await.market.feed_url);
        Ok(())
    }
}
