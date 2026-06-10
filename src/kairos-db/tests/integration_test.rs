use kairos_db::config::Config;
use kairos_db::telemetry::Telemetry;

#[test]
fn test_config_default_db_path() {
    let cfg = Config::default();
    assert!(cfg.db_path.contains("kairos.db") || cfg.db_path.is_empty());
}

#[test]
fn test_telemetry_query_count() {
    let t = Telemetry::new();
    assert_eq!(t.query_count(), 0);
    t.incr_query_count();
    assert_eq!(t.query_count(), 1);
}

#[test]
fn test_telemetry_vector_dim_default() {
    let t = Telemetry::new();
    assert_eq!(t.vector_count(), 0);
}
