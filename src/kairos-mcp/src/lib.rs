//! kairos-mcp: MCP protocol router and service registry — production-hardened
//! Implements all 25 items of Subsystem 4: MCP Protocol (items 513-537)
#![deny(unsafe_code)]
#![deny(clippy::all)]

pub mod config;
pub mod error;
pub mod protocol;
pub mod registry;
pub mod transport;
pub mod auth;
pub mod rate_limit;
pub mod audit;
pub mod server;
pub mod client;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub registry: Arc<registry::ServiceRegistry>,
    pub transport_manager: Arc<transport::TransportManager>,
    pub auth_manager: Arc<auth::AuthManager>,
    pub rate_limiter: Arc<rate_limit::RateLimiter>,
    pub audit_logger: Arc<audit::AuditLogger>,
    pub server: Arc<server::McpServer>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let registry = Arc::new(registry::ServiceRegistry::new());
        let server = Arc::new(server::McpServer::new(Arc::clone(&config)));
        let transport_manager = Arc::new(transport::TransportManager::new(Arc::clone(&config), Arc::clone(&server)));
        let auth_manager = Arc::new(auth::AuthManager::new(Arc::clone(&config)));
        let rate_limiter = Arc::new(rate_limit::RateLimiter::new(Arc::clone(&config)));
        let audit_logger = Arc::new(audit::AuditLogger::new(Arc::clone(&config)).await?);

        info!("kairos-mcp AppState initialized");
        Ok(Self { config, registry, transport_manager, auth_manager, rate_limiter, audit_logger, server })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        self.transport_manager.start_all().await?;
        self.registry.start_heartbeat().await?;
        info!("kairos-mcp started");
        Ok(())
    }
}

pub async fn run(state: AppState) -> anyhow::Result<()> {
    state.start().await?;
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let cfg = config::Config::default();
        let state = AppState::new(cfg).await.unwrap();
        assert!(state.registry.is_healthy());
    }
}