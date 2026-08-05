use crate::{
    component::component::Percent, framework::component::{Component, ComponentId}, hal::battery::BatteryHal, kernel::event::{ Event, EventEnvelope, EventKind},
};

pub struct Power {
    hal: BatteryHal,
    subscriptions: &'static [EventKind],
    pub value: Percent,
}

impl Power {
    fn new(hal: BatteryHal, subscriptions: &'static [EventKind]) -> Power {
        Power {
            hal,
            subscriptions,
            value: 0.0,
        }
    }
}

impl Component for Power {
    fn id(&self) -> ComponentId {
        ComponentId::Power
    }

    fn subscriptions(&self) -> &'static [EventKind] {
        self.subscriptions
    }

    // MADE BY GENIUS KERBIN-GOSHA; ЛИЧНО Я БЛЯТЬ СИДЕЛ ОКОЛО РАКЕТЫ, Я ПРОДАЛ ВСЮ СЕМЬЮ РАДИ НАУТБУКА ЧТОБЫ НАПИСАТЬ ЭТОТ КОД, Я БЛЯТЬ СИДЕЛ СУКА ДЕЛАЛ
    async fn update(&mut self) -> Vec<Event> {
        let mut v: Vec<Event> = Vec::new();

        let battaries = self.hal.get().await.map_err(|e| {
            match e {
                crate::framework::error::FlightError::HardwareFailure(reason) => {
                    v.push(Event::ComponentFault { component: self.id(), reason: reason });
                }

                _ => ()
            };
        });

        let sum_amount: f32 = battaries.iter().map(|b| b.amount).sum();
        let sum_max: f32 = battaries.iter().map(|b| b.max_amount).sum();

        let p: Percent = (sum_amount / sum_max) * 100.0;
        if p <= 30.0 {
            v.push(Event::BatteryLow { percent: p });
        };
        if p <= 10.0 {
            v.push(Event::BatteryCritical { percent: p });
        };

        v
    }

    async fn on_event(&mut self, _envelope: &EventEnvelope) -> Vec<Event> {
        Vec::new()
    }
}
