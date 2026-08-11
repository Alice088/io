//! Orientation HAL device. Sync code via `self.exec.block_on(...)`.
//! Runs in the worker thread; top level uses `World::set_orientation()`.

use stayputnik::services::space_center::Vessel;

use crate::{
    fdir::{error::Error, reason::Reason},
    ksp::exec::Executor,
};

pub struct OrientationHal {
    vessel: Vessel,
    exec: Executor,
}

impl OrientationHal {
    pub fn new(vessel: Vessel, exec: Executor) -> OrientationHal {
        OrientationHal { vessel, exec }
    }

    pub fn set(&self, roll: f64, pitch: f64, yaw: f64) -> Result<(), Error> {
        let control = self
            .exec
            .block_on(self.vessel.control())
            .map_err(|_| Error::Hardware(Reason::ControlFault))?;

        self.exec
            .block_on(control.set_roll(roll as f32))
            .map_err(|_| Error::Hardware(Reason::ControlFault))?;
        self.exec
            .block_on(control.set_pitch(pitch as f32))
            .map_err(|_| Error::Hardware(Reason::ControlFault))?;
        self.exec
            .block_on(control.set_yaw(yaw as f32))
            .map_err(|_| Error::Hardware(Reason::ControlFault))?;

        Ok(())
    }
}
