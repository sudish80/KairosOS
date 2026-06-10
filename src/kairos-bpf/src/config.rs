//! Configuration for kairos-bpf
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ring_buffer_size: usize,
    pub perf_event_pages: usize,
    pub sampling_rate_hz: u32,
    pub programs: ProgramConfig,
    pub remediation: RemediationConfig,
    pub thermal: ThermalConfig,
    pub scheduler: SchedulerConfig,
    pub anomaly: AnomalyConfig,
    pub endpoints: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    pub execsnoop: bool,
    pub tcptop: bool,
    pub filemon: bool,
    pub anomaly: bool,
    pub schedlatency: bool,
    pub oomkill: bool,
    pub custom_programs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationConfig {
    pub enabled: bool,
    pub max_actions_per_minute: u32,
    pub cooldown_seconds: u64,
    pub auto_apply: bool,
    pub notification_webhook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalConfig {
    pub enabled: bool,
    pub critical_temp_c: u16,
    pub throttle_temp_c: u16,
    pub throttle_duration_ms: u64,
    pub quantize_model_on_throttle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub latency_threshold_us: u64,
    pub boost_priority_delta: i32,
    pub rt_runtime_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    pub enabled: bool,
    pub window_seconds: u64,
    pub stddev_threshold: f64,
    pub min_samples: usize,
}

impl Default for Config {
    fn default() -> Self {
        let mut endpoints = HashMap::new();
        endpoints.insert("bpf".into(), "unix:///var/run/kairos/bpf.sock".into());
        endpoints.insert("mcp".into(), "unix:///var/run/kairos/mcp.sock".into());

        Self {
            ring_buffer_size: 1024 * 1024,
            perf_event_pages: 64,
            sampling_rate_hz: 99,
            programs: ProgramConfig {
                execsnoop: true,
                tcptop: true,
                filemon: true,
                anomaly: true,
                schedlatency: true,
                oomkill: true,
                custom_programs: vec![],
            },
            remediation: RemediationConfig {
                enabled: true,
                max_actions_per_minute: 10,
                cooldown_seconds: 60,
                auto_apply: true,
                notification_webhook: None,
            },
            thermal: ThermalConfig {
                enabled: true,
                critical_temp_c: 95,
                throttle_temp_c: 85,
                throttle_duration_ms: 5000,
                quantize_model_on_throttle: true,
            },
            scheduler: SchedulerConfig {
                enabled: true,
                latency_threshold_us: 10000,
                boost_priority_delta: 10,
                rt_runtime_us: 950000,
            },
            anomaly: AnomalyConfig {
                enabled: true,
                window_seconds: 60,
                stddev_threshold: 3.0,
                min_samples: 100,
            },
            endpoints,
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = if path.ends_with(".toml") {
            toml::from_str(&content)?
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)?
        } else {
            anyhow::bail!("Unsupported config format");
        };
        Ok(config)
    }
}
