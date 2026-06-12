use kairos_finance::config::Config;
use kairos_finance::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_market_feed_url() {
    let cfg = Config::default();
    assert!(cfg.market.feed_url.contains("wss://") || cfg.market.feed_url.is_empty());
}

#[test]
fn test_telemetry_record_tick() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_tick();
    let m = t.metrics();
    assert_eq!(m["ticks_received"], 1);
}
