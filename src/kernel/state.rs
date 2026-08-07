use crate::planet::traits::Planet;

pub enum State {
    Boot,
    Deploy,
    Maneuver(Box<dyn Planet>),
    Capture,
    Sceince,
    Safe
}