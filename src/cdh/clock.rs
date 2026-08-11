use std::time::Instant;

pub type Ms = u64;

pub struct MissionClock {
    time: Instant,
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

    pub fn sec(&self) -> u64 {
        self.time.elapsed().as_secs()
    }
}
