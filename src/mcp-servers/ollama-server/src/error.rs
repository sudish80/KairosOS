use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Ollama API error: {0}")]
    OllamaApi(String),
    #[error("Request timeout")]
    Timeout,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl McpError {
    pub fn code(&self) -> i32 {
        match self {
            Self::MethodNotFound(_) => -32601,
            Self::InvalidParams(_) => -32602,
            Self::Internal(_) => -32603,
            Self::OllamaApi(_) => -32000,
            Self::Timeout => -32001,
            Self::Http(_) => -32002,
            Self::Json(_) => -32700,
        }
    }
}
