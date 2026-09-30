use macroquad::color::Color;
use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use crate::audio::CabinetAudioSink;
use crate::ui::font::Fonts;
use crate::ui::layout::HStack;
use crate::ui::scaler::UiScaler;
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

/// Visual presentation style for the FilterBar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FilterBarStyle {
    /// Segmented glass pill buttons with active neon glow (e.g. Circuit category filters, Track Manager chips).
    #[default]
    Pills,
    /// Traditional arcade shelf tab bar with bottom glowing underline (e.g. Settings tabs).
    Shelf,
}

/// An individual tab or category entry in a FilterBar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterItem {
    pub label: String,
    pub badge: Option<String>,
}

impl FilterItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            badge: None,
        }
    }

    pub fn with_badge(label: impl Into<String>, badge: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            badge: Some(badge.into()),
        }
    }
}

/// Navigation action returned after processing user input on a FilterBar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterBarAction {
    None,
    /// The active tab was changed to a new index.
    Changed(usize),
    /// The active tab was confirmed (Enter / Gamepad A).
    Confirmed(usize),
    /// Focus requested to move up (Up / D-pad Up).
    ExitUp,
    /// Focus requested to move down (Down / D-pad Down).
    ExitDown,
}

/// Reusable gamepad and keyboard-friendly Filter & Tab Bar component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterBar {
    pub items: Vec<FilterItem>,
    pub active_idx: usize,
    pub is_focused: bool,
    pub style: FilterBarStyle,
    pub gap: f32,
}

impl FilterBar {
    /// Creates a new FilterBar from item labels.
    pub fn from_labels(labels: &[&str]) -> Self {
        let items = labels.iter().map(|l| FilterItem::new(*l)).collect();
        Self {
            items,
            active_idx: 0,
            is_focused: false,
            style: FilterBarStyle::Pills,
            gap: 4.0,
        }
    }

    /// Creates a new FilterBar with given items.
    pub fn new(items: Vec<FilterItem>) -> Self {
        Self {
            items,
            active_idx: 0,
            is_focused: false,
            style: FilterBarStyle::Pills,
            gap: 4.0,
        }
    }

    pub fn with_style(mut self, style: FilterBarStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_active(mut self, active: usize) -> Self {
        if !self.items.is_empty() {
            self.active_idx = active.min(self.items.len() - 1);
        }
        self
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Cycles to the previous tab (wraps around). Returns true if changed.
    pub fn prev(&mut self) -> bool {
        if self.items.len() <= 1 {
            return false;
        }
        if self.active_idx == 0 {
            self.active_idx = self.items.len() - 1;
        } else {
            self.active_idx -= 1;
        }
        true
    }

    /// Cycles to the next tab (wraps around). Returns true if changed.
    pub fn next(&mut self) -> bool {
        if self.items.len() <= 1 {
            return false;
        }
        if self.active_idx + 1 >= self.items.len() {
            self.active_idx = 0;
        } else {
            self.active_idx += 1;
        }
        true
    }

    /// Sets the active index. Returns true if changed.
    pub fn set_active(&mut self, idx: usize) -> bool {
        if idx < self.items.len() && idx != self.active_idx {
            self.active_idx = idx;
            true
        } else {
            false
        }
    }

    /// Processes keyboard, gamepad, and mouse interactions.
    pub fn handle_input(
        &mut self,
        gamepad_left: bool,
        gamepad_right: bool,
        gamepad_up: bool,
        gamepad_down: bool,
        gamepad_prev_bumper: bool,
        gamepad_next_bumper: bool,
        gamepad_select: bool,
        rect: (f32, f32, f32, f32),
        audio: Option<&dyn CabinetAudioSink>,
    ) -> FilterBarAction {
        if self.items.is_empty() {
            return FilterBarAction::None;
        }

        // Global bumper quick-cycling (LB / RB, Q / E, [ / ])
        if gamepad_prev_bumper || safe_key_pressed(KeyCode::Q) || safe_key_pressed(KeyCode::LeftBracket) {
            if self.prev() {
                if let Some(a) = audio { a.play_ui_move(); }
                return FilterBarAction::Changed(self.active_idx);
            }
        }
        if gamepad_next_bumper || safe_key_pressed(KeyCode::E) || safe_key_pressed(KeyCode::RightBracket) {
            if self.next() {
                if let Some(a) = audio { a.play_ui_move(); }
                return FilterBarAction::Changed(self.active_idx);
            }
        }

        // Mouse click on items
        let (x, y, w, h) = rect;
        let stack = HStack::new_equal(x, y, w, h, self.items.len(), self.gap);
        let (mx, my) = safe_mouse_pos();
        if safe_mouse_pressed(MouseButton::Left) {
            if let Some(clicked_idx) = stack.hit_test(mx, my) {
                if self.set_active(clicked_idx) {
                    if let Some(a) = audio { a.play_ui_move(); }
                    return FilterBarAction::Changed(self.active_idx);
                } else {
                    return FilterBarAction::Confirmed(self.active_idx);
                }
            }
        }

        // When focused, standard directional navigation
        if self.is_focused {
            if safe_key_pressed(KeyCode::Left)
                || safe_key_pressed(KeyCode::A)
                || gamepad_left
            {
                if self.prev() {
                    if let Some(a) = audio { a.play_ui_move(); }
                    return FilterBarAction::Changed(self.active_idx);
                }
            }

            if safe_key_pressed(KeyCode::Right)
                || safe_key_pressed(KeyCode::D)
                || gamepad_right
            {
                if self.next() {
                    if let Some(a) = audio { a.play_ui_move(); }
                    return FilterBarAction::Changed(self.active_idx);
                }
            }

            if safe_key_pressed(KeyCode::Up)
                || safe_key_pressed(KeyCode::W)
                || gamepad_up
            {
                return FilterBarAction::ExitUp;
            }

            if safe_key_pressed(KeyCode::Down)
                || safe_key_pressed(KeyCode::S)
                || gamepad_down
            {
                return FilterBarAction::ExitDown;
            }

            if safe_key_pressed(KeyCode::Enter)
                || safe_key_pressed(KeyCode::Space)
                || gamepad_select
            {
                if let Some(a) = audio { a.play_ui_select(); }
                return FilterBarAction::Confirmed(self.active_idx);
            }
        }

        FilterBarAction::None
    }

    /// Renders the FilterBar with active styles and focus highlight.
    pub fn draw(
        &self,
        scaler: &UiScaler,
        fonts: &Fonts,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        accent_color: Color,
    ) {
        if self.items.is_empty() {
            return;
        }

        let stack = HStack::new_equal(x, y, w, h, self.items.len(), scaler.s(self.gap));
        let (mx, my) = safe_mouse_pos();

        match self.style {
            FilterBarStyle::Pills => {
                for (i, item) in self.items.iter().enumerate() {
                    let rect = stack.item_rect(i);
                    let is_active = i == self.active_idx;
                    let is_hovered = rect.contains(mx, my);

                    let pill_bg = if is_active {
                        Color::new(0.12, 0.28, 0.45, 0.95)
                    } else if is_hovered {
                        Palette::UI_CARD_BG_HOVER
                    } else {
                        Color::new(0.06, 0.08, 0.12, 0.85)
                    };

                    let pill_border = if is_active && self.is_focused {
                        Palette::NEON_GOLD
                    } else if is_active {
                        accent_color
                    } else if is_hovered {
                        Palette::WHITE
                    } else {
                        Palette::UI_CARD_BORDER
                    };

                    let border_w = if is_active && self.is_focused {
                        2.2
                    } else if is_active {
                        1.8
                    } else {
                        1.0
                    };

                    scaler.draw_glass_card(rect.x, rect.y, rect.w, rect.h, pill_bg, pill_border, border_w);

                    let text_col = if is_active {
                        Palette::WHITE
                    } else if is_hovered {
                        Palette::WHITE
                    } else {
                        Palette::UI_TEXT_MUTED
                    };

                    let display_text = if let Some(ref badge) = item.badge {
                        format!("{} [{}]", item.label, badge)
                    } else if is_active && self.is_focused {
                        format!("< {} >", item.label)
                    } else {
                        item.label.clone()
                    };

                    fonts.draw_ui_bold_centered(
                        &display_text,
                        rect.x + rect.w * 0.5,
                        rect.y + rect.h * 0.5 + scaler.s(4.0),
                        scaler.font_s(11.0),
                        text_col,
                    );
                }
            }
            FilterBarStyle::Shelf => {
                // Shelf base container
                scaler.draw_glass_card(
                    x,
                    y,
                    w,
                    h,
                    Color::new(0.06, 0.08, 0.12, 0.88),
                    if self.is_focused { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER },
                    if self.is_focused { 2.0 } else { 1.2 },
                );

                for (idx, item) in self.items.iter().enumerate() {
                    let rect = stack.item_rect(idx);
                    let is_active = idx == self.active_idx;
                    let is_hovered = rect.contains(mx, my);

                    if is_active {
                        draw_rectangle(
                            rect.x + scaler.s(2.0),
                            rect.y + scaler.s(2.0),
                            rect.w - scaler.s(4.0),
                            rect.h - scaler.s(4.0),
                            if self.is_focused {
                                Color::new(accent_color.r * 0.40, accent_color.g * 0.40, accent_color.b * 0.40, 0.98)
                            } else {
                                Color::new(accent_color.r * 0.25, accent_color.g * 0.25, accent_color.b * 0.25, 0.95)
                            },
                        );
                        if self.is_focused {
                            draw_rectangle_lines(
                                rect.x + scaler.s(2.0),
                                rect.y + scaler.s(2.0),
                                rect.w - scaler.s(4.0),
                                rect.h - scaler.s(4.0),
                                1.8 * scaler.scale,
                                Palette::NEON_GOLD,
                            );
                        }
                        // Glowing underline
                        draw_rectangle(
                            rect.x + scaler.s(8.0),
                            rect.y + rect.h - scaler.s(3.0),
                            rect.w - scaler.s(16.0),
                            scaler.s(3.0),
                            if self.is_focused { Palette::NEON_GOLD } else { accent_color },
                        );
                    } else if is_hovered {
                        draw_rectangle(
                            rect.x + scaler.s(2.0),
                            rect.y + scaler.s(2.0),
                            rect.w - scaler.s(4.0),
                            rect.h - scaler.s(4.0),
                            Color::new(1.0, 1.0, 1.0, 0.05),
                        );
                    }

                    let text_col = if is_active {
                        Palette::WHITE
                    } else if is_hovered {
                        Palette::WHITE
                    } else {
                        Palette::UI_TEXT_MUTED
                    };

                    let display_text = if let Some(ref badge) = item.badge {
                        format!("{} [{}]", item.label, badge)
                    } else {
                        item.label.clone()
                    };

                    fonts.draw_ui_bold_centered(
                        &display_text,
                        rect.x + rect.w * 0.5,
                        rect.y + rect.h * 0.5 + scaler.s(4.0),
                        scaler.font_s(11.0),
                        text_col,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_bar_navigation_and_cycling() {
        let mut bar = FilterBar::from_labels(&["CLASSIC", "RALLY", "GT"]);
        assert_eq!(bar.active_idx, 0);

        bar.next();
        assert_eq!(bar.active_idx, 1);
        bar.next();
        assert_eq!(bar.active_idx, 2);
        bar.next();
        assert_eq!(bar.active_idx, 0); // wrap-around

        bar.prev();
        assert_eq!(bar.active_idx, 2);
    }

    #[test]
    fn test_filter_bar_exit_signals() {
        let mut bar = FilterBar::from_labels(&["PRESETS", "CUSTOM"]).with_focus(true);
        // Up should signal ExitUp
        let act = bar.handle_input(false, false, true, false, false, false, false, (0.0, 0.0, 100.0, 30.0), None);
        assert_eq!(act, FilterBarAction::ExitUp);

        // Down should signal ExitDown
        let act = bar.handle_input(false, false, false, true, false, false, false, (0.0, 0.0, 100.0, 30.0), None);
        assert_eq!(act, FilterBarAction::ExitDown);
    }
}
