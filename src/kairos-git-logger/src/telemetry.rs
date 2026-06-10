use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use crate::config;

pub struct Telemetry {
    config: Arc<RwLock<config::Config>>,
    commits_created: AtomicU64,
    files_changed: AtomicU64,
    watcher_events: AtomicU64,
    errors_total: AtomicU64,
    last_commit_timestamp: AtomicU64,
}

impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            commits_created: AtomicU64::new(0),
            files_changed: AtomicU64::new(0),
            watcher_events: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            last_commit_timestamp: AtomicU64::new(0),
        }
    }

    pub fn record_commit(&self, files: u64) {
        self.commits_created.fetch_add(1, Ordering::Relaxed);
        self.files_changed.fetch_add(files, Ordering::Relaxed);
        self.last_commit_timestamp.store(
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            Ordering::Relaxed,
        );
    }
    pub fn record_watcher_event(&self) { self.watcher_events.fetch_add(1, Ordering::Relaxed); }
    pub fn record_error(&self) { self.errors_total.fetch_add(1, Ordering::Relaxed); }

    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "commits_created": self.commits_created.load(Ordering::Relaxed),
            "files_changed": self.files_changed.load(Ordering::Relaxed),
            "watcher_events": self.watcher_events.load(Ordering::Relaxed),
            "errors_total": self.errors_total.load(Ordering::Relaxed),
        })
    }
}
