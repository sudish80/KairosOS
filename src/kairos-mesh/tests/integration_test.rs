use kairos_mesh::config::Config;
use kairos_mesh::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_wg_port() {
    let cfg = Config::default();
    assert!(cfg.wg.listen_port == 51820 || cfg.wg.listen_port == 0);
}

#[test]
fn test_telemetry_record_peer_connect() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_peer_connect();
    let m = t.metrics();
    assert_eq!(m["peers_discovered"], 1);
}

#[test]
fn test_config_key_path() {
    let cfg = Config::default();
    assert!(cfg.wg.private_key_file.contains("wg") || cfg.wg.private_key_file.is_empty());
}
