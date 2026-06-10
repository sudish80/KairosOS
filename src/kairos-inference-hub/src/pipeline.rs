//! Inference pipeline — speculative decoding orchestration
use crate::config;
use crate::error::InferenceError;
use crate::kv_cache::KVCache;
use crate::router::ModelRouter;
use crate::speculator::SpeculativeEngine;
use crate::telemetry::Telemetry;
use crate::InferenceResult;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

pub struct InferencePipeline {
    config: Arc<RwLock<config::Config>>,
    router: Arc<ModelRouter>,
    speculator: Arc<SpeculativeEngine>,
    kv_cache: Arc<KVCache>,
    telemetry: Arc<Telemetry>,
}

impl InferencePipeline {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        router: Arc<ModelRouter>,
        speculator: Arc<SpeculativeEngine>,
        kv_cache: Arc<KVCache>,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        Self {
            config,
            router,
            speculator,
            kv_cache,
            telemetry,
        }
    }

    pub async fn infer(&self, model: &str, prompt: &str) -> anyhow::Result<InferenceResult> {
        let start = Instant::now();

        // 1. Route to optimal model
        let model_handle = self.router.route(model, prompt).await?;

        // 2. Check KV cache
        let cache_key = format!("{}:{}", model, prompt);
        let cached = self.kv_cache.get(&cache_key).await;
        if let Some(cached_output) = cached {
            self.telemetry.record_cache_hit();
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            return Ok(InferenceResult {
                model: model_handle.name.clone(),
                output: cached_output,
                tokens_generated: 0,
                tokens_draft: 0,
                tokens_accepted: 0,
                acceptance_rate: 1.0,
                latency_ms: latency,
                spec_latency_ms: 0.0,
                cache_hit_rate: 1.0,
            });
        }
        self.telemetry.record_cache_miss();

        // 3. Run speculative decoding
        let speculation_start = Instant::now();
        let (output, stats) = self.speculator.speculate(model_handle, prompt).await?;
        let spec_latency = speculation_start.elapsed().as_secs_f64() * 1000.0;

        // 4. Cache result
        self.kv_cache.set(cache_key, output.clone()).await;

        let total_latency = start.elapsed().as_secs_f64() * 1000.0;
        self.telemetry.record_request(
            stats.tokens_generated as u64,
            stats.tokens_draft as u64,
            stats.tokens_accepted as u64,
            (start.elapsed().as_nanos()) as u64,
        );

        info!(
            "Inference complete: {} tokens (draft: {}, accepted: {}, rate: {:.2}) in {:.1}ms",
            stats.tokens_generated,
            stats.tokens_draft,
            stats.tokens_accepted,
            stats.acceptance_rate,
            total_latency
        );

        Ok(InferenceResult {
            model: output.model_name,
            output: output.text,
            tokens_generated: stats.tokens_generated,
            tokens_draft: stats.tokens_draft,
            tokens_accepted: stats.tokens_accepted,
            acceptance_rate: stats.acceptance_rate,
            latency_ms: total_latency,
            spec_latency_ms: spec_latency,
            cache_hit_rate: 0.0,
        })
    }
}
