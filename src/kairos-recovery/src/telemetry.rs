use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use crate::config;

pub struct Telemetry {
    config: Arc<RwLock<config::Config>>,
    updates_applied: AtomicU64,
    rollbacks_performed: AtomicU64,
    verity_checks: AtomicU64,
    verity_failures: AtomicU64,
    health_checks: AtomicU64,
    health_failures: AtomicU64,
    errors_total: AtomicU64,
}

impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            updates_applied: AtomicU64::new(0),
            rollbacks_performed: AtomicU64::new(0),
            verity_checks: AtomicU64::new(0),
            verity_failures: AtomicU64::new(0),
            health_checks: AtomicU64::new(0),
            health_failures: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
        }
    }

    pub fn record_update(&self) { self.updates_applied.fetch_add(1, Ordering::Relaxed); }
    pub fn record_rollback(&self) { self.rollbacks_performed.fetch_add(1, Ordering::Relaxed); }
    pub fn record_verity_check(&self, passed: bool) {
        self.verity_checks.fetch_add(1, Ordering::Relaxed);
        if !passed { self.verity_failures.fetch_add(1, Ordering::Relaxed); }
    }
    pub fn record_health(&self, passed: bool) {
        self.health_checks.fetch_add(1, Ordering::Relaxed);
        if !passed { self.health_failures.fetch_add(1, Ordering::Relaxed); }
    }
    pub fn record_update_check(&self, _available: bool) {
        self.verity_checks.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) { self.errors_total.fetch_add(1, Ordering::Relaxed); }

    pub fn updates_applied(&self) -> u64 { self.updates_applied.load(Ordering::Relaxed) }
    pub fn rollbacks_performed(&self) -> u64 { self.rollbacks_performed.load(Ordering::Relaxed) }
    pub fn verity_checks(&self) -> u64 { self.verity_checks.load(Ordering::Relaxed) }
    pub fn verity_failures(&self) -> u64 { self.verity_failures.load(Ordering::Relaxed) }
    pub fn health_checks(&self) -> u64 { self.health_checks.load(Ordering::Relaxed) }
    pub fn health_failures(&self) -> u64 { self.health_failures.load(Ordering::Relaxed) }
    pub fn errors_total(&self) -> u64 { self.errors_total.load(Ordering::Relaxed) }

    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "updates_applied": self.updates_applied.load(Ordering::Relaxed),
            "rollbacks_performed": self.rollbacks_performed.load(Ordering::Relaxed),
            "verity_checks": self.verity_checks.load(Ordering::Relaxed),
            "verity_failures": self.verity_failures.load(Ordering::Relaxed),
            "health_checks": self.health_checks.load(Ordering::Relaxed),
            "health_failures": self.health_failures.load(Ordering::Relaxed),
            "errors_total": self.errors_total.load(Ordering::Relaxed),
        })
    }
}
