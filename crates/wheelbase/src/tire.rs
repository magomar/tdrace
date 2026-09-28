//! Tire, wheel spin and contact models.
//!
//! Governed by `specs/043_vehicle_dynamics_rebuild_and_simplified_handling_settings.md`:
//! normalized combined-slip tire with load sensitivity, and implicit wheel spin.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::config::{PacejkaTireConfig, TireConfig, WheelAssemblyConfig};
use super::surface::SurfaceType;

/// Floor on the slip-ratio reference speed (m/s).
///
/// Keeps the slip definition finite at standstill and bounds the longitudinal tire stiffness
/// seen by the explicit chassis integrator: with `C / (v_ref * m_corner) * dt < 2` the
/// wheel-chassis contact stays stable down to 0 m/s at 60 Hz and 120 Hz.
pub const SLIP_REFERENCE_SPEED: f32 = 4.0;

/// Default baseline tire temperature (nominal warm tire in °C).
pub fn default_tire_temperature() -> f32 {
    65.0
}

/// Identifier for each of the 4 wheels on the chassis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WheelId {
    FrontLeft = 0,
    FrontRight = 1,
    RearLeft = 2,
    RearRight = 3,
}

impl WheelId {
    pub const ALL: [Self; 4] = [
        Self::FrontLeft,
        Self::FrontRight,
        Self::RearLeft,
        Self::RearRight,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn is_front(self) -> bool {
        matches!(self, Self::FrontLeft | Self::FrontRight)
    }

    #[inline]
    pub const fn is_rear(self) -> bool {
        matches!(self, Self::RearLeft | Self::RearRight)
    }

    #[inline]
    pub const fn is_left(self) -> bool {
        matches!(self, Self::FrontLeft | Self::RearLeft)
    }

    #[inline]
    pub const fn is_right(self) -> bool {
        matches!(self, Self::FrontRight | Self::RearRight)
    }
}

/// Comprehensive telemetry data for an individual wheel contact patch.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WheelTelemetry {
    /// Identifier of the wheel.
    pub id: WheelId,
    /// Slip angle (angle between wheel heading and actual contact patch velocity) in radians.
    pub slip_angle: f32,
    /// Longitudinal slip ratio (relative difference between wheel rotational speed and road speed).
    pub slip_ratio: f32,
    /// Dynamic normal vertical load on this tire in Newtons (Fz).
    pub normal_load: f32,
    /// Generated lateral cornering force in wheel frame in Newtons (Fy).
    pub lateral_force: f32,
    /// Generated longitudinal drive/braking force in wheel frame in Newtons (Fx).
    pub longitudinal_force: f32,
    /// Steering angle of the wheel relative to vehicle centerline in radians.
    pub steer_angle: f32,
    /// Contact patch linear velocity vector in world coordinates (m/s).
    pub world_velocity: Vec2,
    /// Contact patch position in world coordinates (meters).
    pub wheel_pos_world: Vec2,
    /// Skid intensity normalized to [0.0, 1.0] for tire smoke, skid marks, and sound effects.
    pub skid_intensity: f32,
    /// Whether this tire is currently actively slipping/skidding.
    pub is_skidding: bool,
    /// Surface type currently under this wheel.
    pub surface: SurfaceType,
    /// Surface contamination level [0.0 = clean rubber, 1.0 = heavily coated with off-track dirt/gravel/sand/mud].
    #[serde(default)]
    pub dirt_contamination: f32,
    /// Surface material that coated the tire tread.
    #[serde(default)]
    pub dirt_surface: SurfaceType,
    /// Rotational angular velocity of the wheel in radians/second.
    #[serde(default)]
    pub angular_velocity: f32,
    /// Tread surface bulk temperature in degrees Celsius (nominal ~80-105 C).
    #[serde(default = "default_tire_temperature")]
    pub temperature: f32,
    /// Mechanical tread wear accumulated [0.0 = brand new, 1.0 = worn out].
    #[serde(default)]
    pub wear: f32,
    /// Whether the wheel is currently locked under braking (omega == 0 while vehicle is moving).
    #[serde(default)]
    pub is_locked: bool,
}

impl Default for WheelTelemetry {
    fn default() -> Self {
        Self {
            id: WheelId::FrontLeft,
            slip_angle: 0.0,
            slip_ratio: 0.0,
            normal_load: 0.0,
            lateral_force: 0.0,
            longitudinal_force: 0.0,
            steer_angle: 0.0,
            world_velocity: Vec2::ZERO,
            wheel_pos_world: Vec2::ZERO,
            skid_intensity: 0.0,
            is_skidding: false,
            surface: SurfaceType::Asphalt,
            dirt_contamination: 0.0,
            dirt_surface: SurfaceType::Asphalt,
            angular_velocity: 0.0,
            temperature: default_tire_temperature(),
            wear: 0.0,
            is_locked: false,
        }
    }
}

/// Computes pure lateral force using the Pacejka Magic Formula curve adapted for arcade drifting.
///
/// Used by the motorbike model. Car tires use [`combined_slip_forces`].
/// Returns lateral force Fy in Newtons.
#[inline]
pub fn pacejka_lateral_force(
    slip_angle: f32,
    normal_load: f32,
    friction_coeff: f32,
    config: &PacejkaTireConfig,
    is_handbraking: bool,
) -> f32 {
    if normal_load <= 1e-4 {
        return 0.0;
    }

    let mu = if is_handbraking {
        friction_coeff * config.handbrake_lateral_friction_multiplier
    } else {
        friction_coeff
    };

    let b = config.stiffness_b;
    let c = config.shape_c;
    let d = mu * normal_load * config.peak_d;
    let e = config.curvature_e;

    // Pacejka formulation: Fy = D * sin(C * arctan(B*alpha - E*(B*alpha - arctan(B*alpha))))
    let b_alpha = b * slip_angle;
    let inner = b_alpha - e * (b_alpha - b_alpha.atan());
    let base_force = d * (c * inner.atan()).sin();

    // Post-peak slide retention for smooth arcade drift control
    let abs_slip = slip_angle.abs();
    let peak_slip = 0.22; // ~12.6 degrees peak grip
    if abs_slip > peak_slip {
        let slide_d = mu * normal_load * config.drift_slide_friction;
        let slide_force = slide_d * slip_angle.signum();
        // Blend from peak to sliding plateau smoothly
        let blend = ((abs_slip - peak_slip) / 0.35).min(1.0);
        base_force * (1.0 - blend) + slide_force * blend
    } else {
        base_force
    }
}

/// Shape of the rising branch: a Pacejka curve (C = 1.45, E = -0.15) rescaled so it peaks at s = 1.
/// Its initial slope (B * C = 2.56) matches a real tire's cornering stiffness at a given peak slip;
/// a softer rise (e.g. s * (2 - s), slope 2) under-damps the chassis yaw.
const CURVE_C: f32 = 1.45;
const CURVE_E: f32 = -0.15;
/// `B` such that `C * atan(B - E * (B - atan(B))) = PI / 2` (peak at s = 1).
const CURVE_B: f32 = 1.7646;

/// Normalized tire curve (Spec 043): 0 at s = 0, 1.0 at the peak (s = 1), then a smooth fall
/// to `slide_grip` over `falloff` peak-widths.
#[inline]
pub fn normalized_grip_curve(s: f32, tire: &TireConfig) -> f32 {
    let s = s.abs();
    if s <= 1.0 {
        let bs = CURVE_B * s;
        (CURVE_C * (bs - CURVE_E * (bs - bs.atan())).atan()).sin().min(1.0)
    } else {
        let t = ((s - 1.0) / tire.falloff.max(0.1)).min(1.0);
        1.0 - (1.0 - tire.slide_grip) * t * t * (3.0 - 2.0 * t)
    }
}

/// Load-sensitive friction envelope `mu_eff * Fz` in Newtons.
///
/// Heavily loaded tires deliver less grip per newton of load, so lateral load transfer
/// reduces an axle's total grip and the roll balance moves the handling balance.
#[inline]
pub fn tire_friction_envelope(normal_load: f32, nominal_load: f32, friction_coeff: f32, tire: &TireConfig) -> f32 {
    if normal_load <= 1e-4 {
        return 0.0;
    }
    let load_ratio = normal_load / nominal_load.max(1.0);
    let sensitivity = (1.0 - tire.load_sensitivity * (load_ratio - 1.0)).clamp(0.5, 1.3);
    friction_coeff * tire.grip * sensitivity * normal_load
}

/// Combined-slip tire forces on the normalized slip vector (Spec 043).
///
/// `sx = slip_ratio / peak_slip_ratio`, `sy = tan(slip_angle) / tan(peak_slip_angle)`.
/// The resultant `F = envelope * curve(|s|)` points along the slip vector, so wheelspin and
/// lock-up erode lateral grip naturally. `power_slide < 1` blends towards the friction-circle
/// budget, which keeps more lateral grip under longitudinal slip (arcade). Returns `(Fx, Fy)` in the wheel frame: `Fx > 0` pushes the car
/// forward (wheel spinning faster than the road), `Fy` has the sign of `slip_angle`.
#[inline]
pub fn combined_slip_forces(slip_ratio: f32, slip_angle: f32, envelope: f32, tire: &TireConfig) -> (f32, f32) {
    if envelope <= 1e-4 {
        return (0.0, 0.0);
    }
    let sx = slip_ratio / tire.peak_slip_ratio.max(1e-3);
    let sy = slip_angle.tan() / tire.peak_slip_angle().tan().max(1e-3);
    let s = (sx * sx + sy * sy).sqrt();
    if s < 1e-6 {
        return (0.0, 0.0);
    }
    let f = envelope * normalized_grip_curve(s, tire);
    let fx = f * sx / s;
    let mut fy = f * sy / s;
    let arcade = 1.0 - tire.power_slide.clamp(0.0, 1.0);
    if arcade > 0.0 {
        // Lateral force may use whatever the friction circle leaves after Fx.
        let fy_pure = envelope * normalized_grip_curve(sy, tire) * sy.signum();
        let budget = (1.0 - (fx / envelope).powi(2)).max(0.0).sqrt();
        let fy_circle = fy_pure * budget;
        if fy_circle.abs() > fy.abs() {
            fy += (fy_circle - fy) * arcade;
        }
    }
    (fx, fy)
}

/// Longitudinal stiffness `dFx/d(slip_ratio)` at the given combined slip state (N per unit slip).
///
/// Used by the implicit wheel spin integrator. Evaluated by central difference and floored at a
/// small positive value so the implicit step stays well conditioned past the peak.
#[inline]
pub fn longitudinal_slip_stiffness(slip_ratio: f32, slip_angle: f32, envelope: f32, tire: &TireConfig) -> f32 {
    let h = tire.peak_slip_ratio.max(1e-3) * 0.05;
    let (fx_hi, _) = combined_slip_forces(slip_ratio + h, slip_angle, envelope, tire);
    let (fx_lo, _) = combined_slip_forces(slip_ratio - h, slip_angle, envelope, tire);
    let slope = (fx_hi - fx_lo) / (2.0 * h);
    slope.max(envelope * 0.05)
}

/// Applies the friction circle / ellipse limit to combine longitudinal and lateral forces.
///
/// Ensures that sqrt(Fx^2 + Fy^2) <= mu * Fz.
#[inline]
pub fn solve_combined_slip_forces(
    fx_demand: f32,
    fy_demand: f32,
    max_friction_force: f32,
) -> (f32, f32) {
    if max_friction_force <= 1e-4 {
        return (0.0, 0.0);
    }

    let total_demand_sq = fx_demand * fx_demand + fy_demand * fy_demand;
    let max_force_sq = max_friction_force * max_friction_force;

    if total_demand_sq > max_force_sq {
        let scale = max_friction_force / total_demand_sq.sqrt();
        (fx_demand * scale, fy_demand * scale)
    } else {
        (fx_demand, fy_demand)
    }
}

/// Calculates normalized skid intensity [0.0, 1.0] and skidding status.
#[inline]
pub fn compute_skid_telemetry(
    slip_angle: f32,
    slip_ratio: f32,
    speed: f32,
    is_handbraking: bool,
    config: &TireConfig,
    surface: SurfaceType,
) -> (f32, bool) {
    if speed < 0.5 {
        return (0.0, false);
    }

    let abs_slip = slip_angle.abs();
    let lat_intensity = if abs_slip > config.skid_threshold {
        let range = (config.skid_full_threshold - config.skid_threshold).max(1e-3);
        ((abs_slip - config.skid_threshold) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let abs_ratio = slip_ratio.abs();
    let long_intensity = if abs_ratio > 0.15 {
        ((abs_ratio - 0.15) / 0.50).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut intensity = (lat_intensity + long_intensity * 0.7).clamp(0.0, 1.0);

    if is_handbraking && speed > 1.5 {
        intensity = intensity.max(0.75);
    }

    // A locked wheel under braking produces maximum skid smoke telemetry
    if (abs_ratio >= 0.85 || slip_ratio <= -0.85) && speed > 1.0 {
        intensity = 1.0;
    }

    let is_skidding = (intensity > 0.08 || slip_ratio <= -0.85)
        && (surface.produces_tire_smoke() || surface.produces_debris_particles());

    (intensity, is_skidding)
}

/// Physical wheel assembly combining configuration, rotational dynamics, and thermal wear state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WheelAssembly {
    /// Assembly configuration (radius, width, inertia, compound, torque distribution).
    pub config: WheelAssemblyConfig,
    /// Rotational angular velocity of the wheel in radians/second.
    pub angular_velocity: f32,
    /// Tread surface bulk temperature in degrees Celsius.
    pub temperature: f32,
    /// Mechanical tread wear accumulated [0.0 = brand new, 1.0 = worn out].
    pub wear: f32,
    /// Whether the wheel is currently locked under braking.
    pub is_locked: bool,
}

impl Default for WheelAssembly {
    fn default() -> Self {
        Self::new(WheelAssemblyConfig::default())
    }
}

impl WheelAssembly {
    /// Creates a new wheel assembly initialized at rest with nominal warm temperature (65°C) and fresh tread.
    pub fn new(config: WheelAssemblyConfig) -> Self {
        Self {
            config,
            angular_velocity: 0.0,
            temperature: default_tire_temperature(),
            wear: 0.0,
            is_locked: false,
        }
    }

    /// Evaluates the dynamic thermal grip multiplier based on tread temperature T.
    ///
    /// Optimal operating window is [80°C, 105°C] where grip reaches ~1.05.
    /// Below 45°C (cold tire), grip is reduced (~0.88 - 0.96).
    /// Above 120°C (overheated tire), grip degrades by >= 15% down towards 0.55 clamp limit.
    pub fn thermal_grip_multiplier(&self) -> f32 {
        let t = self.temperature;
        let mult = if t < 45.0 {
            // Cold tire: ramp from 0.96 at 0°C up to 1.00 at 45°C
            0.96 + (t.max(0.0) / 45.0) * 0.04
        } else if t < 75.0 {
            // Warm-up phase: ramp from 1.00 to 1.06 at 75°C
            1.00 + ((t - 45.0) / 30.0) * 0.06
        } else if t <= 105.0 {
            // Optimal peak grip window: 1.06
            1.06
        } else if t <= 120.0 {
            // Overheating transition: drops to 0.88 at 120°C (17% drop from peak, satisfying >=15% SLA)
            1.06 - ((t - 105.0) / 15.0) * 0.18
        } else {
            // Severe overheat: gradual degradation from 0.88 down to 0.82 at 200°C
            0.88 - ((t - 120.0) / 80.0) * 0.06
        };
        mult.clamp(0.82, 1.08)
    }

    /// Unclamped longitudinal slip ratio `(omega*r - v) / max(|v|, SLIP_REFERENCE_SPEED)` (Spec 043).
    ///
    /// Positive when the wheel spins faster than the road (drive), -1.0 when locked.
    #[inline]
    pub fn slip_ratio_raw(&self, v_long: f32) -> f32 {
        let v_wheel = self.angular_velocity * self.config.tire_radius;
        (v_wheel - v_long) / v_long.abs().max(SLIP_REFERENCE_SPEED)
    }

    /// Longitudinal slip ratio clamped to [-1.0, 1.0] for telemetry, skid and audio consumers.
    #[inline]
    pub fn compute_slip_ratio(&self, v_long: f32) -> f32 {
        self.slip_ratio_raw(v_long).clamp(-1.0, 1.0)
    }

    /// Wheel angular velocity that produces the given slip ratio at road speed `v_long`.
    #[inline]
    pub fn omega_for_slip(&self, slip_ratio: f32, v_long: f32) -> f32 {
        (v_long + slip_ratio * v_long.abs().max(SLIP_REFERENCE_SPEED)) / self.config.tire_radius.max(1e-2)
    }

    /// Integrates wheel spin with a linearized backward-Euler step (Spec 043):
    ///
    /// `I * d(omega)/dt = T_drive - T_brake - r * Fx(slip)`
    ///
    /// The tire force is linearized around the current slip, so the step stays stable at any
    /// road speed. Brake torque can stop the wheel but never reverse it.
    ///
    /// Parameters:
    /// - `drive_torque`: powertrain torque at the hub (N·m, sign = drive direction; coast torque is negative).
    /// - `brake_torque`: service brake + handbrake torque magnitude (N·m, >= 0).
    /// - `v_long`: contact patch speed along the wheel heading (m/s).
    /// - `slip_angle`: current slip angle (rad), for the combined-slip force.
    /// - `envelope`: current friction envelope of this tire (N).
    pub fn step_implicit(
        &mut self,
        drive_torque: f32,
        brake_torque: f32,
        v_long: f32,
        slip_angle: f32,
        envelope: f32,
        dt: f32,
    ) {
        let inertia = self.config.rotational_inertia.max(1e-3);
        let r = self.config.tire_radius.max(1e-2);
        let brake_torque = brake_torque.max(0.0);

        if !self.angular_velocity.is_finite() || !drive_torque.is_finite() || !brake_torque.is_finite() {
            self.angular_velocity = v_long / r;
            self.is_locked = false;
            return;
        }

        // Static hold when parked
        if v_long.abs() < 0.20 && brake_torque > 0.0 && drive_torque.abs() <= brake_torque {
            self.angular_velocity = 0.0;
            self.is_locked = false;
            return;
        }

        let tire = &self.config.tire_model;
        let slip = self.slip_ratio_raw(v_long);
        let (fx0, _) = combined_slip_forces(slip, slip_angle, envelope, tire);
        let stiffness = if envelope > 1e-4 {
            longitudinal_slip_stiffness(slip, slip_angle, envelope, tire)
        } else {
            0.0
        };
        let k = r * r * stiffness / v_long.abs().max(SLIP_REFERENCE_SPEED);
        let effective_inertia = inertia + dt * k;

        let omega_free = self.angular_velocity + dt * (drive_torque - r * fx0) / effective_inertia;
        let brake_dw = dt * brake_torque / effective_inertia;
        self.angular_velocity = if omega_free.abs() <= brake_dw {
            0.0
        } else {
            omega_free - brake_dw * omega_free.signum()
        };

        // Physical rotational limit (-550 to +550 rad/s ~ >600 km/h)
        self.angular_velocity = self.angular_velocity.clamp(-550.0, 550.0);
        self.is_locked = v_long.abs() > 0.5 && self.angular_velocity.abs() < 1e-3 && brake_torque > 0.0;
    }

    /// Effective rotational inertia `I + dt * r^2 * dFx/dslip / v_ref` seen by the implicit step.
    ///
    /// Used by the differential coupling so locking torques stay consistent with the tire.
    pub fn effective_inertia(&self, v_long: f32, slip_angle: f32, envelope: f32, dt: f32) -> f32 {
        let inertia = self.config.rotational_inertia.max(1e-3);
        if envelope <= 1e-4 {
            return inertia;
        }
        let r = self.config.tire_radius.max(1e-2);
        let slip = self.slip_ratio_raw(v_long);
        let stiffness = longitudinal_slip_stiffness(slip, slip_angle, envelope, &self.config.tire_model);
        inertia + dt * r * r * stiffness / v_long.abs().max(SLIP_REFERENCE_SPEED)
    }

    /// Integrates thermal dissipation and mechanical tread wear over dt.
    ///
    /// - Frictional work dissipation: P_diss = (|Fx * s| + |Fy * alpha|) * max(|v_long|, |omega * r|)
    /// - Heating: dT_heat = P_diss * k_heat
    /// - Cooling: dT_cool = k_cool * (1 + 0.035 * v) * (T - T_ambient)
    /// - Wear: dW = k_wear * P_diss * temp_factor
    pub fn step_thermal_and_wear(
        &mut self,
        fx: f32,
        fy: f32,
        slip_ratio: f32,
        slip_angle: f32,
        wheel_speed: f32,
        dt: f32,
    ) {
        let r = self.config.tire_radius;
        let v_rub = wheel_speed.max((self.angular_velocity * r).abs()).max(1.0);

        // Dissipated frictional mechanical power in Watts
        let p_long = (fx * slip_ratio).abs();
        let p_lat = (fy * slip_angle).abs();
        let p_diss = (p_long + p_lat) * v_rub;

        // Calibrated heating coefficient: 8s of sustained donut/power drift brings rear tire from 60°C to >120°C
        let k_heat = 0.00028;
        let heat_rate = p_diss * k_heat;

        // Convective cooling towards ambient 25°C
        let ambient_temp = 25.0;
        let k_cool = 0.065;
        let cool_rate = k_cool * (1.0 + 0.035 * wheel_speed) * (self.temperature - ambient_temp);

        self.temperature += (heat_rate - cool_rate) * dt;
        self.temperature = self.temperature.clamp(0.0, 200.0);

        // Mechanical tread wear
        let temp_wear_boost = if self.temperature > 105.0 {
            1.0 + (self.temperature - 105.0) * 0.05
        } else {
            1.0
        };
        let k_wear = 0.00000025;
        let wear_rate = p_diss * k_wear * temp_wear_boost;
        self.wear = (self.wear + wear_rate * dt).clamp(0.0, 1.0);
    }

    /// Surface friction coefficient scaled by tread temperature and wear.
    #[inline]
    pub fn effective_friction(&self, friction_coeff: f32) -> f32 {
        let wear_mult = (1.0 - 0.20 * self.wear).max(0.50);
        friction_coeff * self.thermal_grip_multiplier() * wear_mult
    }

    /// Load-sensitive friction envelope of this tire including thermal and wear effects (N).
    #[inline]
    pub fn friction_envelope(&self, normal_load: f32, nominal_load: f32, friction_coeff: f32) -> f32 {
        tire_friction_envelope(normal_load, nominal_load, self.effective_friction(friction_coeff), &self.config.tire_model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pacejka_lateral_force() {
        let cfg = PacejkaTireConfig::default();
        let normal_load = 2500.0;
        let friction_coeff = 1.0;

        // Zero slip produces zero lateral force
        let f0 = pacejka_lateral_force(0.0, normal_load, friction_coeff, &cfg, false);
        assert!(f0.abs() < 1e-4);

        // Small slip angle generates restoring lateral force
        let f_small = pacejka_lateral_force(0.1, normal_load, friction_coeff, &cfg, false);
        assert!(f_small > 0.0);

        // Negative slip angle produces negative lateral force
        let f_neg = pacejka_lateral_force(-0.1, normal_load, friction_coeff, &cfg, false);
        assert!(f_neg < 0.0);
        assert!((f_small + f_neg).abs() < 1e-3);

        // Handbrake reduces lateral grip
        let f_hb = pacejka_lateral_force(0.1, normal_load, friction_coeff, &cfg, true);
        assert!(f_hb < f_small);
    }

    #[test]
    fn test_curve_constant_places_peak_at_one() {
        let tire = TireConfig::default();
        let at_peak = normalized_grip_curve(1.0, &tire);
        assert!((at_peak - 1.0).abs() < 1e-4, "curve(1) = {at_peak}");
        assert!(normalized_grip_curve(0.98, &tire) < 1.0);
        // Initial slope ~ B * C = 2.56
        let slope = normalized_grip_curve(0.01, &tire) / 0.01;
        assert!((slope - 2.56).abs() < 0.05, "slope = {slope}");
    }

    #[test]
    fn test_normalized_curve_peaks_at_one_and_falls_to_slide_grip() {
        let tire = TireConfig::default();
        assert_eq!(normalized_grip_curve(0.0, &tire), 0.0);
        assert!((normalized_grip_curve(1.0, &tire) - 1.0).abs() < 1e-4);
        assert!(normalized_grip_curve(0.5, &tire) < 1.0);
        assert!(normalized_grip_curve(1.2, &tire) < 1.0);
        let deep = normalized_grip_curve(1.0 + tire.falloff + 1.0, &tire);
        assert!((deep - tire.slide_grip).abs() < 1e-6);
    }

    #[test]
    fn test_combined_slip_peak_and_symmetry() {
        let tire = TireConfig { power_slide: 1.0, ..TireConfig::default() };
        let env = 3000.0;
        let (_, fy_peak) = combined_slip_forces(0.0, tire.peak_slip_angle(), env, &tire);
        assert!((fy_peak - env).abs() < 1.0, "pure lateral peak equals envelope, got {fy_peak}");
        let (_, fy_neg) = combined_slip_forces(0.0, -tire.peak_slip_angle(), env, &tire);
        assert!((fy_peak + fy_neg).abs() < 1e-3);
        let (fx_peak, _) = combined_slip_forces(tire.peak_slip_ratio, 0.0, env, &tire);
        assert!((fx_peak - env).abs() < 1.0);
        let (fx_neg, _) = combined_slip_forces(-tire.peak_slip_ratio, 0.0, env, &tire);
        assert!(fx_neg < 0.0);
    }

    #[test]
    fn test_wheelspin_erodes_lateral_grip_and_resultant_stays_in_envelope() {
        let tire = TireConfig { power_slide: 1.0, ..TireConfig::default() };
        let env = 3000.0;
        let alpha = tire.peak_slip_angle() * 0.8;
        let (_, fy_free) = combined_slip_forces(0.0, alpha, env, &tire);
        let (fx_spin, fy_spin) = combined_slip_forces(0.4, alpha, env, &tire);
        assert!(fy_spin < fy_free * 0.5, "spinning wheel keeps {fy_spin} of {fy_free}");
        assert!((fx_spin * fx_spin + fy_spin * fy_spin).sqrt() <= env + 1e-2);

        // Arcade power_slide keeps more lateral grip under the same wheelspin
        let arcade = TireConfig { power_slide: 0.4, ..tire };
        let (_, fy_arcade) = combined_slip_forces(0.4, alpha, env, &arcade);
        assert!(fy_arcade > fy_spin);
    }

    #[test]
    fn test_load_sensitivity_reduces_grip_per_newton() {
        let tire = TireConfig::default();
        let nominal = 2500.0;
        let light = tire_friction_envelope(1500.0, nominal, 1.0, &tire) / 1500.0;
        let heavy = tire_friction_envelope(3500.0, nominal, 1.0, &tire) / 3500.0;
        assert!(light > heavy);
        // Transferring load across an axle loses total grip
        let even = 2.0 * tire_friction_envelope(2500.0, nominal, 1.0, &tire);
        let split = tire_friction_envelope(1500.0, nominal, 1.0, &tire) + tire_friction_envelope(3500.0, nominal, 1.0, &tire);
        assert!(split < even);
    }

    #[test]
    fn test_friction_circle_clamping() {
        let max_f = 2000.0;
        let (fx, fy) = solve_combined_slip_forces(3000.0, 4000.0, max_f);
        let total = (fx * fx + fy * fy).sqrt();
        assert!((total - max_f).abs() < 1e-2);
        assert!(fx > 0.0 && fy > 0.0);

        // Within circle returns unchanged
        let (fx_in, fy_in) = solve_combined_slip_forces(500.0, 500.0, max_f);
        assert_eq!(fx_in, 500.0);
        assert_eq!(fy_in, 500.0);
    }

    #[test]
    fn test_wheel_assembly_rotational_inertia_and_lockup() {
        let mut wheel = WheelAssembly::new(WheelAssemblyConfig {
            tire_radius: 0.32,
            tire_width: 0.24,
            rotational_inertia: 1.25,
            tire_model: TireConfig::default(),
            brake_bias_factor: 0.30,
            drive_torque_factor: 0.50,
        });
        let envelope = 2500.0;
        let dt = 1.0 / 120.0;

        // Initial standstill state
        assert_eq!(wheel.angular_velocity, 0.0);
        assert!(!wheel.is_locked);
        assert_eq!(wheel.compute_slip_ratio(0.0), 0.0);

        // Drive torque spins the wheel up from rest
        wheel.step_implicit(600.0, 0.0, 0.0, 0.0, envelope, dt);
        assert!(wheel.angular_velocity > 0.0);

        // Rolling at 33.3 m/s (~120 km/h) with no torque stays synchronous
        wheel.angular_velocity = 33.3 / 0.32;
        for _ in 0..10 {
            wheel.step_implicit(0.0, 0.0, 33.3, 0.0, envelope, dt);
        }
        assert!(wheel.compute_slip_ratio(33.3).abs() < 0.01);

        // Heavy braking far above the tire's torque capacity locks the wheel
        for _ in 0..30 {
            wheel.step_implicit(0.0, 5000.0, 33.3, 0.0, envelope, dt);
        }
        assert_eq!(wheel.angular_velocity, 0.0);
        assert!(wheel.is_locked);
        assert_eq!(wheel.compute_slip_ratio(33.3), -1.0);

        // Skid telemetry registers maximum intensity (1.0) under lockup
        let (intensity, is_skidding) = compute_skid_telemetry(0.0, -1.0, 33.3, false, &wheel.config.tire_model, SurfaceType::Asphalt);
        assert_eq!(intensity, 1.0);
        assert!(is_skidding);

        // Releasing the brake lets the road spin the wheel back up to rolling speed
        for _ in 0..60 {
            wheel.step_implicit(0.0, 0.0, 33.3, 0.0, envelope, dt);
        }
        assert!(!wheel.is_locked);
        assert!(wheel.compute_slip_ratio(33.3).abs() < 0.02);
    }

    #[test]
    fn test_implicit_wheel_step_holds_peak_traction_under_moderate_drive() {
        // A drive torque below the tire's capacity settles at a small, steady slip (no runaway)
        let mut wheel = WheelAssembly::new(WheelAssemblyConfig::default());
        let envelope = 2500.0;
        let r = wheel.config.tire_radius;
        let torque = 0.6 * envelope * r;
        wheel.angular_velocity = 20.0 / r;
        for _ in 0..240 {
            wheel.step_implicit(torque, 0.0, 20.0, 0.0, envelope, 1.0 / 120.0);
        }
        let slip = wheel.slip_ratio_raw(20.0);
        let (fx, _) = combined_slip_forces(slip, 0.0, envelope, &wheel.config.tire_model);
        assert!(slip > 0.0 && slip < wheel.config.tire_model.peak_slip_ratio, "slip = {slip}");
        assert!((fx * r - torque).abs() < torque * 0.02, "tire torque balances drive torque");
    }

    #[test]
    fn test_wheel_assembly_thermal_fade_and_wear() {
        let mut wheel = WheelAssembly::new(WheelAssemblyConfig::default());
        wheel.temperature = 85.0; // Optimal nominal temperature
        let nominal_grip = wheel.thermal_grip_multiplier();
        assert!(nominal_grip >= 1.04);

        // Simulate 8.5 seconds of high-energy power drift (e.g. sustained donut slide)
        let dt = 1.0 / 60.0;
        let steps = (8.5 / dt) as usize;
        for _ in 0..steps {
            wheel.step_thermal_and_wear(3500.0, 4200.0, 0.35, 0.40, 18.0, dt);
        }

        // Surface temperature must rise above 120°C
        assert!(wheel.temperature > 120.0, "Tire temperature ({:.1}°C) should exceed 120°C", wheel.temperature);

        // Overheated thermal grip must drop by at least 15% relative to nominal operating grip
        let overheated_grip = wheel.thermal_grip_multiplier();
        let grip_drop = (nominal_grip - overheated_grip) / nominal_grip;
        assert!(grip_drop >= 0.15, "Thermal grip drop ({:.1}%) should be >= 15%", grip_drop * 100.0);

        // Tread wear must have accumulated
        assert!(wheel.wear > 0.0);
    }

    #[test]
    fn test_wheel_assembly_numerical_stability() {
        let mut wheel = WheelAssembly::new(WheelAssemblyConfig::default());
        wheel.angular_velocity = f32::NAN;
        wheel.step_implicit(100.0, 0.0, 20.0, 0.0, 2500.0, 0.016);
        // Recovers to kinematic match (20.0 / 0.32 = 62.5 rad/s) rather than propagating NaN
        assert!((wheel.angular_velocity - (20.0 / 0.32)).abs() < 1e-3);

        // Stiff low-speed case: dt*k >> I must not oscillate or blow up
        let mut wheel = WheelAssembly::new(WheelAssemblyConfig::default());
        for step in 0..600 {
            let v = 0.02 * step as f32 / 60.0;
            wheel.step_implicit(800.0, 0.0, v, 0.0, 2500.0, 1.0 / 120.0);
            assert!(wheel.angular_velocity.is_finite() && wheel.angular_velocity.abs() <= 550.0);
        }
    }
}
