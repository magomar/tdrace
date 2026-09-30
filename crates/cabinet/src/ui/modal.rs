use macroquad::color::Color;
use macroquad::shapes::{draw_line, draw_rectangle};
use serde::{Deserialize, Serialize};

use crate::ui::font::Fonts;
use crate::ui::layout::LayoutRect;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Uniform modal window container with backdrop dimming, chrome frame, and title bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModalContainer {
    pub title: String,
    pub bounds: LayoutRect,
    pub is_open: bool,
    pub dim_alpha: f32,
}

impl ModalContainer {
    pub fn new(title: impl Into<String>, bounds: LayoutRect) -> Self {
        Self {
            title: title.into(),
            bounds,
            is_open: true,
            dim_alpha: 0.65,
        }
    }

    pub fn with_dim_alpha(mut self, alpha: f32) -> Self {
        self.dim_alpha = alpha.clamp(0.0, 1.0);
        self
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Returns the content area inside the modal beneath the title bar.
    pub fn content_rect(&self) -> LayoutRect {
        let title_h = 44.0;
        let pad = 16.0;
        LayoutRect::new(
            self.bounds.x + pad,
            self.bounds.y + title_h,
            (self.bounds.w - pad * 2.0).max(0.0),
            (self.bounds.h - title_h - pad).max(0.0),
        )
    }

    /// Renders the modal container with full-screen dimming backdrop, chrome frame, and title.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts, screen_w: f32, screen_h: f32) {
        if !self.is_open {
            return;
        }

        // Full-screen backdrop dim
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, self.dim_alpha));

        // Glassmorphism card frame
        scaler.draw_glass_card(
            self.bounds.x,
            self.bounds.y,
            self.bounds.w,
            self.bounds.h,
            Color::new(0.07, 0.10, 0.16, 0.96),
            Palette::NEON_GOLD,
            2.0,
        );

        // Title bar text
        let title_y = self.bounds.y + scaler.s(28.0);
        fonts.draw_display_centered(
            &self.title,
            self.bounds.x + self.bounds.w * 0.5,
            title_y,
            scaler.font_s(18.0),
            Palette::WHITE,
        );

        // Accent divider below title
        let line_y = self.bounds.y + scaler.s(38.0);
        let margin_x = self.bounds.w * 0.15;
        draw_line(
            self.bounds.x + margin_x,
            line_y,
            self.bounds.x + self.bounds.w - margin_x,
            line_y,
            1.5,
            Palette::NEON_GOLD,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_container_open_close_and_content_rect() {
        let bounds = LayoutRect::new(100.0, 100.0, 600.0, 400.0);
        let mut modal = ModalContainer::new("SETTINGS", bounds);
        assert!(modal.is_open);
        assert_eq!(modal.title, "SETTINGS");

        modal.close();
        assert!(!modal.is_open);

        modal.toggle();
        assert!(modal.is_open);

        let content = modal.content_rect();
        assert_eq!(content.x, 116.0);
        assert_eq!(content.y, 144.0);
        assert_eq!(content.w, 600.0 - 32.0);
        assert_eq!(content.h, 400.0 - 44.0 - 16.0);
    }
}
