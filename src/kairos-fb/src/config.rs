use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub display: DisplayConfig, pub drm: DrmConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig { pub width: u32, pub height: u32, pub bpp: u32, pub fb_device: String, pub double_buffer: bool }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrmConfig { pub card: String, pub connector: String, pub mode: String, pub vsync: bool }
impl Default for Config {
    fn default() -> Self { Self {
        general: GeneralConfig { daemonize: true, log_level: "info".into() },
        display: DisplayConfig { width: 1920, height: 1080, bpp: 32, fb_device: "/dev/fb0".into(), double_buffer: true },
        drm: DrmConfig { card: "/dev/dri/card0".into(), connector: "HDMI-A-1".into(), mode: "1920x1080".into(), vsync: true },
    } }
}
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
