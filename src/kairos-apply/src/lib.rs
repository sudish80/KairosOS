//! kairos-apply: Declarative configuration state applier — production-hardened
#![deny(unsafe_code)]

pub mod config;
pub mod error;
pub mod telemetry;
pub mod worker;
pub mod generation;
pub mod parser;
pub mod validator;
pub mod rollback;
pub mod diff;
pub mod applier;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub config: Arc<RwLock<config::Config>>,
    pub telemetry: Arc<telemetry::Telemetry>,
    pub generation_store: Arc<generation::GenerationStore>,
    pub parser: Arc<parser::DeclarativeParser>,
    pub validator: Arc<validator::ConfigValidator>,
    pub diff_engine: Arc<diff::DiffEngine>,
    pub rollback_manager: Arc<rollback::RollbackManager>,
    pub applier: Arc<applier::StateApplier>,
}

impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = Arc::new(RwLock::new(cfg));
        let telemetry = Arc::new(telemetry::Telemetry::new(Arc::clone(&config)));
        let generation_store = Arc::new(generation::GenerationStore::new(Arc::clone(&config)).await?);
        let parser = Arc::new(parser::DeclarativeParser::new(Arc::clone(&config)));
        let validator = Arc::new(validator::ConfigValidator::new(Arc::clone(&config)));
        let diff_engine = Arc::new(diff::DiffEngine::new(Arc::clone(&config)));
        let rollback_manager = Arc::new(rollback::RollbackManager::new(Arc::clone(&config)).await?);
        let applier = Arc::new(applier::StateApplier::new(
            Arc::clone(&config),
            Arc::clone(&generation_store),
            Arc::clone(&validator),
            Arc::clone(&diff_engine),
            Arc::clone(&rollback_manager),
            Arc::clone(&telemetry),
        ));

        info!("kairos-apply AppState initialized");
        Ok(Self { config, telemetry, generation_store, parser, validator, diff_engine, rollback_manager, applier })
    }
}
