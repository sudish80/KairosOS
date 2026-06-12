use kairos_db::config::Config;
use kairos_db::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_db_path() {
    let cfg = Config::default();
    assert!(cfg.storage.db_path.contains("kairos.db") || cfg.storage.db_path.is_empty());
}

#[test]
fn test_telemetry_record_query() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_query();
    let m = t.metrics();
    assert_eq!(m["queries"], 1);
}

#[test]
fn test_telemetry_vector_indexed() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_vector();
    let m = t.metrics();
    assert_eq!(m["vectors_indexed"], 1);
}
