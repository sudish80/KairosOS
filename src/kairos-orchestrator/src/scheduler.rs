use std::sync::Arc; use tokio::sync::RwLock;
use tracing::info; use crate::config; use crate::dag::{DagEngine, DagNode};
pub struct TaskScheduler { config: Arc<RwLock<config::Config>>, dag: Arc<DagEngine> }
impl TaskScheduler {
    pub fn new(config: Arc<RwLock<config::Config>>, dag: Arc<DagEngine>) -> Self { Self { config, dag } }
    pub async fn schedule_next(&self) -> Vec<DagNode> { self.dag.get_ready().await }
    pub async fn start_scheduling_loop(&self) {
        let dag = Arc::clone(&self.dag);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let ready = dag.get_ready().await;
                if !ready.is_empty() { info!("Scheduler: {} tasks ready", ready.len()); }
            }
        });
    }
}
