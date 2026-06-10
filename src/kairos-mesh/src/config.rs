use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub wg: WgConfig, pub discovery: DiscoveryConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String, pub node_id: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WgConfig { pub interface: String, pub listen_port: u16, pub private_key_file: String, pub peers: Vec<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig { pub interval_secs: u64, pub seeds: Vec<String>, pub protocol: String }
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig { daemonize: true, log_level: "info".into(), node_id: "kairos-node-1".into() },
            wg: WgConfig { interface: "kmesh0".into(), listen_port: 51820, private_key_file: "/etc/kairos/wg/key".into(), peers: vec![] },
            discovery: DiscoveryConfig { interval_secs: 30, seeds: vec!["10.0.0.1:51820".into()], protocol: "kadmelia".into() },
        }
    }
}
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
