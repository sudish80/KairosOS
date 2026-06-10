#![deny(unsafe_code)]
pub mod config;
pub mod error;
pub mod telemetry;
pub mod worker;
pub mod dag;
pub mod scheduler;
pub mod executor;
pub mod resource;
pub mod pipeline;
pub mod task;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::Telemetry>,
    pub dag_engine: Arc<dag::DagEngine>,
    pub task_scheduler: Arc<scheduler::TaskScheduler>,
    pub task_executor: Arc<executor::TaskExecutor>,
    pub resource_manager: Arc<resource::ResourceManager>,
    pub pipeline_manager: Arc<pipeline::PipelineManager>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::Telemetry::new(Arc::clone(&config)));
        let dag_engine = Arc::new(dag::DagEngine::new(Arc::clone(&config)));
        let resource_manager = Arc::new(resource::ResourceManager::new(Arc::clone(&config)));
        let task_scheduler = Arc::new(scheduler::TaskScheduler::new(Arc::clone(&config), Arc::clone(&dag_engine)));
        let task_executor = Arc::new(executor::TaskExecutor::new(Arc::clone(&config), Arc::clone(&resource_manager)));
        let pipeline_manager = Arc::new(pipeline::PipelineManager::new(Arc::clone(&config), Arc::clone(&dag_engine), Arc::clone(&task_scheduler)));

        info!("kairos-orchestrator AppState initialized");
        Ok(Self { config, telemetry, dag_engine, task_scheduler, task_executor, resource_manager, pipeline_manager })
    }

    pub async fn submit_pipeline(&self, name: &str, tasks: Vec<task::TaskDef>) -> anyhow::Result<String> {
        self.pipeline_manager.submit(name, tasks).await
    }
}
