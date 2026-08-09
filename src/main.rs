use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    framework::{battery::Battery, gyro::Gyro, stab::Stab},
    ksp::world::World,
    kernel::{
        clock::MissionClock,
        scheduler::{Scheduler, Task},
        watchdog::Watchdog,
    },
};

mod fdir;
mod framework;
mod hal;
mod kernel;
mod ksp;
mod planet;
mod cdh;

fn main() {
    let world = Arc::new(World::new().expect("world connect failed"));
    let battery = Battery::new(Arc::clone(&world));
    let gyro = Gyro::new(Arc::clone(&world));


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
    // tick 20ms: the stab loop is sampling-limited (limit cycle ~ T^2),
    // and each stab update is only ~1ms of kRPC calls (measured live).
    let mut scheduler = Scheduler::new(8, 20, Box::new(clock));

    // Read + parse commands from stdin on a dedicated thread.
    let bus = scheduler.bus();
    thread::spawn(move || crate::cdh::read_stdin(&bus));
    
    scheduler.add_task(Task::new(Box::new(battery), 1000)).unwrap();
    scheduler.add_task(Task::new(Box::new(gyro), 100)).unwrap();
    scheduler
        .add_task(Task::new(Box::new(Stab::new(scheduler.bus(), Arc::clone(&world))), 20))
        .unwrap();

    scheduler.run(watchdog);
}