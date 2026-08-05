mod config;
mod error;
mod resources;
mod session;

pub use config::KrpcConfig;
pub use error::{
    Result,
};
pub use resources::*;
pub use session::KrpcSession;

use crate::hal::{
    AttitudeHal,
    power::BatteryHal,
    propulsion::PropulsionHal,
    telemetry::TelemetryHal,
};

pub struct Krpc {
    session: KrpcSession,

    battery: BatteryHal,
    propulsion: PropulsionHal,
    attitude: AttitudeHal,
    telemetry: TelemetryHal,

    config: KrpcConfig,
}

impl Krpc {
    pub async fn connect(
        config: KrpcConfig,
    ) -> Result<Self> {
        let session = KrpcSession::connect(&config).await?;

        let vessel = session.active_vessel().await?;

        let battery = BatteryHal::open(
            &vessel,
            config.battery_hz,
        )
        .await?;

        let propulsion = PropulsionHal::open(
            &vessel,
            config.propulsion_hz,
        )
        .await?;

        let attitude = AttitudeHal::open(
            &vessel,
            config.attitude_hz,
        )
        .await?;

        let telemetry = TelemetryHal::open(
            session.space_center(),
            &vessel,
            config.navigation_hz,
        )
        .await?;

        Ok(Self {
            session,

            battery,
            propulsion,
            attitude,
            telemetry,

            config,
        })
    }

    pub fn battery(&mut self) -> &mut BatteryHal {
        &mut self.battery
    }

    pub fn propulsion(&mut self) -> &mut PropulsionHal {
        &mut self.propulsion
    }

    pub fn attitude(&mut self) -> &mut AttitudeHal {
        &mut self.attitude
    }

    pub fn telemetry(&mut self) -> &mut TelemetryHal {
        &mut self.telemetry
    }

    pub async fn refresh_vessel(&mut self) -> Result<()> {
        let vessel = self.session.active_vessel().await?;

        let battery = BatteryHal::open(
            &vessel,
            self.config.battery_hz,
        )
        .await?;

        let propulsion = PropulsionHal::open(
            &vessel,
            self.config.propulsion_hz,
        )
        .await?;

        let attitude = AttitudeHal::open(
            &vessel,
            self.config.attitude_hz,
        )
        .await?;

        let telemetry = TelemetryHal::open(
            self.session.space_center(),
            &vessel,
            self.config.navigation_hz,
        )
        .await?;

        self.battery = battery;
        self.propulsion = propulsion;
        self.attitude = attitude;
        self.telemetry = telemetry;

        Ok(())
    }
}