use thiserror::Error;
#[derive(Error, Debug)]
pub enum LlmError { #[error("Runtime error: {0}")] Runtime(String), #[error("Model error: {0}")] Model(String), #[error("Quantization error: {0}")] Quantization(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
