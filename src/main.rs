use std::{sync::Arc, thread, time::Duration};

use stayputnik::services::space_center::SpaceCenter;
use tokio::sync::Mutex;

use crate::{
    framework::battery::Battery,
    hal::battery::BatteryHal,
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
mod planet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = stayputnik::Client::connect("io-sat", "127.0.0.1", 50_000)
        .await?
        .into_shared();
    let sc = SpaceCenter::new(client);

    let vessel = sc.active_vessel().await?;
    let battery = Battery::new(BatteryHal::new(vessel));

    let watchdog = Arc::new(Mutex::new(Watchdog::new(500)));
    let wd = watchdog.clone();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(20));

        if wd.blocking_lock().expired() {
            println!("WATCHDOG RESET");
            std::process::exit(1);
        }
    });

    let clock = MissionClock::start();
    let mut scheduler = Scheduler::new(8, 100, Box::new(clock));
    scheduler
        .add_task(Task::new(Box::new(battery), 1000))
        .unwrap();

    scheduler.run(watchdog.clone()).await;

    Ok(())
}
