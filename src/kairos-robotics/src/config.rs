use serde::{Deserialize, Serialize}; use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config { pub general: GeneralConfig, pub control: ControlConfig, pub kinematics: KinematicsConfig }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig { pub daemonize: bool, pub log_level: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlConfig { pub loop_hz: u32, pub pid_kp: f64, pub pid_ki: f64, pub pid_kd: f64, pub pwm_device: String, pub encoder_device: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinematicsConfig { pub num_joints: u8, pub link_lengths: Vec<f64>, pub dh_params: String }
impl Default for Config { fn default() -> Self { Self {
    general: GeneralConfig { daemonize: true, log_level: "info".into() },
    control: ControlConfig { loop_hz: 1000, pid_kp: 1.0, pid_ki: 0.1, pid_kd: 0.05, pwm_device: "/dev/pwm".into(), encoder_device: "/dev/encoder".into() },
    kinematics: KinematicsConfig { num_joints: 6, link_lengths: vec![0.3, 0.25, 0.2, 0.15, 0.1, 0.05], dh_params: "/etc/kairos/robotics/dh.toml".into() },
} } }
impl Config { pub fn load(path: &Path) -> anyhow::Result<Self> { Ok(toml::from_str(&std::fs::read_to_string(path)?)?) } }
