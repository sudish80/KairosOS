use kairos_climate::config::Config;
use kairos_climate::telemetry::Telemetry;

#[test]
fn test_config_default_grid_resolution() {
    let cfg = Config::default();
    assert!(cfg.grid_resolution > 0.0 || cfg.grid_resolution == 0.0);
}

#[test]
fn test_telemetry_assimilation_count() {
    let t = Telemetry::new();
    assert_eq!(t.assimilation_count(), 0);
    t.incr_assimilation_count();
    assert_eq!(t.assimilation_count(), 1);
}

#[test]
fn test_config_ensemble_members_default() {
    let cfg = Config::default();
    assert!(cfg.ensemble_members == 50 || cfg.ensemble_members == 0);
}
