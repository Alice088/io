use std::sync::{Arc, Mutex};

use crate::kernel::{
    clock::{MissionClock, Ms},
    task::{Task, TaskState},
    watchdog::Watchdog,
};

const TICK: Ms = 10;

pub struct Scheduler<const N: usize> {
    tasks: [Task; N],
    watchdog: Watchdog,
}

impl<const N: usize> Scheduler<N> {
    pub fn new(tasks: [Task; N]) -> Self {
        Self {
            tasks,
            watchdog: Watchdog::new(100),
        }
    }

    pub fn run(&mut self, clock: &MissionClock, watchdog: Arc<Mutex<Watchdog>>) {
        loop {
            {
                let mut wd = watchdog.lock().unwrap();
                wd.kick();
            }
            let now = clock.ms();

            for task in &mut self.tasks {
                if task.state == TaskState::Disabled {
                    continue;
                }

                if now.wrapping_sub(task.next) < Ms::MAX / 2 {
                    task.state = TaskState::Running;

                    println!("[{} ms] RUN {}", now, task.name);

                    (task.callback)();


                    task.state = TaskState::Ready;

                    task.next = now + task.period;
                }
            }
            
            MissionClock::sleep_ms(TICK);
        }
    }
}
