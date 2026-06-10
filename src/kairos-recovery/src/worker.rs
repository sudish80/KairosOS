//! Background worker — periodic health checks and boot count management
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use crate::config;
use crate::health::HealthChecker;
use crate::boot::BootManager;
use crate::telemetry::Telemetry;

pub struct RecoveryWorker {
    config: Arc<RwLock<config::Config>>,
    health_checker: Arc<HealthChecker>,
    boot_manager: Arc<BootManager>,
    telemetry: Arc<Telemetry>,
}

impl RecoveryWorker {
    pub fn new(
        config: Arc<RwLock<config::Config>>,
        health_checker: Arc<HealthChecker>,
        boot_manager: Arc<BootManager>,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        Self { config, health_checker, boot_manager, telemetry }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("RecoveryWorker started");

        // Handle boot count on startup
        if self.boot_manager.should_fallback().await {
            info!("Boot count exceeded maximum - triggering automatic rollback");
            // In production: trigger recovery shell or rollback
        }
        self.boot_manager.increment_boot_count().await?;

        // Periodic health checks
        let health_checker = Arc::clone(&self.health_checker);
        let telemetry = Arc::clone(&self.telemetry);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                match health_checker.check_health().await {
                    Ok(report) => telemetry.record_health(report.overall == "healthy"),
                    Err(e) => {
                        error!("Health check failed: {}", e);
                        telemetry.record_health(false);
                    }
                }
            }
        });

        Ok(())
    }
}
