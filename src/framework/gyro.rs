use std::sync::Arc;

use crate::{framework::component::Component, kernel::event::Event, ksp::world::World};

pub type Quaternion = (f64, f64, f64, f64);

pub struct Gyro {
    world: Arc<World>,

    pub angular_velocity: (f64, f64, f64),
    pub rotation: Quaternion,
    pub fault: bool,
}

impl Gyro {
    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            angular_velocity: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 0.0),
            fault: false,
        }
    }
}

impl Component for Gyro {
    fn name(&self) -> &'static str {
        "gyro"
    }

    fn update(&mut self) {
        match self.world.gyro() {
            Ok(g) => {
                self.angular_velocity = g.angular_velocity;
                self.rotation = g.rotation;
                self.fault = false;
            }

            Err(e) => {
                self.fault = true;
                println!("GYRO FAULT: {e}");
            }
        }
    }

    fn on_event(&mut self, _event: Event) {}
}
