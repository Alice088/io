use std::sync::{Arc, Mutex};

use crate::kernel::{
    clock::{MissionClock, Ms},
    task::Task,
    watchdog::Watchdog,
};

const TICK: Ms = 10;

pub struct Scheduler<const N: usize> {
    tasks: [Task; N],
}

impl<const N: usize> Scheduler<N> {
    pub fn new(tasks: [Task; N]) -> Self {
        Self { tasks }
    }

    pub async fn run(&mut self, clock: &MissionClock, watchdog: Arc<Mutex<Watchdog>>) {
        loop {
            {
                let mut wd = watchdog.lock().unwrap();
                wd.kick();
            }
            let now = clock.ms();

            for task in &mut self.tasks {
                if now.wrapping_sub(task.next) < Ms::MAX / 2 {
                    println!("[{} ms] RUN {}", now, task.name);

                    (task.callback)().await;

                    task.next = now + task.period;
                }
            }

            MissionClock::sleep_ms(TICK);
        }
    }
}
