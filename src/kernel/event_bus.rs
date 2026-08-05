use crate::{
    framework::component,
    kernel::{
        clock::Ms,
        event::{Event, EventEnvelope, EventKind},
    },
};

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventBusError {
    #[error("duplicate subscriber {0}")]
    DuplicateSubscriber(component::Id),

    #[error("unknown subscriber {0}")]
    UnknownSubscriber(component::Id),

    #[error("pending queue is full")]
    PendingQueueFull,

    #[error("inbox for component {0} is full")]
    InboxFull(component::Id),

    #[error("event sequence overflow")]
    SequenceOverflow,
}

struct Subscriber {
    subscriptions: BTreeSet<EventKind>,
    inbox: VecDeque<EventEnvelope>,
}

impl Subscriber {
    fn new(subscriptions: &[EventKind]) -> Self {
        Self {
            subscriptions: subscriptions.iter().copied().collect(),
            inbox: VecDeque::new(),
        }
    }

    fn accepts(&self, kind: EventKind) -> bool {
        self.subscriptions.contains(&kind)
    }
}

pub struct EventBus {
    subscribers: BTreeMap<component::Id, Subscriber>,
    pending: VecDeque<EventEnvelope>,
    next_sequence: u16,
    max_pending_events: usize,
    max_inbox_events: usize,
}

impl EventBus {
    pub fn new(max_pending_events: usize, max_inbox_events: usize) -> Self {
        assert!(
            max_pending_events > 0,
            "max_pending_events must be greater than zero",
        );

        assert!(
            max_inbox_events > 0,
            "max_inbox_events must be greater than zero",
        );

        Self {
            subscribers: BTreeMap::new(),
            pending: VecDeque::new(),
            next_sequence: 0,
            max_pending_events,
            max_inbox_events,
        }
    }

    pub fn register_subscriber(
        &mut self,
        component: component::Id,
        subscriptions: &[EventKind],
    ) -> Result<(), EventBusError> {
        if self.subscribers.contains_key(&component) {
            return Err(EventBusError::DuplicateSubscriber(component));
        }

        self.subscribers
            .insert(component, Subscriber::new(subscriptions));

        Ok(())
    }

    pub fn publish(
        &mut self,
        source: component::Id,
        timestamp: Ms,
        event: Event,
    ) -> Result<u16, EventBusError> {
        if self.pending.len() >= self.max_pending_events {
            return Err(EventBusError::PendingQueueFull);
        }

        let next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(EventBusError::SequenceOverflow)?;

        let seq = self.next_sequence;

        self.pending.push_back(EventEnvelope {
            seq,
            source,
            timestamp_ms: timestamp,
            event,
        });

        self.next_sequence = next_sequence;

        Ok(seq)
    }

    pub fn dispatch_pending(&mut self) -> Result<(), EventBusError> {
        while let Some(envelope) = self.pending.front().cloned() {
            let event_kind = envelope.event.kind();

            let targets: Vec<component::Id> = self
                .subscribers
                .iter()
                .filter_map(|(component, subscriber)| {
                    subscriber.accepts(event_kind).then_some(*component)
                })
                .collect();

            for component in &targets {
                let subscriber = self
                    .subscribers
                    .get(component)
                    .expect("target subscriber must exist");

                if subscriber.inbox.len() >= self.max_inbox_events {
                    return Err(EventBusError::InboxFull(*component));
                }
            }

            self.pending.pop_front();

            for component in targets {
                let subscriber = self
                    .subscribers
                    .get_mut(&component)
                    .expect("target subscriber must exist");

                subscriber.inbox.push_back(envelope.clone());
            }
        }

        Ok(())
    }

    pub fn next_for(
        &mut self,
        component: component::Id,
    ) -> Result<Option<EventEnvelope>, EventBusError> {
        let subscriber = self
            .subscribers
            .get_mut(&component)
            .ok_or(EventBusError::UnknownSubscriber(component))?;

        Ok(subscriber.inbox.pop_front())
    }

    pub fn pending_for(
        &self,
        component: component::Id,
    ) -> Result<usize, EventBusError> {
        let subscriber = self
            .subscribers
            .get(&component)
            .ok_or(EventBusError::UnknownSubscriber(component))?;

        Ok(subscriber.inbox.len())
    }

    pub fn pending_dispatch_count(&self) -> usize {
        self.pending.len()
    }
}