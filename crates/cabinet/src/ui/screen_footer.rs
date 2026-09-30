use macroquad::color::Color;
use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::{draw_line, draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use crate::audio::CabinetAudioSink;
use crate::ui::layout::LayoutRect;
use crate::ui::theme::Palette;

#[inline]
fn safe_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(|| is_key_pressed(key)).unwrap_or(false)
}

#[inline]
fn safe_mouse_pos() -> (f32, f32) {
    std::panic::catch_unwind(mouse_position).unwrap_or((-1000.0, -1000.0))
}

#[inline]
fn safe_mouse_pressed(btn: MouseButton) -> bool {
    std::panic::catch_unwind(|| is_mouse_button_pressed(btn)).unwrap_or(false)
}

/// Key or gamepad button prompt displayed in the footer bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FooterPrompt {
    pub badge: String,
    pub label: String,
}

impl FooterPrompt {
    pub fn new(badge: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            badge: badge.into(),
            label: label.into(),
        }
    }
}

/// Call-to-Action (CTA) hero button situated on the right side of the screen footer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeroActionButton {
    pub label: String,
    pub is_focused: bool,
    pub is_disabled: bool,
    pub width: f32,
    pub height: f32,
}

impl HeroActionButton {
    pub fn new(label: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            label: label.into(),
            is_focused: false,
            is_disabled: false,
            width,
            height,
        }
    }
}

/// Standardized bottom screen footer bar presenting controller hints and a hero CTA button.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenFooter {
    pub prompts: Vec<FooterPrompt>,
    pub hero_button: Option<HeroActionButton>,
    pub base_x: f32,
    pub base_y: f32,
    pub width: f32,
    pub height: f32,
}

impl ScreenFooter {
    /// Creates a new screen footer at the specified coordinates.
    pub fn new(base_x: f32, base_y: f32, width: f32, height: f32) -> Self {
        Self {
            prompts: Vec::new(),
            hero_button: None,
            base_x,
            base_y,
            width,
            height,
        }
    }

    /// Adds a controller / keyboard prompt chip.
    pub fn add_prompt(&mut self, badge: impl Into<String>, label: impl Into<String>) {
        self.prompts.push(FooterPrompt::new(badge, label));
    }

    /// Attaches or updates the hero action button.
    pub fn set_hero_button(&mut self, button: HeroActionButton) {
        self.hero_button = Some(button);
    }

    /// Returns the bounding rectangle of the full footer bar.
    #[inline]
    pub fn bounds(&self) -> LayoutRect {
        LayoutRect::new(self.base_x, self.base_y, self.width, self.height)
    }

    /// Returns the bounding rectangle of the hero CTA button, if present.
    pub fn hero_button_rect(&self) -> Option<LayoutRect> {
        self.hero_button.as_ref().map(|btn| {
            let margin_right = 32.0;
            let x = self.base_x + self.width - btn.width - margin_right;
            let y = self.base_y + (self.height - btn.height) * 0.5;
            LayoutRect::new(x, y, btn.width, btn.height)
        })
    }

    /// Checks if a screen coordinate hits the hero action button.
    pub fn hit_test_hero_button(&self, px: f32, py: f32) -> bool {
        if let Some(rect) = self.hero_button_rect() {
            if let Some(btn) = &self.hero_button {
                if !btn.is_disabled {
                    return rect.contains(px, py);
                }
            }
        }
        false
    }

    /// Processes keyboard, mouse, and confirmation events on the hero button.
    /// Returns true if the hero action was confirmed.
    pub fn handle_input(&mut self, audio: Option<&dyn CabinetAudioSink>) -> bool {
        let (mx, my) = safe_mouse_pos();
        let clicked = safe_mouse_pressed(MouseButton::Left) && self.hit_test_hero_button(mx, my);

        let mut confirmed = clicked;

        if let Some(btn) = &mut self.hero_button {
            if !btn.is_disabled {
                if btn.is_focused && (safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::Space)) {
                    confirmed = true;
                }
            }
        }

        if confirmed {
            if let Some(audio) = audio {
                audio.play_ui_select();
            }
        }

        confirmed
    }

    /// Renders the footer background bar, top divider line, and hero button frame.
    pub fn render_frame(&self) {
        let b = self.bounds();

        // Footer background
        draw_rectangle(b.x, b.y, b.w, b.h, Color::new(0.04, 0.05, 0.08, 0.95));
        // Top divider line
        draw_line(
            b.x,
            b.y,
            b.x + b.w,
            b.y,
            1.5,
            Color::new(0.20, 0.28, 0.40, 0.60),
        );

        // Hero CTA button rendering if present
        if let (Some(btn), Some(rect)) = (&self.hero_button, self.hero_button_rect()) {
            let (bg, border, thickness) = if btn.is_disabled {
                (
                    Color::new(0.12, 0.12, 0.16, 0.50),
                    Color::new(0.25, 0.25, 0.30, 0.40),
                    1.0,
                )
            } else if btn.is_focused {
                (
                    Color::new(0.25, 0.20, 0.05, 0.95),
                    Palette::NEON_GOLD,
                    2.4,
                )
            } else {
                (
                    Color::new(0.10, 0.15, 0.25, 0.85),
                    Palette::NEON_CYAN,
                    1.5,
                )
            };

            draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, border);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_prompts_and_layout() {
        let mut footer = ScreenFooter::new(0.0, 1020.0, 1920.0, 60.0);
        footer.add_prompt("[A]", "SELECT");
        footer.add_prompt("[B]", "BACK");
        footer.add_prompt("[LB/RB]", "SWITCH CATEGORY");

        assert_eq!(footer.prompts.len(), 3);
        assert_eq!(footer.prompts[0].badge, "[A]");
        assert_eq!(footer.prompts[0].label, "SELECT");
        assert_eq!(footer.bounds(), LayoutRect::new(0.0, 1020.0, 1920.0, 60.0));
    }

    #[test]
    fn test_hero_button_position_and_hit_test() {
        let mut footer = ScreenFooter::new(0.0, 1020.0, 1920.0, 60.0);
        footer.set_hero_button(HeroActionButton::new("START RACE", 200.0, 44.0));

        let rect = footer.hero_button_rect().unwrap();
        // x = 1920.0 - 200.0 - 32.0 = 1688.0
        // y = 1020.0 + (60.0 - 44.0) * 0.5 = 1028.0
        assert_eq!(rect, LayoutRect::new(1688.0, 1028.0, 200.0, 44.0));

        assert!(footer.hit_test_hero_button(1700.0, 1030.0));
        assert!(!footer.hit_test_hero_button(100.0, 1030.0));

        // When disabled, hit testing returns false
        if let Some(btn) = &mut footer.hero_button {
            btn.is_disabled = true;
        }
        assert!(!footer.hit_test_hero_button(1700.0, 1030.0));
    }
}
