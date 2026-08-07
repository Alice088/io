use std::{
    sync::Arc,
    time::Duration,
};

use tokio::{sync::Mutex, time::interval};

use crate::{
    fdir::{error::Error, reason::Reason}, framework::component::Component, kernel::{
        clock::{MissionClock, Ms}, watchdog::Watchdog,
    },
};

pub struct Task {
    name: &'static str,
    period: Ms,
    next: Ms,
    c: Box<dyn Component>,
}

impl Task {
    pub fn new(c: Box<dyn Component>, period: Ms) -> Self{
        Self { name: c.name(), period, next: 0, c }
    }
}


pub struct Scheduler {
    tasks: Vec<Task>,
    tasks_limit: u8,
    tick: Ms,
    clock: Box<MissionClock>
}

impl Scheduler {
    pub fn new(tasks_limit: u8, tick: Ms, clock: Box<MissionClock>) -> Scheduler {
        Scheduler {
            tasks_limit,
            tasks: Vec::new(),
            tick,
            clock
        }
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), Error> {
        if self.tasks.len() >= self.tasks_limit as usize {
            return Err(Error::Software(Reason::MaxScdulerTasks));
        }

        self.tasks.push(task);
        Ok(())
    }

    pub async fn run(&mut self, watchdog: Arc<Mutex<Watchdog>>) {
        let mut ticker = interval(Duration::from_millis(self.tick));

        loop {
            ticker.tick().await;

            {
                let mut wd = watchdog.lock().await;
                wd.kick();
            }

            let now = self.clock.ms();

            for task in self.tasks.iter_mut() {
                if now >= task.next {
                    println!("({}s){}: RUN {}", self.clock.sec(), now, task.name);
                    task.c.update().await;
                    task.next = now + task.period;
                }
            }
        }
    }
}
