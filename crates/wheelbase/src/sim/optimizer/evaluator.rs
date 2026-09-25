use crate::car::{Car, CarControls};
use crate::config::{CarConfig, DifferentialType};
use crate::surface::SurfaceType;
use super::cma_es::ParameterBound;
use super::constraints::{DrivetrainConstraint, DynamicMetrics};
use super::targets::CalibrationTarget;

/// Evaluator that maps normalized parameter vectors to `CarConfig`, executes the headless simulation battery,
/// and computes the objective cost and constraint penalties (Spec 035).
pub struct OptimizationEvaluator {
    pub base_config: CarConfig,
    pub bounds: Vec<ParameterBound>,
    pub constraint: DrivetrainConstraint,
    pub target: CalibrationTarget,
    pub dt: f32,
    pub penalty_stiffness: f32,
}

impl OptimizationEvaluator {
    pub fn new(
        base_config: CarConfig,
        bounds: Vec<ParameterBound>,
        constraint: DrivetrainConstraint,
        target: CalibrationTarget,
        dt: f32,
        penalty_stiffness: f32,
    ) -> Self {
        Self {
            base_config,
            bounds,
            constraint,
            target,
            dt,
            penalty_stiffness,
        }
    }

    /// Converts normalized [0, 1] parameter vector into a concrete `CarConfig`.
    pub fn decode_config(&self, normalized_params: &[f32]) -> CarConfig {
        let mut cfg = self.base_config;

        for (i, bound) in self.bounds.iter().enumerate() {
            let val = bound.from_normalized(normalized_params[i]);
            match bound.name.as_str() {
                "caster_jacking_factor" => cfg.caster_jacking_factor = val,
                "speed_sensitive_steer_factor" => cfg.speed_sensitive_steer_factor = val,
                "angular_damping" => cfg.angular_damping = val,
                "brake_bias" => cfg.brake_bias = val,
                "drive_bias" => cfg.drive_bias = val,
                "max_steer_angle" => cfg.max_steer_angle = val,
                "steer_speed" => cfg.steer_speed = val,
                "steer_return_speed" => cfg.steer_return_speed = val,
                "weight_transfer_lateral" => cfg.weight_transfer_lateral = val,
                "weight_transfer_longitudinal" => cfg.weight_transfer_longitudinal = val,
                "downforce_coefficient" => cfg.downforce_coefficient = val,
                "air_drag_coefficient" => cfg.air_drag_coefficient = val,
                "power_lock" => {
                    if let DifferentialType::LimitedSlip { coast_lock, preload_nm, .. } = cfg.rear_differential {
                        cfg.rear_differential = DifferentialType::LimitedSlip {
                            power_lock: val,
                            coast_lock,
                            preload_nm,
                        };
                    }
                }
                "coast_lock" => {
                    if let DifferentialType::LimitedSlip { power_lock, preload_nm, .. } = cfg.rear_differential {
                        cfg.rear_differential = DifferentialType::LimitedSlip {
                            power_lock,
                            coast_lock: val,
                            preload_nm,
                        };
                    }
                }
                "preload_nm" => {
                    if let DifferentialType::LimitedSlip { power_lock, coast_lock, .. } = cfg.rear_differential {
                        cfg.rear_differential = DifferentialType::LimitedSlip {
                            power_lock,
                            coast_lock,
                            preload_nm: val,
                        };
                    }
                }
                _ => {}
            }
        }

        // Project hard constraints (equality manifolds, locked spools, power/coast deltas)
        self.constraint.project_hard_constraints(&mut cfg);

        cfg
    }

    /// Evaluates candidate parameters by executing dynamic simulations and returns composite loss.
    pub fn evaluate(&self, normalized_params: &[f32]) -> (f32, DynamicMetrics, Vec<(String, f32)>) {
        let config = self.decode_config(normalized_params);
        let dt = self.dt;

        let mut metrics = DynamicMetrics::default();

        // --------------------------------------------------------------------
        // Test 1: Low-Speed Turning Circle & Inside Rear Unloading
        // --------------------------------------------------------------------
        let mut car_turn = Car::new(config);
        let ctrl_turn = CarControls::new(0.20, 1.0, 0.0, false);
        let static_rear_load = (config.mass * 9.81 * (config.cg_to_front / config.wheelbase)) * 0.5;

        let mut min_inside_load = static_rear_load;
        for _ in 0..120 {
            car_turn.step(&ctrl_turn, SurfaceType::Asphalt, dt);
            // In a right turn, wheel 3 is inside; in a left turn, wheel 2 is inside
            let inside_load = car_turn.state.wheels[2].normal_load.min(car_turn.state.wheels[3].normal_load);
            if inside_load < min_inside_load {
                min_inside_load = inside_load;
            }
        }
        let speed_low = car_turn.state.local_velocity.x.abs().max(1.0);
        let yaw_rate_low = car_turn.state.angular_velocity.abs();
        let turning_diameter = if yaw_rate_low > 1e-2 {
            2.0 * (speed_low / yaw_rate_low)
        } else {
            50.0
        };
        metrics.turning_diameter_m = turning_diameter;
        metrics.inside_rear_unloading_ratio = ((static_rear_load - min_inside_load) / static_rear_load.max(1.0)).clamp(0.0, 1.0);

        // --------------------------------------------------------------------
        // Test 2: Apex Acceleration & Differential Slip Loss
        // --------------------------------------------------------------------
        let mut car_accel = Car::new(config);
        car_accel.state.local_velocity.x = 40.0 / 3.6; // 40 km/h entry
        let mut slip_integral = 0.0f32;
        let mut t_exit = 0.0f32;
        let ctrl_accel = CarControls::new(1.0, 0.20, 0.0, false);

        for step in 0..180 {
            car_accel.step(&ctrl_accel, SurfaceType::Asphalt, dt);
            let omega_rl = car_accel.state.wheel_assemblies[2].angular_velocity;
            let omega_rr = car_accel.state.wheel_assemblies[3].angular_velocity;
            slip_integral += (omega_rl - omega_rr).abs() * dt;

            if car_accel.state.local_velocity.x >= (100.0 / 3.6) && t_exit == 0.0 {
                t_exit = (step as f32) * dt;
            }
        }
        metrics.axle_slip_differential_integral = slip_integral;
        let exit_time_score = if t_exit > 0.0 { t_exit } else { 5.0 };

        // --------------------------------------------------------------------
        // Test 3: Trail-Braking Deceleration & Stability
        // --------------------------------------------------------------------
        let mut car_brake = Car::new(config);
        car_brake.state.local_velocity.x = 120.0 / 3.6; // 120 km/h approach
        let ctrl_trail = CarControls::new(0.0, 0.35, 1.0, false); // Steering + Hard Brake
        let mut peak_yaw_accel = 0.0f32;
        let mut max_sideslip = 0.0f32;
        let mut prev_omega = 0.0f32;

        for _ in 0..120 {
            car_brake.step(&ctrl_trail, SurfaceType::Asphalt, dt);
            let omega = car_brake.state.angular_velocity;
            let yaw_accel = (omega - prev_omega).abs() / dt;
            prev_omega = omega;
            if yaw_accel > peak_yaw_accel {
                peak_yaw_accel = yaw_accel;
            }
            let sideslip = car_brake.state.sideslip_angle.to_degrees().abs();
            if sideslip > max_sideslip {
                max_sideslip = sideslip;
            }
        }
        metrics.peak_yaw_acceleration_rad_s2 = peak_yaw_accel;
        metrics.max_sideslip_deg = max_sideslip;

        // --------------------------------------------------------------------
        // Test 4: Steady-State Skidpad Peak Lateral G
        // --------------------------------------------------------------------
        let mut car_skidpad = Car::new(config);
        car_skidpad.state.local_velocity.x = 60.0 / 3.6;
        let ctrl_skid = CarControls::new(0.60, 0.40, 0.0, false);
        let mut peak_ay = 0.0f32;
        for _ in 0..150 {
            car_skidpad.step(&ctrl_skid, SurfaceType::Asphalt, dt);
            let ay = (car_skidpad.state.local_velocity.x * car_skidpad.state.angular_velocity).abs() / 9.81;
            if ay > peak_ay {
                peak_ay = ay;
            }
        }

        // --------------------------------------------------------------------
        // Composite Objective Cost Calculation
        // --------------------------------------------------------------------
        let mut base_cost = 0.0f32;

        // Turning circle error
        if let Some(target_d) = self.target.target_turning_diameter_m {
            let d_err = (metrics.turning_diameter_m - target_d).abs();
            base_cost += d_err * 2.0;
        }

        // Lateral grip error
        let lat_g_err = (peak_ay - self.target.target_peak_lat_g).abs();
        base_cost += lat_g_err * 5.0;

        // Acceleration exit time
        base_cost += exit_time_score * 3.0;

        // Differential slip loss penalty
        base_cost += metrics.axle_slip_differential_integral * 0.15;

        // Dynamic constraint evaluation (penalty method)
        let (penalty, violations) = self
            .constraint
            .evaluate_dynamic_penalties(&metrics, self.penalty_stiffness);

        let total_cost = base_cost + penalty;

        (total_cost, metrics, violations)
    }
}
