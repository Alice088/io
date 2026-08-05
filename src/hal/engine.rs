use stayputnik::services::space_center::{
    Control,
    Vessel,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PropulsionHalError {
    #[error("kRPC propulsion error: {0}")]
    Stayputnik(
        #[from]
        stayputnik::Error,
    ),

    #[error(
        "invalid throttle {0}; expected 0.0..=1.0"
    )]
    InvalidThrottle(f32),
}

pub struct PropulsionHal {
    control: Control,
}

impl PropulsionHal {
    pub async fn open(
        vessel: &Vessel,
        _update_hz: f32,
    ) -> Result<Self, PropulsionHalError> {
        let control = vessel.control().await?;

        Ok(Self {
            control,
        })
    }

    pub async fn set_throttle(
        &self,
        value: f32,
    ) -> Result<(), PropulsionHalError> {
        if !value.is_finite()
            || !(0.0..=1.0).contains(&value)
        {
            return Err(
                PropulsionHalError::InvalidThrottle(value),
            );
        }

        self.control
            .set_throttle(value)
            .await?;

        Ok(())
    }

    pub async fn stop(
        &self,
    ) -> Result<(), PropulsionHalError> {
        self.set_throttle(0.0).await
    }

    pub async fn activate_next_stage(
        &self,
    ) -> Result<usize, PropulsionHalError> {
        let detached = self
            .control
            .activate_next_stage()
            .await?;

        Ok(detached.len())
    }
}