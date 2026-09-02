use serde::{Deserialize, Serialize};

/// Configuration for digital keyboard input smoothing and progressive steering/turning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DigitalInputConfig {
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
    /// Throttle/acceleration rise rate in units/second.
    pub throttle_rise_rate: f32,
    /// Service brake/deceleration rise rate in units/second.
    pub brake_rise_rate: f32,
}

impl Default for DigitalInputConfig {
    fn default() -> Self {
        Self {
            steer_rise_rate: 7.2,
            steer_return_rate: 13.5,
            steer_exponent: 1.05,
            speed_sensitive_factor: 0.0025,
            min_speed_steer_limit: 0.70,
            throttle_rise_rate: 9.5,
            brake_rise_rate: 16.0,
        }
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

        // 3. Dynamic speed-sensitive scaling
        let speed_scale = (1.0 / (1.0 + speed_mps * self.config.speed_sensitive_factor))
            .max(self.config.min_speed_steer_limit);

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
