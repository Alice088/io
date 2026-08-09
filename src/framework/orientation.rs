use std::sync::Arc;

use crate::{
    framework::component::Component,
    fdir::error::Error,
    kernel::event::Event,
    ksp::world::World,
};

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
    pub fn set(&mut self, roll: f64, pitch: f64, yaw: f64) -> Result<(), Error> {
        match self.world.set_orientation(roll, pitch, yaw) {
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
