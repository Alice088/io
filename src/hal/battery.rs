use stayputnik::services::space_center::Vessel;


const RESOURCE_EC: &str = "ElectricCharge";

pub struct BatteryHal {
    vessel: Vessel,
}

pub struct Battery {
    pub amount: f32,
    pub max_amount: f32,
}

// MADE BY GENIUS KERBIN-GOSHA; Я БЛЯТЬ СИДЕЛ СУКА.
impl BatteryHal {
    pub fn new(vessel: Vessel) -> BatteryHal {
        BatteryHal { vessel }
    }

    pub async fn get(&self) -> Result<Battery, error::FlightError> {
        let resources = self
            .vessel
            .resources()
            .await
            .map_err(|_| error::FlightError::HardwareFailure(error::Reason::BatteryFault))?;

        let amount = resources
            .amount(RESOURCE_EC)
            .await
            .map_err(|_| error::FlightError::HardwareFailure(error::Reason::BatteryFault))?;

        let max_amount = resources
            .max(RESOURCE_EC)
            .await
            .map_err(|_| error::FlightError::HardwareFailure(error::Reason::BatteryFault))?;

        Ok(Battery {
            amount: amount as f32,
            max_amount: max_amount as f32,
        })
    }
}
