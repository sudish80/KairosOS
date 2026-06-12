use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Telemetry {
    _config: Arc<RwLock<config::Config>>,
    frames_rendered: AtomicU64,
    vsync_count: AtomicU64,
    errors_total: AtomicU64,
}
impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            _config: config,
            frames_rendered: AtomicU64::new(0),
            vsync_count: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
        }
    }
    pub fn record_frame(&self) {
        self.frames_rendered.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_vsync(&self) {
        self.vsync_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"frames_rendered": self.frames_rendered.load(Ordering::Relaxed), "vsync_count": self.vsync_count.load(Ordering::Relaxed), "errors_total": self.errors_total.load(Ordering::Relaxed)})
    }
}
