use thiserror::Error;
#[derive(Error, Debug)]
pub enum FbError { #[error("Canvas error: {0}")] Canvas(String), #[error("DRM error: {0}")] Drm(String), #[error("Compositor error: {0}")] Compositor(String), #[error("IO error: {0}")] Io(#[from] std::io::Error) }
