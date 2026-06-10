use thiserror::Error;

#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Pipeline error: {0}")]
    Pipeline(String),
    #[error("Quantization error: {0}")]
    Quantization(String),
    #[error("KV cache error: {0}")]
    KVCache(String),
    #[error("Speculation rejected after {0} attempts")]
    SpeculationRejected(usize),
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    #[error("Router error: {0}")]
    Router(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
