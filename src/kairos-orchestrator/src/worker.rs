use crate::config;
use crate::executor::TaskExecutor;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct OrchestratorWorker {
    config: Arc<RwLock<config::Config>>,
    executor: Arc<TaskExecutor>,
}
impl OrchestratorWorker {
    pub fn new(config: Arc<RwLock<config::Config>>, executor: Arc<TaskExecutor>) -> Self {
        Self { config, executor }
    }
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("OrchestratorWorker started");
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
        Ok(())
    }
}
