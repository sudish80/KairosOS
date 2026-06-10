use thiserror::Error;
#[derive(Error, Debug)]
pub enum VisionError { #[error("Capture error: {0}")] Capture(String), #[error("Detection error: {0}")] Detection(String), #[error("Pipeline error: {0}")] Pipeline(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
