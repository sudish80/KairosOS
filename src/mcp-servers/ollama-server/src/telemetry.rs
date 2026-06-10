use std::sync::atomic::{AtomicU64, Ordering};

pub struct Telemetry {
    requests: AtomicU64,
    generate_ops: AtomicU64,
    list_model_ops: AtomicU64,
    errors: AtomicU64,
    tokens_generated: AtomicU64,
    total_latency_ms: AtomicU64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            requests: AtomicU64::new(0),
            generate_ops: AtomicU64::new(0),
            list_model_ops: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            tokens_generated: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        }
    }
    pub fn record_request(&self) { self.requests.fetch_add(1, Ordering::Relaxed); }
    pub fn record_generate(&self, tokens: u64, latency_ms: u64) { self.generate_ops.fetch_add(1, Ordering::Relaxed); self.tokens_generated.fetch_add(tokens, Ordering::Relaxed); self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed); }
    pub fn record_list_models(&self) { self.list_model_ops.fetch_add(1, Ordering::Relaxed); }
    pub fn record_error(&self) { self.errors.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "total_requests": self.requests.load(Ordering::Relaxed),
            "generate_ops": self.generate_ops.load(Ordering::Relaxed),
            "list_model_ops": self.list_model_ops.load(Ordering::Relaxed),
            "errors": self.errors.load(Ordering::Relaxed),
            "tokens_generated": self.tokens_generated.load(Ordering::Relaxed),
            "avg_latency_ms": if self.generate_ops.load(Ordering::Relaxed) > 0 { self.total_latency_ms.load(Ordering::Relaxed) / self.generate_ops.load(Ordering::Relaxed) } else { 0 },
        })
    }
}
