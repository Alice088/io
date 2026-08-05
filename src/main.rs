use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::kernel::clock;
use crate::kernel::scheduler::Scheduler;
use crate::kernel::watchdog::Watchdog;
mod kernel;
mod framework;

use kernel::task::Task;

fn main() {
    let watchdog = Arc::new(Mutex::new(Watchdog::new(4000)));

    let wd = watchdog.clone();

    let tasks = [
        Task::new("Gyro", 1000, || {
            println!("task1");
        }),
    ];

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(20));

        let watchdog = wd.lock().unwrap();

        if watchdog.expired() {
            println!("WATCHDOG RESET");

            std::process::exit(1);
        }
    });

    let mut scheduler = Scheduler::new(tasks);

    let c = clock::MissionClock::start();

    scheduler.run(&c, watchdog);
}