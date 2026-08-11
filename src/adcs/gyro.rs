use std::sync::{Arc, Mutex};

use crate::{
    fsw::{component::Component, event::Event},
    ksp::world::World,
};

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub type NormalizedQuaternion = Quaternion;

#[derive(Debug, Clone, Copy)]
pub struct AngularVelocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct RotationError {
    pub axis_x: f64,
    pub axis_y: f64,
    pub axis_z: f64,
    pub angle: f64,
}

impl AngularVelocity {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from(current: (f64, f64, f64)) -> Self {
        Self {
            x: current.0,
            y: current.1,
            z: current.2,
        }
    }
}

impl Quaternion {
    pub fn new() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from(current: (f64, f64, f64, f64)) -> Self {
        Self {
            w: current.0,
            x: current.1,
            y: current.2,
            z: current.3,
        }
    }

    pub fn normalize(target: &Quaternion) -> NormalizedQuaternion {
        let q_len = (target.w.powf(2.0)
            + target.x.powf(2.0)
            + target.y.powf(2.0)
            + target.z.powf(2.0))
            .sqrt();

        if q_len < 1e-12 {
            return Quaternion::new();
        }

        NormalizedQuaternion {
            w: target.w / q_len,
            x: target.x / q_len,
            y: target.y / q_len,
            z: target.z / q_len,
        }
    }

    pub fn mul(&self, rhs: &NormalizedQuaternion) -> NormalizedQuaternion {
        NormalizedQuaternion {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,

            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,

            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,

            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }

    pub fn inverse(&self) -> NormalizedQuaternion {
        NormalizedQuaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub fn to_axis_angle(&self) -> RotationError {
        let w = self.w.clamp(-1.0, 1.0);

        let angle = 2.0 * w.acos();

        let sin_half_angle = (1.0 - w * w).sqrt();

        if sin_half_angle < 1e-8 {
            return RotationError {
                axis_x: 0.0,
                axis_y: 0.0,
                axis_z: 0.0,
                angle: 0.0,
            };
        }

        RotationError {
            axis_x: self.x / sin_half_angle,
            axis_y: self.y / sin_half_angle,
            axis_z: self.z / sin_half_angle,
            angle,
        }
    }
}

/// Latest gyro telemetry, published for readers (e.g. stab) via shared handle.
#[derive(Debug, Clone, Copy)]
pub struct GyroState {
    pub angular_velocity: AngularVelocity,
    pub current: NormalizedQuaternion,
}

pub struct Gyro {
    world: Arc<World>,

    pub angular_velocity: AngularVelocity,
    pub current: NormalizedQuaternion,

    shared: Arc<Mutex<GyroState>>,
}

impl Gyro {
    /// Creates the gyro and returns it together with the shared snapshot handle.
    pub fn new(world: Arc<World>) -> (Self, Arc<Mutex<GyroState>>) {
        let shared = Arc::new(Mutex::new(GyroState {
            angular_velocity: AngularVelocity::new(),
            current: Quaternion::new(),
        }));
        (
            Self {
                world,
                angular_velocity: AngularVelocity::new(),
                current: Quaternion::new(),
                shared: Arc::clone(&shared),
            },
            shared,
        )
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

                *self.shared.lock().unwrap() = GyroState {
                    angular_velocity: self.angular_velocity,
                    current: self.current,
                };
            }

            Err(e) => {
                println!("GYRO FAULT: {e}");
            }
        }
    }

    fn on_event(&mut self, _event: Event) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    fn assert_quaternion_eq(actual: Quaternion, expected: Quaternion) {
        assert!((actual.w - expected.w).abs() < EPS);
        assert!((actual.x - expected.x).abs() < EPS);
        assert!((actual.y - expected.y).abs() < EPS);
        assert!((actual.z - expected.z).abs() < EPS);
    }

    #[test]
    fn from_tuple_keeps_standard_wxyz_order() {
        let q = Quaternion::from((1.0, 2.0, 3.0, 4.0));

        assert!((q.w - 1.0).abs() < EPS);
        assert!((q.x - 2.0).abs() < EPS);
        assert!((q.y - 3.0).abs() < EPS);
        assert!((q.z - 4.0).abs() < EPS);
    }

    #[test]
    fn angular_velocity_from_tuple_keeps_xyz_order() {
        let v = AngularVelocity::from((1.0, 2.0, 3.0));

        assert!((v.x - 1.0).abs() < EPS);
        assert!((v.y - 2.0).abs() < EPS);
        assert!((v.z - 3.0).abs() < EPS);
    }

    #[test]
    fn quaternion_normalize() {
        let q = Quaternion {
            w: 2.0,
            x: 3.0,
            y: 4.0,
            z: 5.0,
        };

        let normalized = Quaternion::normalize(&q);

        let length = (normalized.w * normalized.w
            + normalized.x * normalized.x
            + normalized.y * normalized.y
            + normalized.z * normalized.z)
            .sqrt();

        assert!(
            (length - 1.0).abs() < 1e-10,
            "Quaternion is not normalized: length = {}",
            length
        );
    }

    #[test]
    fn mul_returns_expected_quaternion() {
        let lhs = Quaternion {
            w: 1.0,
            x: 2.0,
            y: 3.0,
            z: 4.0,
        };

        let rhs = Quaternion {
            w: 5.0,
            x: 6.0,
            y: 7.0,
            z: 8.0,
        };

        let result = lhs.mul(&rhs);

        assert_quaternion_eq(
            result,
            Quaternion {
                w: -60.0,
                x: 12.0,
                y: 30.0,
                z: 24.0,
            },
        );
    }

    #[test]
    fn mul_by_identity_returns_same_quaternion() {
        let q = Quaternion {
            w: 0.5,
            x: 0.5,
            y: 0.5,
            z: 0.5,
        };

        let identity = Quaternion {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        assert_quaternion_eq(q.mul(&identity), q);
    }

    #[test]
    fn inverse_returns_conjugate() {
        let q = Quaternion {
            w: 0.5,
            x: 0.1,
            y: -0.2,
            z: 0.3,
        };

        let result = q.inverse();

        assert_quaternion_eq(
            result,
            Quaternion {
                w: 0.5,
                x: -0.1,
                y: 0.2,
                z: -0.3,
            },
        );
    }

    #[test]
    fn normalized_quaternion_mul_inverse_returns_identity() {
        let current = Quaternion {
            w: 0.5,
            x: 0.5,
            y: 0.5,
            z: 0.5,
        };

        let target = Quaternion {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let current = Quaternion::normalize(&current);
        let target = Quaternion::normalize(&target);

        let error = target.mul(&current.inverse());

        assert_quaternion_eq(
            error,
            Quaternion {
                w: 0.5,
                x: -0.5,
                y: -0.5,
                z: -0.5,
            },
        );
    }
}
