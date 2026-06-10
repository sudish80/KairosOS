#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod gate; pub mod simulator; pub mod pqc; pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub gate_engine: std::sync::Arc<gate::GateEngine>,
    pub simulator: std::sync::Arc<simulator::QuantumSimulator>,
}
impl AppState { pub async fn new(cfg: config::Config) -> anyhow::Result<Self> { let c=std::sync::Arc::new(tokio::sync::RwLock::new(cfg)); let t=std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&c))); let g=std::sync::Arc::new(gate::GateEngine::new(std::sync::Arc::clone(&c))); let s=std::sync::Arc::new(simulator::QuantumSimulator::new(std::sync::Arc::clone(&c))); tracing::info!("kairos-quantum AppState initialized"); Ok(Self{config:c,telemetry:t,gate_engine:g,simulator:s}) }
}
