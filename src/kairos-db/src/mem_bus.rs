use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct MemoryBus {
    config: Arc<RwLock<config::Config>>,
    messages: Arc<RwLock<Vec<String>>>,
}
impl MemoryBus {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
    pub async fn publish(&self, _channel: &str, _msg: &str) -> anyhow::Result<()> {
        self.messages.write().await.push(_msg.to_string());
        Ok(())
    }
    pub async fn subscribe(&self, _channels: &[&str]) -> Vec<String> {
        self.messages.write().await.drain(..).collect()
    }
}
