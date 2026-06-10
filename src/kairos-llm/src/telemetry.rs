use std::sync::Arc; use std::sync::atomic::{AtomicU64, Ordering}; use tokio::sync::RwLock; use crate::config;
pub struct Telemetry { config: Arc<RwLock<config::Config>>, inferences: AtomicU64, tokens_gen: AtomicU64, models_loaded: AtomicU64, errors_total: AtomicU64 }
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self { Self { config: c, inferences: AtomicU64::new(0), tokens_gen: AtomicU64::new(0), models_loaded: AtomicU64::new(0), errors_total: AtomicU64::new(0) } }
    pub fn record_inference(&self, tokens: u64) { self.inferences.fetch_add(1, Ordering::Relaxed); self.tokens_gen.fetch_add(tokens, Ordering::Relaxed); }
    pub fn record_model_load(&self) { self.models_loaded.fetch_add(1, Ordering::Relaxed); }
    pub fn record_error(&self) { self.errors_total.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value { serde_json::json!({"inferences": self.inferences.load(Ordering::Relaxed), "tokens_generated": self.tokens_gen.load(Ordering::Relaxed), "models_loaded": self.models_loaded.load(Ordering::Relaxed), "errors_total": self.errors_total.load(Ordering::Relaxed)}) }
}
