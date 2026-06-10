#![deny(unsafe_code)]
pub mod config;
pub mod error;
pub mod model;
pub mod quantizer;
pub mod runtime;
pub mod telemetry;
pub mod worker;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub runtime: std::sync::Arc<runtime::LlmRuntime>,
    pub quantizer: std::sync::Arc<quantizer::Quantizer>,
    pub model_registry: std::sync::Arc<model::ModelRegistry>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry =
            std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let model_registry =
            std::sync::Arc::new(model::ModelRegistry::new(std::sync::Arc::clone(&config)));
        let quantizer =
            std::sync::Arc::new(quantizer::Quantizer::new(std::sync::Arc::clone(&config)));
        let runtime = std::sync::Arc::new(runtime::LlmRuntime::new(
            std::sync::Arc::clone(&config),
            std::sync::Arc::clone(&model_registry),
        ));
        tracing::info!("kairos-llm AppState initialized");
        Ok(Self {
            config,
            telemetry,
            runtime,
            quantizer,
            model_registry,
        })
    }
}
