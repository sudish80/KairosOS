use std::sync::Arc; use std::sync::atomic::{AtomicU64, Ordering}; use tokio::sync::RwLock; use crate::config;
pub struct Telemetry { c: Arc<RwLock<config::Config>>, control_loops: AtomicU64, movements: AtomicU64, errors: AtomicU64 }
impl Telemetry { pub fn new(c: Arc<RwLock<config::Config>>) -> Self { Self{c, control_loops: AtomicU64::new(0), movements: AtomicU64::new(0), errors: AtomicU64::new(0)} }
    pub fn record_loop(&self) { self.control_loops.fetch_add(1, Ordering::Relaxed); } pub fn record_movement(&self) { self.movements.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value { serde_json::json!({"control_loops": self.control_loops.load(Ordering::Relaxed), "movements_executed": self.movements.load(Ordering::Relaxed), "errors": self.errors.load(Ordering::Relaxed)}) }
}
