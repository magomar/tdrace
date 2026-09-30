use macroquad::color::Color;
use serde::{Deserialize, Serialize};

use crate::ui::font::Fonts;
use crate::ui::layout::LayoutRect;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Contextual tooltip widget providing on-hover or on-focus descriptive help.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tooltip {
    pub text: String,
    pub anchor: LayoutRect,
    pub is_visible: bool,
}

impl Tooltip {
    pub fn new(text: impl Into<String>, anchor: LayoutRect) -> Self {
        Self {
            text: text.into(),
            anchor,
            is_visible: true,
        }
    }

    pub fn with_visible(mut self, is_visible: bool) -> Self {
        self.is_visible = is_visible;
        self
    }

    /// Computes the tooltip floating bounding box relative to its anchor element.
    pub fn tooltip_rect(&self, scaler: &UiScaler, fonts: &Fonts) -> LayoutRect {
        let font_size = scaler.font_s(11.0);
        let dim = fonts.measure_ui_regular(&self.text, font_size);
        let pad_h = scaler.s(10.0);
        let pad_v = scaler.s(6.0);
        let w = dim.width + pad_h * 2.0;
        let h = dim.height + pad_v * 2.0;

        let center_x = self.anchor.x + self.anchor.w * 0.5;
        let x = (center_x - w * 0.5).max(scaler.s(8.0));
        let y = self.anchor.y - h - scaler.s(6.0);

        LayoutRect::new(x, y, w, h)
    }

    /// Renders the tooltip bubble.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts) {
        if !self.is_visible || self.text.is_empty() {
            return;
        }

        let rect = self.tooltip_rect(scaler, fonts);
        scaler.draw_glass_card(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.04, 0.06, 0.10, 0.95),
            Palette::UI_CARD_BORDER_GLOW,
            1.0,
        );

        let pad_h = scaler.s(10.0);
        fonts.draw_ui_regular(
            &self.text,
            rect.x + pad_h,
            rect.y + rect.h * 0.72,
            scaler.font_s(11.0),
            Palette::WHITE,
        );
    }
}

/// Compact inline keyboard or controller shortcut indicator chip (e.g. `[SPACE] Drift`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HelpChip {
    pub shortcut: String,
    pub label: String,
}

impl HelpChip {
    pub fn new(shortcut: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            shortcut: shortcut.into(),
            label: label.into(),
        }
    }

    /// Renders the help chip at `(x, y)` and returns its total rendered width.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts, x: f32, y: f32) -> f32 {
        let font_size = scaler.font_s(10.5);
        let sc_dim = fonts.measure_ui_bold(&self.shortcut, font_size);
        let lbl_dim = fonts.measure_ui_regular(&self.label, font_size);

        let chip_h = scaler.s(18.0);
        let sc_w = sc_dim.width + scaler.s(10.0);
        let total_w = sc_w + scaler.s(6.0) + lbl_dim.width + scaler.s(6.0);

        // Shortcut pill
        scaler.draw_glass_card(
            x,
            y,
            sc_w,
            chip_h,
            Palette::UI_PILL_BG,
            Palette::UI_CARD_BORDER,
            1.0,
        );
        fonts.draw_ui_bold_centered(
            &self.shortcut,
            x + sc_w * 0.5,
            y + chip_h * 0.70,
            font_size,
            Palette::NEON_GOLD,
        );

        // Label
        fonts.draw_ui_regular(
            &self.label,
            x + sc_w + scaler.s(6.0),
            y + chip_h * 0.70,
            font_size,
            Palette::UI_TEXT_MUTED,
        );

        total_w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_properties() {
        let anchor = LayoutRect::new(100.0, 200.0, 150.0, 40.0);
        let tt = Tooltip::new("Toggle Assisted Steering", anchor);
        assert!(tt.is_visible);
        assert_eq!(tt.text, "Toggle Assisted Steering");
        assert_eq!(tt.anchor, anchor);
    }

    #[test]
    fn test_help_chip_creation() {
        let chip = HelpChip::new("SPACE", "Handbrake Drift");
        assert_eq!(chip.shortcut, "SPACE");
        assert_eq!(chip.label, "Handbrake Drift");
    }
}
