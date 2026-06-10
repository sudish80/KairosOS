use std::sync::Arc; use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock; use crate::config;
pub struct Telemetry { config: Arc<RwLock<config::Config>>, peers_connected: AtomicU64, bytes_sent: AtomicU64, bytes_recv: AtomicU64, discovery_events: AtomicU64, errors_total: AtomicU64 }
impl Telemetry {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config, peers_connected: AtomicU64::new(0), bytes_sent: AtomicU64::new(0), bytes_recv: AtomicU64::new(0), discovery_events: AtomicU64::new(0), errors_total: AtomicU64::new(0) } }
    pub fn record_peer_connect(&self) { self.peers_connected.fetch_add(1, Ordering::Relaxed); }
    pub fn record_tx(&self, bytes: u64) { self.bytes_sent.fetch_add(bytes, Ordering::Relaxed); }
    pub fn record_rx(&self, bytes: u64) { self.bytes_recv.fetch_add(bytes, Ordering::Relaxed); }
    pub fn record_discovery(&self) { self.discovery_events.fetch_add(1, Ordering::Relaxed); }
    pub fn record_error(&self) { self.errors_total.fetch_add(1, Ordering::Relaxed); }
    pub fn metrics(&self) -> serde_json::Value { serde_json::json!({"peers_connected": self.peers_connected.load(Ordering::Relaxed), "bytes_sent": self.bytes_sent.load(Ordering::Relaxed), "bytes_recv": self.bytes_recv.load(Ordering::Relaxed), "discovery_events": self.discovery_events.load(Ordering::Relaxed), "errors_total": self.errors_total.load(Ordering::Relaxed)}) }
}
