use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::car::Car;

/// A single snapshot of high-frequency vehicle telemetry during a simulation tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPoint {
    /// Simulation elapsed time in seconds.
    pub time: f32,
    /// 2D world position in meters.
    pub position: Vec2,
    /// Absolute vehicle ground speed in m/s.
    pub speed_mps: f32,
    /// Speed in km/h.
    pub speed_kmh: f32,
    /// Longitudinal acceleration in g (forward = positive, braking/drag = negative).
    pub accel_longitudinal_g: f32,
    /// Lateral cornering acceleration in g (right = positive, left = negative).
    pub accel_lateral_g: f32,
    /// Yaw orientation angle in radians (-PI..PI].
    pub yaw_angle_rad: f32,
    /// Yaw angular velocity in rad/s.
    pub yaw_rate_rad_s: f32,
    /// Steering angle in radians.
    pub steer_angle_rad: f32,
    /// Chassis sideslip angle in radians (angle between heading and velocity vector).
    pub sideslip_angle_rad: f32,
    /// Longitudinal slip ratio for each wheel [FL, FR, RL, RR].
    pub wheel_slips: [f32; 4],
    /// Lateral slip angle in radians for each wheel [FL, FR, RL, RR].
    pub wheel_slip_angles: [f32; 4],
    /// Anti-lock braking system active flag.
    pub abs_active: bool,
    /// Traction control system active flag.
    pub tcs_active: bool,
    /// Electronic stability control active flag.
    pub esc_active: bool,
}

impl TelemetryPoint {
    /// Captures a telemetry point from current car state and simulation timestamp.
    pub fn capture(car: &Car, time: f32) -> Self {
        let state = car.state();
        let g = 9.80665f32;
        let mut wheel_slips = [0.0; 4];
        let mut wheel_slip_angles = [0.0; 4];
        for i in 0..4 {
            wheel_slips[i] = state.wheels[i].slip_ratio;
            wheel_slip_angles[i] = state.wheels[i].slip_angle;
        }

        Self {
            time,
            position: state.position,
            speed_mps: state.speed,
            speed_kmh: state.speed * 3.6,
            accel_longitudinal_g: state.acceleration_local.x / g,
            accel_lateral_g: state.acceleration_local.y / g,
            yaw_angle_rad: state.angle,
            yaw_rate_rad_s: state.angular_velocity,
            steer_angle_rad: state.steer_angle,
            sideslip_angle_rad: state.sideslip_angle,
            wheel_slips,
            wheel_slip_angles,
            abs_active: state.abs_active,
            tcs_active: state.tcs_active,
            esc_active: state.esc_active,
        }
    }
}
