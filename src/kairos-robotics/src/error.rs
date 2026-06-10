use thiserror::Error;
#[derive(Error, Debug)]
pub enum RoboticsError {
    #[error("Control error: {0}")]
    Control(String),
    #[error("Kinematics error: {0}")]
    Kinematics(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
