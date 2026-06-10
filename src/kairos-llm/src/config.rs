use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub runtime: RuntimeConfig, pub quant: QuantConfig, pub models: ModelsConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig { pub llama_bin: String, pub gpu_layers: u32, pub ctx_size: u32, pub threads: u16, pub port: u16 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantConfig { pub default_type: String, pub allow_quantization: bool, pub calibration_set: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig { pub models_dir: String, pub default_model: String, pub auto_download: bool }
impl Default for Config {
    fn default() -> Self { Self {
        general: GeneralConfig { daemonize: true, log_level: "info".into() },
        runtime: RuntimeConfig { llama_bin: "/usr/lib/kairos/bin/llama-server".into(), gpu_layers: 35, ctx_size: 4096, threads: 8, port: 8081 },
        quant: QuantConfig { default_type: "q4_k_m".into(), allow_quantization: true, calibration_set: "/var/lib/kairos/models/calibration".into() },
        models: ModelsConfig { models_dir: "/var/lib/kairos/models".into(), default_model: "llama-3.2-3b-q4.gguf".into(), auto_download: false },
    } }
}
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
