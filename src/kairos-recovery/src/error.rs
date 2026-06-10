use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecoveryError {
    #[error("Partition error: {0}")]
    Partition(String),
    #[error("Verity error: {0}")]
    Verity(String),
    #[error("Boot error: {0}")]
    Boot(String),
    #[error("Update error: {0}")]
    Update(String),
    #[error("Health check failed: {0}")]
    HealthCheck(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
