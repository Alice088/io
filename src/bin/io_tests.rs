//! `io_tests` — standalone kRPC connectivity smoke test.
//!
//! Connects to a running kRPC server and fetches the active vessel.
//! Usage: `cargo run --bin io_tests` (kRPC server must be up).

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

    let frame = vessel.reference_frame().await?;
    let current_q = &Quaternion::from(vessel.rotation(&frame).await?);
    let current_norm_q = Quaternion::normalize(current_q);

    // println!("current_q: {:?}", current_q);
    // println!("current_norm_q: {:?}", current_norm_q);
    let target_q = Quaternion::normalize(&Quaternion {
        w: (90.0_f64.to_radians() / 2.0).cos(),
        x: 0.0,
        y: (90.0_f64.to_radians() / 2.0).sin(),
        z: 0.0,
    });

    // let qerror = current_norm_q.mul(&target_q.inverse());

    // let angle = 2.0 * qerror.w.acos();
    // let s = (1.0 - qerror.w * qerror.w).sqrt();

    // let axis_x = qerror.x / s;
    // let axis_y = qerror.y / s;
    // let axis_z = qerror.z / s;

    // println!("target offset: {angle:.3} rad about ({axis_x:.3}, {axis_y:.3}, {axis_z:.3})");

    // let kp = 0.5;

    // let roll_cmd = (kp * axis_x * angle).clamp(-1.0, 1.0);
    // let pitch_cmd = (kp * axis_y * angle).clamp(-1.0, 1.0);
    // let yaw_cmd = (kp * axis_z * angle).clamp(-1.0, 1.0);

    // control.set_roll(roll_cmd as f32).await?;
    // control.set_pitch(0.5).await?;
    // control.set_yaw(yaw_cmd as f32).await?;

    // sleep(Duration::new(1, 0)).await;

    // control.set_roll(0.0).await?;
    // control.set_pitch(0.0).await?;
    // control.set_yaw(0.0).await?;

    // loop {
    //     let current = Quaternion::normalize(&Quaternion::from(vessel.rotation(&frame).await?));

    //     let qerror = target_q.mul(&current.inverse());

    //     let angle = 2.0 * qerror.w.clamp(-1.0, 1.0).acos();

    //     println!(
    //         "error: {:.2}°, q = ({:.4}, {:.4}, {:.4}, {:.4})",
    //         angle.to_degrees(),
    //         qerror.w,
    //         qerror.x,
    //         qerror.y,
    //         qerror.z
    //     );

    //     if angle < 2.0_f64.to_radians() {
    //         control.set_pitch(0.0).await?;
    //         break;
    //     }

    //     let command = (0.5 * qerror.y).clamp(-1.0, 1.0);

    //     control.set_roll(0.0).await?;
    //     control.set_pitch(command as f32).await?;
    //     control.set_yaw(0.0).await?;

    //     tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    // }

    let current = Quaternion::normalize(&Quaternion::from(vessel.rotation(&frame).await?));

    let delta = Quaternion {
        w: (45.0_f64.to_radians()).cos(),
        x: 0.0,
        y: (45.0_f64.to_radians()).sin(),
        z: 0.0,
    };

    let target = Quaternion::normalize(&delta.mul(&current));

    println!("current = {:?}", current);
    println!("target  = {:?}", target);

    let error = target.mul(&current.inverse());

    println!("error   = {:?}", error);

    let angle = 2.0 * error.w.acos();
    let s = (1.0 - error.w * error.w).sqrt();

    let axis_y = error.y / s;

    let command = (0.5 * axis_y * angle).clamp(-1.0, 1.0);

    println!(
        "angle: {:.2}°, axis_y: {:.3}, pitch: {:.3}",
        angle.to_degrees(),
        axis_y,
        command
    );

    let axis_x = error.x / s;
    let axis_y = error.y / s;
    let axis_z = error.z / s;

    let kp = 0.5;

    let roll_cmd = (kp * axis_x * angle).clamp(-1.0, 1.0);
    let pitch_cmd = (kp * axis_y * angle).clamp(-1.0, 1.0);
    let yaw_cmd = (kp * axis_z * angle).clamp(-1.0, 1.0);

    control.set_roll(roll_cmd as f32).await?;
    control.set_pitch(pitch_cmd as f32).await?;
    control.set_yaw(yaw_cmd as f32).await?;

    sleep(Duration::new(1, 0)).await;

    control.set_roll(0.0).await?;
    control.set_pitch(0.0).await?;
    control.set_yaw(0.0).await?;

    let current_move = Quaternion::normalize(&Quaternion::from(vessel.rotation(&frame).await?));

    println!("current after move {:?}", current_move);
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
pub struct RotationError {
    pub axis_x: f64,
    pub axis_y: f64,
    pub axis_z: f64,
    pub angle: f64,
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
            y: current.0,
            x: current.1,
            z: current.2,
        }
    }
}

impl Quaternion {
    pub fn new() -> Self {
        Self {
            w: 1.0,
            y: 0.0,
            x: 0.0,
            z: 0.0,
        }
    }

    pub fn from(current: (f64, f64, f64, f64)) -> Self {
        Self {
            w: current.0,
            y: current.1,
            x: current.2,
            z: current.3,
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
