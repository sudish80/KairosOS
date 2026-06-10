#![deny(unsafe_code)]
pub mod config;
pub mod error;
pub mod market;
pub mod risk;
pub mod telemetry;
pub mod trading;
pub mod worker;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub market: std::sync::Arc<market::MarketFeed>,
    pub trading: std::sync::Arc<trading::TradingEngine>,
    pub risk: std::sync::Arc<risk::RiskManager>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry =
            std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let market = std::sync::Arc::new(market::MarketFeed::new(std::sync::Arc::clone(&config)));
        let trading = std::sync::Arc::new(trading::TradingEngine::new(
            std::sync::Arc::clone(&config),
            std::sync::Arc::clone(&market),
        ));
        let risk = std::sync::Arc::new(risk::RiskManager::new(std::sync::Arc::clone(&config)));
        tracing::info!("kairos-finance AppState initialized");
        Ok(Self {
            config,
            telemetry,
            market,
            trading,
            risk,
        })
    }
}
