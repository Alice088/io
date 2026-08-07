use std::{
    sync::{Arc},
    time::Duration,
};

use tokio::{sync::Mutex, time::interval};

use crate::{
    fdir::{error::Error, reason::Reason},
    framework::component::Component,
    kernel::{
        clock::Ms,
        watchdog::{self, Watchdog},
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
}

impl Scheduler {
    pub fn new(tasks_limit: u8, tick: Ms) -> Scheduler {
        Scheduler {
            tasks_limit,
            tasks: Vec::new(),
            tick,
        }
    }

    pub fn add_task(&mut self, task: Task) -> Option<Error> {
        if self.tasks.len() > 50 {
            return Some(Error::Software(Reason::MaxScdulerTasks));
        }

        self.tasks.push(task);
        None
    }

    pub async fn run(&mut self, watchdog: Arc<Mutex<Watchdog>>) {
        let mut ticker = interval(Duration::from_millis(self.tick));

        loop {
            ticker.tick().await;

            let mut wd = watchdog.lock().await;
            wd.kick();

            let now = std::time::Instant::now();

            for task in self.tasks.iter_mut() {
                if now.elapsed().as_millis() as Ms >= task.next {
                    let c = task.component.as_mut();
                    c.update();
                    task.next += task.period
                }
            }
        }
    }
}
