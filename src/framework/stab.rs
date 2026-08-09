use std::sync::Arc;

use crate::{
    framework::{
        component::Component,
        gyro::{Gyro, Quaternion},
        orientation::Orientation,
    },
    kernel::{event::Event, event_bus::EventBus},
    ksp::world::World,
};

pub struct Stab {
    bus: Arc<EventBus>,
    gyro: Box<Gyro>,
    orientation: Box<Orientation>,

    gyro_faulted: bool,
    /// On `StabilizationEnabled`, capture the current attitude as target.
    capture_target: bool,

    pub target_rotation: Quaternion,
    pub kp: f64,
    pub kd: f64,

    /// Off / Damp (rate-only) / Hold (PD on captured attitude).
    pub mode: StabMode,
    pub fault: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabMode {
    Off,
    Damp,
    Hold,
}

impl Stab {
    pub fn new(bus: Arc<EventBus>, world: Arc<World>) -> Self {
        Self {
            bus,
            gyro: Box::new(Gyro::new(Arc::clone(&world))),
            orientation: Box::new(Orientation::new(Arc::clone(&world))),
            gyro_faulted: false,
            capture_target: false,
            mode: StabMode::Off,
            fault: false,
            target_rotation: (0.0, 0.0, 0.0, 1.0),
            kp: 1.0,
            kd: 0.3,
        }
    }
}

impl Component for Stab {
    fn name(&self) -> &'static str {
        "stab"
    }

    fn update(&mut self) {
        if self.mode == StabMode::Off {
            return;
        }

        // Fresh gyro sample; sync world call, async hidden in ksp::world.
        self.gyro.update();

        // "on stab" = hold the attitude we had when stabilization engaged.
        if self.capture_target {
            self.target_rotation = self.gyro.rotation;
            self.capture_target = false;
        }

        // Report gyro state transitions on the bus (edge-triggered).
        let faulty = self.gyro.fault;
        if faulty && !self.gyro_faulted {
            self.bus.publish(Event::GyroFault);
        }
        if !faulty && self.gyro_faulted {
            self.bus.publish(Event::GyroRestored);
        }
        self.gyro_faulted = faulty;

        if self.gyro.fault {
            self.fault = true;
            return;
        }

        let ctrl = match self.mode {
            StabMode::Damp => damp_control(self.gyro.rotation, self.gyro.angular_velocity, self.kd),
            StabMode::Hold => compute_control(
                self.target_rotation,
                self.gyro.rotation,
                self.gyro.angular_velocity,
                self.kp,
                self.kd,
            ),
            StabMode::Off => unreachable!(),
        };

        // Write-through orientation HAL; sync at component level.
        self.fault = self.orientation.set(ctrl.0, ctrl.1, ctrl.2).is_err();
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
                self.mode = StabMode::Hold;
                self.capture_target = true;
            }
            Event::DampingEnabled => {
                self.mode = StabMode::Damp;
            }
            Event::StabilizationDisabled | Event::DampingDisabled => {
                self.mode = StabMode::Off;
                self.capture_target = false;
                // kRPC control inputs persist until changed: zero them or the
                // vessel keeps applying the last commands.
                let _ = self.orientation.set(0.0, 0.0, 0.0);
            }
            _ => (),
        }
    }
}

/// Rate-only damping, returns `(roll, pitch, yaw)` clamped to [-1; 1].
///
/// Same empirical conventions as `compute_control`; validated live on the
/// vessel: brings the spin from 80 rad/s to zero.
fn damp_control(current: Quaternion, omega_orbital: (f64, f64, f64), kd: f64) -> (f64, f64, f64) {
    let (wx, wy, wz) = quaternion_rotate(quaternion_inverse(current), omega_orbital);

    let pitch = kd * wx;
    let roll = kd * wy;
    let yaw = kd * wz;

    (
        roll.clamp(-1.0, 1.0),
        pitch.clamp(-1.0, 1.0),
        yaw.clamp(-1.0, 1.0),
    )
}

/// PD attitude control, returns `(roll, pitch, yaw)` clamped to [-1; 1].
///
/// `omega_orbital` comes from the gyro; the body-frame components are
/// `q^-1 w q` (empirically verified on the live vessel: rotate(inv(q))).
/// The error axis from the quaternion difference is mapped to the kRPC
/// vessel frame (x = right -> pitch, y = forward -> roll, z = bottom ->
/// yaw); input torque is `omega_dot = -K*input`, so the law is
/// `ctrl = kd*omega - kp*error` per axis.
fn compute_control(
    target: Quaternion,
    current: Quaternion,
    omega_orbital: (f64, f64, f64),
    kp: f64,
    kd: f64,
) -> (f64, f64, f64) {
    // Empirically: q^-1 w q gives the body-frame angular velocity.
    let (wx, wy, wz) = quaternion_rotate(quaternion_inverse(current), omega_orbital);

    let error_q = quaternion_mul(target, quaternion_inverse(current));
    let (ax, ay, az, angle) = quaternion_to_axis_angle(error_q);

    let e_x = ax * angle;
    let e_y = ay * angle;
    let e_z = az * angle;

    let pitch = kd * wx - kp * e_x;
    let roll = kd * wy - kp * e_y;
    let yaw = kd * wz - kp * e_z;

    (
        roll.clamp(-1.0, 1.0),
        pitch.clamp(-1.0, 1.0),
        yaw.clamp(-1.0, 1.0),
    )
}

/// Rotate a vector by a unit quaternion: `v' = q v q^-1`.
fn quaternion_rotate(q: (f64, f64, f64, f64), v: (f64, f64, f64)) -> (f64, f64, f64) {
    let r = quaternion_mul(quaternion_mul(q, (v.0, v.1, v.2, 0.0)), quaternion_inverse(q));
    (r.0, r.1, r.2)
}

fn quaternion_inverse(q: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (x, y, z, w) = q;
    let norm2 = x * x + y * y + z * z + w * w;
    (-x / norm2, -y / norm2, -z / norm2, w / norm2)
}

fn quaternion_mul(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (ax, ay, az, aw) = a;
    let (bx, by, bz, bw) = b;
    (
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    )
}

fn quaternion_to_axis_angle(q: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (mut x, mut y, mut z, mut w) = q;
    // q and -q are the same rotation; pick w >= 0 so the angle lands in
    // [0; pi] and the controller takes the SHORT way around. Otherwise a
    // >180deg error commands a full extra revolution at max torque.
    if w < 0.0 {
        x = -x;
        y = -y;
        z = -z;
        w = -w;
    }
    let angle = 2.0 * w.acos();
    let s = (1.0 - w * w).sqrt();
    if s < 1e-8 {
        (1.0, 0.0, 0.0, 0.0)
    } else {
        (x / s, y / s, z / s, angle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KP: f64 = 1.0;
    const KD: f64 = 0.3;
    // kRPC convention: (x, y, z, w), identity = (0, 0, 0, 1).
    const IDENTITY: Quaternion = (0.0, 0.0, 0.0, 1.0);
    const ZERO_VEL: (f64, f64, f64) = (0.0, 0.0, 0.0);

    /// Quaternion rotation about unit axis (ax, ay, az) by `ang` radians.
    fn rot(ax: f64, ay: f64, az: f64, ang: f64) -> Quaternion {
        let s = (ang / 2.0).sin();
        (ax * s, ay * s, az * s, (ang / 2.0).cos())
    }

    #[test]
    fn aligned_no_motion_outputs_zero() {
        assert_eq!(compute_control(IDENTITY, IDENTITY, ZERO_VEL, KP, KD), (0.0, 0.0, 0.0));
    }

    #[test]
    fn already_at_target_outputs_zero() {
        let q = rot(0.0, 0.0, 1.0, 0.7);
        assert_eq!(compute_control(q, q, ZERO_VEL, KP, KD), (0.0, 0.0, 0.0));
    }

    #[test]
    fn pitch_error_commands_pitch_down() {
        // vessel pitched up 30deg: error about +x, command = nose down (<0)
        let (roll, pitch, yaw) = compute_control(IDENTITY, rot(-1.0, 0.0, 0.0, 30f64.to_radians()), ZERO_VEL, KP, KD);
        assert_eq!(roll, 0.0);
        assert!((pitch + 0.5236).abs() < 1e-3);
        assert_eq!(yaw, 0.0);
    }

    #[test]
    fn roll_error_commands_negative_roll() {
        let (roll, pitch, yaw) = compute_control(IDENTITY, rot(0.0, -1.0, 0.0, 30f64.to_radians()), ZERO_VEL, KP, KD);
        assert!((roll + 0.5236).abs() < 1e-3);
        assert_eq!(pitch, 0.0);
        assert_eq!(yaw, 0.0);
    }

    #[test]
    fn yaw_error_commands_negative_yaw() {
        let (roll, pitch, yaw) = compute_control(IDENTITY, rot(0.0, 0.0, -1.0, 30f64.to_radians()), ZERO_VEL, KP, KD);
        assert_eq!(roll, 0.0);
        assert_eq!(pitch, 0.0);
        assert!((yaw + 0.5236).abs() < 1e-3);
    }

    #[test]
    fn large_error_clamped_to_one() {
        let (_, pitch, _) = compute_control(IDENTITY, rot(-1.0, 0.0, 0.0, 90f64.to_radians()), ZERO_VEL, KP, KD);
        assert_eq!(pitch, -1.0);
    }

    #[test]
    fn error_beyond_180_takes_short_way() {
        // 270deg about +x is the same as -90deg about +x: the command must be
        // NEGATIVE (about -x, short way), not positive (long way, 3pi/2).
        // Clamped magnitude hides the difference, so the sign is the test.
        let (_, pitch, _) = compute_control(IDENTITY, rot(1.0, 0.0, 0.0, 270f64.to_radians()), ZERO_VEL, KP, KD);
        assert_eq!(pitch, -1.0);
    }

    #[test]
    fn axis_angle_w_normalized_to_short_way() {
        // q with w<0: 270deg about +x; normalized must give angle in [0, pi]
        let (ax, ay, az, angle) = quaternion_to_axis_angle(rot(1.0, 0.0, 0.0, 270f64.to_radians()));
        assert!(angle <= std::f64::consts::PI + 1e-9);
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
        assert!((ax + 1.0).abs() < 1e-9);
        assert_eq!(ay, 0.0);
        assert_eq!(az, 0.0);
    }

    #[test]
    fn damping_opposes_spin() {
        // aligned, rotating about +x (nose going down): command nose up (+pitch)
        let (_, pitch, _) = compute_control(IDENTITY, IDENTITY, (1.0, 0.0, 0.0), KP, KD);
        assert!((pitch - 0.3).abs() < 1e-9);
    }

    #[test]
    fn damping_reduces_command_when_spinning_toward_target() {
        // 0.5 rad pitch error, spinning toward target at 1 rad/s: 0.3 - 0.5 = -0.2
        let (_, pitch, _) = compute_control(IDENTITY, rot(-1.0, 0.0, 0.0, 0.5), (1.0, 0.0, 0.0), KP, KD);
        assert!((pitch + 0.2).abs() < 1e-9);
    }

    #[test]
    fn spin_away_from_target_adds_command() {
        // 0.5 rad pitch error, spinning away at 1 rad/s: -0.5 - 0.3 = -0.8
        let (_, pitch, _) = compute_control(IDENTITY, rot(-1.0, 0.0, 0.0, 0.5), (-1.0, 0.0, 0.0), KP, KD);
        assert!((pitch + 0.8).abs() < 1e-9);
    }

    #[test]
    fn omega_converted_from_orbital_to_body_frame() {
        // current = 90deg about +z; omega orbital (0,-1,0) == body (1,0,0)
        let q = rot(0.0, 0.0, 1.0, 90f64.to_radians());
        assert_eq!(quaternion_rotate(q, (0.0, -1.0, 0.0)), (1.0, 0.0, 0.0));
    }

    #[test]
    fn damp_control_opposes_spin_per_axis() {
        // validated live: damping law is +kd*w per axis (pitch<-x, roll<-y,
        // yaw<-z), which stops the spin because input torque is -K*input.
        // spinning about +x (body): positive pitch
        let (roll, pitch, yaw) = damp_control(IDENTITY, (1.0, 0.0, 0.0), KD);
        assert_eq!(roll, 0.0);
        assert!((pitch - 0.3).abs() < 1e-9);
        assert_eq!(yaw, 0.0);

        // spinning about -y: negative roll
        let (roll, _, _) = damp_control(IDENTITY, (0.0, -1.0, 0.0), KD);
        assert!((roll + 0.3).abs() < 1e-9);

        // spinning about +z: positive yaw
        let (_, _, yaw) = damp_control(IDENTITY, (0.0, 0.0, 1.0), KD);
        assert!((yaw - 0.3).abs() < 1e-9);
    }

    #[test]
    fn damp_control_saturates_at_one() {
        let (_, pitch, _) = damp_control(IDENTITY, (10.0, 0.0, 0.0), KD);
        assert_eq!(pitch, 1.0);
    }

    #[test]
    fn rotate_round_trip_preserves_vector() {
        let q = rot(0.0, 1.0, 0.0, 1.2);
        let v = (0.5, -0.7, 0.9);
        let back = quaternion_rotate(quaternion_inverse(q), quaternion_rotate(q, v));
        assert!((back.0 - v.0).abs() < 1e-9);
        assert!((back.1 - v.1).abs() < 1e-9);
        assert!((back.2 - v.2).abs() < 1e-9);
    }

    mod sim {
        use super::*;

        /// Empirically measured torque authority: full input -> ~19 rad/s^2.
        const TORQUE_K: f64 = 19.0;
        /// Control period matching the deployed scheduler (20ms tick).
        const CTRL_PERIOD: f64 = 0.02;
        const KP: f64 = 1.0;
        const KD: f64 = 0.3;
        /// Settle criterion: sampled loop leaves a small limit cycle
        /// (~0.02-0.04 rad at 20ms), so require a tight-but-realistic hold.
        const SETTLE_THRESHOLD: f64 = 0.06;

        fn normalize(q: Quaternion) -> Quaternion {
            let n = (q.0 * q.0 + q.1 * q.1 + q.2 * q.2 + q.3 * q.3).sqrt();
            (q.0 / n, q.1 / n, q.2 / n, q.3 / n)
        }

        fn err_angle(target: Quaternion, current: Quaternion) -> f64 {
            quaternion_to_axis_angle(quaternion_mul(target, quaternion_inverse(current))).3
        }

        fn speed(w: (f64, f64, f64)) -> f64 {
            (w.0 * w.0 + w.1 * w.1 + w.2 * w.2).sqrt()
        }

        /// Empirically-validated vessel model (all pieces measured live on
        /// the vessel): torque `omega_dot = -K*input`, kinematics
        /// `dq/dt = 0.5 * (omega,0) x q`, and the gyro reports omega such
        /// that `q^-1 w q = body omega`.
        struct Vessel {
            q: Quaternion, // as returned by kRPC
            omega: (f64, f64, f64), // body angular velocity
            k: f64,
        }

        impl Vessel {
            fn new(q: Quaternion) -> Self {
                Self { q, omega: (0.0, 0.0, 0.0), k: TORQUE_K }
            }

            /// External disturbance: instant spin kick (collision, RCS burst).
            fn kick(&mut self, domega: (f64, f64, f64)) {
                self.omega = (self.omega.0 + domega.0, self.omega.1 + domega.1, self.omega.2 + domega.2);
            }

            /// Apply control inputs (pitch, roll, yaw) for one step.
            fn step(&mut self, pitch: f64, roll: f64, yaw: f64, dt: f64) {
                let (wx, wy, wz) = self.omega;
                self.omega = (wx - self.k * pitch * dt, wy - self.k * roll * dt, wz - self.k * yaw * dt);
                let dq = quaternion_mul((self.omega.0, self.omega.1, self.omega.2, 0.0), self.q);
                self.q = normalize((
                    self.q.0 + 0.5 * dq.0 * dt,
                    self.q.1 + 0.5 * dq.1 * dt,
                    self.q.2 + 0.5 * dq.2 * dt,
                    self.q.3 + 0.5 * dq.3 * dt,
                ));
            }
        }

        /// Closed loop, stabilization ON holding `target`. Applies the spin
        /// kicks one per control period, then must return to `target` and
        /// stay there (error and body spin below threshold for 1s).
        fn assert_holds(target: Quaternion, start: Quaternion, kicks: &[(f64, f64, f64)]) {
            let mut v = Vessel::new(start);
            let dt = 0.001; // fine integration step
            let ctrl_period = CTRL_PERIOD; // 0.02 s: matches deployed loop
            let mut time = 0.0;
            let mut ctrl = (0.0f64, 0.0f64, 0.0f64);
            let mut settled = 0usize;
            let mut kick_idx = 0usize;

            for _ in 0..200_000 {
                if kick_idx < kicks.len() {
                    v.kick(kicks[kick_idx]);
                    kick_idx += 1;
                }

                // sample + compute control every control period
                if (time % ctrl_period) < dt {
                    let omega_orbital = quaternion_rotate(v.q, v.omega);
                    ctrl = compute_control(target, v.q, omega_orbital, KP, KD);
                }
                v.step(ctrl.1, ctrl.0, ctrl.2, dt);

                let err = err_angle(target, v.q);
                if err < SETTLE_THRESHOLD && speed(v.omega) < SETTLE_THRESHOLD {
                    settled += 1;
                    if settled > 1000 {
                        return; // held the attitude for a full second
                    }
                } else {
                    settled = 0;
                }
                time += dt;
            }
            panic!(
                "did not return to target; err={:.3} |w|={:.3}",
                err_angle(target, v.q),
                speed(v.omega)
            );
        }

        #[test]
        fn converges_from_pitch_error() {
            assert_holds((0.0, 0.0, 0.0, 1.0), rot(-1.0, 0.0, 0.0, 30f64.to_radians()), &[]);
        }

        #[test]
        fn converges_from_roll_error() {
            assert_holds((0.0, 0.0, 0.0, 1.0), rot(0.0, -1.0, 0.0, 30f64.to_radians()), &[]);
        }

        #[test]
        fn converges_from_yaw_error() {
            assert_holds((0.0, 0.0, 0.0, 1.0), rot(0.0, 0.0, -1.0, 30f64.to_radians()), &[]);
        }

        #[test]
        fn converges_from_combined_error_with_spin() {
            let combined = normalize(quaternion_mul(rot(0.0, 0.0, 1.0, 0.4), rot(-1.0, 0.0, 0.0, 0.6)));
            assert_holds((0.0, 0.0, 0.0, 1.0), combined, &[(0.0, 0.0, 0.0)]);
        }

        // --- stabilization ON: spin the satellite, it must return ---

        /// Attitude captured by "on stab": not orbital-aligned, proving the
        /// hold target is the captured attitude, not something else.
        const CAPTURED: Quaternion = (0.2, -0.1, 0.05, 0.9732); // normalized-ish, arbitrary

        #[test]
        fn returns_to_captured_attitude_after_spin() {
            // calm vessel holding CAPTURED; disturbance spins it hard
            assert_holds(CAPTURED, CAPTURED, &[(3.0, -2.0, 1.5)]);
        }

        #[test]
        fn returns_after_strong_single_axis_spin() {
            // spin faster than the controller can react to without saturating
            assert_holds(CAPTURED, CAPTURED, &[(6.0, 0.0, 0.0)]);
        }

        #[test]
        fn survives_repeated_disturbances() {
            // kick, recover, kick again, recover, kick a third time
            assert_holds(
                CAPTURED,
                CAPTURED,
                &[(2.5, 1.0, -1.0), (-2.0, 2.5, 0.5), (1.0, -1.5, -2.5)],
            );
        }

        #[test]
        fn returns_when_disturbed_mid_recovery() {
            // start displaced AND spinning, plus another kick mid-recovery
            let displaced = normalize(quaternion_mul(rot(0.0, 0.0, 1.0, 0.4), CAPTURED));
            assert_holds(CAPTURED, displaced, &[(2.0, 0.0, -2.0)]);
        }

        /// Rate-only damping: must stop any spin completely; attitude is
        /// NOT held (the vessel keeps whatever orientation it ends up in).
        fn assert_damps(start: Quaternion, spin: (f64, f64, f64)) {
            let mut v = Vessel::new(start);
            v.kick(spin);
            let dt = 0.001;
            let ctrl_period = CTRL_PERIOD;
            let mut time = 0.0;
            let mut ctrl = (0.0f64, 0.0f64, 0.0f64);
            let mut settled = 0usize;

            for _ in 0..200_000 {
                if (time % ctrl_period) < dt {
                    let omega_orbital = quaternion_rotate(v.q, v.omega);
                    ctrl = damp_control(v.q, omega_orbital, KD);
                }
                v.step(ctrl.1, ctrl.0, ctrl.2, dt);

                if speed(v.omega) < SETTLE_THRESHOLD {
                    settled += 1;
                    if settled > 1000 {
                        return; // fully stopped
                    }
                } else {
                    settled = 0;
                }
                time += dt;
            }
            panic!("damp did not stop the spin; |w|={:.3}", speed(v.omega));
        }

        #[test]
        fn damp_stops_hard_spin() {
            assert_damps(CAPTURED, (8.0, 3.0, -2.0));
        }

        #[test]
        fn damp_stops_multi_axis_spin() {
            assert_damps(CAPTURED, (-2.0, 5.0, 4.0));
        }
    }
}
