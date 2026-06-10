use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Telemetry {
    c: Arc<RwLock<config::Config>>,
    ticks_recv: AtomicU64,
    orders: AtomicU64,
    pnl: AtomicU64,
    errors: AtomicU64,
}
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self {
        Self {
            c,
            ticks_recv: AtomicU64::new(0),
            orders: AtomicU64::new(0),
            pnl: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
    pub fn record_tick(&self) {
        self.ticks_recv.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_order(&self) {
        self.orders.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"ticks_received": self.ticks_recv.load(Ordering::Relaxed), "orders_executed": self.orders.load(Ordering::Relaxed), "errors": self.errors.load(Ordering::Relaxed)})
    }
}
