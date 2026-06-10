#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod canvas; pub mod drm; pub mod compositor;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub canvas: std::sync::Arc<canvas::Canvas>,
    pub drm_manager: std::sync::Arc<drm::DrmManager>,
    pub compositor: std::sync::Arc<compositor::Compositor>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry = std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let canvas = std::sync::Arc::new(canvas::Canvas::new(std::sync::Arc::clone(&config)));
        let drm_manager = std::sync::Arc::new(drm::DrmManager::new(std::sync::Arc::clone(&config)));
        let compositor = std::sync::Arc::new(compositor::Compositor::new(std::sync::Arc::clone(&config), std::sync::Arc::clone(&canvas)));
        tracing::info!("kairos-fb AppState initialized");
        Ok(Self { config, telemetry, canvas, drm_manager, compositor })
    }
}
