pub mod arch;
pub mod audit;
pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod plugin;
pub mod protocol;
pub mod rate_limit;
pub mod registry;
pub mod server;
pub mod transport;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub registry: Arc<registry::ServiceRegistry>,
    pub transport_manager: Arc<transport::TransportManager>,
    pub auth_manager: Arc<auth::AuthManager>,
    pub rate_limiter: Arc<rate_limit::RateLimiter>,
    pub audit_logger: Arc<audit::AuditLogger>,
    pub server: Arc<server::McpServer>,
    pub plugin_engine: Arc<plugin::PluginEngine>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let registry = Arc::new(registry::ServiceRegistry::new());
        let server = Arc::new(server::McpServer::new(Arc::clone(&config)));
        let transport_manager = Arc::new(transport::TransportManager::new(
            Arc::clone(&config),
            Arc::clone(&server),
        ));
        let auth_manager = Arc::new(auth::AuthManager::new(Arc::clone(&config)));
        let rate_limiter = Arc::new(rate_limit::RateLimiter::new(Arc::clone(&config)));
        let audit_logger = Arc::new(audit::AuditLogger::new(Arc::clone(&config)).await?);
        let plugin_engine = Arc::new(plugin::PluginEngine::new(Arc::clone(&config)));

        info!("kairos-mcp AppState initialized");
        Ok(Self {
            config,
            registry,
            transport_manager,
            auth_manager,
            rate_limiter,
            audit_logger,
            server,
            plugin_engine,
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        self.transport_manager.start_all().await?;
        self.registry.start_heartbeat().await?;
        info!("kairos-mcp started");
        Ok(())
    }

    pub async fn discover_plugins(&self) -> anyhow::Result<()> {
        let cfg = self.config.read().await;
        let plugin_dir = Path::new(&cfg.plugin.directory);
        let plugins = self.plugin_engine.discover(plugin_dir).await?;
        info!("Discovered {} plugins from {:?}", plugins.len(), plugin_dir);
        for p in &plugins {
            let name = p.manifest.name.clone();
            let engine = Arc::clone(&self.plugin_engine);
            self.server
                .register_method(&format!("plugin:{}:execute", name), move |params| {
                    let engine = Arc::clone(&engine);
                    let n = name.clone();
                    let result = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current()
                            .block_on(engine.execute_plugin(&n, "run", params))
                    });
                    serde_json::json!(
                        result.unwrap_or(serde_json::json!({"error": "plugin execution failed"}))
                    )
                })
                .await;
        }
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
