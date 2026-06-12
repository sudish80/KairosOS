use kairos_build::config::Config;
use kairos_build::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_buildroot_dir() {
    let cfg = Config::default();
    assert!(cfg.build.buildroot_dir.contains("buildroot") || cfg.build.buildroot_dir.is_empty());
}

#[test]
fn test_telemetry_record_build() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_build();
    let m = t.metrics();
    assert_eq!(m["builds"], 1);
}

#[test]
fn test_config_target_arch_default() {
    let cfg = Config::default();
    assert!(cfg.build.target == "aarch64" || cfg.build.target.is_empty());
}
