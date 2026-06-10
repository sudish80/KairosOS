use std::sync::Arc; use tokio::sync::RwLock; use tokio::process::Command;
use tracing::{info, error}; use crate::config;
pub struct ImageBuilder { config: Arc<RwLock<config::Config>> }
impl ImageBuilder {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn build(&self) -> anyhow::Result<()> {
        let cfg = self.config.read().await;
        let status = Command::new("make")
            .args(["-C", &cfg.build.buildroot_dir, &format!("O={}", cfg.general.workspace),
                   &format!("BR2_EXTERNAL={}", cfg.build.config_file), &format!("-j{}", cfg.build.jobs)])
            .status().await.map_err(|e| anyhow::anyhow!("make failed: {}", e))?;
        if status.success() { info!("Buildroot build completed"); Ok(()) }
        else { Err(anyhow::anyhow!("Buildroot build failed with status {:?}", status.code())) }
    }
}
