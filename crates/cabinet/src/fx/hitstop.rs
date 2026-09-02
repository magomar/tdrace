use serde::{Deserialize, Serialize};

/// Micro-pause freeze frame controller for high-impact hits and critical moments.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct HitStop {
    pub duration_remaining: f32,
    pub time_scale: f32,
}

impl HitStop {
    pub fn new() -> Self {
        Self {
            duration_remaining: 0.0,
            time_scale: 1.0,
        }
    }

    /// Freezes gameplay simulation for `duration_s` seconds (e.g. 0.06s for a standard hit).
    pub fn freeze(&mut self, duration_s: f32) {
        self.duration_remaining = duration_s.max(self.duration_remaining);
        self.time_scale = 0.0;
    }

    /// Triggers slow-motion time dilation with specified time scale (e.g. 0.2 for 5x slow-mo).
    pub fn slow_motion(&mut self, time_scale: f32, duration_s: f32) {
        self.duration_remaining = duration_s;
        self.time_scale = time_scale.clamp(0.0, 1.0);
    }

    /// Updates hit-stop countdown and returns effective delta time for gameplay simulation.
    pub fn step(&mut self, real_dt: f32) -> f32 {
        if self.duration_remaining > 0.0 {
            self.duration_remaining = (self.duration_remaining - real_dt).max(0.0);
            if self.duration_remaining == 0.0 {
                self.time_scale = 1.0;
            }
            real_dt * self.time_scale
        } else {
            self.time_scale = 1.0;
            real_dt
        }
    }
}
