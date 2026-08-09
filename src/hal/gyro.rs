//! Gyro HAL device. Sync code via `self.exec.block_on(...)`.
//! Runs in the worker thread; top level uses `World::gyro()`.

use stayputnik::services::space_center::Vessel;

use crate::{
    fdir::{error::Error, reason::Reason},
    ksp::exec::Executor,
};

pub struct GyroHal {
    vessel: Vessel,
    exec: Executor,
}

pub struct Gyro {
    pub wx: f64,
    pub wy: f64,
    pub wz: f64,
}

impl Gyro {
    pub fn new(wx: f64, wy: f64, wz: f64) -> Self {
        Self {
            wx,
            wy,
            wz,
        }
    }
}

impl GyroHal {
    pub fn new(vessel: Vessel, exec: Executor) -> GyroHal {
        GyroHal { vessel, exec }
    }

    /// Reads angular velocities (blocking).
    pub fn get(&self) -> Result<Gyro, Error> {
        let frame = self
            .exec
            .block_on(self.vessel.orbital_reference_frame())
            .map_err(|_| Error::Hardware(Reason::GyroFault))?;

        let (x, y, z) = self
            .exec
            .block_on(self.vessel.angular_velocity(&frame))
            .map_err(|_| Error::Hardware(Reason::GyroFault))?;

        Ok(Gyro::new(deadzone(x), deadzone(y), deadzone(z)))
    }
}

fn deadzone(value: f64) -> f64 {
    if value.abs() > 0.001 { value } else { 0.0 }
}