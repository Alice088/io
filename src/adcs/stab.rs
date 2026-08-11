use std::sync::Arc;

use crate::{
    adcs::{
        gyro::{Gyro, NormalizedQuaternion, Quaternion},
        orientation::{Orientation, Target},
    },
    fsw::{component::Component, event::Event, event_bus::EventBus},
    ksp::world::World,
};

pub struct Stab {
    on: bool,
    bus: Arc<EventBus>,
    gyro: Box<Gyro>,
    orientation: Box<Orientation>,

    mode: StabMode,
    target: Quaternion,
    Kd: f64,
    Kp: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabMode {
    Off,
    Damp,
    Hold,
}

impl Stab {
    pub fn new(bus: Arc<EventBus>, world: Arc<World>, orientation: Box<Orientation>) -> Self {
        Self {
            bus,
            gyro: Box::new(Gyro::new(Arc::clone(&world))),
            orientation,
            target: Quaternion::new(),
            Kd: 0.5,
            Kp: 0.1,
            mode: StabMode::Off,
            on: false
        }
    }

    // команда = P × ошибка − D × скорость вращения
    fn pd_control(&self, qerror: &NormalizedQuaternion) -> Target {
        let velocity = &self.gyro.angular_velocity;

        let mut target = Target::new();

        target.roll = self.Kp * qerror.roll - self.Kd * velocity.roll;
        target.pitch = self.Kp * qerror.pitch - self.Kd * velocity.pitch;
        target.yaw = self.Kp * qerror.yaw - self.Kd * velocity.yaw;
        target
    }
}

impl Component for Stab {
    fn name(&self) -> &'static str {
        "stab"
    }

    fn update(&mut self) {
        if !self.on {
            return;
        }

        let qerror = self.gyro.current.sub(&self.target);
        let target = self.pd_control(&qerror);

       match self.orientation.set(&target) {
           Err(e) => {
            println!("error: {}", e)
           },
           _ => ()
       }
    }

    fn event_subscriptions(&self) -> &'static [Event] {
        &[
            Event::StabilizationEnabled,
            Event::StabilizationDisabled,
            Event::DampingEnabled,
            Event::DampingDisabled,
        ]
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::StabilizationEnabled => {
                self.target = self.gyro.current.clone();
                println!("t: {:?}; c: {:?}", self.target, self.gyro.current.clone());
                self.mode = StabMode::Hold;
                self.on = true;
            }
            Event::DampingEnabled => {
                self.mode = StabMode::Damp;
            }
            Event::StabilizationDisabled | Event::DampingDisabled => {
                self.mode = StabMode::Off;
                self.on = true;
            }
            _ => (),
        }
    }
}
