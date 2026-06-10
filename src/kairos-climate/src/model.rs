use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct ClimateModel {
    config: Arc<RwLock<config::Config>>,
}
impl ClimateModel {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn step(&self) -> anyhow::Result<()> {
        tracing::debug!("Model step");
        Ok(())
    }
}
