use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub market: MarketConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub feed_url: String,
    pub symbols: Vec<String>,
    pub reconnect_secs: u64,
    pub buffer_size: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub strategy: String,
    pub max_positions: u32,
    pub order_timeout_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_drawdown: f64,
    pub max_leverage: f64,
    pub var_confidence: f64,
    pub position_limit: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                log_level: "info".into(),
            },
            market: MarketConfig {
                feed_url: "wss://market-data.kairos.local/feed".into(),
                symbols: vec!["BTC/USD".into(), "ETH/USD".into()],
                reconnect_secs: 5,
                buffer_size: 10000,
            },
            trading: TradingConfig {
                strategy: "momentum".into(),
                max_positions: 10,
                order_timeout_ms: 1000,
            },
            risk: RiskConfig {
                max_drawdown: 0.15,
                max_leverage: 2.0,
                var_confidence: 0.95,
                position_limit: 100000,
            },
        }
    }
}
impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }
}
