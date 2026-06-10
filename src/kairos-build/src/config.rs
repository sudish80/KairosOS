use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub build: BuildConfig, pub image: ImageConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String, pub workspace: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig { pub buildroot_dir: String, pub config_file: String, pub target: String, pub jobs: u16, pub ccache: bool }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig { pub output_dir: String, pub compress: bool, pub format: String, pub verify: bool }
impl Default for Config { fn default() -> Self {
    Self {
        general: GeneralConfig { daemonize: true, log_level: "info".into(), workspace: "/var/lib/kairos/build/workspace".into() },
        build: BuildConfig { buildroot_dir: "/opt/buildroot".into(), config_file: "/etc/kairos/buildroot.config".into(), target: "aarch64".into(), jobs: 8, ccache: true },
        image: ImageConfig { output_dir: "/var/lib/kairos/build/output".into(), compress: true, format: "ext4".into(), verify: true },
    }
} }
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
