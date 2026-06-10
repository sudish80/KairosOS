use std::sync::Arc; use tokio::sync::RwLock; use tracing::info; use crate::config;
pub struct QuantumWorker { config: Arc<RwLock<config::Config>> }
impl QuantumWorker { pub fn new(c: Arc<RwLock<config::Config>>) -> Self { Self{config:c} } pub async fn start(&self) -> anyhow::Result<()> { info!("QuantumWorker started"); Ok(()) } }
