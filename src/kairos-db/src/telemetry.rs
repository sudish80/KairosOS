use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Telemetry {
    _config: Arc<RwLock<config::Config>>,
    queries: AtomicU64,
    vectors_indexed: AtomicU64,
    bus_msgs: AtomicU64,
    cache_hits: AtomicU64,
    errors_total: AtomicU64,
}
impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            _config: config,
            queries: AtomicU64::new(0),
            vectors_indexed: AtomicU64::new(0),
            bus_msgs: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
        }
    }
    pub fn record_query(&self) {
        self.queries.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_vector(&self) {
        self.vectors_indexed.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_bus_msg(&self) {
        self.bus_msgs.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"queries": self.queries.load(Ordering::Relaxed), "vectors_indexed": self.vectors_indexed.load(Ordering::Relaxed), "bus_messages": self.bus_msgs.load(Ordering::Relaxed), "cache_hit_rate": 0.0, "errors_total": self.errors_total.load(Ordering::Relaxed)})
    }
}
