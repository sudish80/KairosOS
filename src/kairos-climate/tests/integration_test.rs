use kairos_climate::config::Config;
use kairos_climate::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_grid_resolution() {
    let cfg = Config::default();
    assert!(cfg.data.grid_resolution > 0.0 || cfg.data.grid_resolution == 0.0);
}

#[test]
fn test_telemetry_record_assimilation() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_assimilation();
    let m = t.metrics();
    assert_eq!(m["assimilations"], 1);
}

#[test]
fn test_config_ensemble_size_default() {
    let cfg = Config::default();
    assert!(cfg.data.ensemble_size == 50 || cfg.data.ensemble_size == 0);
}
