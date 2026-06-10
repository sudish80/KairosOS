use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Telemetry {
    c: Arc<RwLock<config::Config>>,
    msgs_rx: AtomicU64,
    msgs_tx: AtomicU64,
    errors: AtomicU64,
}
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self {
        Self {
            c,
            msgs_rx: AtomicU64::new(0),
            msgs_tx: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
    pub fn record_rx(&self) {
        self.msgs_rx.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_tx(&self) {
        self.msgs_tx.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"messages_received": self.msgs_rx.load(Ordering::Relaxed), "messages_transmitted": self.msgs_tx.load(Ordering::Relaxed), "errors": self.errors.load(Ordering::Relaxed)})
    }
}
