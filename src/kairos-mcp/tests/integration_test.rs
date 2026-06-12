use kairos_mcp::config::Config;

#[test]
fn test_config_default_socket() {
    let cfg = Config::default();
    assert_eq!(cfg.transport.unix_socket_path, "/var/run/kairos/mcp.sock");
}

#[test]
fn test_config_default_log_level() {
    let cfg = Config::default();
    assert_eq!(cfg.general.log_level, "info");
}

#[test]
fn test_config_plugin_defaults() {
    let cfg = Config::default();
    assert_eq!(cfg.plugin.directory, "/usr/lib/kairos/plugins");
    assert!(cfg.plugin.enabled);
}
