use std::sync::Arc;

use crate::{
    framework::component::Component,
    ksp::world::World,
};

pub struct Gyro {
    world: Arc<World>,

    pub wx: f64,
    pub xy: f64,
    pub xz: f64,
    pub fault: bool,
}

impl Gyro {
    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            wx: 0.0,
            xy: 0.0,
            xz: 0.0,
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
                self.wx = g.wx;
                self.xy = g.wy;
                self.xz = g.wz;
                self.fault = false;
            }

            Err(e) => {
                self.fault = true;
                println!("GYRO FAULT: {e}");
            }
        }

        println!("GYRO: wx={} wy={} wz={}", self.wx, self.xy, self.xz);
    }
}