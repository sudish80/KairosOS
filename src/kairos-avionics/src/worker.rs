use std::sync::Arc; use tokio::sync::RwLock; use tracing::info; use crate::config;
pub struct AvionicsWorker { config: Arc<RwLock<config::Config>> }
impl AvionicsWorker { pub fn new(c: Arc<RwLock<config::Config>>) -> Self { Self { config: c } }
    pub async fn start(&self) -> anyhow::Result<()> { info!("AvionicsWorker started"); Ok(()) }
}
