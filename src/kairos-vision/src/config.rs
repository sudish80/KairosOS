use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub capture: CaptureConfig,
    pub detect: DetectConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub daemonize: bool,
    pub log_level: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub device: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub buffer_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectConfig {
    pub model: String,
    pub confidence: f64,
    pub labels_path: String,
    pub max_detections: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                daemonize: true,
                log_level: "info".into(),
            },
            capture: CaptureConfig {
                device: "/dev/video0".into(),
                width: 1280,
                height: 720,
                fps: 30,
                buffer_count: 4,
            },
            detect: DetectConfig {
                model: "/var/lib/kairos/models/yolov8n.gguf".into(),
                confidence: 0.5,
                labels_path: "/etc/kairos/vision/coco.txt".into(),
                max_detections: 100,
            },
        }
    }
}
impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }
}
