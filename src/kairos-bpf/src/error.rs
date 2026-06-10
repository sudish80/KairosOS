//! Error types for kairos-bpf
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("eBPF program load failed: {0}")]
    ProgramLoad(String),
    #[error("eBPF map operation failed: {0}")]
    MapOp(String),
    #[error("Ring buffer error: {0}")]
    RingBuffer(String),
    #[error("Perf event error: {0}")]
    PerfEvent(String),
    #[error("Remediation action failed: {0}")]
    Remediation(String),
    #[error("Policy evaluation failed: {0}")]
    Policy(String),
    #[error("Anomaly detection failed: {0}")]
    Anomaly(String),
    #[error("Thermal governor error: {0}")]
    Thermal(String),
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    #[error("Telemetry error: {0}")]
    Telemetry(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
