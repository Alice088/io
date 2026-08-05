use crate::{
    component::component::Percent, framework::{component, error::FlightError}, hal::battery::BatteryHal, kernel::event::{Event, EventEnvelope, EventKind},
};
use async_trait::async_trait;

pub struct Power {
    hal: BatteryHal,
    subscriptions: &'static [EventKind],
    pub value: Percent,
}

impl Power {
    pub fn new(hal: BatteryHal, subscriptions: &'static [EventKind]) -> Power {
        Power {
            hal,
            subscriptions,
            value: 0.0,
        }
    }
}

#[async_trait]
impl component::Component  for Power {
    fn id(&self) -> component::ComponentId {
        component::ComponentId::Power
    }

    fn subscriptions(&self) -> &'static [EventKind] {
        self.subscriptions
    }

    // MADE BY GENIUS KERBIN-GOSHA; ЛИЧНО Я БЛЯТЬ СИДЕЛ ОКОЛО РАКЕТЫ, Я ПРОДАЛ ВСЮ СЕМЬЮ РАДИ НАУТБУКА ЧТОБЫ НАПИСАТЬ ЭТОТ КОД, Я БЛЯТЬ СИДЕЛ СУКА
    async fn update(&mut self) -> Result<Vec<Event>, FlightError> {
        let mut v: Vec<Event> = Vec::new();

        let battery = self
            .hal
            .get()
            .await
            .map_err(|e| {
                match e {
                    crate::framework::error::FlightError::HardwareFailure(reason) => {
                        v.push(Event::ComponentFault {
                            component: self.id(),
                            reason: reason,
                        });
                    }

                    _ => (),
                };
            })
            .unwrap();

        let p: Percent = (battery.amount / battery.max_amount) * 100.0;
        if p <= 30.0 {
            v.push(Event::BatteryLow { percent: p });
        };
        if p <= 10.0 {
            v.push(Event::BatteryCritical { percent: p });
        };

        Ok(v)
    }

    async fn on_event(&mut self, _envelope: &EventEnvelope) -> Result<Vec<Event>, FlightError> {
        Ok(Vec::new())
    }
}
