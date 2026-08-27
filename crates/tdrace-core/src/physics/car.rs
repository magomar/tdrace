use std::f32::consts::PI;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::config::CarConfig;
use super::surface::SurfaceType;
use super::tire::{
    compute_skid_telemetry, pacejka_lateral_force, solve_combined_slip_forces, WheelId,
    WheelTelemetry,
};

/// Normalizes an angle in radians to (-PI, PI].
#[inline]
pub fn normalize_angle(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle <= -PI {
        angle += 2.0 * PI;
    }
    angle
}

/// Driver control inputs applied at each physics step.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CarControls {
    /// Throttle input [0.0 = idle, 1.0 = full gas].
    pub throttle: f32,
    /// Steering input [-1.0 = full left, 0.0 = center, +1.0 = full right].
    pub steer: f32,
    /// Service brake input [0.0 = release, 1.0 = full brake].
    pub brake: f32,
    /// Handbrake flag (locks rear wheels and initiates drifts).
    pub handbrake: bool,
    /// Reverse gear flag (applies reverse torque when active).
    pub reverse: bool,
}

impl Default for CarControls {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            steer: 0.0,
            brake: 0.0,
            handbrake: false,
            reverse: false,
        }
    }
}

impl CarControls {
    pub const fn new(throttle: f32, steer: f32, brake: f32, handbrake: bool) -> Self {
        Self {
            throttle,
            steer,
            brake,
            handbrake,
            reverse: false,
        }
    }

    /// Full forward throttle helper.
    pub const fn accelerate() -> Self {
        Self {
            throttle: 1.0,
            steer: 0.0,
            brake: 0.0,
            handbrake: false,
            reverse: false,
        }
    }

    /// Full service brake helper.
    pub const fn full_brake() -> Self {
        Self {
            throttle: 0.0,
            steer: 0.0,
            brake: 1.0,
            handbrake: false,
            reverse: false,
        }
    }

    /// Handbrake turn helper.
    pub const fn handbrake_turn(steer: f32) -> Self {
        Self {
            throttle: 0.5,
            steer,
            brake: 0.0,
            handbrake: true,
            reverse: false,
        }
    }

    /// Clamps input values to their valid operating ranges and sanitizes NaN / Inf floats to 0.0.
    #[inline]
    pub fn clamped(&self) -> Self {
        let throttle = if self.throttle.is_finite() {
            self.throttle.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let steer = if self.steer.is_finite() {
            self.steer.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let brake = if self.brake.is_finite() {
            self.brake.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            throttle,
            steer,
            brake,
            handbrake: self.handbrake,
            reverse: self.reverse,
        }
    }
}

/// Complete serializable state of the vehicle at any instant in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarState {
    /// World position (x, y) in meters.
    pub position: Vec2,
    /// Linear velocity vector in world coordinates (m/s).
    pub velocity: Vec2,
    /// Yaw orientation angle in radians (0 = pointing along +X, PI/2 = pointing along +Y).
    pub angle: f32,
    /// Angular yaw velocity (yaw rate) in radians/second.
    pub angular_velocity: f32,
    /// Current front wheel steering angle relative to chassis in radians.
    pub steer_angle: f32,
    /// Filtered vehicle body acceleration in local frame (x = forward, y = right) for weight transfer.
    pub acceleration_local: Vec2,
    /// Detailed telemetry for all 4 wheels.
    pub wheels: [WheelTelemetry; 4],
    /// Scalar road speed in meters/second.
    pub speed: f32,
    /// Velocity decomposed into local chassis coordinates (forward, right).
    pub local_velocity: Vec2,
    /// Overall body sideslip angle (drift angle) in radians.
    pub sideslip_angle: f32,
    /// Whether the car is actively in a controlled drift.
    pub is_drifting: bool,
    /// Cumulative drift score accumulated during slide.
    pub drift_score: f32,
    /// Whether Traction Control System (TCS) is actively intervening / cutting engine torque.
    pub tcs_active: bool,
    /// Whether Electronic Stability Control (ESC) is actively applying stabilizing yaw torque.
    pub esc_active: bool,
    /// Whether Anti-lock Braking System (ABS) is actively modulating brake force.
    pub abs_active: bool,
    /// Elevation / vertical altitude above ground in meters (z >= 0.0).
    pub elevation: f32,
    /// Vertical velocity in m/s (positive = ascending, negative = falling).
    pub vertical_velocity: f32,
    /// Whether the car is currently airborne (off the ground).
    pub is_airborne: bool,
    /// Time spent in the air during the current jump in seconds.
    pub air_time: f32,
    /// Cumulative count of jumps completed.
    pub jump_count: u32,
    /// Flag indicating the vehicle touched down on the ground during this physics tick.
    pub just_landed: bool,
}

impl Default for CarState {
    fn default() -> Self {
        let mut wheels = [WheelTelemetry::default(); 4];
        for (i, w) in wheels.iter_mut().enumerate() {
            w.id = WheelId::ALL[i];
        }
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            angle: 0.0,
            angular_velocity: 0.0,
            steer_angle: 0.0,
            acceleration_local: Vec2::ZERO,
            wheels,
            speed: 0.0,
            local_velocity: Vec2::ZERO,
            sideslip_angle: 0.0,
            is_drifting: false,
            drift_score: 0.0,
            tcs_active: false,
            esc_active: false,
            abs_active: false,
            elevation: 0.0,
            vertical_velocity: 0.0,
            is_airborne: false,
            air_time: 0.0,
            jump_count: 0,
            just_landed: false,
        }
    }
}

/// 4-Wheel Top-Down Arcade Vehicle Physics Model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Car {
    pub config: CarConfig,
    pub state: CarState,
}

impl Car {
    /// Creates a new car instance with the given configuration at the origin.
    pub fn new(config: CarConfig) -> Self {
        Self {
            config,
            state: CarState::default(),
        }
    }

    /// Sets the initial pose (position and yaw angle).
    pub fn with_pose(mut self, position: Vec2, angle: f32) -> Self {
        self.state.position = position;
        self.state.angle = normalize_angle(angle);
        self
    }

    /// Gets an immutable reference to the car's current state.
    #[inline]
    pub fn state(&self) -> &CarState {
        &self.state
    }

    /// Gets a mutable reference to the car's state.
    #[inline]
    pub fn state_mut(&mut self) -> &mut CarState {
        &mut self.state
    }

    /// Restores the complete car state (useful for rewinds, save states, networking).
    #[inline]
    pub fn set_state(&mut self, state: CarState) {
        self.state = state;
    }

    /// Gets an immutable reference to the car configuration.
    #[inline]
    pub fn config(&self) -> &CarConfig {
        &self.config
    }

    /// Updates the car configuration.
    #[inline]
    pub fn set_config(&mut self, config: CarConfig) {
        self.config = config;
    }

    /// Returns the current forward unit vector in world space.
    #[inline]
    pub fn forward_vector(&self) -> Vec2 {
        Vec2::new(self.state.angle.cos(), self.state.angle.sin())
    }

    /// Returns the current right unit vector in world space.
    #[inline]
    pub fn right_vector(&self) -> Vec2 {
        Vec2::new(self.state.angle.sin(), -self.state.angle.cos())
    }

    /// Current speed in km/h.
    #[inline]
    pub fn speed_kmh(&self) -> f32 {
        self.state.speed * 3.6
    }

    /// Current speed in mph.
    #[inline]
    pub fn speed_mph(&self) -> f32 {
        self.state.speed * 2.23694
    }

    /// Computes world positions of all 4 wheels.
    pub fn wheel_positions_world(&self) -> [Vec2; 4] {
        let fwd = self.forward_vector();
        let right = self.right_vector();
        let lf = self.config.cg_to_front;
        let lr = self.config.cg_to_rear;
        let half_w = self.config.track_width * 0.5;

        [
            self.state.position + fwd * lf - right * half_w, // FL
            self.state.position + fwd * lf + right * half_w, // FR
            self.state.position - fwd * lr - right * half_w, // RL
            self.state.position - fwd * lr + right * half_w, // RR
        ]
    }

    /// Calculates Ackermann individual steering angles for front wheels.
    ///
    /// Inner wheel in a turn steers more sharply than outer wheel to eliminate scrubbing.
    #[inline]
    pub fn compute_ackermann_angles(&self, steer_angle: f32) -> (f32, f32) {
        if steer_angle.abs() < 1e-4 {
            return (0.0, 0.0);
        }

        let l = self.config.wheelbase;
        let half_w = self.config.track_width * 0.5;
        let abs_steer = steer_angle.abs();
        let r_center = l / abs_steer.tan();

        if steer_angle < 0.0 {
            // Turning right (steer_angle < 0, clockwise): FR is inner, FL is outer
            let r_inner = (r_center - half_w).max(0.2);
            let r_outer = r_center + half_w;
            let delta_fr = -(l / r_inner).atan();
            let delta_fl = -(l / r_outer).atan();
            (delta_fl, delta_fr)
        } else {
            // Turning left (steer_angle > 0, counter-clockwise): FL is inner, FR is outer
            let r_inner = (r_center - half_w).max(0.2);
            let r_outer = r_center + half_w;
            let delta_fl = (l / r_inner).atan();
            let delta_fr = (l / r_outer).atan();
            (delta_fl, delta_fr)
        }
    }

    /// Steps the physics simulation forward by a fixed timestep `dt` over a uniform surface.
    #[inline]
    pub fn step(&mut self, controls: &CarControls, surface: SurfaceType, dt: f32) {
        self.step_per_wheel(controls, [surface; 4], dt);
    }

    /// Steps the physics simulation forward with independent surface types per wheel.
    pub fn step_per_wheel(
        &mut self,
        controls: &CarControls,
        surfaces: [SurfaceType; 4],
        dt: f32,
    ) {
        // 0. Update vertical elevation dynamics
        self.state.just_landed = false;
        if self.state.elevation > 0.0 || self.state.vertical_velocity.abs() > 1e-4 {
            let gravity_z = 13.5f32; // snappy arcade gravity
            self.state.vertical_velocity -= gravity_z * dt;
            self.state.elevation += self.state.vertical_velocity * dt;
            if self.state.elevation <= 0.0 {
                self.state.elevation = 0.0;
                self.state.vertical_velocity = 0.0;
                self.state.is_airborne = false;
                self.state.air_time = 0.0;
                self.state.just_landed = true;
            } else {
                self.state.is_airborne = true;
                self.state.air_time += dt;
            }
        } else {
            self.state.elevation = 0.0;
            self.state.vertical_velocity = 0.0;
            self.state.is_airborne = false;
            self.state.air_time = 0.0;
        }

        let clamped_ctrl = controls.clamped();
        let fwd = self.forward_vector();
        let right = self.right_vector();

        // Decompose velocity into vehicle chassis frame
        let v_long = self.state.velocity.dot(fwd);
        let v_lat = self.state.velocity.dot(right);
        self.state.local_velocity = Vec2::new(v_long, v_lat);
        self.state.speed = self.state.velocity.length();

        // 1. Steering dynamics with speed-sensitive limit and counter-steer assist
        // steer > 0 is steering right (clockwise, -steer_angle in Cartesian coords)
        // steer < 0 is steering left (counter-clockwise, +steer_angle in Cartesian coords)
        let speed_factor = 1.0 + self.state.speed * self.config.speed_sensitive_steer_factor;
        let mut target_steer = (-clamped_ctrl.steer * self.config.max_steer_angle) / speed_factor;

        // Check if player is counter-steering against a drift (opposite to lateral velocity / yaw)
        let is_counter_steering = (clamped_ctrl.steer * v_lat) < -0.05;

        // Counter-steer / self-aligning drift recovery assist
        if self.config.assists.counter_steer_assist_enabled
            && !clamped_ctrl.handbrake
            && self.state.speed > 2.0
            && self.state.sideslip_angle.abs() > 0.04
        {
            let align_angle = -self.state.sideslip_angle * self.config.assists.counter_steer_assist_strength;
            if clamped_ctrl.steer.abs() < 0.35 {
                let blend = 1.0 - (clamped_ctrl.steer.abs() / 0.35);
                target_steer += align_angle * blend;
            }
        }

        let steer_rate = if clamped_ctrl.steer.abs() < 1e-3 {
            self.config.steer_return_speed
        } else if is_counter_steering {
            self.config.steer_speed * self.config.counter_steer_assist
        } else {
            self.config.steer_speed
        };

        let steer_delta = target_steer - self.state.steer_angle;
        let max_steer_change = steer_rate * dt;
        self.state.steer_angle += steer_delta.clamp(-max_steer_change, max_steer_change);

        // 2. Wheel positions and Ackermann angles
        let (steer_fl, steer_fr) = self.compute_ackermann_angles(self.state.steer_angle);
        let wheel_steer_angles = [steer_fl, steer_fr, 0.0, 0.0];

        let lf = self.config.cg_to_front;
        let lr = self.config.cg_to_rear;
        let half_w = self.config.track_width * 0.5;
        let wheelbase = self.config.wheelbase;

        // Local offsets: (x_forward, y_right)
        // FL = (lf, -half_w) [left]
        // FR = (lf, +half_w) [right]
        // RL = (-lr, -half_w) [left]
        // RR = (-lr, +half_w) [right]
        let wheel_local_offsets = [
            Vec2::new(lf, -half_w), // FL
            Vec2::new(lf, half_w),  // FR
            Vec2::new(-lr, -half_w), // RL
            Vec2::new(-lr, half_w),  // RR
        ];

        // 3. Dynamic Weight Transfer Calculation
        let g = 9.81;
        let total_weight = self.config.mass * g;
        let static_front_load = total_weight * (lr / wheelbase);
        let static_rear_load = total_weight * (lf / wheelbase);

        let a_long = self.state.acceleration_local.x;
        let a_lat = self.state.acceleration_local.y;

        // Acceleration squat (a_long > 0): front unloads, rear loads
        let delta_fz_long = (self.config.mass * a_long * (self.config.cg_height / wheelbase))
            * self.config.weight_transfer_longitudinal;

        // Cornering roll (a_lat > 0, turning right): right wheels unload (-), left wheels load (+)
        let delta_fz_lat_f = (self.config.mass * a_lat * (self.config.cg_height / self.config.track_width) * (lr / wheelbase))
            * self.config.weight_transfer_lateral;

        let delta_fz_lat_r = (self.config.mass * a_lat * (self.config.cg_height / self.config.track_width) * (lf / wheelbase))
            * self.config.weight_transfer_lateral;

        let min_load_f = static_front_load * 0.05 * 0.5;
        let min_load_r = static_rear_load * 0.05 * 0.5;

        // Aerodynamic downforce scaling with speed squared
        let speed_sq = self.state.speed * self.state.speed;
        let total_downforce = self.config.downforce_coefficient * speed_sq;
        let downforce_front = total_downforce * (lr / wheelbase) * 0.5;
        let downforce_rear = total_downforce * (lf / wheelbase) * 0.5;

        // Ground contact scaling when airborne
        let ground_contact = if self.state.elevation > 0.0 {
            (1.0 - (self.state.elevation / 0.35)).clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Wheel 0 = FL (left), Wheel 1 = FR (right), Wheel 2 = RL (left), Wheel 3 = RR (right)
        let normal_loads = [
            (((static_front_load - delta_fz_long) * 0.5 + delta_fz_lat_f * 0.5 + downforce_front).max(min_load_f)) * ground_contact, // FL (left)
            (((static_front_load - delta_fz_long) * 0.5 - delta_fz_lat_f * 0.5 + downforce_front).max(min_load_f)) * ground_contact, // FR (right)
            (((static_rear_load + delta_fz_long) * 0.5 + delta_fz_lat_r * 0.5 + downforce_rear).max(min_load_r)) * ground_contact,  // RL (left)
            (((static_rear_load + delta_fz_long) * 0.5 - delta_fz_lat_r * 0.5 + downforce_rear).max(min_load_r)) * ground_contact,  // RR (right)
        ];

        // 4. Force calculation per wheel
        let mut total_wheel_force_world = Vec2::ZERO;
        let mut total_wheel_torque = 0.0;

        let omega = self.state.angular_velocity;

        // Drive / Brake torque requests with top-speed governor and TCS
        let top_speed = self.config.top_speed_mps;
        let speed_ratio = v_long / top_speed;
        let engine_taper = if speed_ratio < 0.90 {
            1.0
        } else {
            (1.0 - (speed_ratio - 0.90) / 0.10).clamp(0.0, 1.0)
        };

        let mut tcs_active = false;
        let mut drive_torque_multiplier = 1.0f32;

        if self.config.assists.tcs_enabled
            && clamped_ctrl.throttle > 0.0
            && !clamped_ctrl.reverse
            && !(self.config.assists.handbrake_bypass && clamped_ctrl.handbrake)
        {
            let rear_slip_lat = self.state.wheels[2].slip_angle.abs().max(self.state.wheels[3].slip_angle.abs());
            let is_cornering = self.state.steer_angle.abs() > 0.02 || self.state.sideslip_angle.abs() > 0.03 || rear_slip_lat > 0.08;

            let thresh = self.config.assists.tcs_slip_threshold;
            if is_cornering && rear_slip_lat > thresh {
                let excess_lat = (rear_slip_lat - thresh).max(0.0) / thresh;
                let cut = (excess_lat * self.config.assists.tcs_strength).clamp(0.0, 0.75);
                drive_torque_multiplier = 1.0 - cut;
                tcs_active = true;
            }
        }
        self.state.tcs_active = tcs_active;

        let total_drive_force = if clamped_ctrl.reverse {
            -clamped_ctrl.throttle * self.config.max_reverse_force
        } else if clamped_ctrl.throttle > 0.0 {
            clamped_ctrl.throttle * self.config.max_engine_force * engine_taper * drive_torque_multiplier
        } else if self.config.engine_braking_coefficient > 0.0 && v_long.abs() > 0.05 {
            // Engine braking opposes motion on throttle release, creating realistic coast-down and turn-in pitch
            -self.config.engine_braking_coefficient * total_weight * (v_long / 1.5).tanh()
        } else {
            0.0
        };

        // Brake force opposing motion
        let total_brake_force = clamped_ctrl.brake * self.config.max_brake_force;
        let mut abs_active = false;

        for i in 0..4 {
            let wheel_id = WheelId::ALL[i];
            let offset_local = wheel_local_offsets[i];
            let offset_world = fwd * offset_local.x + right * offset_local.y;
            let wheel_pos_world = self.state.position + offset_world;

            // Contact patch world velocity = V_cg + omega x r
            let v_rot = Vec2::new(-omega * offset_world.y, omega * offset_world.x);
            let wheel_v_world = self.state.velocity + v_rot;

            // Wheel orientation
            let wheel_angle_world = self.state.angle + wheel_steer_angles[i];
            let wheel_fwd = Vec2::new(wheel_angle_world.cos(), wheel_angle_world.sin());
            let wheel_right = Vec2::new(wheel_angle_world.sin(), -wheel_angle_world.cos());

            let w_v_long = wheel_v_world.dot(wheel_fwd);
            let w_v_lat = wheel_v_world.dot(wheel_right);

            // Slip angle: angle between wheel direction and velocity vector
            let slip_angle = -w_v_lat.atan2(w_v_long.abs().max(0.2));

            // Surface properties
            let surf = surfaces[i];
            let mu = surf.friction_coefficient();
            let fz = normal_loads[i];
            let max_friction = mu * fz;

            // Longitudinal demand: Drive + Brake + Handbrake + Rolling Resistance
            let drive_share = if wheel_id.is_front() {
                self.config.drive_bias * 0.5
            } else {
                (1.0 - self.config.drive_bias) * 0.5
            };

            let brake_share = if wheel_id.is_front() {
                self.config.brake_bias * 0.5
            } else {
                (1.0 - self.config.brake_bias) * 0.5
            };

            let mut fx_demand = total_drive_force * drive_share;

            // Service braking opposes wheel rolling direction
            if total_brake_force > 0.0 {
                let brake_dir = if w_v_long.abs() > 0.1 {
                    -w_v_long.signum()
                } else {
                    -v_long.signum()
                };
                let mut wheel_brake_force = total_brake_force * brake_share;

                // ABS (Anti-lock Braking System): modulate brake force to preserve steering authority and avoid lockup
                if self.config.assists.abs_enabled && w_v_long.abs() > 0.5 {
                    let is_steering_wheel = wheel_steer_angles[i].abs() > 0.01 || clamped_ctrl.steer.abs() > 0.02;
                    let target_lat_reserve: f32 = if is_steering_wheel {
                        0.70 // Reserve 70% friction circle radius for lateral cornering
                    } else {
                        0.20 // Reserve 20% for directional stability
                    };

                    let max_fx_abs = max_friction * (1.0f32 - target_lat_reserve * target_lat_reserve).sqrt();
                    if wheel_brake_force > max_fx_abs {
                        let excess = wheel_brake_force - max_fx_abs;
                        wheel_brake_force = wheel_brake_force - excess * self.config.assists.abs_strength;
                        abs_active = true;
                    }
                }

                fx_demand += wheel_brake_force * brake_dir;
            }

            // Handbrake applies directly to rear wheels
            let is_handbraking_wheel = clamped_ctrl.handbrake && wheel_id.is_rear();
            if is_handbraking_wheel {
                let hb_dir = if w_v_long.abs() > 0.1 {
                    -w_v_long.signum()
                } else {
                    -1.0
                };
                fx_demand += self.config.handbrake_force * 0.5 * hb_dir;
            }

            // Rolling resistance opposes wheel forward motion
            let rr_coeff = self.config.rolling_resistance_coefficient * surf.rolling_resistance_multiplier();
            let rr_force = -rr_coeff * fz * (w_v_long / 0.5).tanh();
            fx_demand += rr_force;

            // Lateral demand: Pacejka Magic Formula
            let fy_demand = pacejka_lateral_force(
                slip_angle,
                fz,
                mu,
                &self.config.tire,
                is_handbraking_wheel,
            );

            // Friction ellipse combination
            let (fx, fy) = solve_combined_slip_forces(fx_demand, fy_demand, max_friction);

            // Longitudinal slip ratio estimation
            let slip_ratio = if max_friction > 1e-3 {
                (fx / max_friction).clamp(-1.0, 1.0)
            } else {
                0.0
            };

            // Skid telemetry (suppressed in mid-air)
            let (skid_intensity, is_skidding) = if self.state.elevation > 0.08 {
                (0.0, false)
            } else {
                compute_skid_telemetry(
                    slip_angle,
                    slip_ratio,
                    wheel_v_world.length(),
                    is_handbraking_wheel,
                    &self.config.tire,
                    surf,
                )
            };

            // Transform wheel forces to world frame
            let wheel_force_world = wheel_fwd * fx + wheel_right * fy;
            total_wheel_force_world += wheel_force_world;

            // Torque around CG = r_world.x * F_world.y - r_world.y * F_world.x
            let torque = offset_world.x * wheel_force_world.y - offset_world.y * wheel_force_world.x;
            total_wheel_torque += torque;

            // Store telemetry
            self.state.wheels[i] = WheelTelemetry {
                id: wheel_id,
                slip_angle,
                slip_ratio,
                normal_load: fz,
                lateral_force: fy,
                longitudinal_force: fx,
                steer_angle: wheel_steer_angles[i],
                world_velocity: wheel_v_world,
                wheel_pos_world,
                skid_intensity,
                is_skidding,
                surface: surf,
            };
        }
        self.state.abs_active = abs_active;

        // 5. Aerodynamic drag, yaw damping, and ESC
        let avg_surface_drag: f32 = surfaces.iter().map(|s| s.surface_drag_multiplier()).sum::<f32>() / 4.0;
        let avg_surface_mu: f32 = surfaces.iter().map(|s| s.friction_coefficient()).sum::<f32>() / 4.0;
        let drag_fwd = -self.config.air_drag_coefficient * v_long * v_long.abs() * avg_surface_drag;
        let drag_lat = -self.config.lateral_drag_coefficient * v_lat * v_lat.abs() * avg_surface_drag;
        let drag_world = fwd * drag_fwd + right * drag_lat;

        let base_yaw_damping = -self.config.angular_damping * omega;

        let mut esc_torque = 0.0f32;
        let mut esc_active = false;

        if self.config.assists.esc_enabled
            && self.state.speed > 2.5
            && !(self.config.assists.handbrake_bypass && clamped_ctrl.handbrake)
        {
            let wheelbase = self.config.wheelbase;
            let kinematic_yaw_rate = (v_long / wheelbase) * self.state.steer_angle.tan();
            // Max physical yaw rate governed by tire grip: omega_max = (mu * g) / V
            let max_physical_yaw_rate = ((avg_surface_mu * g) / v_long.abs().max(2.0)).max(0.40);
            let target_yaw_rate = kinematic_yaw_rate.clamp(-max_physical_yaw_rate, max_physical_yaw_rate);

            // ESC targets oversteer (rotating faster into turn than commanded or spinning out)
            let is_oversteering = (omega.signum() == target_yaw_rate.signum() && omega.abs() > (target_yaw_rate.abs() + 0.10))
                || (omega.abs() > 0.35 && target_yaw_rate.abs() < 0.1)
                || (omega.signum() != target_yaw_rate.signum() && omega.abs() > 0.25);

            if is_oversteering {
                let yaw_error = omega - target_yaw_rate;
                let yaw_thresh = self.config.assists.esc_yaw_threshold;
                if yaw_error.abs() > yaw_thresh {
                    let excess_yaw = (yaw_error.abs() - yaw_thresh) * yaw_error.signum();
                    let esc_gain = self.config.inertia * 5.0 * self.config.assists.esc_strength;
                    esc_torque = -excess_yaw * esc_gain;
                    esc_active = true;
                }
            }
        }
        self.state.esc_active = esc_active;

        let yaw_damping_torque = base_yaw_damping + esc_torque;

        // 6. Net world forces & accelerations
        let net_force_world = total_wheel_force_world + drag_world;
        let net_torque = total_wheel_torque + yaw_damping_torque;

        let linear_accel_world = net_force_world / self.config.mass;
        let angular_accel = net_torque / self.config.inertia;

        // Local acceleration for next frame weight transfer
        let accel_long = linear_accel_world.dot(fwd);
        let accel_lat = linear_accel_world.dot(right);
        // Exponential smoothing filter to eliminate numerical oscillations
        let alpha_filter = (dt * 15.0).min(1.0);
        self.state.acceleration_local = self.state.acceleration_local * (1.0 - alpha_filter)
            + Vec2::new(accel_long, accel_lat) * alpha_filter;

        // 7. Numerical Integration (Semi-implicit Euler)
        self.state.velocity += linear_accel_world * dt;
        self.state.angular_velocity += angular_accel * dt;

        // Low speed resting lock to prevent micro-jitter when stopped
        if self.state.speed < 0.05
            && clamped_ctrl.throttle < 1e-3
            && clamped_ctrl.brake < 1e-3
            && !clamped_ctrl.handbrake
        {
            self.state.velocity = Vec2::ZERO;
            self.state.angular_velocity = 0.0;
        }

        self.state.position += self.state.velocity * dt;
        self.state.angle = normalize_angle(self.state.angle + self.state.angular_velocity * dt);

        // Update body sideslip and drift status
        let updated_fwd = self.forward_vector();
        let updated_right = self.right_vector();
        let updated_v_long = self.state.velocity.dot(updated_fwd);
        let updated_v_lat = self.state.velocity.dot(updated_right);
        self.state.local_velocity = Vec2::new(updated_v_long, updated_v_lat);
        self.state.speed = self.state.velocity.length();

        self.state.sideslip_angle = updated_v_lat.atan2(updated_v_long.abs().max(0.1));

        let is_any_rear_skidding = self.state.wheels[2].is_skidding || self.state.wheels[3].is_skidding;
        let is_drifting = self.state.sideslip_angle.abs() > 0.16
            && self.state.speed > 4.0
            && is_any_rear_skidding;

        self.state.is_drifting = is_drifting;
        if is_drifting {
            self.state.drift_score += self.state.sideslip_angle.abs() * self.state.speed * dt;
        }
    }

    /// Initiates a ballistic jump launch with given launch direction, speed, and ramp angle.
    pub fn launch_jump(&mut self, direction: Vec2, launch_speed: f32, ramp_angle_deg: f32) {
        let dir = direction.normalize_or_zero();
        let speed_along_dir = self.state.velocity.dot(dir).max(0.0);
        let angle_rad = ramp_angle_deg.to_radians();
        let v_z = speed_along_dir * angle_rad.sin() * 0.75 + launch_speed;
        self.state.vertical_velocity = v_z.max(3.0);
        self.state.elevation = 0.05;
        self.state.is_airborne = true;
        self.state.air_time = 0.0;
        self.state.jump_count += 1;
    }

    /// Checks if car can launch off the given jump ramp.
    pub fn try_trigger_jump_ramp(&mut self, ramp: &crate::track::geometry::JumpRamp) -> bool {
        if self.state.elevation <= 0.1 && ramp.contains(self.state.position) {
            let speed_along_dir = self.state.velocity.dot(ramp.direction);
            if speed_along_dir > 3.5 {
                self.launch_jump(ramp.direction, ramp.launch_speed, ramp.ramp_angle_deg);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car_initialization() {
        let car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(10.0, 20.0), 0.0);
        assert_eq!(car.state.position, Vec2::new(10.0, 20.0));
        assert_eq!(car.state.angle, 0.0);
        assert_eq!(car.forward_vector(), Vec2::new(1.0, 0.0));
        assert_eq!(car.right_vector(), Vec2::new(0.0, -1.0));
    }

    #[test]
    fn test_ackermann_angles() {
        let car = Car::new(CarConfig::sports_car());
        // Straight
        let (fl, fr) = car.compute_ackermann_angles(0.0);
        assert_eq!(fl, 0.0);
        assert_eq!(fr, 0.0);

        // Right turn (steer_angle < 0, clockwise): inner wheel FR has larger negative angle magnitude than outer FL
        let (fl_r, fr_r) = car.compute_ackermann_angles(-0.3);
        assert!(fl_r < 0.0);
        assert!(fr_r < 0.0);
        assert!(fr_r.abs() > fl_r.abs(), "Inner wheel FR ({fr_r}) must steer more than outer FL ({fl_r})");

        // Left turn (steer_angle > 0, counter-clockwise): inner wheel FL has larger positive angle than outer FR
        let (fl_l, fr_l) = car.compute_ackermann_angles(0.3);
        assert!(fl_l > 0.0);
        assert!(fr_l > 0.0);
        assert!(fl_l > fr_l, "Inner wheel FL ({}) must exceed outer FR ({})", fl_l, fr_l);
    }

    #[test]
    fn test_straight_line_step() {
        let mut car = Car::new(CarConfig::sports_car());
        let controls = CarControls::new(1.0, 0.0, 0.0, false);

        for _ in 0..60 {
            car.step(&controls, SurfaceType::Asphalt, 1.0 / 60.0);
        }

        assert!(car.state.speed > 5.0, "Car should accelerate forward, speed is {}", car.state.speed);
        assert!(car.state.position.x > 1.0, "Car should move in +X");
        assert!(car.state.position.y.abs() < 1e-3, "Car should not deviate laterally");
    }

    #[test]
    fn test_state_save_restore() {
        let mut car = Car::new(CarConfig::sports_car());
        let ctrl = CarControls::new(1.0, 0.2, 0.0, false);
        for _ in 0..100 {
            car.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
        }

        let saved = car.state().clone();
        for _ in 0..50 {
            car.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
        }
        assert_ne!(car.state().position, saved.position);

        car.set_state(saved.clone());
        assert_eq!(car.state(), &saved);
    }
}
