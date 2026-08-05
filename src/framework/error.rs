#[derive(Debug)]
pub enum FlightError {
    HardwareFailure(Reason),
    InvalidState(Reason),
}

#[derive(Debug, Clone)]
pub enum Reason {
    BatteryFault
}