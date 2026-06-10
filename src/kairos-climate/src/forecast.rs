use crate::config;
use crate::model::ClimateModel;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct ForecastEngine {
    config: Arc<RwLock<config::Config>>,
    model: Arc<ClimateModel>,
}
impl ForecastEngine {
    pub fn new(config: Arc<RwLock<config::Config>>, model: Arc<ClimateModel>) -> Self {
        Self { config, model }
    }
    pub async fn run_forecast(&self, _days: u32) -> anyhow::Result<()> {
        info!("Running {} day forecast", _days);
        Ok(())
    }
}
