use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Telemetry {
    _config: Arc<RwLock<config::Config>>,
    generations_created: AtomicU64,
    generations_applied: AtomicU64,
    rollbacks_triggered: AtomicU64,
    errors_total: AtomicU64,
    last_apply_duration_ns: AtomicU64,
    last_apply_success: AtomicU64,
}

impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            _config: config,
            generations_created: AtomicU64::new(0),
            generations_applied: AtomicU64::new(0),
            rollbacks_triggered: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            last_apply_duration_ns: AtomicU64::new(0),
            last_apply_success: AtomicU64::new(1),
        }
    }

    pub fn record_generation_created(&self) {
        self.generations_created.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_apply(&self, success: bool, duration_ns: u64) {
        self.generations_applied.fetch_add(1, Ordering::Relaxed);
        self.last_apply_duration_ns
            .store(duration_ns, Ordering::Relaxed);
        self.last_apply_success
            .store(success as u64, Ordering::Relaxed);
    }
    pub fn record_rollback(&self) {
        self.rollbacks_triggered.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "generations_created": self.generations_created.load(Ordering::Relaxed),
            "generations_applied": self.generations_applied.load(Ordering::Relaxed),
            "rollbacks_triggered": self.rollbacks_triggered.load(Ordering::Relaxed),
            "errors_total": self.errors_total.load(Ordering::Relaxed),
            "last_apply_duration_ms": self.last_apply_duration_ns.load(Ordering::Relaxed) / 1_000_000,
            "last_apply_success": self.last_apply_success.load(Ordering::Relaxed) == 1,
        })
    }
}
