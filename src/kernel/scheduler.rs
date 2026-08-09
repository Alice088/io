use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    fdir::{error::Error, reason::Reason},
    framework::component::Component,
    kernel::{
        clock::{MissionClock, Ms},
        event::Event,
        event_bus::EventBus,
        watchdog::Watchdog,
    },
};

pub struct Task {
    name: &'static str,
    period: Ms,
    next: Ms,
    c: Box<dyn Component>,
}

impl Task {
    pub fn new(c: Box<dyn Component>, period: Ms) -> Self {
        Self {
            name: c.name(),
            period,
            next: 0,
            c,
        }
    }
}

pub struct Scheduler {
    tasks: Vec<Task>,
    tasks_limit: u8,
    tick: Ms,
    clock: Box<MissionClock>,
    bus: Arc<EventBus>,
}

impl Scheduler {
    pub fn new(tasks_limit: u8, tick: Ms, clock: Box<MissionClock>) -> Scheduler {
        Scheduler {
            tasks_limit,
            tasks: Vec::new(),
            tick,
            clock,
            bus: Arc::new(EventBus::new()),
        }
    }

    /// Clone of the shared event bus. Hand out to publishers
    /// (components, telemetry, ...) so they can publish events.
    pub fn bus(&self) -> Arc<EventBus> {
        Arc::clone(&self.bus)
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), Error> {
        if self.tasks.len() >= self.tasks_limit as usize {
            return Err(Error::Software(Reason::MaxScdulerTasks));
        }

        self.tasks.push(task);
        Ok(())
    }

    pub fn run(&mut self, watchdog: Arc<Mutex<Watchdog>>) {
        loop {
            thread::sleep(Duration::from_millis(self.tick));

            watchdog.lock().unwrap().kick();

            let now = self.clock.ms();

            for task in self.tasks.iter_mut() {
                if now >= task.next {
                    println!("({}s){}: RUN {}", self.clock.sec(), now, task.name);
                    task.c.update();
                    task.next = now + task.period;
                }
            }

            self.dispatch_pending();
        }
    }

    /// Drain the bus and deliver every event to subscribed components,
    /// in FIFO order, calling `on_event` on each match.
    pub fn dispatch_pending(&mut self) {
        let events = self.bus.drain();
        self.dispatch(events);
    }

    fn dispatch(&mut self, events: Vec<Event>) {
        for event in events {
            for task in self.tasks.iter_mut() {
                if task.c.event_subscriptions().contains(&event) {
                    task.c.on_event(event);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Recording {
        name: &'static str,
        subs: &'static [Event],
        received: Arc<Mutex<Vec<Event>>>,
    }

    impl Recording {
        fn new(name: &'static str, subs: &'static [Event]) -> (Self, Arc<Mutex<Vec<Event>>>) {
            let received = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    name,
                    subs,
                    received: Arc::clone(&received),
                },
                received,
            )
        }
    }

    impl Component for Recording {
        fn name(&self) -> &'static str {
            self.name
        }

        fn update(&mut self) {}

        fn event_subscriptions(&self) -> &'static [Event] {
            self.subs
        }

        fn on_event(&mut self, event: Event) {
            self.received.lock().unwrap().push(event);
        }
    }

    fn events(received: &Arc<Mutex<Vec<Event>>>) -> Vec<Event> {
        received.lock().unwrap().clone()
    }

    #[test]
    fn no_subscriptions_means_no_callbacks() {
        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));
        let (comp, rec) = Recording::new("quiet", &[]);
        sched.add_task(Task::new(Box::new(comp), 100)).unwrap();

        sched.dispatch(vec![Event::BatteryCritical, Event::Boot]);

        assert!(events(&rec).is_empty());
    }

    #[test]
    fn matching_event_delivered() {
        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));
        let (comp, rec) = Recording::new("listener", &[Event::BatteryCritical]);
        sched.add_task(Task::new(Box::new(comp), 100)).unwrap();

        sched.dispatch(vec![Event::BatteryCritical]);

        assert_eq!(events(&rec), vec![Event::BatteryCritical]);
    }

    #[test]
    fn unmatched_event_not_delivered() {
        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));
        let (comp, rec) = Recording::new("gyro-fan", &[Event::GyroFault]);
        sched.add_task(Task::new(Box::new(comp), 100)).unwrap();

        sched.dispatch(vec![Event::BatteryCritical, Event::Boot]);

        assert!(events(&rec).is_empty());
    }

    #[test]
    fn only_matching_subscribers_notified() {
        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));

        let (a, rec_a) = Recording::new("a", &[Event::BatteryCritical]);
        let (b, rec_b) = Recording::new("b", &[Event::WatchdogTimeout]);
        sched.add_task(Task::new(Box::new(a), 100)).unwrap();
        sched.add_task(Task::new(Box::new(b), 100)).unwrap();

        sched.dispatch(vec![Event::BatteryCritical]);

        assert_eq!(events(&rec_a), vec![Event::BatteryCritical]);
        assert!(events(&rec_b).is_empty());
    }

    #[test]
    fn multiple_events_delivered_in_fifo_order() {
        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));
        let (comp, rec) = Recording::new("fifo", &[Event::Boot, Event::Deploy]);
        sched.add_task(Task::new(Box::new(comp), 100)).unwrap();

        sched.dispatch(vec![Event::Boot, Event::Deploy]);

        assert_eq!(events(&rec), vec![Event::Boot, Event::Deploy]);
    }

    #[test]
    fn duplicate_events_all_delivered() {
        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));
        let (comp, rec) = Recording::new("dup", &[Event::BatteryLow]);
        sched.add_task(Task::new(Box::new(comp), 100)).unwrap();

        sched.dispatch(vec![Event::BatteryLow, Event::BatteryLow]);

        assert_eq!(events(&rec), vec![Event::BatteryLow, Event::BatteryLow]);
    }

    #[test]
    fn dispatch_pending_drains_bus_end_to_end() {
        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));
        let (comp, rec) = Recording::new("link", &[Event::LinkLost]);
        sched.add_task(Task::new(Box::new(comp), 100)).unwrap();

        sched.bus().publish(Event::LinkLost);
        sched.dispatch_pending();

        assert_eq!(events(&rec), vec![Event::LinkLost]);

        // second dispatch: bus empty, nothing new
        sched.dispatch_pending();
        assert_eq!(events(&rec), vec![Event::LinkLost]);
    }

    #[test]
    fn default_component_methods_are_safe() {
        struct Minimal {
            _name: &'static str,
        }

        impl Component for Minimal {
            fn name(&self) -> &'static str {
                "minimal"
            }

            fn update(&mut self) {}
        }

        let mut sched = Scheduler::new(8, 100, Box::new(MissionClock::start()));
        sched
            .add_task(Task::new(Box::new(Minimal { _name: "m" }), 100))
            .unwrap();

        // defaults: no subscriptions, no-op on_event -> no panic, no delivery
        sched.dispatch(vec![Event::GyroFault, Event::LinkLost]);
    }
}
