use serde::{Deserialize, Serialize};

/// Selectable steering smoothing profiles for digital keyboard controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SteeringProfile {
    /// Direct esports response: instantaneous turning with no speed attenuation.
    Direct,
    /// Balanced response: progressive rise, center gamma curve, and hold-lock bleed.
    Balanced,
    /// Smooth arcade response: higher damping and softer center for relaxed driving.
    Smooth,
}

impl Default for SteeringProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

impl std::fmt::Display for SteeringProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "Direct"),
            Self::Balanced => write!(f, "Balanced"),
            Self::Smooth => write!(f, "Smooth"),
        }
    }
}

impl SteeringProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Direct => "Direct (Raw)",
            Self::Balanced => "Balanced (Progressive)",
            Self::Smooth => "Smooth (Arcade)",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            Self::Balanced => Self::Smooth,
            Self::Smooth => Self::Direct,
            Self::Direct => Self::Balanced,
        }
    }
}

fn default_steering_profile() -> SteeringProfile {
    SteeringProfile::Balanced
}

fn default_hold_bleed_rate() -> f32 {
    2.5
}

/// Configuration for digital keyboard input smoothing and progressive steering/turning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DigitalInputConfig {
    /// Active steering profile preset.
    #[serde(default = "default_steering_profile")]
    pub profile: SteeringProfile,
    /// Turn rise rate in units/second (how quickly turning reaches full lock).
    pub steer_rise_rate: f32,
    /// Turn return-to-center rate in units/second when turn keys are released.
    pub steer_return_rate: f32,
    /// Non-linear response exponent (> 1.0 creates a soft zone near center for precision).
    pub steer_exponent: f32,
    /// Dynamic speed-sensitive steering attenuation factor.
    pub speed_sensitive_factor: f32,
    /// Minimum steering limit fraction at top speed.
    pub min_speed_steer_limit: f32,
    /// Rate at which sustained turn-key hold bleeds off speed attenuation towards 1.0 full lock (units/sec).
    #[serde(default = "default_hold_bleed_rate")]
    pub hold_bleed_rate: f32,
    /// Throttle/acceleration rise rate in units/second.
    pub throttle_rise_rate: f32,
    /// Service brake/deceleration rise rate in units/second.
    pub brake_rise_rate: f32,
}

impl DigitalInputConfig {
    pub fn from_profile(profile: SteeringProfile) -> Self {
        match profile {
            SteeringProfile::Direct => Self {
                profile,
                steer_rise_rate: 12.0,
                steer_return_rate: 20.0,
                steer_exponent: 1.0,
                speed_sensitive_factor: 0.0,
                min_speed_steer_limit: 1.0,
                hold_bleed_rate: 0.0,
                throttle_rise_rate: 12.0,
                brake_rise_rate: 10.0,
            },
            SteeringProfile::Balanced => Self {
                profile,
                steer_rise_rate: 6.5,
                steer_return_rate: 13.0,
                steer_exponent: 1.25,
                speed_sensitive_factor: 0.004,
                min_speed_steer_limit: 0.75,
                hold_bleed_rate: 2.5,
                throttle_rise_rate: 9.5,
                brake_rise_rate: 6.5,
            },
            SteeringProfile::Smooth => Self {
                profile,
                steer_rise_rate: 5.0,
                steer_return_rate: 10.0,
                steer_exponent: 1.40,
                speed_sensitive_factor: 0.008,
                min_speed_steer_limit: 0.60,
                hold_bleed_rate: 1.5,
                throttle_rise_rate: 8.0,
                brake_rise_rate: 5.5,
            },
        }
    }

    /// Applies a steering profile to this config, preserving throttle/brake rates.
    pub fn set_profile(&mut self, profile: SteeringProfile) {
        let preset = Self::from_profile(profile);
        self.profile = profile;
        self.steer_rise_rate = preset.steer_rise_rate;
        self.steer_return_rate = preset.steer_return_rate;
        self.steer_exponent = preset.steer_exponent;
        self.speed_sensitive_factor = preset.speed_sensitive_factor;
        self.min_speed_steer_limit = preset.min_speed_steer_limit;
        self.hold_bleed_rate = preset.hold_bleed_rate;
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
    /// Continuous turn hold factor [0.0, 1.0] for progressive lock bleed.
    pub steer_hold_factor: f32,
}

impl DigitalInputFilter {
    pub fn new(config: DigitalInputConfig) -> Self {
        Self {
            config,
            current_steer: 0.0,
            current_throttle: 0.0,
            current_brake: 0.0,
            steer_hold_factor: 0.0,
        }
    }

    /// Resets all filtered states to zero.
    pub fn reset(&mut self) {
        self.current_steer = 0.0;
        self.current_throttle = 0.0;
        self.current_brake = 0.0;
        self.steer_hold_factor = 0.0;
    }

    /// Filters and smooths raw digital inputs over timestep `dt` with forward speed scaling.
    /// Returns `(smoothed_steer, smoothed_throttle, smoothed_brake)`.
    pub fn update(
        &mut self,
        target_steer: f32,
        target_throttle: f32,
        target_brake: f32,
        speed_mps: f32,
        dt: f32,
    ) -> (f32, f32, f32) {
        let dt = dt.max(1e-5);

        // 1. Steering smoothing
        let is_centering = target_steer == 0.0
            || (target_steer.signum() != self.current_steer.signum() && self.current_steer.abs() > 0.02);

        let steer_rate = if is_centering {
            self.config.steer_return_rate
        } else {
            self.config.steer_rise_rate
        };

        let steer_step = steer_rate * dt;
        if self.current_steer < target_steer {
            self.current_steer = (self.current_steer + steer_step).min(target_steer);
        } else if self.current_steer > target_steer {
            self.current_steer = (self.current_steer - steer_step).max(target_steer);
        }

        // 2. Non-linear center curve (gamma)
        let steer_abs = self.current_steer.abs();
        let curved_steer = self.current_steer.signum() * steer_abs.powf(self.config.steer_exponent);

        // 3. Dynamic speed-sensitive scaling with progressive hold-lock bleed
        if target_steer.abs() > 0.1 && target_steer.signum() == self.current_steer.signum() {
            if self.config.hold_bleed_rate > 0.0 {
                self.steer_hold_factor = (self.steer_hold_factor + self.config.hold_bleed_rate * dt).min(1.0);
            }
        } else {
            self.steer_hold_factor = 0.0;
        }

        let base_speed_scale = (1.0 / (1.0 + speed_mps * self.config.speed_sensitive_factor))
            .max(self.config.min_speed_steer_limit);
        let speed_scale = base_speed_scale + (1.0 - base_speed_scale) * self.steer_hold_factor;

        let final_steer = (curved_steer * speed_scale).clamp(-1.0, 1.0);

        // 4. Throttle smoothing
        let throttle_step = self.config.throttle_rise_rate * dt;
        if self.current_throttle < target_throttle {
            self.current_throttle = (self.current_throttle + throttle_step).min(target_throttle);
        } else {
            self.current_throttle = target_throttle;
        }

        // 5. Brake smoothing
        let brake_step = self.config.brake_rise_rate * dt;
        if self.current_brake < target_brake {
            self.current_brake = (self.current_brake + brake_step).min(target_brake);
        } else {
            self.current_brake = target_brake;
        }

        (final_steer, self.current_throttle, self.current_brake)
    }
}
