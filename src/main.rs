use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use hal::krpc;
use kernel::task::Task;
use stayputnik::services::space_center::SpaceCenter;

use crate::component::power::Power;
use crate::framework::component::Component;
use crate::hal::battery::BatteryHal;
use crate::kernel::clock;
use crate::kernel::scheduler::Scheduler;
use crate::kernel::watchdog::Watchdog;

mod component;
mod framework;
mod hal;
mod kernel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = krpc::KrpcConfig::default();
    let krpc = krpc::Krpc::connect(config).await?;
    let sc = SpaceCenter::new(krpc.client());

    let watchdog = Arc::new(Mutex::new(Watchdog::new(4000)));

    let wd = watchdog.clone();

    let battery_hal = BatteryHal::new(sc.active_vessel().await.expect("failed get vessel"));
    let power = Arc::new(tokio::sync::Mutex::new(Power::new(battery_hal, &[])));

    let tasks = [Task::new(
        "power",
        20,
        Box::new({
            let power = Arc::clone(&power);

            move || {
                let power = Arc::clone(&power);

                Box::pin(async move {
                    let mut power = power.lock().await;

                    match power.update().await {
                        Ok(events) => {
                            for event in events {
                                println!("EVENT: {:?}", event);
                            }
                        }

                        Err(e) => {
                            println!("POWER ERROR: {:?}", e);
                        }
                    }
                })
            }
        }),
    )];

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

    scheduler.run(&c, watchdog).await;

    drop(krpc);
    Ok(())
}
