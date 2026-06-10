use crate::config;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
pub struct ResourceManager {
    config: Arc<RwLock<config::Config>>,
    allocated_cpu: Arc<RwLock<f64>>,
    allocated_mem: Arc<RwLock<u64>>,
}
impl ResourceManager {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self {
        Self {
            config,
            allocated_cpu: Arc::new(RwLock::new(0.0)),
            allocated_mem: Arc::new(RwLock::new(0)),
        }
    }
    pub async fn reserve(&self, cpu: f64, mem: u64) -> bool {
        let cfg = self.config.read().await;
        let mut ac = self.allocated_cpu.write().await;
        let mut am = self.allocated_mem.write().await;
        if *ac + cpu <= cfg.resource.cpu_limit && *am + mem <= cfg.resource.memory_limit_mb {
            *ac += cpu;
            *am += mem;
            true
        } else {
            false
        }
    }
    pub async fn release(&self, cpu: f64, mem: u64) {
        *self.allocated_cpu.write().await -= cpu;
        *self.allocated_mem.write().await -= mem;
    }
}
