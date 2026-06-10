//! Model router — selects optimal model based on prompt characteristics, load, latency
use crate::config;
use crate::error::InferenceError;
use crate::models::{ModelHandle, ModelRegistry, ModelType};
use crate::scheduler::InferenceScheduler;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub struct ModelRouter {
    config: Arc<RwLock<config::Config>>,
    model_registry: Arc<ModelRegistry>,
    scheduler: Arc<InferenceScheduler>,
}

impl ModelRouter {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        model_registry: Arc<ModelRegistry>,
        scheduler: Arc<InferenceScheduler>,
    ) -> Self {
        Self {
            config,
            model_registry,
            scheduler,
        }
    }

    pub async fn route(&self, model_name: &str, prompt: &str) -> anyhow::Result<ModelHandle> {
        let cfg = self.config.read().await;

        // Try requested model first
        if let Ok(handle) = self.model_registry.get_or_load(model_name).await {
            return Ok(handle);
        }

        // Try fallback chain: oracle → fallback → draft
        for fallback in [
            &cfg.models.oracle_model,
            &cfg.models.fallback_model,
            &cfg.models.draft_model,
        ] {
            if let Ok(handle) = self.model_registry.get_or_load(fallback).await {
                info!("Routed '{}' to fallback model '{}'", model_name, fallback);
                return Ok(handle);
            }
        }

        Err(InferenceError::ModelNotFound(model_name.to_string()).into())
    }

    pub async fn select_draft(&self, oracle: &ModelHandle) -> anyhow::Result<ModelHandle> {
        let cfg = self.config.read().await;
        self.model_registry
            .get_or_load(&cfg.models.draft_model)
            .await
    }

    fn estimate_prompt_complexity(&self, prompt: &str) -> f64 {
        let length = prompt.len() as f64;
        let has_code = prompt.contains("```") as u32 as f64;
        let has_math = prompt.chars().filter(|c| c.is_ascii_punctuation()).count() as f64;
        length * 0.01 + has_code * 10.0 + has_math * 0.5
    }
}
