use macroquad::color::Color;
use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::{draw_circle, draw_rectangle, draw_rectangle_lines};

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

/// Navigation and interaction actions emitted by a SwatchPicker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwatchAction {
    None,
    Changed(usize),
    Confirmed(usize),
    ExitTop,
    ExitBottom,
    ExitLeft,
    ExitRight,
}

/// Horizontal color swatch selector ribbon with directional gamepad navigation.
#[derive(Debug, Clone, PartialEq)]
pub struct SwatchPicker {
    pub colors: Vec<Color>,
    pub selected_idx: usize,
    pub is_focused: bool,
    pub swatch_size: f32,
    pub gap: f32,
    pub base_x: f32,
    pub base_y: f32,
    pub wrap: bool,
}

impl SwatchPicker {
    /// Creates a new SwatchPicker with the given color palette.
    pub fn new(
        base_x: f32,
        base_y: f32,
        swatch_size: f32,
        gap: f32,
        colors: Vec<Color>,
    ) -> Self {
        Self {
            colors,
            selected_idx: 0,
            is_focused: false,
            swatch_size: swatch_size.max(12.0),
            gap: gap.max(2.0),
            base_x,
            base_y,
            wrap: false,
        }
    }

    /// Number of colors in the picker.
    #[inline]
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Returns true if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Returns the currently selected color if available.
    pub fn selected_color(&self) -> Option<Color> {
        self.colors.get(self.selected_idx).copied()
    }

    /// Computes total width of the swatch strip.
    pub fn total_width(&self) -> f32 {
        if self.colors.is_empty() {
            0.0
        } else {
            (self.colors.len() as f32) * self.swatch_size + ((self.colors.len() - 1) as f32) * self.gap
        }
    }

    /// Returns the bounding rectangle of the swatch at `idx`.
    pub fn swatch_rect(&self, idx: usize) -> LayoutRect {
        let x = self.base_x + (idx as f32) * (self.swatch_size + self.gap);
        LayoutRect::new(x, self.base_y, self.swatch_size, self.swatch_size)
    }

    /// Returns the overall bounding rectangle of the swatch picker.
    pub fn bounds(&self) -> LayoutRect {
        LayoutRect::new(self.base_x, self.base_y, self.total_width(), self.swatch_size)
    }

    /// Hit-tests a coordinate against all swatches.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        for i in 0..self.colors.len() {
            if self.swatch_rect(i).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Navigates left with optional wrapping or boundary exit signal.
    pub fn nav_left(&mut self) -> SwatchAction {
        if self.colors.is_empty() {
            return SwatchAction::ExitLeft;
        }
        if self.selected_idx == 0 {
            if self.wrap {
                self.selected_idx = self.colors.len() - 1;
                SwatchAction::Changed(self.selected_idx)
            } else {
                SwatchAction::ExitLeft
            }
        } else {
            self.selected_idx -= 1;
            SwatchAction::Changed(self.selected_idx)
        }
    }

    /// Navigates right with optional wrapping or boundary exit signal.
    pub fn nav_right(&mut self) -> SwatchAction {
        if self.colors.is_empty() {
            return SwatchAction::ExitRight;
        }
        if self.selected_idx + 1 >= self.colors.len() {
            if self.wrap {
                self.selected_idx = 0;
                SwatchAction::Changed(self.selected_idx)
            } else {
                SwatchAction::ExitRight
            }
        } else {
            self.selected_idx += 1;
            SwatchAction::Changed(self.selected_idx)
        }
    }

    /// Processes keyboard, mouse, and directional events when focused.
    pub fn handle_input(&mut self, audio: Option<&dyn CabinetAudioSink>) -> SwatchAction {
        if !self.is_focused || self.colors.is_empty() {
            return SwatchAction::None;
        }

        let mut action = SwatchAction::None;

        if safe_key_pressed(KeyCode::Left) || safe_key_pressed(KeyCode::A) {
            action = self.nav_left();
        } else if safe_key_pressed(KeyCode::Right) || safe_key_pressed(KeyCode::D) {
            action = self.nav_right();
        } else if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) {
            action = SwatchAction::ExitTop;
        } else if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) {
            action = SwatchAction::ExitBottom;
        } else if safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::Space) {
            action = SwatchAction::Confirmed(self.selected_idx);
        }

        // Mouse hit test
        let (mx, my) = safe_mouse_pos();
        if let Some(hovered) = self.hit_test(mx, my) {
            if hovered != self.selected_idx {
                self.selected_idx = hovered;
                action = SwatchAction::Changed(hovered);
            }
            if safe_mouse_pressed(MouseButton::Left) {
                action = SwatchAction::Confirmed(hovered);
            }
        }

        if let Some(audio) = audio {
            match action {
                SwatchAction::Changed(_) => audio.play_ui_move(),
                SwatchAction::Confirmed(_) => audio.play_ui_select(),
                _ => {}
            }
        }

        action
    }

    /// Renders the swatch chips and focus indicator frame.
    pub fn render_frame(&self) {
        for (i, color) in self.colors.iter().enumerate() {
            let r = self.swatch_rect(i);
            let is_selected = i == self.selected_idx;

            // Fill color
            draw_rectangle(r.x, r.y, r.w, r.h, *color);

            // Border styling
            if is_selected && self.is_focused {
                draw_rectangle_lines(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 2.4, Palette::NEON_GOLD);
                // Center selection dot
                let center = r.center();
                draw_circle(center.0, center.1, 3.0, Palette::WHITE);
            } else if is_selected {
                draw_rectangle_lines(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0, 1.8, Palette::NEON_CYAN);
            } else {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Palette::UI_CARD_BORDER);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swatch_picker_layout_and_rects() {
        let colors = vec![Palette::RED, Palette::GREEN, Palette::BLUE];
        let picker = SwatchPicker::new(10.0, 20.0, 30.0, 5.0, colors);

        assert_eq!(picker.len(), 3);
        assert_eq!(picker.total_width(), 30.0 * 3.0 + 5.0 * 2.0); // 100.0

        let r0 = picker.swatch_rect(0);
        assert_eq!(r0, LayoutRect::new(10.0, 20.0, 30.0, 30.0));

        let r1 = picker.swatch_rect(1);
        assert_eq!(r1, LayoutRect::new(45.0, 20.0, 30.0, 30.0));

        assert_eq!(picker.hit_test(15.0, 25.0), Some(0));
        assert_eq!(picker.hit_test(50.0, 25.0), Some(1));
        assert_eq!(picker.hit_test(150.0, 25.0), None);
    }

    #[test]
    fn test_swatch_picker_navigation_and_exits() {
        let colors = vec![Palette::RED, Palette::GREEN];
        let mut picker = SwatchPicker::new(0.0, 0.0, 20.0, 4.0, colors);

        assert_eq!(picker.nav_left(), SwatchAction::ExitLeft);
        assert_eq!(picker.nav_right(), SwatchAction::Changed(1));
        assert_eq!(picker.nav_right(), SwatchAction::ExitRight);

        // With wrap enabled
        picker.wrap = true;
        assert_eq!(picker.nav_right(), SwatchAction::Changed(0));
        assert_eq!(picker.nav_left(), SwatchAction::Changed(1));
    }
}
