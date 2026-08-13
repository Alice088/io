use std::time::Duration;

use anyhow::Context;
use stayputnik::services::space_center::SpaceCenter;
use stayputnik::Client;
use tokio::time::sleep;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 50_000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("connecting to kRPC at {HOST}:{PORT} ...");

    let client = Client::connect("io-tests", HOST, PORT)
        .await
        .context("kRPC connect failed (server up?)")?
        .into_shared();

    let sc = SpaceCenter::new(client.clone());
    let vessel = sc
        .active_vessel()
        .await
        .context("no active vessel (build one in KSP)")?;

    let name = vessel.name().await.context("failed to read vessel name")?;

    println!("connected ✓");
    println!("active vessel: {name}");

    let control = vessel
        .control()
        .await
        .context("no active vessel (build one in KSP)")?;
    println!("control handle: {control:?}");

    let kp = 1.0;
    let kd = 0.5;

    let frame = vessel.orbital_reference_frame().await?;
    let target_q = Quaternion::from(vessel.rotation(&frame).await?);

    let mut iteration = 0;
    loop {
        sleep(Duration::from_millis(10)).await;

        iteration += 1;
        println!("=== Iter {} ===", iteration);

        let current_q = Quaternion::from(vessel.rotation(&frame).await?);
        println!("current_q: {:?}", current_q);
        println!("target_q: {:?}", target_q);

        let mut error_q = Quaternion::normalize(&Quaternion::inverse(&current_q).mul(&target_q));
        if error_q.w < 0.0 {
            error_q.full_inverse();
        }
        println!(
            "error_q (vec): ({:.3}, {:.3}, {:.3})",
            error_q.x, error_q.y, error_q.z
        );

        let (ax, ay, az, angle) = error_q.to_axis_angle();

        let error_vec = [ax * angle, ay * angle, az * angle];

        let velocity = vessel.angular_velocity(&frame).await?;
        println!(
            "velocity: ({:.3}, {:.3}, {:.3})",
            velocity.0, velocity.1, velocity.2
        );

        let local_vel = Quaternion::rotate_vector(&current_q, velocity);
        println!(
            "velocity local: ({:.3}, {:.3}, {:.3})",
            local_vel.0, local_vel.1, local_vel.2
        );

        println!("angle: {angle}");
        const ANGLE_THRESHOLD: f64 = 0.02; // ~1°
        const VEL_THRESHOLD: f64 = 0.01; // рад/с
        if angle < ANGLE_THRESHOLD
            && local_vel.0.abs() < VEL_THRESHOLD
            && local_vel.1.abs() < VEL_THRESHOLD
            && local_vel.2.abs() < VEL_THRESHOLD
        {
            println!("Target reached, holding.");
            control.set_roll(0.0).await?;
            control.set_pitch(0.0).await?;
            control.set_yaw(0.0).await?;
            continue;
        }

        let cm = VecControlMomentum {
            x: -kp * error_vec[0] - kd * local_vel.0,
            y: -kp * error_vec[1] - kd * local_vel.1,
            z: -kp * error_vec[2] - kd * local_vel.2,
        };
        println!("cm: ({:.3}, {:.3}, {:.3})", cm.x, cm.y, cm.z);

        const MAX_MOMENT: f64 = 1.0;
        let cmd_x = (cm.x / MAX_MOMENT).clamp(-1.0, 1.0);
        let cmd_y = (cm.y / MAX_MOMENT).clamp(-1.0, 1.0);
        let cmd_z = (cm.z / MAX_MOMENT).clamp(-1.0, 1.0);
        println!("cmd: x={:.3}, y={:.3}, z={:.3}", cmd_x, cmd_y, cmd_z);

        println!("Setting controls...");
        control.set_roll(cmd_y as f32).await?;
        control.set_pitch(cmd_x as f32).await?;
        control.set_yaw(cmd_z as f32).await?;
        println!("Controls set.");
    }

    Ok(())
}

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
pub struct VecControlMomentum {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl AngularVelocity {
    pub fn new() -> Self {
        Self {
            y: 0.0,
            x: 0.0,
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
            x: current.0,
            y: current.1,
            z: current.2,
            w: current.3,
        }
    }

    pub fn normalize(target: &Quaternion) -> NormalizedQuaternion {
        let q_len =
            (target.w.powf(2.0) + target.y.powf(2.0) + target.x.powf(2.0) + target.z.powf(2.0))
                .sqrt();

        NormalizedQuaternion {
            w: target.w / q_len,
            y: target.y / q_len,
            x: target.x / q_len,
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

    pub fn full_inverse(&mut self) {
        self.w = -self.w;
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
    }

    pub fn to_axis_angle(&self) -> (f64, f64, f64, f64) {
        let w = self.w.clamp(-1.0, 1.0);
        let angle = 2.0 * w.acos();
        let sin_half = (1.0 - w * w).sqrt();
        if sin_half < 1e-10 {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let scale = 1.0 / sin_half;
        (self.x * scale, self.y * scale, self.z * scale, angle)
    }

    fn rotate_vector(q: &Quaternion, v: (f64, f64, f64)) -> (f64, f64, f64) {
        let v_q = Quaternion {
            w: 0.0,
            x: v.0,
            y: v.1,
            z: v.2,
        };
        let q_inv = q.inverse();
        let tmp = q_inv.mul(&v_q);
        let res = tmp.mul(&q);
        (res.x, res.y, res.z)
    }
}
