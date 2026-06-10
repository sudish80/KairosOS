#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod wg; pub mod discovery; pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub wg_manager: std::sync::Arc<wg::WireGuardManager>,
    pub discovery: std::sync::Arc<discovery::NodeDiscovery>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry = std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let wg_manager = std::sync::Arc::new(wg::WireGuardManager::new(std::sync::Arc::clone(&config)));
        let discovery = std::sync::Arc::new(discovery::NodeDiscovery::new(std::sync::Arc::clone(&config)));
        tracing::info!("kairos-mesh AppState initialized");
        Ok(Self { config, telemetry, wg_manager, discovery })
    }
}
