use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::car::normalize_angle;
use super::config::TireConfig;
use super::surface::{SurfaceSampler, SurfaceType};
use super::tire::{pacejka_lateral_force, solve_combined_slip_forces};

/// Configuration parameters for a 2-wheel single-track motorcycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotorbikeConfig {
    /// Total curb mass including typical rider in kilograms.
    pub mass: f32,
    /// Distance between front and rear axle contact points in meters (the wheelbase).
    pub wheelbase_len: f32,
    /// Center of gravity height above ground in meters.
    pub cog_height: f32,
    /// Static weight distribution fraction on front wheel [0.0, 1.0] (typically ~0.50).
    pub front_weight_ratio: f32,
    /// Maximum achievable lean angle before footpegs/fairings contact track (radians).
    pub max_lean_rad: f32,
    /// Roll/lean response rate (radians per second per steering unit).
    pub lean_rate: f32,
    /// Camber thrust stiffness coefficient (C_gamma): lateral force per radian of lean.
    pub camber_stiffness: f32,
    /// Maximum engine drive force on rear tire in Newtons.
    pub max_drive_force: f32,
    /// Maximum braking force on front wheel in Newtons.
    pub max_brake_front: f32,
    /// Maximum braking force on rear wheel in Newtons.
    pub max_brake_rear: f32,
    /// Rolling resistance coefficient.
    pub rolling_resistance: f32,
    /// Aerodynamic drag coefficient.
    pub drag_coefficient: f32,
    /// Tire parameters for Pacejka slip solver.
    pub tire: TireConfig,
}

impl Default for MotorbikeConfig {
    fn default() -> Self {
        Self::superbike()
    }
}

impl MotorbikeConfig {
    /// Preset: High-performance 1000cc Superbike.
    pub fn superbike() -> Self {
        Self {
            mass: 195.0,
            wheelbase_len: 1.42,
            cog_height: 0.52,
            front_weight_ratio: 0.52,
            max_lean_rad: 58.0f32.to_radians(),
            lean_rate: 6.5,
            camber_stiffness: 1200.0,
            max_drive_force: 4200.0,
            max_brake_front: 3800.0,
            max_brake_rear: 1500.0,
            rolling_resistance: 18.0,
            drag_coefficient: 0.38,
            tire: TireConfig::default(),
        }
    }

    /// Preset: Lightweight 450cc Motocross / Dirt Bike.
    pub fn motocross() -> Self {
        Self {
            mass: 115.0,
            wheelbase_len: 1.48,
            cog_height: 0.62,
            front_weight_ratio: 0.48,
            max_lean_rad: 50.0f32.to_radians(),
            lean_rate: 8.0,
            camber_stiffness: 900.0,
            max_drive_force: 3100.0,
            max_brake_front: 2400.0,
            max_brake_rear: 1800.0,
            rolling_resistance: 24.0,
            drag_coefficient: 0.45,
            tire: TireConfig::default(),
        }
    }
}

/// Control inputs for operating a single-track motorbike.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotorbikeControls {
    /// Throttle [0.0 = idle, 1.0 = full gas].
    pub throttle: f32,
    /// Steer direction [-1.0 = full left, +1.0 = full right].
    pub steer: f32,
    /// Front service brake lever [0.0 = release, 1.0 = full brake].
    pub brake_front: f32,
    /// Rear foot brake pedal [0.0 = release, 1.0 = full brake].
    pub brake_rear: f32,
    /// Rider body weight lean shift [-1.0 = hang-off left, +1.0 = hang-off right].
    pub rider_lean: f32,
}

impl Default for MotorbikeControls {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            steer: 0.0,
            brake_front: 0.0,
            brake_rear: 0.0,
            rider_lean: 0.0,
        }
    }
}

/// Dynamic kinematic and telemetry state of a single-track motorcycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotorbikeState {
    /// 2D chassis center position in world space (meters).
    pub position: Vec2,
    /// 2D linear velocity in world space (m/s).
    pub velocity: Vec2,
    /// Chassis yaw orientation in radians.
    pub angle: f32,
    /// Yaw rate in radians/second.
    pub angular_velocity: f32,
    /// Chassis roll/lean angle in radians (+ = leaning right, - = leaning left).
    pub lean_angle: f32,
    /// Forward velocity along chassis centerline (m/s).
    pub speed: f32,
    /// Front wheel normal load in Newtons.
    pub front_load: f32,
    /// Rear wheel normal load in Newtons.
    pub rear_load: f32,
    /// Whether front wheel is airborne due to acceleration pitch (Wheelie).
    pub is_wheelie: bool,
    /// Whether rear wheel is airborne due to hard braking pitch (Stoppie / Endo).
    pub is_stoppie: bool,
    /// Whether the motorcycle has lost grip and crashed into a lowside slide.
    pub is_lowside: bool,
    /// Whether the motorcycle has violently snapped into a highside flip.
    pub is_highside: bool,
}

impl Default for MotorbikeState {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            angle: 0.0,
            angular_velocity: 0.0,
            lean_angle: 0.0,
            speed: 0.0,
            front_load: 0.0,
            rear_load: 0.0,
            is_wheelie: false,
            is_stoppie: false,
            is_lowside: false,
            is_highside: false,
        }
    }
}

/// 2-Wheel Single-Track Motorcycle Simulation Model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Motorbike {
    pub config: MotorbikeConfig,
    pub state: MotorbikeState,
}

impl Motorbike {
    pub fn new(config: MotorbikeConfig) -> Self {
        let g = 9.81;
        let total_weight = config.mass * g;
        let front_load = total_weight * config.front_weight_ratio;
        let rear_load = total_weight * (1.0 - config.front_weight_ratio);

        Self {
            config,
            state: MotorbikeState {
                front_load,
                rear_load,
                ..Default::default()
            },
        }
    }

    pub fn with_pose(mut self, position: Vec2, angle: f32) -> Self {
        self.state.position = position;
        self.state.angle = normalize_angle(angle);
        self
    }

    /// Forward unit direction vector of the chassis.
    #[inline]
    pub fn forward_vector(&self) -> Vec2 {
        Vec2::new(self.state.angle.cos(), self.state.angle.sin())
    }

    /// Lateral right unit direction vector of the chassis.
    #[inline]
    pub fn right_vector(&self) -> Vec2 {
        Vec2::new(self.state.angle.sin(), -self.state.angle.cos())
    }

    /// Advances the single-track motorcycle simulation by fixed timestep `dt`.
    pub fn step(&mut self, controls: &MotorbikeControls, surface: SurfaceType, dt: f32) {
        if self.state.is_lowside || self.state.is_highside {
            // Sliding along ground after crash
            self.state.velocity *= (1.0 - 2.5 * dt).max(0.0);
            self.state.position += self.state.velocity * dt;
            self.state.speed = self.state.velocity.length();
            return;
        }

        let g = 9.81;
        let fwd = self.forward_vector();
        let right = self.right_vector();

        let v_long = self.state.velocity.dot(fwd);
        let v_lat = self.state.velocity.dot(right);
        let speed = self.state.velocity.length();
        self.state.speed = speed;

        // 1. Lean angle dynamics: roll into corner based on steering and speed
        let target_lean = (controls.steer * self.config.max_lean_rad + controls.rider_lean * 0.2)
            .clamp(-self.config.max_lean_rad, self.config.max_lean_rad);
        let lean_diff = target_lean - self.state.lean_angle;
        self.state.lean_angle += lean_diff * (self.config.lean_rate * dt).min(1.0);

        // 2. Drive and Brake forces
        let drive_force = if !self.state.is_wheelie {
            controls.throttle * self.config.max_drive_force
        } else {
            // Cut power slightly when balance point exceeded to avoid continuous loop-out
            controls.throttle * self.config.max_drive_force * 0.6
        };

        let brake_front = controls.brake_front * self.config.max_brake_front;
        let brake_rear = controls.brake_rear * self.config.max_brake_rear;
        let net_brake = brake_front + brake_rear;

        // 3. Dynamic longitudinal load transfer (pitch)
        let accel_est = (drive_force - net_brake) / self.config.mass;
        let delta_load = (self.config.mass * accel_est * self.config.cog_height) / self.config.wheelbase_len;

        let total_weight = self.config.mass * g;
        let raw_front = total_weight * self.config.front_weight_ratio - delta_load;
        let raw_rear = total_weight * (1.0 - self.config.front_weight_ratio) + delta_load;

        self.state.front_load = raw_front.max(0.0);
        self.state.rear_load = raw_rear.max(0.0);

        self.state.is_wheelie = raw_front <= 0.0 && drive_force > 0.0;
        self.state.is_stoppie = raw_rear <= 0.0 && brake_front > 0.0;

        // 4. Lateral force generation: Slip angle + Camber thrust from lean
        let friction_mu = surface.friction_coefficient();
        let front_camber_force = self.config.camber_stiffness * self.state.lean_angle;
        let rear_camber_force = self.config.camber_stiffness * self.state.lean_angle * 0.8;

        let slip_angle_front = (-v_lat / v_long.abs().max(1.0)) + controls.steer * 0.15;
        let slip_angle_rear = -v_lat / v_long.abs().max(1.0);

        let fy_front = pacejka_lateral_force(
            slip_angle_front,
            self.state.front_load,
            friction_mu,
            &self.config.tire,
            false,
        ) + front_camber_force;

        let fy_rear = pacejka_lateral_force(
            slip_angle_rear,
            self.state.rear_load,
            friction_mu,
            &self.config.tire,
            false,
        ) + rear_camber_force;

        // Clamp using combined slip friction circle
        let (fx_front, fy_front_clamped) = solve_combined_slip_forces(-brake_front, fy_front, self.state.front_load * friction_mu);
        let (fx_rear, fy_rear_clamped) = solve_combined_slip_forces(drive_force - brake_rear, fy_rear, self.state.rear_load * friction_mu);

        // 5. Yaw torque and integration
        let aero_drag = 0.5 * self.config.drag_coefficient * speed * speed;
        let total_fx = fx_front + fx_rear - aero_drag;
        let total_fy = fy_front_clamped + fy_rear_clamped;

        // Yaw acceleration: single-track wheelbase distance
        let l_front = self.config.wheelbase_len * (1.0 - self.config.front_weight_ratio);
        let l_rear = self.config.wheelbase_len * self.config.front_weight_ratio;
        let yaw_torque = fy_front_clamped * l_front - fy_rear_clamped * l_rear;
        let yaw_inertia = (1.0 / 12.0) * self.config.mass * (self.config.wheelbase_len * self.config.wheelbase_len);

        self.state.angular_velocity += (yaw_torque / yaw_inertia) * dt;
        self.state.angular_velocity *= 1.0 - (4.0 * dt).min(1.0); // Natural damping
        self.state.angle = normalize_angle(self.state.angle + self.state.angular_velocity * dt);

        // Velocity integration
        let accel_world = fwd * total_fx + right * total_fy;
        self.state.velocity += (accel_world / self.config.mass) * dt;
        self.state.position += self.state.velocity * dt;

        // 6. Crash detection: Lowside (excessive lean while sliding)
        if self.state.lean_angle.abs() > self.config.max_lean_rad * 0.95 && slip_angle_rear.abs() > 0.45 {
            self.state.is_lowside = true;
        }
    }

    /// Advances using an arbitrary terrain `SurfaceSampler`.
    pub fn step_with_sampler<S: SurfaceSampler>(
        &mut self,
        controls: &MotorbikeControls,
        sampler: &S,
        dt: f32,
    ) {
        let props = sampler.sample_surface(self.state.position);
        self.step(controls, props.surface_type, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motorbike_initialization() {
        let bike = Motorbike::new(MotorbikeConfig::superbike());
        assert!(bike.state.front_load > 0.0);
        assert!(bike.state.rear_load > 0.0);
        assert!(!bike.state.is_wheelie);
        assert!(!bike.state.is_stoppie);
    }

    #[test]
    fn test_motorbike_acceleration_and_wheelie() {
        let mut bike = Motorbike::new(MotorbikeConfig::superbike());
        let ctrl = MotorbikeControls {
            throttle: 1.0,
            ..Default::default()
        };

        for _ in 0..120 {
            bike.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
        }

        assert!(bike.state.speed > 10.0, "Bike should accelerate forward");
        assert!(bike.state.rear_load > bike.state.front_load, "Rear load should exceed front load under power");
    }

    #[test]
    fn test_motorbike_lean_and_camber_cornering() {
        let mut bike = Motorbike::new(MotorbikeConfig::superbike());
        let ctrl = MotorbikeControls {
            throttle: 0.5,
            steer: 0.5, // Turn right
            ..Default::default()
        };

        for _ in 0..60 {
            bike.step(&ctrl, SurfaceType::Asphalt, 1.0 / 60.0);
        }

        assert!(bike.state.lean_angle > 0.1, "Bike should lean into right turn");
    }
}
