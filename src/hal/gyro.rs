use stayputnik::services::space_center::{ReferenceFrame, Vessel};

use crate::fdir::{error::Error, reason::Reason};

pub struct GyroHal {
    vessel: Vessel,
}

pub struct Gyro {
    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
}

impl Gyro {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            roll: x,
            pitch: y,
            yaw: z,
        }
    }
}

impl GyroHal {
    pub fn new(vessel: Vessel) -> GyroHal {
        GyroHal { vessel }
    }

    pub async fn get(&self) -> Result<Gyro, Error> {
        let frame = self
            .vessel
            .reference_frame()
            .await
            .map_err(|_| Error::Hardware(Reason::GyroFault))?;

        let (wx, wy, wz) = self
            .vessel
            .angular_velocity(&frame)
            .await
            .map_err(|_| Error::Hardware(Reason::GyroFault))?;

        Ok(Gyro::new(wx, wy, wz))
    }
}
