use std::sync::atomic::{AtomicU64, Ordering};

pub struct Telemetry {
    requests: AtomicU64,
    list_ops: AtomicU64,
    status_ops: AtomicU64,
    action_ops: AtomicU64,
    journal_ops: AtomicU64,
    errors: AtomicU64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            requests: AtomicU64::new(0),
            list_ops: AtomicU64::new(0),
            status_ops: AtomicU64::new(0),
            action_ops: AtomicU64::new(0),
            journal_ops: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_list(&self) {
        self.list_ops.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_status(&self) {
        self.status_ops.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_action(&self) {
        self.action_ops.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_journal(&self) {
        self.journal_ops.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "total_requests": self.requests.load(Ordering::Relaxed),
            "list_ops": self.list_ops.load(Ordering::Relaxed),
            "status_ops": self.status_ops.load(Ordering::Relaxed),
            "action_ops": self.action_ops.load(Ordering::Relaxed),
            "journal_ops": self.journal_ops.load(Ordering::Relaxed),
            "errors": self.errors.load(Ordering::Relaxed),
        })
    }
}
