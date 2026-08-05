use crate::{
    framework::component,
    kernel::{
        clock::Ms,
        event::{Event, EventEnvelope, EventKind},
    },
};

pub type Id = u8;

pub trait Component {
    fn id(&self) -> component::Id;
    fn subscriptions(&self) -> &'static [EventKind];
    fn update(&mut self) -> Vec<Event>;
    fn on_event(&mut self, envelope: &EventEnvelope) -> Vec<Event>;
}