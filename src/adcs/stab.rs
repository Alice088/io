use std::sync::{Arc, Mutex};

use crate::{
    adcs::{
        gyro::{GyroState, NormalizedQuaternion, Quaternion},
        orientation::{Orientation, Target},
    },
    fsw::{component::Component, event::Event, event_bus::EventBus},
};

pub struct Stab {
    on: bool,
    bus: Arc<EventBus>,
    gyro: Arc<Mutex<GyroState>>,
    orientation: Box<Orientation>,

    mode: StabMode,
    target: Quaternion,
    kd: f64,
    kp: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabMode {
    Off,
    Damp,
    Hold,
}

impl Stab {
    pub fn new(
        gyro: Arc<Mutex<GyroState>>,
        bus: Arc<EventBus>,
        orientation: Box<Orientation>,
    ) -> Self {
        Self {
            bus,
            gyro,
            orientation,
            target: Quaternion::new(),
            kd: 0.05,
            kp: 0.1,
            mode: StabMode::Off,
            on: false,
        }
    }

    // команда = P × ошибка − D × скорость вращения
    fn pd_control(&self, error_x: f64, error_y: f64, error_z: f64) -> Target {
        let g = self.gyro.lock().unwrap();
        let velocity = &g.angular_velocity;

        let mut target = Target::new();

        let roll = match self.mode {
            StabMode::Off => 0.0,
            StabMode::Damp => -self.kd * velocity.x,
            StabMode::Hold => self.kp * error_x - self.kd * velocity.x,
        };

        let pitch = match self.mode {
            StabMode::Off => 0.0,
            StabMode::Damp => -self.kd * velocity.y,
            StabMode::Hold => self.kp * error_y - self.kd * velocity.y,
        };

        let yaw = match self.mode {
            StabMode::Off => 0.0,
            StabMode::Damp => -self.kd * velocity.z,
            StabMode::Hold => self.kp * error_z - self.kd * velocity.z,
        };

        target.roll = roll;
        target.pitch = pitch;
        target.yaw = yaw;

        target
    }
}

impl Component for Stab {
    fn name(&self) -> &'static str {
        "stab"
    }

    fn update(&mut self) {
        if !self.on || self.mode == StabMode::Off {
            return;
        }

        let state = *self.gyro.lock().unwrap();
        let qerror = Quaternion::normalize(&self.target.mul(&state.current.inverse()));
        let rotation_error = qerror.to_axis_angle();

        if rotation_error.angle < 1e-6 {
            return;
        }

        let error_x = rotation_error.axis_x * rotation_error.angle;
        let error_y = rotation_error.axis_y * rotation_error.angle;
        let error_z = rotation_error.axis_z * rotation_error.angle;

        let target = self.pd_control(error_x, error_y, error_z);

        match self.orientation.set(&target) {
            Err(e) => {
                println!("error: {}", e)
            }
            _ => (),
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
                self.target = Quaternion::normalize(&self.gyro.lock().unwrap().current);
                self.mode = StabMode::Hold;
                self.on = true;
            }
            Event::DampingEnabled => {
                self.target = Quaternion::normalize(&self.gyro.lock().unwrap().current);
                self.mode = StabMode::Damp;
                self.on = true;
            }
            Event::StabilizationDisabled | Event::DampingDisabled => {
                self.mode = StabMode::Off;
                self.on = false;
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adcs::gyro::AngularVelocity;

    use super::*;

    fn pd_control_axis(kp: f64, kd: f64, error: f64, velocity: f64) -> f64 {
        kp * error - kd * velocity
    }

    #[test]
    fn pd_control_returns_expected_values() {
        let kp = 2.0;
        let kd = 0.5;

        let qerror = Quaternion {
            w: 1.0,
            x: 0.4,
            y: -0.3,
            z: 0.2,
        };

        let velocity = AngularVelocity {
            x: 0.1,
            y: -0.2,
            z: 0.3,
        };

        let roll = pd_control_axis(kp, kd, qerror.x, velocity.x);
        let pitch = pd_control_axis(kp, kd, qerror.y, velocity.y);
        let yaw = pd_control_axis(kp, kd, qerror.z, velocity.z);

        assert!((roll - 0.75).abs() < 1e-10);
        assert!((pitch - (-0.5)).abs() < 1e-10);
        assert!((yaw - 0.25).abs() < 1e-10);
    }
}
