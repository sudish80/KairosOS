use crate::config;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Telemetry {
    config: Arc<RwLock<config::Config>>,
    requests_total: AtomicU64,
    tokens_generated: AtomicU64,
    tokens_draft: AtomicU64,
    tokens_accepted: AtomicU64,
    tokens_rejected: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    latency_total_ns: AtomicU64,
    errors_total: AtomicU64,
}

impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            requests_total: AtomicU64::new(0),
            tokens_generated: AtomicU64::new(0),
            tokens_draft: AtomicU64::new(0),
            tokens_accepted: AtomicU64::new(0),
            tokens_rejected: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            latency_total_ns: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self, tokens: u64, draft: u64, accepted: u64, latency_ns: u64) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.tokens_generated.fetch_add(tokens, Ordering::Relaxed);
        self.tokens_draft.fetch_add(draft, Ordering::Relaxed);
        self.tokens_accepted.fetch_add(accepted, Ordering::Relaxed);
        self.tokens_rejected
            .fetch_add(draft.saturating_sub(accepted), Ordering::Relaxed);
        self.latency_total_ns
            .fetch_add(latency_ns, Ordering::Relaxed);
    }
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn metrics(&self) -> serde_json::Value {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = self.requests_total.load(Ordering::Relaxed);
        let draft = self.tokens_draft.load(Ordering::Relaxed);
        serde_json::json!({
            "requests_total": total,
            "tokens_generated": self.tokens_generated.load(Ordering::Relaxed),
            "tokens_draft": draft,
            "tokens_accepted": self.tokens_accepted.load(Ordering::Relaxed),
            "tokens_rejected": self.tokens_rejected.load(Ordering::Relaxed),
            "acceptance_rate": if draft > 0 { self.tokens_accepted.load(Ordering::Relaxed) as f64 / draft as f64 } else { 0.0 },
            "cache_hit_rate": if hits + misses > 0 { hits as f64 / (hits + misses) as f64 } else { 0.0 },
            "avg_latency_ms": if total > 0 { self.latency_total_ns.load(Ordering::Relaxed) / total / 1_000_000 } else { 0 },
            "errors_total": self.errors_total.load(Ordering::Relaxed),
        })
    }
}
