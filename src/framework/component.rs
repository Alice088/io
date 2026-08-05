use crate::{
    kernel::{
        event::{Event, EventEnvelope, EventKind},
    },
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentId {
    Power = 1,
    Adcs = 2,
    Gnc = 3,
    Mission = 4,
    Telemetry = 5,
}

pub type Id = u8;

pub trait Component {
    fn id(&self) -> ComponentId;
    fn subscriptions(&self) -> &'static [EventKind];
    async fn update(&mut self) -> Vec<Event>;
    fn on_event(&mut self, envelope: &EventEnvelope) -> Vec<Event>;
}