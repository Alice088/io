//! Gyro HAL device. Sync code via `self.exec.block_on(...)`.
//! Runs in the worker thread; top level uses `World::gyro()`.

use stayputnik::services::space_center::Vessel;

use crate::{
    fdir::{error::Error, reason::Reason},
    framework::gyro::Quaternion,
    ksp::exec::Executor,
};

pub struct GyroHal {
    vessel: Vessel,
    exec: Executor,
}

pub struct Gyro {
    pub angular_velocity: (f64, f64, f64),
    pub rotation: Quaternion,
}

impl Gyro {
    pub fn new(angular_velocity: (f64, f64, f64), rotation: Quaternion) -> Self {
        Self {
            angular_velocity,
            rotation,
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

        let angular_velocity = (deadzone(x), deadzone(y), deadzone(z));

        let quaternion = self
            .exec
            .block_on(self.vessel.rotation(&frame))
            .map_err(|_| Error::Hardware(Reason::GyroFault))?;


        Ok(Gyro::new(angular_velocity, quaternion))
    }
}

fn deadzone(value: f64) -> f64 {
    if value.abs() > 0.001 {
        value
    } else {
        0.0
    }
}
