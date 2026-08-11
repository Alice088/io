//! Only place with tokio-runtime + kRPC connection.
//!
//! `World` is a sync facade over the game: `battery()`, `gyro()` and
//! universal `call(...)` block the caller and return fresh data. A worker
//! thread owns the runtime, kRPC client and active vessel, and runs jobs
//! one at a time. HAL code stays sync via `Executor::block_on`.

use std::{
    any::Any,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread::{self, JoinHandle},
    time::Duration,
};

use stayputnik::services::space_center::Vessel;
use stayputnik::ClientRef;

use crate::{
    fdir::{error::Error, reason::Reason},
    hal::{
        battery::{Battery, BatteryHal},
        gyro::{Gyro, GyroHal},
        orientation::OrientationHal
    },
    ksp::exec::Executor,
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const CALL_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_CAPACITY: usize = 16;

/// Worker job: closure gets (exec, vessel, client), calls the game and
/// sends the result via the captured channel.
///
/// `client` builds any stayputnik service (`SpaceCenter::new(client.clone())`,
/// `UI::new(...)`, ...); `vessel` has all vessel data.
type Job = Box<dyn FnOnce(&Executor, &Vessel, &ClientRef) + Send>;

pub struct World {
    tx: Option<SyncSender<Job>>,
    worker: Option<JoinHandle<()>>,
}

impl World {
    /// Connects to kRPC and spawns the worker thread. Blocks until
    /// connected (timeout `CONNECT_TIMEOUT`).
    pub fn new() -> Result<World, Error> {
        let (tx, rx) = sync_channel(REQUEST_CAPACITY);
        let (ready_tx, ready_rx): (SyncSender<Result<(), Error>>, Receiver<Result<(), Error>>) =
            sync_channel(1);

        let name = "io-sat".to_string();
        let address = "127.0.0.1".to_string();

        let worker = thread::spawn(move || worker(rx, name, address, 50_000, ready_tx));

        ready_rx
            .recv_timeout(CONNECT_TIMEOUT)
            .map_err(|_| Error::Hardware(Reason::LinkLost))??;

        Ok(World {
            tx: Some(tx),
            worker: Some(worker),
        })
    }

    /// Fresh battery charge (blocking kRPC call).
    pub fn battery(&self) -> Result<Battery, Error> {
        self.call(|exec, vessel, _| {
            BatteryHal::new(vessel.clone(), exec.clone()).get()
        })?
    }

    /// Fresh angular velocities (blocking kRPC call).
    pub fn gyro(&self) -> Result<Gyro, Error> {
        self.call(|exec, vessel, _| {
            GyroHal::new(vessel.clone(), exec.clone()).get()
        })?
    }

    /// Applies roll/pitch/yaw control inputs, each in [-1; 1] (blocking kRPC call).
    pub fn set_orientation(&self, roll: f64, pitch: f64, yaw: f64) -> Result<(), Error> {
        self.call(move |exec, vessel, _| {
            OrientationHal::new(vessel.clone(), exec.clone()).set(roll, pitch, yaw)
        })?
    }

    /// Universal sync call.
    ///
    /// Runs `job` in the worker thread with:
    /// - `exec` — async executor (`exec.block_on(any_call)`);
    /// - `vessel` — active vessel;
    /// - `client` — kRPC client for any stayputnik service.
    ///
    /// Blocks until `job` finishes (timeout `CALL_TIMEOUT`). Transport
    /// errors return `Error::Hardware(Reason::LinkLost)`; `job` errors
    /// pass through. The closure must be `Send + 'static`.
    pub fn call<T: Send + 'static>(
        &self,
        job: impl FnOnce(&Executor, &Vessel, &ClientRef) -> T + Send + 'static,
    ) -> Result<T, Error> {
        let (tx, rx) = sync_channel(1);

        let job: Job = Box::new(move |exec: &Executor, vessel: &Vessel, client: &ClientRef| {
            let _ = tx.send(Box::new(job(exec, vessel, client)) as Box<dyn Any + Send>);
        });

        self.tx
            .as_ref()
            .ok_or(Error::Hardware(Reason::LinkLost))?
            .send(job)
            .map_err(|_| Error::Hardware(Reason::LinkLost))?;

        let reply = rx
            .recv_timeout(CALL_TIMEOUT)
            .map_err(|_| Error::Hardware(Reason::LinkLost))?;

        reply
            .downcast::<T>()
            .map(|boxed| *boxed)
            .map_err(|_| Error::Hardware(Reason::LinkLost))
    }
}

impl Drop for World {
    fn drop(&mut self) {
        // Close channel so the worker exits.
        self.tx.take();
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

/// Worker thread: owns the current_thread runtime; all async lives here.
fn worker(
    rx: Receiver<Job>,
    name: String,
    address: String,
    port: u16,
    ready: SyncSender<Result<(), Error>>,
) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => {
            let _ = ready.send(Err(Error::Hardware(Reason::LinkLost)));
            return;
        }
    };

    // Connect to kRPC; block until ready.
    let connected = rt.block_on(async {
        let client = stayputnik::Client::connect(&name, &address, port)
            .await
            .map_err(|_| Error::Hardware(Reason::LinkLost))?
            .into_shared();

        let sc = stayputnik::services::space_center::SpaceCenter::new(client.clone());
        let vessel = sc
            .active_vessel()
            .await
            .map_err(|_| Error::Hardware(Reason::LinkLost))?;

        Ok::<_, Error>((client, vessel))
    });

    let (client, vessel) = match connected {
        Ok(pair) => pair,
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };

    // Move runtime into Executor for HAL devices.
    let exec = Executor::new(rt);
    let _ = ready.send(Ok(()));

    // Serve jobs one by one.
    while let Ok(job) = rx.recv() {
        job(&exec, &vessel, &client);
    }
}
