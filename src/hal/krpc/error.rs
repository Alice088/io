use stayputnik::services::krpc::GameScene;
use thiserror::Error;

use crate::hal::{
    attitude::AttitudeHalError,
    power::BatteryHalError,
    propulsion::PropulsionHalError,
};

#[derive(Debug, Error)]
pub enum KrpcError {
    #[error("kRPC error: {0}")]
    Stayputnik(
        #[from]
        stayputnik::Error,
    ),

    #[error("KSP is not in Flight scene: {0:?}")]
    NotInFlight(GameScene),

    #[error("battery HAL error: {0}")]
    Battery(
        #[from]
        BatteryHalError,
    ),

    #[error("propulsion HAL error: {0}")]
    Propulsion(
        #[from]
        PropulsionHalError,
    ),

    #[error("attitude HAL error: {0}")]
    Attitude(
        #[from]
        AttitudeHalError,
    ),
}

pub type Result<T> =
    std::result::Result<T, KrpcError>;