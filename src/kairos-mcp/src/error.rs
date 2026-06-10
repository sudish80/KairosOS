//! Error types for the MCP daemon
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
