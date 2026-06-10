use thiserror::Error;
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("DAG error: {0}")] Dag(String),
    #[error("Scheduler error: {0}")] Scheduler(String),
    #[error("Executor error: {0}")] Executor(String),
    #[error("Resource error: {0}")] Resource(String),
    #[error("Pipeline error: {0}")] Pipeline(String),
    #[error("IO error: {0}")] Io(#[from] std::io::Error),
}
