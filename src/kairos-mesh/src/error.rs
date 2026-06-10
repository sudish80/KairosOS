use thiserror::Error;
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("WireGuard error: {0}")]
    Wg(String),
    #[error("Discovery error: {0}")]
    Discovery(String),
    #[error("Consensus error: {0}")]
    Consensus(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
