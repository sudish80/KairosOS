use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct Telemetry {
    c: Arc<RwLock<config::Config>>,
    sequences: AtomicU64,
    alignments: AtomicU64,
    bases_processed: AtomicU64,
    errors: AtomicU64,
}
impl Telemetry {
    pub fn new(c: Arc<RwLock<config::Config>>) -> Self {
        Self {
            c,
            sequences: AtomicU64::new(0),
            alignments: AtomicU64::new(0),
            bases_processed: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
    pub fn record_sequence(&self, bases: u64) {
        self.sequences.fetch_add(1, Ordering::Relaxed);
        self.bases_processed.fetch_add(bases, Ordering::Relaxed);
    }
    pub fn record_alignment(&self) {
        self.alignments.fetch_add(1, Ordering::Relaxed);
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"sequences_processed": self.sequences.load(Ordering::Relaxed), "alignments_completed": self.alignments.load(Ordering::Relaxed), "bases_processed": self.bases_processed.load(Ordering::Relaxed), "errors": self.errors.load(Ordering::Relaxed)})
    }
}
