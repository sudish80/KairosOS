use thiserror::Error;
#[derive(Error, Debug)] pub enum QuantumError { #[error("Gate error: {0}")] Gate(String), #[error("Simulation error: {0}")] Simulation(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
