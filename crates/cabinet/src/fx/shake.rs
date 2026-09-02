use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Decaying trauma-based screen shake camera controller (inspired by game-feel / Comfy conventions).
/// Non-linear power formula: `offset = max_offset * (trauma ^ 2) * noise`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScreenShake {
    /// Current trauma value in [0.0, 1.0].
    pub trauma: f32,
    /// Decay rate in trauma units per second (e.g. 1.8 means full trauma clears in ~0.55s).
    pub decay_rate: f32,
    /// Maximum translational offset in world/screen pixels at trauma = 1.0.
    pub max_offset: f32,
    /// Maximum rotational perturbation in radians at trauma = 1.0.
    pub max_angle_rad: f32,
    time: f32,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            decay_rate: 1.8,
            max_offset: 16.0,
            max_angle_rad: 0.05,
            time: 0.0,
        }
    }
}

impl ScreenShake {
    pub fn new(max_offset: f32, decay_rate: f32) -> Self {
        Self {
            trauma: 0.0,
            decay_rate,
            max_offset,
            max_angle_rad: 0.05,
            time: 0.0,
        }
    }

    /// Adds trauma to the camera, clamping to 1.0 max.
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    /// Updates trauma decay over timestep `dt`.
    pub fn update(&mut self, dt: f32) {
        self.time += dt * 25.0;
        self.trauma = (self.trauma - self.decay_rate * dt).max(0.0);
    }

    /// Computes the current 2D translation offset `(dx, dy)` and rotational angle `d_rot`.
    pub fn sample_shake(&self) -> (Vec2, f32) {
        if self.trauma <= 1e-4 {
            return (Vec2::ZERO, 0.0);
        }

        let shake = self.trauma * self.trauma; // Non-linear exponent
        let sin1 = (self.time * 1.3).sin();
        let cos1 = (self.time * 1.7).cos();
        let sin2 = (self.time * 2.1).sin();

        let offset = Vec2::new(
            self.max_offset * shake * sin1,
            self.max_offset * shake * cos1,
        );
        let angle = self.max_angle_rad * shake * sin2;

        (offset, angle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_shake_decay() {
        let mut shake = ScreenShake::new(20.0, 2.0);
        shake.add_trauma(1.0);
        assert_eq!(shake.trauma, 1.0);

        shake.update(0.25);
        assert!((shake.trauma - 0.5).abs() < 1e-4);

        shake.update(0.30);
        assert_eq!(shake.trauma, 0.0);
    }
}
