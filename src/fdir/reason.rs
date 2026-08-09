use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum Reason {
    #[error("max scheduler tasks")]
    MaxScdulerTasks,

    #[error("battery fault")]
    BatteryFault,

    #[error("gyro fault")]
    GyroFault,

    #[error("orientation control fault")]
    ControlFault,

    #[error("link to the game lost")]
    LinkLost,
}