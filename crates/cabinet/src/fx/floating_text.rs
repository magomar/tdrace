use glam::Vec2;
use macroquad::color::Color;

use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Individual floating text entity that floats, pops, and decays.
#[derive(Debug, Clone)]
pub struct FloatingTextItem {
    pub text: String,
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub font_size: f32,
    pub scale: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl FloatingTextItem {
    /// Progress of lifetime from 0.0 (fresh) to 1.0 (expired).
    #[inline]
    pub fn progress(&self) -> f32 {
        if self.max_lifetime <= 0.0 {
            1.0
        } else {
            1.0 - (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
        }
    }

    /// Computed alpha transparency based on remaining lifetime.
    pub fn alpha(&self) -> f32 {
        let fade_threshold = 0.45;
        let remaining_ratio = if self.max_lifetime <= 0.0 {
            0.0
        } else {
            self.lifetime / self.max_lifetime
        };

        if remaining_ratio < fade_threshold {
            (remaining_ratio / fade_threshold).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

/// Lightweight particle-like manager for floating arcade scores, combo counters,
/// and alert notifications.
#[derive(Debug, Clone)]
pub struct FloatingTextManager {
    pub items: Vec<FloatingTextItem>,
    pub max_capacity: usize,
}

impl Default for FloatingTextManager {
    fn default() -> Self {
        Self::new(64)
    }
}

impl FloatingTextManager {
    /// Creates a manager with specified maximum entity capacity.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_capacity.min(128)),
            max_capacity,
        }
    }

    /// Number of active floating text elements.
    #[inline]
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Whether there are no active floating text elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clears all active floating text.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Spawns a floating text element with custom parameters.
    pub fn spawn(
        &mut self,
        text: impl Into<String>,
        pos: Vec2,
        vel: Vec2,
        color: Color,
        font_size: f32,
        initial_scale: f32,
        duration: f32,
    ) {
        if self.items.len() >= self.max_capacity {
            // Remove the item closest to expiration to make room
            if let Some(min_idx) = self
                .items
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.lifetime.partial_cmp(&b.1.lifetime).unwrap())
                .map(|(idx, _)| idx)
            {
                self.items.swap_remove(min_idx);
            }
        }

        self.items.push(FloatingTextItem {
            text: text.into(),
            pos,
            vel,
            color,
            font_size,
            scale: initial_scale,
            lifetime: duration,
            max_lifetime: duration,
        });
    }

    /// Spawns a standard "+N PTS" arcade score popup.
    pub fn spawn_score(&mut self, score: u32, pos: Vec2) {
        let text = format!("+{score} PTS");
        self.spawn(
            text,
            pos,
            Vec2::new(0.0, -32.0),
            Palette::NEON_GOLD,
            16.0,
            1.35,
            0.9,
        );
    }

    /// Spawns a "COMBO xN!" arcade combo notification.
    pub fn spawn_combo(&mut self, combo: u32, pos: Vec2) {
        let text = format!("COMBO x{combo}!");
        self.spawn(
            text,
            pos,
            Vec2::new(0.0, -42.0),
            Palette::NEON_CYAN,
            20.0,
            1.55,
            1.1,
        );
    }

    /// Spawns an alert or milestone banner (e.g. "PERFECT APEX", "NEW RECORD!").
    pub fn spawn_alert(&mut self, text: impl Into<String>, pos: Vec2, color: Color) {
        self.spawn(
            text,
            pos,
            Vec2::new(0.0, -24.0),
            color,
            22.0,
            1.6,
            1.3,
        );
    }

    /// Steps animation physics and lifetimes by `dt`, purging expired elements.
    pub fn update(&mut self, dt: f32) {
        for item in &mut self.items {
            item.lifetime -= dt;
            item.pos += item.vel * dt;
            item.vel *= 0.95; // Atmospheric damping
            // Spring scale back towards 1.0
            item.scale += (1.0 - item.scale) * (9.0 * dt).min(1.0);
        }

        self.items.retain(|item| item.lifetime > 0.0);
    }

    /// Renders all active floating text popups using embedded fonts and scale adjustments.
    pub fn draw(&self, fonts: &Fonts, scaler: &UiScaler) {
        for item in &self.items {
            let alpha = item.alpha();
            if alpha <= 0.01 {
                continue;
            }

            let effective_size = scaler.font_s(item.font_size * item.scale);
            let col = Color::new(item.color.r, item.color.g, item.color.b, item.color.a * alpha);
            let shadow = Color::new(0.0, 0.0, 0.0, 0.75 * alpha);

            fonts.draw_display_centered_with_shadow(
                &item.text,
                item.pos.x,
                item.pos.y,
                effective_size,
                col,
                shadow,
                scaler.s(2.0),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_text_spawning_and_lifecycle() {
        let mut mgr = FloatingTextManager::new(10);
        assert!(mgr.is_empty());
        assert_eq!(mgr.count(), 0);

        mgr.spawn_score(250, Vec2::new(100.0, 100.0));
        assert_eq!(mgr.count(), 1);

        let item = &mgr.items[0];
        assert_eq!(item.text, "+250 PTS");
        assert_eq!(item.pos, Vec2::new(100.0, 100.0));
        assert!(item.scale > 1.0);
        assert_eq!(item.alpha(), 1.0);

        // Step by 0.5s: position should rise, scale should decay toward 1.0
        mgr.update(0.5);
        assert_eq!(mgr.count(), 1);
        let updated = &mgr.items[0];
        assert!(updated.pos.y < 100.0, "Text should rise vertically");
        assert!(updated.scale < 1.35, "Scale should settle towards 1.0");

        // Step beyond remaining lifetime (total 0.9s)
        mgr.update(0.6);
        assert_eq!(mgr.count(), 0, "Expired item should be removed");
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_capacity_capping() {
        let mut mgr = FloatingTextManager::new(3);
        mgr.spawn("Text 1", Vec2::ZERO, Vec2::ZERO, Palette::WHITE, 12.0, 1.0, 0.3);
        mgr.spawn("Text 2", Vec2::ZERO, Vec2::ZERO, Palette::WHITE, 12.0, 1.0, 1.0);
        mgr.spawn("Text 3", Vec2::ZERO, Vec2::ZERO, Palette::WHITE, 12.0, 1.0, 2.0);
        assert_eq!(mgr.count(), 3);

        // Spawning a 4th item should replace the lowest lifetime item ("Text 1")
        mgr.spawn("Text 4", Vec2::ZERO, Vec2::ZERO, Palette::WHITE, 12.0, 1.0, 5.0);
        assert_eq!(mgr.count(), 3);
        assert!(!mgr.items.iter().any(|item| item.text == "Text 1"));
        assert!(mgr.items.iter().any(|item| item.text == "Text 4"));
    }
}
