#![deny(unsafe_code)]
pub mod alignment;
pub mod config;
pub mod error;
pub mod pipeline;
pub mod sequence;
pub mod telemetry;
pub mod worker;
pub struct AppState {
    pub config: std::sync::Arc<tokio::sync::RwLock<config::Config>>,
    pub telemetry: std::sync::Arc<telemetry::Telemetry>,
    pub seq_engine: std::sync::Arc<sequence::SequenceEngine>,
    pub aligner: std::sync::Arc<alignment::SequenceAligner>,
    pub bio_pipeline: std::sync::Arc<pipeline::BioPipeline>,
}
impl AppState {
    pub async fn new(cfg: config::Config) -> anyhow::Result<Self> {
        let c = std::sync::Arc::new(tokio::sync::RwLock::new(cfg));
        let t = std::sync::Arc::new(telemetry::Telemetry::new(std::sync::Arc::clone(&c)));
        let s = std::sync::Arc::new(sequence::SequenceEngine::new(std::sync::Arc::clone(&c)));
        let a = std::sync::Arc::new(alignment::SequenceAligner::new(std::sync::Arc::clone(&c)));
        let p = std::sync::Arc::new(pipeline::BioPipeline::new(
            std::sync::Arc::clone(&c),
            std::sync::Arc::clone(&s),
            std::sync::Arc::clone(&a),
        ));
        tracing::info!("kairos-bio AppState initialized");
        Ok(Self {
            config: c,
            telemetry: t,
            seq_engine: s,
            aligner: a,
            bio_pipeline: p,
        })
    }
}
