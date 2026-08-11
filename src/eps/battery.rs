use std::sync::Arc;

use crate::{
    fsw::{component::Component, event::Event},
    ksp::world::World,
};

const LOW_PERCENT: f32 = 30.0;
const CRITICAL_PERCENT: f32 = 10.0;

pub struct Battery {
    world: Arc<World>,

    pub percent: f32,
    pub low: bool,
    pub critical: bool,
    pub fault: bool,
}

impl Battery {
    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            percent: 0.0,
            low: false,
            critical: false,
            fault: false,
        }
    }
}

/// amount/max_amount -> percent in [0; 100]. 0 if max_amount <= 0.
pub fn compute_percent(amount: f32, max_amount: f32) -> f32 {
    if max_amount <= 0.0 {
        return 0.0;
    }

    ((amount / max_amount) * 100.0).clamp(0.0, 100.0)
}

impl Component for Battery {
    fn name(&self) -> &'static str {
        "battery"
    }

    fn update(&mut self) {
        match self.world.battery() {
            Ok(b) => {
                self.percent = compute_percent(b.amount, b.max_amount);
                self.low = self.percent <= LOW_PERCENT;
                self.critical = self.percent <= CRITICAL_PERCENT;
                self.fault = false;
            }

            Err(e) => {
                self.fault = true;
                println!("BATTERY FAULT: {e}");
            }
        }
    }

    fn on_event(&mut self, _event: Event) {}
}

#[cfg(test)]
mod tests {
    use super::compute_percent;

    #[test]
    fn normal_half_charge() {
        assert_eq!(compute_percent(50.0, 100.0), 50.0);
    }

    #[test]
    fn full_charge() {
        assert_eq!(compute_percent(100.0, 100.0), 100.0);
    }

    #[test]
    fn empty_charge() {
        assert_eq!(compute_percent(0.0, 100.0), 0.0);
    }

    #[test]
    fn zero_max_avoids_div_by_zero() {
        assert_eq!(compute_percent(10.0, 0.0), 0.0);
    }

    #[test]
    fn amount_over_max_clamped_to_100() {
        assert_eq!(compute_percent(120.0, 100.0), 100.0);
    }

    #[test]
    fn negative_amount_clamped_to_0() {
        assert_eq!(compute_percent(-5.0, 100.0), 0.0);
    }
}
