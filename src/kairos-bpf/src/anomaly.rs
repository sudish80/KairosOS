//! Anomaly detector using statistical and ML-based methods
use crate::error::Result;
use crate::telemetry::TelemetryStore;
use crate::config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;
use tracing::{info, debug, warn};

pub struct AnomalyDetector {
    telemetry: Arc<TelemetryStore>,
    config: Arc<RwLock<Config>>,
    syscall_history: Arc<RwLock<VecDeque<SyscallSample>>>,
    memory_history: Arc<RwLock<VecDeque<MemorySample>>>,
    network_history: Arc<RwLock<VecDeque<NetworkSample>>>,
}

#[derive(Debug, Clone)]
struct SyscallSample {
    timestamp: std::time::Instant,
    pid: u32,
    count: u64,
}

#[derive(Debug, Clone)]
struct MemorySample {
    timestamp: std::time::Instant,
    pid: u32,
    bytes: u64,
}

#[derive(Debug, Clone)]
struct NetworkSample {
    timestamp: std::time::Instant,
    pid: u32,
    bytes_sent: u64,
    bytes_recv: u64,
}

impl AnomalyDetector {
    pub fn new(telemetry: Arc<TelemetryStore>, config: Arc<RwLock<Config>>) -> Self {
        Self {
            telemetry,
            config,
            syscall_history: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            memory_history: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            network_history: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
        }
    }

    pub async fn detection_loop(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            if let Err(e) = self.detect_anomalies().await {
                warn!("Anomaly detection failed: {}", e);
            }
        }
    }

    async fn detect_anomalies(&self) -> Result<()> {
        let cfg = self.config.read().await;
        if !cfg.anomaly.enabled {
            return Ok(());
        }

        self.detect_syscall_anomalies(&cfg).await?;
        self.detect_memory_anomalies(&cfg).await?;
        self.detect_network_anomalies(&cfg).await?;

        Ok(())
    }

    async fn detect_syscall_anomalies(&self, cfg: &Config) -> Result<()> {
        // Statistical anomaly detection on syscall rates
        let history = self.syscall_history.read().await;
        if history.len() < cfg.anomaly.min_samples {
            return Ok(());
        }

        // Calculate mean and stddev
        let rates: Vec<f64> = history.iter().map(|s| s.count as f64).collect();
        let mean = rates.iter().sum::<f64>() / rates.len() as f64;
        let variance = rates.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / rates.len() as f64;
        let stddev = variance.sqrt();

        // Check recent samples
        for sample in history.iter().rev().take(10) {
            let zscore = (sample.count as f64 - mean) / stddev.max(1.0);
            if zscore > cfg.anomaly.stddev_threshold {
                warn!("Syscall anomaly detected: pid={} zscore={:.2}", sample.pid, zscore);
                // Emit anomaly event
            }
        }

        Ok(())
    }

    async fn detect_memory_anomalies(&self, cfg: &Config) -> Result<()> {
        let history = self.memory_history.read().await;
        if history.len() < cfg.anomaly.min_samples {
            return Ok(());
        }

        let rates: Vec<f64> = history.iter().map(|s| s.bytes as f64).collect();
        let mean = rates.iter().sum::<f64>() / rates.len() as f64;
        let variance = rates.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / rates.len() as f64;
        let stddev = variance.sqrt();

        for sample in history.iter().rev().take(10) {
            let zscore = (sample.bytes as f64 - mean) / stddev.max(1.0);
            if zscore > cfg.anomaly.stddev_threshold {
                warn!("Memory anomaly detected: pid={} zscore={:.2}", sample.pid, zscore);
            }
        }

        Ok(())
    }

    async fn detect_network_anomalies(&self, cfg: &Config) -> Result<()> {
        let history = self.network_history.read().await;
        if history.len() < cfg.anomaly.min_samples {
            return Ok(());
        }

        let total_bytes: Vec<f64> = history.iter()
            .map(|s| (s.bytes_sent + s.bytes_recv) as f64)
            .collect();
        let mean = total_bytes.iter().sum::<f64>() / total_bytes.len() as f64;
        let variance = total_bytes.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / total_bytes.len() as f64;
        let stddev = variance.sqrt();

        for sample in history.iter().rev().take(10) {
            let total = (sample.bytes_sent + sample.bytes_recv) as f64;
            let zscore = (total - mean) / stddev.max(1.0);
            if zscore > cfg.anomaly.stddev_threshold {
                warn!("Network anomaly detected: pid={} zscore={:.2}", sample.pid, zscore);
            }
        }

        Ok(())
    }

    pub async fn record_syscall(&self, pid: u32, count: u64) {
        let mut history = self.syscall_history.write().await;
        history.push_back(SyscallSample {
            timestamp: std::time::Instant::now(),
            pid,
            count,
        });
        if history.len() > 10000 { history.pop_front(); }
    }

    pub async fn record_memory(&self, pid: u32, bytes: u64) {
        let mut history = self.memory_history.write().await;
        history.push_back(MemorySample {
            timestamp: std::time::Instant::now(),
            pid,
            bytes,
        });
        if history.len() > 10000 { history.pop_front(); }
    }

    pub async fn record_network(&self, pid: u32, bytes_sent: u64, bytes_recv: u64) {
        let mut history = self.network_history.write().await;
        history.push_back(NetworkSample {
            timestamp: std::time::Instant::now(),
            pid,
            bytes_sent,
            bytes_recv,
        });
        if history.len() > 10000 { history.pop_front(); }
    }
}