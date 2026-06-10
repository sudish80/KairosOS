use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct Telemetry {
    requests: AtomicU64,
    read_ops: AtomicU64,
    write_ops: AtomicU64,
    list_ops: AtomicU64,
    stat_ops: AtomicU64,
    errors: AtomicU64,
    bytes_read: AtomicU64,
    bytes_written: AtomicU64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_read(&self, bytes: u64) {
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        self.bytes_read.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn record_write(&self, bytes: u64) {
        self.write_ops.fetch_add(1, Ordering::Relaxed);
        self.bytes_written.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn record_list(&self) {
        self.list_ops.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_stat(&self) {
        self.stat_ops.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "total_requests": self.requests.load(Ordering::Relaxed),
            "read_ops": self.read_ops.load(Ordering::Relaxed),
            "write_ops": self.write_ops.load(Ordering::Relaxed),
            "list_ops": self.list_ops.load(Ordering::Relaxed),
            "stat_ops": self.stat_ops.load(Ordering::Relaxed),
            "errors": self.errors.load(Ordering::Relaxed),
            "bytes_read": self.bytes_read.load(Ordering::Relaxed),
            "bytes_written": self.bytes_written.load(Ordering::Relaxed),
        })
    }
}
