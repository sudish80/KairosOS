//! kairos-inference-hub: Speculative inference pipeline — production-hardened
//! Implements draft/oracle model orchestration, quantizer, KV cache, model routing
#![deny(unsafe_code)]

pub mod config;
pub mod error;
pub mod telemetry;
pub mod worker;
pub mod pipeline;
pub mod models;
pub mod quantizer;
pub mod kv_cache;
pub mod scheduler;
pub mod router;
pub mod speculator;
pub mod metrics;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::Telemetry>,
    pub model_registry: Arc<models::ModelRegistry>,
    pub quantizer: Arc<quantizer::Quantizer>,
    pub kv_cache: Arc<kv_cache::KVCache>,
    pub scheduler: Arc<scheduler::InferenceScheduler>,
    pub router: Arc<router::ModelRouter>,
    pub speculator: Arc<speculator::SpeculativeEngine>,
    pub pipeline: Arc<pipeline::InferencePipeline>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::Telemetry::new(Arc::clone(&config)));
        let model_registry = Arc::new(models::ModelRegistry::new(Arc::clone(&config)).await?);
        let quantizer = Arc::new(quantizer::Quantizer::new(Arc::clone(&config)));
        let kv_cache = Arc::new(kv_cache::KVCache::new(Arc::clone(&config)));
        let scheduler = Arc::new(scheduler::InferenceScheduler::new(Arc::clone(&config)));
        let router = Arc::new(router::ModelRouter::new(
            Arc::clone(&config), Arc::clone(&model_registry), Arc::clone(&scheduler),
        ));
        let speculator = Arc::new(speculator::SpeculativeEngine::new(
            Arc::clone(&config), Arc::clone(&model_registry), Arc::clone(&kv_cache),
        ));
        let pipeline = Arc::new(pipeline::InferencePipeline::new(
            Arc::clone(&config), Arc::clone(&router), Arc::clone(&speculator),
            Arc::clone(&kv_cache), Arc::clone(&telemetry),
        ));

        info!("kairos-inference-hub AppState initialized");
        Ok(Self { config, telemetry, model_registry, quantizer, kv_cache, scheduler, router, speculator, pipeline })
    }

    pub async fn infer(&self, model: &str, prompt: &str) -> anyhow::Result<InferenceResult> {
        self.pipeline.infer(model, prompt).await
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InferenceResult {
    pub model: String,
    pub output: String,
    pub tokens_generated: usize,
    pub tokens_draft: usize,
    pub tokens_accepted: usize,
    pub acceptance_rate: f64,
    pub latency_ms: f64,
    pub spec_latency_ms: f64,
    pub cache_hit_rate: f64,
}
