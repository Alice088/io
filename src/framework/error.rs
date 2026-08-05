use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlightError {
    #[error("hardware failure by {0}")]
    HardwareFailure(Reason),

    #[error("invalide state by {0}")]
    InvalidState(Reason),
}

#[derive(Debug, Clone, Error)]
pub enum Reason {
    
    #[error("battery fault")]
    BatteryFault
}