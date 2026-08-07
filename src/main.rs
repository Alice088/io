use std::sync::{Arc};

use tokio::{sync::Mutex, time::error::Error};

use crate::kernel::{scheduler::{Scheduler, Task}, watchdog::Watchdog};

mod hal;
mod kernel;
mod planet;
mod fdir;
mod framework;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let watchdog = Arc::new(Mutex::new(Watchdog::new(4000)));
    let scheduler = Scheduler::new(50, 1);

    let 
    scheduler.add_task(Task::new(, period)).unwrap();
    
    Ok(())
}
