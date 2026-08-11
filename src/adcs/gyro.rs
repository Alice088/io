use std::sync::Arc;

use crate::{
    fsw::{component::Component, event::Event},
    ksp::world::World,
};

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub w: f64,
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

pub type NormalizedQuaternion = Quaternion;

pub struct AngularVelocity {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

impl AngularVelocity {
    pub fn new() -> Self {
        Self {
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
        }
    }

    pub fn from(current: (f64, f64, f64)) -> Self {
        Self {
            pitch: current.0,
            roll: current.1,
            yaw: current.2,
        }
    }
}

impl Quaternion {
    pub fn new() -> Self {
        Self {
            w: 0.0,
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
        }
    }

    pub fn from(current: (f64, f64, f64, f64)) -> Self {
        Self {
            w: current.0,
            pitch: current.1,
            roll: current.2,
            yaw: current.3,
        }
    }

    pub fn normalize(target: &Quaternion) -> NormalizedQuaternion {
        let q_len = (target.w.powf(2.0)
            + target.pitch.powf(2.0)
            + target.roll.powf(2.0)
            + target.yaw.powf(2.0))
        .sqrt();

        NormalizedQuaternion {
            w: target.w / q_len,
            pitch: target.pitch / q_len,
            roll: target.roll / q_len,
            yaw: target.yaw / q_len,
        }
    }

    pub fn sub(&self, rhs: &NormalizedQuaternion) -> NormalizedQuaternion {
        println!("rhz: {:?}", rhs);
        NormalizedQuaternion {
            w: self.w - rhs.w,
            roll: self.roll - rhs.roll,
            pitch: self.pitch - rhs.pitch,
            yaw: self.yaw - rhs.yaw,
        }
    }
}

pub struct Gyro {
    world: Arc<World>,

    pub angular_velocity: AngularVelocity,
    pub current: NormalizedQuaternion,
}

impl Gyro {
    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            angular_velocity: AngularVelocity::new(),
            current: Quaternion::new(),
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
                self.current = Quaternion::normalize(&g.rotation);
                println!("gyro c: {:?}", self.current);
            }

            Err(e) => {
                println!("GYRO FAULT: {e}");
            }
        }
    }

    fn on_event(&mut self, _event: Event) {}
}
