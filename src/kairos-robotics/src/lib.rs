#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod control; pub mod kinematics; pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub controller: std::sync::Arc<control::MotorController>,
    pub kinematics: std::sync::Arc<kinematics::KinematicsEngine>,
}
impl AppState { pub async fn new(cfg: config::Config) -> anyhow::Result<Self> { let c=std::sync::Arc::new(tokio::sync::RwLock::new(cfg)); let t=std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&c))); let m=std::sync::Arc::new(control::MotorController::new(std::sync::Arc::clone(&c))); let k=std::sync::Arc::new(kinematics::KinematicsEngine::new(std::sync::Arc::clone(&c))); tracing::info!("kairos-robotics AppState initialized"); Ok(Self{config:c,telemetry:t,controller:m,kinematics:k}) }
}
