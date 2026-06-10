#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod arinc; pub mod mavlink; pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub arinc: std::sync::Arc<arinc::Arinc429>,
    pub mavlink: std::sync::Arc<mavlink::MavlinkBus>,
}
impl AppState { pub async fn new(cfg: config::Config) -> anyhow::Result<Self> { let c=std::sync::Arc::new(tokio::sync::RwLock::new(cfg)); let t=std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&c))); let a=std::sync::Arc::new(arinc::Arinc429::new(std::sync::Arc::clone(&c))); let m=std::sync::Arc::new(mavlink::MavlinkBus::new(std::sync::Arc::clone(&c))); tracing::info!("kairos-avionics AppState initialized"); Ok(Self{config:c,telemetry:t,arinc:a,mavlink:m}) }
}
