use thiserror::Error;
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Vector error: {0}")]
    Vector(String),
    #[error("Query error: {0}")]
    Query(String),
    #[error("Bus error: {0}")]
    Bus(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
