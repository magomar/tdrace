use glam::Vec2;
use macroquad::color::Color;
use macroquad::text::draw_text;

/// Floating animated drift score notification popup.
#[derive(Debug, Clone)]
pub struct DriftPopup {
    pub world_pos: Vec2,
    pub text: String,
    pub color: Color,
    pub lifetime: f32,
    pub remaining_life: f32,
    pub float_speed: f32,
}

#[derive(Debug, Clone)]
pub struct DriftPopupManager {
    popups: Vec<DriftPopup>,
    max_popups: usize,
}

impl DriftPopupManager {
    pub fn new(max_popups: usize) -> Self {
        Self {
            popups: Vec::with_capacity(max_popups),
            max_popups,
        }
    }

    pub fn clear(&mut self) {
        self.popups.clear();
    }

    /// Spawns a drift score popup.
    pub fn spawn_drift_score(&mut self, pos: Vec2, score: f32, multiplier: f32) {
        if score < 50.0 {
            return;
        }

        let text = if multiplier > 1.05 {
            format!("+{:.0} DRIFT! (x{:.1})", score, multiplier)
        } else {
            format!("+{:.0} DRIFT!", score)
        };

        let color = if score > 800.0 {
            Color::new(1.0, 0.25, 0.80, 1.0) // Neon Magenta
        } else if score > 400.0 {
            Color::new(1.0, 0.82, 0.15, 1.0) // Neon Gold
        } else {
            Color::new(0.20, 0.90, 1.0, 1.0) // Neon Cyan
        };

        if self.popups.len() >= self.max_popups {
            self.popups.remove(0);
        }

        self.popups.push(DriftPopup {
            world_pos: pos + Vec2::new(0.0, -1.2),
            text,
            color,
            lifetime: 1.2,
            remaining_life: 1.2,
            float_speed: 1.8,
        });
    }

    /// Spawns a custom floating notification text popup.
    pub fn spawn_text(&mut self, pos: Vec2, text: &str, color: Color) {
        if self.popups.len() >= self.max_popups {
            self.popups.remove(0);
        }

        self.popups.push(DriftPopup {
            world_pos: pos + Vec2::new(0.0, -1.2),
            text: text.to_string(),
            color,
            lifetime: 1.4,
            remaining_life: 1.4,
            float_speed: 1.8,
        });
    }

    pub fn update(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.popups.len() {
            let p = &mut self.popups[i];
            p.remaining_life -= dt;
            p.world_pos.y -= p.float_speed * dt;

            if p.remaining_life <= 0.0 {
                self.popups.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Renders drift popups in world coordinates with high-contrast drop shadow.
    pub fn render_in_world(&self) {
        for p in &self.popups {
            let alpha = (p.remaining_life / p.lifetime).clamp(0.0, 1.0);
            let shadow_col = Color::new(0.0, 0.0, 0.0, alpha * 0.65);
            let text_col = Color::new(p.color.r, p.color.g, p.color.b, alpha);

            // Drop shadow
            draw_text(&p.text, p.world_pos.x - 1.95, p.world_pos.y + 0.05, 1.2, shadow_col);
            // Text foreground
            draw_text(&p.text, p.world_pos.x - 2.0, p.world_pos.y, 1.2, text_col);
        }
    }

    /// List of active popups for screen projection rendering.
    pub fn active_popups(&self) -> &[DriftPopup] {
        &self.popups
    }
}
