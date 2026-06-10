//! Thermal governor for hardware thermal management
use crate::error::Result;
use crate::config::Config;
use crate::telemetry::TelemetryStore;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub struct ThermalGovernor {
    config: Arc<RwLock<Config>>,
    telemetry: Arc<TelemetryStore>,
    current_state: Arc<RwLock<ThermalState>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalState {
    Normal,
    Warm,
    Hot,
    Critical,
}

impl ThermalGovernor {
    pub fn new(config: Arc<RwLock<Config>>, telemetry: Arc<TelemetryStore>) -> Self {
        Self {
            config,
            telemetry,
            current_state: Arc::new(RwLock::new(ThermalState::Normal)),
        }
    }

    pub async fn governor_loop(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let Err(e) = self.check_thermal().await {
                warn!("Thermal check failed: {}", e);
            }
        }
    }

    async fn check_thermal(&self) -> Result<()> {
        let cfg = self.config.read().await;
        if !cfg.thermal.enabled {
            return Ok(());
        }

        // Read thermal zones
        let temps = self.read_thermal_zones().await?;
        let max_temp = temps.iter().max().copied().unwrap_or(0);

        let mut state = self.current_state.write().await;
        let new_state = if max_temp >= cfg.thermal.critical_temp_c {
            ThermalState::Critical
        } else if max_temp >= cfg.thermal.throttle_temp_c {
            ThermalState::Hot
        } else if max_temp >= cfg.thermal.throttle_temp_c - 10 {
            ThermalState::Warm
        } else {
            ThermalState::Normal
        };

        if *state != new_state {
            info!("Thermal state transition: {:?} -> {:?} (max_temp: {}°C)", state, new_state, max_temp);
            *state = new_state;
            self.handle_state_change(new_state, max_temp, &cfg).await?;
        }

        Ok(())
    }

    async fn handle_state_change(&self, state: ThermalState, temp: u16, cfg: &Config) -> Result<()> {
        match state {
            ThermalState::Critical => {
                warn!("CRITICAL TEMPERATURE: {}°C", temp);
                // Emergency throttle all non-essential processes
                self.throttle_all_non_essential().await?;
                // Trigger immediate model quantization if enabled
                if cfg.thermal.quantize_model_on_throttle {
                    self.trigger_model_quantization().await?;
                }
            }
            ThermalState::Hot => {
                warn!("HIGH TEMPERATURE: {}°C", temp);
                // Throttle background processes
                self.throttle_background().await?;
            }
            ThermalState::Warm => {
                info!("Elevated temperature: {}°C", temp);
                // Reduce background task priority
                self.reduce_background_priority().await?;
            }
            ThermalState::Normal => {
                info!("Temperature normalized: {}°C", temp);
                // Restore normal operation
                self.restore_normal_operation().await?;
            }
        }
        Ok(())
    }

    async fn read_thermal_zones(&self) -> Result<Vec<u16>> {
        let mut temps = Vec::new();
        for entry in std::fs::read_dir("/sys/class/thermal")? {
            let entry = entry?;
            let path = entry.path();
            if path.join("type").exists() && path.join("temp").exists() {
                if let Ok(temp_str) = std::fs::read_to_string(path.join("temp")) {
                    if let Ok(temp_millic) = temp_str.trim().parse::<i32>() {
                        temps.push((temp_millic / 1000) as u16);
                    }
                }
            }
        }
        Ok(temps)
    }

    async fn throttle_all_non_essential(&self) -> Result<()> {
        warn!("Throttling all non-essential processes");
        // In production: write to cgroup cpu.max for non-essential cgroups
        Ok(())
    }

    async fn throttle_background(&self) -> Result<()> {
        warn!("Throttling background processes");
        Ok(())
    }

    async fn reduce_background_priority(&self) -> Result<()> {
        info!("Reducing background priority");
        Ok(())
    }

    async fn restore_normal_operation(&self) -> Result<()> {
        info!("Restoring normal operation");
        Ok(())
    }

    async fn trigger_model_quantization(&self) -> Result<()> {
        warn!("Triggering model quantization due to thermal throttling");
        // In production: signal inference hub to quantize
        Ok(())
    }

    pub async fn get_current_state(&self) -> ThermalState {
        *self.current_state.read().await
    }

    pub async fn get_temperatures(&self) -> Result<Vec<u16>> {
        self.read_thermal_zones().await
    }
}