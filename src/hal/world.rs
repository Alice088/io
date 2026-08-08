//! Единственное место проекта, где живут async и stayputnik/kRPC.
//!
//! `World` — граница между синхронным верхом (framework/kernel/main) и
//! async-низом (kRPC). Внутри: отдельный поток со своим
//! current_thread-tokio-runtime, stayputnik-клиент и фоновый поллер
//! телеметрии, который публикует свежие снимки. Публичный API — только
//! синхронный, поэтому async не просачивается на верхние уровни.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use stayputnik::services::space_center::Vessel;

use crate::{
    fdir::{error::Error, reason::Reason},
    hal::battery::Battery,
};

const RESOURCE_EC: &str = "ElectricCharge";

/// Снимки состояния мира: по одному слоту на устройство.
/// Каждый слот хранит последний исход опроса (Ok или ошибку).
struct Shared {
    battery: Arc<Mutex<Result<Battery, Error>>>,
}

pub struct World {
    shared: Arc<Shared>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

/// Синхронный дескриптор последнего снимка батареи, который публикует World.
///
/// `World` должен жить дольше дескриптора (в `main` — держать `World`
/// в переменной до конца миссии), иначе поллер остановится.
pub struct BatteryHandle {
    snapshot: Arc<Mutex<Result<Battery, Error>>>,
}

impl BatteryHandle {
    pub fn get(&self) -> Result<Battery, Error> {
        let guard = self
            .snapshot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match &*guard {
            Ok(b) => Ok(Battery {
                amount: b.amount,
                max_amount: b.max_amount,
            }),
            Err(e) => Err(e.clone()),
        }
    }
}

impl World {
    /// Подключается к kRPC и запускает фоновый поллер.
    ///
    /// Блокирует вызывающий поток до первого снимка (таймаут 10 с), чтобы
    /// гарантировать работоспособность мира при старте — как раньше это
    /// делал `main` через `Client::connect().await?`.
    pub fn new() -> Result<World, Error> {
        let shared = Arc::new(Shared {
            battery: Arc::new(Mutex::new(Err(Error::Hardware(Reason::BatteryFault)))),
        });
        let shutdown = Arc::new(AtomicBool::new(false));

        let (ready_tx, ready_rx): (SyncSender<Result<(), Error>>, Receiver<Result<(), Error>>) =
            sync_channel(1);

        let worker_shared = Arc::clone(&shared);
        let worker_shutdown = Arc::clone(&shutdown);
        let name = "io-sat".to_string();
        let address = "127.0.0.1".to_string();

        let handle = thread::spawn(move || {
            poller(
                worker_shared,
                worker_shutdown,
                name,
                address,
                50_000,
                100,
                ready_tx,
            );
        });

        ready_rx
            .recv_timeout(Duration::from_secs(10))
            .map_err(|_| Error::Hardware(Reason::BatteryFault))??;

        Ok(World {
            shared,
            shutdown,
            worker: Some(handle),
        })
    }

    /// Дескриптор батареи для компонентов.
    ///
    /// `World` должен жить дольше дескриптора (в `main` — держать `World`
    /// в переменной до конца миссии).
    pub fn battery_hal(&self) -> BatteryHandle {
        BatteryHandle {
            snapshot: Arc::clone(&self.shared.battery),
        }
    }
}

impl Drop for World {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

/// Фоновый поток: свой current_thread-runtime, в нём живёт весь async.
fn poller(
    shared: Arc<Shared>,
    shutdown: Arc<AtomicBool>,
    name: String,
    address: String,
    port: u16,
    poll_ms: u64,
    ready: SyncSender<Result<(), Error>>,
) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => {
            let _ = ready.send(Err(Error::Hardware(Reason::BatteryFault)));
            return;
        }
    };

    let ready_inner = ready.clone();
    let startup = rt.block_on(async move {
        let client = stayputnik::Client::connect(&name, &address, port)
            .await
            .map_err(|_| Error::Hardware(Reason::BatteryFault))?
            .into_shared();

        let sc = stayputnik::services::space_center::SpaceCenter::new(client);
        let vessel = sc
            .active_vessel()
            .await
            .map_err(|_| Error::Hardware(Reason::BatteryFault))?;

        publish(&shared, fetch(&vessel).await);
        let _ = ready_inner.send(Ok(()));

        let mut ticker = tokio::time::interval(Duration::from_millis(poll_ms));
        loop {
            ticker.tick().await;
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            publish(&shared, fetch(&vessel).await);
        }

        Ok(())
    });

    if let Err(e) = startup {
        let _ = ready.send(Err(e));
    }
}

fn publish(shared: &Shared, result: Result<Battery, Error>) {
    *shared.battery.lock().unwrap() = result;
}

async fn fetch(vessel: &Vessel) -> Result<Battery, Error> {
    let resources = vessel
        .resources()
        .await
        .map_err(|_| Error::Hardware(Reason::BatteryFault))?;

    let amount = resources
        .amount(RESOURCE_EC)
        .await
        .map_err(|_| Error::Hardware(Reason::BatteryFault))?;

    let max_amount = resources
        .max(RESOURCE_EC)
        .await
        .map_err(|_| Error::Hardware(Reason::BatteryFault))?;

    Ok(Battery {
        amount: amount as f32,
        max_amount: max_amount as f32,
    })
}
