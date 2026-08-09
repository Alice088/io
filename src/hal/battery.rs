//! Battery HAL device. Sync code via `self.exec.block_on(...)`.
//! Runs in the worker thread; top level uses `World::battery()`.

use stayputnik::services::space_center::Vessel;

use crate::{
    fdir::{error::Error, reason::Reason},
    ksp::exec::Executor,
};

const RESOURCE_EC: &str = "ElectricCharge";

pub struct BatteryHal {
    vessel: Vessel,
    exec: Executor,
}

pub struct Battery {
    pub amount: f32,
    pub max_amount: f32,
}

// MADE BY GENIUS KERBIN-GOSHA; Я БЛЯТЬ СИДЕЛ СУКА.
impl BatteryHal {
    pub fn new(vessel: Vessel, exec: Executor) -> BatteryHal {
        BatteryHal { vessel, exec }
    }

    /// Reads battery charge (blocking).
    pub fn get(&self) -> Result<Battery, Error> {
        let resources = self
            .exec
            .block_on(self.vessel.resources())
            .map_err(|_| Error::Hardware(Reason::BatteryFault))?;

        let amount = self
            .exec
            .block_on(resources.amount(RESOURCE_EC))
            .map_err(|_| Error::Hardware(Reason::BatteryFault))?;

        let max_amount = self
            .exec
            .block_on(resources.max(RESOURCE_EC))
            .map_err(|_| Error::Hardware(Reason::BatteryFault))?;

        Ok(Battery {
            amount: amount as f32,
            max_amount: max_amount as f32,
        })
    }
}
