use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::kernel::clock;
use crate::kernel::scheduler::Scheduler;
use crate::kernel::watchdog::Watchdog;
mod kernel;

use kernel::task::Task;

fn main() {
    let watchdog = Arc::new(Mutex::new(Watchdog::new(4000)));

    let wd = watchdog.clone();

    let tasks = [
        Task::new("task1", 1000, || {
            println!("task1");
        }),
        Task::new("task2", 2000, || {
            println!("task2");
        }),
        Task::new("task3", 2000, || {
            println!("task3");
        }),
        Task::new("task4", 2000, || {
            std::thread::sleep(std::time::Duration::from_millis(5000));
            println!("task4");
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
