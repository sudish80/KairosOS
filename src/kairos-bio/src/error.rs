use thiserror::Error;
#[derive(Error, Debug)] pub enum BioError { #[error("Sequence error: {0}")] Sequence(String), #[error("Alignment error: {0}")] Alignment(String), #[error("Pipeline error: {0}")] Pipeline(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
