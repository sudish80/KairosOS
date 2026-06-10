use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub arinc: ArincConfig, pub mavlink: MavlinkConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArincConfig { pub device: String, pub baud: u32, pub labels: Vec<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavlinkConfig { pub device: String, pub baud: u32, pub sys_id: u8, pub comp_id: u8, pub stream_rate: u8 }
impl Default for Config { fn default() -> Self { Self {
    general: GeneralConfig { daemonize: true, log_level: "info".into() },
    arinc: ArincConfig { device: "/dev/ttyS0".into(), baud: 100000, labels: vec!["altitude".into(), "airspeed".into(), "heading".into()] },
    mavlink: MavlinkConfig { device: "/dev/ttyS1".into(), baud: 57600, sys_id: 1, comp_id: 1, stream_rate: 10 },
} } }
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
