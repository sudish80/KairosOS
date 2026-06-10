//! Background worker — dequeues inference requests, manages batch processing
use crate::config;
use crate::pipeline::InferencePipeline;
use crate::scheduler::InferenceScheduler;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct InferenceWorker {
    config: Arc<RwLock<config::Config>>,
    pipeline: Arc<InferencePipeline>,
    scheduler: Arc<InferenceScheduler>,
}

impl InferenceWorker {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        pipeline: Arc<InferencePipeline>,
        scheduler: Arc<InferenceScheduler>,
    ) -> Self {
        Self {
            config,
            pipeline,
            scheduler,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("InferenceWorker started");

        let pipeline = Arc::clone(&self.pipeline);
        let scheduler = Arc::clone(&self.scheduler);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                if let Some(batch) = scheduler.dequeue_batch().await {
                    for request in &batch.requests {
                        match pipeline.infer(&request.model, &request.prompt).await {
                            Ok(result) => debug!(
                                "Batch request {} complete: {} tokens",
                                request.id, result.tokens_generated
                            ),
                            Err(e) => error!("Batch request {} failed: {}", request.id, e),
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
