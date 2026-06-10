use thiserror::Error;
#[derive(Error, Debug)]
pub enum FinanceError { #[error("Market error: {0}")] Market(String), #[error("Trading error: {0}")] Trading(String), #[error("Risk error: {0}")] Risk(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
