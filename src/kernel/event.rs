use crate::{framework::component, kernel::clock::Ms};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EventKind {
    BatteryLow,
    BatteryCritical,
    TargetReached,
    AttitudeLocked,
    BurnStarted,
    BurnCompleted,
    ImageCaptured,
    ComponentFault,
}

#[derive(Debug, Clone)]
pub enum Event {
    BatteryLow {
        percent: u8,
    },

    BatteryCritical {
        percent: u8,
    },

    TargetReached,

    AttitudeLocked,

    BurnStarted {
        duration: Ms,
    },

    BurnCompleted,

    ImageCaptured {
        image_id: u8,
    },

    ComponentFault {
        component: component::Id,
        reason: FaultReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultReason {
    Timeout,
    Overcurrent,
    SensorFailure,
    StorageFull,
    Unknown,
}

impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Event::BatteryLow { .. } => EventKind::BatteryLow,
            Event::BatteryCritical { .. } => EventKind::BatteryCritical,
            Event::TargetReached => EventKind::TargetReached,
            Event::AttitudeLocked => EventKind::AttitudeLocked,
            Event::BurnStarted { .. } => EventKind::BurnStarted,
            Event::BurnCompleted => EventKind::BurnCompleted,
            Event::ImageCaptured { .. } => EventKind::ImageCaptured,
            Event::ComponentFault { .. } => EventKind::ComponentFault,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub seq: u16,
    pub source: component::Id,
    pub timestamp_ms: Ms,
    pub event: Event,
}
