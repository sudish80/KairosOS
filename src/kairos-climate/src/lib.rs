#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod assimilation; pub mod model; pub mod forecast;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub assimilation: std::sync::Arc<assimilation::DataAssimilation>,
    pub model_engine: std::sync::Arc<model::ClimateModel>,
    pub forecast_engine: std::sync::Arc<forecast::ForecastEngine>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry = std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let assimilation = std::sync::Arc::new(assimilation::DataAssimilation::new(std::sync::Arc::clone(&config)));
        let model_engine = std::sync::Arc::new(model::ClimateModel::new(std::sync::Arc::clone(&config)));
        let forecast_engine = std::sync::Arc::new(forecast::ForecastEngine::new(std::sync::Arc::clone(&config), std::sync::Arc::clone(&model_engine)));
        tracing::info!("kairos-climate AppState initialized");
        Ok(Self { config, telemetry, assimilation, model_engine, forecast_engine })
    }
}
