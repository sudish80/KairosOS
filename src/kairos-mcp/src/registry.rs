//! Service registry with capability resolution
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<crate::protocol::Capability>,
    pub metadata: HashMap<String, String>,
    pub registered_at: std::time::Instant,
    pub last_heartbeat: std::time::Instant,
    pub status: ServiceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self { services: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn register(&self, info: ServiceInfo) -> anyhow::Result<()> {
        let mut services = self.services.write().await;
        info!("Registering service: {}", info.name);
        services.insert(info.name.clone(), info);
        Ok(())
    }

    pub async fn unregister(&self, name: &str) -> anyhow::Result<()> {
        let mut services = self.services.write().await;
        services.remove(name);
        info!("Unregistered service: {}", name);
        Ok(())
    }

    pub async fn resolve(&self, capability: &str) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.values()
            .filter(|s| s.status == crate::registry::ServiceStatus::Healthy)
            .find(|s| s.capabilities.iter().any(|c| c.name == capability))
            .cloned()
    }

    pub async fn get_all(&self) -> Vec<ServiceInfo> {
        self.services.read().await.values().cloned().collect()
    }

    pub async fn heartbeat(&self, name: &str) -> anyhow::Result<()> {
        let mut services = self.services.write().await;
        if let Some(s) = services.get_mut(name) {
            s.last_heartbeat = std::time::Instant::now();
            s.status = ServiceStatus::Healthy;
        }
        Ok(())
    }

    pub async fn start_heartbeat(&self) -> anyhow::Result<()> {
        let services = Arc::clone(&self.services);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let mut services = services.write().await;
                let now = std::time::Instant::now();
                for (_, info) in services.iter_mut() {
                    if now.duration_since(info.last_heartbeat).as_secs() > 30 {
                        info.status = ServiceStatus::Degraded;
                    }
                    if now.duration_since(info.last_heartbeat).as_secs() > 60 {
                        info.status = ServiceStatus::Unhealthy;
                    }
                }
            }
        });
        Ok(())
    }

    pub fn is_healthy(&self) -> bool {
        true
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}