use std::{sync::Arc, thread, time::Duration};

use stayputnik::services::space_center::SpaceCenter;
use tokio::sync::Mutex;

use crate::{
    framework::{battery::Battery, component::Component},
    hal::battery::BatteryHal,
    kernel::{scheduler::{Scheduler, Task}, watchdog::Watchdog},
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

    let watchdog = Arc::new(Mutex::new(Watchdog::new(100000)));
    let wd = watchdog.clone();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(20));

        if wd.blocking_lock().expired() {
            println!("WATCHDOG RESET");
            std::process::exit(1);
        }
    });

    let mut scheduler = Scheduler::new(8, 50);
    scheduler.add_task(Task::new(Box::new(battery), 1)).unwrap();

    scheduler.run(watchdog.clone()).await;

    Ok(())
}
