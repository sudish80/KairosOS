use kairos_quantum::config::Config;
use kairos_quantum::telemetry::Telemetry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn test_config_default_num_qubits() {
    let cfg = Config::default();
    assert!(cfg.sim.num_qubits == 32 || cfg.sim.num_qubits == 0);
}

#[test]
fn test_telemetry_record_circuit() {
    let cfg = Arc::new(RwLock::new(Config::default()));
    let t = Telemetry::new(cfg);
    t.record_circuit(10, 1024);
    let m = t.metrics();
    assert_eq!(m["circuits_executed"], 1);
    assert_eq!(m["gates_applied"], 10);
    assert_eq!(m["shots_taken"], 1024);
}
