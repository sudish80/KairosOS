use std::sync::Arc; use std::sync::atomic::{AtomicU64, Ordering}; use tokio::sync::RwLock; use crate::config;
pub struct Telemetry { config: Arc<RwLock<config::Config>>, builds: AtomicU64, images: AtomicU64, packages: AtomicU64, errors_total: AtomicU64 }
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self { Self { config: c, builds: AtomicU64::new(0), images: AtomicU64::new(0), packages: AtomicU64::new(0), errors_total: AtomicU64::new(0) } }
    pub fn record_build(&self) { self.builds.fetch_add(1, Ordering::Relaxed); }
    pub fn record_image(&self) { self.images.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value { serde_json::json!({"builds": self.builds.load(Ordering::Relaxed), "images": self.images.load(Ordering::Relaxed), "packages": self.packages.load(Ordering::Relaxed), "errors_total": self.errors_total.load(Ordering::Relaxed)}) }
}
