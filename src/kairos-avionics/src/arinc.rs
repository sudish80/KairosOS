use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Arinc429 {
    config: Arc<RwLock<config::Config>>,
}
impl Arinc429 {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn read_label(&self, label: &str) -> anyhow::Result<f64> {
        tracing::debug!("Reading ARINC label: {}", label);
        Ok(0.0)
    }
}
