use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct QuantumSimulator {
    config: Arc<RwLock<config::Config>>,
}
impl QuantumSimulator {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self { config }
    }
    pub async fn run_circuit(&self) -> anyhow::Result<Vec<f64>> {
        let n = self.config.read().await.sim.num_qubits;
        Ok(vec![0.0; 1 << n])
    }
}
