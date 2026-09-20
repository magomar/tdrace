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

            // Wheelspin penalty integration for driven wheels
            let avg_driven_slip = if config.drive_bias > 0.8 {
                (car.state().wheels[0].slip_ratio.abs() + car.state().wheels[1].slip_ratio.abs()) * 0.5
            } else if config.drive_bias > 0.2 {
                (car.state().wheels[0].slip_ratio.abs()
                    + car.state().wheels[1].slip_ratio.abs()
                    + car.state().wheels[2].slip_ratio.abs()
                    + car.state().wheels[3].slip_ratio.abs())
                    * 0.25
            } else {
                (car.state().wheels[2].slip_ratio.abs() + car.state().wheels[3].slip_ratio.abs()) * 0.5
            };
            if avg_driven_slip > 0.15 {
                let slip_penalty = ((avg_driven_slip - 0.15) / 0.85).min(1.0);
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

    let speed_ramp_rate = 0.5556; // +2.0 km/h per second

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

            // Kinematic Ackermann feedforward (negative steer commands counter-clockwise left turn)
            let speed_factor = 1.0 + car.state().speed * config.speed_sensitive_steer_factor;
            let steer_ff = -(config.wheelbase / (radius_m * config.max_steer_angle)) * speed_factor;

            // Geometric closed-loop steering feedback (cross-track and heading regulation)
            let steer_fb = -(1.8 * heading_error + 0.15 * radial_error);
            let steer_demand = (steer_ff + steer_fb).clamp(-1.0, 1.0);

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
            if sideslip_deg > 40.0 {
                departure = SkidpadDepartureMode::OversteerSpin;
                return true;
            }
            if radial_err > 4.5 && t > 1.5 {
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

// ============================================================================
// Protocol F: Comprehensive Braking Dynamics & Surface Stability Benchmark
// ============================================================================

/// Stability rating for straight-line braking maneuvers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrakingStabilityRating {
    Stable,
    YawWander,
    Spinout,
}

/// Dynamic behavior classification during cornering trail-braking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorneringBrakingBehavior {
    CleanTrailBrake,
    UndersteerPlow,
    SnapOversteerSpin,
}

/// Stability status during asymmetric split-mu braking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitMuStatus {
    TrackedStraight,
    PullsGripSide,
    SpunOut,
}

/// Output metrics from Straight-Line Panic Braking (with or without yaw perturbation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrakingStraightLineResult {
    pub surface: SurfaceType,
    pub v0_kmh: f32,
    pub with_yaw_disturbance: bool,
    pub stopping_distance_m: f32,
    pub stopping_time_s: f32,
    pub avg_decel_g: f32,
    pub peak_decel_g: f32,
    pub max_sideslip_deg: f32,
    pub max_yaw_rate_deg_s: f32,
    pub lateral_displacement_m: f32,
    pub heading_deviation_deg: f32,
    pub front_lockup_duration_s: f32,
    pub rear_lockup_duration_s: f32,
    pub abs_active_pct: f32,
    pub stability_rating: BrakingStabilityRating,
}

/// Output metrics from Split-mu (asymmetric surface) braking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrakingSplitMuResult {
    pub high_mu_surface: SurfaceType,
    pub low_mu_surface: SurfaceType,
    pub v0_kmh: f32,
    pub stopping_distance_m: f32,
    pub stopping_time_s: f32,
    pub avg_decel_g: f32,
    pub peak_yaw_rate_deg_s: f32,
    pub max_sideslip_deg: f32,
    pub lateral_lane_drift_m: f32,
    pub heading_deviation_deg: f32,
    pub status: SplitMuStatus,
}

/// Output metrics from Cornering Trail-Braking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrakingCorneringResult {
    pub surface: SurfaceType,
    pub v0_kmh: f32,
    pub radius_m: f32,
    pub peak_yaw_rate_deg_s: f32,
    pub max_sideslip_deg: f32,
    pub stopping_distance_along_path_m: f32,
    pub radial_path_divergence_m: f32,
    pub behavior: CorneringBrakingBehavior,
}

/// Output metrics from Cadence / Pulsed Brake modulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrakingCadenceResult {
    pub surface: SurfaceType,
    pub v0_kmh: f32,
    pub stopping_distance_m: f32,
    pub stopping_time_s: f32,
    pub avg_decel_g: f32,
    pub wheel_recovery_latency_ms: f32,
    pub locked_pulse_cycles: u32,
    pub total_pulse_cycles: u32,
}

/// Aggregated multi-scenario braking simulation result for a vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrakingSurfaceExperimentResult {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub category: String,
    pub straight_line_nominal: Vec<BrakingStraightLineResult>,
    pub straight_line_perturbed: Vec<BrakingStraightLineResult>,
    pub split_mu: Vec<BrakingSplitMuResult>,
    pub cornering_trail_braking: Vec<BrakingCorneringResult>,
    pub cadence_pumping: Vec<BrakingCadenceResult>,
}

/// Runs straight-line panic braking, optionally injecting an initial yaw perturbation.
pub fn run_braking_straight_line(
    config: &CarConfig,
    surface: SurfaceType,
    v0_kmh: f32,
    with_yaw_disturbance: bool,
    dt: f32,
) -> BrakingStraightLineResult {
    let v0_mps = v0_kmh / 3.6;
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    let mut peak_decel_g = 0.0f32;
    let mut max_sideslip_rad = 0.0f32;
    let mut max_yaw_rate = 0.0f32;
    let mut front_lockup_s = 0.0f32;
    let mut rear_lockup_s = 0.0f32;
    let mut abs_active_steps = 0u32;
    let mut total_steps = 0u32;

    runner.run_until(
        30.0,
        surface,
        |t, _car| {
            // Apply full service brake
            let mut controls = CarControls::full_brake();
            // In perturbed mode, introduce a short steering jolt (0.04 rad) during early braking
            if with_yaw_disturbance && t >= 0.05 && t <= 0.12 {
                controls.steer = 0.25;
            }
            controls
        },
        |t, car| {
            total_steps += 1;
            let decel_g = (-car.state().acceleration_local.x) / G_ACCEL;
            if decel_g > peak_decel_g {
                peak_decel_g = decel_g;
            }

            let sideslip = car.state().sideslip_angle.abs();
            if sideslip > max_sideslip_rad {
                max_sideslip_rad = sideslip;
            }

            let yaw_rate = car.state().angular_velocity.abs();
            if yaw_rate > max_yaw_rate {
                max_yaw_rate = yaw_rate;
            }

            if car.state().abs_active {
                abs_active_steps += 1;
            }

            // Injected angular velocity perturbation at onset if requested
            if with_yaw_disturbance && (t - 0.05).abs() < dt * 0.5 {
                // Introduce 0.15 rad/s yaw impulse
                // (simulating sudden road camber bump or steering twitch during brake hit)
            }

            let front_locked = car.state().wheels[0].slip_ratio.abs() >= 0.95
                || car.state().wheels[1].slip_ratio.abs() >= 0.95;
            if front_locked {
                front_lockup_s += dt;
            }

            let rear_locked = car.state().wheels[2].slip_ratio.abs() >= 0.95
                || car.state().wheels[3].slip_ratio.abs() >= 0.95;
            if rear_locked {
                rear_lockup_s += dt;
            }

            car.state().speed <= 0.05
        },
    );

    let stopping_distance_m = runner.car.state().position.x.max(0.1);
    let stopping_time_s = runner.time;
    let avg_decel_g = (v0_mps * v0_mps) / (2.0 * stopping_distance_m * G_ACCEL);
    let max_sideslip_deg = max_sideslip_rad.to_degrees();
    let max_yaw_rate_deg_s = max_yaw_rate.to_degrees();
    let lateral_displacement_m = runner.car.state().position.y.abs();
    let heading_deviation_deg = runner.car.state().angle.abs().to_degrees();
    let abs_active_pct = if total_steps > 0 {
        (abs_active_steps as f32 / total_steps as f32) * 100.0
    } else {
        0.0
    };

    let stability_rating = if max_sideslip_deg > 30.0 || heading_deviation_deg > 45.0 {
        BrakingStabilityRating::Spinout
    } else if max_sideslip_deg > 8.0 || heading_deviation_deg > 10.0 {
        BrakingStabilityRating::YawWander
    } else {
        BrakingStabilityRating::Stable
    };

    BrakingStraightLineResult {
        surface,
        v0_kmh,
        with_yaw_disturbance,
        stopping_distance_m,
        stopping_time_s,
        avg_decel_g,
        peak_decel_g,
        max_sideslip_deg,
        max_yaw_rate_deg_s,
        lateral_displacement_m,
        heading_deviation_deg,
        front_lockup_duration_s: front_lockup_s,
        rear_lockup_duration_s: rear_lockup_s,
        abs_active_pct,
        stability_rating,
    }
}

/// Runs Split-mu (asymmetric surface) emergency braking.
/// Left wheels ride on `high_mu`, Right wheels ride on `low_mu`.
pub fn run_braking_split_mu(
    config: &CarConfig,
    high_mu: SurfaceType,
    low_mu: SurfaceType,
    v0_kmh: f32,
    dt: f32,
) -> BrakingSplitMuResult {
    let v0_mps = v0_kmh / 3.6;
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    let surfaces = [high_mu, low_mu, high_mu, low_mu];

    let mut peak_yaw_rate = 0.0f32;
    let mut max_sideslip_rad = 0.0f32;

    runner.run_per_wheel_until(
        30.0,
        surfaces,
        |_t, _car| CarControls::full_brake(),
        |_t, car| {
            let yaw_rate = car.state().angular_velocity.abs();
            if yaw_rate > peak_yaw_rate {
                peak_yaw_rate = yaw_rate;
            }

            let sideslip = car.state().sideslip_angle.abs();
            if sideslip > max_sideslip_rad {
                max_sideslip_rad = sideslip;
            }

            car.state().speed <= 0.05
        },
    );

    let stopping_distance_m = runner.car.state().position.x.max(0.1);
    let stopping_time_s = runner.time;
    let avg_decel_g = (v0_mps * v0_mps) / (2.0 * stopping_distance_m * G_ACCEL);
    let lateral_lane_drift_m = runner.car.state().position.y.abs();
    let heading_deviation_deg = runner.car.state().angle.abs().to_degrees();
    let max_sideslip_deg = max_sideslip_rad.to_degrees();

    let status = if heading_deviation_deg > 45.0 || max_sideslip_deg > 35.0 {
        SplitMuStatus::SpunOut
    } else if lateral_lane_drift_m > 1.5 || heading_deviation_deg > 5.0 {
        SplitMuStatus::PullsGripSide
    } else {
        SplitMuStatus::TrackedStraight
    };

    BrakingSplitMuResult {
        high_mu_surface: high_mu,
        low_mu_surface: low_mu,
        v0_kmh,
        stopping_distance_m,
        stopping_time_s,
        avg_decel_g,
        peak_yaw_rate_deg_s: peak_yaw_rate.to_degrees(),
        max_sideslip_deg,
        lateral_lane_drift_m,
        heading_deviation_deg,
        status,
    }
}

/// Runs Cornering Trail-Braking: enters a steady curve and hits full brakes.
pub fn run_braking_in_turn(
    config: &CarConfig,
    surface: SurfaceType,
    radius_m: f32,
    v0_kmh: f32,
    dt: f32,
) -> BrakingCorneringResult {
    let v0_mps = v0_kmh / 3.6;
    // Start at (R, 0) pointing north (+y) with speed v0
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(Vec2::new(radius_m, 0.0), PI * 0.5, Vec2::new(0.0, v0_mps));

    let mut peak_yaw_rate = 0.0f32;
    let mut max_sideslip_rad = 0.0f32;
    let mut max_radial_divergence = 0.0f32;
    let initial_pos = Vec2::new(radius_m, 0.0);

    // Kinematic steer for circle
    let speed_factor = 1.0 + v0_mps * config.speed_sensitive_steer_factor;
    let turn_steer = -(config.wheelbase / (radius_m * config.max_steer_angle)) * speed_factor;

    runner.run_until(
        15.0,
        surface,
        |t, _car| {
            if t < 0.2 {
                // Settle into turn
                CarControls {
                    throttle: 0.1,
                    steer: turn_steer.clamp(-1.0, 1.0),
                    brake: 0.0,
                    handbrake: false,
                    reverse: false,
                }
            } else {
                // Threshold braking while maintaining steer command
                CarControls {
                    throttle: 0.0,
                    steer: turn_steer.clamp(-1.0, 1.0),
                    brake: 1.0,
                    handbrake: false,
                    reverse: false,
                }
            }
        },
        |t, car| {
            if t >= 0.2 {
                let yaw_rate = car.state().angular_velocity.abs();
                if yaw_rate > peak_yaw_rate {
                    peak_yaw_rate = yaw_rate;
                }

                let sideslip = car.state().sideslip_angle.abs();
                if sideslip > max_sideslip_rad {
                    max_sideslip_rad = sideslip;
                }

                let current_r = car.state().position.length();
                let radial_div = (current_r - radius_m).abs();
                if radial_div > max_radial_divergence {
                    max_radial_divergence = radial_div;
                }
            }

            car.state().speed <= 0.05
        },
    );

    let max_sideslip_deg = max_sideslip_rad.to_degrees();
    let final_pos = runner.car.state().position;
    let stopping_dist = (final_pos - initial_pos).length();

    let behavior = if max_sideslip_deg > 35.0 {
        CorneringBrakingBehavior::SnapOversteerSpin
    } else if max_radial_divergence > 6.0 {
        CorneringBrakingBehavior::UndersteerPlow
    } else {
        CorneringBrakingBehavior::CleanTrailBrake
    };

    BrakingCorneringResult {
        surface,
        v0_kmh,
        radius_m,
        peak_yaw_rate_deg_s: peak_yaw_rate.to_degrees(),
        max_sideslip_deg,
        stopping_distance_along_path_m: stopping_dist,
        radial_path_divergence_m: max_radial_divergence,
        behavior,
    }
}

/// Runs Cadence / Brake Pumping test: cycling brake pedal on and off.
pub fn run_braking_cadence(
    config: &CarConfig,
    surface: SurfaceType,
    v0_kmh: f32,
    dt: f32,
) -> BrakingCadenceResult {
    let v0_mps = v0_kmh / 3.6;
    let mut runner = SimulationRunner::new(config.clone(), dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    let cycle_period = 0.30f32; // 0.15s on, 0.15s off
    let mut total_cycles = 0u32;
    let mut locked_cycles = 0u32;
    let mut spinup_latencies = Vec::new();
    let mut last_cycle_index = 0u32;
    let mut release_time = 0.0f32;
    let mut waiting_spinup = false;

    runner.run_until(
        30.0,
        surface,
        |t, _car| {
            let phase = t % cycle_period;
            if phase < (cycle_period * 0.5) {
                CarControls::full_brake()
            } else {
                CarControls::default() // 0 throttle, 0 brake
            }
        },
        |t, car| {
            let cycle_idx = (t / cycle_period) as u32;
            let phase = t % cycle_period;

            if cycle_idx > last_cycle_index {
                last_cycle_index = cycle_idx;
                total_cycles += 1;
            }

            // When brake is released
            if phase >= (cycle_period * 0.5) && !waiting_spinup {
                waiting_spinup = true;
                release_time = t;
                // Check if any wheel was locked at release
                let was_locked = car.state().wheels.iter().any(|w| w.slip_ratio.abs() >= 0.90);
                if was_locked {
                    locked_cycles += 1;
                }
            }

            // Measure how long until wheel recovers grip (|slip| < 0.12)
            if waiting_spinup {
                let max_slip = car.state().wheels.iter().map(|w| w.slip_ratio.abs()).fold(0.0f32, f32::max);
                if max_slip < 0.12 {
                    spinup_latencies.push((t - release_time) * 1000.0);
                    waiting_spinup = false;
                }
            }

            car.state().speed <= 0.05
        },
    );

    let stopping_distance_m = runner.car.state().position.x.max(0.1);
    let stopping_time_s = runner.time;
    let avg_decel_g = (v0_mps * v0_mps) / (2.0 * stopping_distance_m * G_ACCEL);
    let avg_recovery_latency_ms = if !spinup_latencies.is_empty() {
        spinup_latencies.iter().sum::<f32>() / spinup_latencies.len() as f32
    } else {
        0.0
    };

    BrakingCadenceResult {
        surface,
        v0_kmh,
        stopping_distance_m,
        stopping_time_s,
        avg_decel_g,
        wheel_recovery_latency_ms: avg_recovery_latency_ms,
        locked_pulse_cycles: locked_cycles,
        total_pulse_cycles: total_cycles.max(1),
    }
}

/// Executes the full multi-scenario braking simulation battery for a vehicle.
pub fn run_braking_surface_battery(
    vehicle_id: impl Into<String>,
    vehicle_name: impl Into<String>,
    category: impl Into<String>,
    config: &CarConfig,
    v0_kmh: f32,
    dt: f32,
) -> BrakingSurfaceExperimentResult {
    let surfaces = SurfaceType::ALL;
    let mut straight_nominal = Vec::with_capacity(surfaces.len());
    let mut straight_perturbed = Vec::with_capacity(surfaces.len());
    let mut cornering = Vec::with_capacity(surfaces.len());
    let mut cadence = Vec::with_capacity(surfaces.len());

    for &s in &surfaces {
        straight_nominal.push(run_braking_straight_line(config, s, v0_kmh, false, dt));
        straight_perturbed.push(run_braking_straight_line(config, s, v0_kmh, true, dt));
        cornering.push(run_braking_in_turn(config, s, 40.0, 70.0f32.min(v0_kmh * 0.7), dt));
        cadence.push(run_braking_cadence(config, s, v0_kmh, dt));
    }

    // Split-mu configurations (Asphalt vs each reduced-mu surface)
    let split_targets = [
        SurfaceType::Concrete,
        SurfaceType::Gravel,
        SurfaceType::Mud,
        SurfaceType::Grass,
        SurfaceType::Snow,
        SurfaceType::Water,
        SurfaceType::Ice,
    ];
    let mut split_mu = Vec::with_capacity(split_targets.len());
    for &low_s in &split_targets {
        split_mu.push(run_braking_split_mu(config, SurfaceType::Asphalt, low_s, 100.0, dt));
    }

    BrakingSurfaceExperimentResult {
        vehicle_id: vehicle_id.into(),
        vehicle_name: vehicle_name.into(),
        category: category.into(),
        straight_line_nominal: straight_nominal,
        straight_line_perturbed: straight_perturbed,
        split_mu,
        cornering_trail_braking: cornering,
        cadence_pumping: cadence,
    }
}

// ============================================================================
// Reverse Simulation Battery: Straight-Line & Dynamic Step-Steer
// ============================================================================

/// Reverse steering stability rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReverseSteerStatus {
    Stable,
    OversteerSpin,
    UndersteerPlow,
}

/// Output metrics from Reverse Straight-Line Stability Simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseStraightLineResult {
    pub surface: SurfaceType,
    pub duration_s: f32,
    pub terminal_speed_kmh: f32,
    pub distance_traveled_m: f32,
    pub heading_deviation_deg: f32,
    pub lateral_drift_m: f32,
    pub max_yaw_rate_deg_s: f32,
    pub stable: bool,
}

/// Output metrics from Reverse Step-Steer Bidirectional Dynamic Test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseStepSteerResult {
    pub surface: SurfaceType,
    pub peak_right_yaw_rate_deg_s: f32,
    pub peak_left_yaw_rate_deg_s: f32,
    pub yaw_asymmetry_pct: f32,
    pub reversal_latency_ms: f32,
    pub post_release_residual_yaw_deg_s: f32,
    pub status: ReverseSteerStatus,
}

/// Aggregated reverse simulation benchmark result for a vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseExperimentResult {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub category: String,
    pub straight_line: ReverseStraightLineResult,
    pub step_steer: ReverseStepSteerResult,
}

/// Runs straight-line reverse acceleration from rest to verify neutral heading stability.
pub fn run_reverse_straight_line(
    config: &CarConfig,
    surface: SurfaceType,
    duration_s: f32,
    dt: f32,
) -> ReverseStraightLineResult {
    let mut runner = SimulationRunner::new(config.clone(), dt);
    let mut max_yaw_rate = 0.0f32;

    runner.run_for(duration_s, surface, |_t, _car| {
        let mut ctrl = CarControls::accelerate();
        ctrl.reverse = true;
        ctrl
    });

    for pt in &runner.telemetry {
        let yaw = pt.yaw_rate_rad_s.to_degrees().abs();
        if yaw > max_yaw_rate {
            max_yaw_rate = yaw;
        }
    }

    let final_state = runner.car.state();
    let terminal_speed_kmh = final_state.speed * 3.6;
    let distance_traveled_m = final_state.position.length();
    let heading_deviation_deg = normalize_angle(final_state.angle).to_degrees().abs();
    let lateral_drift_m = final_state.position.y.abs();
    let stable = heading_deviation_deg < 0.1 && lateral_drift_m < 0.05 && max_yaw_rate < 0.5;

    ReverseStraightLineResult {
        surface,
        duration_s,
        terminal_speed_kmh,
        distance_traveled_m,
        heading_deviation_deg,
        lateral_drift_m,
        max_yaw_rate_deg_s: max_yaw_rate,
        stable,
    }
}

/// Runs bidirectional reverse step-steer test:
/// - Evaluates peak right vs peak left steering from identical conditions to test symmetry.
/// - Evaluates dynamic steering reversal latency (switching right -> left) and post-release yaw decay.
pub fn run_reverse_step_steer(
    config: &CarConfig,
    surface: SurfaceType,
    dt: f32,
) -> ReverseStepSteerResult {
    // 1. Right turn isolated peak
    let mut runner_r = SimulationRunner::new(config.clone(), dt);
    let mut peak_right_yaw_deg_s = 0.0f32;
    runner_r.run_for(1.5, surface, |_t, car| {
        let yaw = car.state().angular_velocity.to_degrees().abs();
        if yaw > peak_right_yaw_deg_s {
            peak_right_yaw_deg_s = yaw;
        }
        let mut ctrl = CarControls::accelerate();
        ctrl.reverse = true;
        ctrl.steer = 0.5;
        ctrl
    });

    // 2. Left turn isolated peak
    let mut runner_l = SimulationRunner::new(config.clone(), dt);
    let mut peak_left_yaw_deg_s = 0.0f32;
    runner_l.run_for(1.5, surface, |_t, car| {
        let yaw = car.state().angular_velocity.to_degrees().abs();
        if yaw > peak_left_yaw_deg_s {
            peak_left_yaw_deg_s = yaw;
        }
        let mut ctrl = CarControls::accelerate();
        ctrl.reverse = true;
        ctrl.steer = -0.5;
        ctrl
    });

    let max_peak = peak_right_yaw_deg_s.max(peak_left_yaw_deg_s);
    let yaw_asymmetry_pct = if max_peak > 1e-3 {
        ((peak_right_yaw_deg_s - peak_left_yaw_deg_s).abs() / max_peak) * 100.0
    } else {
        0.0
    };

    // 3. Dynamic reversal & stabilization test:
    // Phase A (0.0..1.5s): Steer Right (+0.5)
    // Phase B (1.5..3.0s): Reverse steer to Left (-0.5)
    // Phase C (3.0..4.5s): Release steer to Center (0.0)
    let mut runner_rev = SimulationRunner::new(config.clone(), dt);
    let mut zero_cross_time = None;
    let mut max_sideslip_deg = 0.0f32;
    let mut initial_yaw_sign = 0.0f32;

    runner_rev.run_for(4.5, surface, |t, car| {
        let yaw_deg_s = car.state().angular_velocity.to_degrees();
        let sideslip_deg = car.state().sideslip_angle.to_degrees().abs();
        if sideslip_deg > max_sideslip_deg {
            max_sideslip_deg = sideslip_deg;
        }

        let mut ctrl = CarControls::accelerate();
        ctrl.reverse = true;

        if t < 1.5 {
            ctrl.steer = 0.5;
            if yaw_deg_s.abs() > 1.0 && initial_yaw_sign == 0.0 {
                initial_yaw_sign = yaw_deg_s.signum();
            }
        } else if t < 3.0 {
            ctrl.steer = -0.5;
            if zero_cross_time.is_none()
                && initial_yaw_sign != 0.0
                && yaw_deg_s.signum() != initial_yaw_sign
                && yaw_deg_s.abs() > 0.5
            {
                zero_cross_time = Some(t);
            }
        } else {
            ctrl.steer = 0.0;
        }
        ctrl
    });

    let reversal_latency_ms = if let Some(zct) = zero_cross_time {
        ((zct - 1.5) * 1000.0).max(0.0)
    } else {
        1500.0
    };

    let post_release_residual_yaw_deg_s = runner_rev.car.state().angular_velocity.to_degrees().abs();

    let status = if max_sideslip_deg > 45.0 || post_release_residual_yaw_deg_s > 45.0 {
        ReverseSteerStatus::OversteerSpin
    } else if max_peak < 5.0 {
        ReverseSteerStatus::UndersteerPlow
    } else {
        ReverseSteerStatus::Stable
    };

    ReverseStepSteerResult {
        surface,
        peak_right_yaw_rate_deg_s: peak_right_yaw_deg_s,
        peak_left_yaw_rate_deg_s: peak_left_yaw_deg_s,
        yaw_asymmetry_pct,
        reversal_latency_ms,
        post_release_residual_yaw_deg_s,
        status,
    }
}

/// Executes the automated reverse simulation battery across a fleet of vehicles.
pub fn run_reverse_simulation_battery(
    vehicles: &[(&str, &str, &str, &CarConfig)],
    surface: SurfaceType,
    dt: f32,
) -> Vec<ReverseExperimentResult> {
    vehicles
        .iter()
        .map(|(id, name, cat, config)| {
            let straight_line = run_reverse_straight_line(config, surface, 3.0, dt);
            let step_steer = run_reverse_step_steer(config, surface, dt);
            ReverseExperimentResult {
                vehicle_id: (*id).to_string(),
                vehicle_name: (*name).to_string(),
                category: (*cat).to_string(),
                straight_line,
                step_steer,
            }
        })
        .collect()
}

