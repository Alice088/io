use crate::{
    framework::component::Component,
    hal::world::{GyroHandle, World},
};

pub struct Gyro {
    hal: GyroHandle,

    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
    pub fault: bool,
}

impl Gyro {
    pub fn new(world: &World) -> Self {
        Self {
            hal: world.gyro_hal(),
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
            fault: false,
        }
    }
}

impl Component for Gyro {
    fn name(&self) -> &'static str {
        "gyro"
    }

    fn update(&mut self) {
        match self.hal.get() {
            Ok(g) => {
                self.pitch = g.pitch;
                self.roll = g.roll;
                self.yaw = g.yaw;
                self.fault = false;
            }

            Err(e) => {
                self.fault = true;
                println!("GYRO FAULT: {e}");
            }
        }

        println!("GYRO: pitch={} roll={} yaw={}", self.pitch, self.roll, self.yaw);
    }
}
