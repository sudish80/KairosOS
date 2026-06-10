use std::sync::atomic::{AtomicU64, Ordering};

pub struct Telemetry {
    requests: AtomicU64,
    list_ops: AtomicU64,
    signal_ops: AtomicU64,
    errors: AtomicU64,
    processes_returned: AtomicU64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            requests: AtomicU64::new(0),
            list_ops: AtomicU64::new(0),
            signal_ops: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            processes_returned: AtomicU64::new(0),
        }
    }
    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_list(&self, count: u64) {
        self.list_ops.fetch_add(1, Ordering::Relaxed);
        self.processes_returned.fetch_add(count, Ordering::Relaxed);
    }
    pub fn record_signal(&self) {
        self.signal_ops.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "total_requests": self.requests.load(Ordering::Relaxed),
            "list_ops": self.list_ops.load(Ordering::Relaxed),
            "signal_ops": self.signal_ops.load(Ordering::Relaxed),
            "errors": self.errors.load(Ordering::Relaxed),
            "processes_returned": self.processes_returned.load(Ordering::Relaxed),
        })
    }
}
