use thiserror::Error;
#[derive(Error, Debug)]
pub enum BuildError { #[error("Build error: {0}")] Build(String), #[error("Image error: {0}")] Image(String), #[error("Package error: {0}")] Package(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
