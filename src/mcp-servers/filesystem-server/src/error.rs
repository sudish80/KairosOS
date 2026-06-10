use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path traversal denied: {0}")]
    PathTraversal(String),
    #[error("File too large: {0} > max")]
    FileTooLarge(u64),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl McpError {
    pub fn code(&self) -> i32 {
        match self {
            Self::MethodNotFound(_) => -32601,
            Self::InvalidParams(_) => -32602,
            Self::Internal(_) => -32603,
            Self::Io(_) => -32000,
            Self::PathTraversal(_) => -32001,
            Self::FileTooLarge(_) => -32002,
            Self::Json(_) => -32700,
        }
    }
}
