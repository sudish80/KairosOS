//! Metrics collection and exposition for Prometheus
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::telemetry::Telemetry;

pub struct MetricsExporter {
    telemetry: Arc<Telemetry>,
}

impl MetricsExporter {
    pub fn new(telemetry: Arc<Telemetry>) -> Self {
        Self { telemetry }
    }

    pub fn collect(&self) -> serde_json::Value {
        self.telemetry.metrics()
    }
}
