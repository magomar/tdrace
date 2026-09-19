use std::f32::consts::PI;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::car::{normalize_angle, CarControls};
use crate::config::CarConfig;
use crate::surface::SurfaceType;
use super::harness::SimulationRunner;

const G_ACCEL: f32 = 9.80665;

// ============================================================================
// Protocol A: Acceleration & Traction
// ============================================================================

/// Output metrics from Protocol A (Standing Start Acceleration).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolAResult {
    pub surface: SurfaceType,
    pub t50_s: Option<f32>,
    pub t100_s: Option<f32>,
    pub t160_s: Option<f32>,
    pub t400m_s: Option<f32>,
    pub v400m_kmh: Option<f32>,
    pub peak_accel_g: f32,
    pub wheelspin_loss_index: f32,
    pub v_terminal_kmh: f32,
    pub total_sim_time_s: f32,
}

/// Runs Protocol A: Standing start longitudinal acceleration to 160 km/h or 400m.
pub fn run_protocol_a(
    config: &CarConfig,
    surface: SurfaceType,
    timeout_s: f32,
    dt: f32,
) -> ProtocolAResult {
    let mut runner = SimulationRunner::new(config.clone(), dt);

    let mut t50 = None;
    let mut t100 = None;
    let mut t160 = None;
    let mut t400m = None;
    let mut v400m = None;
    let mut peak_accel_g = 0.0f32;
    let mut wheelspin_loss_index = 0.0f32;

    let target_50 = 50.0 / 3.6;
    let target_100 = 100.0 / 3.6;
    let target_160 = 160.0 / 3.6;

    runner.run_until(
        timeout_s,
        surface,
        |_t, _car| CarControls::accelerate(),
        |t, car| {
            let speed = car.state().speed;
            let dist = car.state().position.x;
            let accel_g = car.state().acceleration_local.x / G_ACCEL;

            if accel_g > peak_accel_g {
                peak_accel_g = accel_g;
            }

            if t50.is_none() && speed >= target_50 {
                t50 = Some(t);
            }
            if t100.is_none() && speed >= target_100 {
                t100 = Some(t);
            }
            if t160.is_none() && speed >= target_160 {
                t160 = Some(t);
            }
            if t400m.is_none() && dist >= 400.0 {
                t400m = Some(t);
                v400m = Some(speed * 3.6);
            }

            // Wheelspin penalty integration for driven rear wheels
            let slip_rl = car.state().wheels[2].slip_ratio.abs();
            let slip_rr = car.state().wheels[3].slip_ratio.abs();
            let avg_rear_slip = (slip_rl + slip_rr) * 0.5;
            if avg_rear_slip > 0.15 {
                let slip_penalty = ((avg_rear_slip - 0.15) / 0.85).min(1.0);
                wheelspin_loss_index += slip_penalty * dt;
            }

            // Terminate if reached 160 km/h AND passed 400m
            let done_speed = speed >= target_160 || (t > 15.0 && speed < target_100);
            let done_dist = dist >= 400.0;
            done_speed && done_dist
        },
    );

    ProtocolAResult {
        surface,
        t50_s: t50,
        t100_s: t100,
        t160_s: t160,
        t400m_s: t400m,
        v400m_kmh: v400m,
        peak_accel_g,
        wheelspin_loss_index,
        v_terminal_kmh: runner.car.state().speed * 3.6,
        total_sim_time_s: runner.time,
    }
}

// ============================================================================
// Protocol B: Braking & Deceleration
// ============================================================================

/// Output metrics from Protocol B (100-0 km/h Emergency Braking).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolBResult {
    pub surface: SurfaceType,
    pub v0_kmh: f32,
    pub stopping_distance_m: f32,
    pub stopping_time_s: f32,
    pub avg_decel_g: f32,
    pub peak_decel_g: f32,
    pub lockup_duration_s: f32,
    pub abs_efficiency: f32,
}

/// Runs Protocol B: Emergency braking from 100 km/h to standstill.
pub fn run_protocol_b(
    config: &CarConfig,
    surface: SurfaceType,
    v0_kmh: f32,
    dt: f32,
) -> ProtocolBResult {
    let v0_mps = v0_kmh / 3.6;
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    let mut peak_decel_g = 0.0f32;
    let mut lockup_duration_s = 0.0f32;

    runner.run_until(
        30.0,
        surface,
        |_t, _car| CarControls::full_brake(),
        |_t, car| {
            let decel_g = (-car.state().acceleration_local.x) / G_ACCEL;
            if decel_g > peak_decel_g {
                peak_decel_g = decel_g;
            }

            // Check tire lockup
            let is_locked = car.state().wheels.iter().any(|w| w.slip_ratio.abs() >= 0.95);
            if is_locked {
                lockup_duration_s += dt;
            }

            car.state().speed <= 0.05
        },
    );

    let stopping_distance_m = runner.car.state().position.x.max(0.1);
    let stopping_time_s = runner.time;
    let avg_decel_g = (v0_mps * v0_mps) / (2.0 * stopping_distance_m * G_ACCEL);
    let mu = surface.friction_coefficient().max(0.01);
    let abs_efficiency = (avg_decel_g / mu).min(1.5);

    ProtocolBResult {
        surface,
        v0_kmh,
        stopping_distance_m,
        stopping_time_s,
        avg_decel_g,
        peak_decel_g,
        lockup_duration_s,
        abs_efficiency,
    }
}

// ============================================================================
// Protocol C: Constant-Radius Skidpad
// ============================================================================

/// Departure mode for skidpad testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkidpadDepartureMode {
    Understeer,
    OversteerSpin,
    Timeout,
}

/// Output metrics from Protocol C (Constant Radius Skidpad).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolCResult {
    pub surface: SurfaceType,
    pub radius_m: f32,
    pub peak_lateral_accel_g: f32,
    pub critical_speed_kmh: f32,
    pub sideslip_at_limit_deg: f32,
    pub understeer_gradient: f32,
    pub departure_mode: SkidpadDepartureMode,
    pub test_duration_s: f32,
}

/// Runs Protocol C: Constant radius circle skidpad (default R = 30m).
pub fn run_protocol_c(
    config: &CarConfig,
    surface: SurfaceType,
    radius_m: f32,
    dt: f32,
) -> ProtocolCResult {
    // Start at (R, 0) pointing north (+y, angle = PI/2) with speed = 15 km/h
    let v_init = 15.0 / 3.6;
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(Vec2::new(radius_m, 0.0), PI * 0.5, Vec2::new(0.0, v_init));

    let mut peak_lat_accel_g = 0.0f32;
    let mut crit_speed_kmh = 15.0f32;
    let mut sideslip_at_limit_deg = 0.0f32;
    let mut departure = SkidpadDepartureMode::Timeout;

    // Track steering vs lateral acceleration for understeer gradient
    let mut steer_angles_deg = Vec::new();
    let mut lat_accels_g = Vec::new();

    let speed_ramp_rate = 0.1389; // +0.5 km/h per second

    runner.run_until(
        60.0,
        surface,
        |t, car| {
            let pos = car.state().position;
            let current_r = pos.length();
            let radial_error = current_r - radius_m;

            // Desired heading tangent along circle counter-clockwise: atan2(y, x) + PI/2
            let polar_angle = pos.y.atan2(pos.x);
            let target_heading = normalize_angle(polar_angle + PI * 0.5);
            let heading_error = normalize_angle(target_heading - car.state().angle);

            // Stanley / geometric steering demand
            let steer_demand = (1.5 * heading_error - 0.20 * radial_error).clamp(-1.0, 1.0);

            // Longitudinal speed ramp
            let target_v = v_init + speed_ramp_rate * t;
            let v_err = target_v - car.state().speed;
            let (throttle, brake) = if v_err > 0.0 {
                ((v_err * 0.6).clamp(0.0, 1.0), 0.0)
            } else {
                (0.0, ((-v_err) * 0.4).clamp(0.0, 1.0))
            };

            CarControls {
                throttle,
                steer: steer_demand,
                brake,
                handbrake: false,
                reverse: false,
            }
        },
        |t, car| {
            let pos = car.state().position;
            let current_r = pos.length();
            let radial_err = (current_r - radius_m).abs();
            let speed = car.state().speed;
            let lat_accel_g = (car.state().acceleration_local.y.abs()) / G_ACCEL;
            let sideslip_deg = car.state().sideslip_angle.abs().to_degrees();

            if lat_accel_g > peak_lat_accel_g {
                peak_lat_accel_g = lat_accel_g;
                crit_speed_kmh = speed * 3.6;
                sideslip_at_limit_deg = sideslip_deg;
            }

            if t > 1.0 {
                steer_angles_deg.push(car.state().steer_angle.to_degrees().abs());
                lat_accels_g.push(lat_accel_g);
            }

            // Departure check
            if sideslip_deg > 45.0 {
                departure = SkidpadDepartureMode::OversteerSpin;
                return true;
            }
            if radial_err > 3.0 && t > 2.0 {
                departure = SkidpadDepartureMode::Understeer;
                return true;
            }

            false
        },
    );

    // Compute understeer gradient (deg/g)
    let understeer_gradient = if steer_angles_deg.len() >= 10 {
        let first_steer = steer_angles_deg[steer_angles_deg.len() / 4];
        let last_steer = steer_angles_deg[steer_angles_deg.len() - 1];
        let first_ay = lat_accels_g[lat_accels_g.len() / 4];
        let last_ay = lat_accels_g[lat_accels_g.len() - 1];
        let d_ay = (last_ay - first_ay).abs();
        if d_ay > 0.05 {
            (last_steer - first_steer) / d_ay
        } else {
            0.0
        }
    } else {
        0.0
    };

    ProtocolCResult {
        surface,
        radius_m,
        peak_lateral_accel_g: peak_lat_accel_g,
        critical_speed_kmh: crit_speed_kmh,
        sideslip_at_limit_deg,
        understeer_gradient,
        departure_mode: departure,
        test_duration_s: runner.time,
    }
}

// ============================================================================
// Protocol D: Transient Step-Steer & Slalom
// ============================================================================

/// Vehicle stability classification for transient maneuvers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepSteerStatus {
    Stable,
    Drifting,
    Spun,
}

/// Output metrics from Protocol D (Transient Step Steer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolDResult {
    pub surface: SurfaceType,
    pub test_speed_kmh: f32,
    pub peak_yaw_rate_deg_s: f32,
    pub yaw_response_delay_s: f32,
    pub max_sideslip_deg: f32,
    pub recovery_status: StepSteerStatus,
}

/// Runs Protocol D: Transient step steer (80 km/h with 25° step-steer sequence).
pub fn run_protocol_d(
    config: &CarConfig,
    surface: SurfaceType,
    v0_kmh: f32,
    dt: f32,
) -> ProtocolDResult {
    let v0_mps = v0_kmh / 3.6;
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    let mut peak_yaw_rate = 0.0f32;
    let mut max_sideslip_rad = 0.0f32;
    let mut time_at_90pct_yaw = 0.0f32;
    let mut reached_90pct = false;

    runner.run_until(
        5.0,
        surface,
        |t, car| {
            // Sequence:
            // 0.0..1.5s: Step steer +0.40
            // 1.5..3.0s: Counter-steer -0.40
            // 3.0..5.0s: Center 0.0
            let steer = if t < 1.5 {
                0.40
            } else if t < 3.0 {
                -0.40
            } else {
                0.0
            };

            // Maintain target speed
            let v_err = v0_mps - car.state().speed;
            let throttle = (v_err * 0.8).clamp(0.0, 1.0);

            CarControls {
                throttle,
                steer,
                brake: 0.0,
                handbrake: false,
                reverse: false,
            }
        },
        |t, car| {
            let yaw_rate = car.state().angular_velocity.abs();
            let sideslip = car.state().sideslip_angle.abs();

            if yaw_rate > peak_yaw_rate {
                peak_yaw_rate = yaw_rate;
            }
            if sideslip > max_sideslip_rad {
                max_sideslip_rad = sideslip;
            }

            if !reached_90pct && peak_yaw_rate > 0.1 && yaw_rate >= 0.90 * peak_yaw_rate {
                time_at_90pct_yaw = t;
                reached_90pct = true;
            }

            false
        },
    );

    let max_sideslip_deg = max_sideslip_rad.to_degrees();
    let final_sideslip_deg = runner.car.state().sideslip_angle.abs().to_degrees();
    let final_yaw_rate_deg = runner.car.state().angular_velocity.abs().to_degrees();

    let recovery_status = if max_sideslip_deg > 55.0 || final_yaw_rate_deg > 60.0 {
        StepSteerStatus::Spun
    } else if final_sideslip_deg > 15.0 || max_sideslip_deg > 25.0 {
        StepSteerStatus::Drifting
    } else {
        StepSteerStatus::Stable
    };

    ProtocolDResult {
        surface,
        test_speed_kmh: v0_kmh,
        peak_yaw_rate_deg_s: peak_yaw_rate.to_degrees(),
        yaw_response_delay_s: time_at_90pct_yaw,
        max_sideslip_deg,
        recovery_status,
    }
}

// ============================================================================
// Protocol E: Coast-Down Resistance
// ============================================================================

/// Output metrics from Protocol E (Coast-Down Resistance).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolEResult {
    pub surface: SurfaceType,
    pub v0_kmh: f32,
    pub coast_time_s: f32,
    pub coast_distance_m: f32,
    pub avg_drag_decel_g: f32,
}

/// Runs Protocol E: Coast-down from 120 km/h with 0 throttle and 0 brake.
pub fn run_protocol_e(
    config: &CarConfig,
    surface: SurfaceType,
    v0_kmh: f32,
    dt: f32,
) -> ProtocolEResult {
    let v0_mps = v0_kmh / 3.6;
    let mut neutral_config = config.clone();
    neutral_config.engine_braking_coefficient = 0.0; // Disengage transmission (neutral coasting)
    let mut runner = SimulationRunner::new(neutral_config, dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    runner.run_until(
        60.0,
        surface,
        |_t, _car| CarControls::default(),
        |_t, car| car.state().speed <= 0.05,
    );

    let coast_distance_m = runner.car.state().position.x.max(0.1);
    let coast_time_s = runner.time;
    let avg_drag_decel_g = (v0_mps * v0_mps) / (2.0 * coast_distance_m * G_ACCEL);

    ProtocolEResult {
        surface,
        v0_kmh,
        coast_time_s,
        coast_distance_m,
        avg_drag_decel_g,
    }
}
