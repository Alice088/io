use std::time::{Duration, Instant};

pub type Ms = u64;

pub struct MissionClock {
    pub time: Instant,
}

impl MissionClock {
    pub fn start() -> Self {
        Self {
            time: Instant::now(),
        }
    }

    pub fn ms(&self) -> Ms {
        self.time.elapsed().as_millis() as Ms
    }

    pub fn sleep_ms(ms: Ms) {
        std::thread::sleep(Duration::from_millis(ms));
    }
}
