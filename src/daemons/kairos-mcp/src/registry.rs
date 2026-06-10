// MCP Service Registry
// Manages service registrations, capability discovery, and routing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub name: String,
    pub transport: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub transport: String,
    pub capabilities: Vec<String>,
    pub status: ServiceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Online,
    Offline,
    Degraded,
}

pub struct ServiceRegistry {
    services: HashMap<String, ServiceRegistration>,
    service_map: HashMap<String, String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            service_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, registration: ServiceRegistration) {
        for cap in &registration.capabilities {
            self.service_map.insert(cap.clone(), registration.name.clone());
        }
        self.services.insert(registration.name.clone(), registration);
    }

    pub fn unregister(&mut self, name: &str) {
        if let Some(reg) = self.services.remove(name) {
            for cap in &reg.capabilities {
                self.service_map.remove(cap);
            }
        }
    }

    pub fn resolve_resource(&self, uri: &str) -> Option<&ServiceRegistration> {
        let mut best_match: Option<&ServiceRegistration> = None;
        let mut best_len = 0;

        for (cap, svc_name) in &self.service_map {
            if cap.starts_with("resources:") {
                let pattern = &cap["resources:".len()..];
                if pattern.ends_with("/*") {
                    let prefix = &pattern[..pattern.len() - 1];
                    if uri.starts_with(prefix) && prefix.len() > best_len {
                        best_match = self.services.get(svc_name);
                        best_len = prefix.len();
                    }
                } else if uri == pattern && pattern.len() > best_len {
                    best_match = self.services.get(svc_name);
                    best_len = pattern.len();
                }
            }
        }

        best_match
    }

    pub fn resolve_tool(&self, tool_name: &str) -> Option<&ServiceRegistration> {
        let cap = format!("tools:{}", tool_name);
        if let Some(svc_name) = self.service_map.get(&cap) {
            return self.services.get(svc_name);
        }

        for (cap, svc_name) in &self.service_map {
            if cap.starts_with("tools:") && cap[6..] == tool_name {
                return self.services.get(svc_name);
            }
        }

        None
    }

    pub fn list_services(&self) -> Vec<ServiceInfo> {
        self.services.values().map(|reg| ServiceInfo {
            name: reg.name.clone(),
            transport: reg.transport.clone(),
            capabilities: reg.capabilities.clone(),
            status: ServiceStatus::Online,
        }).collect()
    }

    pub fn service_count(&self) -> usize {
        self.services.len()
    }
}
