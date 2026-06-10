use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct RiskManager {
    config: Arc<RwLock<config::Config>>,
}
impl RiskManager {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn check_order(&self, _qty: f64, _price: f64) -> bool {
        let cfg = self.config.read().await;
        (_qty * _price) <= cfg.risk.position_limit as f64
    }
}
