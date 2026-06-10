use kairos_build::config::Config;
use kairos_build::telemetry::Telemetry;

#[test]
fn test_config_default_buildroot_dir() {
    let cfg = Config::default();
    assert!(cfg.buildroot_dir.contains("buildroot") || cfg.buildroot_dir.is_empty());
}

#[test]
fn test_telemetry_build_count() {
    let t = Telemetry::new();
    assert_eq!(t.build_count(), 0);
    t.incr_build_count();
    assert_eq!(t.build_count(), 1);
}

#[test]
fn test_config_target_arch_default() {
    let cfg = Config::default();
    assert!(cfg.target_arch == "x86_64" || cfg.target_arch.is_empty());
}
