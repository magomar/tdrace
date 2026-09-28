//! Digital keyboard handling settings (Spec 042).
//!
//! Five player parameters shape how keyboard input reaches the car. The filter owns the
//! *timing* (steering speed, center precision, pedal speed); steering authority and traction
//! help are car-side aids that the game applies to the player's car. Speed sensitivity is gone:
//! the car's grip-aware steering already maps full input to the useful angle at any speed.

use serde::{Deserialize, Serialize};

/// Calibrated keyboard handling presets. `Custom` is selected when a slider is edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SteeringProfile {
    /// Relaxed: slow steering, short of the grip limit, strong traction help.
    Smooth,
    /// Default: on the grip limit, moderate steering speed and traction help.
    Balanced,
    /// Quick steering, a little past the limit, light traction help. (Pre-042 "agile"/"direct".)
    #[serde(alias = "agile", alias = "direct")]
    Sharp,
    /// Instant keys, well past the limit, no traction help.
    Raw,
    /// Player-edited values.
    Custom,
}

impl Default for SteeringProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

impl std::fmt::Display for SteeringProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Smooth => write!(f, "Smooth"),
            Self::Balanced => write!(f, "Balanced"),
            Self::Sharp => write!(f, "Sharp"),
            Self::Raw => write!(f, "Raw"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

impl SteeringProfile {
    /// The calibrated presets in display order (`Custom` excluded).
    pub const PRESETS: [Self; 4] = [Self::Smooth, Self::Balanced, Self::Sharp, Self::Raw];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Smooth => "Smooth (Relaxed)",
            Self::Balanced => "Balanced (Default)",
            Self::Sharp => "Sharp (Quick)",
            Self::Raw => "Raw (Instant)",
            Self::Custom => "Custom",
        }
    }

    /// Next preset in the in-race cycle. `Custom` cycles to `Smooth`.
    pub fn cycle(&self) -> Self {
        match self {
            Self::Smooth => Self::Balanced,
            Self::Balanced => Self::Sharp,
            Self::Sharp => Self::Raw,
            Self::Raw | Self::Custom => Self::Smooth,
        }
    }

    pub fn to_config(&self) -> DigitalInputConfig {
        DigitalInputConfig::from_profile(*self)
    }

    /// Dropdown index: presets in display order, then `Custom`.
    pub fn to_index(&self) -> usize {
        match self {
            Self::Smooth => 0,
            Self::Balanced => 1,
            Self::Sharp => 2,
            Self::Raw => 3,
            Self::Custom => 4,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Smooth,
            2 => Self::Sharp,
            3 => Self::Raw,
            4 => Self::Custom,
            _ => Self::Balanced,
        }
    }
}

/// The five keyboard handling parameters (Spec 042).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DigitalInputConfig {
    /// Preset these values came from (`Custom` after a slider edit).
    pub profile: SteeringProfile,
    /// Steering Speed: time for a held key to go from center to full input, in ms (30-300).
    /// Centering takes 0.7x this time.
    pub steer_time_ms: f32,
    /// Steering Authority: where full input sits relative to the front grip limit (0.80-1.40).
    /// Applied car-side as `PlayerHandling::steer_overslip`.
    pub steer_authority: f32,
    /// Center Precision: response curve exponent (1.0 = linear, 1.8 = very soft near center).
    pub center_precision: f32,
    /// Pedal Speed: time for throttle and brake to reach full, in ms (0 = instant, max 300).
    pub pedal_time_ms: f32,
    /// Traction Help [0, 1]: eases throttle near the rear grip limit. Applied car-side.
    pub traction_help: f32,
}

impl DigitalInputConfig {
    pub const STEER_TIME_RANGE_MS: (f32, f32) = (30.0, 300.0);
    pub const AUTHORITY_RANGE: (f32, f32) = (0.80, 1.40);
    pub const CENTER_PRECISION_RANGE: (f32, f32) = (1.0, 1.8);
    pub const PEDAL_TIME_RANGE_MS: (f32, f32) = (0.0, 300.0);
    /// Centering time as a fraction of the steering time.
    pub const RETURN_TIME_FRACTION: f32 = 0.7;

    pub fn from_profile(profile: SteeringProfile) -> Self {
        let (steer_time_ms, steer_authority, center_precision, pedal_time_ms, traction_help) = match profile {
            SteeringProfile::Smooth => (220.0, 0.90, 1.5, 220.0, 0.9),
            SteeringProfile::Balanced | SteeringProfile::Custom => (140.0, 1.00, 1.3, 140.0, 0.7),
            SteeringProfile::Sharp => (90.0, 1.07, 1.1, 80.0, 0.35),
            SteeringProfile::Raw => (40.0, 1.15, 1.0, 0.0, 0.0),
        };
        Self {
            profile,
            steer_time_ms,
            steer_authority,
            center_precision,
            pedal_time_ms,
            traction_help,
        }
    }

    /// Applies a preset's five values. `Custom` keeps the current values.
    pub fn set_profile(&mut self, profile: SteeringProfile) {
        if profile != SteeringProfile::Custom {
            *self = Self::from_profile(profile);
        }
        self.profile = profile;
    }

    /// Returns the preset whose five values match these, or `Custom`.
    pub fn matching_profile(&self) -> SteeringProfile {
        SteeringProfile::PRESETS
            .into_iter()
            .find(|p| Self::from_profile(*p).same_values(self))
            .unwrap_or(SteeringProfile::Custom)
    }

    fn same_values(&self, other: &Self) -> bool {
        let eq = |a: f32, b: f32| (a - b).abs() < 1e-3;
        eq(self.steer_time_ms, other.steer_time_ms)
            && eq(self.steer_authority, other.steer_authority)
            && eq(self.center_precision, other.center_precision)
            && eq(self.pedal_time_ms, other.pedal_time_ms)
            && eq(self.traction_help, other.traction_help)
    }

    /// Clamps every value into its UI range.
    pub fn clamped(mut self) -> Self {
        self.steer_time_ms = self.steer_time_ms.clamp(Self::STEER_TIME_RANGE_MS.0, Self::STEER_TIME_RANGE_MS.1);
        self.steer_authority = self.steer_authority.clamp(Self::AUTHORITY_RANGE.0, Self::AUTHORITY_RANGE.1);
        self.center_precision = self
            .center_precision
            .clamp(Self::CENTER_PRECISION_RANGE.0, Self::CENTER_PRECISION_RANGE.1);
        self.pedal_time_ms = self.pedal_time_ms.clamp(Self::PEDAL_TIME_RANGE_MS.0, Self::PEDAL_TIME_RANGE_MS.1);
        self.traction_help = self.traction_help.clamp(0.0, 1.0);
        self
    }

    /// Steering rise rate in input units per second.
    pub fn steer_rise_rate(&self) -> f32 {
        1000.0 / self.steer_time_ms.max(1.0)
    }

    /// Steering centering rate in input units per second.
    pub fn steer_return_rate(&self) -> f32 {
        self.steer_rise_rate() / Self::RETURN_TIME_FRACTION
    }

    /// Throttle / brake rise rate in input units per second (`None` = instant).
    pub fn pedal_rise_rate(&self) -> Option<f32> {
        if self.pedal_time_ms <= 0.5 {
            None
        } else {
            Some(1000.0 / self.pedal_time_ms)
        }
    }
}

impl Default for DigitalInputConfig {
    fn default() -> Self {
        Self::from_profile(SteeringProfile::Balanced)
    }
}

/// State container for digital keyboard input smoothing.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DigitalInputFilter {
    pub config: DigitalInputConfig,
    /// Smoothed raw steering position [-1.0, 1.0].
    pub current_steer: f32,
    /// Smoothed throttle [0.0, 1.0].
    pub current_throttle: f32,
    /// Smoothed brake [0.0, 1.0].
    pub current_brake: f32,
}

impl DigitalInputFilter {
    pub fn new(config: DigitalInputConfig) -> Self {
        Self {
            config,
            current_steer: 0.0,
            current_throttle: 0.0,
            current_brake: 0.0,
        }
    }

    /// Resets all filtered states to zero.
    pub fn reset(&mut self) {
        self.current_steer = 0.0;
        self.current_throttle = 0.0;
        self.current_brake = 0.0;
    }

    /// Filters raw digital inputs over timestep `dt`.
    /// Returns `(smoothed_steer, smoothed_throttle, smoothed_brake)`.
    pub fn update(&mut self, target_steer: f32, target_throttle: f32, target_brake: f32, dt: f32) -> (f32, f32, f32) {
        let dt = dt.max(1e-5);

        // 1. Steering: rise at the steering speed, center (or reverse) faster
        let is_centering = target_steer == 0.0
            || (target_steer.signum() != self.current_steer.signum() && self.current_steer.abs() > 0.02);
        let steer_rate = if is_centering {
            self.config.steer_return_rate()
        } else {
            self.config.steer_rise_rate()
        };
        let steer_step = steer_rate * dt;
        if self.current_steer < target_steer {
            self.current_steer = (self.current_steer + steer_step).min(target_steer);
        } else if self.current_steer > target_steer {
            self.current_steer = (self.current_steer - steer_step).max(target_steer);
        }

        // 2. Center precision curve
        let steer_abs = self.current_steer.abs();
        let final_steer =
            (self.current_steer.signum() * steer_abs.powf(self.config.center_precision.max(1.0))).clamp(-1.0, 1.0);

        // 3. Pedals: ramp up at the pedal speed, release instantly
        let pedal_rate = self.config.pedal_rise_rate();
        let ramp = |current: f32, target: f32| match pedal_rate {
            Some(rate) if current < target => (current + rate * dt).min(target),
            _ => target,
        };
        self.current_throttle = ramp(self.current_throttle, target_throttle);
        self.current_brake = ramp(self.current_brake, target_brake);

        (final_steer, self.current_throttle, self.current_brake)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_are_ordered_and_distinct() {
        let cfgs = SteeringProfile::PRESETS.map(DigitalInputConfig::from_profile);
        for pair in cfgs.windows(2) {
            assert!(pair[1].steer_time_ms < pair[0].steer_time_ms);
            assert!(pair[1].steer_authority > pair[0].steer_authority);
            assert!(pair[1].center_precision < pair[0].center_precision);
            assert!(pair[1].pedal_time_ms < pair[0].pedal_time_ms);
            assert!(pair[1].traction_help < pair[0].traction_help);
        }
        for p in SteeringProfile::PRESETS {
            assert_eq!(DigitalInputConfig::from_profile(p).matching_profile(), p);
        }
    }

    #[test]
    fn test_slider_edit_becomes_custom() {
        let mut cfg = DigitalInputConfig::from_profile(SteeringProfile::Balanced);
        cfg.steer_authority = 1.05;
        assert_eq!(cfg.matching_profile(), SteeringProfile::Custom);
    }

    #[test]
    fn test_legacy_profile_names_deserialize() {
        let agile: SteeringProfile = serde_json::from_str("\"agile\"").unwrap();
        let direct: SteeringProfile = serde_json::from_str("\"direct\"").unwrap();
        let raw: SteeringProfile = serde_json::from_str("\"raw\"").unwrap();
        assert_eq!(agile, SteeringProfile::Sharp);
        assert_eq!(direct, SteeringProfile::Sharp);
        assert_eq!(raw, SteeringProfile::Raw);
    }

    #[test]
    fn test_steering_reaches_full_in_steer_time() {
        let cfg = DigitalInputConfig { center_precision: 1.0, ..DigitalInputConfig::from_profile(SteeringProfile::Balanced) };
        let mut filter = DigitalInputFilter::new(cfg);
        let dt = 1.0 / 1000.0;
        let mut ms = 0;
        while filter.current_steer < 1.0 && ms < 1000 {
            filter.update(1.0, 0.0, 0.0, dt);
            ms += 1;
        }
        assert!((ms as f32 - cfg.steer_time_ms).abs() <= 2.0, "reached full in {ms} ms");
    }

    #[test]
    fn test_raw_pedals_are_instant() {
        let mut filter = DigitalInputFilter::new(DigitalInputConfig::from_profile(SteeringProfile::Raw));
        let (_, throttle, brake) = filter.update(0.0, 1.0, 1.0, 1.0 / 120.0);
        assert_eq!(throttle, 1.0);
        assert_eq!(brake, 1.0);
    }
}
