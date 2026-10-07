//! 4-wheel top-down vehicle dynamics.
//!
//! Governed by `specs/043_vehicle_dynamics_rebuild_and_simplified_handling_settings.md`:
//! slip-based tire forces, implicit wheel spin and differential coupling, roll-balance load
//! transfer, grip-aware steering authority for human drivers, and continuous assists.

use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use super::config::{CarConfig, DifferentialType, EnginePlacement, SuspensionArchetype};
use super::surface::{CompoundId, SurfaceSampler, SurfaceType};
use super::tire::{
    combined_slip_forces, combined_slip_fx, compute_skid_telemetry, WheelAssembly, WheelId,
    WheelTelemetry,
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

fn one_f32() -> f32 {
    1.0
}

/// Steering overslip cap for bots and scripted controllers (linear mapping, see `step_per_wheel`).
pub const BOT_STEER_OVERSLIP: f32 = 1.5;

/// Collision energy (J) the bumpers absorb elastically before any structural damage (Spec 078).
pub const DAMAGE_ENERGY_DEADZONE_J: f32 = 500.0;
/// Collision energy (J) above the dead zone that takes chassis health from 1.0 to 0.0 (Spec 078).
pub const CHASSIS_DAMAGE_CAPACITY_J: f32 = 10000.0;
/// Collision energy (J) above the dead zone that takes engine health from 1.0 to 0.0 (Spec 078).
pub const ENGINE_DAMAGE_CAPACITY_J: f32 = 5000.0;
/// Collision energy (J) that destroys one suspension corner, before the archetype robustness factor (Spec 078).
pub const SUSPENSION_DAMAGE_CAPACITY_J: f32 = 4500.0;
/// Damper compression speed (m/s) above which hitting the bump stop damages the corner (Spec 078).
pub const KERB_BOTTOM_OUT_SPEED_MPS: f32 = 1.8;
/// Excess bump-stop energy (J) that destroys one corner, before the robustness factor (Spec 078).
pub const KERB_BOTTOM_OUT_CAPACITY_J: f32 = 600.0;
/// Vertical touchdown speed (m/s) above which a jump landing damages the suspension (Spec 078).
pub const LANDING_SPEED_LIMIT_MPS: f32 = 4.2;
/// Excess landing energy (J) that destroys one corner, before the robustness factor (Spec 078).
pub const LANDING_CAPACITY_J: f32 = 2500.0;
/// A pushrod corner below this health collapses onto its bump stop (Spec 078).
pub const PUSHROD_COLLAPSE_HEALTH: f32 = 0.30;
/// Aerodynamic drag multiplier while a pushrod corner is collapsed (Spec 078).
pub const PUSHROD_COLLAPSE_DRAG_MULTIPLIER: f32 = 1.40;
/// Highest chassis health a field (pit) repair can restore (Spec 078).
pub const FIELD_REPAIR_CHASSIS_CAP: f32 = 0.70;
/// Highest engine health a field (pit) repair can restore (Spec 078).
pub const FIELD_REPAIR_ENGINE_CAP: f32 = 0.70;
/// Highest suspension corner health a field (pit) repair can restore (Spec 078).
pub const FIELD_REPAIR_SUSPENSION_CAP: f32 = 0.60;

/// Traction help starts to ease the throttle at this rear-axle limit use, and reaches its full
/// cut `TH_WIDTH` later.
const TH_START: f32 = 0.80;
const TH_WIDTH: f32 = 0.20;
/// Lateral acceleration (m/s^2, speed x yaw rate) at which traction help acts fully. Below it (and
/// below `TH_CORNERING_STEER`) the cut fades out, so straight-line drive is never eased.
const TH_CORNERING_ACCEL: f32 = 4.0;
/// Steering input at which traction help acts fully.
const TH_CORNERING_STEER: f32 = 0.25;

/// Fast tanh approximation with saturation clipping past |x| >= 4.0 (Spec 084).
#[inline]
fn fast_tanh_clip(x: f32) -> f32 {
    if x >= 4.0 {
        1.0
    } else if x <= -4.0 {
        -1.0
    } else {
        x.tanh()
    }
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

/// Telemetry record for an individual suspension corner.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SuspensionTelemetry {
    /// Instantaneous suspension deflection (meters, >0 = bump compression, <0 = rebound).
    pub deflection: f32,
    /// Instantaneous deflection velocity (m/s).
    pub deflection_velocity: f32,
    /// Total spring + damping + ARB vertical normal force (Newtons).
    pub normal_force: f32,
    /// Dynamic wheel inclination angle under roll (radians).
    pub dynamic_camber: f32,
    /// True if suspension reached max bump travel this tick (bottomed out).
    pub bottomed_out: bool,
}

/// Discrete impact zone resolved from local collision coordinates (Spec 078).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactZone {
    FrontNose,
    RearTail,
    FlankLeft,
    FlankRight,
    CornerFL,
    CornerFR,
    CornerRL,
    CornerRR,
}

impl ImpactZone {
    pub const ALL: [Self; 8] = [
        Self::FrontNose,
        Self::RearTail,
        Self::FlankLeft,
        Self::FlankRight,
        Self::CornerFL,
        Self::CornerFR,
        Self::CornerRL,
        Self::CornerRR,
    ];

    /// Share of collision energy sent to `(chassis, engine, [suspension FL, FR, RL, RR])` for a hit
    /// in this zone on a car with this engine placement (Spec 078).
    pub const fn damage_weights(self, placement: EnginePlacement) -> (f32, f32, [f32; 4]) {
        match (self, placement) {
            (ImpactZone::FrontNose, EnginePlacement::FrontEngine) => (0.20, 0.70, [0.05, 0.05, 0.0, 0.0]),
            (ImpactZone::FrontNose, EnginePlacement::MidEngine)   => (0.65, 0.05, [0.15, 0.15, 0.0, 0.0]),
            (ImpactZone::FrontNose, EnginePlacement::RearEngine)  => (0.70, 0.00, [0.15, 0.15, 0.0, 0.0]),

            (ImpactZone::RearTail, EnginePlacement::FrontEngine)  => (0.75, 0.05, [0.0, 0.0, 0.10, 0.10]),
            (ImpactZone::RearTail, EnginePlacement::MidEngine)    => (0.45, 0.40, [0.0, 0.0, 0.075, 0.075]),
            (ImpactZone::RearTail, EnginePlacement::RearEngine)   => (0.15, 0.75, [0.0, 0.0, 0.05, 0.05]),

            (ImpactZone::FlankLeft, EnginePlacement::FrontEngine) => (0.60, 0.15, [0.125, 0.0, 0.125, 0.0]),
            (ImpactZone::FlankLeft, EnginePlacement::MidEngine)   => (0.40, 0.45, [0.075, 0.0, 0.075, 0.0]),
            (ImpactZone::FlankLeft, EnginePlacement::RearEngine)  => (0.50, 0.30, [0.10, 0.0, 0.10, 0.0]),

            (ImpactZone::FlankRight, EnginePlacement::FrontEngine)=> (0.60, 0.15, [0.0, 0.125, 0.0, 0.125]),
            (ImpactZone::FlankRight, EnginePlacement::MidEngine)  => (0.40, 0.45, [0.0, 0.075, 0.0, 0.075]),
            (ImpactZone::FlankRight, EnginePlacement::RearEngine) => (0.50, 0.30, [0.0, 0.10, 0.0, 0.10]),

            (ImpactZone::CornerFL, EnginePlacement::FrontEngine)  => (0.25, 0.15, [0.60, 0.0, 0.0, 0.0]),
            (ImpactZone::CornerFL, EnginePlacement::MidEngine)    => (0.35, 0.00, [0.65, 0.0, 0.0, 0.0]),
            (ImpactZone::CornerFL, EnginePlacement::RearEngine)   => (0.35, 0.00, [0.65, 0.0, 0.0, 0.0]),

            (ImpactZone::CornerFR, EnginePlacement::FrontEngine)  => (0.25, 0.15, [0.0, 0.60, 0.0, 0.0]),
            (ImpactZone::CornerFR, EnginePlacement::MidEngine)    => (0.35, 0.00, [0.0, 0.65, 0.0, 0.0]),
            (ImpactZone::CornerFR, EnginePlacement::RearEngine)   => (0.35, 0.00, [0.0, 0.65, 0.0, 0.0]),

            (ImpactZone::CornerRL, EnginePlacement::FrontEngine)  => (0.40, 0.05, [0.0, 0.0, 0.55, 0.0]),
            (ImpactZone::CornerRL, EnginePlacement::MidEngine)    => (0.25, 0.25, [0.0, 0.0, 0.50, 0.0]),
            (ImpactZone::CornerRL, EnginePlacement::RearEngine)   => (0.15, 0.35, [0.0, 0.0, 0.50, 0.0]),

            (ImpactZone::CornerRR, EnginePlacement::FrontEngine)  => (0.40, 0.05, [0.0, 0.0, 0.0, 0.55]),
            (ImpactZone::CornerRR, EnginePlacement::MidEngine)    => (0.25, 0.25, [0.0, 0.0, 0.0, 0.50]),
            (ImpactZone::CornerRR, EnginePlacement::RearEngine)   => (0.15, 0.35, [0.0, 0.0, 0.0, 0.50]),
        }
    }
}

const fn default_suspension_health() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
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
    /// Assist reductions for this physics step, expressed as multipliers applied to drive force.
    #[serde(default = "one_f32")]
    pub traction_help_multiplier: f32,
    #[serde(default = "one_f32")]
    pub tcs_lateral_multiplier: f32,
    #[serde(default = "one_f32")]
    pub tcs_longitudinal_multiplier: f32,
    /// Absolute corrective ESC yaw torque applied during the previous step (N·m).
    #[serde(default)]
    pub esc_corrective_torque: f32,
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
    /// Dynamic vehicle body roll angle in radians (+ = rolled right, - = rolled left).
    #[serde(default)]
    pub roll_angle: f32,
    /// Dynamic vehicle body pitch angle in radians (+ = pitch up/squat, - = pitch down/dive).
    #[serde(default)]
    pub pitch_angle: f32,
    /// Detailed suspension telemetry for each corner [FL, FR, RL, RR].
    #[serde(default)]
    pub suspension: [SuspensionTelemetry; 4],
    /// Ground elevation under each wheel contact patch in meters (z >= 0.0).
    #[serde(default)]
    pub wheel_elevations: [f32; 4],
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
    /// Vehicle chassis structural health [0.0 = destroyed, 1.0 = pristine] (Spec 063).
    #[serde(default = "one_f32")]
    pub health: f32,
    /// Structural health of chassis frame [0.0 = wrecked, 1.0 = pristine] (Spec 078).
    #[serde(default = "one_f32")]
    pub chassis_health: f32,
    /// Mechanical health of engine & cooling block [0.0 = blown, 1.0 = pristine] (Spec 078).
    #[serde(default = "one_f32")]
    pub engine_health: f32,
    /// 4-corner suspension health [FL, FR, RL, RR] [0.0 = destroyed, 1.0 = pristine] (Spec 078).
    #[serde(default = "default_suspension_health")]
    pub suspension_health: [f32; 4],
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
            traction_help_multiplier: 1.0,
            tcs_lateral_multiplier: 1.0,
            tcs_longitudinal_multiplier: 1.0,
            esc_corrective_torque: 0.0,
            is_braking: false,
            road_elevation: 0.0,
            ramp_elevation: 0.0,
            road_bank_angle: 0.0,
            track_right: Vec2::ZERO,
            road_grade_slope: 0.0,
            road_vertical_curvature: 0.0,
            track_forward: Vec2::ZERO,
            roll_angle: 0.0,
            pitch_angle: 0.0,
            suspension: [SuspensionTelemetry::default(); 4],
            wheel_elevations: [0.0; 4],
            elevation: 0.0,
            vertical_velocity: 0.0,
            is_airborne: false,
            air_time: 0.0,
            last_air_time: 0.0,
            jump_count: 0,
            just_landed: false,
            draft_intensity: 0.0,
            health: 1.0,
            chassis_health: 1.0,
            engine_health: 1.0,
            suspension_health: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// 4-Wheel Top-Down Arcade Vehicle Physics Model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Car {
    pub config: CarConfig,
    pub state: CarState,
    /// Runtime-only input source for recovery tuning. Not part of replay/LAN car controls.
    #[serde(skip)]
    pub digital_steering_source: bool,
    /// Transient intentional flick state. Runtime-only; fixed-step deterministic.
    #[serde(skip)]
    previous_control_steer: f32,
    #[serde(skip)]
    flick_headroom_remaining_s: f32,
    #[serde(skip)]
    flick_rearm_remaining_s: f32,
}

impl Car {
    /// Creates a new car instance with the given configuration at the origin.
    pub fn new(mut config: CarConfig) -> Self {
        config.finalize();
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
            digital_steering_source: false,
            previous_control_steer: 0.0,
            flick_headroom_remaining_s: 0.0,
            flick_rearm_remaining_s: 0.0,
        }
    }

    /// Resets mechanical tread wear across all wheels to 0.0.
    pub fn service_tires(&mut self) {
        for w in &mut self.state.wheels {
            w.wear = 0.0;
        }
        for a in &mut self.state.wheel_assemblies {
            a.wear = 0.0;
        }
    }

    /// Restores chassis health (capped at 0.70 in-race ceiling), engine health (capped at 0.70),
    /// and 4-corner suspension health (capped at 0.60) (Spec 078). Returns actual chassis health restored.
    pub fn apply_field_repair(&mut self, amount: f32) -> f32 {
        let old_health = self.state.chassis_health.min(self.state.health);
        self.state.chassis_health = old_health.max((old_health + amount).min(FIELD_REPAIR_CHASSIS_CAP));
        self.state.health = self.state.chassis_health;
        self.state.engine_health = self.state.engine_health.max((self.state.engine_health + amount).min(FIELD_REPAIR_ENGINE_CAP));
        for s in &mut self.state.suspension_health {
            *s = (*s).max((*s + amount).min(FIELD_REPAIR_SUSPENSION_CAP));
        }
        self.state.chassis_health - old_health
    }

    /// Complete post-race garage repair restoring all vehicle components to 100% health (Spec 078).
    pub fn full_garage_repair(&mut self) {
        self.state.chassis_health = 1.0;
        self.state.health = 1.0;
        self.state.engine_health = 1.0;
        self.state.suspension_health = [1.0, 1.0, 1.0, 1.0];
    }

    /// Classifies contact point in world coordinates into a discrete vehicle impact zone (Spec 078).
    pub fn classify_impact_zone(&self, contact_point_world: Vec2) -> ImpactZone {
        let delta = contact_point_world - self.state.position;
        let x_local = delta.dot(self.forward_vector());
        let y_local = delta.dot(self.right_vector());
        self.classify_impact_zone_local(x_local, y_local)
    }

    /// Classifies vehicle local coordinates (x = forward, y = right) into an impact zone (Spec 078).
    pub fn classify_impact_zone_local(&self, x_local: f32, y_local: f32) -> ImpactZone {
        let lf = self.config.cg_to_front;
        let lr = self.config.cg_to_rear;
        let w_half = self.config.chassis.half_width().max(self.config.track_width * 0.5);
        let corner_lat_threshold = 0.55 * w_half;

        if y_local.abs() > corner_lat_threshold {
            // Lateral outer boundary: either corner wheel or flank
            if x_local >= (lf - 0.40) {
                if y_local > 0.0 {
                    ImpactZone::CornerFR
                } else {
                    ImpactZone::CornerFL
                }
            } else if x_local <= (-lr + 0.40) {
                if y_local > 0.0 {
                    ImpactZone::CornerRR
                } else {
                    ImpactZone::CornerRL
                }
            } else if y_local > 0.0 {
                ImpactZone::FlankRight
            } else {
                ImpactZone::FlankLeft
            }
        } else {
            // Central width: Front nose or rear tail
            if x_local >= 0.0 {
                ImpactZone::FrontNose
            } else {
                ImpactZone::RearTail
            }
        }
    }

    /// Applies collision impact damage partitioned by impact zone and engine placement (Spec 078).
    pub fn apply_collision_damage(&mut self, contact_point_world: Vec2, damage_energy: f32) -> ImpactZone {
        let zone = self.classify_impact_zone(contact_point_world);
        self.apply_collision_damage_to_zone(zone, damage_energy);
        zone
    }

    /// Partitions collision damage energy into chassis, engine, and 4-corner suspension health (Spec 078).
    pub fn apply_collision_damage_to_zone(&mut self, zone: ImpactZone, damage_energy: f32) {
        if !self.config.damage_enabled || damage_energy <= 0.0 {
            return;
        }

        // Bumper elastic deformation absorption deadzone.
        // Low-speed bumper taps and glancing blows are absorbed elastically without structural deformation.
        let effective_energy = (damage_energy - DAMAGE_ENERGY_DEADZONE_J).max(0.0);
        if effective_energy <= 0.0 {
            return;
        }

        let (w_chassis, w_engine, w_susp) = zone.damage_weights(self.config.engine_placement);

        let delta_chassis = (effective_energy * w_chassis) / CHASSIS_DAMAGE_CAPACITY_J;
        let delta_engine = (effective_energy * w_engine) / ENGINE_DAMAGE_CAPACITY_J;

        self.state.chassis_health = (self.state.chassis_health - delta_chassis).clamp(0.0, 1.0);
        self.state.health = self.state.chassis_health;
        self.state.engine_health = (self.state.engine_health - delta_engine).clamp(0.0, 1.0);

        for (i, &w_s) in w_susp.iter().enumerate() {
            if w_s > 0.0 {
                let corner_cfg = if i < 2 {
                    self.config.suspension.front
                } else {
                    self.config.suspension.rear
                };
                let k_rob = corner_cfg.archetype.robustness_factor();
                let delta_susp = (effective_energy * w_s) / (SUSPENSION_DAMAGE_CAPACITY_J * k_rob);
                self.state.suspension_health[i] = (self.state.suspension_health[i] - delta_susp).clamp(0.0, 1.0);
            }
        }
    }

    /// Steering pull bias from asymmetric front suspension damage in driver control units (Spec 078).
    /// Returns negative value when front-left is damaged (pulling left), positive when front-right is damaged.
    /// Incorporates a 10% deadzone so minor wear or small discrepancies do not impair steering.
    pub fn steering_pull_bias(&self) -> f32 {
        let delta = self.state.suspension_health[0] - self.state.suspension_health[1];
        if delta.abs() <= 0.10 {
            0.0
        } else {
            let excess = (delta.abs() - 0.10) / 0.90;
            delta.signum() * excess * 0.087
        }
    }

    /// Engine power attenuation ratio as a function of engine health (Spec 078 Section 5.D).
    /// Preserves 100% full engine horsepower when engine health >= 80%.
    pub fn available_engine_power_ratio(&self) -> f32 {
        let h = self.state.engine_health.clamp(0.0, 1.0);
        if h >= 0.80 {
            1.0
        } else {
            0.40 + 0.60 * (h / 0.80).powf(1.3)
        }
    }

    /// Effective aerodynamic drag coefficient including draft reduction and pushrod failure drag penalty (Spec 078).
    pub fn effective_air_drag_coefficient(&self) -> f32 {
        let has_collapsed_pushrod = self.state.suspension_health.iter().enumerate().any(|(i, &h)| {
            let arch = if i < 2 {
                self.config.suspension.front.archetype
            } else {
                self.config.suspension.rear.archetype
            };
            arch == SuspensionArchetype::PushrodInboard && h < PUSHROD_COLLAPSE_HEALTH
        });
        let drag_mult = if has_collapsed_pushrod { PUSHROD_COLLAPSE_DRAG_MULTIPLIER } else { 1.0 };
        self.config.air_drag_coefficient * (1.0 - self.state.draft_intensity.clamp(0.0, 0.50)) * drag_mult
    }

    pub fn set_digital_steering_source(&mut self, digital: bool) {
        self.digital_steering_source = digital;
        if !digital {
            self.flick_headroom_remaining_s = 0.0;
            self.flick_rearm_remaining_s = 0.0;
        }
    }

    pub fn reset_assist_telemetry(&mut self) {
        self.state.traction_help_multiplier = 1.0;
        self.state.tcs_lateral_multiplier = 1.0;
        self.state.tcs_longitudinal_multiplier = 1.0;
        self.state.esc_corrective_torque = 0.0;
    }

    /// Recovery is an electronic assist for analog inputs; digital inputs receive a minimum
    /// playability accommodation in Pro and use the stronger of the two in Arcade/Sport.
    pub fn digital_recovery_strength(&self) -> f32 {
        if self.digital_steering_source {
            0.35f32.max(if self.config.assists.counter_steer_assist_enabled {
                self.config.assists.counter_steer_assist_strength
            } else {
                0.0
            })
        } else if self.config.assists.counter_steer_assist_enabled {
            self.config.assists.counter_steer_assist_strength
        } else {
            0.0
        }
    }

    pub fn flick_headroom_remaining_s(&self) -> f32 {
        self.flick_headroom_remaining_s
    }

    pub fn flick_rearm_remaining_s(&self) -> f32 {
        self.flick_rearm_remaining_s
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
    pub fn set_config(&mut self, mut config: CarConfig) {
        config.finalize();
        for i in 0..4 {
            self.state.wheel_assemblies[i].config = config.wheels[i];
        }
        self.config = config;
    }

    /// Changes tire compound across all wheels on both config and runtime assembly state.
    pub fn set_compound(&mut self, compound: CompoundId) {
        self.config.set_compound(compound);
        for i in 0..4 {
            self.state.wheel_assemblies[i].config.compound = self.config.wheels[i].compound;
        }
    }

    /// Returns the current forward unit vector in world space.
    #[inline]
    pub fn forward_vector(&self) -> Vec2 {
        let (sin, cos) = self.state.angle.sin_cos();
        Vec2::new(cos, sin)
    }

    /// Returns the current right unit vector in world space.
    #[inline]
    pub fn right_vector(&self) -> Vec2 {
        let (sin, cos) = self.state.angle.sin_cos();
        Vec2::new(sin, -cos)
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

    /// Assist reduction multiplier for traction help in the previous step.
    #[inline]
    pub fn last_traction_help_multiplier(&self) -> f32 {
        self.state.traction_help_multiplier
    }

    /// Assist reduction multiplier for lateral TCS in the previous step.
    #[inline]
    pub fn last_tcs_lateral_multiplier(&self) -> f32 {
        self.state.tcs_lateral_multiplier
    }

    /// Assist reduction multiplier for longitudinal TCS in the previous step.
    #[inline]
    pub fn last_tcs_longitudinal_multiplier(&self) -> f32 {
        self.state.tcs_longitudinal_multiplier
    }

    /// Corrective ESC stabilizing yaw torque in N·m applied in the previous step.
    #[inline]
    pub fn last_esc_corrective_torque_nm(&self) -> f32 {
        self.state.esc_corrective_torque
    }

    /// Sets wheel ground elevation offsets (e.g. for kerbs, ruts, or bumps) in meters.
    #[inline]
    pub fn set_wheel_elevations(&mut self, elevations: [f32; 4]) {
        self.state.wheel_elevations = elevations;
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

    /// Largest useful road-wheel steering angle at `speed` on a surface with friction `surface_mu`
    /// (Spec 043 grip-aware authority), in radians.
    ///
    /// `atan(L / R_min) + (f(v) + steer_overslip - 1) * front peak slip angle`, where `R_min = v^2 / (mu * g_eff)`
    /// is the tightest radius the tires can hold (downforce included). At low speed `R_min` shrinks
    /// until the kinematic term alone exceeds mechanical lock, so parking-speed steering keeps full
    /// lock without a separate blend (a speed blend overshot the useful angle at 8-12 m/s).
    pub fn steer_authority(&self, speed: f32, surface_mu: f32) -> f32 {
        self.steer_authority_with(speed, surface_mu, self.config.player.steer_overslip)
    }

    /// [`Car::steer_authority`] with an explicit overslip instead of the driver's setting.
    pub fn steer_authority_with(&self, speed: f32, surface_mu: f32, steer_overslip: f32) -> f32 {
        let lock = self.config.max_steer_angle;
        let tire = &self.config.tire;
        let g_eff =
            9.81 + self.config.downforce_coefficient * speed * speed / self.config.mass.max(1.0);
        let mu = (surface_mu * tire.grip).max(0.01);
        let r_min = (speed * speed / (mu * g_eff)).max(1e-3);
        let kinematic = (self.config.wheelbase / r_min).atan();
        // In a steady corner the rear tires slip too, so the car reaches its grip limit at about
        // kinematic + f(v) * alpha_peak, not kinematic + alpha_peak. Measured across the factory
        // presets and checked against the monotonic-steering gate: f = 0.30 up to 25 m/s, falling to
        // 0.05 at 45 m/s (rally and kart hold more, so Balanced is conservative for them).
        // steer_overslip moves full input around that limit in units of the peak slip angle.
        let high_speed = ((speed - 25.0) / 20.0).clamp(0.0, 1.0);
        let limit_fraction = 0.30 - 0.25 * high_speed;
        // Past-the-limit authority (overslip > 1) shrinks to 30% at high speed: the stable window beyond
        // the limit is ~0.1 peak-widths at 45 m/s (Sharp/Raw keep their extra bite in slow corners).
        let beyond = steer_overslip - 1.0;
        let beyond = if beyond > 0.0 {
            beyond * (1.0 - 0.7 * high_speed)
        } else {
            beyond
        };
        let slip_share = limit_fraction + beyond;
        let grip_limit = kinematic + slip_share.max(0.0) * tire.peak_slip_angle();
        let grip_limit = grip_limit.clamp(0.0, lock);

        if self.config.caster_jacking_factor > 0.0 || speed <= 14.0 || speed >= 18.0 {
            return grip_limit.clamp(0.0, lock);
        }

        // Increase useful turn-in authority at parking/hairpin speeds, then smoothly return to
        // the Spec 043 grip mapping by 18 m/s. This bounded low-speed blend is validated against
        // monotonic curvature, braking distance and the mode/preset matrix.
        let smoothstep = |t: f32| {
            let t = t.clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let blend = smoothstep((speed - 14.0) / 1.5)
            * (1.0 - smoothstep((speed - 17.0) / 1.0));
        let expanded_fraction = 0.68;
        let preset_scale = (1.0 + (steer_overslip - 1.0) * 0.35).clamp(0.94, 1.06);
        let expanded_authority = lock * expanded_fraction * preset_scale;
        (grip_limit + (expanded_authority - grip_limit) * blend).clamp(0.0, lock)
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

/// Locking torque capacity of an axle differential (Spec 043), in N·m.
///
/// Open: 0 (equal torque, no speed coupling). LimitedSlip: preload + ramp * |input torque|
/// (power ramp under drive, coast ramp under engine braking). Spool: unbounded (rigid axle).
fn differential_lock_torque(diff_type: DifferentialType, input_torque: f32) -> f32 {
    match diff_type {
        DifferentialType::Open => 0.0,
        DifferentialType::LimitedSlip {
            power_lock,
            coast_lock,
            preload_nm,
        } => {
                let ramp = if input_torque >= 0.0 {
                    power_lock
                } else {
                    coast_lock
                };
            preload_nm.max(0.0) + ramp.clamp(0.0, 1.0) * input_torque.abs()
        }
        DifferentialType::Spool => f32::INFINITY,
    }
}

/// Implicit axle coupling after the wheel step (Spec 043).
///
/// `t_equalize` is the torque difference (T_R - T_L) that removes the speed difference in one
/// step given the tire-stiffened effective inertias. The differential passes at most its locking
/// torque, from the faster wheel to the slower one. A locked axle in a corner therefore drags the
/// outer wheel and pushes the inner one: an understeer moment, as on a real spool.
/// Returns the updated `(omega_left, omega_right)`.
fn couple_axle(
    diff_type: DifferentialType,
    input_torque: f32,
    omega_l: f32,
    omega_r: f32,
    inertia_l: f32,
    inertia_r: f32,
    dt: f32,
) -> (f32, f32) {
    let il = inertia_l.max(1e-3);
    let ir = inertia_r.max(1e-3);
    let compliance = dt.max(1e-6) * (1.0 / il + 1.0 / ir);
    let t_equalize = 2.0 * (omega_l - omega_r) / compliance;
    let t_lock = Self::differential_lock_torque(diff_type, input_torque);
    let t_x = t_equalize.clamp(-t_lock, t_lock);
    (omega_l - 0.5 * t_x * dt / il, omega_r + 0.5 * t_x * dt / ir)
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
        let p0 = sampler.sample_surface(wheel_positions[0]);
        let p1 = sampler.sample_surface(wheel_positions[1]);
        let p2 = sampler.sample_surface(wheel_positions[2]);
        let p3 = sampler.sample_surface(wheel_positions[3]);
        let surfaces = [
            p0.surface_type,
            p1.surface_type,
            p2.surface_type,
            p3.surface_type,
        ];
        self.state.wheel_elevations = [
            p0.elevation,
            p1.elevation,
            p2.elevation,
            p3.elevation,
        ];
        let center_props = sampler.sample_surface(self.state.position);
        self.state.road_elevation = center_props.elevation;
        self.state.road_bank_angle = center_props.bank_angle;
        self.state.track_right = center_props.track_right;

        self.step_per_wheel(controls, surfaces, dt);
    }

    /// Steps the physics simulation forward with independent surface types per wheel.
    pub fn step_per_wheel(&mut self, controls: &CarControls, surfaces: [SurfaceType; 4], dt: f32) {
        self.reset_assist_telemetry();
        self.flick_rearm_remaining_s = (self.flick_rearm_remaining_s - dt).max(0.0);
        self.flick_headroom_remaining_s = (self.flick_headroom_remaining_s - dt).max(0.0);
        // 0. Update vertical elevation dynamics
        self.state.just_landed = false;
        let mut touchdown_vz = 0.0f32;
        if self.state.elevation > 0.0 || self.state.vertical_velocity.abs() > 1e-4 {
            let gravity_z = 13.5f32; // snappy arcade gravity
            self.state.vertical_velocity -= gravity_z * dt;
            self.state.elevation += self.state.vertical_velocity * dt;
            if self.state.elevation <= 0.0 {
                self.state.elevation = 0.0;
                touchdown_vz = self.state.vertical_velocity.abs();
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
        let (body_sin, body_cos) = self.state.angle.sin_cos();
        let fwd = Vec2::new(body_cos, body_sin);
        let right = Vec2::new(body_sin, -body_cos);

        // Decompose velocity into vehicle chassis frame
        let v_long = self.state.velocity.dot(fwd);
        let v_lat = self.state.velocity.dot(right);
        self.state.local_velocity = Vec2::new(v_long, v_lat);
        self.state.speed = self.state.velocity.length();

        // 1. Steering dynamics: grip-aware authority and counter-steer assist (Spec 043)
        // steer > 0 is steering right (clockwise, -steer_angle in Cartesian coords)
        // steer < 0 is steering left (counter-clockwise, +steer_angle in Cartesian coords)

        // Counter-steering: the wheels point towards where the body is sliding. v_lat < 0 means the
        // car moves to the left of its heading, and steer < 0 steers left, so the product is
        // positive. (Pre-043 the test was `< -0.05`, which flagged normal cornering at speed, where
        // the velocity also points outside the heading; with slip headroom that fed slides.)
        let is_counter_steering = (clamped_ctrl.steer * v_lat) > 0.05;
        let tcs_drift_bypass = self.config.assists.tcs_drift_bypass
            && is_counter_steering
            && self.state.is_drifting
            && v_long > 1.0
            && !clamped_ctrl.reverse;

        // Human drivers (grip-aware steering): full input maps to the largest road-wheel angle the
        // front tires can use at this speed, so more input never gives less turn. Counter-steering
        // adds the body slip angle as headroom, so the wheels can still point down the road in a
        // slide. Bots and scripted controllers keep the linear full-lock mapping.
        let front_mu =
            0.5 * (surfaces[0].friction_coefficient() + surfaces[1].friction_coefficient());
        let steer_delta_per_second =
            (clamped_ctrl.steer - self.previous_control_steer).abs() / dt.max(1e-5);
        let intentional_flick = self.digital_steering_source
            && self.flick_rearm_remaining_s <= 0.0
            && self.previous_control_steer.abs() > 0.35
            && clamped_ctrl.steer.abs() > 0.35
            && clamped_ctrl.steer.signum() != self.previous_control_steer.signum()
            && steer_delta_per_second >= 3.0
            && self.state.speed < 28.0;
        if intentional_flick {
            self.flick_headroom_remaining_s = 0.18;
            self.flick_rearm_remaining_s = 0.30;
        }
        self.previous_control_steer = clamped_ctrl.steer;
        let flick_headroom = if self.config.player.grip_aware_steering {
            self.config.max_steer_angle * 0.18 * (self.flick_headroom_remaining_s / 0.18).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let counter_headroom = if is_counter_steering {
            self.state.sideslip_angle.abs()
        } else {
            0.0
        };
        let mut target_steer = if clamped_ctrl.steer.abs() < 1e-4 {
            0.0
        } else if self.config.player.grip_aware_steering {
            let authority = (self.steer_authority(self.state.speed, front_mu)
                + counter_headroom
                + flick_headroom)
                .min(self.config.max_steer_angle);
            -clamped_ctrl.steer * authority
        } else {
            // Linear full-lock mapping keeps closed-loop controller gains; the angle is capped at a
            // generous grip limit (overslip 1.5) so saturated commands cannot scrub past the tire.
            let cap = (self.steer_authority_with(self.state.speed, front_mu, BOT_STEER_OVERSLIP)
                + counter_headroom)
                .min(self.config.max_steer_angle);
            (-clamped_ctrl.steer * self.config.max_steer_angle).clamp(-cap, cap)
        };

        // Counter-steer / self-aligning drift recovery assist (forward motion only)
        let recovery_strength = if self.digital_steering_source {
            0.35f32.max(if self.config.assists.counter_steer_assist_enabled {
                self.config.assists.counter_steer_assist_strength
            } else {
                0.0
            })
        } else if self.config.assists.counter_steer_assist_enabled {
            self.config.assists.counter_steer_assist_strength
        } else {
            0.0
        };
        if recovery_strength > 0.0
            && !clamped_ctrl.handbrake
            && !clamped_ctrl.reverse
            && v_long > 1.0
            && self.state.speed > 2.0
            && self.state.sideslip_angle.abs() > 0.04
        {
            let align_angle = -self.state.sideslip_angle * recovery_strength;
            if clamped_ctrl.steer.abs() < 0.35 {
                let blend = 1.0 - (clamped_ctrl.steer.abs() / 0.35);
                target_steer += align_angle * blend;
            }
        }

        // Steering pull bias from asymmetric front suspension damage (Spec 078 Section 5.A)
        let pull_bias_ctrl = self.steering_pull_bias();
        let phys_bias = -pull_bias_ctrl;
        let effective_target_steer = target_steer + phys_bias;

        let steer_rate = if clamped_ctrl.steer.abs() < 1e-3 {
            self.config.steer_return_speed
        } else if is_counter_steering {
            self.config.steer_speed * self.config.counter_steer_assist
        } else {
            self.config.steer_speed
        };

        let steer_delta = effective_target_steer - self.state.steer_angle;
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
            bank_rad.sin_cos()
        } else {
            (0.0, 1.0)
        };

        // Longitudinal grade slope & vertical curvature (crest unloading / dip compression)
        let grade_rad = self.state.road_grade_slope;
        let (grade_sin, grade_cos) = if grade_rad.abs() > 1e-4 {
            grade_rad.sin_cos()
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

        let static_front_load =
            effective_normal_weight * (lr / wheelbase) + bank_compression * (lr / wheelbase);
        let static_rear_load =
            effective_normal_weight * (lf / wheelbase) + bank_compression * (lf / wheelbase);

        // Acceleration squat (a_long > 0): front unloads, rear loads
        // Grade incline pitch (grade_sin > 0 = uphill): front unloads, rear loads
        let grade_pitch = self.config.mass * g * grade_sin * (self.config.cg_height / wheelbase);
        let delta_fz_long =
            self.config.mass * a_long * (self.config.cg_height / wheelbase) + grade_pitch;

        // Cornering roll & gravity cross-slope roll moment
        let cross_slope_roll = if bank_deg.abs() > 1e-4 {
            self.config.mass * g * bank_sin * (self.config.cg_height / self.config.track_width)
        } else {
            0.0
        };

        // Lateral load transfer is physical (m * a * h / track); `roll_balance` decides which axle
        // carries it. With load-sensitive tires, the axle that carries more transfer loses more
        // grip, so roll_balance moves the handling balance (Spec 043).
        let roll_balance = self.config.roll_balance.clamp(0.0, 1.0);
        let delta_fz_lat_total =
            self.config.mass * a_lat * (self.config.cg_height / self.config.track_width)
                + cross_slope_roll;
        let delta_fz_lat_f = delta_fz_lat_total * roll_balance;
        let delta_fz_lat_r = delta_fz_lat_total * (1.0 - roll_balance);

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

        // Mechanical Caster Jacking (Spec 032):
        // In solid-axle vehicles (e.g. racing karts with 10°-15° kingpin caster and rigid chassis),
        // steering lock dynamically jacks the chassis diagonally, unloading the inside rear wheel
        // (down to near-zero load) to eliminate rear spool binding and allow razor-sharp apex pivoting.
        let nom_fz_fl =
            (static_front_load - delta_fz_long) * 0.5 + delta_fz_lat_f * 0.5 + downforce_front;
        let nom_fz_fr =
            (static_front_load - delta_fz_long) * 0.5 - delta_fz_lat_f * 0.5 + downforce_front;
        let nom_fz_rl =
            (static_rear_load + delta_fz_long) * 0.5 + delta_fz_lat_r * 0.5 + downforce_rear;
        let nom_fz_rr =
            (static_rear_load + delta_fz_long) * 0.5 - delta_fz_lat_r * 0.5 + downforce_rear;

        let (delta_fz_caster_fl, delta_fz_caster_fr, delta_fz_caster_rl, delta_fz_caster_rr) =
            if self.config.caster_jacking_factor > 1e-4 && self.state.steer_angle.abs() > 1e-4 {
                let steer_frac =
                    (self.state.steer_angle.abs() / self.config.max_steer_angle.max(1e-3)).min(1.0);
                // At walking pace there is no lateral load transfer to unload the inside rear, so the
                // lift curve is front-loaded (steer_frac^0.4) to free the spool up to 4 m/s; by 14 m/s it returns
                // to the racing curve (^1.15). Spec 043: with ^1.15 everywhere, weak karts pivoted in
                // place off the grid; with ^0.4 everywhere the kart spun at 50 km/h.
                let crawl = 1.0 - ((self.state.speed - 4.0) / 10.0).clamp(0.0, 1.0);
                let lift_exponent = 1.15 - 0.75 * crawl;
                let raw_delta = static_rear_load
                    * 0.5
                    * self.config.caster_jacking_factor
                    * steer_frac.powf(lift_exponent);

                if self.state.steer_angle < 0.0 {
                    // Turning right (steer_angle < 0): RR (inside rear) unloads, RL (outside rear) and FR (inside front) load
                    let max_unload = (nom_fz_rr - min_load_r).max(0.0);
                    let eff = raw_delta.min(max_unload);
                    (0.0, eff * 0.5, eff * 0.5, -eff)
                } else {
                    // Turning left (steer_angle > 0): RL (inside rear) unloads, RR (outside rear) and FL (inside front) load
                    let max_unload = (nom_fz_rl - min_load_r).max(0.0);
                    let eff = raw_delta.min(max_unload);
                    (eff * 0.5, 0.0, -eff, eff * 0.5)
                }
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

        // 3B. Dynamic 4-Corner Compliant Suspension & Body Articulation (Spec 076)
        let susp = &self.config.suspension;
        let track_w = self.config.track_width;

        // Roll center axis height at vehicle CG
        let rc_front = susp.front_roll_center_height;
        let rc_rear = susp.rear_roll_center_height;
        let rc_cg = rc_rear + (rc_front - rc_rear) * (lr / wheelbase);
        let h_roll = (self.config.cg_height - rc_cg).max(0.05);

        // Axle roll stiffnesses (springs + anti-roll bars)
        let k_phi_front = 0.5 * susp.front.spring_rate * track_w * track_w + susp.front_arb_rate;
        let k_phi_rear = 0.5 * susp.rear.spring_rate * track_w * track_w + susp.rear_arb_rate;
        let k_phi_total = (k_phi_front + k_phi_rear).max(1000.0);

        // Total overturning roll moment
        let roll_moment = self.config.mass * a_lat * h_roll
            + (if bank_deg.abs() > 1e-4 {
                self.config.mass * g * bank_sin * h_roll
            } else {
                0.0
            });

        let target_roll = if self.state.is_airborne {
            0.0
        } else {
            (roll_moment / k_phi_total).clamp(-0.15, 0.15)
        };

        // Pitch stiffness (front and rear axle springs)
        let k_theta_total = (2.0 * susp.front.spring_rate * lf * lf
            + 2.0 * susp.rear.spring_rate * lr * lr)
            .max(1000.0);
        let h_pitch = self.config.cg_height;

        // Total pitch moment: deceleration dive (a_long < 0) produces positive pitch (dive)
        let pitch_moment = -self.config.mass * a_long * h_pitch
            - (if grade_rad.abs() > 1e-4 {
                self.config.mass * g * grade_sin * h_pitch
            } else {
                0.0
            });

        let target_pitch = if self.state.is_airborne {
            0.0
        } else {
            (pitch_moment / k_theta_total).clamp(-0.15, 0.15)
        };

        // First-order response filter for body roll & pitch
        let f_susp = susp.response_frequency_hz.clamp(1.0, 20.0);
        let alpha_susp = 1.0 - (-2.0 * PI * f_susp * dt).exp();
        self.state.roll_angle += (target_roll - self.state.roll_angle) * alpha_susp;
        self.state.pitch_angle += (target_pitch - self.state.pitch_angle) * alpha_susp;

        let phi = self.state.roll_angle;
        let theta = self.state.pitch_angle;
        let sin_phi = phi.sin();
        let sin_theta = theta.sin();

        let mut strokes = [0.0f32; 4];
        let mut stroke_vels = [0.0f32; 4];
        let mut bottomed_outs = [false; 4];
        let mut bumpstop_forces = [0.0f32; 4];
        let mut damper_forces = [0.0f32; 4];
        let mut dynamic_cambers = [0.0f32; 4];
        let mut mu_cambers = [1.0f32; 4];

        let corner_mass_f = static_front_load * 0.5 / g;
        let corner_mass_r = static_rear_load * 0.5 / g;
        let sqrt_k_m_f = (susp.front.spring_rate * corner_mass_f.max(1.0)).sqrt();
        let sqrt_k_m_r = (susp.rear.spring_rate * corner_mass_r.max(1.0)).sqrt();
        // A curb raises the wheels on it by 4 cm. With every wheel on the curb the whole car sits higher and no
        // spring is compressed (tdrace-le75: the springs were compressed, so cars on curbs had odd loads).
        let all_on_curb = surfaces.iter().all(|s| *s == SurfaceType::Curb);
        let curb_bump = |i: usize| if surfaces[i] == SurfaceType::Curb && !all_on_curb { 0.04 } else { 0.0 };

        for i in 0..4 {
            let wheel_id = WheelId::ALL[i];
            let corner = if wheel_id.is_front() {
                &susp.front
            } else {
                &susp.rear
            };
            let xi = if wheel_id.is_front() { lf } else { -lr };
            let yi = if wheel_id.is_left() { -half_w } else { half_w };

            // Vertical chassis corner displacement (positive theta = braking dive, positive phi = left roll)
            let z_chassis = -xi * sin_theta + yi * sin_phi;

            // Track elevation profile under wheel
            let mut z_track = self.state.wheel_elevations[i];
            if z_track.abs() < 1e-4 && curb_bump(i) > 0.0 {
                z_track = curb_bump(i);
            }

            // Landing compression from aerial drop touchdown
            let (corner_mass, sqrt_k_m) = if wheel_id.is_front() {
                (corner_mass_f, sqrt_k_m_f)
            } else {
                (corner_mass_r, sqrt_k_m_r)
            };
            let h_susp = self.state.suspension_health[i];
            let z_landing = if touchdown_vz > 0.0 {
                let omega_n = sqrt_k_m / corner_mass.max(1.0);
                let zeta = corner.bump_damping_ratio * h_susp;
                if h_susp <= 0.05 {
                    corner.max_bump_travel
                } else {
                    let sqrt_term = (1.0 - zeta * zeta).max(1e-4).sqrt();
                    let peak_ratio = (-(zeta * sqrt_term.atan2(zeta)) / sqrt_term).exp();
                    (touchdown_vz / omega_n) * peak_ratio
                }
            } else {
                0.0
            };

            let delta_z = if self.state.is_airborne {
                -corner.max_rebound_travel
            } else {
                z_track - z_chassis + z_landing
            };

            let mut s = delta_z.clamp(-corner.max_rebound_travel, corner.max_bump_travel);
            if corner.archetype == SuspensionArchetype::PushrodInboard && h_susp < PUSHROD_COLLAPSE_HEALTH {
                s = corner.max_bump_travel;
            }
            strokes[i] = s;

            let prev_s = self.state.suspension[i].deflection;
            let s_dot = if dt > 1e-5 {
                ((s - prev_s) / dt).clamp(-10.0, 10.0)
            } else {
                0.0
            };
            stroke_vels[i] = s_dot;

            // Damping force degraded by suspension health (Spec 078 Section 4.B)
            let eff_h_susp = if h_susp >= 0.85 {
                1.0
            } else {
                h_susp / 0.85
            };
            let c_damping = if s_dot >= 0.0 {
                2.0 * corner.bump_damping_ratio * eff_h_susp * sqrt_k_m
            } else {
                2.0 * corner.rebound_damping_ratio * eff_h_susp * sqrt_k_m
            };
            damper_forces[i] = c_damping * s_dot;

            // Bump-stop bottoming
            let delta_stop = (delta_z - corner.max_bump_travel).max(0.0);
            let bottomed = delta_stop > 0.002
                || (s >= corner.max_bump_travel - 1e-4 && (s_dot > 0.5 || (corner.archetype == SuspensionArchetype::PushrodInboard && h_susp < PUSHROD_COLLAPSE_HEALTH)));
            bottomed_outs[i] = bottomed;
            bumpstop_forces[i] = if delta_stop > 0.0 || (bottomed && corner.archetype == SuspensionArchetype::PushrodInboard && h_susp < PUSHROD_COLLAPSE_HEALTH) {
                4.0 * corner.spring_rate * delta_stop.max(0.004) + 2.0 * c_damping * s_dot.max(0.0)
            } else {
                0.0
            };

            // Kerb bottom-out damage accumulation (Spec 078 Section 4.A)
            if self.config.damage_enabled && (bottomed || delta_stop > 0.0) && s_dot > KERB_BOTTOM_OUT_SPEED_MPS {
                let v_excess = s_dot - KERB_BOTTOM_OUT_SPEED_MPS;
                let k_rob = corner.archetype.robustness_factor();
                let delta_h = (0.5 * corner_mass * v_excess * v_excess) / (KERB_BOTTOM_OUT_CAPACITY_J * k_rob);
                self.state.suspension_health[i] = (self.state.suspension_health[i] - delta_h).clamp(0.0, 1.0);
            }

            // Violent jump touchdown damage accumulation (Spec 078 Section 4.A)
            if self.config.damage_enabled && touchdown_vz > LANDING_SPEED_LIMIT_MPS {
                let v_excess = touchdown_vz - LANDING_SPEED_LIMIT_MPS;
                let k_rob = corner.archetype.robustness_factor();
                let delta_h = (0.5 * corner_mass * v_excess * v_excess) / (LANDING_CAPACITY_J * k_rob);
                self.state.suspension_health[i] = (self.state.suspension_health[i] - delta_h).clamp(0.0, 1.0);
            }

            // Dynamic camber calculation: outside tire in turn (FR in left turn, FL in right turn)
            // degrades towards positive camber (rolling onto outer shoulder)
            let roll_sign = if wheel_id.is_left() { 1.0 } else { -1.0 };
            let susp_wear = (1.0 - self.state.suspension_health[i]).clamp(0.0, 1.0);
            let effective_susp_damage = (susp_wear - 0.15).max(0.0) / 0.85;
            let damage_camber = roll_sign * effective_susp_damage * 0.10;
            let camber = if corner.archetype == SuspensionArchetype::SolidLiveAxle && !wheel_id.is_front() {
                // Live axle coupled camber will be updated after axle stroke calculation
                corner.static_camber + damage_camber
            } else {
                corner.static_camber + roll_sign * phi * (1.0 - corner.camber_recovery) + damage_camber
            };
            dynamic_cambers[i] = camber;

            // Camber grip degradation: quadratic drop-off + shoulder scrub when rolling positive
            let delta_gamma_loss = (roll_sign * phi * (1.0 - corner.camber_recovery)).max(0.0)
                + effective_susp_damage * 0.10;
            mu_cambers[i] = (1.0 - 1.8 * camber * camber - 1.0 * delta_gamma_loss).clamp(0.65, 1.05);
        }

        // Coupled rear solid axle camber update
        if susp.rear.archetype == SuspensionArchetype::SolidLiveAxle {
            let beam_tilt = (strokes[3] - strokes[2]) / track_w.max(0.1);
            let susp_wear_2 = (1.0 - self.state.suspension_health[2]).clamp(0.0, 1.0);
            let susp_wear_3 = (1.0 - self.state.suspension_health[3]).clamp(0.0, 1.0);
            let eff_dmg_2 = (susp_wear_2 - 0.15).max(0.0) / 0.85;
            let eff_dmg_3 = (susp_wear_3 - 0.15).max(0.0) / 0.85;
            let dmg_2 = eff_dmg_2 * 0.10;
            let dmg_3 = -eff_dmg_3 * 0.10;
            dynamic_cambers[2] = susp.rear.static_camber + beam_tilt + dmg_2;
            dynamic_cambers[3] = susp.rear.static_camber - beam_tilt + dmg_3;
            mu_cambers[2] = (1.0 - 1.8 * dynamic_cambers[2] * dynamic_cambers[2] - dmg_2.abs()).clamp(0.65, 1.05);
            mu_cambers[3] = (1.0 - 1.8 * dynamic_cambers[3] * dynamic_cambers[3] - dmg_3.abs()).clamp(0.65, 1.05);
        }

        // Touchdown asymmetric roll snap (Spec 078 Section 5.C)
        if touchdown_vz > 0.0 {
            let fz_left = bumpstop_forces[0] + bumpstop_forces[2] + damper_forces[0] + damper_forces[2];
            let fz_right = bumpstop_forces[1] + bumpstop_forces[3] + damper_forces[1] + damper_forces[3];
            let tau_roll_snap = (fz_left - fz_right) * half_w;
            if (fz_left - fz_right).abs() > 400.0 {
                let roll_snap = (tau_roll_snap / k_phi_total).clamp(-0.40, 0.40);
                self.state.roll_angle += roll_snap;
                self.state.angular_velocity += (roll_snap / half_w) * 0.5;
            }
        }

        // Anti-roll bar forces (represented in roll stiffness k_phi)
        let f_arb_f = susp.front_arb_rate * (strokes[0] - strokes[1]) / (track_w * track_w).max(0.01);
        let f_arb_r = susp.rear_arb_rate * (strokes[2] - strokes[3]) / (track_w * track_w).max(0.01);
        let _arb_forces = [-f_arb_f, f_arb_f, -f_arb_r, f_arb_r];

        // Rigid kart diagonal jacking
        let mut diag_forces = [0.0f32; 4];
        if susp.front.archetype == SuspensionArchetype::RigidKart {
            let shock_0 = (susp.front.spring_rate * self.state.wheel_elevations[0].max(curb_bump(0)) + bumpstop_forces[0]).max(0.0);
            let shock_1 = (susp.front.spring_rate * self.state.wheel_elevations[1].max(curb_bump(1)) + bumpstop_forces[1]).max(0.0);
            let shock_2 = (susp.rear.spring_rate * self.state.wheel_elevations[2].max(curb_bump(2)) + bumpstop_forces[2]).max(0.0);
            let shock_3 = (susp.rear.spring_rate * self.state.wheel_elevations[3].max(curb_bump(3)) + bumpstop_forces[3]).max(0.0);

            diag_forces[0] -= 0.50 * shock_3;
            diag_forces[3] -= 0.50 * shock_0;
            diag_forces[1] -= 0.50 * shock_2;
            diag_forces[2] -= 0.50 * shock_1;
        }

        // Wheel 0 = FL (left), Wheel 1 = FR (right), Wheel 2 = RL (left), Wheel 3 = RR (right)
        let mut normal_loads = [0.0f32; 4];
        let mut nominal_sum = 0.0f32;
        for i in 0..4 {
            let wheel_id = WheelId::ALL[i];
            let corner = if wheel_id.is_front() { &susp.front } else { &susp.rear };
            let nom = match i {
                0 => nom_fz_fl + delta_fz_caster_fl,
                1 => nom_fz_fr + delta_fz_caster_fr,
                2 => nom_fz_rl + delta_fz_caster_rl,
                _ => nom_fz_rr + delta_fz_caster_rr,
            };
            let min_load = if wheel_id.is_front() { min_load_f } else { min_load_r };

            let z_bump = self.state.wheel_elevations[i].max(curb_bump(i));
            let f_susp_bump = corner.spring_rate * z_bump;
            let f_susp_damper = if z_bump > 0.0 || touchdown_vz > 0.0 { damper_forces[i] } else { 0.0 };
            let delta_fz_susp = f_susp_bump + f_susp_damper + bumpstop_forces[i] + diag_forces[i];

            let fz = ((nom + delta_fz_susp).max(min_load)) * ground_contact;
            normal_loads[i] = fz.min(4.0 * total_weight);
            nominal_sum += nom.max(min_load) * ground_contact;
        }
        // The fixed curb height only compresses the springs (there is no chassis heave), so a stiff kart
        // hit its bump stops and the curbs added up to 16 times its weight: a kart on a curb could not move,
        // and other cars gained grip on curbs (tdrace-le75). A curb moves load between the wheels (and
        // unloads a kart's opposite diagonal); outside a landing it does not add to the total.
        let total_load: f32 = normal_loads.iter().sum();
        if touchdown_vz <= 0.0 && surfaces.contains(&SurfaceType::Curb) && total_load > nominal_sum {
            for fz in &mut normal_loads {
                *fz *= nominal_sum / total_load;
            }
        }

        // 4. Tires, wheel spin and drivetrain (Spec 043)
        //
        // Pass A: contact-patch kinematics and friction envelopes.
        // Pass B: drive / brake torques and implicit wheel spin (+ axle coupling).
        // Pass C: tire forces from the integrated slip, chassis force/torque sums and telemetry.
        let mut total_wheel_force_world = Vec2::ZERO;
        let mut total_wheel_torque = 0.0;

        let omega = self.state.angular_velocity;

        // Drive / Brake torque requests with top-speed governor (Spec 038)
        // Average driven-wheel speed (representing driveshaft / differential carrier speed)
        // governs top-speed power taper alongside chassis speed, preventing a single unloaded
        // spinning inside wheel from choking engine power during cornering.
        let power_ratio = self.available_engine_power_ratio();
        let top_speed = self.config.top_speed_mps * (0.60 + 0.40 * power_ratio);
        let mut driven_count = 0.0f32;
        let mut driven_speed_sum = 0.0f32;
        for w in self.state.wheel_assemblies.iter() {
            if w.config.drive_torque_factor > 0.0 {
                driven_count += 1.0;
                driven_speed_sum += (w.angular_velocity * w.config.tire_radius).abs();
            }
        }
        let avg_driven_speed = if driven_count > 0.0 {
            driven_speed_sum / driven_count
        } else {
            v_long.abs()
        };
        let effective_engine_speed = v_long.abs().max(avg_driven_speed);
        let speed_ratio = effective_engine_speed / top_speed;
        let engine_taper = if speed_ratio < 0.90 {
            1.0
        } else {
            (1.0 - (speed_ratio - 0.90) / 0.10).clamp(0.0, 1.0)
        };

        // TCS: the lateral term cuts engine torque when the rear axle slides past the trigger
        // angle; the longitudinal term (per wheel, Pass B) holds driven-wheel slip at its target.
        let tcs_engaged = self.config.assists.tcs_enabled
            && !tcs_drift_bypass
            && clamped_ctrl.throttle > 0.0
            && !(self.config.assists.handbrake_bypass && clamped_ctrl.handbrake);
        let mut tcs_active = false;
        let mut drive_torque_multiplier = 1.0f32;
        if tcs_engaged && !clamped_ctrl.reverse {
            let rear_slip_lat = self.state.wheels[2]
                .slip_angle
                .abs()
                .max(self.state.wheels[3].slip_angle.abs());
            let trigger = self
                .config
                .assists
                .tcs_slip_angle_deg
                .to_radians()
                .max(1e-3);
            if rear_slip_lat > trigger {
                let excess_lat = (rear_slip_lat - trigger) / trigger;
                drive_torque_multiplier =
                    1.0 - (excess_lat * self.config.assists.tcs_strength).clamp(0.0, 0.75);
                self.state.tcs_lateral_multiplier = drive_torque_multiplier;
                tcs_active = true;
            }
        }

        // Engine Drag Reduction (EDR / MSR): fade engine braking out as body sideslip grows, so a lift
        // in a slide lets the rear tires regain lateral traction. Continuous in sideslip only: normal
        // cornering yaw rates no longer switch engine braking off mid-corner (Spec 043).
        let slide_severity = ((self.state.sideslip_angle.abs() - 0.08) / 0.12).clamp(0.0, 1.0);
        let engine_brake_multiplier = 1.0 - 0.85 * slide_severity;

        // Traction help (player aid, Spec 043): ease the throttle as the rear axle nears its limit,
        // the way a good driver (and the AI) feeds throttle out of a corner. "Near the limit" is the
        // larger of the rear tires' combined force use and slip angle relative to peak: lateral
        // force alone drops as drive force grows, so it hid power oversteer until too late.
        let traction_help = self.config.player.traction_help.clamp(0.0, 1.0);
        let throttle_scale = if traction_help > 0.0
            && !clamped_ctrl.reverse
            && !is_counter_steering
            && !self.state.is_drifting
        {
            // Force use counts only while cornering: straight-line drive (a launch, a straight)
            // uses the rear grip fully without any risk of power oversteer, and TCS already limits
            // wheelspin there. Steering input counts as cornering too, because in a direction
            // change the yaw rate passes through zero. Slip angle is a lateral signal by itself.
            let cornering = (self.state.speed * self.state.angular_velocity.abs()
                / TH_CORNERING_ACCEL)
                .max(clamped_ctrl.steer.abs() / TH_CORNERING_STEER)
                .clamp(0.0, 1.0);
            let rear_use = [2usize, 3]
                .iter()
                .map(|&j| {
                    let w = &self.state.wheels[j];
                    let tire = &self.state.wheel_assemblies[j].config.tire_model;
                    let envelope =
                        (w.normal_load * w.surface.friction_coefficient() * tire.grip).max(1.0);
                    let force_use = w.lateral_force.hypot(w.longitudinal_force) / envelope;
                    let slip_use = w.slip_angle.abs() / tire.peak_slip_angle().max(1e-3);
                    (force_use * cornering).max(slip_use)
                })
                .fold(0.0f32, f32::max);
            1.0 - traction_help * ((rear_use - TH_START) / TH_WIDTH).clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.state.traction_help_multiplier = throttle_scale;

        let total_drive_force = if clamped_ctrl.reverse {
            -clamped_ctrl.throttle * self.config.max_reverse_force
        } else if clamped_ctrl.throttle > 0.0 {
            let engine_power_mult = self.available_engine_power_ratio();
            clamped_ctrl.throttle
                * throttle_scale
                * self.config.max_engine_force
                * engine_taper
                * drive_torque_multiplier
                * engine_power_mult
        } else if self.config.engine_braking_coefficient > 0.0 && v_long.abs() > 0.05 {
            // Enhanced generic motor brake with EDR modulation
            let generic_motor_brake_boost = 1.85f32;
            -self.config.engine_braking_coefficient
                * generic_motor_brake_boost
                * total_weight
                * fast_tanh_clip(v_long / 1.5)
                * engine_brake_multiplier
        } else {
            0.0
        };

        // Progressive, non-linear service brake input mapping:
        // Soft, progressive response at light-to-medium pedal travel for delicate trail-braking and apex adjustments,
        // smoothly ramping up to maximum deceleration on full brake application.
        let raw_brake = clamped_ctrl.brake;
        let progressive_brake = if raw_brake > 0.0 { raw_brake.powf(1.4) } else { 0.0 };
        let total_brake_force = progressive_brake * self.config.max_brake_force;
        let mut abs_active = false;

        let total_normal_load: f32 = normal_loads.iter().sum();

        let mut surface_mus = [0.0f32; 4];
        for i in 0..4 {
            let surf = surfaces[i];
            let mut mu = surf.friction_coefficient();
            let affinity = self.state.wheel_assemblies[i].config.compound.surface_affinity.get(surf);
            mu *= affinity;
            if surf == SurfaceType::SheetIce {
                let alpha = self.config.terrain.ice_grip_multiplier.clamp(0.50, 10.0);
                mu = (mu * alpha).min(1.20);
            }
            let prev_dirt = self.state.wheels[i].dirt_contamination;
            if surf.is_rigid_pavement() && prev_dirt > 0.02 {
                mu *= (1.0 - 0.20 * prev_dirt).max(0.65);
            }
            // Camber-induced tire friction degradation (Spec 076)
            mu *= mu_cambers[i];
            surface_mus[i] = mu;
        }

        // Pass A: contact-patch kinematics
        let mut offsets_world = [Vec2::ZERO; 4];
        let mut wheel_v_worlds = [Vec2::ZERO; 4];
        let mut wheel_fwds = [Vec2::ZERO; 4];
        let mut wheel_rights = [Vec2::ZERO; 4];
        let mut wheel_v_longs = [0.0f32; 4];
        let mut slip_angles = [0.0f32; 4];
        let mut envelopes = [0.0f32; 4];
        for i in 0..4 {
            let offset_local = wheel_local_offsets[i];
            let offset_world = fwd * offset_local.x + right * offset_local.y;

            // Contact patch world velocity = V_cg + omega x r
            let v_rot = Vec2::new(-omega * offset_world.y, omega * offset_world.x);
            let wheel_v_world = self.state.velocity + v_rot;

            // Wheel orientation
            let (wheel_fwd, wheel_right) = if i < 2 {
                let wheel_angle_world = self.state.angle + wheel_steer_angles[i];
                let (w_sin, w_cos) = wheel_angle_world.sin_cos();
                (Vec2::new(w_cos, w_sin), Vec2::new(w_sin, -w_cos))
            } else {
                (fwd, right)
            };

            let w_v_long = wheel_v_world.dot(wheel_fwd);
            let w_v_lat = wheel_v_world.dot(wheel_right);

            let nominal_fz = if WheelId::ALL[i].is_front() {
                total_weight * (lr / wheelbase) * 0.5
            } else {
                total_weight * (lf / wheelbase) * 0.5
            };

            offsets_world[i] = offset_world;
            wheel_v_worlds[i] = wheel_v_world;
            wheel_fwds[i] = wheel_fwd;
            wheel_rights[i] = wheel_right;
            wheel_v_longs[i] = w_v_long;
            // Slip angle: angle between wheel direction and velocity vector
            slip_angles[i] = (-w_v_lat / w_v_long.abs().max(2.5)).atan();
            envelopes[i] = self.state.wheel_assemblies[i].friction_envelope(
                normal_loads[i],
                nominal_fz,
                surface_mus[i],
            );

            // Kinematic rolling synchronization: if vehicle was spawned/set at speed without previous lockup
            let r = self.state.wheel_assemblies[i].config.tire_radius.max(1e-2);
            if !self.state.wheel_assemblies[i].is_locked
                && self.state.wheel_assemblies[i].angular_velocity.abs() < 1e-3
                && w_v_long.abs() > 1.0
                && clamped_ctrl.brake < 0.05
                && !clamped_ctrl.handbrake
            {
                self.state.wheel_assemblies[i].angular_velocity = w_v_long / r;
            }
        }

        // Pass B: drive / brake torques and implicit wheel spin, axle by axle
        // Cornering Brake Control (CBC): trim inside rear brake pressure under oversteering yaw divergence
        let kinematic_yaw_rate = if self.state.steer_angle.abs() > 1e-4 {
            (v_long / self.config.wheelbase) * self.state.steer_angle.tan()
        } else {
            0.0
        };
        let yaw_divergence = omega - kinematic_yaw_rate;
        let is_oversteering_under_brake = (omega.signum() == kinematic_yaw_rate.signum()
            && omega.abs() > (kinematic_yaw_rate.abs() + 0.08))
            || (kinematic_yaw_rate.abs() < 0.05 && omega.abs() > 0.08)
            || (omega.signum() != kinematic_yaw_rate.signum() && omega.abs() > 0.12);

        // Electronic Brakeforce Distribution (with ABS): `brake_bias` is the minimum front share;
        // EBD only moves bias forward, towards the dynamic front load share, never rearward.
        let front_brake_share = if self.config.assists.abs_enabled && total_normal_load > 1e-3 {
            let dynamic_front = (normal_loads[0] + normal_loads[1]) / total_normal_load;
            self.config.brake_bias.max(dynamic_front).clamp(0.0, 1.0)
        } else {
            self.config.brake_bias.clamp(0.0, 1.0)
        };
        let abs_target_scale = self.config.assists.abs_slip_threshold / 0.15;
        let is_coasting = !clamped_ctrl.reverse && clamped_ctrl.throttle <= 0.0;
        let tcs_target_scale = self.config.assists.tcs_slip_threshold / 0.18;
        let tcs_strength = self.config.assists.tcs_strength.clamp(0.0, 1.0);

        for (axle_start, is_front_axle) in [(0usize, true), (2usize, false)] {
            let pair = [axle_start, axle_start + 1];
            // Drive torque follows drive_bias; engine-braking retard follows engine_brake_front_share.
            let front_share = if is_coasting {
                self.config.engine_brake_front_share.clamp(0.0, 1.0)
            } else {
                self.config.drive_bias
            };
            let (diff_type, axle_force) = if is_front_axle {
                (
                    self.config.front_differential,
                    total_drive_force * front_share,
                )
            } else {
                (
                    self.config.rear_differential,
                    total_drive_force * (1.0 - front_share),
                )
            };
            // Handbrake declutches the rear axle (arcade convention: the handbrake always wins over
            // throttle on the rear wheels, so a held throttle cannot stop the rear from locking).
            let axle_force = if !is_front_axle && clamped_ctrl.handbrake {
                0.0
            } else {
                axle_force
            };
            let r_axle = self.state.wheel_assemblies[axle_start]
                .config
                .tire_radius
                .max(1e-2);
            let mut axle_torque = axle_force * r_axle;

            // TCS torque reduction at the axle input: trim towards the torque the differential can
            // put down at the slip target (open: twice the weaker side; LSD: plus its locking torque;
            // spool: both sides).
            let axle_driven = axle_force.abs() > 1e-4;
            // Drive direction: forward gear pushes slip positive, reverse gear pushes it negative.
            let drive_dir = if clamped_ctrl.reverse {
                -1.0f32
            } else {
                1.0f32
            };
            let tcs_axle = tcs_engaged && axle_driven && axle_torque * drive_dir > 0.0;
            let tcs_targets = pair.map(|j| {
                self.state.wheel_assemblies[j]
                    .config
                    .tire_model
                    .peak_slip_ratio
                    * tcs_target_scale
                    * drive_dir
            });
            if tcs_axle {
                let caps = [0, 1].map(|k| {
                    let j = pair[k];
                    let a = &self.state.wheel_assemblies[j];
                    (combined_slip_fx(
                        tcs_targets[k],
                        slip_angles[j],
                        envelopes[j],
                        &a.config.tire_model,
                    ) * drive_dir)
                        .max(0.0)
                        * a.config.tire_radius
                });
                let weak = caps[0].min(caps[1]);
                let axle_cap = match diff_type {
                    DifferentialType::Open => 2.0 * weak,
                    DifferentialType::LimitedSlip { .. } => (2.0 * weak
                        + Self::differential_lock_torque(diff_type, axle_torque.abs()))
                    .min(caps[0] + caps[1]),
                    DifferentialType::Spool => caps[0] + caps[1],
                };
                let demand = axle_torque.abs();
                if demand > axle_cap {
                    let before = axle_torque.abs();
                    axle_torque -= (demand - axle_cap) * tcs_strength * drive_dir;
                    if before > 1e-3 {
                        self.state.tcs_longitudinal_multiplier = self
                            .state
                            .tcs_longitudinal_multiplier
                            .min((axle_torque.abs() / before).clamp(0.0, 1.0));
                    }
                    tcs_active = true;
                }
            }

            // ABS rear select-low precomputation (Spec 084): both rear brakes are capped to what the
            // lower-grip rear tire can take at its slip target, evaluated once per axle using combined_slip_fx.
            let rear_abs_cap = if self.config.assists.abs_enabled && !is_front_axle {
                let cap_for = |j: usize| {
                    let a = &self.state.wheel_assemblies[j];
                    let target = a.config.tire_model.peak_slip_ratio * abs_target_scale * 0.6;
                    combined_slip_fx(
                        -target,
                        slip_angles[j],
                        envelopes[j],
                        &a.config.tire_model,
                    )
                    .abs()
                };
                Some(cap_for(2).min(cap_for(3)))
            } else {
                None
            };

            let mut effective_inertias = [0.0f32; 2];
            for (k, &i) in pair.iter().enumerate() {
                let wheel_id = WheelId::ALL[i];
                let r = self.state.wheel_assemblies[i].config.tire_radius.max(1e-2);
                let w_v_long = wheel_v_longs[i];

                let axle_brake_share = if is_front_axle {
                    front_brake_share
                } else {
                    1.0 - front_brake_share
                };
                let mut wheel_brake_force = total_brake_force * axle_brake_share * 0.5;
                if self.config.assists.abs_enabled
                    && is_oversteering_under_brake
                    && w_v_long.abs() > 0.5
                {
                    let yaw_sign = omega.signum();
                    let is_inside_rear = (yaw_sign > 0.0 && wheel_id == WheelId::RearLeft)
                        || (yaw_sign < 0.0 && wheel_id == WheelId::RearRight);
                    if is_inside_rear {
                        let cbc_cut =
                            (yaw_divergence.abs() * 1.5 * self.config.assists.abs_strength)
                                 .clamp(0.0, 0.45);
                        wheel_brake_force *= 1.0 - cbc_cut;
                    }
                }
                if let Some(cap) = rear_abs_cap {
                    if w_v_long.abs() > 0.5 {
                        wheel_brake_force = wheel_brake_force.min(cap);
                    }
                }
                let is_handbraking_wheel = clamped_ctrl.handbrake && wheel_id.is_rear();
                if is_handbraking_wheel {
                    wheel_brake_force += self.config.handbrake_force * 0.5;
                }
                let brake_torque = wheel_brake_force * r;

                let eff_inertia = self.state.wheel_assemblies[i].effective_inertia(
                    w_v_long,
                    slip_angles[i],
                    envelopes[i],
                    dt,
                );
                effective_inertias[k] = eff_inertia;
                self.state.wheel_assemblies[i].step_implicit_with_inertia(
                    0.5 * axle_torque,
                    brake_torque,
                    w_v_long,
                    slip_angles[i],
                    envelopes[i],
                    eff_inertia,
                    dt,
                );

                // ABS: hold braking slip at the target; the target shrinks with lateral utilization so
                // the tire keeps steering authority while cornering (continuous, no steering gate).
                let assembly = &mut self.state.wheel_assemblies[i];
                if self.config.assists.abs_enabled
                    && !is_handbraking_wheel
                    && brake_torque > 0.0
                    && w_v_long.abs() > 0.5
                {
                    let lateral_use = if envelopes[i] > 1e-3 {
                        (self.state.wheels[i].lateral_force.abs() / envelopes[i]).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    // EBD: the rear axle is held further below its peak than the front so it keeps the
                    // lateral stiffness that holds the car straight under braking.
                    let axle_share = if is_front_axle { 1.0 } else { 0.6 };
                    let target = assembly.config.tire_model.peak_slip_ratio
                        * abs_target_scale
                        * axle_share
                        * (1.0 - 0.4 * lateral_use);
                    let omega_limit =
                        assembly.omega_for_slip(-target * w_v_long.signum(), w_v_long);
                    let over_slipping = if w_v_long > 0.0 {
                        assembly.angular_velocity < omega_limit
                    } else {
                        assembly.angular_velocity > omega_limit
                    };
                    if over_slipping {
                        let strength = self.config.assists.abs_strength.clamp(0.0, 1.0);
                        assembly.angular_velocity +=
                            (omega_limit - assembly.angular_velocity) * strength;
                        assembly.is_locked = false;
                        abs_active = true;
                    }
                }
            }

            // Differential coupling (Spec 043): implicit locking torque between the two wheels.
            if axle_driven || diff_type == DifferentialType::Spool {
                let (wl, wr) = Self::couple_axle(
                    diff_type,
                    axle_torque,
                    self.state.wheel_assemblies[pair[0]].angular_velocity,
                    self.state.wheel_assemblies[pair[1]].angular_velocity,
                    effective_inertias[0],
                    effective_inertias[1],
                    dt,
                );
                self.state.wheel_assemblies[pair[0]].angular_velocity = wl;
                self.state.wheel_assemblies[pair[1]].angular_velocity = wr;
            }

            // TCS (longitudinal): once the axle is coupled, catch any runaway above the slip target.
            // A spool shares one speed, so it is held at the grip-weighted mean of both wheels'
            // limits: in a corner the loaded outer wheel must be free to drive while the unloaded
            // inner wheel over-slips (a plain mean pinned a kart spool at zero drive).
            if tcs_axle {
                let limits = pair.map(|j| {
                    let k = j - axle_start;
                    self.state.wheel_assemblies[j].omega_for_slip(tcs_targets[k], wheel_v_longs[j])
                });
                let limits = if diff_type == DifferentialType::Spool {
                    let (ea, eb) = (envelopes[pair[0]], envelopes[pair[1]]);
                    let shared = if ea + eb > 1e-3 {
                        (limits[0] * ea + limits[1] * eb) / (ea + eb)
                    } else {
                        0.5 * (limits[0] + limits[1])
                    };
                    [shared; 2]
                } else {
                    limits
                };
                for (j, omega_limit) in pair.into_iter().zip(limits) {
                    let w = &mut self.state.wheel_assemblies[j];
                    if (w.angular_velocity - omega_limit) * drive_dir > 0.0 {
                        let before = w.angular_velocity.abs();
                        w.angular_velocity += (omega_limit - w.angular_velocity) * tcs_strength;
                        if before > 1e-3 {
                            self.state.tcs_longitudinal_multiplier = self
                                .state
                                .tcs_longitudinal_multiplier
                                .min((w.angular_velocity.abs() / before).clamp(0.0, 1.0));
                        }
                        tcs_active = true;
                    }
                }
            }
        }
        self.state.tcs_active = tcs_active;

        // Pass C: tire forces from the integrated wheel state
        for i in 0..4 {
            let wheel_id = WheelId::ALL[i];
            let offset_world = offsets_world[i];
            let wheel_pos_world = self.state.position + offset_world;
            let wheel_v_world = wheel_v_worlds[i];
            let wheel_fwd = wheel_fwds[i];
            let wheel_right = wheel_rights[i];
            let w_v_long = wheel_v_longs[i];
            let slip_angle = slip_angles[i];
            let surf = surfaces[i];
            let fz = normal_loads[i];
            let envelope = envelopes[i];
            let prev_dirt = self.state.wheels[i].dirt_contamination;
            let prev_dirt_surface = self.state.wheels[i].dirt_surface;
            let is_handbraking_wheel = clamped_ctrl.handbrake && wheel_id.is_rear();

            let raw_slip_ratio = self.state.wheel_assemblies[i].slip_ratio_raw(w_v_long);
            let slip_ratio = raw_slip_ratio.clamp(-1.0, 1.0);
            let (fx_tire, fy_tire) = combined_slip_forces(
                raw_slip_ratio,
                slip_angle,
                envelope,
                &self.state.wheel_assemblies[i].config.tire_model,
            );

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
            let rr_force = -rr_coeff * fz * fast_tanh_clip(w_v_long / 0.5);

            // Low-speed lateral stabilization: below ~3.0 m/s the explicit chassis integration of
            // tire yaw damping violates its stability limit, so lateral force fades out.
            let low_speed_blend = (w_v_long.abs() / 3.0).clamp(0.05, 1.0);
            let fy = fy_tire * low_speed_blend;
            // Rolling resistance shares the tire's longitudinal budget (braking on grass is still
            // grip-limited), but its own drag is always available so off-track coasting slows the car.
            let fx_room = (envelope * envelope - fy * fy)
                .max(0.0)
                .sqrt()
                .max(rr_force.abs());
            let fx = (fx_tire + rr_force).clamp(-fx_room, fx_room);

            // Step thermal dissipation and mechanical tread wear
            self.state.wheel_assemblies[i].step_thermal_and_wear(
                fx,
                fy,
                slip_ratio,
                slip_angle,
                wheel_v_world.length(),
                surf,
                dt,
            );

            // Skid telemetry (suppressed in mid-air)
            let (skid_intensity, is_skidding) =
                if self.state.is_airborne || self.state.elevation > 0.0 {
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
            let torque =
                offset_world.x * wheel_force_world.y - offset_world.y * wheel_force_world.x;
            total_wheel_torque += torque;

            let (new_dirt, new_dirt_surface) =
                if surf.is_loose_deformable() || surf == SurfaceType::Grass {
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

            self.state.suspension[i] = SuspensionTelemetry {
                deflection: strokes[i],
                deflection_velocity: stroke_vels[i],
                normal_force: normal_loads[i],
                dynamic_camber: dynamic_cambers[i],
                bottomed_out: bottomed_outs[i],
            };
        }
        self.state.abs_active = abs_active;

        // 5. Aerodynamic drag, yaw damping, and ESC
        let avg_surface_drag: f32 = surfaces
            .iter()
            .map(|s| s.surface_drag_multiplier())
            .sum::<f32>()
            / 4.0;
        let avg_surface_mu: f32 = surfaces
            .iter()
            .map(|s| s.friction_coefficient())
            .sum::<f32>()
            / 4.0;
        let effective_drag_coeff = self.effective_air_drag_coefficient();
        let drag_fwd = -effective_drag_coeff * v_long * v_long.abs() * avg_surface_drag;
        let drag_lat =
            -self.config.lateral_drag_coefficient * v_lat * v_lat.abs() * avg_surface_drag;
        let drag_world = fwd * drag_fwd + right * drag_lat;

        let base_yaw_damping = -self.config.angular_damping * omega;

        let mut esc_torque = 0.0f32;
        let mut esc_active = false;

        if self.config.assists.esc_enabled
            && self.state.speed > 2.5
            && v_long.abs() > 0.5
            && !(self.config.assists.handbrake_bypass && clamped_ctrl.handbrake)
        {
            // Max physical yaw rate governed by tire grip and aerodynamic downforce
            let downforce_load = self.config.downforce_coefficient * v_long * v_long;
            let effective_g = g + (downforce_load / self.config.mass.max(1.0));
            // The 0.60 rad/s floor only applies at parking speeds; above ~16 m/s it used to exceed
            // the grip limit (mu*g/v) and switched ESC off in fast corners (Spec 043, review B6).
            let speed_fade = ((v_long.abs() - 8.0) / 8.0).clamp(0.0, 1.0);
            let low_speed_floor = 0.60 * (1.0 - speed_fade * speed_fade * (3.0 - 2.0 * speed_fade));
            // Effective surface grip (ice studs, dirt contamination) times tire grip
            let grip_mu = surface_mus.iter().sum::<f32>() * 0.25 * self.config.tire.grip;
            let max_physical_yaw_rate =
                ((grip_mu * effective_g) / v_long.abs().max(2.0)).max(low_speed_floor);
            let target_yaw_rate =
                kinematic_yaw_rate.clamp(-max_physical_yaw_rate, max_physical_yaw_rate);

            let yaw_error = omega - target_yaw_rate;
            // ESC targets oversteer (rotating faster into turn than commanded, opposite to target, or uncommanded yaw)
            let is_oversteering = (omega.signum() == target_yaw_rate.signum()
                && omega.abs() > (target_yaw_rate.abs() + 0.06))
                || (omega.signum() != target_yaw_rate.signum() && omega.abs() > 0.10)
                || (target_yaw_rate.abs() < 0.05 && omega.abs() > 0.08);

            if is_oversteering {
                let yaw_thresh = self.config.assists.esc_yaw_threshold;
                if yaw_error.abs() > yaw_thresh {
                    let excess_yaw = (yaw_error.abs() - yaw_thresh) * yaw_error.signum();
                    let speed_boost = 1.0 + (self.state.speed / 20.0).min(3.5);
                    let esc_gain =
                        self.config.inertia * 10.0 * speed_boost * self.config.assists.esc_strength;
                    esc_torque = -excess_yaw * esc_gain;
                    esc_active = true;
                }
            }

            // Sideslip control (Spec 043): a real ESC also caps body slip. At the limit a car can
            // rotate slowly with a yaw error under the threshold while sideslip keeps growing
            // (sports car, 45 m/s, full input: +0.13 rad/s of sideslip with the yaw term alone).
            let beta = self.state.sideslip_angle;
            let strength = self.config.assists.esc_strength.clamp(0.0, 1.0);
            let beta_limit = self
                .config
                .assists
                .esc_sideslip_limit_deg
                .to_radians()
                .clamp(0.0, PI);
            if strength > 0.0 && beta.abs() > beta_limit {
                let speed_boost = 1.0 + (self.state.speed / 20.0).min(3.5);
                let beta_gain = self.config.inertia * 8.0 * speed_boost * strength;
                esc_torque -= (beta.abs() - beta_limit) * beta.signum() * beta_gain;
                esc_active = true;
            }
        }
        self.state.esc_active = esc_active;
        self.state.esc_corrective_torque = esc_torque.abs();

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
        let static_friction_world =
            if !self.state.is_airborne && ground_contact > 0.0 && self.state.speed < 0.25 {
            let total_slope_gravity = bank_gravity_world + grade_gravity_world;
            if total_slope_gravity.length_squared() > 1e-4 {
                let static_blend = (1.0 - (self.state.speed / 0.25)).clamp(0.0, 1.0);
                let max_static_friction = avg_surface_mu * total_normal_load * ground_contact;

                // Lateral holding: tires cannot roll sideways, so static friction resists lateral slope force
                let lat_slope_force = total_slope_gravity.dot(right);
                    let lat_holding = -lat_slope_force
                        .clamp(-max_static_friction, max_static_friction)
                        * static_blend;

                // Longitudinal holding: resisted by brakes/handbrake or rolling resistance
                let long_slope_force = total_slope_gravity.dot(fwd);
                let long_holding = if is_holding_brakes {
                        -long_slope_force.clamp(-max_static_friction, max_static_friction)
                            * static_blend
                } else {
                        let rr_holding_cap =
                            self.config.rolling_resistance_coefficient * total_normal_load;
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
        let net_force_world = total_wheel_force_world
            + drag_world
            + bank_gravity_world
            + grade_gravity_world
            + static_friction_world;
        let net_torque = total_wheel_torque + yaw_damping_torque;

        let linear_accel_world = net_force_world / self.config.mass;
        let angular_accel = net_torque / self.config.inertia;

        // Local acceleration for next frame weight transfer
        let accel_long = linear_accel_world.dot(fwd);
        let accel_lat = linear_accel_world.dot(right);
        // First-order load transfer response at `weight_transfer_hz` (also removes numerical chatter)
        let alpha_filter = 1.0 - (-2.0 * PI * self.config.weight_transfer_hz.max(0.1) * dt).exp();
        self.state.acceleration_local = self.state.acceleration_local * (1.0 - alpha_filter)
            + Vec2::new(accel_long, accel_lat) * alpha_filter;

        // 7. Numerical Integration (Semi-implicit Euler)
        self.state.velocity += linear_accel_world * dt;
        self.state.angular_velocity += angular_accel * dt;

        // Low speed resting lock to prevent micro-jitter when stopped on flat ground or when holding brakes.
        // A slope is only "too steep" to remain static if the incline angle exceeds the static friction limit:
        // tan(theta) > mu for lateral banking, or grade exceeds rolling/braking limits.
        if self.state.speed < 0.08 {
            let on_steep_bank = bank_rad.abs().tan() > avg_surface_mu;
            let on_steep_grade = if is_holding_brakes {
                grade_rad.abs().tan() > avg_surface_mu
            } else {
                grade_rad.abs() > 0.02
            };
            let on_steep_slope = on_steep_bank || on_steep_grade;
            let drive_force_mag = if clamped_ctrl.reverse {
                clamped_ctrl.throttle * self.config.max_reverse_force
            } else {
                clamped_ctrl.throttle * self.config.max_engine_force
            };
            let brake_holding_mag = clamped_ctrl.brake * self.config.max_brake_force
                + if clamped_ctrl.handbrake {
                    self.config.handbrake_force
                } else {
                    0.0
                };
            let brakes_overpower_engine = is_holding_brakes && brake_holding_mag >= drive_force_mag;

            if (clamped_ctrl.throttle < 1e-3 || brakes_overpower_engine)
                && (is_holding_brakes || !on_steep_slope)
            {
                self.state.velocity = Vec2::ZERO;
                self.state.angular_velocity = 0.0;
            }
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

        let is_any_rear_skidding =
            self.state.wheels[2].is_skidding || self.state.wheels[3].is_skidding;
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
    pub fn launch_jump_with_height(
        &mut self,
        direction: Vec2,
        ramp_angle_deg: f32,
        takeoff_elevation: f32,
    ) {
        let dir = direction.normalize_or_zero();
        let speed_along_dir = self.state.velocity.dot(dir).max(0.0);
        let angle_rad = ramp_angle_deg.to_radians();
        let (sin_theta, cos_theta) = angle_rad.sin_cos();

        // Realistic vertical launch velocity from incline angle and suspension compliance:
        // Long-travel chassis suspension absorbs ~25% of vertical impulse upon climbing the curve.
        // Degradation reduces absorption: eta = 0.25 * min(H_susp) (Spec 078 Section 5.C)
        let min_susp = self.state.suspension_health.iter().copied().fold(1.0f32, f32::min);
        let eff_min_susp = if min_susp >= 0.85 { 1.0 } else { min_susp / 0.85 };
        let absorption = 0.25 * eff_min_susp;
        let suspension_efficiency = 1.0 - absorption;
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
    pub fn try_trigger_jump(&mut self, is_on_ramp: bool, ramp: &JumpRampProperties) -> bool {
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
    fn low_speed_authority_expands_progressively_and_preserves_profile_order() {
        let mut cfg = CarConfig::sports_car();
        let car = Car::new(cfg.clone());
        let stock = [0.90f32, 1.0, 1.07, 1.15].map(|p| {
            let mut c = Car::new(cfg.clone());
            c.config.player.steer_overslip = p;
            c.steer_authority(12.0, 1.0)
        });
        for pair in stock.windows(2) {
            assert!(pair[0] < pair[1], "preset authority order collapsed: {stock:?}");
        }
        let base = car.steer_authority_with(18.0, 1.0, 1.0);
        let expanded = car.steer_authority_with(15.5, 1.0, 1.0);
        assert!(expanded > base * 1.20, "low speed authority did not expand enough");
        let high_speed = car.steer_authority_with(28.0, 1.0, 1.0);
        assert!((high_speed - car.steer_authority_with(28.0, 1.0, 1.0)).abs() < 1e-6);
        cfg.max_steer_angle = 0.2;
        let capped = Car::new(cfg).steer_authority_with(12.0, 1.0, 1.0);
        assert!(capped <= 0.2);
    }

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
        assert!(
            fr_r.abs() > fl_r.abs(),
            "Inner wheel FR ({fr_r}) must steer more than outer FL ({fl_r})"
        );

        // Left turn (steer_angle > 0, counter-clockwise): inner wheel FL has larger positive angle than outer FR
        let (fl_l, fr_l) = car.compute_ackermann_angles(0.3);
        assert!(fl_l > 0.0);
        assert!(fr_l > 0.0);
        assert!(
            fl_l > fr_l,
            "Inner wheel FL ({}) must exceed outer FR ({})",
            fl_l,
            fr_l
        );
    }

    #[test]
    fn test_straight_line_step() {
        let mut car = Car::new(CarConfig::sports_car());
        let controls = CarControls::new(1.0, 0.0, 0.0, false);

        for _ in 0..60 {
            car.step(&controls, SurfaceType::Asphalt, 1.0 / 60.0);
        }

        assert!(
            car.state.speed > 5.0,
            "Car should accelerate forward, speed is {}",
            car.state.speed
        );
        assert!(car.state.position.x > 1.0, "Car should move in +X");
        assert!(
            car.state.position.y.abs() < 1e-3,
            "Car should not deviate laterally"
        );
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
        assert!(
            car.state.local_velocity.x < -3.0,
            "Car should accelerate backward, was {}",
            car.state.local_velocity.x
        );
        assert!(
            car.state.steer_angle.abs() < 1e-3,
            "Steer angle should remain zero without input"
        );
        assert!(
            car.state.angle.abs() < 1e-3,
            "Car should not deviate or force turning, angle was {}",
            car.state.angle
        );

        // Active steering in reverse turns the car
        let mut car_steer = Car::new(CarConfig::sports_car());
        let mut ctrl_steer = CarControls::new(1.0, 0.3, 0.0, false);
        ctrl_steer.reverse = true;
        for _ in 0..120 {
            car_steer.step(&ctrl_steer, SurfaceType::Asphalt, dt);
        }
        assert!(
            car_steer.state.angular_velocity.abs() > 0.1,
            "Steering in reverse should turn the car"
        );

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
        assert!(
            (car_turn.state.angle - angle_at_stop).abs() < 0.01,
            "Car should reverse straight along stopped heading"
        );
        assert!(
            car_turn.state.angular_velocity.abs() < 1e-3,
            "Angular velocity should settle to zero"
        );

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
        assert!(
            car_straighten.state.angular_velocity.abs() < 1e-3,
            "Releasing steering in reverse must eliminate yaw rate"
        );
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
        assert!(
            (car_straight.state.angle - (-0.23337)).abs() < 1e-3,
            "Car must not turn involuntarily in reverse"
        );
        assert!(
            car_straight.state.angular_velocity.abs() < 1e-3,
            "Reverse yaw rate must remain stable"
        );

        // 2. Reversing with yaw disturbance damps smoothly to zero without sign oscillations
        let mut car_disturbed = Car::new(config);
        car_disturbed.state.angular_velocity = 0.05;
        for _ in 0..60 {
            car_disturbed.step(&ctrl_neutral, SurfaceType::Asphalt, dt);
        }
        assert!(
            car_disturbed.state.angular_velocity.abs() < 1e-3,
            "Reverse yaw disturbance must damp out smoothly"
        );

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

        assert!(
            car_right.state.angular_velocity.abs() > 0.1,
            "Right steering must produce yaw in reverse"
        );
        assert!(
            car_left.state.angular_velocity.abs() > 0.1,
            "Left steering must produce yaw in reverse"
        );
        assert!(
            (car_right.state.angular_velocity.abs() - car_left.state.angular_velocity.abs()).abs()
                < 1e-4,
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
        config.tire.peak_slip_angle_deg = 7.8;
        config.tire.grip = 1.20;
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
        println!(
            "After 2s accel: speed={:.2}, v_long={:.2}",
            car_fwd_rev.state.speed, car_fwd_rev.state.local_velocity.x
        );
        // Brake to full stop
        let ctrl_brake = CarControls::full_brake();
        let mut brake_frames = 0;
        while car_fwd_rev.state.local_velocity.x > 0.05 && brake_frames < 300 {
            car_fwd_rev.step(&ctrl_brake, SurfaceType::Asphalt, dt);
            brake_frames += 1;
        }
        println!(
            "Stopped after {} brake frames: speed={:.3}, v_long={:.3}",
            brake_frames, car_fwd_rev.state.speed, car_fwd_rev.state.local_velocity.x
        );

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

        println!(
            "\n=== SIMULATION: Reversing while turning for 180 frames then reverse steering ==="
        );
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
        assert_eq!(
            sports_step.state.wheels[0].longitudinal_force, 0.0,
            "RWD front left wheel should have no drive force"
        );
        assert_eq!(
            sports_step.state.wheels[1].longitudinal_force, 0.0,
            "RWD front right wheel should have no drive force"
        );
        assert!(
            sports_step.state.wheels[2].longitudinal_force < 0.0,
            "RWD rear left wheel must have reverse drive force"
        );
        assert!(
            sports_step.state.wheels[3].longitudinal_force < 0.0,
            "RWD rear right wheel must have reverse drive force"
        );

        let mut rally_step = rally;
        rally_step.step(&ctrl, SurfaceType::Asphalt, dt);
        // Rally car AWD: all 4 wheels receive reverse drive torque
        assert!(
            rally_step.state.wheels[0].longitudinal_force < 0.0,
            "AWD front left wheel must receive reverse drive"
        );
        assert!(
            rally_step.state.wheels[1].longitudinal_force < 0.0,
            "AWD front right wheel must receive reverse drive"
        );
        assert!(
            rally_step.state.wheels[2].longitudinal_force < 0.0,
            "AWD rear left wheel must receive reverse drive"
        );
        assert!(
            rally_step.state.wheels[3].longitudinal_force < 0.0,
            "AWD rear right wheel must receive reverse drive"
        );
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
        assert!(
            !car.state.is_braking,
            "Accelerating must not set is_braking"
        );

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
        assert!(car.state.is_braking, "Handbrake must set is_braking = true");
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
            car_asphalt.state.speed, 0.0,
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

    #[test]
    fn test_reverse_handbrake() {
        let mut car = Car::new(CarConfig::sports_car());
        let dt = 1.0 / 60.0;
        let mut ctrl_rev = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl_rev.reverse = true;
        for _ in 0..120 {
            car.step(&ctrl_rev, SurfaceType::Asphalt, dt);
        }
        let speed_before = car.state.local_velocity.x;
        assert!(
            speed_before < -5.0,
            "Car should reach reverse speed before handbrake, was {:.2}",
            speed_before
        );

        let mut ctrl_hb = CarControls::new(0.0, 0.0, 0.0, true);
        ctrl_hb.reverse = true;
        let mut locked_during_brake = false;
        for _ in 0..180 {
            car.step(&ctrl_hb, SurfaceType::Asphalt, dt);
            if car.state.wheels[2].is_locked && car.state.wheels[3].is_locked {
                locked_during_brake = true;
            }
        }
        assert!(
            locked_during_brake,
            "Rear wheels must lock up when handbrake is engaged in reverse"
        );
        assert_eq!(
            car.state.speed, 0.0,
            "Handbrake must bring reversing car to a complete stop and hold it"
        );
        assert_eq!(
            car.state.wheels[2].longitudinal_force, 0.0,
            "Resting lock must eliminate longitudinal force when held"
        );

        // Test reverse handbrake turn with steering (J-turn / slide)
        let mut car_rev_turn = Car::new(CarConfig::sports_car());
        let mut ctrl_rev = CarControls::new(1.0, 0.0, 0.0, false);
        ctrl_rev.reverse = true;
        for _ in 0..120 {
            car_rev_turn.step(&ctrl_rev, SurfaceType::Asphalt, dt);
        }

        let mut ctrl_hb_turn = CarControls::new(0.8, 1.0, 0.0, true);
        ctrl_hb_turn.reverse = true;
        let mut entered_drift = false;
        let mut max_yaw: f32 = 0.0;
        let mut max_sideslip: f32 = 0.0;
        for _ in 0..60 {
            car_rev_turn.step(&ctrl_hb_turn, SurfaceType::Asphalt, dt);
            let yaw = car_rev_turn.state.angular_velocity.abs();
            let ss = car_rev_turn.state.sideslip_angle.abs();
            max_yaw = max_yaw.max(yaw);
            max_sideslip = max_sideslip.max(ss);
            if car_rev_turn.state.is_drifting {
                entered_drift = true;
            }
        }
        assert!(
            entered_drift,
            "Steering with handbrake in reverse must initiate drift state"
        );
        assert!(
            max_yaw > 1.0,
            "Reverse handbrake turn must generate high yaw rate, got {:.2} rad/s",
            max_yaw
        );
        assert!(
            max_sideslip > 0.4,
            "Reverse handbrake turn must generate high sideslip, got {:.2} rad",
            max_sideslip
        );
    }

    #[test]
    fn test_directional_impact_classification_and_damage_partitioning() {
        let mut car_front = Car::new(CarConfig::sports_car()); // FrontEngine
        assert_eq!(car_front.config.engine_placement, EnginePlacement::FrontEngine);

        // Test zone classification in local coordinates
        assert_eq!(car_front.classify_impact_zone_local(1.5, 0.0), ImpactZone::FrontNose);
        assert_eq!(car_front.classify_impact_zone_local(-1.5, 0.0), ImpactZone::RearTail);
        assert_eq!(car_front.classify_impact_zone_local(1.5, -0.9), ImpactZone::CornerFL);
        assert_eq!(car_front.classify_impact_zone_local(1.5, 0.9), ImpactZone::CornerFR);
        assert_eq!(car_front.classify_impact_zone_local(-1.5, -0.9), ImpactZone::CornerRL);
        assert_eq!(car_front.classify_impact_zone_local(-1.5, 0.9), ImpactZone::CornerRR);
        assert_eq!(car_front.classify_impact_zone_local(0.0, -0.9), ImpactZone::FlankLeft);
        assert_eq!(car_front.classify_impact_zone_local(0.0, 0.9), ImpactZone::FlankRight);

        // Test FrontEngine takes heavy engine damage on FrontNose impact
        car_front.apply_collision_damage_to_zone(ImpactZone::FrontNose, 2000.0);
        assert!(car_front.state.engine_health < 0.80, "FrontEngine should suffer significant engine damage on head-on collision");
        assert!(car_front.state.chassis_health < 1.0);

        // Test RearEngine takes 0 engine damage on FrontNose impact
        let mut car_rear = Car::new(CarConfig::sand_rail()); // RearEngine
        assert_eq!(car_rear.config.engine_placement, EnginePlacement::RearEngine);
        car_rear.apply_collision_damage_to_zone(ImpactZone::FrontNose, 2000.0);
        assert_eq!(car_rear.state.engine_health, 1.0, "RearEngine must suffer 0% engine damage on head-on nose collision");
        assert!(car_rear.state.chassis_health < 1.0, "RearEngine chassis absorbs front impact");

        // Test CornerFL impact damages FL suspension heavily
        let mut car_corner = Car::new(CarConfig::sports_car());
        car_corner.apply_collision_damage_to_zone(ImpactZone::CornerFL, 2000.0);
        assert!(car_corner.state.suspension_health[0] < car_corner.state.suspension_health[1], "FL suspension should take the brunt of CornerFL impact");
        assert_eq!(car_corner.state.suspension_health[2], 1.0);
        assert_eq!(car_corner.state.suspension_health[3], 1.0);

        // Test field repair ceilings (0.70 chassis/engine, 0.60 suspension)
        car_front.state.chassis_health = 0.10;
        car_front.state.engine_health = 0.10;
        car_front.state.suspension_health = [0.10, 0.10, 0.10, 0.10];
        car_front.apply_field_repair(1.0);
        assert!((car_front.state.chassis_health - 0.70).abs() < 1e-4);
        assert!((car_front.state.engine_health - 0.70).abs() < 1e-4);
        assert!((car_front.state.suspension_health[0] - 0.60).abs() < 1e-4);

        // Test full garage repair restores 100%
        car_front.full_garage_repair();
        assert_eq!(car_front.state.chassis_health, 1.0);
        assert_eq!(car_front.state.engine_health, 1.0);
        assert_eq!(car_front.state.suspension_health, [1.0, 1.0, 1.0, 1.0]);
    }
}

