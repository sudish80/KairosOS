use thiserror::Error;

#[derive(Error, Debug)]
pub enum TuiError {
    #[error("Framebuffer error: {0}")]
    Framebuffer(String),
    #[error("DRM error: {0}")]
    Drm(String),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("Input error: {0}")]
    Input(String),
    #[error("Gesture error: {0}")]
    Gesture(String),
    #[error("Layout error: {0}")]
    Layout(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
