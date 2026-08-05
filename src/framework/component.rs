use async_trait::async_trait;
use thiserror::Error;

use crate::{
    framework::error::FlightError,
    kernel::event::{
        Event,
        EventEnvelope,
        EventKind,
    },
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum ComponentId {
    #[error("power component")]
    Power = 1,

    #[error("adcs component")]
    Adcs = 2,
    
    #[error("gnc component")]
    Gnc = 3,

    #[error("mission component")]
    Mission = 4,

    #[error("telemetry component")]
    Telemetry = 5,
}


#[async_trait]
pub trait Component: Send {
    fn id(&self) -> ComponentId;

    fn subscriptions(&self) -> &'static [EventKind];
    
    async fn update(
        &mut self,
    ) -> Result<Vec<Event>, FlightError>;

    async fn on_event(
        &mut self,
        envelope: &EventEnvelope,
    ) -> Result<Vec<Event>, FlightError>;
}