use std::sync::Arc; use std::sync::atomic::{AtomicU64, Ordering}; use tokio::sync::RwLock; use crate::config;
pub struct Telemetry { c: Arc<RwLock<config::Config>>, assimilations: AtomicU64, forecasts: AtomicU64, errors: AtomicU64 }
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self { Self { c, assimilations: AtomicU64::new(0), forecasts: AtomicU64::new(0), errors: AtomicU64::new(0) } }
    pub fn record_assimilation(&self) { self.assimilations.fetch_add(1, Ordering::Relaxed); }
    pub fn record_forecast(&self) { self.forecasts.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value { serde_json::json!({"assimilations": self.assimilations.load(Ordering::Relaxed), "forecasts": self.forecasts.load(Ordering::Relaxed), "errors": self.errors.load(Ordering::Relaxed)}) }
}
