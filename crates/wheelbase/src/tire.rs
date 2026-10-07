//! Tire, wheel spin and contact models.
//!
//! Governed by `specs/043_vehicle_dynamics_rebuild_and_simplified_handling_settings.md`:
//! normalized combined-slip tire with load sensitivity, and implicit wheel spin.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::config::{PacejkaTireConfig, TireConfig, WheelAssemblyConfig};
use super::surface::{CompoundId, SurfaceAffinityMap, SurfaceType};

/// Floor on the slip-ratio reference speed (m/s).
///
/// Keeps the slip definition finite at standstill and bounds the longitudinal tire stiffness
/// seen by the explicit chassis integrator: with `C / (v_ref * m_corner) * dt < 2` the
/// wheel-chassis contact stays stable down to 0 m/s at 60 Hz and 120 Hz.
pub const SLIP_REFERENCE_SPEED: f32 = 4.0;

/// Scales the compound wear rate for slide power past the grip peak (tdrace-6dl0): sprint races are
/// driven on one set of tyres, and only drift abuse should wear them enough to need a pit stop.
const DRIFT_WEAR_GAIN: f32 = 6.0;

/// Share of the pavement tread wear that a slide on loose ground (dirt, gravel, sand, mud, snow) causes.
const LOOSE_GROUND_WEAR: f32 = 0.25;

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
/// Its initial slope (B * C = 2.56) matches a real tire's cornering stiffness at a given peak slip.
/// Precomputed 256-interval lookup table with linear interpolation (Spec 084)
/// eliminating three transcendental calls (two atan, one sin) per evaluation.
const PACEJKA_RISING_LUT: [f32; 257] = [
    0.00000000, 0.00999450, 0.01998720, 0.02997629, 0.03995997, 0.04993644, 0.05990390, 0.06986058,
    0.07980468, 0.08973445, 0.09964811, 0.10954392, 0.11942014, 0.12927505, 0.13910692, 0.14891407,
    0.15869482, 0.16844751, 0.17817049, 0.18786213, 0.19752085, 0.20714504, 0.21673317, 0.22628368,
    0.23579508, 0.24526586, 0.25469458, 0.26407980, 0.27342012, 0.28271414, 0.29196054, 0.30115798,
    0.31030518, 0.31940088, 0.32844385, 0.33743289, 0.34636684, 0.35524458, 0.36406499, 0.37282702,
    0.38152964, 0.39017184, 0.39875266, 0.40727118, 0.41572650, 0.42411775, 0.43244412, 0.44070481,
    0.44889907, 0.45702617, 0.46508543, 0.47307620, 0.48099785, 0.48884981, 0.49663152, 0.50434248,
    0.51198219, 0.51955021, 0.52704613, 0.53446956, 0.54182015, 0.54909759, 0.55630159, 0.56343189,
    0.57048828, 0.57747056, 0.58437856, 0.59121216, 0.59797124, 0.60465573, 0.61126559, 0.61780079,
    0.62426134, 0.63064726, 0.63695863, 0.64319552, 0.64935804, 0.65544631, 0.66146050, 0.66740079,
    0.67326736, 0.67906045, 0.68478029, 0.69042714, 0.69600130, 0.70150304, 0.70693271, 0.71229062,
    0.71757713, 0.72279262, 0.72793747, 0.73301207, 0.73801685, 0.74295223, 0.74781865, 0.75261658,
    0.75734647, 0.76200880, 0.76660407, 0.77113277, 0.77559541, 0.77999252, 0.78432463, 0.78859226,
    0.79279596, 0.79693630, 0.80101381, 0.80502908, 0.80898266, 0.81287514, 0.81670710, 0.82047912,
    0.82419180, 0.82784572, 0.83144149, 0.83497969, 0.83846095, 0.84188585, 0.84525501, 0.84856903,
    0.85182853, 0.85503410, 0.85818637, 0.86128593, 0.86433341, 0.86732940, 0.87027452, 0.87316938,
    0.87601458, 0.87881072, 0.88155841, 0.88425825, 0.88691084, 0.88951678, 0.89207665, 0.89459105,
    0.89706057, 0.89948579, 0.90186729, 0.90420566, 0.90650147, 0.90875528, 0.91096767, 0.91313919,
    0.91527041, 0.91736188, 0.91941416, 0.92142778, 0.92340329, 0.92534123, 0.92724212, 0.92910650,
    0.93093489, 0.93272781, 0.93448576, 0.93620926, 0.93789881, 0.93955491, 0.94117805, 0.94276872,
    0.94432739, 0.94585456, 0.94735068, 0.94881623, 0.95025167, 0.95165745, 0.95303402, 0.95438184,
    0.95570134, 0.95699296, 0.95825712, 0.95949425, 0.96070478, 0.96188911, 0.96304766, 0.96418082,
    0.96528900, 0.96637260, 0.96743199, 0.96846757, 0.96947972, 0.97046879, 0.97143518, 0.97237923,
    0.97330130, 0.97420176, 0.97508095, 0.97593921, 0.97677688, 0.97759430, 0.97839180, 0.97916970,
    0.97992832, 0.98066798, 0.98138900, 0.98209167, 0.98277630, 0.98344319, 0.98409263, 0.98472491,
    0.98534032, 0.98593914, 0.98652164, 0.98708810, 0.98763879, 0.98817397, 0.98869389, 0.98919883,
    0.98968903, 0.99016474, 0.99062621, 0.99107367, 0.99150736, 0.99192753, 0.99233439, 0.99272818,
    0.99310912, 0.99347743, 0.99383333, 0.99417702, 0.99450873, 0.99482865, 0.99513698, 0.99543394,
    0.99571971, 0.99599450, 0.99625848, 0.99651185, 0.99675480, 0.99698749, 0.99721013, 0.99742287,
    0.99762589, 0.99781937, 0.99800347, 0.99817836, 0.99834419, 0.99850113, 0.99864934, 0.99878897,
    0.99892017, 0.99904309, 0.99915788, 0.99926468, 0.99936364, 0.99945490, 0.99953859, 0.99961485,
    0.99968382, 0.99974562, 0.99980038, 0.99984823, 0.99988930, 0.99992371, 0.99995158, 0.99997303,
    1.00000000,
];

/// Normalized tire curve (Spec 043, Spec 084): 0 at s = 0, 1.0 at the peak (s = 1), then a smooth fall
/// to `slide_grip` over `falloff` peak-widths.
#[inline]
pub fn normalized_grip_curve(s: f32, tire: &TireConfig) -> f32 {
    let s = s.abs();
    if s >= 1.0 {
        let t = ((s - 1.0) / tire.falloff.max(0.1)).min(1.0);
        1.0 - (1.0 - tire.slide_grip) * t * t * (3.0 - 2.0 * t)
    } else {
        let scaled = s * 256.0;
        let idx = scaled as usize;
        let frac = scaled - idx as f32;
        PACEJKA_RISING_LUT[idx] + frac * (PACEJKA_RISING_LUT[idx + 1] - PACEJKA_RISING_LUT[idx])
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
    let sy = if slip_angle.abs() < 1e-5 {
        0.0
    } else {
        slip_angle.tan() / tire.peak_slip_angle_tan().max(1e-3)
    };
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

/// Computes only the longitudinal force Fx of combined slip (Spec 084).
///
/// Skips lateral force evaluation and friction circle budget calculation,
/// yielding a significant speedup for implicit integration and ABS/TCS solvers.
#[inline]
pub fn combined_slip_fx(slip_ratio: f32, slip_angle: f32, envelope: f32, tire: &TireConfig) -> f32 {
    if envelope <= 1e-4 {
        return 0.0;
    }
    let sx = slip_ratio / tire.peak_slip_ratio.max(1e-3);
    let sy = if slip_angle.abs() < 1e-5 {
        0.0
    } else {
        slip_angle.tan() / tire.peak_slip_angle_tan().max(1e-3)
    };
    let s_sq = sx * sx + sy * sy;
    if s_sq < 1e-12 {
        return 0.0;
    }
    let s = s_sq.sqrt();
    let f = envelope * normalized_grip_curve(s, tire);
    f * sx / s
}

/// Longitudinal stiffness `dFx/d(slip_ratio)` at the given combined slip state (N per unit slip).
///
/// Used by the implicit wheel spin integrator. Evaluated by central difference and floored at a
/// small positive value so the implicit step stays well conditioned past the peak.
#[inline]
pub fn longitudinal_slip_stiffness(slip_ratio: f32, slip_angle: f32, envelope: f32, tire: &TireConfig) -> f32 {
    if envelope <= 1e-4 {
        return 0.0;
    }
    let sy = if slip_angle.abs() < 1e-5 {
        0.0
    } else {
        slip_angle.tan() / tire.peak_slip_angle_tan().max(1e-3)
    };
    let sy_sq = sy * sy;
    let inv_peak_sr = 1.0 / tire.peak_slip_ratio.max(1e-3);
    let h = tire.peak_slip_ratio.max(1e-3) * 0.05;

    let fx_for_sr = |sr: f32| -> f32 {
        let sx = sr * inv_peak_sr;
        let s_sq = sx * sx + sy_sq;
        if s_sq < 1e-12 {
            0.0
        } else {
            let s = s_sq.sqrt();
            envelope * normalized_grip_curve(s, tire) * sx / s
        }
    };

    let fx_hi = fx_for_sr(slip_ratio + h);
    let fx_lo = fx_for_sr(slip_ratio - h);
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

/// Static compound profile governing friction, wear, and surface affinities (Spec 074).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TireCompoundConfig {
    /// Unique compound identifier.
    pub id: CompoundId,
    /// Human-readable display label (e.g. "Soft Slick", "All-Terrain ATX").
    pub name: &'static str,
    /// Baseline compound grip multiplier on optimal surface (nominally 1.0).
    pub base_grip: f32,
    /// Slip angle in degrees where lateral force peaks (sharp: 6-8°, progressive: 12-14°).
    pub peak_slip_angle_deg: f32,
    /// Longitudinal slip ratio where traction peaks (typically 0.08 - 0.15).
    pub peak_slip_ratio: f32,
    /// Friction retention ratio during deep sliding [0.6 = snappy drop, 1.0 = plateau].
    pub slide_grip: f32,
    /// Rate of mechanical tread loss per unit of frictional dissipation energy (1/J).
    pub wear_rate: f32,
    /// Optimal bulk tread temperature window (°C) [T_min, T_max].
    pub optimal_temp_range: (f32, f32),
    /// Critical overheat temperature (°C) where rubber blistering rapidly reduces grip.
    pub overheat_temp: f32,
    /// Complete affinity multiplier matrix across all 15 SurfaceTypes.
    pub surface_affinity: SurfaceAffinityMap,
}

#[derive(Deserialize)]
struct TireCompoundConfigRaw {
    #[serde(default)]
    pub id: Option<CompoundId>,
    #[serde(default)]
    pub _name: Option<String>,
    #[serde(default)]
    pub base_grip: Option<f32>,
    #[serde(default)]
    pub peak_slip_angle_deg: Option<f32>,
    #[serde(default)]
    pub peak_slip_ratio: Option<f32>,
    #[serde(default)]
    pub slide_grip: Option<f32>,
    #[serde(default)]
    pub wear_rate: Option<f32>,
    #[serde(default)]
    pub optimal_temp_range: Option<(f32, f32)>,
    #[serde(default)]
    pub overheat_temp: Option<f32>,
    #[serde(default)]
    pub surface_affinity: Option<SurfaceAffinityMap>,
}

impl<'de> Deserialize<'de> for TireCompoundConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = TireCompoundConfigRaw::deserialize(deserializer)?;
        Ok(TireCompoundConfig::from(raw))
    }
}

impl From<TireCompoundConfigRaw> for TireCompoundConfig {
    fn from(raw: TireCompoundConfigRaw) -> Self {
        let id = raw.id.unwrap_or(CompoundId::MediumSlick);
        let mut d = TireCompoundConfig::from_id(id);
        if let Some(bg) = raw.base_grip {
            d.base_grip = bg;
        }
        if let Some(psa) = raw.peak_slip_angle_deg {
            d.peak_slip_angle_deg = psa;
        }
        if let Some(psr) = raw.peak_slip_ratio {
            d.peak_slip_ratio = psr;
        }
        if let Some(sg) = raw.slide_grip {
            d.slide_grip = sg;
        }
        if let Some(wr) = raw.wear_rate {
            d.wear_rate = wr;
        }
        if let Some(otr) = raw.optimal_temp_range {
            d.optimal_temp_range = otr;
        }
        if let Some(oht) = raw.overheat_temp {
            d.overheat_temp = oht;
        }
        if let Some(aff) = raw.surface_affinity {
            d.surface_affinity = aff;
        }
        d
    }
}

impl Default for TireCompoundConfig {
    fn default() -> Self {
        Self::from_id(CompoundId::MediumSlick)
    }
}

impl TireCompoundConfig {
    /// Builds a calibrated `TireCompoundConfig` from a `CompoundId`.
    pub const fn from_id(id: CompoundId) -> Self {
        let surface_affinity = SurfaceAffinityMap::for_compound(id);
        match id {
            CompoundId::SoftSlick => Self {
                id,
                name: "Soft Slick",
                base_grip: 1.20,
                peak_slip_angle_deg: 8.0,
                peak_slip_ratio: 0.09,
                slide_grip: 0.82,
                wear_rate: 0.00000045,
                optimal_temp_range: (85.0, 110.0),
                overheat_temp: 120.0,
                surface_affinity,
            },
            CompoundId::MediumSlick => Self {
                id,
                name: "Medium Slick",
                base_grip: 1.00,
                peak_slip_angle_deg: 10.0,
                peak_slip_ratio: 0.10,
                slide_grip: 0.88,
                wear_rate: 0.00000025,
                optimal_temp_range: (80.0, 105.0),
                overheat_temp: 125.0,
                surface_affinity,
            },
            CompoundId::HardSlick => Self {
                id,
                name: "Hard Slick",
                base_grip: 0.95,
                peak_slip_angle_deg: 11.5,
                peak_slip_ratio: 0.11,
                slide_grip: 0.90,
                wear_rate: 0.00000012,
                optimal_temp_range: (75.0, 100.0),
                overheat_temp: 130.0,
                surface_affinity,
            },
            CompoundId::IntermediateWet => Self {
                id,
                name: "Intermediate Wet",
                base_grip: 0.95,
                peak_slip_angle_deg: 11.0,
                peak_slip_ratio: 0.11,
                slide_grip: 0.85,
                wear_rate: 0.00000030,
                optimal_temp_range: (60.0, 85.0),
                overheat_temp: 105.0,
                surface_affinity,
            },
            CompoundId::MonsoonWet => Self {
                id,
                name: "Monsoon Wet",
                base_grip: 0.90,
                peak_slip_angle_deg: 12.0,
                peak_slip_ratio: 0.12,
                slide_grip: 0.85,
                wear_rate: 0.00000040,
                optimal_temp_range: (50.0, 75.0),
                overheat_temp: 95.0,
                surface_affinity,
            },
            CompoundId::AllTerrain => Self {
                id,
                name: "All-Terrain",
                base_grip: 1.00,
                peak_slip_angle_deg: 13.0,
                peak_slip_ratio: 0.13,
                slide_grip: 0.88,
                wear_rate: 0.00000020,
                optimal_temp_range: (65.0, 95.0),
                overheat_temp: 115.0,
                surface_affinity,
            },
            CompoundId::ExtremeMud => Self {
                id,
                name: "Extreme Mud",
                base_grip: 0.95,
                peak_slip_angle_deg: 14.0,
                peak_slip_ratio: 0.15,
                slide_grip: 0.92,
                wear_rate: 0.00000025,
                optimal_temp_range: (60.0, 90.0),
                overheat_temp: 110.0,
                surface_affinity,
            },
            CompoundId::StuddedIce => Self {
                id,
                name: "Studded Ice",
                base_grip: 0.90,
                peak_slip_angle_deg: 13.5,
                peak_slip_ratio: 0.14,
                slide_grip: 0.90,
                wear_rate: 0.00000035,
                optimal_temp_range: (40.0, 75.0),
                overheat_temp: 90.0,
                surface_affinity,
            },
        }
    }

    /// Converts this compound configuration into a baseline [`TireConfig`].
    pub fn to_tire_config(&self) -> TireConfig {
        TireConfig {
            grip: self.base_grip,
            peak_slip_angle_deg: self.peak_slip_angle_deg,
            peak_slip_ratio: self.peak_slip_ratio,
            slide_grip: self.slide_grip,
            falloff: 1.5,
            load_sensitivity: 0.15,
            power_slide: 0.85,
            skid_threshold: 0.10,
            skid_full_threshold: 0.35,
        }
    }
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

    /// Evaluates the dynamic thermal grip multiplier based on tread temperature T and active compound.
    pub fn thermal_grip_multiplier(&self) -> f32 {
        let t = self.temperature;
        let (t_min, t_max) = self.config.compound.optimal_temp_range;
        let t_overheat = self.config.compound.overheat_temp;
        let t_cold_span = (t_min - 35.0).max(10.0);
        let mult = if t < t_min - 35.0 {
            // Cold tire: ramp from 0.96 at 0°C up to 1.00 at t_min - 35°C
            0.96 + (t.max(0.0) / t_cold_span) * 0.04
        } else if t < t_min {
            // Warm-up phase: ramp from 1.00 to 1.06 at t_min
            1.00 + ((t - (t_min - 35.0)) / 35.0) * 0.06
        } else if t <= t_max {
            // Optimal peak grip window: 1.06
            1.06
        } else if t <= t_overheat {
            // Overheating transition: drops to 0.88 at t_overheat
            let span = (t_overheat - t_max).max(1.0);
            1.06 - ((t - t_max) / span) * 0.18
        } else {
            // Severe overheat: gradual degradation from 0.88 down to 0.82
            let span = 80.0f32;
            0.88 - ((t - t_overheat) / span).min(1.0) * 0.06
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
        let eff_inertia = self.effective_inertia(v_long, slip_angle, envelope, dt);
        self.step_implicit_with_inertia(
            drive_torque,
            brake_torque,
            v_long,
            slip_angle,
            envelope,
            eff_inertia,
            dt,
        );
    }

    /// Optimized implicit integration using precomputed effective inertia (Spec 084).
    /// Avoids duplicate evaluations of longitudinal slip stiffness across differential coupling.
    pub fn step_implicit_with_inertia(
        &mut self,
        drive_torque: f32,
        brake_torque: f32,
        v_long: f32,
        slip_angle: f32,
        envelope: f32,
        effective_inertia: f32,
        dt: f32,
    ) {
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
        let fx0 = combined_slip_fx(slip, slip_angle, envelope, tire);

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
    /// - Wear: dW = k_wear * P_slide * temp_factor, where P_slide counts only the sideways slip past the
    ///   tyre's grip peak (tdrace-6dl0): clean driving barely wears the tread, a drift does.
    pub fn step_thermal_and_wear(
        &mut self,
        fx: f32,
        fy: f32,
        slip_ratio: f32,
        slip_angle: f32,
        wheel_speed: f32,
        surface: SurfaceType,
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
        let temp_wear_boost = if self.temperature > self.config.compound.optimal_temp_range.1 {
            1.0 + (self.temperature - self.config.compound.optimal_temp_range.1) * 0.05
        } else {
            1.0
        };
        // Only a sideways slide past the grip peak (a drift) wears the tread; wheelspin and lock-ups do not.
        // Sliding on loose ground (rallycross and autocross drifting) wears it much less than on pavement.
        let slide_lat = (slip_angle.abs() - self.config.tire_model.peak_slip_angle_deg.to_radians()).max(0.0);
        let abrasion = if surface.is_loose_deformable() { LOOSE_GROUND_WEAR } else { 1.0 };
        let p_slide = (fy * slide_lat).abs() * v_rub * abrasion;
        let k_wear = self.config.compound.wear_rate * DRIFT_WEAR_GAIN;
        let wear_rate = p_slide * k_wear * temp_wear_boost;
        self.wear = (self.wear + wear_rate * dt).clamp(0.0, 1.0);
    }

    /// Surface friction coefficient scaled by tread temperature and wear.
    #[inline]
    pub fn effective_friction(&self, friction_coeff: f32) -> f32 {
        let wear_mult = (1.0 - 0.20 * self.wear).max(0.50);
        friction_coeff * self.thermal_grip_multiplier() * wear_mult
    }

    /// Surface friction coefficient scaled by compound surface affinity, tread temperature, and wear (Spec 074).
    #[inline]
    pub fn effective_friction_on_surface(&self, friction_coeff: f32, surface: SurfaceType) -> f32 {
        let affinity = self.config.compound.surface_affinity.get(surface);
        self.effective_friction(friction_coeff) * affinity
    }

    /// Load-sensitive friction envelope of this tire including thermal and wear effects (N).
    #[inline]
    pub fn friction_envelope(&self, normal_load: f32, nominal_load: f32, friction_coeff: f32) -> f32 {
        tire_friction_envelope(normal_load, nominal_load, self.effective_friction(friction_coeff), &self.config.tire_model)
    }

    /// Load-sensitive friction envelope of this tire on a specific surface including compound affinity, thermal and wear effects (N).
    #[inline]
    pub fn friction_envelope_on_surface(&self, normal_load: f32, nominal_load: f32, friction_coeff: f32, surface: SurfaceType) -> f32 {
        let eff_mu = self.effective_friction_on_surface(friction_coeff, surface);
        tire_friction_envelope(normal_load, nominal_load, eff_mu, &self.config.tire_model)
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
            compound: TireCompoundConfig::default(),
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
            wheel.step_thermal_and_wear(3500.0, 4200.0, 0.35, 0.40, 18.0, SurfaceType::Asphalt, dt);
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
