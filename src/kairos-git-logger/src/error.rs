use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitLoggerError {
    #[error("Git error: {0}")]
    Git(String),
    #[error("Watcher error: {0}")]
    Watcher(String),
    #[error("Commit error: {0}")]
    Commit(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Repo error: {0}")]
    Repo(String),
}
