use crate::{framework::error, hal::krpc::ELECTRIC_CHARGE};
use stayputnik::services::space_center::Vessel;

pub struct BatteryHal {
    vessel: &'static Vessel,
}

pub struct Battery {
    pub amount: f32,
    pub max_amount: f32,
}


impl BatteryHal {
    pub fn new(&self, vessel: &'static Vessel) -> BatteryHal {
        BatteryHal { vessel }
    }

    pub async fn get(&self) -> Result<Battery, error::FlightError> {
        let resources = self.vessel
            .resources()
            .await
            .map_err(|_| error::FlightError::HardwareFailure(error::Reason::BatteryFault))?;


        let amount = resources
            .amount(ELECTRIC_CHARGE)
            .await
            .map_err(|_| error::FlightError::HardwareFailure(error::Reason::BatteryFault))?;


        let max_amount = resources
            .max(ELECTRIC_CHARGE)
            .await
            .map_err(|_| error::FlightError::HardwareFailure(error::Reason::BatteryFault))?;


        Ok(Battery {
            amount: amount as f32,
            max_amount: max_amount as f32,
        })
    }
}
