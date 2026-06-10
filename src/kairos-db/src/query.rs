use std::sync::Arc; use tokio::sync::RwLock; use crate::config; use crate::vector::VectorDatabase;
pub struct QueryEngine { config: Arc<RwLock<config::Config>>, vector_db: Arc<VectorDatabase> }
impl QueryEngine {
    pub fn new(config: Arc<RwLock<config::Config>>, vector_db: Arc<VectorDatabase>) -> Self { Self { config, vector_db } }
    pub async fn query(&self, _text: &str) -> anyhow::Result<Vec<String>> { Ok(vec![]) }
}
