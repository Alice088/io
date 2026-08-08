use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    framework::{battery::Battery, gyro::Gyro}, hal::world::World, kernel::{
        clock::MissionClock,
        scheduler::{Scheduler, Task},
        watchdog::Watchdog,
    },
};

mod fdir;
mod framework;
mod hal;
mod kernel;
mod planet;

fn main() {
    let world = World::new().expect("world connect failed");
    let battery = Battery::new(&world);
    let gyro = Gyro::new(&world);


    let watchdog = Arc::new(Mutex::new(Watchdog::new(500)));
    let wd = Arc::clone(&watchdog);

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(20));

        if wd.lock().unwrap().expired() {
            println!("WATCHDOG RESET");
            std::process::exit(1);
        }
    });

    let clock = MissionClock::start();
    let mut scheduler = Scheduler::new(8, 100, Box::new(clock));
    scheduler
        .add_task(Task::new(Box::new(battery), 1000))
        .expect("task limit exceeded");
    scheduler
        .add_task(Task::new(Box::new(gyro), 100))
        .expect("task limit exceeded");

    scheduler.run(watchdog);
}