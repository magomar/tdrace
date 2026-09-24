use std::f32::consts::PI;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::config::CarConfig;
use super::surface::{SurfaceSampler, SurfaceType};
use super::tire::{
    compute_skid_telemetry, solve_combined_slip_forces, WheelAssembly, WheelId, WheelTelemetry,
};

/// Helper returning default wheel assemblies state for CarState deserialization.
pub fn default_wheel_assemblies_state() -> [WheelAssembly; 4] {
    [
        WheelAssembly::default(),
        WheelAssembly::default(),
        WheelAssembly::default(),
        WheelAssembly::default(),
    ]
}

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
    /// Physical wheel assembly dynamics (inertia, rotation, thermal wear) [FL, FR, RL, RR].
    #[serde(default = "default_wheel_assemblies_state")]
    pub wheel_assemblies: [WheelAssembly; 4],
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
    /// Whether brakes (service brake or handbrake) are actively being applied.
    #[serde(default)]
    pub is_braking: bool,
    /// Road surface elevation underneath the vehicle in meters (z >= 0.0).
    #[serde(default)]
    pub road_elevation: f32,
    /// Ramp surface elevation underneath the vehicle while climbing an on-track jump ramp in meters (z >= 0.0).
    #[serde(default)]
    pub ramp_elevation: f32,
    /// Road cross-slope banking angle in degrees (default: 0.0; + = right side elevated / banked left, - = left side elevated / banked right).
    #[serde(default)]
    pub road_bank_angle: f32,
    /// Track transverse right vector in world space for resolving banking incline gravity.
    #[serde(default)]
    pub track_right: Vec2,
    /// Longitudinal road grade slope in radians (+ = uphill, - = downhill).
    #[serde(default)]
    pub road_grade_slope: f32,
    /// Vertical road curvature d(slope)/ds in rad/m (+ = dip/compression, - = crest/unloading).
    #[serde(default)]
    pub road_vertical_curvature: f32,
    /// Track longitudinal forward vector in world space for resolving grade incline gravity.
    #[serde(default)]
    pub track_forward: Vec2,
    /// Elevation / vertical jump altitude above road in meters (z >= 0.0).
    pub elevation: f32,
    /// Vertical velocity in m/s (positive = ascending, negative = falling).
    pub vertical_velocity: f32,
    /// Whether the car is currently airborne (off the ground).
    pub is_airborne: bool,
    /// Time spent in the air during the current jump in seconds.
    pub air_time: f32,
    /// Duration of the most recent aerial jump in seconds (captured upon touchdown).
    #[serde(default)]
    pub last_air_time: f32,
    /// Cumulative count of jumps completed.
    pub jump_count: u32,
    /// Flag indicating the vehicle touched down on the ground during this physics tick.
    pub just_landed: bool,
    /// Aerodynamic drafting / slipstream drag reduction factor [0.0 = clean air, up to ~0.40 = 40% drag reduction in wake].
    #[serde(default)]
    pub draft_intensity: f32,
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
            wheel_assemblies: default_wheel_assemblies_state(),
            speed: 0.0,
            local_velocity: Vec2::ZERO,
            sideslip_angle: 0.0,
            is_drifting: false,
            drift_score: 0.0,
            tcs_active: false,
            esc_active: false,
            abs_active: false,
            is_braking: false,
            road_elevation: 0.0,
            ramp_elevation: 0.0,
            road_bank_angle: 0.0,
            track_right: Vec2::ZERO,
            road_grade_slope: 0.0,
            road_vertical_curvature: 0.0,
            track_forward: Vec2::ZERO,
            elevation: 0.0,
            vertical_velocity: 0.0,
            is_airborne: false,
            air_time: 0.0,
            last_air_time: 0.0,
            jump_count: 0,
            just_landed: false,
            draft_intensity: 0.0,
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
        let mut state = CarState::default();
        for i in 0..4 {
            state.wheel_assemblies[i] = WheelAssembly::new(config.wheels[i]);
            state.wheels[i].angular_velocity = state.wheel_assemblies[i].angular_velocity;
            state.wheels[i].temperature = state.wheel_assemblies[i].temperature;
            state.wheels[i].wear = state.wheel_assemblies[i].wear;
            state.wheels[i].is_locked = state.wheel_assemblies[i].is_locked;
        }
        Self {
            config,
            state,
        }
    }

    /// Sets the initial pose (position and yaw angle).
    pub fn with_pose(mut self, position: Vec2, angle: f32) -> Self {
        self.state.position = position;
        self.state.angle = normalize_angle(angle);
        self
    }

    /// Sets the initial vehicle velocity and synchronizes wheel rotational velocities.
    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.set_velocity(velocity);
        self
    }

    /// Sets vehicle velocity and synchronizes wheel rotational velocities to kinematic rolling speed.
    pub fn set_velocity(&mut self, velocity: Vec2) {
        self.state.velocity = velocity;
        self.state.speed = velocity.length();
        let v_long = velocity.dot(self.forward_vector());
        for (i, w) in self.state.wheel_assemblies.iter_mut().enumerate() {
            let r = w.config.tire_radius.max(1e-2);
            w.angular_velocity = v_long / r;
            self.state.wheels[i].angular_velocity = w.angular_velocity;
        }
    }

    /// Returns total vehicle vertical altitude above ground (road elevation + ramp elevation + jump height).
    #[inline]
    pub fn total_elevation(&self) -> f32 {
        self.state.road_elevation + self.state.ramp_elevation + self.state.elevation
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
        for i in 0..4 {
            self.state.wheel_assemblies[i].config = config.wheels[i];
        }
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

    /// Computes the aerodynamic slipstream drafting intensity [0.0..0.40] based on opponent vehicles
    /// within the forward wake cone (up to 32m ahead, +/- 2.8m lateral), with subtle push-draft support.
    pub fn compute_draft_intensity(&self, other_cars: &[&Car]) -> f32 {
        if self.state.speed < 20.0 {
            return 0.0;
        }

        let my_pos = self.state.position;
        let my_fwd = self.forward_vector();
        let my_right = self.right_vector();

        let mut max_draft = 0.0f32;

        for opp in other_cars {
            if std::ptr::eq(*opp, self) {
                continue;
            }
            if opp.state.speed < 15.0 {
                continue;
            }

            let to_opp = opp.state.position - my_pos;
            let fwd_dist = to_opp.dot(my_fwd);
            let lat_dist = to_opp.dot(my_right).abs();

            // Check if opponent is ahead within slipstream wake cone (2.0m to 32.0m)
            if fwd_dist > 2.0 && fwd_dist < 32.0 && lat_dist < 2.8 {
                let dist_factor = 1.0 - (fwd_dist - 2.0) / 30.0;
                let lat_factor = (1.0 - lat_dist / 2.8).clamp(0.0, 1.0);
                let draft = 0.38 * dist_factor * lat_factor;
                if draft > max_draft {
                    max_draft = draft;
                }
            } else if fwd_dist < -2.0 && fwd_dist > -10.0 && lat_dist < 2.0 {
                // Subtle push draft from trailing car
                let push_draft = 0.06 * (1.0 - (-fwd_dist - 2.0) / 8.0) * (1.0 - lat_dist / 2.0);
                if push_draft > max_draft {
                    max_draft = push_draft;
                }
            }
        }

        max_draft
    }

    /// Steps the physics simulation forward by a fixed timestep `dt` over a uniform surface.
    #[inline]
    pub fn step(&mut self, controls: &CarControls, surface: SurfaceType, dt: f32) {
        self.step_per_wheel(controls, [surface; 4], dt);
    }

    /// Steps the physics simulation forward using an arbitrary terrain `SurfaceSampler`.
    ///
    /// Evaluates contact patches for all 4 wheels in world coordinates and queries the sampler.
    pub fn step_with_sampler<S: SurfaceSampler>(
        &mut self,
        controls: &CarControls,
        sampler: &S,
        dt: f32,
    ) {
        let wheel_positions = self.wheel_positions_world();
        let surfaces = [
            sampler.sample_surface(wheel_positions[0]).surface_type,
            sampler.sample_surface(wheel_positions[1]).surface_type,
            sampler.sample_surface(wheel_positions[2]).surface_type,
            sampler.sample_surface(wheel_positions[3]).surface_type,
        ];
        let center_props = sampler.sample_surface(self.state.position);
        self.state.road_elevation = center_props.elevation;
        self.state.road_bank_angle = center_props.bank_angle;
        self.state.track_right = center_props.track_right;

        self.step_per_wheel(controls, surfaces, dt);
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
                self.state.last_air_time = self.state.air_time;
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
        self.state.is_braking = clamped_ctrl.brake > 0.05 || clamped_ctrl.handbrake;
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

        // Counter-steer / self-aligning drift recovery assist (forward motion only)
        if self.config.assists.counter_steer_assist_enabled
            && !clamped_ctrl.handbrake
            && !clamped_ctrl.reverse
            && v_long > 1.0
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

        // 3. Dynamic Weight Transfer Calculation & Superelevation (Banking) & 3D Grade Slope
        let g = 9.81;
        let total_weight = self.config.mass * g;

        let a_long = self.state.acceleration_local.x;
        let a_lat = self.state.acceleration_local.y;

        // Banking cross-slope angle & dynamic centripetal compression
        let bank_deg = self.state.road_bank_angle;
        let bank_rad = bank_deg.to_radians();
        let (bank_sin, bank_cos) = if bank_deg.abs() > 1e-4 {
            (bank_rad.sin(), bank_rad.cos())
        } else {
            (0.0, 1.0)
        };

        // Longitudinal grade slope & vertical curvature (crest unloading / dip compression)
        let grade_rad = self.state.road_grade_slope;
        let (grade_sin, grade_cos) = if grade_rad.abs() > 1e-4 {
            (grade_rad.sin(), grade_rad.cos())
        } else {
            (0.0, 1.0)
        };

        let vert_curv = self.state.road_vertical_curvature;
        let speed_sq = self.state.speed * self.state.speed;

        // Dynamic vertical acceleration from road vertical curvature: a_z = v^2 * kappa_z
        // kappa_z > 0 = dip (upward centrifugal acceleration -> compression)
        // kappa_z < 0 = crest (downward centrifugal acceleration -> unloading)
        let vert_centrifugal = speed_sq * vert_curv;

        // Airborne crest launch: if crest curvature is sharp and speed is high enough to exceed gravity
        if self.state.elevation <= 0.0 && vert_curv < -1e-3 && -vert_centrifugal > g * 1.05 {
            let v_launch = ((-vert_centrifugal - g).max(0.0)).sqrt() * 0.45;
            if v_launch > 0.5 && self.state.vertical_velocity <= 0.0 {
                self.state.vertical_velocity = v_launch.min(4.5);
                self.state.is_airborne = true;
            }
        }

        let g_eff = (g * grade_cos * bank_cos + vert_centrifugal).max(g * 0.05);
        let effective_normal_weight = self.config.mass * g_eff;

        // Centripetal acceleration pressing the car into the banked turn
        let bank_compression = if bank_deg.abs() > 1e-4 {
            self.config.mass * a_lat.abs() * bank_sin.abs()
        } else {
            0.0
        };

        let static_front_load = effective_normal_weight * (lr / wheelbase) + bank_compression * (lr / wheelbase);
        let static_rear_load = effective_normal_weight * (lf / wheelbase) + bank_compression * (lf / wheelbase);

        // Acceleration squat (a_long > 0): front unloads, rear loads
        // Grade incline pitch (grade_sin > 0 = uphill): front unloads, rear loads
        let grade_pitch = self.config.mass * g * grade_sin * (self.config.cg_height / wheelbase);
        let delta_fz_long = (self.config.mass * a_long * (self.config.cg_height / wheelbase) + grade_pitch)
            * self.config.weight_transfer_longitudinal;

        // Cornering roll & gravity cross-slope roll moment
        let cross_slope_roll = if bank_deg.abs() > 1e-4 {
            self.config.mass * g * bank_sin * (self.config.cg_height / self.config.track_width)
        } else {
            0.0
        };

        let delta_fz_lat_f = ((self.config.mass * a_lat * (self.config.cg_height / self.config.track_width) + cross_slope_roll) * (lr / wheelbase))
            * self.config.weight_transfer_lateral;

        let delta_fz_lat_r = ((self.config.mass * a_lat * (self.config.cg_height / self.config.track_width) + cross_slope_roll) * (lf / wheelbase))
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
        let max_driven_wheel_linear_speed = self.state.wheel_assemblies.iter()
            .filter(|w| w.config.drive_torque_factor > 0.0)
            .map(|w| (w.angular_velocity * w.config.tire_radius).abs())
            .fold(0.0f32, f32::max);
        let effective_engine_speed = v_long.abs().max(max_driven_wheel_linear_speed);
        let speed_ratio = effective_engine_speed / top_speed;
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
            let thresh = self.config.assists.tcs_slip_threshold;
            let rear_slip_lat = self.state.wheels[2].slip_angle.abs().max(self.state.wheels[3].slip_angle.abs());
            let is_cornering = self.state.steer_angle.abs() > 0.02 || self.state.sideslip_angle.abs() > 0.03 || rear_slip_lat > 0.08;

            let max_driven_slip_long = self.state.wheel_assemblies.iter()
                .filter(|w| w.config.drive_torque_factor > 0.0)
                .map(|w| w.compute_slip_ratio(v_long))
                .fold(0.0f32, f32::max);

            let mut cut = 0.0f32;
            if is_cornering && rear_slip_lat > thresh {
                let excess_lat = (rear_slip_lat - thresh).max(0.0) / thresh;
                cut = cut.max((excess_lat * self.config.assists.tcs_strength).clamp(0.0, 0.75));
                tcs_active = true;
            }

            if max_driven_slip_long > thresh {
                let excess_long = (max_driven_slip_long - thresh) / (1.0 - thresh).max(0.05);
                // At standstill / low-speed launch, blend TCS intervention to allow standing takeoff without bogging down
                let launch_blend = (v_long.abs() / 3.0).clamp(0.25, 1.0);
                cut = cut.max((excess_long * self.config.assists.tcs_strength * launch_blend).clamp(0.0, 0.65));
                tcs_active = true;
            }

            if tcs_active {
                drive_torque_multiplier = 1.0 - cut;
            }
        }
        self.state.tcs_active = tcs_active;

        let mut engine_brake_multiplier = 1.0f32;
        // Engine Drag Reduction (EDR / MSR): prevent lift-off snap oversteer when the chassis is in
        // transient sideslip or high yaw rate, letting the rear tires regain lateral restoring traction.
        if self.state.sideslip_angle.abs() > 0.08 || omega.abs() > 0.15 {
            let slide_severity = ((self.state.sideslip_angle.abs() - 0.08) / 0.12)
                .max((omega.abs() - 0.15) / 0.25)
                .clamp(0.0, 1.0);
            engine_brake_multiplier *= 1.0 - 0.85 * slide_severity;
        }

        let total_drive_force = if clamped_ctrl.reverse {
            -clamped_ctrl.throttle * self.config.max_reverse_force
        } else if clamped_ctrl.throttle > 0.0 {
            clamped_ctrl.throttle * self.config.max_engine_force * engine_taper * drive_torque_multiplier
        } else if self.config.engine_braking_coefficient > 0.0 && v_long.abs() > 0.05 {
            // Enhanced generic motor brake with EDR modulation
            let generic_motor_brake_boost = 1.85f32;
            -self.config.engine_braking_coefficient
                * generic_motor_brake_boost
                * total_weight
                * (v_long / 1.5).tanh()
                * engine_brake_multiplier
        } else {
            0.0
        };

        // Progressive, non-linear service brake input mapping:
        // Soft, progressive response at light-to-medium pedal travel for delicate trail-braking and apex adjustments,
        // smoothly ramping up to maximum deceleration on full brake application.
        let raw_brake = clamped_ctrl.brake;
        let progressive_brake = raw_brake.powf(1.4);
        let total_brake_force = progressive_brake * self.config.max_brake_force;
        let mut abs_active = false;

        let total_normal_load: f32 = normal_loads.iter().sum();

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
            let slip_angle = -w_v_lat.atan2(w_v_long.abs().max(2.5));

            // Surface properties
            let surf = surfaces[i];
            let mut mu = surf.friction_coefficient();
            if surf == SurfaceType::SheetIce {
                let alpha = self.config.terrain.ice_grip_multiplier.clamp(0.50, 10.0);
                mu = (mu * alpha).min(1.20);
            }
            let prev_dirt = self.state.wheels[i].dirt_contamination;
            let prev_dirt_surface = self.state.wheels[i].dirt_surface;

            // Apply minor temporary grip penalty if tire is contaminated with loose dirt/gravel on pavement
            if surf.is_rigid_pavement() && prev_dirt > 0.02 {
                mu *= (1.0 - 0.20 * prev_dirt).max(0.65);
            }

            let fz = normal_loads[i];
            let max_friction = mu * fz;

            let r = self.state.wheel_assemblies[i].config.tire_radius.max(1e-2);

            // Kinematic rolling synchronization: if vehicle was spawned/set at speed without previous lockup
            if !self.state.wheel_assemblies[i].is_locked
                && self.state.wheel_assemblies[i].angular_velocity.abs() < 1e-3
                && w_v_long.abs() > 1.0
            {
                self.state.wheel_assemblies[i].angular_velocity = w_v_long / r;
            }

            // Dynamic Electronic Brakeforce Distribution (EBD):
            // Blends nominal brake bias with dynamic normal load fraction.
            // As weight transfers forward under deceleration, front brake share increases and rear decreases.
            // Safety limiter: ensure rear brake share never exceeds dynamic rear load capability,
            // guaranteeing the front axle saturates before the rear axle (stable understeer).
            let static_share = self.state.wheel_assemblies[i].config.brake_bias_factor;
            let brake_share = if self.config.assists.abs_enabled {
                let dynamic_load_share = if total_normal_load > 1e-3 {
                    fz / total_normal_load
                } else {
                    0.25
                };
                let nominal_share = 0.20 * static_share + 0.80 * dynamic_load_share;
                if wheel_id.is_rear() {
                    nominal_share.min(dynamic_load_share * 0.75)
                } else {
                    nominal_share
                }
            } else {
                static_share
            };

            let drive_share = self.state.wheel_assemblies[i].config.drive_torque_factor;
            let (wheel_drive_force, mut engine_retard_force) = if clamped_ctrl.throttle > 0.0 || clamped_ctrl.reverse {
                (total_drive_force * drive_share, 0.0)
            } else {
                (0.0, (-total_drive_force * drive_share).max(0.0))
            };

            // Electronic Drag Reduction (EDR / MSR): prevent engine braking overrun from breaking tire grip when ABS is enabled
            if self.config.assists.abs_enabled && engine_retard_force > max_friction * 0.70 {
                engine_retard_force = max_friction * 0.70;
            }

            let mut wheel_brake_force = total_brake_force * brake_share;

            // Dynamic EBD limiter: rear axle retarding force must not exceed rear dynamic load envelope
            if wheel_id.is_rear() && self.config.assists.abs_enabled {
                let max_rear_retard = max_friction * 0.82;
                if wheel_brake_force + engine_retard_force > max_rear_retard {
                    wheel_brake_force = (max_rear_retard - engine_retard_force).max(0.0);
                }
            }

            let is_handbraking_wheel = clamped_ctrl.handbrake && wheel_id.is_rear();
            if is_handbraking_wheel {
                wheel_brake_force += self.config.handbrake_force * 0.5;
            }

            // Anti-lock Braking System (ABS) & Cornering Brake Control (CBC)
            if self.config.assists.abs_enabled && w_v_long.abs() > 0.5 {
                let is_cornering = wheel_steer_angles[i].abs() > 0.01
                    || clamped_ctrl.steer.abs() > 0.02
                    || self.state.sideslip_angle.abs() > 0.02
                    || omega.abs() > 0.06;

                // CBC: trim inside rear brake pressure under oversteering yaw divergence
                let kinematic_yaw_rate = (v_long / self.config.wheelbase) * self.state.steer_angle.tan();
                let yaw_divergence = omega - kinematic_yaw_rate;
                let is_oversteering_under_brake = (omega.signum() == kinematic_yaw_rate.signum() && omega.abs() > (kinematic_yaw_rate.abs() + 0.08))
                    || (kinematic_yaw_rate.abs() < 0.05 && omega.abs() > 0.08)
                    || (omega.signum() != kinematic_yaw_rate.signum() && omega.abs() > 0.12);

                if is_oversteering_under_brake {
                    let yaw_sign = omega.signum();
                    let is_inside_rear = (yaw_sign > 0.0 && wheel_id == WheelId::RearLeft)
                        || (yaw_sign < 0.0 && wheel_id == WheelId::RearRight);
                    if is_inside_rear {
                        let cbc_cut = (yaw_divergence.abs() * 1.5 * self.config.assists.abs_strength).clamp(0.0, 0.45);
                        wheel_brake_force *= 1.0 - cbc_cut;
                    }
                }

                // ABS: preserve lateral grip envelope and prevent lockup
                let target_lat_reserve: f32 = if is_cornering {
                    if wheel_id.is_front() {
                        0.72 // High front lateral authority for crisp turn-in under threshold braking
                    } else {
                        0.65 // High rear lateral reserve guarantees rear axle stays planted
                    }
                } else {
                    if wheel_id.is_rear() {
                        0.35 // Reserve 35% lateral grip on rear axle in straight lines
                    } else {
                        0.20 // Front maximizes longitudinal stopping power (98% peak Fx)
                    }
                };

                let max_fx_abs = max_friction * (1.0f32 - target_lat_reserve * target_lat_reserve).sqrt();
                let total_retard = wheel_brake_force + engine_retard_force;
                if total_retard > max_fx_abs {
                    let excess = total_retard - max_fx_abs;
                    wheel_brake_force = (wheel_brake_force - excess * self.config.assists.abs_strength).max(0.0);
                    abs_active = true;
                }

                // If wheel has nevertheless started to slip heavily, modulate further
                let current_slip = self.state.wheel_assemblies[i].compute_slip_ratio(w_v_long);
                if current_slip < -self.config.assists.abs_slip_threshold || self.state.wheel_assemblies[i].is_locked {
                    let excess_slip = (-current_slip - self.config.assists.abs_slip_threshold).max(0.0);
                    let pulse_cut = (excess_slip * 2.0 * self.config.assists.abs_strength).clamp(0.0, 0.90);
                    wheel_brake_force *= 1.0 - pulse_cut;
                    abs_active = true;
                }

                // Generically cap wheel braking force near the tire traction envelope when ABS is active
                let max_traction_cap = max_friction * 0.98;
                if wheel_brake_force > max_traction_cap {
                    wheel_brake_force = max_traction_cap;
                }
            }

            let total_retard_force = wheel_brake_force + engine_retard_force;
            let total_brake_torque = total_retard_force * r;
            let brake_dir = if w_v_long.abs() > 0.1 {
                -w_v_long.signum()
            } else {
                -v_long.signum()
            };

            let mut fx_demand = wheel_drive_force + total_retard_force * brake_dir;

            // Rolling resistance opposes wheel forward motion
            let base_rr_mult = surf.rolling_resistance_multiplier();
            let effective_rr_mult = if surf.is_sand() {
                let gamma = self.config.terrain.sand_flotation.clamp(0.10, 1.0);
                1.0 + (base_rr_mult - 1.0) * gamma
            } else if surf.is_mud() {
                let gamma = self.config.terrain.mud_flotation.clamp(0.10, 1.0);
                1.0 + (base_rr_mult - 1.0) * gamma
            } else {
                base_rr_mult
            };
            let rr_coeff = self.config.rolling_resistance_coefficient * effective_rr_mult;
            let rr_force = -rr_coeff * fz * (w_v_long / 0.5).tanh();
            fx_demand += rr_force;

            // Lateral demand: Pacejka Magic Formula with thermal degradation and low-speed stabilization
            let low_speed_blend = (w_v_long.abs() / 3.0).clamp(0.05, 1.0);
            let fy_demand = self.state.wheel_assemblies[i].lateral_force(
                slip_angle,
                fz,
                mu,
                is_handbraking_wheel,
            ) * low_speed_blend;

            // Friction ellipse combination
            let (fx, fy) = solve_combined_slip_forces(fx_demand, fy_demand, max_friction);

            // Step wheel rotational dynamics
            let drive_torque = wheel_drive_force * r;
            let fx_grip_capacity = (max_friction * max_friction - fy * fy).max(0.0).sqrt();
            self.state.wheel_assemblies[i].step_rotation(drive_torque, total_brake_torque, fx_grip_capacity, w_v_long, dt);

            // Exact kinematic slip ratio from integrated wheel rotational velocity
            let slip_ratio = self.state.wheel_assemblies[i].compute_slip_ratio(w_v_long);

            // Step thermal dissipation and mechanical tread wear
            self.state.wheel_assemblies[i].step_thermal_and_wear(
                fx,
                fy,
                slip_ratio,
                slip_angle,
                wheel_v_world.length(),
                dt,
            );

            // Skid telemetry (suppressed in mid-air)
            let (skid_intensity, is_skidding) = if self.state.is_airborne || self.state.elevation > 0.0 {
                (0.0, false)
            } else {
                compute_skid_telemetry(
                    slip_angle,
                    slip_ratio,
                    wheel_v_world.length(),
                    is_handbraking_wheel,
                    &self.state.wheel_assemblies[i].config.tire_model,
                    surf,
                )
            };

            // Transform wheel forces to world frame
            let wheel_force_world = wheel_fwd * fx + wheel_right * fy;
            total_wheel_force_world += wheel_force_world;

            // Torque around CG = r_world.x * F_world.y - r_world.y * F_world.x
            let torque = offset_world.x * wheel_force_world.y - offset_world.y * wheel_force_world.x;
            total_wheel_torque += torque;

            let (new_dirt, new_dirt_surface) = if surf.is_loose_deformable() || surf == SurfaceType::Grass {
                let accumulated = (prev_dirt + 3.5 * dt).min(1.0);
                (accumulated, surf)
            } else if surf.is_rigid_pavement() {
                let speed = wheel_v_world.length();
                let scrubbed = (prev_dirt - 0.08 * (speed / 10.0).max(0.1) * dt).max(0.0);
                (scrubbed, if scrubbed > 0.001 { prev_dirt_surface } else { surf })
            } else {
                (prev_dirt, prev_dirt_surface)
            };

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
                dirt_contamination: new_dirt,
                dirt_surface: new_dirt_surface,
                angular_velocity: self.state.wheel_assemblies[i].angular_velocity,
                temperature: self.state.wheel_assemblies[i].temperature,
                wear: self.state.wheel_assemblies[i].wear,
                is_locked: self.state.wheel_assemblies[i].is_locked,
            };
        }
        self.state.abs_active = abs_active;

        // 5. Aerodynamic drag, yaw damping, and ESC
        let avg_surface_drag: f32 = surfaces.iter().map(|s| s.surface_drag_multiplier()).sum::<f32>() / 4.0;
        let avg_surface_mu: f32 = surfaces.iter().map(|s| s.friction_coefficient()).sum::<f32>() / 4.0;
        let effective_drag_coeff = self.config.air_drag_coefficient * (1.0 - self.state.draft_intensity.clamp(0.0, 0.50));
        let drag_fwd = -effective_drag_coeff * v_long * v_long.abs() * avg_surface_drag;
        let drag_lat = -self.config.lateral_drag_coefficient * v_lat * v_lat.abs() * avg_surface_drag;
        let drag_world = fwd * drag_fwd + right * drag_lat;

        let base_yaw_damping = -self.config.angular_damping * omega;

        let mut esc_torque = 0.0f32;
        let mut esc_active = false;

        if self.config.assists.esc_enabled
            && self.state.speed > 2.5
            && v_long.abs() > 0.5
            && !(self.config.assists.handbrake_bypass && clamped_ctrl.handbrake)
        {
            let wheelbase = self.config.wheelbase;
            let kinematic_yaw_rate = (v_long / wheelbase) * self.state.steer_angle.tan();
            // Max physical yaw rate governed by tire grip and aerodynamic downforce
            let downforce_load = self.config.downforce_coefficient * v_long * v_long;
            let effective_g = g + (downforce_load / self.config.mass.max(1.0));
            let max_physical_yaw_rate = ((avg_surface_mu * effective_g) / v_long.abs().max(2.0)).max(0.60);
            let target_yaw_rate = kinematic_yaw_rate.clamp(-max_physical_yaw_rate, max_physical_yaw_rate);

            let yaw_error = omega - target_yaw_rate;
            // ESC targets oversteer (rotating faster into turn than commanded, opposite to target, or uncommanded yaw)
            let is_oversteering = (omega.signum() == target_yaw_rate.signum() && omega.abs() > (target_yaw_rate.abs() + 0.06))
                || (omega.signum() != target_yaw_rate.signum() && omega.abs() > 0.10)
                || (target_yaw_rate.abs() < 0.05 && omega.abs() > 0.08);

            if is_oversteering {
                let yaw_thresh = self.config.assists.esc_yaw_threshold;
                if yaw_error.abs() > yaw_thresh {
                    let excess_yaw = (yaw_error.abs() - yaw_thresh) * yaw_error.signum();
                    let speed_boost = 1.0 + (self.state.speed / 20.0).min(3.5);
                    let esc_gain = self.config.inertia * 10.0 * speed_boost * self.config.assists.esc_strength;
                    esc_torque = -excess_yaw * esc_gain;
                    esc_active = true;
                }
            }
        }
        self.state.esc_active = esc_active;

        let yaw_damping_torque = base_yaw_damping + esc_torque;

        // Transverse gravity downhill slope force from banking
        let bank_gravity_world = if bank_deg.abs() > 1e-4 {
            let track_r = if self.state.track_right.length_squared() > 0.5 {
                self.state.track_right.normalize()
            } else {
                right
            };
            // Downhill force along cross-slope: when bank_deg > 0 (right side elevated),
            // slope pulls downhill towards the left (-track_r)
            -self.config.mass * g * bank_sin * track_r
        } else {
            Vec2::ZERO
        };

        // Longitudinal gravity downhill slope force from grade elevation
        let grade_gravity_world = if grade_rad.abs() > 1e-4 {
            let track_f = if self.state.track_forward.length_squared() > 0.5 {
                self.state.track_forward.normalize()
            } else {
                fwd
            };
            // Downhill force along track: when grade_slope > 0 (uphill along track_forward),
            // slope pulls downhill backwards (-track_f)
            -self.config.mass * g * grade_sin * track_f
        } else {
            Vec2::ZERO
        };

        let is_holding_brakes = clamped_ctrl.brake > 0.05 || clamped_ctrl.handbrake;

        // Static friction reaction for stationary or near-stopped vehicle on slopes:
        // Rubber tires cannot roll laterally; static Coulomb friction resists downhill slope forces
        // up to the traction limit (mu * N).
        let static_friction_world = if !self.state.is_airborne && ground_contact > 0.0 && self.state.speed < 0.25 {
            let total_slope_gravity = bank_gravity_world + grade_gravity_world;
            if total_slope_gravity.length_squared() > 1e-4 {
                let static_blend = (1.0 - (self.state.speed / 0.25)).clamp(0.0, 1.0);
                let max_static_friction = avg_surface_mu * total_normal_load * ground_contact;

                // Lateral holding: tires cannot roll sideways, so static friction resists lateral slope force
                let lat_slope_force = total_slope_gravity.dot(right);
                let lat_holding = -lat_slope_force.clamp(-max_static_friction, max_static_friction) * static_blend;

                // Longitudinal holding: resisted by brakes/handbrake or rolling resistance
                let long_slope_force = total_slope_gravity.dot(fwd);
                let long_holding = if is_holding_brakes {
                    -long_slope_force.clamp(-max_static_friction, max_static_friction) * static_blend
                } else {
                    let rr_holding_cap = self.config.rolling_resistance_coefficient * total_normal_load;
                    -long_slope_force.clamp(-rr_holding_cap, rr_holding_cap) * static_blend
                };

                right * lat_holding + fwd * long_holding
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };

        // 6. Net world forces & accelerations
        let net_force_world = total_wheel_force_world + drag_world + bank_gravity_world + grade_gravity_world + static_friction_world;
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

        // Low speed resting lock to prevent micro-jitter when stopped on flat ground or when holding brakes.
        // A slope is only "too steep" to remain static if the incline angle exceeds the static friction limit:
        // tan(theta) > mu for lateral banking, or grade exceeds rolling/braking limits.
        let on_steep_bank = bank_rad.abs().tan() > avg_surface_mu;
        let on_steep_grade = if is_holding_brakes {
            grade_rad.abs().tan() > avg_surface_mu
        } else {
            grade_rad.abs() > 0.02
        };
        let on_steep_slope = on_steep_bank || on_steep_grade;
        if self.state.speed < 0.05
            && clamped_ctrl.throttle < 1e-3
            && (is_holding_brakes || !on_steep_slope)
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
        let is_drifting = !self.state.is_airborne
            && self.state.elevation <= 0.0
            && self.state.sideslip_angle.abs() > 0.16
            && self.state.speed > 4.0
            && is_any_rear_skidding;

        self.state.is_drifting = is_drifting;
        if is_drifting {
            self.state.drift_score += self.state.sideslip_angle.abs() * self.state.speed * dt;
        }
    }

    /// Resets the accumulated single-maneuver drift score to zero.
    #[inline]
    pub fn reset_drift_score(&mut self) {
        self.state.drift_score = 0.0;
    }

    /// Initiates a ballistic jump launch with given launch direction, speed, and ramp angle.
    pub fn launch_jump(&mut self, direction: Vec2, _launch_speed: f32, ramp_angle_deg: f32) {
        self.launch_jump_with_height(direction, ramp_angle_deg, 0.05);
    }

    /// Initiates a realistic ballistic jump launch off a ramp lip of specified height.
    pub fn launch_jump_with_height(&mut self, direction: Vec2, ramp_angle_deg: f32, takeoff_elevation: f32) {
        let dir = direction.normalize_or_zero();
        let speed_along_dir = self.state.velocity.dot(dir).max(0.0);
        let angle_rad = ramp_angle_deg.to_radians();
        let (sin_theta, cos_theta) = angle_rad.sin_cos();

        // Realistic vertical launch velocity from incline angle and suspension compliance:
        // Long-travel chassis suspension absorbs ~25% of vertical impulse upon climbing the curve.
        let suspension_efficiency = 0.75f32;
        let v_z = speed_along_dir * sin_theta * suspension_efficiency;

        // Partition forward momentum along ramp incline (conserving kinetic energy):
        let lateral_v = self.state.velocity - dir * speed_along_dir;
        self.state.velocity = dir * (speed_along_dir * cos_theta) + lateral_v;
        self.state.speed = self.state.velocity.length();

        self.state.vertical_velocity = v_z;
        self.state.elevation = takeoff_elevation.max(0.05);
        self.state.ramp_elevation = 0.0;
        self.state.is_airborne = true;
        self.state.air_time = 0.0;
        self.state.jump_count += 1;
    }

    /// Checks if car can launch off the given jump ramp.
    pub fn try_trigger_jump(
        &mut self,
        is_on_ramp: bool,
        ramp: &JumpRampProperties,
    ) -> bool {
        if !self.state.is_airborne && is_on_ramp {
            let speed_along_dir = self.state.velocity.dot(ramp.direction);
            if speed_along_dir > 3.5 {
                self.launch_jump_with_height(ramp.direction, ramp.ramp_angle_deg, ramp.height);
                return true;
            }
        }
        false
    }
}

/// Minimal kinematic properties required to trigger a jump ramp launch.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JumpRampProperties {
    pub direction: Vec2,
    #[serde(default)]
    pub launch_speed: f32,
    pub ramp_angle_deg: f32,
    #[serde(default)]
    pub height: f32,
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
    fn test_reverse_straight_line_neutral_steer() {
        let mut car = Car::new(CarConfig::sports_car());
        let dt = 1.0 / 60.0;
        let mut ctrl = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl.reverse = true;

        for _ in 0..120 {
            car.step(&ctrl, SurfaceType::Asphalt, dt);
        }
        assert!(car.state.local_velocity.x < -3.0, "Car should accelerate backward, was {}", car.state.local_velocity.x);
        assert!(car.state.steer_angle.abs() < 1e-3, "Steer angle should remain zero without input");
        assert!(car.state.angle.abs() < 1e-3, "Car should not deviate or force turning, angle was {}", car.state.angle);

        // Active steering in reverse turns the car
        let mut car_steer = Car::new(CarConfig::sports_car());
        let mut ctrl_steer = CarControls::new(1.0, 0.3, 0.0, false);
        ctrl_steer.reverse = true;
        for _ in 0..120 {
            car_steer.step(&ctrl_steer, SurfaceType::Asphalt, dt);
        }
        assert!(car_steer.state.angular_velocity.abs() > 0.1, "Steering in reverse should turn the car");

        // Transition from forward turning to stopping to reverse without steering
        let mut car_turn = Car::new(CarConfig::sports_car());
        let ctrl_fwd_turn = CarControls::new(0.8, 0.5, 0.0, false);
        for _ in 0..40 {
            car_turn.step(&ctrl_fwd_turn, SurfaceType::Asphalt, dt);
        }
        
        let ctrl_brake = CarControls::new(0.0, 0.0, 1.0, false);
        while car_turn.state.local_velocity.x > 0.25 {
            car_turn.step(&ctrl_brake, SurfaceType::Asphalt, dt);
        }
        let angle_at_stop = car_turn.state.angle;

        // Reversing with neutral steer maintains heading angle without drifting
        let mut ctrl_rev = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl_rev.reverse = true;
        for _ in 0..60 {
            car_turn.step(&ctrl_rev, SurfaceType::Asphalt, dt);
        }
        assert!((car_turn.state.angle - angle_at_stop).abs() < 0.01, "Car should reverse straight along stopped heading");
        assert!(car_turn.state.angular_velocity.abs() < 1e-3, "Angular velocity should settle to zero");

        // After turning in reverse, releasing steer straightens out trajectory
        let mut car_straighten = Car::new(CarConfig::sports_car());
        let mut ctrl_turn_rev = CarControls::new(1.0, 0.5, 0.0, false);
        ctrl_turn_rev.reverse = true;
        for _ in 0..60 {
            car_straighten.step(&ctrl_turn_rev, SurfaceType::Asphalt, dt);
        }
        let mut ctrl_neutral_rev = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl_neutral_rev.reverse = true;
        for _ in 0..60 {
            car_straighten.step(&ctrl_neutral_rev, SurfaceType::Asphalt, dt);
        }
        assert!(car_straighten.state.angular_velocity.abs() < 1e-3, "Releasing steering in reverse must eliminate yaw rate");
    }

    #[test]
    fn test_reverse_heading_stability_and_steering_symmetry() {
        let config = CarConfig::sports_car();
        let dt = 1.0 / 60.0;

        // 1. Reversing with arbitrary heading and zero steer maintains heading without limit-cycle chatter
        let mut car_straight = Car::new(config);
        car_straight.state.angle = -0.23337;
        let mut ctrl_neutral = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl_neutral.reverse = true;
        for _ in 0..60 {
            car_straight.step(&ctrl_neutral, SurfaceType::Asphalt, dt);
        }
        assert!((car_straight.state.angle - (-0.23337)).abs() < 1e-3, "Car must not turn involuntarily in reverse");
        assert!(car_straight.state.angular_velocity.abs() < 1e-3, "Reverse yaw rate must remain stable");

        // 2. Reversing with yaw disturbance damps smoothly to zero without sign oscillations
        let mut car_disturbed = Car::new(config);
        car_disturbed.state.angular_velocity = 0.05;
        for _ in 0..60 {
            car_disturbed.step(&ctrl_neutral, SurfaceType::Asphalt, dt);
        }
        assert!(car_disturbed.state.angular_velocity.abs() < 1e-3, "Reverse yaw disturbance must damp out smoothly");

        // 3. Symmetrical left and right steering in reverse
        let mut car_right = Car::new(config);
        let mut ctrl_right = CarControls::new(1.0, 0.5, 0.0, false);
        ctrl_right.reverse = true;
        for _ in 0..60 {
            car_right.step(&ctrl_right, SurfaceType::Asphalt, dt);
        }

        let mut car_left = Car::new(config);
        let mut ctrl_left = CarControls::new(1.0, -0.5, 0.0, false);
        ctrl_left.reverse = true;
        for _ in 0..60 {
            car_left.step(&ctrl_left, SurfaceType::Asphalt, dt);
        }

        assert!(car_right.state.angular_velocity.abs() > 0.1, "Right steering must produce yaw in reverse");
        assert!(car_left.state.angular_velocity.abs() > 0.1, "Left steering must produce yaw in reverse");
        assert!(
            (car_right.state.angular_velocity.abs() - car_left.state.angular_velocity.abs()).abs() < 1e-4,
            "Reverse steering must be perfectly symmetric"
        );
        assert!(
            (car_right.state.angle.abs() - car_left.state.angle.abs()).abs() < 1e-4,
            "Reverse turn angles must be perfectly symmetric"
        );
    }

    #[test]
    fn test_reverse_simulation_extended() {
        let mut config = CarConfig::sports_car();
        config.max_reverse_force = 6175.0; // GT3 evo reverse power
        config.mass = 1260.0;
        config.tire.stiffness_b = 13.0;
        config.tire.peak_d = 1.20;
        let dt = 1.0 / 60.0;
        let mut car = Car::new(config);
        let mut ctrl = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl.reverse = true;

        println!("\n=== SIMULATION: Neutral Steer in Reverse for 300 frames (5s) ===");
        for frame in 0..300 {
            car.step(&ctrl, SurfaceType::Asphalt, dt);
            if frame % 30 == 0 || frame == 299 {
                println!(
                    "frame={:3}: v_long={:6.2} v_lat={:6.3} speed={:5.2} angle={:7.4} omega={:7.4}",
                    frame,
                    car.state.local_velocity.x,
                    car.state.local_velocity.y,
                    car.state.speed,
                    car.state.angle,
                    car.state.angular_velocity
                );
            }
        }

        println!("\n=== SIMULATION: Disturbed Steer / Turn in Reverse ===");
        let mut car_turn = Car::new(config);
        // Start reverse with a tiny initial yaw rate 0.01 rad/s
        car_turn.state.angular_velocity = 0.01;
        for frame in 0..300 {
            car_turn.step(&ctrl, SurfaceType::Asphalt, dt);
            if frame % 30 == 0 || frame == 299 {
                println!(
                    "frame={:3}: v_long={:6.2} v_lat={:6.3} speed={:5.2} angle={:7.4} omega={:7.4}",
                    frame,
                    car_turn.state.local_velocity.x,
                    car_turn.state.local_velocity.y,
                    car_turn.state.speed,
                    car_turn.state.angle,
                    car_turn.state.angular_velocity
                );
            }
        }

        println!("\n=== SIMULATION: Reversing then steering right then trying to steer left ===");
        let mut car_steer = Car::new(config);
        let mut ctrl_r = CarControls::new(1.0, 0.5, 0.0, false);
        ctrl_r.reverse = true;
        // Turn right for 60 frames (1 sec)
        for _frame in 0..60 {
            car_steer.step(&ctrl_r, SurfaceType::Asphalt, dt);
        }
        println!(
            "After 1s right steer: v_long={:6.2} angle={:7.4} omega={:7.4}",
            car_steer.state.local_velocity.x,
            car_steer.state.angle,
            car_steer.state.angular_velocity
        );

        // Now steer left (-0.5) for 120 frames (2 sec) to turn the other way
        let mut ctrl_l = CarControls::new(1.0, -0.5, 0.0, false);
        ctrl_l.reverse = true;
        for frame in 0..120 {
            car_steer.step(&ctrl_l, SurfaceType::Asphalt, dt);
            if frame % 20 == 0 || frame == 119 {
                println!(
                    "steer left frame={:3}: v_long={:6.2} v_lat={:6.3} angle={:7.4} omega={:7.4}",
                    frame,
                    car_steer.state.local_velocity.x,
                    car_steer.state.local_velocity.y,
                    car_steer.state.angle,
                    car_steer.state.angular_velocity
                );
            }
        }

        println!("\n=== SIMULATION: Forward driving, brake to 0, then reverse with steer=0 ===");
        let mut car_fwd_rev = Car::new(config);
        // Drive forward for 2 seconds
        let ctrl_accel = CarControls::accelerate();
        for _ in 0..120 {
            car_fwd_rev.step(&ctrl_accel, SurfaceType::Asphalt, dt);
        }
        println!("After 2s accel: speed={:.2}, v_long={:.2}", car_fwd_rev.state.speed, car_fwd_rev.state.local_velocity.x);
        // Brake to full stop
        let ctrl_brake = CarControls::full_brake();
        let mut brake_frames = 0;
        while car_fwd_rev.state.local_velocity.x > 0.05 && brake_frames < 300 {
            car_fwd_rev.step(&ctrl_brake, SurfaceType::Asphalt, dt);
            brake_frames += 1;
        }
        println!("Stopped after {} brake frames: speed={:.3}, v_long={:.3}", brake_frames, car_fwd_rev.state.speed, car_fwd_rev.state.local_velocity.x);

        // Now reverse for 300 frames with steer=0
        let mut ctrl_rev = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl_rev.reverse = true;
        for frame in 0..300 {
            car_fwd_rev.step(&ctrl_rev, SurfaceType::Asphalt, dt);
            if frame % 30 == 0 || frame == 299 {
                println!(
                    "post-stop rev frame={:3}: v_long={:6.2} v_lat={:6.3} angle={:7.4} omega={:7.4}",
                    frame,
                    car_fwd_rev.state.local_velocity.x,
                    car_fwd_rev.state.local_velocity.y,
                    car_fwd_rev.state.angle,
                    car_fwd_rev.state.angular_velocity
                );
            }
        }

        println!("\n=== SIMULATION: Reversing while turning for 180 frames then reverse steering ===");
        let mut car_turn_swap = Car::new(config);
        let mut ctrl_turn1 = CarControls::new(1.0, 0.25, 0.0, false);
        ctrl_turn1.reverse = true;
        for frame in 0..180 {
            car_turn_swap.step(&ctrl_turn1, SurfaceType::Asphalt, dt);
            if frame % 60 == 0 || frame == 179 {
                println!(
                    "turn1 frame={:3}: v_long={:6.2} v_lat={:6.3} angle={:7.4} omega={:7.4}",
                    frame,
                    car_turn_swap.state.local_velocity.x,
                    car_turn_swap.state.local_velocity.y,
                    car_turn_swap.state.angle,
                    car_turn_swap.state.angular_velocity
                );
            }
        }

        // Now player steers the OTHER direction (-0.25)
        println!("--- Now steering opposite (-0.25) ---");
        let mut ctrl_turn2 = CarControls::new(1.0, -0.25, 0.0, false);
        ctrl_turn2.reverse = true;
        for frame in 0..180 {
            car_turn_swap.step(&ctrl_turn2, SurfaceType::Asphalt, dt);
            if frame % 30 == 0 || frame == 179 {
                println!(
                    "opposite frame={:3}: v_long={:6.2} v_lat={:6.3} angle={:7.4} omega={:7.4}",
                    frame,
                    car_turn_swap.state.local_velocity.x,
                    car_turn_swap.state.local_velocity.y,
                    car_turn_swap.state.angle,
                    car_turn_swap.state.angular_velocity
                );
            }
        }
    }

    #[test]
    fn test_reverse_drive_bias_distribution() {
        let sports = Car::new(CarConfig::sports_car()); // RWD: drive_bias = 0.0
        let rally = Car::new(CarConfig::rally_car());   // AWD: drive_bias = 0.5
        let dt = 1.0 / 60.0;

        let mut ctrl = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl.reverse = true;

        let mut sports_step = sports;
        sports_step.step(&ctrl, SurfaceType::Asphalt, dt);
        // Sports car RWD: front wheels have 0 longitudinal drive demand, rear wheels have full drive demand
        assert_eq!(sports_step.state.wheels[0].longitudinal_force, 0.0, "RWD front left wheel should have no drive force");
        assert_eq!(sports_step.state.wheels[1].longitudinal_force, 0.0, "RWD front right wheel should have no drive force");
        assert!(sports_step.state.wheels[2].longitudinal_force < 0.0, "RWD rear left wheel must have reverse drive force");
        assert!(sports_step.state.wheels[3].longitudinal_force < 0.0, "RWD rear right wheel must have reverse drive force");

        let mut rally_step = rally;
        rally_step.step(&ctrl, SurfaceType::Asphalt, dt);
        // Rally car AWD: all 4 wheels receive reverse drive torque
        assert!(rally_step.state.wheels[0].longitudinal_force < 0.0, "AWD front left wheel must receive reverse drive");
        assert!(rally_step.state.wheels[1].longitudinal_force < 0.0, "AWD front right wheel must receive reverse drive");
        assert!(rally_step.state.wheels[2].longitudinal_force < 0.0, "AWD rear left wheel must receive reverse drive");
        assert!(rally_step.state.wheels[3].longitudinal_force < 0.0, "AWD rear right wheel must receive reverse drive");
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

    #[test]
    fn test_step_with_sampler() {
        let mut car_uniform = Car::new(CarConfig::sports_car());
        let mut car_sampler = Car::new(CarConfig::sports_car());
        let sampler = crate::surface::UniformSurface(SurfaceType::Asphalt);
        let ctrl = CarControls::new(1.0, 0.1, 0.0, false);

        for _ in 0..60 {
            car_uniform.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
            car_sampler.step_with_sampler(&ctrl, &sampler, 1.0 / 60.0);
        }

        assert!((car_uniform.state.position - car_sampler.state.position).length() < 1e-4);
        assert!((car_uniform.state.speed - car_sampler.state.speed).abs() < 1e-4);
    }

    #[test]
    fn test_grade_slope_resistance() {
        let mut car_flat = Car::new(CarConfig::sports_car());
        let mut car_uphill = Car::new(CarConfig::sports_car());

        car_uphill.state.road_grade_slope = 0.12; // ~6.9 degrees uphill
        car_uphill.state.track_forward = Vec2::new(1.0, 0.0);

        let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
        let dt = 1.0 / 60.0;

        for _ in 0..120 {
            car_flat.step(&ctrl, SurfaceType::Asphalt, dt);
            car_uphill.step(&ctrl, SurfaceType::Asphalt, dt);
        }

        assert!(
            car_flat.state.speed > car_uphill.state.speed + 1.0,
            "Flat car speed ({}) should significantly exceed uphill car speed ({})",
            car_flat.state.speed,
            car_uphill.state.speed
        );
        assert!(
            car_flat.state.position.x > car_uphill.state.position.x + 2.0,
            "Flat car should travel farther than uphill car"
        );
    }

    #[test]
    fn test_crest_unloading() {
        let mut car_flat = Car::new(CarConfig::sports_car());
        let mut car_crest = Car::new(CarConfig::sports_car());

        car_flat.state.velocity = Vec2::new(35.0, 0.0);
        car_flat.state.speed = 35.0;

        car_crest.state.velocity = Vec2::new(35.0, 0.0);
        car_crest.state.speed = 35.0;
        car_crest.state.road_vertical_curvature = -0.006; // Crest: convex vertical curve

        let ctrl = CarControls::new(0.5, 0.0, 0.0, false);
        let dt = 1.0 / 60.0;

        car_flat.step(&ctrl, SurfaceType::Asphalt, dt);
        car_crest.step(&ctrl, SurfaceType::Asphalt, dt);

        let flat_load: f32 = car_flat.state.wheels.iter().map(|w| w.normal_load).sum();
        let crest_load: f32 = car_crest.state.wheels.iter().map(|w| w.normal_load).sum();

        assert!(
            crest_load < flat_load * 0.85,
            "Crest load ({}) should be significantly unloaded compared to flat load ({})",
            crest_load,
            flat_load
        );
    }

    #[test]
    fn test_is_braking_state_off_throttle_vs_braking() {
        let mut car = Car::new(CarConfig::sports_car());
        let dt = 1.0 / 60.0;

        // 1. Initial state: is_braking must be false
        assert!(!car.state.is_braking);

        // 2. Accelerate to high speed (~100 km/h)
        while car.speed_kmh() < 100.0 {
            car.step(&CarControls::accelerate(), SurfaceType::Asphalt, dt);
        }
        assert!(!car.state.is_braking, "Accelerating must not set is_braking");

        // 3. Off-throttle coasting (engine braking):
        // Slip ratio on rear wheel becomes negative from engine drag,
        // but is_braking MUST remain false!
        for _ in 0..60 {
            car.step(&CarControls::default(), SurfaceType::Asphalt, dt);
        }
        assert!(
            !car.state.is_braking,
            "Releasing throttle (off-throttle engine braking) must NOT set is_braking"
        );

        // 4. Active service brake: must set is_braking = true
        car.step(&CarControls::full_brake(), SurfaceType::Asphalt, dt);
        assert!(
            car.state.is_braking,
            "Applying service brake must set is_braking = true"
        );

        // 5. Release brake: must return to false
        car.step(&CarControls::default(), SurfaceType::Asphalt, dt);
        assert!(
            !car.state.is_braking,
            "Releasing service brake must return is_braking to false"
        );

        // 6. Handbrake: must set is_braking = true
        car.step(&CarControls::handbrake_turn(0.0), SurfaceType::Asphalt, dt);
        assert!(
            car.state.is_braking,
            "Handbrake must set is_braking = true"
        );
    }

    #[test]
    fn test_stopped_car_on_superelevated_segment_remains_static() {
        let mut car_asphalt = Car::new(CarConfig::sports_car());
        car_asphalt.state.road_bank_angle = 15.0; // 15 degrees banking
        car_asphalt.state.track_right = Vec2::new(0.0, 1.0); // Track right is +Y

        let ctrl = CarControls::default();
        let dt = 1.0 / 60.0;

        // Step physics multiple frames without control input
        for _ in 0..60 {
            car_asphalt.step(&ctrl, SurfaceType::Asphalt, dt);
        }

        // On asphalt (mu = 1.0, tan(15 deg) = 0.268), static friction must hold stopped car completely static!
        assert_eq!(
            car_asphalt.state.velocity,
            Vec2::ZERO,
            "Stopped car on 15 deg banked asphalt must remain completely static, got {:?}",
            car_asphalt.state.velocity
        );
        assert_eq!(
            car_asphalt.state.speed,
            0.0,
            "Stopped car speed must be 0.0"
        );

        // On ice (mu = 0.08, tan(15 deg) = 0.268 > 0.08), static friction is exceeded, so it slides downhill (-Y)
        let mut car_ice = Car::new(CarConfig::sports_car());
        car_ice.state.road_bank_angle = 15.0;
        car_ice.state.track_right = Vec2::new(0.0, 1.0);
        for _ in 0..10 {
            car_ice.step(&ctrl, SurfaceType::SheetIce, dt);
        }
        assert!(
            car_ice.state.velocity.y < 0.0,
            "Car on icy banking exceeding friction limit must slide downhill (-Y), got {:?}",
            car_ice.state.velocity
        );
    }

    #[test]
    fn test_terrain_interaction_flotation_and_ice_studs() {
        let dt = 1.0 / 60.0;
        let ctrl = CarControls::accelerate();

        // 1. Sand Flotation: Sand Rail Buggy (gamma = 0.30) vs Sports Car (gamma = 1.00) on PackedSand
        let mut buggy = Car::new(CarConfig::sand_rail());
        let mut sports = Car::new(CarConfig::sports_car());

        for _ in 0..120 {
            buggy.step(&ctrl, SurfaceType::PackedSand, dt);
            sports.step(&ctrl, SurfaceType::PackedSand, dt);
        }

        assert!(
            buggy.speed_kmh() > sports.speed_kmh() * 1.3,
            "Sand Rail Buggy speed ({:.1} km/h) should exceed Sports Car ({:.1} km/h) by at least 30% on PackedSand",
            buggy.speed_kmh(),
            sports.speed_kmh()
        );

        // 2. Studded Ice Racer (alpha = 8.125) vs Unstudded Sports Car (alpha = 1.00) on SheetIce
        let mut ice_racer_cfg = CarConfig::sports_car();
        ice_racer_cfg.terrain.ice_grip_multiplier = 8.125;
        let mut ice_racer = Car::new(ice_racer_cfg);
        let mut unstudded = Car::new(CarConfig::sports_car());

        let steer_ctrl = CarControls {
            throttle: 0.5,
            steer: 0.5,
            brake: 0.0,
            handbrake: false,
            reverse: false,
        };
        ice_racer.state.velocity = Vec2::new(50.0 / 3.6, 0.0);
        ice_racer.state.speed = 50.0 / 3.6;
        unstudded.state.velocity = Vec2::new(50.0 / 3.6, 0.0);
        unstudded.state.speed = 50.0 / 3.6;

        for _ in 0..60 {
            ice_racer.step(&steer_ctrl, SurfaceType::SheetIce, dt);
            unstudded.step(&steer_ctrl, SurfaceType::SheetIce, dt);
        }

        assert!(
            ice_racer.state.angular_velocity.abs() > unstudded.state.angular_velocity.abs() * 1.8,
            "Studded ice racer yaw rate ({:.2}) must exceed unstudded ({:.2}) by at least 1.8x",
            ice_racer.state.angular_velocity.abs(),
            unstudded.state.angular_velocity.abs()
        );
    }
}

