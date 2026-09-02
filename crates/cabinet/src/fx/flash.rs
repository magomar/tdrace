use macroquad::color::Color;
use macroquad::shapes::draw_rectangle;

/// High-visibility full-screen color flash effect for impacts, collisions, and rewards.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenFlash {
    pub color: Color,
    pub initial_alpha: f32,
    pub duration: f32,
    pub timer: f32,
}

impl ScreenFlash {
    pub fn new() -> Self {
        Self::default()
    }

    /// Triggers a screen flash with specified color and duration in seconds.
    pub fn trigger(&mut self, color: Color, duration: f32) {
        self.color = color;
        self.initial_alpha = color.a.clamp(0.0, 1.0);
        self.duration = duration.max(1e-4);
        self.timer = duration;
    }

    /// Updates the flash decay timer over timestep `dt`.
    pub fn update(&mut self, dt: f32) {
        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
        }
    }

    /// Renders the flash overlay if currently active.
    pub fn draw(&self, sw: f32, sh: f32) {
        if self.timer > 0.0 {
            let progress = self.timer / self.duration;
            let alpha = self.initial_alpha * progress;
            let flash_col = Color::new(self.color.r, self.color.g, self.color.b, alpha);
            draw_rectangle(0.0, 0.0, sw, sh, flash_col);
        }
    }
}
