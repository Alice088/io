use async_trait::async_trait;

use crate::{
    framework::component::Component,
    hal::battery::BatteryHal,
};

const LOW_PERCENT: f32 = 30.0;
const CRITICAL_PERCENT: f32 = 10.0;

pub struct Battery {
    hal: BatteryHal,

    pub percent: f32,
    pub low: bool,
    pub critical: bool,
    pub fault: bool,
}

impl Battery {
    pub fn new(hal: BatteryHal) -> Self {
        Self {
            hal,
            percent: 0.0,
            low: false,
            critical: false,
            fault: false,
        }
    }
}

/// amount/max_amount -> percent, clamped to [0; 100].
/// max_amount == 0 -> 0.0 (нет деления на ноль).
pub fn compute_percent(amount: f32, max_amount: f32) -> f32 {
    if max_amount <= 0.0 {
        return 0.0;
    }

    ((amount / max_amount) * 100.0).clamp(0.0, 100.0)
}

#[async_trait]
impl Component for Battery {
    fn name(&self) -> &'static str {
        "battery"
    }

    async fn update(&mut self) {
        match self.hal.get().await {
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
