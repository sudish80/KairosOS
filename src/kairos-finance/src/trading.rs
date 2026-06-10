use std::sync::Arc; use tokio::sync::RwLock; use crate::config; use crate::market::MarketFeed;
pub struct TradingEngine { config: Arc<RwLock<config::Config>>, _market: Arc<MarketFeed> }
impl TradingEngine {
    pub fn new(config: Arc<RwLock<config::Config>>, market: Arc<MarketFeed>) -> Self { Self { config, _market: market } }
    pub async fn execute(&self, _symbol: &str, _side: &str, _qty: f64) -> anyhow::Result<String> { tracing::info!("Order: {} {} {}", _side, _qty, _symbol); Ok("order-1".into()) }
}
