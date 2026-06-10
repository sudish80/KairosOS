use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub sim: SimConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig { pub num_qubits: u8, pub backend: String, pub shots: u32, pub max_circuit_depth: u32, pub data_dir: String }
impl Default for Config { fn default() -> Self { Self {
    general: GeneralConfig { daemonize: true, log_level: "info".into() },
    sim: SimConfig { num_qubits: 32, backend: "statevector".into(), shots: 1024, max_circuit_depth: 100, data_dir: "/var/lib/kairos/quantum".into() },
} } }
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
