use crate::cdh::clock::Ms;
use std::time::Duration;

pub struct Watchdog {
    last_kick: std::time::Instant,
    deadline: Duration,
}

impl Watchdog {
    pub fn new(deadline: Ms) -> Self {
        Self {
            last_kick: std::time::Instant::now(),
            deadline: Duration::from_millis(deadline),
        }
    }

    pub fn kick(&mut self) {
        self.last_kick = std::time::Instant::now();
    }

    pub fn expired(&self) -> bool {
        self.last_kick.elapsed() > self.deadline
    }
}
