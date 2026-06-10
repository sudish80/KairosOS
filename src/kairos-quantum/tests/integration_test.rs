use kairos_quantum::config::Config;
use kairos_quantum::telemetry::Telemetry;

#[test]
fn test_config_default_max_qubits() {
    let cfg = Config::default();
    assert!(cfg.max_qubits == 32 || cfg.max_qubits == 0);
}

#[test]
fn test_telemetry_gate_count() {
    let t = Telemetry::new();
    assert_eq!(t.gate_count(), 0);
    t.incr_gate_count();
    assert_eq!(t.gate_count(), 1);
}

#[test]
fn test_telemetry_shots_default() {
    let t = Telemetry::new();
    assert_eq!(t.shot_count(), 0);
}
