use stayputnik::services::space_center::{
    AutoPilot,
    Control,
    Vessel,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttitudeHalError {
    #[error("kRPC attitude error: {0}")]
    Stayputnik(
        #[from]
        stayputnik::Error,
    ),

    #[error(
        "invalid {name} value {value}; expected {min}..={max}"
    )]
    InvalidControlValue {
        name: &'static str,
        value: f32,
        min: f32,
        max: f32,
    },
}

pub struct AttitudeHal {
    control: Control,
    autopilot: AutoPilot,
}

impl AttitudeHal {
    pub async fn open(
        vessel: &Vessel,
        _update_hz: f32,
    ) -> Result<Self, AttitudeHalError> {
        let control = vessel.control().await?;
        let autopilot = vessel.auto_pilot().await?;

        Ok(Self {
            control,
            autopilot,
        })
    }

    pub async fn set_manual(
        &self,
        pitch: f32,
        yaw: f32,
        roll: f32,
    ) -> Result<(), AttitudeHalError> {
        let pitch = validate(
            "pitch",
            pitch,
            -1.0,
            1.0,
        )?;

        let yaw = validate(
            "yaw",
            yaw,
            -1.0,
            1.0,
        )?;

        let roll = validate(
            "roll",
            roll,
            -1.0,
            1.0,
        )?;

        self.control.set_pitch(pitch).await?;
        self.control.set_yaw(yaw).await?;
        self.control.set_roll(roll).await?;

        Ok(())
    }

    pub async fn point_to(
        &self,
        pitch_deg: f32,
        heading_deg: f32,
    ) -> Result<(), AttitudeHalError> {
        let pitch_deg = validate(
            "target pitch",
            pitch_deg,
            -90.0,
            90.0,
        )?;

        let heading_deg = validate(
            "target heading",
            heading_deg,
            0.0,
            360.0,
        )?;

        self.autopilot
            .target_pitch_and_heading(
                pitch_deg,
                heading_deg,
            )
            .await?;

        self.autopilot.engage().await?;

        Ok(())
    }

    pub async fn disengage(
        &self,
    ) -> Result<(), AttitudeHalError> {
        self.autopilot.disengage().await?;

        Ok(())
    }
}

fn validate(
    name: &'static str,
    value: f32,
    min: f32,
    max: f32,
) -> Result<f32, AttitudeHalError> {
    if !value.is_finite()
        || value < min
        || value > max
    {
        return Err(
            AttitudeHalError::InvalidControlValue {
                name,
                value,
                min,
                max,
            },
        );
    }

    Ok(value)
}