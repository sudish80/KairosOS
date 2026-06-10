use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub data: DataConfig,
    pub model: ModelConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub input_dir: String,
    pub output_dir: String,
    pub grid_resolution: f64,
    pub ensemble_size: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub timestep_secs: u64,
    pub diffusion_coeff: f64,
    pub advection_scheme: String,
    pub radiation: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                log_level: "info".into(),
            },
            data: DataConfig {
                input_dir: "/var/lib/kairos/climate/input".into(),
                output_dir: "/var/lib/kairos/climate/output".into(),
                grid_resolution: 0.25,
                ensemble_size: 50,
            },
            model: ModelConfig {
                timestep_secs: 3600,
                diffusion_coeff: 0.01,
                advection_scheme: "upwind".into(),
                radiation: true,
            },
        }
    }
}
impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }
}
