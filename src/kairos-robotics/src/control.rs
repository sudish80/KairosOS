use crate::config;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
pub struct MotorController {
    config: Arc<RwLock<config::Config>>,
    target_pos: Arc<RwLock<f64>>,
    current_pos: Arc<RwLock<f64>>,
}
impl MotorController {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            target_pos: Arc::new(RwLock::new(0.0)),
            current_pos: Arc::new(RwLock::new(0.0)),
        }
    }
    pub async fn set_target(&self, pos: f64) {
        *self.target_pos.write().await = pos;
    }
    pub async fn pwm_write(&self, duty: f64) -> anyhow::Result<()> {
        let dev = self.config.read().await.control.pwm_device.clone();
        fs::write(&dev, format!("{}", (duty * 255.0) as u8)).await?;
        Ok(())
    }
}
