use macroquad::color::Color;
use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
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

/// An individual checkable option in a ChecklistModal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChecklistItem<T> {
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub data: T,
}

impl<T> ChecklistItem<T> {
    pub fn new(id: impl Into<String>, label: impl Into<String>, checked: bool, data: T) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            checked,
            disabled: false,
            data,
        }
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Action resulting from user input on a ChecklistModal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecklistAction {
    None,
    ItemToggled(usize, bool),
    Confirmed,
    Cancelled,
}

/// Multi-select modal dialog for bulk category promotion, rules, or options.
pub struct ChecklistModal<T> {
    pub title: String,
    pub items: Vec<ChecklistItem<T>>,
    pub focused_idx: usize,
    pub is_open: bool,
    pub base_x: f32,
    pub base_y: f32,
    pub width: f32,
    pub height: f32,
    pub item_height: f32,
    pub gap: f32,
}

impl<T> ChecklistModal<T> {
    /// Creates a new ChecklistModal centered at (base_x, base_y).
    pub fn new(
        title: impl Into<String>,
        base_x: f32,
        base_y: f32,
        width: f32,
        height: f32,
        items: Vec<ChecklistItem<T>>,
    ) -> Self {
        Self {
            title: title.into(),
            items,
            focused_idx: 0,
            is_open: true,
            base_x,
            base_y,
            width,
            height,
            item_height: 36.0,
            gap: 4.0,
        }
    }

    /// Toggles the checked status of item at `idx`.
    pub fn toggle(&mut self, idx: usize) -> bool {
        if let Some(item) = self.items.get_mut(idx) {
            if !item.disabled {
                item.checked = !item.checked;
                return true;
            }
        }
        false
    }

    /// Checks all non-disabled items.
    pub fn select_all(&mut self) {
        for item in &mut self.items {
            if !item.disabled {
                item.checked = true;
            }
        }
    }

    /// Unchecks all non-disabled items.
    pub fn clear_all(&mut self) {
        for item in &mut self.items {
            if !item.disabled {
                item.checked = false;
            }
        }
    }

    /// Number of checked items.
    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|i| i.checked).count()
    }

    /// Returns references to all checked items.
    pub fn selected_items(&self) -> Vec<&ChecklistItem<T>> {
        self.items.iter().filter(|i| i.checked).collect()
    }

    /// Bounding rectangle of the modal dialogue window.
    #[inline]
    pub fn bounds(&self) -> LayoutRect {
        LayoutRect::new(self.base_x, self.base_y, self.width, self.height)
    }

    /// Bounding rectangle of an item row.
    pub fn item_rect(&self, idx: usize) -> LayoutRect {
        let content_y = self.base_y + 60.0; // Header offset
        let y = content_y + (idx as f32) * (self.item_height + self.gap);
        let pad_x = 24.0;
        LayoutRect::new(self.base_x + pad_x, y, self.width - pad_x * 2.0, self.item_height)
    }

    /// Hit-tests a coordinate against item rows.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        for i in 0..self.items.len() {
            if self.item_rect(i).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Navigates up.
    pub fn nav_up(&mut self) {
        if !self.items.is_empty() && self.focused_idx > 0 {
            self.focused_idx -= 1;
        }
    }

    /// Navigates down.
    pub fn nav_down(&mut self) {
        if !self.items.is_empty() && self.focused_idx + 1 < self.items.len() {
            self.focused_idx += 1;
        }
    }

    /// Handles keyboard and mouse input when open.
    pub fn handle_input(&mut self, audio: Option<&dyn CabinetAudioSink>) -> ChecklistAction {
        if !self.is_open || self.items.is_empty() {
            return ChecklistAction::None;
        }

        let mut action = ChecklistAction::None;

        if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) {
            self.nav_up();
            if let Some(audio) = audio {
                audio.play_ui_move();
            }
        } else if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) {
            self.nav_down();
            if let Some(audio) = audio {
                audio.play_ui_move();
            }
        } else if safe_key_pressed(KeyCode::Space) {
            if self.toggle(self.focused_idx) {
                let status = self.items[self.focused_idx].checked;
                action = ChecklistAction::ItemToggled(self.focused_idx, status);
                if let Some(audio) = audio {
                    audio.play_ui_select();
                }
            }
        } else if safe_key_pressed(KeyCode::Enter) {
            self.is_open = false;
            if let Some(audio) = audio {
                audio.play_ui_select();
            }
            return ChecklistAction::Confirmed;
        } else if safe_key_pressed(KeyCode::Escape) {
            self.is_open = false;
            if let Some(audio) = audio {
                audio.play_ui_cancel();
            }
            return ChecklistAction::Cancelled;
        }

        // Mouse click support
        let (mx, my) = safe_mouse_pos();
        if let Some(idx) = self.hit_test(mx, my) {
            if safe_mouse_pressed(MouseButton::Left) {
                self.focused_idx = idx;
                if self.toggle(idx) {
                    let status = self.items[idx].checked;
                    action = ChecklistAction::ItemToggled(idx, status);
                    if let Some(audio) = audio {
                        audio.play_ui_select();
                    }
                }
            }
        }

        action
    }

    /// Standard modal background and row frames rendering.
    pub fn render_frame(&self) {
        if !self.is_open {
            return;
        }

        let b = self.bounds();
        // Modal window glass backdrop
        draw_rectangle(b.x, b.y, b.w, b.h, Palette::UI_CARD_BG);
        draw_rectangle_lines(b.x, b.y, b.w, b.h, 2.0, Palette::UI_CARD_BORDER);

        // Header separator line
        draw_rectangle(b.x, b.y + 50.0, b.w, 1.5, Palette::UI_CARD_BORDER);

        // Items
        for (i, item) in self.items.iter().enumerate() {
            let r = self.item_rect(i);
            let is_focused = i == self.focused_idx;

            let (bg, border, thickness) = if is_focused {
                (
                    Palette::UI_CARD_BG_HOVER,
                    Palette::NEON_GOLD,
                    2.0,
                )
            } else if item.checked {
                (
                    Color::new(0.08, 0.16, 0.24, 0.85),
                    Palette::NEON_CYAN,
                    1.2,
                )
            } else {
                (
                    Color::new(0.05, 0.07, 0.10, 0.60),
                    Palette::UI_CARD_BORDER,
                    1.0,
                )
            };

            draw_rectangle(r.x, r.y, r.w, r.h, bg);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, thickness, border);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checklist_modal_toggle_and_batch_actions() {
        let items = vec![
            ChecklistItem::new("t1", "Track 1", false, 100),
            ChecklistItem::new("t2", "Track 2", true, 200),
            ChecklistItem::new("t3", "Track 3 (Disabled)", false, 300).with_disabled(true),
        ];

        let mut modal = ChecklistModal::new("Promote Tracks", 100.0, 100.0, 500.0, 400.0, items);

        assert_eq!(modal.selected_count(), 1);

        // Toggle t1
        assert!(modal.toggle(0));
        assert!(modal.items[0].checked);
        assert_eq!(modal.selected_count(), 2);

        // Toggle disabled item fails
        assert!(!modal.toggle(2));
        assert!(!modal.items[2].checked);

        // Select All (skips disabled)
        modal.select_all();
        assert_eq!(modal.selected_count(), 2);

        // Clear All
        modal.clear_all();
        assert_eq!(modal.selected_count(), 0);
    }

    #[test]
    fn test_checklist_modal_rects_and_navigation() {
        let items = vec![
            ChecklistItem::new("1", "A", false, ()),
            ChecklistItem::new("2", "B", false, ()),
        ];

        let mut modal = ChecklistModal::new("Title", 50.0, 50.0, 400.0, 300.0, items);
        assert_eq!(modal.focused_idx, 0);

        modal.nav_down();
        assert_eq!(modal.focused_idx, 1);

        modal.nav_down(); // Clamped at end
        assert_eq!(modal.focused_idx, 1);

        modal.nav_up();
        assert_eq!(modal.focused_idx, 0);

        let r0 = modal.item_rect(0);
        assert_eq!(r0, LayoutRect::new(74.0, 110.0, 352.0, 36.0));
        assert!(modal.hit_test(100.0, 120.0).is_some());
    }
}
