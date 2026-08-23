use serde::{Deserialize, Serialize};

/// Configuration for digital keyboard input smoothing and progressive steering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DigitalInputConfig {
    /// Steering rise rate in units/second (how quickly steering reaches full lock).
    pub steer_rise_rate: f32,
    /// Steering return-to-center rate in units/second when steering keys are released.
    pub steer_return_rate: f32,
    /// Steering non-linear response exponent (> 1.0 creates a soft zone near center for precision).
    pub steer_exponent: f32,
    /// Dynamic speed-sensitive steering attenuation factor.
    /// Attenuates max steering angle at higher vehicle speeds to avoid sudden snap spins.
    pub speed_sensitive_factor: f32,
    /// Minimum steering limit fraction at top speed.
    pub min_speed_steer_limit: f32,
    /// Throttle rise rate in units/second to avoid instantaneous torque spikes.
    pub throttle_rise_rate: f32,
    /// Service brake rise rate in units/second.
    pub brake_rise_rate: f32,
}

impl Default for DigitalInputConfig {
    fn default() -> Self {
        Self {
            steer_rise_rate: 7.2,
            steer_return_rate: 13.5,
            steer_exponent: 1.05,
            speed_sensitive_factor: 0.005,
            min_speed_steer_limit: 0.65,
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

    /// Filters and smooths raw digital inputs over timestep `dt` with vehicle forward speed scaling.
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
            // Immediate drop on release for engine braking / coasting
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digital_input_filter_ramp_and_centering() {
        let mut filter = DigitalInputFilter::default();
        let dt = 1.0 / 60.0;

        // Step 1 frame with full right steer at 0 speed
        let (s1, _, _) = filter.update(1.0, 0.0, 0.0, 0.0, dt);
        assert!(s1 > 0.0 && s1 < 0.2, "Steering should ramp smoothly, got {s1}");

        // Step 30 frames (0.5s) with full right steer
        for _ in 0..30 {
            filter.update(1.0, 0.0, 0.0, 0.0, dt);
        }
        assert_eq!(filter.current_steer, 1.0, "Steering should reach 1.0 after 0.5s");

        // Release keys: center rate is fast
        filter.update(0.0, 0.0, 0.0, 0.0, dt);
        assert!(filter.current_steer < 0.85, "Steering should quickly center on release");
    }

    #[test]
    fn test_speed_sensitive_steering_attenuation() {
        let mut filter = DigitalInputFilter::default();
        let dt = 1.0 / 60.0;

        // Hold full steer until saturated at 0 speed
        for _ in 0..60 {
            filter.update(1.0, 0.0, 0.0, 0.0, dt);
        }
        let (steer_low_speed, _, _) = filter.update(1.0, 0.0, 0.0, 0.0, dt);
        assert_eq!(steer_low_speed, 1.0);

        // At 50 m/s (~180 km/h), steering should be moderately attenuated
        let (steer_high_speed, _, _) = filter.update(1.0, 0.0, 0.0, 50.0, dt);
        assert!(
            steer_high_speed < 0.90 && steer_high_speed >= 0.65,
            "High speed steer should be moderately attenuated, got {steer_high_speed}"
        );
    }

    #[test]
    fn test_non_linear_exponential_curve() {
        let config = DigitalInputConfig {
            steer_exponent: 1.5,
            speed_sensitive_factor: 0.0,
            ..Default::default()
        };
        let mut filter = DigitalInputFilter::new(config);
        filter.current_steer = 0.5;

        let (steer, _, _) = filter.update(0.5, 0.0, 0.0, 0.0, 1.0 / 60.0);
        assert!(
            steer < 0.40 && steer > 0.30,
            "Non-linear exponent should soften mid-point steering, got {steer}"
        );
    }

    #[test]
    fn test_throttle_and_brake_smoothing() {
        let mut filter = DigitalInputFilter::default();
        let dt = 1.0 / 60.0;

        // Throttle rise
        let (_, t1, _) = filter.update(0.0, 1.0, 0.0, 0.0, dt);
        assert!(t1 > 0.0 && t1 < 0.25, "Throttle should rise smoothly, got {t1}");

        // Throttle release is immediate
        filter.current_throttle = 1.0;
        let (_, t_rel, _) = filter.update(0.0, 0.0, 0.0, 0.0, dt);
        assert_eq!(t_rel, 0.0, "Throttle release should be instant for coasting");
    }
}
