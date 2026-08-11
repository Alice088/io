use crate::fsw::event::Event;

pub trait Component: Send {
    fn name(&self) -> &'static str;
    fn update(&mut self);

    /// Event types this component subscribes to. Default: none.
    fn event_subscriptions(&self) -> &'static [Event] {
        &[]
    }

    /// Called by the scheduler for every subscribed event. Default: no-op.
    fn on_event(&mut self, _event: Event) {}
}
