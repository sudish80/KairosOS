#![deny(unsafe_code)]
pub mod config; pub mod error; pub mod telemetry; pub mod worker;
pub mod storage; pub mod vector; pub mod query; pub mod mem_bus;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub storage: std::sync::Arc<storage::StorageEngine>,
    pub vector_db: std::sync::Arc<vector::VectorDatabase>,
    pub query_engine: std::sync::Arc<query::QueryEngine>,
    pub mem_bus: std::sync::Arc<mem_bus::MemoryBus>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let telemetry = std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&config)));
        let storage = std::sync::Arc::new(storage::StorageEngine::new(std::sync::Arc::clone(&config)).await?);
        let vector_db = std::sync::Arc::new(vector::VectorDatabase::new(std::sync::Arc::clone(&config)));
        let query_engine = std::sync::Arc::new(query::QueryEngine::new(std::sync::Arc::clone(&config), std::sync::Arc::clone(&vector_db)));
        let mem_bus = std::sync::Arc::new(mem_bus::MemoryBus::new(std::sync::Arc::clone(&config)));
        tracing::info!("kairos-db AppState initialized");
        Ok(Self { config, telemetry, storage, vector_db, query_engine, mem_bus })
    }
}
