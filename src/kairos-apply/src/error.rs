use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplyError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Generation error: {0}")]
    Generation(String),
    #[error("Rollback error: {0}")]
    Rollback(String),
    #[error("Diff error: {0}")]
    Diff(String),
    #[error("State error: {0}")]
    State(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Health check failed: {0}")]
    HealthCheck(String),
}
