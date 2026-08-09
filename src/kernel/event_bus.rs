use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::kernel::event::Event;

/// Thread-safe event bus.
///
/// Publish from anywhere (any thread), drain from the single consumer
/// (e.g. the scheduler loop). Clones share the same underlying queue,
/// so the bus can be handed out freely.
#[derive(Clone)]
pub struct EventBus {
    queue: Arc<Mutex<VecDeque<Event>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Lock with poison recovery: a poisoned mutex still holds valid data.
    fn lock(&self) -> MutexGuard<'_, VecDeque<Event>> {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Push one event. Never fails.
    pub fn publish(&self, event: Event) {
        self.lock().push_back(event);
    }

    /// Take all pending events in FIFO order. Empty vec if none.
    pub fn drain(&self) -> Vec<Event> {
        self.lock().drain(..).collect()
    }

    /// Number of pending events.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bus_drains_empty() {
        let bus = EventBus::new();
        assert!(bus.is_empty());
        assert_eq!(bus.len(), 0);
        assert_eq!(bus.drain(), Vec::<Event>::new());
    }

    #[test]
    fn publish_then_drain_preserves_fifo_order() {
        let bus = EventBus::new();
        bus.publish(Event::BatteryLow);
        bus.publish(Event::BatteryCritical);
        bus.publish(Event::WatchdogTimeout);

        assert_eq!(
            bus.drain(),
            vec![
                Event::BatteryLow,
                Event::BatteryCritical,
                Event::WatchdogTimeout
            ]
        );
    }

    #[test]
    fn drain_clears_bus() {
        let bus = EventBus::new();
        bus.publish(Event::Boot);
        assert_eq!(bus.drain().len(), 1);
        assert_eq!(bus.drain().len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    fn publish_after_drain_still_works() {
        let bus = EventBus::new();
        bus.publish(Event::Boot);
        bus.drain();
        bus.publish(Event::Deploy);
        assert_eq!(bus.drain(), vec![Event::Deploy]);
    }

    #[test]
    fn len_tracks_pending_events() {
        let bus = EventBus::new();
        assert_eq!(bus.len(), 0);
        bus.publish(Event::GyroFault);
        bus.publish(Event::GyroRestored);
        assert_eq!(bus.len(), 2);
        bus.drain();
        assert_eq!(bus.len(), 0);
    }

    #[test]
    fn clone_shares_same_queue() {
        let bus_a = EventBus::new();
        let bus_b = bus_a.clone();

        bus_a.publish(Event::LinkLost);
        assert_eq!(bus_b.drain(), vec![Event::LinkLost]);

        bus_b.publish(Event::LinkRestored);
        assert_eq!(bus_a.drain(), vec![Event::LinkRestored]);
    }

    #[test]
    fn concurrent_publish_is_safe_and_lossless() {
        let bus = EventBus::new();
        let n_threads = 8;
        let per_thread = 500;

        let handles: Vec<_> = (0..n_threads)
            .map(|t| {
                let bus = bus.clone();
                std::thread::spawn(move || {
                    for i in 0..per_thread {
                        let e = if (t + i) % 2 == 0 {
                            Event::BatteryLow
                        } else {
                            Event::BatteryCritical
                        };
                        bus.publish(e);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(bus.len(), n_threads * per_thread);
        assert_eq!(bus.drain().len(), n_threads * per_thread);
        assert!(bus.is_empty());
    }

    #[test]
    fn event_is_copy_and_eq() {
        let a = Event::Boot;
        let b = a; // Copy: a still usable
        assert_eq!(a, b);
    }
}
