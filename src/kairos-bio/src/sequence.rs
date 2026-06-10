use std::sync::Arc; use tokio::sync::RwLock; use tokio::fs; use crate::config;
pub struct SequenceEngine { config: Arc<RwLock<config::Config>> }
impl SequenceEngine {
    pub fn new(config: Arc<RwLock<config::Config>>) -> Self { Self { config } }
    pub async fn load_reference(&self) -> anyhow::Result<String> {
        let path = self.config.read().await.sequence.reference_genome.clone();
        if std::path::Path::new(&path).exists() { Ok(fs::read_to_string(&path).await?) } else { Ok(">ref\nACGTN".into()) }
    }
    pub fn gc_content(&self, seq: &str) -> f64 { let gc = seq.chars().filter(|c| *c == 'G' || *c == 'C' || *c == 'g' || *c == 'c').count() as f64; gc / seq.len().max(1) as f64 }
    pub fn reverse_complement(&self, seq: &str) -> String { seq.chars().rev().map(|c| match c {'A'=>'T','T'=>'A','G'=>'C','C'=>'G','a'=>'t','t'=>'a','g'=>'c','c'=>'g',_=>c}).collect() }
}
