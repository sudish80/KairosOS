use std::sync::Arc; use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock; use crate::config;
pub struct Telemetry {
    config: Arc<RwLock<config::Config>>,
    pipelines_submitted: AtomicU64, pipelines_completed: AtomicU64, tasks_executed: AtomicU64,
    errors_total: AtomicU64, avg_execution_time_ns: AtomicU64,
}
impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config, pipelines_submitted: AtomicU64::new(0), pipelines_completed: AtomicU64::new(0),
            tasks_executed: AtomicU64::new(0), errors_total: AtomicU64::new(0), avg_execution_time_ns: AtomicU64::new(0) }
    }
    pub fn record_submission(&self) { self.pipelines_submitted.fetch_add(1, Ordering::Relaxed); }
    pub fn record_completion(&self) { self.pipelines_completed.fetch_add(1, Ordering::Relaxed); }
    pub fn record_task(&self) { self.tasks_executed.fetch_add(1, Ordering::Relaxed); }
    pub fn record_error(&self) { self.errors_total.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "pipelines_submitted": self.pipelines_submitted.load(Ordering::Relaxed),
            "pipelines_completed": self.pipelines_completed.load(Ordering::Relaxed),
            "tasks_executed": self.tasks_executed.load(Ordering::Relaxed),
            "errors_total": self.errors_total.load(Ordering::Relaxed),
        })
    }
}
