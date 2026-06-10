use std::sync::Arc; use std::sync::atomic::{AtomicU64, Ordering}; use tokio::sync::RwLock; use crate::config;
pub struct Telemetry { c: Arc<RwLock<config::Config>>, frames: AtomicU64, detections: AtomicU64, fps: AtomicU64, errors: AtomicU64 }
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self { Self { c, frames: AtomicU64::new(0), detections: AtomicU64::new(0), fps: AtomicU64::new(0), errors: AtomicU64::new(0) } }
    pub fn record_frame(&self) { self.frames.fetch_add(1, Ordering::Relaxed); }
    pub fn record_detection(&self) { self.detections.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value { serde_json::json!({"frames_processed": self.frames.load(Ordering::Relaxed), "detections": self.detections.load(Ordering::Relaxed), "errors": self.errors.load(Ordering::Relaxed)}) }
}
