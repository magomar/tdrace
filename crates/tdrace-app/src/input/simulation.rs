//! # Deterministic Keyboard Input Dynamics & Car Control Simulation Harness
//!
//! Evaluates how digital keyboard inputs impact vehicle handling, cornering authority,
//! tire scrub drag, yaw stability, and transition dynamics across prototypical driving profiles
//! and surfaces.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use cabinet::input::filter::{DigitalInputConfig, DigitalInputFilter, SteeringProfile};
use tdrace_core::physics::car::{normalize_angle, CarControls};
use tdrace_core::physics::config::{CarConfig, PlayerHandling};
use tdrace_core::physics::sim::SimulationRunner;
use tdrace_core::physics::surface::SurfaceType;

const G_ACCEL: f32 = 9.80665;

/// Prototypical keyboard steering interaction patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyboardSteerPattern {
    /// Driver holds turn key continuously throughout maneuver ("Lead Finger / Hold Lock").
    SustainedHold,
    /// Driver rapidly taps turn key with short pulses (75ms ON / 75ms OFF) to tease traction threshold ("Micro-Feathering").
    RapidFeathering,
    /// Driver pulses turn key with medium rhythm (180ms ON / 120ms OFF) ("Cadence Tapping").
    CadencePulse,
    /// Scandinavian flick / entry pulse (250ms ON, 200ms coast, then maintenance taps) ("Tap-and-Coast").
    TapAndCoast,
    /// Direction reversal from full right (+1.0) to full left (-1.0) ("Snap Countersteer / Chicane").
    SnapCountersteer,
    /// Sustained turn key hold with throttle lift-off during entry to transfer weight forward ("Lift-Off Turn").
    LiftOffTurn,
}

impl KeyboardSteerPattern {
    /// Styles driven through the sweeper corner (Spec 043 key style matrix).
    pub const SWEEPER: [Self; 5] = [
        Self::SustainedHold,
        Self::RapidFeathering,
        Self::CadencePulse,
        Self::TapAndCoast,
        Self::LiftOffTurn,
    ];
    /// Styles driven through the chicane (the reversal itself is scripted by the scenario).
    pub const CHICANE: [Self; 3] = [Self::SustainedHold, Self::RapidFeathering, Self::CadencePulse];

    /// Short stable id used in driver profile ids and reports (matches the pre-043 report ids).
    pub fn id(&self) -> &'static str {
        match self {
            Self::SustainedHold => "hold",
            Self::RapidFeathering => "feathering",
            Self::CadencePulse => "cadence",
            Self::TapAndCoast => "tap_coast",
            Self::SnapCountersteer => "snap",
            Self::LiftOffTurn => "lift_off",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::SustainedHold => "Sustained Hold (Full Lock)",
            Self::RapidFeathering => "Rapid Feathering (Staccato Taps)",
            Self::CadencePulse => "Cadence Pulse (Medium Taps)",
            Self::TapAndCoast => "Tap-and-Coast (Flick Entry)",
            Self::SnapCountersteer => "Snap Countersteer (Direction Reversal)",
            Self::LiftOffTurn => "Lift-Off Turn-In (Weight Transfer)",
        }
    }
}

/// Complete keyboard driver profile combining an input pattern with a steering filter preset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardDriverProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steer_pattern: KeyboardSteerPattern,
    pub filter_profile: SteeringProfile,
    pub custom_filter_config: Option<DigitalInputConfig>,
}

impl KeyboardDriverProfile {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        steer_pattern: KeyboardSteerPattern,
        filter_profile: SteeringProfile,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            steer_pattern,
            filter_profile,
            custom_filter_config: None,
        }
    }

    /// Sustained hold with default Balanced filter profile.
    pub fn sustained_hold_balanced() -> Self {
        Self::new(
            "hold_balanced",
            "Sustained Hold (Balanced)",
            "Continuous key press on the default Balanced preset",
            KeyboardSteerPattern::SustainedHold,
            SteeringProfile::Balanced,
        )
    }

    /// Sustained hold with the quick Sharp preset (pre-043 "Direct").
    pub fn sustained_hold_sharp() -> Self {
        Self::new(
            "hold_sharp",
            "Sustained Hold (Sharp)",
            "Continuous key press on the quick Sharp preset",
            KeyboardSteerPattern::SustainedHold,
            SteeringProfile::Sharp,
        )
    }

    /// Sustained hold with Smooth (arcade damped) filter profile.
    pub fn sustained_hold_smooth() -> Self {
        Self::new(
            "hold_smooth",
            "Sustained Hold (Smooth Arcade)",
            "Continuous key press on the relaxed Smooth preset",
            KeyboardSteerPattern::SustainedHold,
            SteeringProfile::Smooth,
        )
    }

    /// Rapid staccato feathering (75ms ON / 75ms OFF) on Balanced filter.
    pub fn rapid_feathering_balanced() -> Self {
        Self::new(
            "feathering_balanced",
            "Rapid Feathering (Balanced)",
            "High-frequency 6.67 Hz staccato taps to modulate slip angle and mitigate scrub",
            KeyboardSteerPattern::RapidFeathering,
            SteeringProfile::Balanced,
        )
    }

    /// Cadence rhythmic pulsing (180ms ON / 120ms OFF) on Balanced filter.
    pub fn cadence_pulse_balanced() -> Self {
        Self::new(
            "cadence_balanced",
            "Cadence Pulse (Balanced)",
            "Rhythmic 3.33 Hz medium-duration pulses for controlled cornering",
            KeyboardSteerPattern::CadencePulse,
            SteeringProfile::Balanced,
        )
    }

    /// Scandinavian flick / tap and coast on Balanced filter.
    pub fn tap_and_coast_balanced() -> Self {
        Self::new(
            "tap_coast_balanced",
            "Tap-and-Coast (Balanced)",
            "Initial entry pulse followed by coasting and gentle maintenance taps",
            KeyboardSteerPattern::TapAndCoast,
            SteeringProfile::Balanced,
        )
    }

    /// Throttle lift-off corner entry on Balanced filter.
    pub fn lift_off_turn_balanced() -> Self {
        Self::new(
            "lift_off_balanced",
            "Lift-Off Turn (Balanced)",
            "Sustained turn key with throttle lift during entry to induce forward load transfer",
            KeyboardSteerPattern::LiftOffTurn,
            SteeringProfile::Balanced,
        )
    }

    /// Snap countersteer reversal on Balanced filter.
    pub fn snap_countersteer_balanced() -> Self {
        Self::new(
            "snap_countersteer_balanced",
            "Snap Countersteer (Balanced)",
            "Full lock reversal (+1.0 to -1.0) testing transition latency and pendulum overshoot",
            KeyboardSteerPattern::SnapCountersteer,
            SteeringProfile::Balanced,
        )
    }

    /// A driver pressing keys in `pattern` on handling preset `profile`.
    /// Id: `{pattern id}_{preset}`, e.g. `hold_balanced`.
    pub fn for_pattern(pattern: KeyboardSteerPattern, profile: SteeringProfile) -> Self {
        Self::new(
            format!("{}_{}", pattern.id(), profile.to_string().to_lowercase()),
            format!("{} ({})", pattern.name(), profile),
            format!("{} on the {} handling preset", pattern.name(), profile),
            pattern,
            profile,
        )
    }

    /// Car-side handling aids this driver's keyboard settings apply (Spec 043).
    pub fn player_handling(&self) -> PlayerHandling {
        let cfg = self.filter_config();
        PlayerHandling::human(cfg.steer_authority, cfg.traction_help)
    }

    pub fn filter_config(&self) -> DigitalInputConfig {
        self.custom_filter_config
            .unwrap_or_else(|| DigitalInputConfig::from_profile(self.filter_profile))
    }

    /// Samples raw binary keyboard states `(raw_steer, raw_throttle, raw_brake)` at simulation time `t`.
    pub fn sample_raw_inputs(&self, t: f32, _scenario_duration: f32) -> (f32, f32, f32) {
        match self.steer_pattern {
            KeyboardSteerPattern::SustainedHold => (1.0, 1.0, 0.0),
            KeyboardSteerPattern::RapidFeathering => {
                let period = 0.150; // 75ms ON / 75ms OFF
                let phase = t % period;
                let steer = if phase < 0.075 { 1.0 } else { 0.0 };
                (steer, 1.0, 0.0)
            }
            KeyboardSteerPattern::CadencePulse => {
                let period = 0.300; // 180ms ON / 120ms OFF
                let phase = t % period;
                let steer = if phase < 0.180 { 1.0 } else { 0.0 };
                (steer, 1.0, 0.0)
            }
            KeyboardSteerPattern::TapAndCoast => {
                let steer = if t < 0.250 {
                    1.0 // Initial sharp turn-in
                } else if t < 0.450 {
                    0.0 // Coast / set slip
                } else {
                    let phase = (t - 0.450) % 0.200;
                    if phase < 0.100 { 1.0 } else { 0.0 } // Maintenance taps
                };
                (steer, 1.0, 0.0)
            }
            KeyboardSteerPattern::SnapCountersteer => {
                let steer = if t < 1.0 {
                    1.0 // Turn right
                } else if t < 2.0 {
                    -1.0 // Snap turn left
                } else {
                    0.0 // Center
                };
                (steer, 1.0, 0.0)
            }
            KeyboardSteerPattern::LiftOffTurn => {
                let steer = 1.0;
                let throttle = if t < 0.600 {
                    0.0 // Lift off throttle during initial turn-in
                } else {
                    1.0 // Power out
                };
                (steer, throttle, 0.0)
            }
        }
    }
}

/// Categorical vehicle handling outcome under keyboard sweeper evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyboardHandlingOutcome {
    /// Optimal high-speed line carving with high speed retention (> 75%) and balanced slip angles.
    CleanCarve,
    /// Excessive front tire scrub drag causing heavy deceleration (< 65% speed retention) and front wash-out.
    ScrubUndersteer,
    /// Controllable dynamic oversteer slide with rear tire slip dominating front tire slip.
    PowerSlide,
    /// Uncontrollable vehicle rotation (yaw rate > 250 deg/s or heading deviation > 120 deg).
    Spinout,
}

impl KeyboardHandlingOutcome {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::CleanCarve => "CLEAN CARVE",
            Self::ScrubUndersteer => "SCRUB UNDERSTEER",
            Self::PowerSlide => "POWER SLIDE",
            Self::Spinout => "SPINOUT",
        }
    }
}

/// Telemetry metrics produced by the high-speed sweeper cornering test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardSweeperResult {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub surface: SurfaceType,
    pub driver_profile_id: String,
    pub driver_profile_name: String,
    pub filter_profile: SteeringProfile,
    #[serde(default = "default_pattern")]
    pub steer_pattern: KeyboardSteerPattern,
    pub entry_speed_kmh: f32,
    pub exit_speed_kmh: f32,
    pub min_speed_kmh: f32,
    pub speed_retention_pct: f32,
    pub speed_loss_kmh: f32,
    pub avg_steer_angle_deg: f32,
    pub peak_steer_angle_deg: f32,
    pub avg_yaw_rate_deg_s: f32,
    pub peak_yaw_rate_deg_s: f32,
    pub avg_lateral_g: f32,
    pub peak_lateral_g: f32,
    pub effective_radius_m: f32,
    pub heading_change_deg: f32,
    pub peak_front_slip_deg: f32,
    pub peak_rear_slip_deg: f32,
    pub understeer_slip_delta_deg: f32,
    pub outcome: KeyboardHandlingOutcome,
}

fn default_pattern() -> KeyboardSteerPattern {
    KeyboardSteerPattern::SustainedHold
}

/// How much the key-pressing style changes the result for one car, surface and preset
/// (Spec 043 Key Style Sensitivity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyStyleSensitivity {
    pub vehicle_id: String,
    pub surface: SurfaceType,
    pub filter_profile: SteeringProfile,
    pub min_exit_kmh: f32,
    pub max_exit_kmh: f32,
    /// (max - min) / max exit speed across styles, in percent.
    pub spread_pct: f32,
    pub fastest_style: KeyboardSteerPattern,
    pub slowest_style: KeyboardSteerPattern,
    /// Sustained Hold exit speed as a percentage of Rapid Feathering exit speed.
    pub hold_vs_feathering_pct: f32,
}

/// Groups sweeper results by (vehicle, surface, preset) and measures the spread across key styles.
pub fn key_style_sensitivity(results: &[KeyboardSweeperResult]) -> Vec<KeyStyleSensitivity> {
    let mut groups: Vec<(String, SurfaceType, SteeringProfile, Vec<&KeyboardSweeperResult>)> = Vec::new();
    for r in results {
        match groups
            .iter_mut()
            .find(|g| g.0 == r.vehicle_id && g.1 == r.surface && g.2 == r.filter_profile)
        {
            Some(g) => g.3.push(r),
            None => groups.push((r.vehicle_id.clone(), r.surface, r.filter_profile, vec![r])),
        }
    }
    groups
        .into_iter()
        .filter(|g| g.3.len() > 1)
        .map(|(vehicle_id, surface, filter_profile, rs)| {
            let fastest = rs.iter().max_by(|a, b| a.exit_speed_kmh.total_cmp(&b.exit_speed_kmh)).unwrap();
            let slowest = rs.iter().min_by(|a, b| a.exit_speed_kmh.total_cmp(&b.exit_speed_kmh)).unwrap();
            let exit_of = |p: KeyboardSteerPattern| rs.iter().find(|r| r.steer_pattern == p).map(|r| r.exit_speed_kmh);
            let hold_vs_feathering_pct = match (
                exit_of(KeyboardSteerPattern::SustainedHold),
                exit_of(KeyboardSteerPattern::RapidFeathering),
            ) {
                (Some(h), Some(f)) if f > 1e-3 => h / f * 100.0,
                _ => 0.0,
            };
            KeyStyleSensitivity {
                vehicle_id,
                surface,
                filter_profile,
                min_exit_kmh: slowest.exit_speed_kmh,
                max_exit_kmh: fastest.exit_speed_kmh,
                spread_pct: (fastest.exit_speed_kmh - slowest.exit_speed_kmh) / fastest.exit_speed_kmh.max(1e-3) * 100.0,
                fastest_style: fastest.steer_pattern,
                slowest_style: slowest.steer_pattern,
                hold_vs_feathering_pct,
            }
        })
        .collect()
}

/// Categorical transition outcome in the S-chicane direction reversal test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChicaneTransitionOutcome {
    /// Crisp, rapid transition with minimal yaw latency (< 260ms) and zero fishtailing.
    CrispTransition,
    /// Stable reversal with mild, damped secondary pendulum oscillation.
    MildDampedPendulum,
    /// Violent rear axle break-away with multi-cycle fishtailing.
    ViolentSnapOversteer,
    /// Total loss of directional control during direction reversal.
    Spinout,
}

impl ChicaneTransitionOutcome {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::CrispTransition => "CRISP TRANSITION",
            Self::MildDampedPendulum => "MILD PENDULUM",
            Self::ViolentSnapOversteer => "SNAP OVERSTEER",
            Self::Spinout => "SPINOUT",
        }
    }
}

/// Telemetry metrics produced by the S-chicane / slalom direction reversal test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardChicaneResult {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub surface: SurfaceType,
    pub driver_profile_id: String,
    pub entry_speed_kmh: f32,
    pub exit_speed_kmh: f32,
    pub reversal_latency_ms: f32,
    pub peak_yaw_overshoot_deg_s: f32,
    pub fishtail_oscillation_count: u32,
    pub max_lateral_displacement_m: f32,
    pub peak_front_slip_deg: f32,
    pub peak_rear_slip_deg: f32,
    pub outcome: ChicaneTransitionOutcome,
}

/// Outcome of countersteer recovery on slippery surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlideCatchOutcome {
    /// Successfully arrested slide and stabilized within 1.5s.
    Recovered,
    /// Arrested slide after prolonged oscillation (1.5s to 3.0s).
    DelayedRecovery,
    /// Vehicle spun out (> 60 deg deviation from recovery line).
    SpunOut,
}

impl SlideCatchOutcome {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Recovered => "RECOVERED",
            Self::DelayedRecovery => "DELAYED RECOVERY",
            Self::SpunOut => "SPUN OUT",
        }
    }
}

/// Telemetry metrics produced by the low-grip slide catch test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardSlideCatchResult {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub surface: SurfaceType,
    pub driver_profile_id: String,
    pub initial_slide_yaw_deg_s: f32,
    pub recovery_time_s: Option<f32>,
    pub max_sideslip_deg: f32,
    pub final_heading_error_deg: f32,
    pub outcome: SlideCatchOutcome,
}

/// Comprehensive vehicle evaluation dataset across sweeper, chicane, and recovery tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleKeyboardMatrix {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub category: String,
    pub mass_kg: f32,
    pub power_bhp: u16,
    pub drivetrain: String,
    pub sweeper_results: Vec<KeyboardSweeperResult>,
    pub chicane_results: Vec<KeyboardChicaneResult>,
    pub catch_results: Vec<KeyboardSlideCatchResult>,
}

// ============================================================================
// Simulation Execution Functions
// ============================================================================

/// Runs a high-speed sweeper cornering test evaluating speed preservation, turning radius,
/// and tire scrub drag under the specified keyboard input profile.
pub fn run_keyboard_sweeper_simulation(
    vehicle_id: &str,
    vehicle_name: &str,
    car_config: &CarConfig,
    surface: SurfaceType,
    driver: &KeyboardDriverProfile,
    v0_kmh: f32,
    duration_s: f32,
    dt: f32,
) -> KeyboardSweeperResult {
    let v0_mps = v0_kmh / 3.6;
    let mut runner = SimulationRunner::new(car_config.clone(), dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    let mut filter = DigitalInputFilter::new(driver.filter_config());
    runner.car.config.player = driver.player_handling();

    let mut min_speed_mps = v0_mps;
    let mut sum_speed = 0.0f32;
    let mut sum_steer_deg = 0.0f32;
    let mut peak_steer_deg = 0.0f32;
    let mut sum_yaw_rate = 0.0f32;
    let mut peak_yaw_rate = 0.0f32;
    let mut sum_lateral_g = 0.0f32;
    let mut peak_lateral_g = 0.0f32;
    let mut peak_front_slip = 0.0f32;
    let mut peak_rear_slip = 0.0f32;
    let mut step_count = 0u32;

    let initial_heading = runner.car.state().angle;

    runner.run_for(duration_s, surface, |t, car| {
        let (raw_steer, raw_throttle, raw_brake) = driver.sample_raw_inputs(t, duration_s);
        let speed_mps = car.state().speed;
        let (steer, throttle, brake) = filter.update(raw_steer, raw_throttle, raw_brake, dt);

        // Update telemetry tracking
        if speed_mps < min_speed_mps {
            min_speed_mps = speed_mps;
        }
        sum_speed += speed_mps;

        let steer_deg = car.state().steer_angle.to_degrees().abs();
        sum_steer_deg += steer_deg;
        if steer_deg > peak_steer_deg {
            peak_steer_deg = steer_deg;
        }

        let yaw_rate_deg_s = car.state().angular_velocity.to_degrees().abs();
        sum_yaw_rate += yaw_rate_deg_s;
        if yaw_rate_deg_s > peak_yaw_rate {
            peak_yaw_rate = yaw_rate_deg_s;
        }

        let lat_g = (speed_mps * car.state().angular_velocity.abs()) / G_ACCEL;
        sum_lateral_g += lat_g;
        if lat_g > peak_lateral_g {
            peak_lateral_g = lat_g;
        }

        let front_slip = ((car.state().wheels[0].slip_angle.abs() + car.state().wheels[1].slip_angle.abs()) * 0.5)
            .to_degrees();
        let rear_slip = ((car.state().wheels[2].slip_angle.abs() + car.state().wheels[3].slip_angle.abs()) * 0.5)
            .to_degrees();

        if front_slip > peak_front_slip {
            peak_front_slip = front_slip;
        }
        if rear_slip > peak_rear_slip {
            peak_rear_slip = rear_slip;
        }

        step_count += 1;

        CarControls {
            throttle,
            steer,
            brake,
            handbrake: false,
            reverse: false,
        }
    });

    let exit_speed_mps = runner.car.state().speed;
    let exit_speed_kmh = exit_speed_mps * 3.6;
    let speed_loss_kmh = (v0_kmh - exit_speed_kmh).max(0.0);
    let speed_retention_pct = if v0_kmh > 1e-2 {
        (exit_speed_kmh / v0_kmh) * 100.0
    } else {
        0.0
    };

    let avg_speed_mps = if step_count > 0 { sum_speed / (step_count as f32) } else { 0.0 };
    let avg_yaw_rate_deg_s = if step_count > 0 { sum_yaw_rate / (step_count as f32) } else { 0.0 };
    let avg_yaw_rate_rad_s = avg_yaw_rate_deg_s.to_radians();
    let effective_radius_m = if avg_yaw_rate_rad_s > 1e-3 {
        avg_speed_mps / avg_yaw_rate_rad_s
    } else {
        999.0
    };

    let heading_change_deg = normalize_angle(runner.car.state().angle - initial_heading)
        .to_degrees()
        .abs();

    let understeer_slip_delta_deg = peak_front_slip - peak_rear_slip;

    // Outcome determination
    let outcome = if peak_yaw_rate > 240.0 || (heading_change_deg > 140.0 && exit_speed_kmh < 15.0) {
        KeyboardHandlingOutcome::Spinout
    } else if understeer_slip_delta_deg > 3.0 && speed_retention_pct < 68.0 {
        KeyboardHandlingOutcome::ScrubUndersteer
    } else if peak_rear_slip > peak_front_slip + 3.0 && peak_rear_slip > 10.0 {
        KeyboardHandlingOutcome::PowerSlide
    } else {
        KeyboardHandlingOutcome::CleanCarve
    };

    KeyboardSweeperResult {
        vehicle_id: vehicle_id.to_string(),
        vehicle_name: vehicle_name.to_string(),
        surface,
        driver_profile_id: driver.id.clone(),
        driver_profile_name: driver.name.clone(),
        filter_profile: driver.filter_profile,
        steer_pattern: driver.steer_pattern,
        entry_speed_kmh: v0_kmh,
        exit_speed_kmh,
        min_speed_kmh: min_speed_mps * 3.6,
        speed_retention_pct,
        speed_loss_kmh,
        avg_steer_angle_deg: if step_count > 0 { sum_steer_deg / (step_count as f32) } else { 0.0 },
        peak_steer_angle_deg: peak_steer_deg,
        avg_yaw_rate_deg_s,
        peak_yaw_rate_deg_s: peak_yaw_rate,
        avg_lateral_g: if step_count > 0 { sum_lateral_g / (step_count as f32) } else { 0.0 },
        peak_lateral_g,
        effective_radius_m,
        heading_change_deg,
        peak_front_slip_deg: peak_front_slip,
        peak_rear_slip_deg: peak_rear_slip,
        understeer_slip_delta_deg,
        outcome,
    }
}

/// Runs an S-chicane transient direction reversal test (Left 1.0s, Snap Right 1.0s, Center 1.0s).
pub fn run_keyboard_chicane_simulation(
    vehicle_id: &str,
    vehicle_name: &str,
    car_config: &CarConfig,
    surface: SurfaceType,
    driver: &KeyboardDriverProfile,
    v0_kmh: f32,
    dt: f32,
) -> KeyboardChicaneResult {
    let v0_mps = v0_kmh / 3.6;
    let mut runner = SimulationRunner::new(car_config.clone(), dt)
        .with_state(Vec2::ZERO, 0.0, Vec2::new(v0_mps, 0.0));

    let mut filter = DigitalInputFilter::new(driver.filter_config());
    runner.car.config.player = driver.player_handling();

    let mut initial_yaw_sign = 0.0f32;
    let mut reversal_latency_ms = 0.0f32;
    let mut measured_reversal = false;
    let mut peak_overshoot_deg_s = 0.0f32;
    let mut fishtail_count = 0u32;
    let mut max_lateral_disp_m = 0.0f32;
    let mut peak_front_slip = 0.0f32;
    let mut peak_rear_slip = 0.0f32;
    let mut prev_yaw_sign = 0.0f32;

    let total_duration = 3.0f32;
    let reversal_time = 1.0f32;

    runner.run_for(total_duration, surface, |t, car| {
        let base_sign = if t < reversal_time {
            1.0f32
        } else if t < 2.0 {
            -1.0f32
        } else {
            0.0f32
        };

        let raw_steer = if base_sign == 0.0 {
            0.0
        } else {
            match driver.steer_pattern {
                KeyboardSteerPattern::RapidFeathering => {
                    let phase = t % 0.150;
                    if phase < 0.075 { base_sign } else { 0.0 }
                }
                KeyboardSteerPattern::CadencePulse => {
                    let phase = t % 0.300;
                    if phase < 0.180 { base_sign } else { 0.0 }
                }
                _ => base_sign,
            }
        };

        let (steer, throttle, brake) = filter.update(raw_steer, 1.0, 0.0, dt);

        let yaw_rate = car.state().angular_velocity;
        let yaw_deg_s = yaw_rate.to_degrees();

        // 1. Establish initial yaw sign during phase 1, then detect true sign reversal
        if t < reversal_time {
            if yaw_deg_s.abs() > 2.0 && initial_yaw_sign == 0.0 {
                initial_yaw_sign = yaw_rate.signum();
            }
        } else if !measured_reversal {
            if initial_yaw_sign != 0.0 && yaw_rate.signum() != initial_yaw_sign && yaw_deg_s.abs() > 1.0 {
                reversal_latency_ms = (t - reversal_time) * 1000.0;
                measured_reversal = true;
            }
        }

        // 2. Measure overshoot after reversal
        if t >= reversal_time && measured_reversal {
            if yaw_deg_s.abs() > peak_overshoot_deg_s {
                peak_overshoot_deg_s = yaw_deg_s.abs();
            }
        }

        // 3. Fishtail oscillation counter (detecting sign flips after t=2.0 centering)
        if t >= 2.0 {
            let current_sign = yaw_rate.signum();
            if prev_yaw_sign != 0.0 && current_sign != prev_yaw_sign && yaw_deg_s.abs() > 4.0 {
                fishtail_count += 1;
            }
            prev_yaw_sign = current_sign;
        }

        let lat_disp = car.state().position.y.abs();
        if lat_disp > max_lateral_disp_m {
            max_lateral_disp_m = lat_disp;
        }

        let front_slip = ((car.state().wheels[0].slip_angle.abs() + car.state().wheels[1].slip_angle.abs()) * 0.5)
            .to_degrees();
        let rear_slip = ((car.state().wheels[2].slip_angle.abs() + car.state().wheels[3].slip_angle.abs()) * 0.5)
            .to_degrees();

        if front_slip > peak_front_slip {
            peak_front_slip = front_slip;
        }
        if rear_slip > peak_rear_slip {
            peak_rear_slip = rear_slip;
        }

        CarControls {
            throttle,
            steer,
            brake,
            handbrake: false,
            reverse: false,
        }
    });

    if !measured_reversal {
        reversal_latency_ms = 2000.0; // Timed out
    }

    let final_heading = normalize_angle(runner.car.state().angle).to_degrees().abs();
    // A timed-out reversal is a spin only if the rear slid: a car that keeps its line because the
    // filtered input is too small to reverse it (Smooth feathering on dirt) is in control.
    let outcome = if final_heading > 80.0 || (reversal_latency_ms > 1200.0 && peak_rear_slip > 10.0) {
        ChicaneTransitionOutcome::Spinout
    } else if fishtail_count > 1 || peak_rear_slip > 22.0 {
        ChicaneTransitionOutcome::ViolentSnapOversteer
    } else if fishtail_count == 1 || reversal_latency_ms > 350.0 {
        ChicaneTransitionOutcome::MildDampedPendulum
    } else {
        ChicaneTransitionOutcome::CrispTransition
    };

    KeyboardChicaneResult {
        vehicle_id: vehicle_id.to_string(),
        vehicle_name: vehicle_name.to_string(),
        surface,
        driver_profile_id: driver.id.clone(),
        entry_speed_kmh: v0_kmh,
        exit_speed_kmh: runner.car.state().speed * 3.6,
        reversal_latency_ms,
        peak_yaw_overshoot_deg_s: peak_overshoot_deg_s,
        fishtail_oscillation_count: fishtail_count,
        max_lateral_displacement_m: max_lateral_disp_m,
        peak_front_slip_deg: peak_front_slip,
        peak_rear_slip_deg: peak_rear_slip,
        outcome,
    }
}

/// Runs a low-grip slide catch test where the vehicle enters an induced yaw slide
/// and the driver attempts recovery via opposite lock.
pub fn run_keyboard_slide_catch_simulation(
    vehicle_id: &str,
    vehicle_name: &str,
    car_config: &CarConfig,
    surface: SurfaceType,
    driver: &KeyboardDriverProfile,
    v0_kmh: f32,
    dt: f32,
) -> KeyboardSlideCatchResult {
    let v0_mps = v0_kmh / 3.6;
    // Initial condition: 15 deg yaw angle perturbation with initial angular rate
    let initial_heading = 15.0f32.to_radians();
    let initial_slide_yaw_deg_s = 35.0f32;

    let mut runner = SimulationRunner::new(car_config.clone(), dt)
        .with_state(Vec2::ZERO, initial_heading, Vec2::new(v0_mps, 0.0));
    runner.car.state_mut().angular_velocity = initial_slide_yaw_deg_s.to_radians();

    let mut filter = DigitalInputFilter::new(driver.filter_config());
    runner.car.config.player = driver.player_handling();

    let mut recovery_time_s = None;
    let mut max_sideslip_deg = 0.0f32;
    let total_duration = 3.0f32;

    runner.run_until(
        total_duration,
        surface,
        |t, car| {
            // Driver applies opposite lock. The induced slide rotates the car left (heading +15 deg,
            // yaw +35 deg/s, counter-clockwise), so the catch is a RIGHT steer (+1.0). The pre-043
            // harness steered -1.0 (into the slide), which is why every hold "spun out".
            let raw_steer = match driver.steer_pattern {
                KeyboardSteerPattern::RapidFeathering => {
                    let phase = t % 0.150;
                    if phase < 0.075 { 1.0 } else { 0.0 }
                }
                _ => 1.0, // Sustained countersteer hold
            };

            let (steer, throttle, brake) = filter.update(raw_steer, 0.8, 0.0, dt);

            let sideslip = car.state().sideslip_angle.to_degrees().abs();
            if sideslip > max_sideslip_deg {
                max_sideslip_deg = sideslip;
            }

            CarControls {
                throttle,
                steer,
                brake,
                handbrake: false,
                reverse: false,
            }
        },
        |t, car| {
            let heading_err = normalize_angle(car.state().angle).to_degrees().abs();
            let yaw_rate = car.state().angular_velocity.to_degrees().abs();

            if t > 0.3 && heading_err < 3.0 && yaw_rate < 5.0 && recovery_time_s.is_none() {
                recovery_time_s = Some(t);
                return true;
            }

            // Terminate if car spun out completely (> 80 deg)
            heading_err > 80.0
        },
    );

    let final_heading_error = normalize_angle(runner.car.state().angle).to_degrees().abs();
    let outcome = match recovery_time_s {
        Some(t) if t <= 1.5 => SlideCatchOutcome::Recovered,
        Some(_) => SlideCatchOutcome::DelayedRecovery,
        None if final_heading_error < 10.0 => SlideCatchOutcome::DelayedRecovery,
        None => SlideCatchOutcome::SpunOut,
    };

    KeyboardSlideCatchResult {
        vehicle_id: vehicle_id.to_string(),
        vehicle_name: vehicle_name.to_string(),
        surface,
        driver_profile_id: driver.id.clone(),
        initial_slide_yaw_deg_s,
        recovery_time_s,
        max_sideslip_deg,
        final_heading_error_deg: final_heading_error,
        outcome,
    }
}
