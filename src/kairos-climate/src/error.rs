use thiserror::Error;
#[derive(Error, Debug)]
pub enum ClimateError { #[error("Assimilation error: {0}")] Assimilation(String), #[error("Model error: {0}")] Model(String), #[error("Forecast error: {0}")] Forecast(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
