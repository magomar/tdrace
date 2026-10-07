use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::surface::CompoundId;
use super::tire::TireCompoundConfig;

/// Configuration for an individual wheel corner or axle assembly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WheelAssemblyConfig {
    /// Rolling radius of the tire under nominal load (meters).
    pub tire_radius: f32,
    /// Width of the tire contact patch (meters).
    pub tire_width: f32,
    /// Rotational polar moment of inertia (kg·m²).
    pub rotational_inertia: f32,
    /// Active physical tire compound model governing slip and terrain affinities (Spec 074).
    #[serde(default)]
    pub compound: TireCompoundConfig,
    /// Tire model of this wheel. Derived from `CarConfig::tire` / `rear_axle` by `CarConfig::finalize()`.
    #[serde(default)]
    pub tire_model: TireConfig,
    /// Proportion of total brake torque routed to this wheel [0.0, 1.0].
    pub brake_bias_factor: f32,
    /// Proportion of differential drive torque routed to this wheel [0.0, 1.0].
    pub drive_torque_factor: f32,
}

impl Default for WheelAssemblyConfig {
    fn default() -> Self {
        Self {
            tire_radius: 0.32,
            tire_width: 0.24,
            rotational_inertia: 1.25,
            compound: TireCompoundConfig::default(),
            tire_model: TireConfig::default(),
            brake_bias_factor: 0.25, // 25% per wheel = 50% front / 50% rear baseline
            drive_torque_factor: 0.50, // RWD: 50% per rear wheel
        }
    }
}

impl WheelAssemblyConfig {
    pub const fn new(
        tire_radius: f32,
        tire_width: f32,
        rotational_inertia: f32,
        tire_model: TireConfig,
        brake_bias_factor: f32,
        drive_torque_factor: f32,
    ) -> Self {
        Self {
            tire_radius,
            tire_width,
            rotational_inertia,
            compound: TireCompoundConfig::from_id(CompoundId::MediumSlick),
            tire_model,
            brake_bias_factor,
            drive_torque_factor,
        }
    }

    /// Constructs a physically consistent wheel assembly where rotational inertia
    /// is derived directly from corner mass: I = 0.5 * m * r^2 (Spec 074).
    pub fn from_corner_mass(
        tire_radius: f32,
        tire_width: f32,
        corner_mass: f32,
        compound: TireCompoundConfig,
        brake_bias_factor: f32,
        drive_torque_factor: f32,
    ) -> Self {
        let rotational_inertia = 0.5 * corner_mass * tire_radius * tire_radius;
        let tire_model = compound.to_tire_config();
        Self {
            tire_radius,
            tire_width,
            rotational_inertia,
            compound,
            tire_model,
            brake_bias_factor,
            drive_torque_factor,
        }
    }

    /// Computes rotational polar moment of inertia I = 0.5 * m * r^2 from wheel mass.
    pub fn from_mass(
        tire_radius: f32,
        tire_width: f32,
        tire_mass: f32,
        tire_model: TireConfig,
        brake_bias_factor: f32,
        drive_torque_factor: f32,
    ) -> Self {
        let rotational_inertia = 0.5 * tire_mass * tire_radius * tire_radius;
        Self {
            tire_radius,
            tire_width,
            rotational_inertia,
            compound: TireCompoundConfig::from_id(CompoundId::MediumSlick),
            tire_model,
            brake_bias_factor,
            drive_torque_factor,
        }
    }
}

/// Helper returning standard default wheel assemblies for all 4 corners.
pub fn default_wheel_assemblies() -> [WheelAssemblyConfig; 4] {
    [WheelAssemblyConfig::default(); 4]
}

/// Classic Pacejka Magic Formula lateral tire parameters.
///
/// Used by the motorbike model. Car tires use [`TireConfig`] (Spec 043).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PacejkaTireConfig {
    /// Pacejka B (Stiffness factor). Determines slope at low slip angles.
    pub stiffness_b: f32,
    /// Pacejka C (Shape factor). Controls shape and peak prominence (typically ~1.4 - 1.6).
    pub shape_c: f32,
    /// Pacejka D (Peak factor multiplier, base peak is scaled by mu * Fz).
    pub peak_d: f32,
    /// Pacejka E (Curvature factor). Controls drop-off after peak.
    pub curvature_e: f32,
    /// Friction retention ratio during high slip drift (slide friction / peak friction).
    pub drift_slide_friction: f32,
    /// Rear tire lateral friction multiplier when handbrake is engaged (allows rear breakout).
    pub handbrake_lateral_friction_multiplier: f32,
    /// Minimum slip angle (radians) to trigger tire squeal and skid marks.
    pub skid_threshold: f32,
    /// Slip angle (radians) corresponding to 100% skid intensity and dense tire smoke.
    pub skid_full_threshold: f32,
}

impl Default for PacejkaTireConfig {
    fn default() -> Self {
        Self {
            stiffness_b: 9.5,
            shape_c: 1.45,
            peak_d: 1.0,
            curvature_e: -0.15,
            drift_slide_friction: 0.88,
            handbrake_lateral_friction_multiplier: 0.38,
            skid_threshold: 0.10,
            skid_full_threshold: 0.35,
        }
    }
}

/// Slip angle (degrees) at which a Pacejka curve with the given B, C, E peaks.
///
/// Solves `C * atan(B*a - E*(B*a - atan(B*a))) = PI/2` for `a`. Used to migrate legacy
/// Pacejka tire configs to [`TireConfig::peak_slip_angle_deg`].
pub fn pacejka_peak_slip_angle_deg(b: f32, c: f32, e: f32) -> f32 {
    if b <= 1e-3 || c <= 1.0 {
        return TireConfig::DEFAULT_PEAK_SLIP_ANGLE_DEG;
    }
    let target = (std::f32::consts::FRAC_PI_2 / c).tan();
    let f = |y: f32| y - e * (y - y.atan()) - target;
    let (mut lo, mut hi) = (0.0f32, 50.0f32);
    if f(hi) < 0.0 {
        return TireConfig::DEFAULT_PEAK_SLIP_ANGLE_DEG;
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (0.5 * (lo + hi) / b).to_degrees().clamp(3.0, 25.0)
}

/// Car tire model (Spec 043): normalized combined slip with load sensitivity.
///
/// Every field is a designer knob expressed in a unit a driver can feel.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(from = "TireConfigRaw")]
pub struct TireConfig {
    /// Grip multiplier on the surface friction coefficient (1.0 = road tire on asphalt).
    pub grip: f32,
    /// Slip angle in degrees where lateral grip peaks (sharp turn-in: 6-8, lazy: 12-14).
    pub peak_slip_angle_deg: f32,
    /// Longitudinal slip ratio where traction/braking grip peaks (0.08-0.15).
    pub peak_slip_ratio: f32,
    /// Grip retained deep in a slide relative to peak (0.6 = snappy, 1.0 = flat plateau).
    pub slide_grip: f32,
    /// Width of the post-peak grip fall, in multiples of the peak slip (1.0-3.0).
    pub falloff: f32,
    /// Load sensitivity: grip per newton drops as load rises above nominal (0 = linear, 0.1-0.25 = real).
    pub load_sensitivity: f32,
    /// How much longitudinal slip steals lateral grip (1.0 = physical slip-vector direction,
    /// 0.0 = lateral keeps the whole friction-circle budget left after Fx; arcade ~0.6-0.85).
    pub power_slide: f32,
    /// Minimum slip angle (radians) to trigger tire squeal and skid marks.
    pub skid_threshold: f32,
    /// Slip angle (radians) corresponding to 100% skid intensity and dense tire smoke.
    pub skid_full_threshold: f32,
}

impl TireConfig {
    pub const DEFAULT_PEAK_SLIP_ANGLE_DEG: f32 = 10.5;

    /// Peak lateral slip angle in radians.
    #[inline]
    pub fn peak_slip_angle(&self) -> f32 {
        self.peak_slip_angle_deg.to_radians()
    }

    /// Peak lateral slip angle tangent (Spec 084).
    /// Uses precomputed constant for default 10.5 deg to eliminate runtime tan() overhead.
    #[inline]
    pub fn peak_slip_angle_tan(&self) -> f32 {
        if (self.peak_slip_angle_deg - Self::DEFAULT_PEAK_SLIP_ANGLE_DEG).abs() < 1e-4 {
            0.18533965
        } else {
            self.peak_slip_angle_deg.to_radians().tan()
        }
    }
}

impl Default for TireConfig {
    fn default() -> Self {
        Self {
            grip: 1.0,
            peak_slip_angle_deg: Self::DEFAULT_PEAK_SLIP_ANGLE_DEG,
            peak_slip_ratio: 0.10,
            slide_grip: 0.88,
            falloff: 1.5,
            load_sensitivity: 0.15,
            power_slide: 0.85,
            skid_threshold: 0.10,
            skid_full_threshold: 0.35,
        }
    }
}

/// Serde input for [`TireConfig`] that also accepts the legacy Pacejka field names.
#[derive(Deserialize)]
struct TireConfigRaw {
    #[serde(default, alias = "peak_d")]
    grip: Option<f32>,
    #[serde(default)]
    peak_slip_angle_deg: Option<f32>,
    #[serde(default)]
    peak_slip_ratio: Option<f32>,
    #[serde(default, alias = "drift_slide_friction")]
    slide_grip: Option<f32>,
    #[serde(default)]
    falloff: Option<f32>,
    #[serde(default)]
    load_sensitivity: Option<f32>,
    #[serde(default)]
    power_slide: Option<f32>,
    #[serde(default)]
    skid_threshold: Option<f32>,
    #[serde(default)]
    skid_full_threshold: Option<f32>,
    // Legacy Pacejka shape (pre Spec 043): only used to derive the peak slip angle.
    #[serde(default)]
    stiffness_b: Option<f32>,
    #[serde(default)]
    shape_c: Option<f32>,
    #[serde(default)]
    curvature_e: Option<f32>,
    // Legacy handbrake grip multiplier: handbrake slides now come from rear wheel lock-up.
    #[serde(default, rename = "handbrake_lateral_friction_multiplier")]
    _handbrake_lateral_friction_multiplier: Option<f32>,
}

impl From<TireConfigRaw> for TireConfig {
    fn from(raw: TireConfigRaw) -> Self {
        let d = TireConfig::default();
        let legacy_peak = raw.stiffness_b.map(|b| {
            pacejka_peak_slip_angle_deg(
                b,
                raw.shape_c.unwrap_or(1.45),
                raw.curvature_e.unwrap_or(-0.15),
            )
        });
        Self {
            grip: raw.grip.unwrap_or(d.grip),
            peak_slip_angle_deg: raw
                .peak_slip_angle_deg
                .or(legacy_peak)
                .unwrap_or(d.peak_slip_angle_deg),
            peak_slip_ratio: raw.peak_slip_ratio.unwrap_or(d.peak_slip_ratio),
            slide_grip: raw.slide_grip.unwrap_or(d.slide_grip),
            falloff: raw.falloff.unwrap_or(d.falloff),
            load_sensitivity: raw.load_sensitivity.unwrap_or(d.load_sensitivity),
            power_slide: raw.power_slide.unwrap_or(d.power_slide),
            skid_threshold: raw.skid_threshold.unwrap_or(d.skid_threshold),
            skid_full_threshold: raw.skid_full_threshold.unwrap_or(d.skid_full_threshold),
        }
    }
}

/// Configuration for electronic driver assists (TCS, ESC, Counter-Steer Drift Recovery).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DriverAssistsConfig {
    /// Traction Control System (TCS) enabled.
    /// Prevents excessive drive wheel slip under acceleration to eliminate snap power-oversteer.
    pub tcs_enabled: bool,
    /// TCS longitudinal slip target. 0.18 holds driven wheels at the tire's peak slip ratio;
    /// higher values allow proportionally more wheelspin (0.30 = 1.67x peak).
    pub tcs_slip_threshold: f32,
    /// TCS torque reduction strength [0.0 = none, 1.0 = full cut down to grip limit].
    pub tcs_strength: f32,
    /// TCS lateral trigger: rear slip angle (degrees) above which engine torque is cut (Spec 043).
    #[serde(default = "default_tcs_slip_angle_deg")]
    pub tcs_slip_angle_deg: f32,
    /// Independent ESC body-sideslip intervention threshold, in degrees.
    #[serde(default = "default_esc_sideslip_limit_deg")]
    pub esc_sideslip_limit_deg: f32,
    /// Skip TCS torque and wheel-slip corrections during established forward counter-steered drifts.
    #[serde(default)]
    pub tcs_drift_bypass: bool,

    /// Electronic Stability Control (ESC) enabled.
    /// Applies corrective stabilizing yaw moment when unintended sideslip/yaw rate occurs.
    pub esc_enabled: bool,
    /// ESC yaw rate error threshold (rad/s) before intervention starts.
    pub esc_yaw_threshold: f32,
    /// ESC stabilizing damping strength.
    pub esc_strength: f32,

    /// Counter-steer / Self-aligning drift recovery assist enabled.
    /// Helps digital and gamepad players catch slides and self-align when counter-steering.
    pub counter_steer_assist_enabled: bool,
    /// Strength of self-aligning steering torque assistance.
    pub counter_steer_assist_strength: f32,

    /// Anti-lock Braking System (ABS) enabled.
    /// Prevents excessive brake lockup to preserve lateral steering grip during braking.
    pub abs_enabled: bool,
    /// ABS braking slip target. 0.15 holds braked wheels at the tire's peak slip ratio;
    /// higher values allow proportionally deeper slip. The target shrinks while cornering.
    pub abs_slip_threshold: f32,
    /// ABS modulation strength [0.0 = disabled, 1.0 = full pressure modulation].
    pub abs_strength: f32,

    /// Handbrake bypass: whether holding the handbrake temporarily disengages TCS and relaxes ESC
    /// so intentional handbrake power-drifts are 100% responsive and uninhibited.
    pub handbrake_bypass: bool,
}

fn default_tcs_slip_angle_deg() -> f32 {
    12.0
}

fn default_esc_sideslip_limit_deg() -> f32 {
    12.0
}

impl Default for DriverAssistsConfig {
    fn default() -> Self {
        Self::arcade()
    }
}

impl DriverAssistsConfig {
    /// Arcade preset: full assists active for forgiving, accessible keyboard and controller driving.
    pub const fn arcade() -> Self {
        Self {
            tcs_enabled: true,
            tcs_slip_threshold: 0.18,
            tcs_strength: 0.75,
            tcs_slip_angle_deg: 18.0,
            esc_sideslip_limit_deg: 22.0,
            tcs_drift_bypass: false,
            esc_enabled: true,
            esc_yaw_threshold: 0.10,
            esc_strength: 0.70,
            counter_steer_assist_enabled: true,
            counter_steer_assist_strength: 0.75,
            abs_enabled: true,
            abs_slip_threshold: 0.15,
            abs_strength: 0.95,
            handbrake_bypass: true,
        }
    }

    /// Sport preset: mild assists allowing moderate slip angles and aggressive powerslides.
    pub const fn sport() -> Self {
        Self {
            tcs_enabled: true,
            tcs_slip_threshold: 0.45,
            tcs_strength: 0.25,
            tcs_slip_angle_deg: 32.0,
            esc_sideslip_limit_deg: 38.0,
            tcs_drift_bypass: true,
            esc_enabled: true,
            esc_yaw_threshold: 0.40,
            esc_strength: 0.30,
            counter_steer_assist_enabled: true,
            counter_steer_assist_strength: 0.55,
            abs_enabled: true,
            abs_slip_threshold: 0.20,
            abs_strength: 0.75,
            handbrake_bypass: true,
        }
    }

    /// Pro / Raw preset: all electronic assists completely off for pure simulation physics.
    pub const fn raw() -> Self {
        Self {
            tcs_enabled: false,
            tcs_slip_threshold: 0.50,
            tcs_strength: 0.0,
            tcs_slip_angle_deg: 30.0,
            esc_sideslip_limit_deg: 180.0,
            tcs_drift_bypass: false,
            esc_enabled: false,
            esc_yaw_threshold: 1.0,
            esc_strength: 0.0,
            counter_steer_assist_enabled: false,
            counter_steer_assist_strength: 0.0,
            abs_enabled: false,
            abs_slip_threshold: 0.50,
            abs_strength: 0.0,
            handbrake_bypass: true,
        }
    }
}

/// Standard driver assist difficulty profiles of varied difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssistProfile {
    /// Arcade / Beginner: Full TCS, ESC yaw stabilization, and counter-steer assist. Easiest handling.
    Arcade,
    /// Sport / Intermediate: Mild TCS, relaxed ESC, responsive power slides and agile rotation.
    Sport,
    /// Pro / Expert: All electronic aids OFF. Pure simulation physics.
    Pro,
}

impl Default for AssistProfile {
    fn default() -> Self {
        Self::Arcade
    }
}

impl AssistProfile {
    pub const ALL: [Self; 3] = [Self::Arcade, Self::Sport, Self::Pro];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Arcade => "Arcade (Assists: Full)",
            Self::Sport => "Sport (Assists: Mild)",
            Self::Pro => "Pro (Assists: OFF)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Arcade => "ARCADE",
            Self::Sport => "SPORT",
            Self::Pro => "PRO",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Arcade => "Full TCS + ESC + Counter-steer. Highly stable, zero snap spinouts.",
            Self::Sport => "Mild TCS + Relaxed ESC. Allows responsive power slides & tight turns.",
            Self::Pro => "Electronic aids disabled. Pure raw vehicle physics simulation.",
        }
    }

    pub fn to_config(&self) -> DriverAssistsConfig {
        match self {
            Self::Arcade => DriverAssistsConfig::arcade(),
            Self::Sport => DriverAssistsConfig::sport(),
            Self::Pro => DriverAssistsConfig::raw(),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Arcade => Self::Sport,
            Self::Sport => Self::Pro,
            Self::Pro => Self::Arcade,
        }
    }
}

/// Vehicle-specific terrain interaction parameters (tire flotation, paddle thrust, and ice stud penetration).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrainInteractionConfig {
    /// Sand flotation factor gamma_sand in [0.10, 1.00].
    /// Scales rolling resistance increase on sand: 1.0 = standard road tires, 0.30 = paddle tires / lightweight dune buggy.
    pub sand_flotation: f32,
    /// Mud flotation factor gamma_mud in [0.10, 1.00].
    /// Scales rolling resistance increase on mud: 1.0 = standard tires, 0.25 = chevron tractor paddles / mud bogger.
    pub mud_flotation: f32,
    /// Ice grip multiplier alpha_ice in [0.50, 10.00].
    /// Scales available friction on ice: 1.0 = unstudded road tire (mu=0.08), 8.125 = tungsten-studded ice racer (mu=0.65).
    pub ice_grip_multiplier: f32,
}

impl Default for TerrainInteractionConfig {
    fn default() -> Self {
        Self {
            sand_flotation: 1.0,
            mud_flotation: 1.0,
            ice_grip_multiplier: 1.0,
        }
    }
}

/// Rear tire relative to the front `tire` (Spec 043). Ratios, so every later tire change (catalog
/// grip stat, module tuning) reaches both axles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RearAxleTire {
    /// Rear grip / front grip.
    pub grip_scale: f32,
    /// Rear peak slip angle / front peak slip angle (< 1 = stiffer rear = more stable).
    pub peak_slip_scale: f32,
}

impl Default for RearAxleTire {
    /// A slightly stiffer rear (peak at 0.85x the front slip angle): a stable understeer gradient
    /// in the linear range. With equal tires the factory cars were neutral and several GT,
    /// stock-car and kart models diverged into slow spins above ~40 m/s with a steer key held.
    fn default() -> Self {
        Self {
            grip_scale: 1.0,
            peak_slip_scale: 0.85,
        }
    }
}

impl RearAxleTire {
    /// Identical front and rear tires (neutral linear balance, lively rear).
    pub const NEUTRAL: Self = Self {
        grip_scale: 1.0,
        peak_slip_scale: 1.0,
    };

    /// The rear tire for a given front tire.
    pub fn apply(&self, front: &TireConfig) -> TireConfig {
        TireConfig {
            grip: front.grip * self.grip_scale,
            peak_slip_angle_deg: front.peak_slip_angle_deg * self.peak_slip_scale,
            ..*front
        }
    }
}

/// Per-driver handling aids (Spec 043). Set from the player's handling preset; bots use the default.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerHandling {
    /// Grip-aware steering: full input maps to the largest angle the front tires can use at this
    /// speed. On for human drivers. Off (linear full-lock mapping) for bots and scripted
    /// controllers, whose closed-loop steering gains were tuned for the linear mapping.
    #[serde(default)]
    pub grip_aware_steering: bool,
    /// Steering authority: where full input sits relative to the front grip limit, in units of the
    /// front peak slip angle (0.85 = just short of the limit, 1.0 = on the limit, 1.3 = past it).
    pub steer_overslip: f32,
    /// Traction help [0, 1]: eases throttle as the rear axle nears its lateral limit.
    pub traction_help: f32,
    /// Enables the Spec 072 preset-aware low-speed authority envelope for human drivers.
    #[serde(default)]
    pub low_speed_authority_enabled: bool,
}

impl Default for PlayerHandling {
    fn default() -> Self {
        Self {
            grip_aware_steering: false,
            steer_overslip: 1.0,
            traction_help: 0.0,
            low_speed_authority_enabled: false,
        }
    }
}

impl PlayerHandling {
    /// Human-driver handling with grip-aware steering on.
    pub fn human(steer_overslip: f32, traction_help: f32) -> Self {
        Self {
            grip_aware_steering: true,
            steer_overslip,
            traction_help,
            low_speed_authority_enabled: false,
        }
    }
}

/// Type and mechanical characteristics of an axle differential.
///
/// Governs dynamic cross-axle torque distribution and rotational speed coupling
/// across driven wheels (Spec 034).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DifferentialType {
    /// 100% mechanical lock between left and right wheels (omega_L == omega_R).
    /// Used on Karts, NASCAR Cup/Trucks/TA1, and Extreme Off-Road Spool buggies.
    Spool,
    /// Salisbury multi-plate clutch limited-slip differential.
    /// Used on GT3, LMH Hypercars, Sports Cars, and Rallycross machines.
    LimitedSlip {
        /// Locking factor under power / acceleration in [0.0, 1.0].
        power_lock: f32,
        /// Locking factor under coast / trailing throttle in [0.0, 1.0].
        coast_lock: f32,
        /// Static clutch pack spring preload in N*m.
        preload_nm: f32,
    },
    /// Conventional open differential with 50/50 torque split.
    Open,
}

impl Default for DifferentialType {
    fn default() -> Self {
        Self::Open
    }
}

/// Default load transfer response (Hz). The pre-043 filter `alpha = dt * 15` was ~2.4 Hz.
pub const DEFAULT_WEIGHT_TRANSFER_HZ: f32 = 3.0;

pub fn default_front_differential() -> DifferentialType {
    DifferentialType::Open
}

pub fn default_rear_differential() -> DifferentialType {
    DifferentialType::LimitedSlip {
        power_lock: 0.50,
        coast_lock: 0.30,
        preload_nm: 60.0,
    }
}

pub fn default_damage_enabled() -> bool {
    true
}

/// Physical exterior chassis dimensions and anchor points (Spec 075).
///
/// Decouples visual geometry and collision bounds from dynamic mass distribution.
/// All longitudinal measurements are relative to front and rear axle centers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChassisSkeleton {
    /// Distance from front axle center to outermost front bumper / splitter tip (meters).
    pub front_overhang: f32,
    /// Distance from rear axle center to outermost rear bumper / diffuser edge (meters).
    pub rear_overhang: f32,
    /// Overall bodywork/fender width (meters).
    pub body_width: f32,
    /// Longitudinal distance from front axle to front edge of cockpit/cabin (meters, negative = rearward).
    pub cabin_start_offset: f32,
    /// Longitudinal distance from rear axle to rear edge of cockpit/cabin (meters, positive = forward).
    pub cabin_end_offset: f32,
    /// Lateral headlight socket spread expressed as a fraction of `body_width` [0.0..1.0].
    pub headlight_spread: f32,
    /// Lateral taillight socket spread expressed as a fraction of `body_width` [0.0..1.0].
    pub taillight_spread: f32,
    /// Longitudinal inset from bumper edges for light fixtures (meters).
    pub light_inset: f32,
}

impl Default for ChassisSkeleton {
    fn default() -> Self {
        Self {
            front_overhang: 0.80,
            rear_overhang: 0.90,
            body_width: 1.80,
            cabin_start_offset: -0.40,
            cabin_end_offset: 0.30,
            headlight_spread: 0.65,
            taillight_spread: 0.70,
            light_inset: 0.05,
        }
    }
}

impl ChassisSkeleton {
    pub const fn new(
        front_overhang: f32,
        rear_overhang: f32,
        body_width: f32,
        cabin_start_offset: f32,
        cabin_end_offset: f32,
        headlight_spread: f32,
        taillight_spread: f32,
        light_inset: f32,
    ) -> Self {
        Self {
            front_overhang,
            rear_overhang,
            body_width,
            cabin_start_offset,
            cabin_end_offset,
            headlight_spread,
            taillight_spread,
            light_inset,
        }
    }

    /// Synthesizes a proportional default chassis skeleton from wheelbase and track width.
    pub fn synthesize_proportional(wheelbase: f32, track_width: f32) -> Self {
        Self {
            front_overhang: (wheelbase * 0.32).clamp(0.12, 1.30),
            rear_overhang: (wheelbase * 0.38).clamp(0.10, 1.40),
            body_width: track_width + 0.30,
            cabin_start_offset: -wheelbase * 0.16,
            cabin_end_offset: wheelbase * 0.12,
            headlight_spread: 0.65,
            taillight_spread: 0.70,
            light_inset: 0.05,
        }
    }

    /// Computes total bumper-to-bumper vehicle length given a wheelbase.
    #[inline]
    pub const fn total_length(&self, wheelbase: f32) -> f32 {
        wheelbase + self.front_overhang + self.rear_overhang
    }

    /// Computes half of the total vehicle length.
    #[inline]
    pub const fn half_length(&self, wheelbase: f32) -> f32 {
        self.total_length(wheelbase) * 0.5
    }

    /// Computes the half-width of the vehicle bodywork.
    #[inline]
    pub const fn half_width(&self) -> f32 {
        self.body_width * 0.5
    }

    /// Computes the signed longitudinal offset from Center of Gravity (CG) to the geometric center.
    ///
    /// Positive offset indicates geometric center is forward of CG; negative indicates rearward of CG.
    #[inline]
    pub fn geometric_center_offset_from_cg(&self, lf: f32, lr: f32) -> f32 {
        let front_extent = lf + self.front_overhang;
        let rear_extent = lr + self.rear_overhang;
        (front_extent - rear_extent) * 0.5
    }

    /// Computes world positions of left and right headlight fixtures.
    pub fn headlight_positions_world(&self, pos: Vec2, fwd: Vec2, right: Vec2, lf: f32) -> (Vec2, Vec2) {
        let front_tip = pos + fwd * (lf + self.front_overhang - self.light_inset);
        let half_spread = self.half_width() * self.headlight_spread;
        (front_tip - right * half_spread, front_tip + right * half_spread)
    }

    /// Computes world positions of left and right taillight/brakelight fixtures.
    pub fn taillight_positions_world(&self, pos: Vec2, fwd: Vec2, right: Vec2, lr: f32) -> (Vec2, Vec2) {
        let rear_tip = pos - fwd * (lr + self.rear_overhang - self.light_inset);
        let half_spread = self.half_width() * self.taillight_spread;
        (rear_tip - right * half_spread, rear_tip + right * half_spread)
    }

    /// Computes `(front_extent, rear_extent, half_width)` for SAT collision `BodyHull`.
    #[inline]
    pub fn to_body_hull(&self, lf: f32, lr: f32) -> (f32, f32, f32) {
        (lf + self.front_overhang, lr + self.rear_overhang, self.half_width())
    }
}

/// High-level suspension kinematic architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuspensionArchetype {
    /// 1. Racing Karts: near-zero travel, rigid tubular chassis, no dampers.
    RigidKart,
    /// 2. NASCAR Cup / Classic Muscle: Solid rear live axle with coupled roll and axle tramp.
    SolidLiveAxle,
    /// 3. Grassroots / GT4 / Rally Hatch: MacPherson strut front, camber loss under heavy roll.
    MacPhersonStrut,
    /// 4. GT3 / Sports Prototypes: Double wishbone front & rear with camber gain geometry.
    DoubleWishbone,
    /// 5. Hypercars / GT1 Legends: Inboard pushrod, rising-rate heave spring, sensitive to bottoming.
    PushrodInboard,
    /// 6. Rallycross / Extreme Off-Road: Long-travel (250-400mm) bypass damping with high compliance.
    LongTravelOffRoad,
}

impl Default for SuspensionArchetype {
    fn default() -> Self {
        Self::DoubleWishbone
    }
}

impl SuspensionArchetype {
    pub const ALL: [Self; 6] = [
        Self::RigidKart,
        Self::SolidLiveAxle,
        Self::MacPhersonStrut,
        Self::DoubleWishbone,
        Self::PushrodInboard,
        Self::LongTravelOffRoad,
    ];

    /// Structural resilience factor governing resistance to collision, kerb, and landing damage (Spec 078).
    /// Higher values indicate greater robustness against failure.
    pub fn robustness_factor(&self) -> f32 {
        match self {
            Self::PushrodInboard => 0.45,
            Self::RigidKart => 0.55,
            Self::MacPhersonStrut => 0.75,
            Self::DoubleWishbone => 1.00,
            Self::SolidLiveAxle => 1.45,
            Self::LongTravelOffRoad => 2.20,
        }
    }

    /// Post-race replacement part cost multiplier (Spec 078).
    pub fn part_cost_multiplier(&self) -> f32 {
        match self {
            Self::RigidKart => 0.50,
            Self::SolidLiveAxle => 0.65,
            Self::MacPhersonStrut => 0.80,
            Self::LongTravelOffRoad => 1.10,
            Self::DoubleWishbone => 1.20,
            Self::PushrodInboard => 2.20,
        }
    }
}

/// Detailed suspension geometry and compliance settings for an axle or vehicle corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SuspensionCornerConfig {
    /// Suspension kinematic architecture.
    pub archetype: SuspensionArchetype,
    /// Wheel rate stiffness (N/m).
    pub spring_rate: f32,
    /// Damping ratio in bump / compression [0.2 - 0.9].
    pub bump_damping_ratio: f32,
    /// Damping ratio in rebound / extension [0.4 - 1.2].
    pub rebound_damping_ratio: f32,
    /// Maximum bump travel before hitting bump stop (meters).
    pub max_bump_travel: f32,
    /// Maximum rebound extension travel (meters).
    pub max_rebound_travel: f32,
    /// Static camber angle at rest (radians, negative = top tilted inward).
    pub static_camber: f32,
    /// Camber recovery factor [0.0 = full camber loss with roll, 1.0 = full camber preservation].
    pub camber_recovery: f32,
}

impl Default for SuspensionCornerConfig {
    fn default() -> Self {
        Self {
            archetype: SuspensionArchetype::DoubleWishbone,
            spring_rate: 85_000.0,
            bump_damping_ratio: 0.68,
            rebound_damping_ratio: 0.82,
            max_bump_travel: 0.045,
            max_rebound_travel: 0.035,
            static_camber: -0.052,
            camber_recovery: 0.85,
        }
    }
}

/// Vehicle-level suspension setup comprising front and rear axle configurations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SuspensionConfig {
    /// Front axle corner suspension settings.
    pub front: SuspensionCornerConfig,
    /// Rear axle corner suspension settings.
    pub rear: SuspensionCornerConfig,
    /// Front anti-roll bar torsional stiffness (N*m/rad).
    pub front_arb_rate: f32,
    /// Rear anti-roll bar torsional stiffness (N*m/rad).
    pub rear_arb_rate: f32,
    /// Height of front roll center above ground (meters).
    pub front_roll_center_height: f32,
    /// Height of rear roll center above ground (meters).
    pub rear_roll_center_height: f32,
    /// Suspension natural frequency for body lag filtering (Hz).
    pub response_frequency_hz: f32,
}

impl Default for SuspensionConfig {
    fn default() -> Self {
        Self::for_archetype(SuspensionArchetype::DoubleWishbone)
    }
}

impl SuspensionConfig {
    /// Generates factory calibrated presets for standard archetypes.
    pub fn for_archetype(archetype: SuspensionArchetype) -> Self {
        match archetype {
            SuspensionArchetype::RigidKart => Self::rigid_kart(),
            SuspensionArchetype::SolidLiveAxle => Self::solid_live_axle(),
            SuspensionArchetype::MacPhersonStrut => Self::macpherson_strut(),
            SuspensionArchetype::DoubleWishbone => Self::double_wishbone(),
            SuspensionArchetype::PushrodInboard => Self::pushrod_inboard(),
            SuspensionArchetype::LongTravelOffRoad => Self::long_travel_offroad(),
        }
    }

    pub fn rigid_kart() -> Self {
        let corner = SuspensionCornerConfig {
            archetype: SuspensionArchetype::RigidKart,
            spring_rate: 450_000.0,
            bump_damping_ratio: 0.10,
            rebound_damping_ratio: 0.15,
            max_bump_travel: 0.008,
            max_rebound_travel: 0.005,
            static_camber: 0.0,
            camber_recovery: 0.0,
        };
        Self {
            front: corner,
            rear: corner,
            front_arb_rate: 0.0,
            rear_arb_rate: 0.0,
            front_roll_center_height: 0.02,
            rear_roll_center_height: 0.02,
            response_frequency_hz: 8.0,
        }
    }

    pub fn double_wishbone() -> Self {
        let front = SuspensionCornerConfig {
            archetype: SuspensionArchetype::DoubleWishbone,
            spring_rate: 85_000.0,
            bump_damping_ratio: 0.68,
            rebound_damping_ratio: 0.82,
            max_bump_travel: 0.045,
            max_rebound_travel: 0.035,
            static_camber: -0.052, // ~ -3.0 deg GT3 setup
            camber_recovery: 0.90,
        };
        let rear = SuspensionCornerConfig {
            archetype: SuspensionArchetype::DoubleWishbone,
            spring_rate: 95_000.0,
            bump_damping_ratio: 0.70,
            rebound_damping_ratio: 0.85,
            max_bump_travel: 0.045,
            max_rebound_travel: 0.035,
            static_camber: -0.035, // ~ -2.0 deg GT3 setup
            camber_recovery: 0.90,
        };
        Self {
            front,
            rear,
            front_arb_rate: 4500.0,
            rear_arb_rate: 3200.0,
            front_roll_center_height: 0.08,
            rear_roll_center_height: 0.10,
            response_frequency_hz: 4.2,
        }
    }

    pub fn macpherson_strut() -> Self {
        let front = SuspensionCornerConfig {
            archetype: SuspensionArchetype::MacPhersonStrut,
            spring_rate: 38_000.0,
            bump_damping_ratio: 0.58,
            rebound_damping_ratio: 0.70,
            max_bump_travel: 0.075,
            max_rebound_travel: 0.060,
            static_camber: -0.035, // ~ -2.0 deg GT4 setup
            camber_recovery: 0.40, // MacPherson loses camber with roll
        };
        let rear = SuspensionCornerConfig {
            archetype: SuspensionArchetype::DoubleWishbone,
            spring_rate: 42_000.0,
            bump_damping_ratio: 0.60,
            rebound_damping_ratio: 0.72,
            max_bump_travel: 0.070,
            max_rebound_travel: 0.055,
            static_camber: -0.026,
            camber_recovery: 0.75,
        };
        Self {
            front,
            rear,
            front_arb_rate: 1800.0,
            rear_arb_rate: 1400.0,
            front_roll_center_height: 0.06,
            rear_roll_center_height: 0.09,
            response_frequency_hz: 3.5,
        }
    }

    pub fn solid_live_axle() -> Self {
        let front = SuspensionCornerConfig {
            archetype: SuspensionArchetype::DoubleWishbone,
            spring_rate: 65_000.0,
            bump_damping_ratio: 0.62,
            rebound_damping_ratio: 0.75,
            max_bump_travel: 0.065,
            max_rebound_travel: 0.050,
            static_camber: -0.060,
            camber_recovery: 0.80,
        };
        let rear = SuspensionCornerConfig {
            archetype: SuspensionArchetype::SolidLiveAxle,
            spring_rate: 55_000.0,
            bump_damping_ratio: 0.55,
            rebound_damping_ratio: 0.70,
            max_bump_travel: 0.065,
            max_rebound_travel: 0.050,
            static_camber: 0.0,
            camber_recovery: 0.0,
        };
        Self {
            front,
            rear,
            front_arb_rate: 4800.0,
            rear_arb_rate: 1800.0,
            front_roll_center_height: 0.09,
            rear_roll_center_height: 0.22, // High truck-arm roll center
            response_frequency_hz: 3.8,
        }
    }

    pub fn pushrod_inboard() -> Self {
        let front = SuspensionCornerConfig {
            archetype: SuspensionArchetype::PushrodInboard,
            spring_rate: 140_000.0,
            bump_damping_ratio: 0.82,
            rebound_damping_ratio: 0.95,
            max_bump_travel: 0.025,
            max_rebound_travel: 0.020,
            static_camber: -0.045,
            camber_recovery: 0.95,
        };
        let rear = SuspensionCornerConfig {
            archetype: SuspensionArchetype::PushrodInboard,
            spring_rate: 160_000.0,
            bump_damping_ratio: 0.85,
            rebound_damping_ratio: 0.98,
            max_bump_travel: 0.025,
            max_rebound_travel: 0.020,
            static_camber: -0.030,
            camber_recovery: 0.95,
        };
        Self {
            front,
            rear,
            front_arb_rate: 8500.0,
            rear_arb_rate: 6500.0,
            front_roll_center_height: 0.05,
            rear_roll_center_height: 0.06,
            response_frequency_hz: 5.5,
        }
    }

    pub fn long_travel_offroad() -> Self {
        let corner = SuspensionCornerConfig {
            archetype: SuspensionArchetype::LongTravelOffRoad,
            spring_rate: 22_000.0,
            bump_damping_ratio: 0.45,
            rebound_damping_ratio: 0.65,
            max_bump_travel: 0.240,
            max_rebound_travel: 0.180,
            static_camber: -0.015,
            camber_recovery: 0.60,
        };
        Self {
            front: corner,
            rear: corner,
            front_arb_rate: 1200.0,
            rear_arb_rate: 800.0,
            front_roll_center_height: 0.16,
            rear_roll_center_height: 0.18,
            response_frequency_hz: 2.8,
        }
    }
}

/// Physical powertrain and engine layout architecture (Spec 078).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnginePlacement {
    /// Front-mounted engine (Stock Car, GT4 Coupe, Rally Hatch).
    FrontEngine,
    /// Mid-mounted engine behind cockpit, ahead of rear axle (Supercar Lites, Ferrari 296, Kart).
    MidEngine,
    /// Rear-mounted engine over or behind rear axle (Porsche 911 GT3).
    RearEngine,
}

impl Default for EnginePlacement {
    fn default() -> Self {
        Self::FrontEngine
    }
}

impl EnginePlacement {
    pub const ALL: [Self; 3] = [Self::FrontEngine, Self::MidEngine, Self::RearEngine];

    /// Powertrain overhaul labor and complexity cost multiplier for garage repairs (Spec 078).
    pub fn repair_cost_multiplier(&self) -> f32 {
        match self {
            Self::FrontEngine => 1.00,
            Self::MidEngine => 1.35,
            Self::RearEngine => 1.50,
        }
    }
}

/// Vehicle physical dimensions, mass properties, powertrain parameters, and steering geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(from = "CarConfigRaw")]
pub struct CarConfig {
    /// Physical engine mounting location governing collision vulnerability (Spec 078).
    #[serde(default)]
    pub engine_placement: EnginePlacement,
    /// Whether mechanical damage simulation (collisions, kerb strikes, jump landings) is enabled.
    #[serde(default = "default_damage_enabled")]
    pub damage_enabled: bool,
    /// Total vehicle mass in kilograms.
    pub mass: f32,
    /// Yaw moment of inertia around the vertical axis in kg*m^2.
    pub inertia: f32,
    /// Wheelbase (distance from front to rear axle) in meters.
    pub wheelbase: f32,
    /// Track width (distance between left and right wheels) in meters.
    pub track_width: f32,
    /// Distance from vehicle center of gravity (CG) to front axle in meters.
    pub cg_to_front: f32,
    /// Distance from vehicle center of gravity (CG) to rear axle in meters.
    pub cg_to_rear: f32,
    /// Height of center of gravity above ground in meters (governs dynamic weight transfer).
    pub cg_height: f32,

    /// Maximum forward engine tractive force at drive wheels in Newtons.
    pub max_engine_force: f32,
    /// Maximum reverse tractive force in Newtons.
    pub max_reverse_force: f32,
    /// Maximum total braking force in Newtons.
    pub max_brake_force: f32,
    /// Additional braking force applied to rear wheels during handbrake in Newtons.
    pub handbrake_force: f32,
    /// Front braking distribution bias [0.0 = full rear, 0.5 = 50/50, 1.0 = full front].
    pub brake_bias: f32,
    /// Drive power distribution bias [0.0 = RWD, 0.5 = AWD, 1.0 = FWD].
    pub drive_bias: f32,
    /// Front axle differential model (governs cross-axle torque distribution on wheels 0 and 1).
    #[serde(default = "default_front_differential")]
    pub front_differential: DifferentialType,
    /// Rear axle differential model (governs cross-axle torque distribution on wheels 2 and 3).
    #[serde(default = "default_rear_differential")]
    pub rear_differential: DifferentialType,
    /// Maximum top speed reachable in m/s (engine power tapers near top speed).
    pub top_speed_mps: f32,

    /// Maximum front wheel steering angle in radians.
    pub max_steer_angle: f32,
    /// Steering actuator rate of change in rad/s.
    pub steer_speed: f32,
    /// Steering auto-centering return rate in rad/s.
    pub steer_return_speed: f32,
    /// Steering speed multiplier when player is counter-steering during a drift.
    pub counter_steer_assist: f32,

    /// Aerodynamic drag coefficient (0.5 * Cd * A * air_density).
    pub air_drag_coefficient: f32,
    /// Lateral aerodynamic drag coefficient.
    pub lateral_drag_coefficient: f32,
    /// Rolling resistance coefficient on standard asphalt.
    pub rolling_resistance_coefficient: f32,
    /// Yaw angular velocity damping coefficient in N*m*s/rad.
    pub angular_damping: f32,

    /// Handling balance: front axle share of lateral load transfer [0.35-0.65] (Spec 043).
    /// Higher = more understeer (the front tires lose grip first), lower = more oversteer.
    pub roll_balance: f32,
    /// How fast load transfer follows the chassis acceleration, in Hz [2-10] (Spec 043).
    /// Low = lazy, floaty weight shifts; high = sharp, twitchy.
    pub weight_transfer_hz: f32,
    /// Caster jacking diagonal load transfer factor [0.0 = cars with differential, ~1.0-1.5 = karts with solid axle].
    #[serde(default)]
    pub caster_jacking_factor: f32,

    /// Engine braking retarding coefficient on throttle release [0.0 = none, 0.15 = strong].
    pub engine_braking_coefficient: f32,
    /// Front axle share of engine-braking retard [0.2-0.6] (Spec 043). Lower = more lift-off oversteer.
    pub engine_brake_front_share: f32,
    /// Aerodynamic downforce coefficient (0.5 * Cl * A * air_density) scaling vertical load with V^2.
    pub downforce_coefficient: f32,

    /// Tire model. The rear axle uses it scaled by `rear_axle`.
    pub tire: TireConfig,
    /// Rear tire relative to `tire` (staggered tires, stability tuning).
    #[serde(default)]
    pub rear_axle: RearAxleTire,
    /// Driver electronic stability and traction assistance settings.
    pub assists: DriverAssistsConfig,
    /// Terrain interaction modifiers (sand flotation, mud paddles, ice studs).
    pub terrain: TerrainInteractionConfig,
    /// Per-driver handling aids set from the player's settings (bots keep the defaults).
    #[serde(default)]
    pub player: PlayerHandling,
    /// Decoupled wheel assembly configurations for all 4 corners [FL, FR, RL, RR].
    #[serde(default = "default_wheel_assemblies")]
    pub wheels: [WheelAssemblyConfig; 4],
    /// Physical chassis exterior dimensions and anchor points (Spec 075).
    #[serde(default)]
    pub chassis: ChassisSkeleton,
    /// 4-corner suspension compliance, damping, and roll kinematics (Spec 076).
    #[serde(default)]
    pub suspension: SuspensionConfig,
}

#[derive(Deserialize)]
struct CarConfigRaw {
    pub mass: f32,
    pub inertia: f32,
    pub wheelbase: f32,
    pub track_width: f32,
    pub cg_to_front: f32,
    pub cg_to_rear: f32,
    pub cg_height: f32,
    pub max_engine_force: f32,
    pub max_reverse_force: f32,
    pub max_brake_force: f32,
    pub handbrake_force: f32,
    pub brake_bias: f32,
    pub drive_bias: f32,
    #[serde(default)]
    pub front_differential: Option<DifferentialType>,
    #[serde(default)]
    pub rear_differential: Option<DifferentialType>,
    pub top_speed_mps: f32,
    pub max_steer_angle: f32,
    pub steer_speed: f32,
    pub steer_return_speed: f32,
    pub counter_steer_assist: f32,
    // Legacy (pre Spec 043) physics steering attenuation: replaced by grip-aware authority.
    #[serde(default, rename = "speed_sensitive_steer_factor")]
    pub _speed_sensitive_steer_factor: Option<f32>,
    pub air_drag_coefficient: f32,
    pub lateral_drag_coefficient: f32,
    pub rolling_resistance_coefficient: f32,
    pub angular_damping: f32,
    // Legacy (pre Spec 043) load transfer scales: accepted and ignored.
    #[serde(default, rename = "weight_transfer_longitudinal")]
    pub _weight_transfer_longitudinal: Option<f32>,
    #[serde(default, rename = "weight_transfer_lateral")]
    pub _weight_transfer_lateral: Option<f32>,
    #[serde(default)]
    pub roll_balance: Option<f32>,
    #[serde(default)]
    pub weight_transfer_hz: Option<f32>,
    #[serde(default)]
    pub caster_jacking_factor: f32,
    pub engine_braking_coefficient: f32,
    #[serde(default)]
    pub engine_brake_front_share: Option<f32>,
    pub downforce_coefficient: f32,
    pub tire: TireConfig,
    #[serde(default)]
    pub rear_axle: RearAxleTire,
    pub assists: DriverAssistsConfig,
    #[serde(default)]
    pub terrain: TerrainInteractionConfig,
    #[serde(default)]
    pub player: PlayerHandling,
    #[serde(default)]
    pub wheels: Option<[WheelAssemblyConfig; 4]>,
    #[serde(default)]
    pub chassis: Option<ChassisSkeleton>,
    #[serde(default)]
    pub suspension: Option<SuspensionConfig>,
    #[serde(default)]
    pub engine_placement: Option<EnginePlacement>,
    #[serde(default)]
    pub damage_enabled: Option<bool>,
}

impl From<CarConfigRaw> for CarConfig {
    fn from(raw: CarConfigRaw) -> Self {
        let wheels = raw.wheels.unwrap_or_else(|| {
            CarConfig::default_wheel_assemblies_for(raw.tire, raw.brake_bias, raw.drive_bias)
        });

        let front_differential = raw
            .front_differential
            .unwrap_or_else(default_front_differential);
        let rear_differential = raw.rear_differential.unwrap_or_else(|| {
            if raw.caster_jacking_factor > 0.0 {
                DifferentialType::Spool
            } else {
                default_rear_differential()
            }
        });

        let chassis = raw.chassis.unwrap_or_else(|| {
            ChassisSkeleton::synthesize_proportional(raw.wheelbase, raw.track_width)
        });

        let suspension = raw.suspension.unwrap_or_else(|| {
            if raw.caster_jacking_factor > 0.0 {
                SuspensionConfig::for_archetype(SuspensionArchetype::RigidKart)
            } else {
                SuspensionConfig::default()
            }
        });

        let engine_placement = raw.engine_placement.unwrap_or_else(|| {
            if raw.caster_jacking_factor > 0.0 {
                EnginePlacement::MidEngine
            } else {
                EnginePlacement::FrontEngine
            }
        });

        let mut cfg = Self {
            engine_placement,
            damage_enabled: raw.damage_enabled.unwrap_or(true),
            mass: raw.mass,
            inertia: raw.inertia,
            wheelbase: raw.wheelbase,
            track_width: raw.track_width,
            cg_to_front: raw.cg_to_front,
            cg_to_rear: raw.cg_to_rear,
            cg_height: raw.cg_height,
            max_engine_force: raw.max_engine_force,
            max_reverse_force: raw.max_reverse_force,
            max_brake_force: raw.max_brake_force,
            handbrake_force: raw.handbrake_force,
            brake_bias: raw.brake_bias,
            drive_bias: raw.drive_bias,
            front_differential,
            rear_differential,
            top_speed_mps: raw.top_speed_mps,
            max_steer_angle: raw.max_steer_angle,
            steer_speed: raw.steer_speed,
            steer_return_speed: raw.steer_return_speed,
            counter_steer_assist: raw.counter_steer_assist,
            air_drag_coefficient: raw.air_drag_coefficient,
            lateral_drag_coefficient: raw.lateral_drag_coefficient,
            rolling_resistance_coefficient: raw.rolling_resistance_coefficient,
            angular_damping: raw.angular_damping,
            roll_balance: raw
                .roll_balance
                .unwrap_or(raw.cg_to_rear / (raw.cg_to_front + raw.cg_to_rear).max(1e-3)),
            weight_transfer_hz: raw.weight_transfer_hz.unwrap_or(DEFAULT_WEIGHT_TRANSFER_HZ),
            caster_jacking_factor: raw.caster_jacking_factor,
            engine_braking_coefficient: raw.engine_braking_coefficient,
            engine_brake_front_share: raw
                .engine_brake_front_share
                .unwrap_or(0.35 + 0.30 * raw.drive_bias.clamp(0.0, 1.0)),
            downforce_coefficient: raw.downforce_coefficient,
            tire: raw.tire,
            rear_axle: raw.rear_axle,
            assists: raw.assists,
            terrain: raw.terrain,
            player: raw.player,
            wheels,
            chassis,
            suspension,
        };
        cfg.finalize();
        cfg
    }
}

impl Default for CarConfig {
    fn default() -> Self {
        Self::sports_car()
    }
}

impl CarConfig {
    /// Generates standard 4-wheel assemblies matching current tire model, brake bias, and drive bias.
    pub fn default_wheel_assemblies_for(
        tire: TireConfig,
        brake_bias: f32,
        drive_bias: f32,
    ) -> [WheelAssemblyConfig; 4] {
        let front_brake = brake_bias * 0.5;
        let rear_brake = (1.0 - brake_bias) * 0.5;
        let front_drive = drive_bias * 0.5;
        let rear_drive = (1.0 - drive_bias) * 0.5;
        [
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                compound: TireCompoundConfig::from_id(CompoundId::MediumSlick),
                tire_model: tire,
                brake_bias_factor: front_brake,
                drive_torque_factor: front_drive,
            },
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                compound: TireCompoundConfig::from_id(CompoundId::MediumSlick),
                tire_model: tire,
                brake_bias_factor: front_brake,
                drive_torque_factor: front_drive,
            },
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                compound: TireCompoundConfig::from_id(CompoundId::MediumSlick),
                tire_model: tire,
                brake_bias_factor: rear_brake,
                drive_torque_factor: rear_drive,
            },
            WheelAssemblyConfig {
                tire_radius: 0.32,
                tire_width: 0.24,
                rotational_inertia: 1.25,
                compound: TireCompoundConfig::from_id(CompoundId::MediumSlick),
                tire_model: tire,
                brake_bias_factor: rear_brake,
                drive_torque_factor: rear_drive,
            },
        ]
    }

    /// Derives every per-wheel field from the axle-level settings (Spec 043 single source of truth).
    ///
    /// Tire model from `tire` / `rear_axle`, brake share from `brake_bias`, drive share from
    /// `drive_bias`. Wheel geometry (radius, width, inertia) stays per wheel. Idempotent.
    /// Called by every preset, by deserialization, and by `Car::new` / `Car::set_config`.
    pub fn finalize(&mut self) {
        let rear_tire = self.rear_axle.apply(&self.tire);
        let bb = self.brake_bias.clamp(0.0, 1.0);
        let db = self.drive_bias.clamp(0.0, 1.0);
        for (i, w) in self.wheels.iter_mut().enumerate() {
            let front = i < 2;
            let tire_cfg = if front { self.tire } else { rear_tire };
            w.tire_model = tire_cfg;
            w.compound.base_grip = tire_cfg.grip;
            w.brake_bias_factor = if front { bb * 0.5 } else { (1.0 - bb) * 0.5 };
            w.drive_torque_factor = if front { db * 0.5 } else { (1.0 - db) * 0.5 };
        }
        self.wheelbase = self.wheelbase.clamp(0.90, 4.50);
        self.track_width = self.track_width.clamp(0.70, 3.00);
        if self.chassis.body_width <= 0.0 {
            self.chassis = ChassisSkeleton::synthesize_proportional(self.wheelbase, self.track_width);
        }
        self.chassis.front_overhang = self.chassis.front_overhang.clamp(0.10, 2.00);
        self.chassis.rear_overhang = self.chassis.rear_overhang.clamp(0.10, 2.00);
    }

    /// Builder form of [`CarConfig::finalize`].
    #[must_use]
    pub fn finalized(mut self) -> Self {
        self.finalize();
        self
    }

    /// Equips all four wheels with the specified tire compound profile.
    pub fn set_compound(&mut self, compound_id: CompoundId) {
        let profile = TireCompoundConfig::from_id(compound_id);
        for w in self.wheels.iter_mut() {
            w.compound = profile;
            w.compound.base_grip = w.tire_model.grip;
        }
    }

    /// Builder method to equip all four wheels with the specified tire compound.
    #[must_use]
    pub fn with_compound(mut self, compound_id: CompoundId) -> Self {
        self.set_compound(compound_id);
        self
    }

    /// Standard balanced sports car tuned for GeneRally-style arcade drift racing.
    pub fn sports_car() -> Self {
        let tire = TireConfig::default();
        Self {
            mass: 1050.0,
            inertia: 1450.0,
            wheelbase: 2.40,
            track_width: 1.40,
            cg_to_front: 1.10,
            cg_to_rear: 1.30,
            cg_height: 0.35,

            max_engine_force: 6800.0,
            max_reverse_force: 4420.0,
            max_brake_force: 11500.0,
            handbrake_force: 7500.0,
            brake_bias: 0.56,
            drive_bias: 0.0, // RWD arcade feel
            front_differential: DifferentialType::Open,
            rear_differential: DifferentialType::LimitedSlip {
                power_lock: 0.56,
                coast_lock: 0.33,
                preload_nm: 65.0,
            },
            top_speed_mps: 58.0, // ~208 km/h

            max_steer_angle: 0.68, // ~39 deg responsive turning lock
            steer_speed: 5.5,
            steer_return_speed: 7.0,
            counter_steer_assist: 1.3,

            air_drag_coefficient: 0.42,
            lateral_drag_coefficient: 1.20,
            rolling_resistance_coefficient: 0.015,
            angular_damping: 120.0,

            roll_balance: 0.66,
            weight_transfer_hz: DEFAULT_WEIGHT_TRANSFER_HZ,
            caster_jacking_factor: 0.0,

            engine_braking_coefficient: 0.12,
            engine_brake_front_share: 0.35,
            downforce_coefficient: 0.65,

            tire,
            // Rear tires peak earlier (stiffer) than the fronts: a stable understeer gradient in
            // the linear range. With equal tires the car is neutral and diverges above ~40 m/s.
            rear_axle: RearAxleTire {
                grip_scale: 1.0,
                peak_slip_scale: 0.81,
            },
            assists: DriverAssistsConfig::arcade(),
            terrain: TerrainInteractionConfig::default(),
            player: PlayerHandling::default(),
            wheels: Self::default_wheel_assemblies_for(tire, 0.56, 0.0),
            chassis: ChassisSkeleton::new(0.80, 0.90, 1.70, -0.40, 0.30, 0.65, 0.70, 0.05),
            suspension: SuspensionConfig::for_archetype(SuspensionArchetype::DoubleWishbone),
            engine_placement: EnginePlacement::FrontEngine,
            damage_enabled: true,
        }
        .finalized()
    }

    /// Dedicated drift machine: aggressive rear power, loose tail, quick counter-steer.
    pub fn drift_car() -> Self {
        let mut cfg = Self::sports_car();
        cfg.mass = 980.0;
        cfg.inertia = 1250.0;
        cfg.max_engine_force = 8200.0;
        cfg.max_reverse_force = 5330.0;
        cfg.max_steer_angle = 0.78; // ~45 deg wide drift lock
        cfg.counter_steer_assist = 1.6;
        cfg.angular_damping = 114.0;
        cfg.roll_balance = 0.50;
        cfg.brake_bias = 0.56;
        cfg.engine_braking_coefficient = 0.10;
        cfg.downforce_coefficient = 0.45;
        cfg.tire.slide_grip = 0.92;
        cfg.rear_axle = RearAxleTire::NEUTRAL; // a drift car keeps a lively rear
        cfg.drive_bias = 0.0;
        cfg.rear_differential = DifferentialType::LimitedSlip {
            power_lock: 0.48,
            coast_lock: 0.16,
            preload_nm: 62.0,
        };
        cfg.assists = DriverAssistsConfig::sport();
        cfg.finalized()
    }

    /// Go-kart preset: ultra-responsive, lightweight, high lateral grip, direct steering.
    /// Features staggered open-wheel dimensions: narrow front tires (r=0.18m, w=0.12m)
    /// and wide rear tires (r=0.20m, w=0.21m) delivering >= 35% higher peak lateral force.
    pub fn kart() -> Self {
        let front_tire = TireConfig {
            grip: 1.35, // sticky kart slicks (real karts pull ~1.5-2 g)
            peak_slip_angle_deg: 7.0,
            slide_grip: 0.88,
            skid_threshold: 0.08,
            skid_full_threshold: 0.28,
            ..TireConfig::default()
        };
        // Wide rear slicks: more grip than the narrow fronts (1.50 vs 1.35)
        let rear_axle = RearAxleTire {
            grip_scale: 1.50 / 1.35,
            peak_slip_scale: 0.85,
        };
        let rear_tire = rear_axle.apply(&front_tire);
        // I_front = 0.15 kg*m^2, I_rear = 0.24 kg*m^2 (I_rear > I_front)
        let wheels = [
            WheelAssemblyConfig {
                tire_radius: 0.18,
                tire_width: 0.12,
                rotational_inertia: 0.15,
                compound: TireCompoundConfig::from_id(CompoundId::SoftSlick),
                tire_model: front_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.18,
                tire_width: 0.12,
                rotational_inertia: 0.15,
                compound: TireCompoundConfig::from_id(CompoundId::SoftSlick),
                tire_model: front_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.20,
                tire_width: 0.21,
                rotational_inertia: 0.24,
                compound: TireCompoundConfig::from_id(CompoundId::SoftSlick),
                tire_model: rear_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.50,
            },
            WheelAssemblyConfig {
                tire_radius: 0.20,
                tire_width: 0.21,
                rotational_inertia: 0.24,
                compound: TireCompoundConfig::from_id(CompoundId::SoftSlick),
                tire_model: rear_tire,
                brake_bias_factor: 0.25,
                drive_torque_factor: 0.50,
            },
        ];

        Self {
            mass: 180.0,
            inertia: 120.0,
            wheelbase: 1.05,
            track_width: 0.85,
            cg_to_front: 0.60,
            cg_to_rear: 0.45,
            cg_height: 0.18,

            max_engine_force: 2200.0,
            max_reverse_force: 1430.0,
            max_brake_force: 2400.0,
            handbrake_force: 1800.0,
            brake_bias: 0.54,
            drive_bias: 0.0,
            front_differential: DifferentialType::Open,
            rear_differential: DifferentialType::Spool,
            top_speed_mps: 32.0, // ~115 km/h

            max_steer_angle: 0.73, // ~41.8 deg direct 1:1 racing kart lock
            steer_speed: 10.5,
            steer_return_speed: 14.0,
            counter_steer_assist: 1.25,

            air_drag_coefficient: 0.35,
            lateral_drag_coefficient: 1.00,
            rolling_resistance_coefficient: 0.018,
            angular_damping: 35.0,

            roll_balance: 0.50,
            weight_transfer_hz: 5.0,
            caster_jacking_factor: 1.25,

            engine_braking_coefficient: 0.18,
            engine_brake_front_share: 0.35,
            downforce_coefficient: 0.10,

            tire: front_tire,
            rear_axle,
            assists: DriverAssistsConfig {
                tcs_enabled: true,
                tcs_slip_threshold: 0.16,
                tcs_strength: 0.70,
                tcs_slip_angle_deg: 12.0,
                esc_sideslip_limit_deg: 12.0,
                tcs_drift_bypass: false,
                esc_enabled: false, // Pure analog chassis yaw rotation for karts
                esc_yaw_threshold: 0.40,
                esc_strength: 0.0,
                counter_steer_assist_enabled: true,
                counter_steer_assist_strength: 0.60,
                abs_enabled: true,
                abs_slip_threshold: 0.20,
                abs_strength: 0.75,
                handbrake_bypass: true,
            },
            terrain: TerrainInteractionConfig {
                sand_flotation: 1.0,
                mud_flotation: 1.0,
                ice_grip_multiplier: 0.80,
            },
            player: PlayerHandling::default(),
            wheels,
            chassis: ChassisSkeleton::new(0.16, 0.12, 1.10, -0.15, 0.10, 0.65, 0.70, 0.05),
            suspension: SuspensionConfig::for_archetype(SuspensionArchetype::RigidKart),
            engine_placement: EnginePlacement::MidEngine,
            damage_enabled: true,
        }
        .finalized()
    }

    /// Alias for `kart()` representing the classic 200cc sprint kart.
    pub fn classic_kart() -> Self {
        Self::kart()
    }

    /// Rally spec: AWD traction, softened tire curve for loose surfaces, high ride height.
    pub fn rally_car() -> Self {
        let mut cfg = Self::sports_car();
        cfg.chassis = ChassisSkeleton::new(0.76, 0.68, 1.82, -0.35, 0.25, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::LongTravelOffRoad);
        cfg.drive_bias = 0.5; // AWD
        cfg.angular_damping = 126.0;
        cfg.engine_brake_front_share = 0.50;
        cfg.weight_transfer_hz = 2.5;
        cfg.brake_bias = 0.62;
        cfg.front_differential = DifferentialType::LimitedSlip {
            power_lock: 0.60,
            coast_lock: 0.40,
            preload_nm: 70.0,
        };
        cfg.rear_differential = DifferentialType::LimitedSlip {
            power_lock: 0.70,
            coast_lock: 0.50,
            preload_nm: 85.0,
        };
        cfg.cg_height = 0.42;
        cfg.max_engine_force = 7500.0;
        cfg.max_reverse_force = 4875.0;
        cfg.engine_braking_coefficient = 0.14;
        cfg.downforce_coefficient = 0.70;
        cfg.tire.peak_slip_angle_deg = 12.7;
        cfg.tire.slide_grip = 0.90;
        cfg.terrain = TerrainInteractionConfig {
            sand_flotation: 0.70,
            mud_flotation: 0.70,
            ice_grip_multiplier: 3.50,
        };
        for w in &mut cfg.wheels {
            w.tire_radius = 0.33;
            w.tire_width = 0.22;
            w.rotational_inertia = 1.30;
        }
        // Gravel and dirt tyres. Spec 074 gave every car MediumSlick, so rally and autocross cars ran
        // slicks on gravel (affinity 0.32) and bots stalled on Classic RX/AX circuits (tdrace-lxkv).
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// 850 BHP Trans-Am TA1 / NASCAR Cup tubular spaceframe V8 stock car spec.
    ///
    /// Characterized by massive pushrod V8 power (11,800 N tractive force, ~850 BHP),
    /// heavy inertia, low-to-moderate aerodynamic downforce, and progressive rear slip
    /// that demands precise throttle modulation to avoid power-oversteer while remaining
    /// planted and controllable at high superspeedway speeds.
    pub fn stock_car_ta1() -> Self {
        let tire = TireConfig {
            grip: 1.10,
            peak_slip_angle_deg: 8.5,
            slide_grip: 0.86,
            skid_threshold: 0.09,
            skid_full_threshold: 0.28,
            ..TireConfig::default()
        };
        let wheels = [
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                compound: TireCompoundConfig::from_id(CompoundId::HardSlick),
                tire_model: tire,
                brake_bias_factor: 0.31,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                compound: TireCompoundConfig::from_id(CompoundId::HardSlick),
                tire_model: tire,
                brake_bias_factor: 0.31,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                compound: TireCompoundConfig::from_id(CompoundId::HardSlick),
                tire_model: tire,
                brake_bias_factor: 0.19,
                drive_torque_factor: 0.50,
            },
            WheelAssemblyConfig {
                tire_radius: 0.36,
                tire_width: 0.30,
                rotational_inertia: 1.65,
                compound: TireCompoundConfig::from_id(CompoundId::HardSlick),
                tire_model: tire,
                brake_bias_factor: 0.19,
                drive_torque_factor: 0.50,
            },
        ];

        Self {
            mass: 1260.0,
            inertia: 1680.0,
            wheelbase: 2.75,
            track_width: 1.85,
            cg_to_front: 1.35,
            cg_to_rear: 1.40,
            cg_height: 0.32,

            max_engine_force: 11800.0, // ~850 BHP pushrod V8
            max_reverse_force: 7670.0,
            max_brake_force: 21000.0,
            handbrake_force: 7000.0,
            brake_bias: 0.57,
            drive_bias: 0.0, // RWD
            front_differential: DifferentialType::Open,
            rear_differential: DifferentialType::Spool,
            top_speed_mps: 89.0, // ~320 km/h (~200 mph)

            max_steer_angle: 0.47, // ~27 deg quick-ratio stock car steering box
            steer_speed: 7.5,
            steer_return_speed: 10.0,
            counter_steer_assist: 1.35,

            air_drag_coefficient: 0.52,
            lateral_drag_coefficient: 1.35,
            rolling_resistance_coefficient: 0.013,
            angular_damping: 148.0,

            roll_balance: 0.51,
            weight_transfer_hz: DEFAULT_WEIGHT_TRANSFER_HZ,
            caster_jacking_factor: 0.0,

            engine_braking_coefficient: 0.18,
            engine_brake_front_share: 0.35,
            downforce_coefficient: 1.25, // Moderate downforce package

            tire,
            rear_axle: RearAxleTire::default(),
            assists: DriverAssistsConfig::sport(),
            terrain: TerrainInteractionConfig::default(),
            player: PlayerHandling::default(),
            wheels,
            chassis: ChassisSkeleton::new(0.98, 1.25, 1.98, -0.45, 0.35, 0.70, 0.75, 0.06),
            suspension: SuspensionConfig::for_archetype(SuspensionArchetype::SolidLiveAxle),
            engine_placement: EnginePlacement::FrontEngine,
            damage_enabled: true,
        }
        .finalized()
    }

    /// Alias for `stock_car_ta1()` representing the 850 BHP Trans-Am TA1 spaceframe racer.
    pub fn trans_am_ta1() -> Self {
        Self::stock_car_ta1()
    }

    /// 300 BHP Sand Rail Buggy: ultralight chromoly spaceframe, pure RWD, wide stance,
    /// rear-biased weight, high-travel suspension compliance, and paddle tire grip.
    pub fn sand_rail() -> Self {
        let tire = TireConfig {
            grip: 1.20,
            peak_slip_angle_deg: 15.0,
            slide_grip: 0.94,
            load_sensitivity: 0.10, // compliant off-road carcass
            skid_threshold: 0.08,
            skid_full_threshold: 0.28,
            ..TireConfig::default()
        };
        let wheels = [
            WheelAssemblyConfig {
                tire_radius: 0.38,
                tire_width: 0.18,
                rotational_inertia: 1.10,
                compound: TireCompoundConfig::from_id(CompoundId::AllTerrain),
                tire_model: tire,
                brake_bias_factor: 0.275,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.38,
                tire_width: 0.18,
                rotational_inertia: 1.10,
                compound: TireCompoundConfig::from_id(CompoundId::AllTerrain),
                tire_model: tire,
                brake_bias_factor: 0.275,
                drive_torque_factor: 0.0,
            },
            WheelAssemblyConfig {
                tire_radius: 0.42,
                tire_width: 0.38,
                rotational_inertia: 1.85,
                compound: TireCompoundConfig::from_id(CompoundId::AllTerrain),
                tire_model: tire,
                brake_bias_factor: 0.225,
                drive_torque_factor: 0.50,
            },
            WheelAssemblyConfig {
                tire_radius: 0.42,
                tire_width: 0.38,
                rotational_inertia: 1.85,
                compound: TireCompoundConfig::from_id(CompoundId::AllTerrain),
                tire_model: tire,
                brake_bias_factor: 0.225,
                drive_torque_factor: 0.50,
            },
        ];

        Self {
            mass: 590.0,
            inertia: 720.0,
            wheelbase: 2.40,
            track_width: 1.75,
            cg_to_front: 1.44,
            cg_to_rear: 0.96,
            cg_height: 0.46,

            max_engine_force: 8800.0, // 300 BHP explosive power-to-weight
            max_reverse_force: 5000.0,
            max_brake_force: 11500.0,
            handbrake_force: 8200.0,
            brake_bias: 0.58,
            drive_bias: 0.0, // Pure RWD
            front_differential: DifferentialType::Open,
            rear_differential: DifferentialType::Spool,
            top_speed_mps: 55.5, // ~200 km/h

            max_steer_angle: 0.65, // ~37.2 deg responsive off-road lock
            steer_speed: 8.5,
            steer_return_speed: 9.5,
            counter_steer_assist: 1.55,

            air_drag_coefficient: 0.48,
            lateral_drag_coefficient: 1.40,
            rolling_resistance_coefficient: 0.018,
            angular_damping: 135.7,

            roll_balance: 0.42,
            weight_transfer_hz: 2.5,
            caster_jacking_factor: 0.0,

            engine_braking_coefficient: 0.14,
            engine_brake_front_share: 0.35,
            downforce_coefficient: 0.35,

            tire,
            rear_axle: RearAxleTire::default(),
            assists: DriverAssistsConfig::sport(),
            terrain: TerrainInteractionConfig {
                sand_flotation: 0.30,
                mud_flotation: 0.65,
                ice_grip_multiplier: 1.50,
            },
            player: PlayerHandling::default(),
            wheels,
            chassis: ChassisSkeleton::new(0.15, 0.42, 1.95, -0.40, 0.25, 0.60, 0.65, 0.05),
            suspension: SuspensionConfig::for_archetype(SuspensionArchetype::LongTravelOffRoad),
            engine_placement: EnginePlacement::RearEngine,
            damage_enabled: true,
        }
        .finalized()
    }

    /// 150 BHP Cross Car (Continental Autocross T1/T2).
    /// Single-seat tubular spaceframe buggy with a 750cc-850cc motorcycle superbike engine,
    /// ultra-fast rev acceleration, agile steering, and open-wheel long-travel articulation.
    pub fn cross_car() -> Self {
        let mut cfg = Self::sand_rail();
        cfg.mass = 420.0;
        cfg.inertia = 480.0;
        cfg.wheelbase = 2.15;
        cfg.track_width = 1.55;
        cfg.cg_to_front = 1.10;
        cfg.cg_to_rear = 1.05;
        cfg.cg_height = 0.38;
        cfg.max_engine_force = 5800.0;
        cfg.top_speed_mps = 50.0; // ~180 km/h
        cfg.drive_bias = 0.0; // RWD
        cfg.chassis = ChassisSkeleton::new(0.22, 0.28, 1.65, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::LongTravelOffRoad);
        cfg.engine_placement = EnginePlacement::MidEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.30;
            w.tire_width = 0.20;
            w.rotational_inertia = 0.85;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// 550+ BHP Touring AX Saloon (Continental Autocross T4).
    /// High-downforce closed-cockpit silhouette touring saloon with 50:50 AWD traction,
    /// front splitter, double wishbone geometry, and explosive gravel acceleration.
    pub fn touring_ax() -> Self {
        let mut cfg = Self::rally_car();
        cfg.mass = 1150.0;
        cfg.inertia = 1550.0;
        cfg.wheelbase = 2.55;
        cfg.track_width = 1.85;
        cfg.cg_to_front = 1.25;
        cfg.cg_to_rear = 1.30;
        cfg.cg_height = 0.39;
        cfg.max_engine_force = 10500.0;
        cfg.top_speed_mps = 58.0;
        cfg.drive_bias = 0.5; // AWD
        cfg.chassis = ChassisSkeleton::new(0.85, 0.78, 1.92, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::DoubleWishbone);
        cfg.engine_placement = EnginePlacement::FrontEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.33;
            w.tire_width = 0.24;
            w.rotational_inertia = 1.30;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// 4WD Dirt SuperBuggy & Buggy 1600 (Continental Autocross T3/T5).
    /// Mid-engine 4WD spaceframe dirt buggy combining long suspension travel with
    /// high power-to-weight and balanced 50:50 all-wheel-drive traction.
    pub fn super_buggy() -> Self {
        let mut cfg = Self::sand_rail();
        cfg.mass = 680.0;
        cfg.inertia = 780.0;
        cfg.wheelbase = 2.60;
        cfg.track_width = 1.82;
        cfg.cg_to_front = 1.32;
        cfg.cg_to_rear = 1.28;
        cfg.cg_height = 0.40;
        cfg.max_engine_force = 9200.0;
        cfg.top_speed_mps = 56.0;
        cfg.drive_bias = 0.5; // 4WD
        cfg.chassis = ChassisSkeleton::new(0.24, 0.48, 1.90, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::LongTravelOffRoad);
        cfg.engine_placement = EnginePlacement::MidEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.35;
            w.tire_width = 0.24;
            w.rotational_inertia = 1.20;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// Classic Baja Dune Buggy (Extreme Off-Road T1 Volkskraft).
    /// Classic air-cooled rear boxer engine, compact Beetle floorpan with forward curved nose,
    /// high rear engine overhang, and long-travel off-road swing/trailing arm suspension.
    pub fn dune_buggy_baja() -> Self {
        let mut cfg = Self::sand_rail();
        cfg.mass = 650.0;
        cfg.inertia = 760.0;
        cfg.wheelbase = 2.20;
        cfg.track_width = 1.65;
        cfg.cg_to_front = 1.28;
        cfg.cg_to_rear = 0.92; // 58% rear weight bias
        cfg.cg_height = 0.44;
        cfg.max_engine_force = 6200.0;
        cfg.top_speed_mps = 48.0;
        cfg.drive_bias = 0.0; // RWD
        cfg.chassis = ChassisSkeleton::new(0.42, 0.52, 1.75, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::LongTravelOffRoad);
        cfg.engine_placement = EnginePlacement::RearEngine;
        for (i, w) in cfg.wheels.iter_mut().enumerate() {
            if i < 2 {
                w.tire_radius = 0.34;
                w.tire_width = 0.18;
                w.rotational_inertia = 1.05;
            } else {
                w.tire_radius = 0.38;
                w.tire_width = 0.26;
                w.rotational_inertia = 1.45;
            }
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// 800 BHP Unlimited AWD Trophy Truck (Extreme Off-Road T2/T6/T7).
    /// Full-size off-road truck spaceframe with a front-mounted big-block V8, massive 30-inch
    /// suspension travel, high high-speed bump absorption, and substantial truck hull bodywork.
    pub fn trophy_truck() -> Self {
        let mut cfg = Self::sand_rail();
        cfg.mass = 2200.0;
        cfg.inertia = 3200.0;
        cfg.wheelbase = 3.20;
        cfg.track_width = 2.10;
        cfg.cg_to_front = 1.55;
        cfg.cg_to_rear = 1.65;
        cfg.cg_height = 0.52;
        cfg.max_engine_force = 13500.0;
        cfg.top_speed_mps = 60.0;
        cfg.drive_bias = 0.5; // AWD
        cfg.chassis = ChassisSkeleton::new(0.95, 1.20, 2.25, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::LongTravelOffRoad);
        cfg.engine_placement = EnginePlacement::FrontEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.46; // ~37-inch desert race tire
            w.tire_width = 0.32;
            w.rotational_inertia = 2.20;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// Heavy Mud Bogger 4x4 (Extreme Off-Road T4).
    /// High-riser dual solid live axle chassis with extreme ground clearance, elevated center of
    /// gravity, deep chevron tractor tires, and high-torque mud churning capability.
    pub fn mud_bogger() -> Self {
        let mut cfg = Self::sand_rail();
        cfg.mass = 2600.0;
        cfg.inertia = 3900.0;
        cfg.wheelbase = 3.10;
        cfg.track_width = 2.25;
        cfg.cg_to_front = 1.50;
        cfg.cg_to_rear = 1.60;
        cfg.cg_height = 0.65; // High-riser center of gravity
        cfg.max_engine_force = 14000.0;
        cfg.top_speed_mps = 45.0;
        cfg.drive_bias = 0.5;
        cfg.chassis = ChassisSkeleton::new(0.90, 1.15, 2.35, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::SolidLiveAxle);
        cfg.engine_placement = EnginePlacement::FrontEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.52; // 44-inch chevron tractor tire
            w.tire_width = 0.40;
            w.rotational_inertia = 3.10;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// 1500 BHP Monster Truck (Extreme Off-Road T5).
    /// Massive tubular chassis with 66-inch Terra tires, supercharged alcohol V8 mounted
    /// centrally, immense roll inertia, planetary 4-wheel steer capability, and huge jump compliance.
    pub fn monster_truck() -> Self {
        let mut cfg = Self::sand_rail();
        cfg.mass = 4200.0;
        cfg.inertia = 6500.0;
        cfg.wheelbase = 3.60;
        cfg.track_width = 2.60;
        cfg.cg_to_front = 1.80;
        cfg.cg_to_rear = 1.80;
        cfg.cg_height = 0.80;
        cfg.max_engine_force = 22000.0;
        cfg.top_speed_mps = 42.0;
        cfg.drive_bias = 0.5;
        cfg.chassis = ChassisSkeleton::new(0.80, 0.80, 2.75, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::LongTravelOffRoad);
        cfg.engine_placement = EnginePlacement::MidEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.84; // 66-inch Terra flotation tire
            w.tire_width = 0.65;
            w.rotational_inertia = 5.50;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// Compact FWD Junior Rally Supermini (Rallycross T1).
    /// Agile, lightweight front-wheel-drive hatchback with MacPherson strut front suspension,
    /// crisp lift-off oversteer rotation, and responsive naturally-aspirated power delivery.
    pub fn rally_junior_fwd() -> Self {
        let mut cfg = Self::rally_car();
        cfg.mass = 1030.0;
        cfg.inertia = 1250.0;
        cfg.wheelbase = 2.35;
        cfg.track_width = 1.50;
        cfg.cg_to_front = 1.05;
        cfg.cg_to_rear = 1.30;
        cfg.cg_height = 0.40;
        cfg.max_engine_force = 5800.0;
        cfg.top_speed_mps = 50.0;
        cfg.drive_bias = 1.0; // 100% FWD
        cfg.chassis = ChassisSkeleton::new(0.72, 0.58, 1.72, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::MacPhersonStrut);
        cfg.engine_placement = EnginePlacement::FrontEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.31;
            w.tire_width = 0.20;
            w.rotational_inertia = 1.10;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// 500+ BHP Group B Rally Monster (Rallycross T7).
    /// Ultra-lightweight mid-engine silhouette monster with explosive turbo boost,
    /// 50:50 AWD mechanical lock, high aerodynamic rear wing, and hair-trigger handling dynamics.
    pub fn rally_group_b() -> Self {
        let mut cfg = Self::rally_car();
        cfg.mass = 960.0;
        cfg.inertia = 1200.0;
        cfg.wheelbase = 2.30;
        cfg.track_width = 1.68;
        cfg.cg_to_front = 1.15;
        cfg.cg_to_rear = 1.15;
        cfg.cg_height = 0.38;
        cfg.max_engine_force = 11000.0;
        cfg.top_speed_mps = 64.0;
        cfg.drive_bias = 0.5;
        cfg.chassis = ChassisSkeleton::new(0.78, 0.75, 1.85, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::DoubleWishbone);
        cfg.engine_placement = EnginePlacement::MidEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.33;
            w.tire_width = 0.24;
            w.rotational_inertia = 1.25;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// RX1e / Group E Dual-Motor Electric RX Platform (Rallycross T5/T6).
    /// Low-center-of-gravity battery chassis with instantaneous dual-motor torque (500 kW / 680 BHP),
    /// seamless electronic torque distribution, and aggressive AWD corner exit traction.
    pub fn rally_electric_rx() -> Self {
        let mut cfg = Self::rally_car();
        cfg.mass = 1300.0;
        cfg.inertia = 1650.0;
        cfg.wheelbase = 2.45;
        cfg.track_width = 1.75;
        cfg.cg_to_front = 1.22;
        cfg.cg_to_rear = 1.23;
        cfg.cg_height = 0.36; // Lower CG from underfloor battery pack
        cfg.max_engine_force = 12500.0;
        cfg.top_speed_mps = 62.0;
        cfg.drive_bias = 0.5;
        cfg.chassis = ChassisSkeleton::new(0.74, 0.60, 1.80, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::DoubleWishbone);
        cfg.engine_placement = EnginePlacement::MidEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.34;
            w.tire_width = 0.25;
            w.rotational_inertia = 1.35;
        }
        cfg.set_compound(CompoundId::AllTerrain);
        cfg.finalized()
    }

    /// 250cc Twin Superkart GP with Aerodynamic Wings (Karting T5/T6).
    /// High-downforce aerodynamic racing kart equipped with front nosecone wing and high rear wing
    /// (Cl = 0.65), achieving speeds over 230 km/h with extreme lateral grip and rigid chassis agility.
    pub fn superkart_gp() -> Self {
        let mut cfg = Self::kart();
        cfg.mass = 215.0;
        cfg.inertia = 120.0;
        cfg.wheelbase = 1.25;
        cfg.track_width = 1.05;
        cfg.cg_to_front = 0.65;
        cfg.cg_to_rear = 0.60;
        cfg.cg_height = 0.20;
        cfg.max_engine_force = 4200.0;
        cfg.top_speed_mps = 65.0; // ~235 km/h
        cfg.downforce_coefficient = 0.65;
        cfg.chassis = ChassisSkeleton::new(0.32, 0.35, 1.20, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::RigidKart);
        cfg.engine_placement = EnginePlacement::MidEngine;
        for (i, w) in cfg.wheels.iter_mut().enumerate() {
            if i < 2 {
                w.tire_radius = 0.16;
                w.tire_width = 0.14;
            } else {
                w.tire_radius = 0.17;
                w.tire_width = 0.22;
            }
        }
        cfg.set_compound(CompoundId::SoftSlick);
        cfg.finalized()
    }

    /// V8 Super Truck (NASCAR T4 Craftsman Truck).
    /// Tubular spaceframe pickup truck chassis with high greenhouse, upright aerodynamic wake,
    /// solid live rear axle, and heavy pushrod V8 power delivery.
    pub fn stock_car_truck() -> Self {
        let mut cfg = Self::stock_car_ta1();
        cfg.mass = 1520.0;
        cfg.inertia = 2350.0;
        cfg.wheelbase = 2.85;
        cfg.track_width = 1.86;
        cfg.cg_to_front = 1.40;
        cfg.cg_to_rear = 1.45;
        cfg.cg_height = 0.44;
        cfg.max_engine_force = 11000.0;
        cfg.top_speed_mps = 78.0;
        cfg.air_drag_coefficient = 0.46; // Higher pickup drag
        cfg.chassis = ChassisSkeleton::new(0.96, 1.35, 2.00, -0.40, 0.30, 0.65, 0.70, 0.05);
        cfg.suspension = SuspensionConfig::for_archetype(SuspensionArchetype::SolidLiveAxle);
        cfg.engine_placement = EnginePlacement::FrontEngine;
        for w in &mut cfg.wheels {
            w.tire_radius = 0.36;
            w.tire_width = 0.30;
            w.rotational_inertia = 1.65;
        }
        cfg.set_compound(CompoundId::HardSlick);
        cfg.finalized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_presets() {
        let sports = CarConfig::sports_car();
        let drift = CarConfig::drift_car();
        let kart = CarConfig::kart();
        let rally = CarConfig::rally_car();
        let stock = CarConfig::stock_car_ta1();
        let sand = CarConfig::sand_rail();

        assert_eq!(sports.drive_bias, 0.0);
        assert_eq!(rally.drive_bias, 0.5);
        assert!(drift.max_steer_angle > sports.max_steer_angle);
        assert!(kart.mass < sports.mass);

        assert_eq!(stock.drive_bias, 0.0);
        assert!(stock.max_engine_force > 10000.0);
        assert!(stock.top_speed_mps * 3.6 > 310.0);
        assert!(stock.mass > sports.mass);
        assert_eq!(CarConfig::trans_am_ta1(), stock);

        // Sand rail verification
        assert_eq!(sand.drive_bias, 0.0);
        assert_eq!(sand.mass, 590.0);
        assert_eq!(sand.max_engine_force, 8800.0);
        assert!(sand.top_speed_mps * 3.6 > 195.0);
        assert!(sand.track_width > sports.track_width);

        // Engine placement verification (Spec 078)
        assert_eq!(sports.engine_placement, EnginePlacement::FrontEngine);
        assert_eq!(drift.engine_placement, EnginePlacement::FrontEngine);
        assert_eq!(kart.engine_placement, EnginePlacement::MidEngine);
        assert_eq!(rally.engine_placement, EnginePlacement::FrontEngine);
        assert_eq!(stock.engine_placement, EnginePlacement::FrontEngine);
        assert_eq!(sand.engine_placement, EnginePlacement::RearEngine);

        // All presets must have 4 populated wheels
        assert_eq!(sports.wheels.len(), 4);
        assert_eq!(drift.wheels.len(), 4);
        assert_eq!(kart.wheels.len(), 4);
        assert_eq!(rally.wheels.len(), 4);
        assert_eq!(stock.wheels.len(), 4);
        assert_eq!(sand.wheels.len(), 4);
    }

    #[test]
    fn test_suspension_archetype_factors_and_engine_placement() {
        assert!((SuspensionArchetype::PushrodInboard.robustness_factor() - 0.45).abs() < 1e-6);
        assert!((SuspensionArchetype::PushrodInboard.part_cost_multiplier() - 2.20).abs() < 1e-6);

        assert!((SuspensionArchetype::RigidKart.robustness_factor() - 0.55).abs() < 1e-6);
        assert!((SuspensionArchetype::RigidKart.part_cost_multiplier() - 0.50).abs() < 1e-6);

        assert!((SuspensionArchetype::MacPhersonStrut.robustness_factor() - 0.75).abs() < 1e-6);
        assert!((SuspensionArchetype::MacPhersonStrut.part_cost_multiplier() - 0.80).abs() < 1e-6);

        assert!((SuspensionArchetype::DoubleWishbone.robustness_factor() - 1.00).abs() < 1e-6);
        assert!((SuspensionArchetype::DoubleWishbone.part_cost_multiplier() - 1.20).abs() < 1e-6);

        assert!((SuspensionArchetype::SolidLiveAxle.robustness_factor() - 1.45).abs() < 1e-6);
        assert!((SuspensionArchetype::SolidLiveAxle.part_cost_multiplier() - 0.65).abs() < 1e-6);

        assert!((SuspensionArchetype::LongTravelOffRoad.robustness_factor() - 2.20).abs() < 1e-6);
        assert!((SuspensionArchetype::LongTravelOffRoad.part_cost_multiplier() - 1.10).abs() < 1e-6);

        assert!((EnginePlacement::FrontEngine.repair_cost_multiplier() - 1.00).abs() < 1e-6);
        assert!((EnginePlacement::MidEngine.repair_cost_multiplier() - 1.35).abs() < 1e-6);
        assert!((EnginePlacement::RearEngine.repair_cost_multiplier() - 1.50).abs() < 1e-6);
    }

    #[test]
    fn test_engine_placement_serde_compatibility() {
        // Deserializing without engine_placement defaults gracefully
        let json_without = r#"{
            "mass": 1000.0,
            "inertia": 1200.0,
            "wheelbase": 2.5,
            "track_width": 1.5,
            "cg_to_front": 1.2,
            "cg_to_rear": 1.3,
            "cg_height": 0.3,
            "max_engine_force": 6000.0,
            "max_reverse_force": 3000.0,
            "max_brake_force": 10000.0,
            "handbrake_force": 5000.0,
            "brake_bias": 0.6,
            "drive_bias": 0.0,
            "top_speed_mps": 50.0,
            "max_steer_angle": 0.6,
            "steer_speed": 5.0,
            "steer_return_speed": 6.0,
            "counter_steer_assist": 1.0,
            "air_drag_coefficient": 0.4,
            "lateral_drag_coefficient": 1.0,
            "rolling_resistance_coefficient": 0.015,
            "angular_damping": 100.0,
            "engine_braking_coefficient": 0.1,
            "downforce_coefficient": 0.5,
            "tire": {
                "grip": 1.0,
                "peak_slip_angle_deg": 8.0,
                "slide_grip": 0.8,
                "skid_threshold": 0.1,
                "skid_full_threshold": 0.3
            },
            "assists": {
                "tcs_enabled": false,
                "tcs_slip_threshold": 0.2,
                "tcs_strength": 0.5,
                "tcs_slip_angle_deg": 10.0,
                "esc_sideslip_limit_deg": 10.0,
                "tcs_drift_bypass": false,
                "esc_enabled": false,
                "esc_yaw_threshold": 0.3,
                "esc_strength": 0.5,
                "counter_steer_assist_enabled": false,
                "counter_steer_assist_strength": 0.5,
                "abs_enabled": false,
                "abs_slip_threshold": 0.2,
                "abs_strength": 0.5,
                "handbrake_bypass": true
            }
        }"#;

        let cfg: CarConfig = serde_json::from_str(json_without).unwrap();
        assert_eq!(cfg.engine_placement, EnginePlacement::FrontEngine);

        // Deserializing with explicit rear_engine
        let json_with_rear = json_without.replace(
            "\"mass\": 1000.0,",
            "\"engine_placement\": \"rear_engine\",\n            \"mass\": 1000.0,",
        );
        let cfg_rear: CarConfig = serde_json::from_str(&json_with_rear).unwrap();
        assert_eq!(cfg_rear.engine_placement, EnginePlacement::RearEngine);
    }

    #[test]
    fn test_staggered_kart_wheel_assemblies() {
        let kart = CarConfig::classic_kart();
        // Front narrow tires: r = 0.18, w = 0.12
        assert_eq!(kart.wheels[0].tire_radius, 0.18);
        assert_eq!(kart.wheels[0].tire_width, 0.12);
        assert_eq!(kart.wheels[1].tire_radius, 0.18);
        assert_eq!(kart.wheels[1].tire_width, 0.12);

        // Rear wide tires: r = 0.20, w = 0.21
        assert_eq!(kart.wheels[2].tire_radius, 0.20);
        assert_eq!(kart.wheels[2].tire_width, 0.21);
        assert_eq!(kart.wheels[3].tire_radius, 0.20);
        assert_eq!(kart.wheels[3].tire_width, 0.21);

        // Rear rotational inertia is greater than front: I_rear > I_front
        assert!(kart.wheels[2].rotational_inertia > kart.wheels[0].rotational_inertia);

        // Wide rear slicks carry more grip than the narrow fronts (Spec 043: grip is a true mu scale)
        let f_front = kart.wheels[0].tire_model.grip;
        let f_rear = kart.wheels[2].tire_model.grip;
        assert!(
            f_rear > f_front,
            "rear grip ({}) should exceed front grip ({})",
            f_rear,
            f_front
        );
    }

    #[test]
    fn test_finalize_derives_wheels_from_axle_settings() {
        let mut cfg = CarConfig::sports_car();
        cfg.tire.grip = 1.3;
        cfg.drive_bias = 1.0;
        cfg.brake_bias = 0.70;
        cfg.finalize();
        for i in 0..4 {
            assert_eq!(cfg.wheels[i].tire_model.grip, 1.3);
        }
        assert!((cfg.wheels[0].drive_torque_factor - 0.5).abs() < 1e-6);
        assert_eq!(cfg.wheels[2].drive_torque_factor, 0.0);
        assert!((cfg.wheels[0].brake_bias_factor - 0.35).abs() < 1e-6);
        assert!((cfg.wheels[3].brake_bias_factor - 0.15).abs() < 1e-6);

        // Rear override (staggered kart)
        let kart = CarConfig::kart();
        assert_eq!(kart.wheels[0].tire_model, kart.tire);
        assert_eq!(kart.wheels[2].tire_model, kart.rear_axle.apply(&kart.tire));
        assert!(kart.wheels[2].tire_model.grip > kart.wheels[0].tire_model.grip);

        // A later tire change reaches both axles (the catalog grip stat does this)
        let mut sports = CarConfig::sports_car();
        sports.tire.grip = 1.3;
        sports.finalize();
        assert!((sports.wheels[2].tire_model.grip - 1.3).abs() < 1e-6);
        assert!(
            sports.wheels[2].tire_model.peak_slip_angle_deg
                < sports.wheels[0].tire_model.peak_slip_angle_deg
        );

        // Idempotent
        let again = cfg.finalized();
        assert_eq!(again, cfg);
    }

    #[test]
    fn test_pacejka_peak_slip_angle_migration() {
        // Default legacy sports tire (B=9.5, C=1.45, E=-0.15) peaks at ~10.7 deg
        let peak = pacejka_peak_slip_angle_deg(9.5, 1.45, -0.15);
        assert!((peak - 10.72).abs() < 0.05, "peak = {peak}");
        // Stiffer tire peaks earlier
        assert!(pacejka_peak_slip_angle_deg(13.5, 1.45, -0.15) < peak);
    }

    #[test]
    fn test_legacy_config_deserialization_backward_compatibility() {
        // Legacy JSON without `wheels` array
        let legacy_json = r#"{
            "mass": 1050.0,
            "inertia": 1450.0,
            "wheelbase": 2.40,
            "track_width": 1.40,
            "cg_to_front": 1.10,
            "cg_to_rear": 1.30,
            "cg_height": 0.35,
            "max_engine_force": 6800.0,
            "max_reverse_force": 4420.0,
            "max_brake_force": 11500.0,
            "handbrake_force": 7500.0,
            "brake_bias": 0.60,
            "drive_bias": 0.0,
            "top_speed_mps": 58.0,
            "max_steer_angle": 0.68,
            "steer_speed": 5.5,
            "steer_return_speed": 7.0,
            "counter_steer_assist": 1.3,
            "speed_sensitive_steer_factor": 0.002,
            "air_drag_coefficient": 0.42,
            "lateral_drag_coefficient": 1.20,
            "rolling_resistance_coefficient": 0.015,
            "angular_damping": 160.0,
            "weight_transfer_longitudinal": 1.0,
            "weight_transfer_lateral": 1.0,
            "engine_braking_coefficient": 0.12,
            "downforce_coefficient": 0.65,
            "tire": {
                "stiffness_b": 15.0,
                "shape_c": 1.45,
                "peak_d": 1.10,
                "curvature_e": -0.15,
                "drift_slide_friction": 0.88,
                "handbrake_lateral_friction_multiplier": 0.38,
                "skid_threshold": 0.10,
                "skid_full_threshold": 0.35
            },
            "assists": {
                "tcs_enabled": true,
                "tcs_slip_threshold": 0.18,
                "tcs_strength": 0.75,
                "esc_enabled": true,
                "esc_yaw_threshold": 0.10,
                "esc_strength": 0.85,
                "counter_steer_assist_enabled": true,
                "counter_steer_assist_strength": 0.70,
                "abs_enabled": true,
                "abs_slip_threshold": 0.15,
                "abs_strength": 0.95,
                "handbrake_bypass": true
            },
            "terrain": {
                "sand_flotation": 1.0,
                "mud_flotation": 1.0,
                "ice_grip_multiplier": 1.0
            }
        }"#;

        let deserialized: Result<CarConfig, _> = serde_json::from_str(legacy_json);
        assert!(
            deserialized.is_ok(),
            "Failed to deserialize legacy config: {:?}",
            deserialized.err()
        );
        let config = deserialized.unwrap();

        // All 4 wheels inherit the migrated tire: peak_d -> grip, Pacejka B/C/E -> peak slip angle
        let expected_peak = pacejka_peak_slip_angle_deg(15.0, 1.45, -0.15);
        for i in 0..4 {
            assert_eq!(config.wheels[i].tire_model.grip, 1.10);
            // Rear axle: default stiffer rear (peak at 0.85x the front)
            let axle = if i < 2 {
                1.0
            } else {
                RearAxleTire::default().peak_slip_scale
            };
            assert!(
                (config.wheels[i].tire_model.peak_slip_angle_deg - expected_peak * axle).abs()
                    < 1e-4
            );
            assert_eq!(config.wheels[i].tire_model.slide_grip, 0.88);
            assert_eq!(config.wheels[i].tire_radius, 0.32);
            assert_eq!(config.wheels[i].rotational_inertia, 1.25);
        }

        // Check brake bias distribution across axles (0.60 brake_bias -> 0.30 front, 0.20 rear)
        assert!((config.wheels[0].brake_bias_factor - 0.30).abs() < 1e-4);
        assert!((config.wheels[1].brake_bias_factor - 0.30).abs() < 1e-4);
        assert!((config.wheels[2].brake_bias_factor - 0.20).abs() < 1e-4);
        assert!((config.wheels[3].brake_bias_factor - 0.20).abs() < 1e-4);

        // Check drive bias (0.0 RWD -> 0.0 front, 0.50 rear)
        assert_eq!(config.wheels[0].drive_torque_factor, 0.0);
        assert_eq!(config.wheels[1].drive_torque_factor, 0.0);
        assert_eq!(config.wheels[2].drive_torque_factor, 0.50);
        assert_eq!(config.wheels[3].drive_torque_factor, 0.50);
    }

    #[test]
    fn test_wheel_assembly_inertia_derivation() {
        let r = 0.35;
        let w = 0.28;
        let m = 22.0; // 22 kg
        let compound = TireCompoundConfig::from_id(CompoundId::SoftSlick);
        let assembly = WheelAssemblyConfig::from_corner_mass(r, w, m, compound, 0.30, 0.50);

        let expected_i = 0.5 * m * r * r;
        assert!((assembly.rotational_inertia - expected_i).abs() < 1e-6);
        assert_eq!(assembly.compound.id, CompoundId::SoftSlick);
        assert_eq!(assembly.tire_radius, r);
        assert_eq!(assembly.tire_width, w);
        assert_eq!(assembly.brake_bias_factor, 0.30);
        assert_eq!(assembly.drive_torque_factor, 0.50);
    }
}
