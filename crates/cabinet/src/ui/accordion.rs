use macroquad::color::Color;
use macroquad::input::{is_key_pressed, KeyCode};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

use crate::audio::CabinetAudioSink;
use crate::ui::font::Fonts;
use crate::ui::layout::LayoutRect;
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

#[inline]
fn safe_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(|| is_key_pressed(key)).unwrap_or(false)
}

/// Navigation action returned after processing user input on an Accordion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionNavAction {
    None,
    /// Selected index changed (Up / Down / Stick / D-pad).
    Selected(usize),
    /// Item was expanded or collapsed (Enter / Space / Gamepad A).
    Toggled(usize, bool),
    /// Reached top of list and requested to exit up.
    ExitTop,
    /// Reached bottom of list and requested to exit down.
    ExitBottom,
}

/// An individual collapsible row/card within an Accordion.
#[derive(Debug, Clone, PartialEq)]
pub struct AccordionItem<T = ()> {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tag: Option<String>,
    pub is_expanded: bool,
    pub accent_color: Color,
    pub data: T,
}

impl<T: Default> AccordionItem<T> {
    pub fn new(id: impl Into<String>, title: impl Into<String>, accent_color: Color) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            tag: None,
            is_expanded: false,
            accent_color,
            data: T::default(),
        }
    }
}

/// Reusable gamepad and keyboard-friendly Accordion component.
#[derive(Debug, Clone, PartialEq)]
pub struct Accordion<T = ()> {
    pub items: Vec<AccordionItem<T>>,
    pub selected_idx: usize,
    pub is_focused: bool,
    pub collapsed_h: f32,
    pub expanded_h: f32,
    pub gap: f32,
    /// If true, only one item can be expanded at a time (expanding an item collapses others).
    pub single_expand: bool,
}

impl<T: Default> Accordion<T> {
    pub fn new(items: Vec<AccordionItem<T>>, collapsed_h: f32, expanded_h: f32, gap: f32) -> Self {
        Self {
            items,
            selected_idx: 0,
            is_focused: true,
            collapsed_h,
            expanded_h,
            gap,
            single_expand: true,
        }
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn with_single_expand(mut self, single: bool) -> Self {
        self.single_expand = single;
        self
    }

    pub fn with_selected(mut self, idx: usize) -> Self {
        if !self.items.is_empty() {
            self.selected_idx = idx.min(self.items.len() - 1);
            if self.single_expand {
                for (i, item) in self.items.iter_mut().enumerate() {
                    item.is_expanded = i == self.selected_idx;
                }
            }
        }
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

    /// Toggles expanded state of selected item.
    pub fn toggle_selected(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        let currently_expanded = self.items[self.selected_idx].is_expanded;
        let new_state = !currently_expanded;

        if self.single_expand {
            for (i, item) in self.items.iter_mut().enumerate() {
                item.is_expanded = i == self.selected_idx && new_state;
            }
        } else {
            self.items[self.selected_idx].is_expanded = new_state;
        }

        new_state
    }

    /// Computes the bounding rectangle for each item given top coordinate `top_y`, screen height `sh`,
    /// and bottom padding limit. Returns `Vec<(index, LayoutRect, is_expanded)>`.
    pub fn compute_rects(
        &self,
        x: f32,
        w: f32,
        top_y: f32,
        bottom_limit: f32,
    ) -> (Vec<(usize, LayoutRect, bool)>, f32) {
        if self.items.is_empty() {
            return (Vec::new(), 0.0);
        }

        // Calculate raw y positions
        let mut cur_y = top_y;
        let mut raw_rects = Vec::with_capacity(self.items.len());

        for (idx, item) in self.items.iter().enumerate() {
            let is_exp = if self.single_expand {
                idx == self.selected_idx && item.is_expanded
            } else {
                item.is_expanded
            };
            let h = if is_exp { self.expanded_h } else { self.collapsed_h };
            raw_rects.push((idx, LayoutRect::new(x, cur_y, w, h), is_exp));
            cur_y += h + self.gap;
        }

        // Compute viewport scrolling to keep selected row visible
        let selected_rect = raw_rects.get(self.selected_idx).map(|r| r.1).unwrap_or_default();
        let selected_bottom = selected_rect.y + selected_rect.h;
        let scroll_y = if selected_bottom > bottom_limit {
            selected_bottom - bottom_limit
        } else {
            0.0
        };

        // Apply scroll offset
        let final_rects = raw_rects
            .into_iter()
            .map(|(idx, rect, is_exp)| {
                (
                    idx,
                    LayoutRect::new(rect.x, rect.y - scroll_y, rect.w, rect.h),
                    is_exp,
                )
            })
            .collect();

        (final_rects, scroll_y)
    }

    /// Processes keyboard, gamepad, and mouse navigation.
    pub fn handle_input(
        &mut self,
        gamepad_up: bool,
        gamepad_down: bool,
        gamepad_toggle: bool,
        audio: Option<&dyn CabinetAudioSink>,
    ) -> AccordionNavAction {
        if self.items.is_empty() {
            return AccordionNavAction::None;
        }

        if !self.is_focused {
            return AccordionNavAction::None;
        }

        // Up navigation
        if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) || gamepad_up {
            if self.selected_idx == 0 {
                return AccordionNavAction::ExitTop;
            } else {
                self.selected_idx -= 1;
                if self.single_expand {
                    for (i, item) in self.items.iter_mut().enumerate() {
                        item.is_expanded = i == self.selected_idx;
                    }
                }
                if let Some(a) = audio { a.play_ui_move(); }
                return AccordionNavAction::Selected(self.selected_idx);
            }
        }

        // Down navigation
        if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) || gamepad_down {
            if self.selected_idx + 1 >= self.items.len() {
                return AccordionNavAction::ExitBottom;
            } else {
                self.selected_idx += 1;
                if self.single_expand {
                    for (i, item) in self.items.iter_mut().enumerate() {
                        item.is_expanded = i == self.selected_idx;
                    }
                }
                if let Some(a) = audio { a.play_ui_move(); }
                return AccordionNavAction::Selected(self.selected_idx);
            }
        }

        // Toggle Expand/Collapse (Enter / Space / Gamepad A)
        if safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::Space) || gamepad_toggle {
            let state = self.toggle_selected();
            if let Some(a) = audio { a.play_ui_select(); }
            return AccordionNavAction::Toggled(self.selected_idx, state);
        }

        AccordionNavAction::None
    }

    /// Renders an individual accordion header item.
    pub fn draw_header(
        &self,
        scaler: &UiScaler,
        fonts: &Fonts,
        rect: &LayoutRect,
        item: &AccordionItem<T>,
        is_selected: bool,
        is_expanded: bool,
    ) {
        let bg_color = if is_selected {
            Color::new(0.10, 0.13, 0.21, 0.97)
        } else {
            Color::new(0.06, 0.08, 0.12, 0.85)
        };

        let border_col = if is_selected && self.is_focused {
            Palette::NEON_GOLD
        } else if is_selected {
            item.accent_color
        } else {
            Palette::UI_CARD_BORDER
        };

        let border_w = if is_selected { 2.0 } else { 1.0 };
        scaler.draw_glass_card(rect.x, rect.y, rect.w, rect.h, bg_color, border_col, border_w);

        // Left accent stripe
        let stripe_w = scaler.s(4.0);
        draw_rectangle(rect.x, rect.y, stripe_w, rect.h, item.accent_color);

        // Chevron indicator
        let chevron = if is_expanded { "▼" } else { "►" };
        fonts.draw_ui_bold(
            chevron,
            rect.x + scaler.s(16.0),
            rect.y + scaler.s(22.0),
            scaler.font_s(12.0),
            item.accent_color,
        );

        // Tag badge if present
        let mut text_x = rect.x + scaler.s(36.0);
        if let Some(ref tag) = item.tag {
            let badge_w = scaler.s(70.0);
            let badge_h = scaler.s(20.0);
            let badge_y = rect.y + scaler.s(10.0);
            draw_rectangle(
                text_x,
                badge_y,
                badge_w,
                badge_h,
                Color::new(item.accent_color.r, item.accent_color.g, item.accent_color.b, 0.20),
            );
            draw_rectangle_lines(text_x, badge_y, badge_w, badge_h, 1.0, item.accent_color);
            fonts.draw_ui_bold_centered(
                tag,
                text_x + badge_w * 0.5,
                badge_y + scaler.s(14.0),
                scaler.font_s(10.0),
                item.accent_color,
            );
            text_x += badge_w + scaler.s(12.0);
        }

        // Title
        fonts.draw_display(
            &item.title,
            text_x,
            rect.y + scaler.s(22.0),
            scaler.font_s(14.0),
            if is_selected { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
        );

        // Subtitle if present
        if let Some(ref sub) = item.subtitle {
            fonts.draw_ui_regular(
                sub,
                text_x,
                rect.y + scaler.s(38.0),
                scaler.font_s(10.5),
                Palette::UI_TEXT_MUTED,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accordion_navigation_and_selection() {
        let items = vec![
            AccordionItem::<()>::new("1", "Tier 1", Palette::NEON_CYAN),
            AccordionItem::<()>::new("2", "Tier 2", Palette::NEON_GOLD),
            AccordionItem::<()>::new("3", "Tier 3", Palette::RED),
        ];
        let mut acc = Accordion::new(items, 50.0, 150.0, 8.0).with_selected(0);
        assert_eq!(acc.selected_idx, 0);
        assert!(acc.items[0].is_expanded);

        // Navigation down
        let act = acc.handle_input(false, true, false, None);
        assert_eq!(act, AccordionNavAction::Selected(1));
        assert_eq!(acc.selected_idx, 1);
        assert!(acc.items[1].is_expanded);
        assert!(!acc.items[0].is_expanded);

        // Boundary exit top
        acc.selected_idx = 0;
        let act = acc.handle_input(true, false, false, None);
        assert_eq!(act, AccordionNavAction::ExitTop);
    }

    #[test]
    fn test_accordion_layout_rects_and_scrolling() {
        let items = vec![
            AccordionItem::<()>::new("1", "A", Palette::NEON_CYAN),
            AccordionItem::<()>::new("2", "B", Palette::NEON_GOLD),
        ];
        let acc = Accordion::new(items, 50.0, 150.0, 10.0).with_selected(0);
        let (rects, scroll) = acc.compute_rects(0.0, 200.0, 0.0, 500.0);
        assert_eq!(scroll, 0.0);
        assert_eq!(rects[0].1.h, 150.0); // Expanded
        assert_eq!(rects[1].1.h, 50.0);  // Collapsed
        assert_eq!(rects[1].1.y, 160.0); // 150 + 10 gap
    }
}
