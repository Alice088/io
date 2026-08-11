use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    adcs::{gyro::Gyro, orientation::Orientation, stab::Stab}, cdh::clock::MissionClock, eps::battery::Battery, fdir::watchdog::Watchdog, fsw::scheduler::{Scheduler, Task}, hal::orientation, ksp::world::World,
};

mod adcs;
mod cdh;
mod eps;
mod fdir;
mod fsw;
mod hal;
mod ksp;
mod ttc;

fn main() {
    let world = Arc::new(World::new().expect("world connect failed"));

    let battery = Battery::new(Arc::clone(&world));
    let gyro = Gyro::new(Arc::clone(&world));
    let orientation = Orientation::new(Arc::clone(&world));


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
    let mut scheduler = Scheduler::new(8, 20, Box::new(clock));

    let bus = scheduler.bus();
    let b = Arc::clone(&bus);
    thread::spawn(move || crate::cdh::read_stdin(&b));

    let stab = Stab::new(Arc::clone(&bus), Arc::clone(&world), Box::new(orientation));
    
    scheduler.add_task(Task::new(Box::new(battery), 1000)).unwrap();
    scheduler.add_task(Task::new(Box::new(gyro), 100)).unwrap();
    scheduler.add_task(Task::new(Box::new(stab), 100)).unwrap();

    scheduler.run(watchdog);
}