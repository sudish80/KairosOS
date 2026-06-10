//! kairos-bpf: eBPF telemetry and autonomous remediation daemon — production-hardened
//! Implements all 25 items of Subsystem 1: eBPF Subsystem (items 1-25)
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

pub mod config;
pub mod error;
pub mod telemetry;
pub mod worker;
pub mod programs;
pub mod maps;
pub mod policy;
pub mod remediation;
pub mod anomaly;
pub mod scheduler;
pub mod thermal;
pub mod heal;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug, instrument};

/// Core application state for eBPF daemon
pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::TelemetryStore>,
    pub policy_engine: Arc<policy::PolicyEngine>,
    pub remediation: Arc<remediation::RemediationEngine>,
    pub anomaly_detector: Arc<anomaly::AnomalyDetector>,
    pub scheduler: Arc<scheduler::Scheduler>,
    pub thermal_governor: Arc<thermal::ThermalGovernor>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::TelemetryStore::new()?);
        let policy_engine = Arc::new(policy::PolicyEngine::new(Arc::clone(&telemetry)));
        let remediation = Arc::new(remediation::RemediationEngine::new(Arc::clone(&telemetry), Arc::clone(&config)).await?);
        let anomaly_detector = Arc::new(anomaly::AnomalyDetector::new(Arc::clone(&telemetry), Arc::clone(&config)));
        let scheduler = Arc::new(scheduler::Scheduler::new(Arc::clone(&config)));
        let thermal_governor = Arc::new(thermal::ThermalGovernor::new(Arc::clone(&config), Arc::clone(&telemetry)));

        info!("kairos-bpf AppState initialized");
        Ok(Self { config, telemetry, policy_engine, remediation, anomaly_detector, scheduler, thermal_governor })
    }

    /// Start all background loops
    pub async fn start(&self) -> anyhow::Result<()> {
        // Start telemetry ingestion
        let telemetry = Arc::clone(&self.telemetry);
        tokio::spawn(async move { telemetry.ingestion_loop().await });

        // Start policy evaluation
        let policy = Arc::clone(&self.policy_engine);
        tokio::spawn(async move { policy.evaluation_loop().await });

        // Start anomaly detection
        let anomaly = Arc::clone(&self.anomaly_detector);
        tokio::spawn(async move { anomaly.detection_loop().await });

        // Start scheduler
        let sched = Arc::clone(&self.scheduler);
        tokio::spawn(async move { sched.scheduling_loop().await });

        // Start thermal governor
        let thermal = Arc::clone(&self.thermal_governor);
        tokio::spawn(async move { thermal.governor_loop().await });

        // Start remediation engine
        let remediation = Arc::clone(&self.remediation);
        tokio::spawn(async move { remediation.remediation_loop().await });

        info!("All background loops started");
        Ok(())
    }
}

/// Initialize all eBPF programs and maps
pub async fn initialize_ebpf(state: &AppState) -> anyhow::Result<()> {
    // Load all 6 core BPF programs
    programs::execsnoop::load()?;
    programs::tcptop::load()?;
    programs::filemon::load()?;
    programs::anomaly::load()?;
    programs::schedlatency::load()?;
    programs::oomkill::load()?;

    // Initialize ring buffer maps
    maps::init_ring_buffers()?;

    // Initialize percpu arrays
    maps::init_percpu_arrays()?;

    // Initialize hash maps for tracking
    maps::init_hash_maps()?;

    info!("All eBPF programs and maps initialized");
    Ok(())
}

/// Main entry for the daemon
pub async fn run(state: AppState) -> anyhow::Result<()> {
    initialize_ebpf(&state).await?;
    state.start().await?;

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let cfg = config::Config::default();
        let state = AppState::new(cfg).await.unwrap();
        assert!(state.telemetry.is_healthy());
    }

    #[tokio::test]
    async fn test_ebpf_initialization() {
        let cfg = config::Config::default();
        let state = AppState::new(cfg).await.unwrap();
        initialize_ebpf(&state).await.unwrap();
    }
}
