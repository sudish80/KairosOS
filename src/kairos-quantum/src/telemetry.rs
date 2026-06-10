use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Telemetry {
    c: Arc<RwLock<config::Config>>,
    circuits: AtomicU64,
    gates: AtomicU64,
    shots: AtomicU64,
    errors: AtomicU64,
}
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self {
        Self {
            c,
            circuits: AtomicU64::new(0),
            gates: AtomicU64::new(0),
            shots: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
    pub fn record_circuit(&self, g: u64, s: u64) {
        self.circuits.fetch_add(1, Ordering::Relaxed);
        self.gates.fetch_add(g, Ordering::Relaxed);
        self.shots.fetch_add(s, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"circuits_executed": self.circuits.load(Ordering::Relaxed), "gates_applied": self.gates.load(Ordering::Relaxed), "shots_taken": self.shots.load(Ordering::Relaxed), "errors": self.errors.load(Ordering::Relaxed)})
    }
}
