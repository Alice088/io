use std::sync::Arc;

use crate::{
    fdir::error::Error,
    fsw::{component::Component, event::Event},
    ksp::world::World,
};

#[derive(Debug)]
pub struct Target {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64
}

impl Target {
    pub fn new() -> Self {
        Self { roll: 0.0, pitch: 0.0, yaw: 0.0 }
    }
}

/// Write-only orientation device: sync facade over the kRPC worker thread.
/// Async (kRPC) is hidden inside `ksp::world`; component level is fully sync.
pub struct Orientation {
    world: Arc<World>,

    pub fault: bool,
}

impl Orientation {
    pub fn new(world: Arc<World>) -> Self {
        Self { world, fault: false }
    }

    /// Applies control inputs, blocking until applied. Sets `fault` on error.
    pub fn set(&mut self, target: &Target) -> Result<(), Error> {
        match self.world.set_orientation(target.roll, target.pitch, target.yaw) {
            Ok(()) => {
                self.fault = false;
                Ok(())
            }
            Err(e) => {
                self.fault = true;
                println!("ORIENTATION FAULT: {e}");
                Err(e)
            }
        }
    }
}

impl Component for Orientation {
    fn name(&self) -> &'static str {
        "orientation"
    }

    fn update(&mut self) {}

    fn on_event(&mut self, _event: Event) {}
}
