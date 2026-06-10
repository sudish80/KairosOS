use thiserror::Error;
#[derive(Error, Debug)] pub enum AvionicsError { #[error("ARINC error: {0}")] Arinc(String), #[error("MAVLink error: {0}")] Mavlink(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
