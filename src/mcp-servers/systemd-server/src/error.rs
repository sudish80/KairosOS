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
    #[error("Service action failed: {0}")]
    ServiceActionFailed(String),
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Action not allowed: {0}")]
    ActionNotAllowed(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl McpError {
    pub fn code(&self) -> i32 {
        match self {
            Self::MethodNotFound(_) => -32601,
            Self::InvalidParams(_) => -32602,
            Self::Internal(_) => -32603,
            Self::Io(_) => -32000,
            Self::ServiceActionFailed(_) => -32001,
            Self::ServiceNotFound(_) => -32002,
            Self::ActionNotAllowed(_) => -32003,
            Self::Json(_) => -32700,
            Self::Utf8(_) => -32004,
        }
    }
}
