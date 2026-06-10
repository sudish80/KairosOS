#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod builder; pub mod image; pub mod package;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub builder: std::sync::Arc<builder::ImageBuilder>,
    pub image_manager: std::sync::Arc<image::ImageManager>,
    pub package_manager: std::sync::Arc<package::PackageManager>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry = std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let builder = std::sync::Arc::new(builder::ImageBuilder::new(std::sync::Arc::clone(&config)));
        let image_manager = std::sync::Arc::new(image::ImageManager::new(std::sync::Arc::clone(&config)));
        let package_manager = std::sync::Arc::new(package::PackageManager::new(std::sync::Arc::clone(&config)));
        tracing::info!("kairos-build AppState initialized");
        Ok(Self { config, telemetry, builder, image_manager, package_manager })
    }
}
