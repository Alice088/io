use crate::kernel::clock::Ms;
use std::time::Duration;

pub struct Watchdog {
    last_kick: std::time::Instant,
    timeout: Duration,
}

impl Watchdog {
    pub fn new(timeout_ms: Ms) -> Self {
        Self {
            last_kick: std::time::Instant::now(),
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    pub fn kick(&mut self) {
        self.last_kick = std::time::Instant::now();
    }

    pub fn expired(&self) -> bool {
        self.last_kick.elapsed() > self.timeout
    }
}
