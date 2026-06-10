use std::sync::Arc; use tokio::sync::RwLock;
use tracing::info; use crate::config; use crate::dag::DagEngine; use crate::scheduler::TaskScheduler;
use crate::task::TaskDef;
pub struct PipelineManager { config: Arc<RwLock<config::Config>>, dag: Arc<DagEngine>, scheduler: Arc<TaskScheduler> }
impl PipelineManager {
    pub fn new(config: Arc<RwLock<config::Config>>, dag: Arc<DagEngine>, scheduler: Arc<TaskScheduler>) -> Self { Self { config, dag, scheduler } }
    pub async fn submit(&self, name: &str, tasks: Vec<TaskDef>) -> anyhow::Result<String> {
        let ids = self.dag.build(tasks).await?;
        info!("Pipeline '{}' submitted with {} tasks", name, ids.len());
        self.scheduler.start_scheduling_loop().await;
        Ok(name.to_string())
    }
}
