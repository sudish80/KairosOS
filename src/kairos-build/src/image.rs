use std::sync::Arc; use tokio::sync::RwLock; use crate::config;
pub struct ImageManager { config: Arc<RwLock<config::Config>> }
impl ImageManager {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn package_image(&self) -> anyhow::Result<String> {
        let cfg = self.config.read().await;
        let output = format!("{}/kairos-{}.{}", cfg.image.output_dir, chrono::Utc::now().format("%Y%m%d-%H%M%S"), cfg.image.format);
        tracing::info!("Packaging image: {}", output);
        Ok(output)
    }
}
