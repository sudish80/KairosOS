use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct GateEngine {
    config: Arc<RwLock<config::Config>>,
}
impl GateEngine {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub fn hadamard(&self, _state: &mut Vec<Complex64>) {}
    pub fn cnot(&self, _state: &mut Vec<Complex64>, _control: usize, _target: usize) {}
    pub fn pauli_x(&self, _state: &mut Vec<Complex64>, _qubit: usize) {}
    pub fn pauli_z(&self, _state: &mut Vec<Complex64>, _qubit: usize) {}
    pub fn phase(&self, _state: &mut Vec<Complex64>, _qubit: usize, _angle: f64) {}
}
type Complex64 = num_complex::Complex64;
