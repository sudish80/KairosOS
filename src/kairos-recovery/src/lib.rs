//! kairos-recovery: A/B partition management, dm-verity, recovery shell — production-hardened
#![deny(unsafe_code)]

pub mod boot;
pub mod config;
pub mod digtwin;
pub mod error;
pub mod health;
pub mod partitions;
pub mod predict;
pub mod recovery;
pub mod telemetry;
pub mod update;
pub mod verity;
pub mod worker;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::Telemetry>,
    pub partition_manager: Arc<partitions::PartitionManager>,
    pub verity_manager: Arc<verity::VerityManager>,
    pub recovery_shell: Arc<recovery::RecoveryShell>,
    pub boot_manager: Arc<boot::BootManager>,
    pub update_engine: Arc<update::UpdateEngine>,
    pub health_checker: Arc<health::HealthChecker>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::Telemetry::new(Arc::clone(&config)));
        let partition_manager =
            Arc::new(partitions::PartitionManager::new(Arc::clone(&config)).await?);
        let verity_manager = Arc::new(verity::VerityManager::new(Arc::clone(&config)));
        let recovery_shell = Arc::new(recovery::RecoveryShell::new(Arc::clone(&config)));
        let boot_manager = Arc::new(boot::BootManager::new(
            Arc::clone(&config),
            Arc::clone(&partition_manager),
        ));
        let update_engine = Arc::new(update::UpdateEngine::new(
            Arc::clone(&config),
            Arc::clone(&partition_manager),
            Arc::clone(&verity_manager),
            Arc::clone(&boot_manager),
            Arc::clone(&telemetry),
        ));
        let health_checker = Arc::new(health::HealthChecker::new(
            Arc::clone(&config),
            Arc::clone(&partition_manager),
        ));

        info!("kairos-recovery AppState initialized");
        Ok(Self {
            config,
            telemetry,
            partition_manager,
            verity_manager,
            recovery_shell,
            boot_manager,
            update_engine,
            health_checker,
        })
    }
}

#[derive(Debug)]
pub enum Slot {
    A,
    B,
}

impl Slot {
    pub fn current() -> Self {
        // In production: read from /proc/cmdline or EFI vars
        if std::path::Path::new("/usr/lib/kairos/slot_a").exists() {
            Slot::A
        } else {
            Slot::B
        }
    }
}
