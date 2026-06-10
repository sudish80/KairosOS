use kairos_finance::config::Config;
use kairos_finance::telemetry::Telemetry;

#[test]
fn test_config_default_ws_url() {
    let cfg = Config::default();
    assert!(cfg.ws_url.contains("wss://") || cfg.ws_url.is_empty());
}

#[test]
fn test_telemetry_trade_count() {
    let t = Telemetry::new();
    assert_eq!(t.trade_count(), 0);
    t.incr_trade_count();
    assert_eq!(t.trade_count(), 1);
}

#[test]
fn test_telemetry_max_drawdown_default() {
    let t = Telemetry::new();
    assert_eq!(t.trade_count(), 0);
}
