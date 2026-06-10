pub mod config;
pub mod error;
pub mod telemetry;
pub mod handler;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::Telemetry>,
    pub handler: handler::ProcessHandler,
}

impl AppState {
    pub fn new(cfg: config::Config) -> Self {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::Telemetry::new());
        let handler = handler::ProcessHandler::new(Arc::clone(&config), Arc::clone(&telemetry));
        Self { config, telemetry, handler }
    }
}
