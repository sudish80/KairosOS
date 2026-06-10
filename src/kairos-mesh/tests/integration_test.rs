use kairos_mesh::config::Config;
use kairos_mesh::telemetry::Telemetry;

#[test]
fn test_config_default_wg_port() {
    let cfg = Config::default();
    assert!(cfg.wg_port == 51820 || cfg.wg_port == 0);
}

#[test]
fn test_telemetry_peer_count() {
    let t = Telemetry::new();
    assert_eq!(t.discovered_peers(), 0);
    t.incr_discovered_peers();
    assert_eq!(t.discovered_peers(), 1);
}

#[test]
fn test_config_key_path() {
    let cfg = Config::default();
    assert!(cfg.wg_key_path.contains("wg") || cfg.wg_key_path.is_empty());
}
