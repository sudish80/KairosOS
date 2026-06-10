//! Metrics collection and exposition for Prometheus
use crate::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

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
