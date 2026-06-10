use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use crate::config;

pub struct Telemetry {
    config: Arc<RwLock<config::Config>>,
    frames_rendered: AtomicU64,
    frame_time_ns: AtomicU64,
    input_events: AtomicU64,
    gestures_detected: AtomicU64,
    terminal_writes: AtomicU64,
    errors_total: AtomicU64,
}

impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            frames_rendered: AtomicU64::new(0),
            frame_time_ns: AtomicU64::new(0),
            input_events: AtomicU64::new(0),
            gestures_detected: AtomicU64::new(0),
            terminal_writes: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
        }
    }

    pub fn record_frame(&self, time_ns: u64) {
        self.frames_rendered.fetch_add(1, Ordering::Relaxed);
        self.frame_time_ns.store(time_ns, Ordering::Relaxed);
    }
    pub fn record_input(&self) { self.input_events.fetch_add(1, Ordering::Relaxed); }
    pub fn record_gesture(&self) { self.gestures_detected.fetch_add(1, Ordering::Relaxed); }
    pub fn record_terminal_write(&self) { self.terminal_writes.fetch_add(1, Ordering::Relaxed); }
    pub fn record_error(&self) { self.errors_total.fetch_add(1, Ordering::Relaxed); }

    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "frames_rendered": self.frames_rendered.load(Ordering::Relaxed),
            "frame_time_ms": self.frame_time_ns.load(Ordering::Relaxed) / 1_000_000,
            "input_events": self.input_events.load(Ordering::Relaxed),
            "gestures_detected": self.gestures_detected.load(Ordering::Relaxed),
            "terminal_writes": self.terminal_writes.load(Ordering::Relaxed),
            "errors_total": self.errors_total.load(Ordering::Relaxed),
            "fps": 1_000_000_000u64 / self.frame_time_ns.load(Ordering::Relaxed).max(1),
        })
    }
}
