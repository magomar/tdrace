use macroquad::color::Color;
use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use crate::audio::CabinetAudioSink;
use crate::ui::layout::{LayoutRect, NavBoundaryExit};
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

/// An individual item in a 2D Card Grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardGridItem<T> {
    pub id: String,
    pub data: T,
    pub disabled: bool,
}

impl<T> CardGridItem<T> {
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Self {
            id: id.into(),
            data,
            disabled: false,
        }
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Action resulting from user input on a CardGrid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardGridAction {
    None,
    Selected(usize),
    Confirmed(usize),
    Exit(NavBoundaryExit),
}

/// 2D Matrix Layout Container with seamless 2D navigation and windowed scrolling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardGrid<T> {
    pub items: Vec<CardGridItem<T>>,
    pub columns: usize,
    pub card_width: f32,
    pub card_height: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub base_x: f32,
    pub base_y: f32,
    pub selected_idx: usize,
    pub is_focused: bool,
    pub wrap_navigation: bool,
}

impl<T> CardGrid<T> {
    /// Creates a new 2D CardGrid container.
    pub fn new(
        base_x: f32,
        base_y: f32,
        columns: usize,
        card_width: f32,
        card_height: f32,
        gap_x: f32,
        gap_y: f32,
    ) -> Self {
        Self {
            items: Vec::new(),
            columns: columns.max(1),
            card_width,
            card_height,
            gap_x,
            gap_y,
            base_x,
            base_y,
            selected_idx: 0,
            is_focused: false,
            wrap_navigation: false,
        }
    }

    /// Appends an item to the grid.
    pub fn add_item(&mut self, item: CardGridItem<T>) {
        self.items.push(item);
    }

    /// Sets the full item collection, ensuring selection is clamped.
    pub fn set_items(&mut self, items: Vec<CardGridItem<T>>) {
        self.items = items;
        self.clamp_selection();
    }

    /// Returns the number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the grid is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clamps current selection to valid bounds.
    pub fn clamp_selection(&mut self) {
        if self.items.is_empty() {
            self.selected_idx = 0;
        } else if self.selected_idx >= self.items.len() {
            self.selected_idx = self.items.len() - 1;
        }
    }

    /// Returns the column and row index for a given item index: `(col, row)`.
    #[inline]
    pub fn col_row_of(&self, idx: usize) -> (usize, usize) {
        let col = idx % self.columns;
        let row = idx / self.columns;
        (col, row)
    }

    /// Returns the item index for a given column and row.
    #[inline]
    pub fn idx_of(&self, col: usize, row: usize) -> usize {
        row * self.columns + col
    }

    /// Total number of rows required to display all items.
    #[inline]
    pub fn total_rows(&self) -> usize {
        if self.items.is_empty() {
            0
        } else {
            (self.items.len() + self.columns - 1) / self.columns
        }
    }

    /// Returns the bounding rectangle of the card at `idx`.
    pub fn item_rect(&self, idx: usize) -> LayoutRect {
        let (col, row) = self.col_row_of(idx);
        let x = self.base_x + (col as f32) * (self.card_width + self.gap_x);
        let y = self.base_y + (row as f32) * (self.card_height + self.gap_y);
        LayoutRect::new(x, y, self.card_width, self.card_height)
    }

    /// Returns the bounding rectangle for windowed scrolling relative to `start_row`.
    pub fn item_rect_windowed(&self, idx: usize, start_row: usize) -> LayoutRect {
        let (col, row) = self.col_row_of(idx);
        let rel_row = row.saturating_sub(start_row);
        let x = self.base_x + (col as f32) * (self.card_width + self.gap_x);
        let y = self.base_y + (rel_row as f32) * (self.card_height + self.gap_y);
        LayoutRect::new(x, y, self.card_width, self.card_height)
    }

    /// Computes the visible row range `(start_row, end_row)` for windowed rendering.
    pub fn visible_row_range(&self, visible_rows: usize) -> (usize, usize) {
        let total = self.total_rows();
        if total <= visible_rows {
            (0, total)
        } else {
            let (_, current_row) = self.col_row_of(self.selected_idx);
            let half = visible_rows / 2;
            let start = current_row.saturating_sub(half).min(total - visible_rows);
            let end = (start + visible_rows).min(total);
            (start, end)
        }
    }

    /// Hit-tests a virtual screen coordinate against all visible cards.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        for i in 0..self.items.len() {
            if self.item_rect(i).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Hit-tests a virtual screen coordinate in a windowed view.
    pub fn hit_test_windowed(
        &self,
        px: f32,
        py: f32,
        start_row: usize,
        visible_rows: usize,
    ) -> Option<usize> {
        let end_row = (start_row + visible_rows).min(self.total_rows());
        let start_idx = start_row * self.columns;
        let end_idx = (end_row * self.columns).min(self.items.len());

        for i in start_idx..end_idx {
            if self.item_rect_windowed(i, start_row).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Navigates Left in 2D space.
    pub fn nav_left(&mut self) -> CardGridAction {
        if self.items.is_empty() {
            return CardGridAction::Exit(NavBoundaryExit::ExitLeft);
        }
        let (col, _) = self.col_row_of(self.selected_idx);
        if col == 0 {
            if self.wrap_navigation {
                // Wrap to end of current row (or last available item)
                let row = self.selected_idx / self.columns;
                let rightmost_col = (self.columns - 1).min(self.items.len() - 1 - row * self.columns);
                self.selected_idx = self.idx_of(rightmost_col, row);
                CardGridAction::Selected(self.selected_idx)
            } else {
                CardGridAction::Exit(NavBoundaryExit::ExitLeft)
            }
        } else {
            self.selected_idx -= 1;
            CardGridAction::Selected(self.selected_idx)
        }
    }

    /// Navigates Right in 2D space.
    pub fn nav_right(&mut self) -> CardGridAction {
        if self.items.is_empty() {
            return CardGridAction::Exit(NavBoundaryExit::ExitRight);
        }
        let (col, row) = self.col_row_of(self.selected_idx);
        let is_last_item = self.selected_idx + 1 >= self.items.len();
        let is_row_end = col + 1 >= self.columns;

        if is_last_item || is_row_end {
            if self.wrap_navigation {
                self.selected_idx = self.idx_of(0, row);
                CardGridAction::Selected(self.selected_idx)
            } else {
                CardGridAction::Exit(NavBoundaryExit::ExitRight)
            }
        } else {
            self.selected_idx += 1;
            CardGridAction::Selected(self.selected_idx)
        }
    }

    /// Navigates Up in 2D space.
    pub fn nav_up(&mut self) -> CardGridAction {
        if self.items.is_empty() {
            return CardGridAction::Exit(NavBoundaryExit::ExitTop);
        }
        let (col, row) = self.col_row_of(self.selected_idx);
        if row == 0 {
            if self.wrap_navigation {
                let last_row = self.total_rows() - 1;
                let target = self.idx_of(col, last_row).min(self.items.len() - 1);
                self.selected_idx = target;
                CardGridAction::Selected(self.selected_idx)
            } else {
                CardGridAction::Exit(NavBoundaryExit::ExitTop)
            }
        } else {
            self.selected_idx = self.idx_of(col, row - 1);
            CardGridAction::Selected(self.selected_idx)
        }
    }

    /// Navigates Down in 2D space.
    pub fn nav_down(&mut self) -> CardGridAction {
        if self.items.is_empty() {
            return CardGridAction::Exit(NavBoundaryExit::ExitBottom);
        }
        let (col, row) = self.col_row_of(self.selected_idx);
        let total_rows = self.total_rows();

        if row + 1 >= total_rows {
            if self.wrap_navigation {
                self.selected_idx = self.idx_of(col, 0).min(self.items.len() - 1);
                CardGridAction::Selected(self.selected_idx)
            } else {
                CardGridAction::Exit(NavBoundaryExit::ExitBottom)
            }
        } else {
            let next_idx = self.idx_of(col, row + 1);
            // If the next row does not have an element in this column, clamp to the last item
            self.selected_idx = next_idx.min(self.items.len() - 1);
            CardGridAction::Selected(self.selected_idx)
        }
    }

    /// Processes keyboard, mouse, and directional events when focused.
    pub fn handle_input(&mut self, audio: Option<&dyn CabinetAudioSink>) -> CardGridAction {
        if !self.is_focused || self.items.is_empty() {
            return CardGridAction::None;
        }

        let mut action = CardGridAction::None;

        if safe_key_pressed(KeyCode::Left) || safe_key_pressed(KeyCode::A) {
            action = self.nav_left();
        } else if safe_key_pressed(KeyCode::Right) || safe_key_pressed(KeyCode::D) {
            action = self.nav_right();
        } else if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) {
            action = self.nav_up();
        } else if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) {
            action = self.nav_down();
        } else if safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::Space) {
            action = CardGridAction::Confirmed(self.selected_idx);
        }

        // Mouse hover and click support
        let (mx, my) = safe_mouse_pos();
        if let Some(hovered) = self.hit_test(mx, my) {
            if hovered != self.selected_idx {
                self.selected_idx = hovered;
                action = CardGridAction::Selected(hovered);
            }
            if safe_mouse_pressed(MouseButton::Left) {
                action = CardGridAction::Confirmed(hovered);
            }
        }

        if let Some(audio) = audio {
            match action {
                CardGridAction::Selected(_) => audio.play_ui_move(),
                CardGridAction::Confirmed(_) => audio.play_ui_select(),
                _ => {}
            }
        }

        action
    }

    /// Standard rendering helper for card backgrounds and focus border.
    pub fn render_card_frame(
        &self,
        rect: &LayoutRect,
        is_selected: bool,
        is_disabled: bool,
    ) {
        let (bg, border, thickness) = if is_selected && self.is_focused {
            (
                Color::new(0.25, 0.20, 0.05, 0.95),
                Palette::NEON_GOLD,
                2.4,
            )
        } else if is_selected {
            (
                Palette::UI_CARD_BG_HOVER,
                Palette::NEON_CYAN,
                1.5,
            )
        } else if is_disabled {
            (
                Color::new(0.08, 0.08, 0.10, 0.4),
                Palette::UI_CARD_BORDER,
                1.0,
            )
        } else {
            (
                Palette::UI_CARD_BG,
                Palette::UI_CARD_BORDER,
                1.0,
            )
        };

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, border);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_grid_col_row_and_indices() {
        let mut grid: CardGrid<String> = CardGrid::new(100.0, 50.0, 3, 80.0, 100.0, 10.0, 15.0);
        for i in 0..8 {
            grid.add_item(CardGridItem::new(format!("item_{i}"), format!("data_{i}")));
        }

        assert_eq!(grid.len(), 8);
        assert_eq!(grid.total_rows(), 3); // 8 items in 3 cols -> 3 rows (3, 3, 2)
        assert_eq!(grid.col_row_of(0), (0, 0));
        assert_eq!(grid.col_row_of(2), (2, 0));
        assert_eq!(grid.col_row_of(3), (0, 1));
        assert_eq!(grid.col_row_of(7), (1, 2));

        assert_eq!(grid.idx_of(0, 0), 0);
        assert_eq!(grid.idx_of(2, 0), 2);
        assert_eq!(grid.idx_of(0, 1), 3);
        assert_eq!(grid.idx_of(1, 2), 7);
    }

    #[test]
    fn test_card_grid_rect_math() {
        let mut grid: CardGrid<i32> = CardGrid::new(50.0, 20.0, 2, 100.0, 60.0, 10.0, 10.0);
        grid.add_item(CardGridItem::new("a", 1));
        grid.add_item(CardGridItem::new("b", 2));
        grid.add_item(CardGridItem::new("c", 3));

        let rect0 = grid.item_rect(0);
        assert_eq!(rect0, LayoutRect::new(50.0, 20.0, 100.0, 60.0));

        let rect1 = grid.item_rect(1);
        assert_eq!(rect1, LayoutRect::new(160.0, 20.0, 100.0, 60.0));

        let rect2 = grid.item_rect(2);
        assert_eq!(rect2, LayoutRect::new(50.0, 90.0, 100.0, 60.0));

        assert_eq!(grid.hit_test(60.0, 30.0), Some(0));
        assert_eq!(grid.hit_test(170.0, 30.0), Some(1));
        assert_eq!(grid.hit_test(60.0, 100.0), Some(2));
        assert_eq!(grid.hit_test(10.0, 10.0), None);
    }

    #[test]
    fn test_card_grid_2d_orthogonal_navigation() {
        let mut grid: CardGrid<&'static str> = CardGrid::new(0.0, 0.0, 3, 50.0, 50.0, 5.0, 5.0);
        for i in 0..7 {
            grid.add_item(CardGridItem::new(format!("c_{i}"), "val"));
        }
        // Row 0: 0, 1, 2
        // Row 1: 3, 4, 5
        // Row 2: 6 (col 0)

        // Start at 0, moving left emits ExitLeft
        assert_eq!(grid.nav_left(), CardGridAction::Exit(NavBoundaryExit::ExitLeft));
        assert_eq!(grid.selected_idx, 0);

        // Move right to 1, then 2, then ExitRight
        assert_eq!(grid.nav_right(), CardGridAction::Selected(1));
        assert_eq!(grid.nav_right(), CardGridAction::Selected(2));
        assert_eq!(grid.nav_right(), CardGridAction::Exit(NavBoundaryExit::ExitRight));

        // Up at row 0 emits ExitTop
        assert_eq!(grid.nav_up(), CardGridAction::Exit(NavBoundaryExit::ExitTop));

        // Down from (col 2, row 0) moves to (col 2, row 1) = index 5
        assert_eq!(grid.nav_down(), CardGridAction::Selected(5));
        assert_eq!(grid.selected_idx, 5);

        // Down from (col 2, row 1) targets row 2. Row 2 only has index 6 (col 0).
        // It clamps to the last available item (6)!
        assert_eq!(grid.nav_down(), CardGridAction::Selected(6));
        assert_eq!(grid.selected_idx, 6);

        // Down at bottom emits ExitBottom
        assert_eq!(grid.nav_down(), CardGridAction::Exit(NavBoundaryExit::ExitBottom));

        // Up from 6 moves to row 1, col 0 = index 3
        assert_eq!(grid.nav_up(), CardGridAction::Selected(3));
        assert_eq!(grid.selected_idx, 3);
    }

    #[test]
    fn test_card_grid_wrap_navigation() {
        let mut grid: CardGrid<()> = CardGrid::new(0.0, 0.0, 2, 40.0, 40.0, 0.0, 0.0);
        grid.wrap_navigation = true;
        grid.add_item(CardGridItem::new("0", ()));
        grid.add_item(CardGridItem::new("1", ()));
        grid.add_item(CardGridItem::new("2", ()));
        grid.add_item(CardGridItem::new("3", ()));

        // Left from 0 wraps to 1
        assert_eq!(grid.nav_left(), CardGridAction::Selected(1));
        // Right from 1 wraps to 0
        assert_eq!(grid.nav_right(), CardGridAction::Selected(0));

        // Up from 0 wraps to 2
        assert_eq!(grid.nav_up(), CardGridAction::Selected(2));
        // Down from 2 wraps to 0
        assert_eq!(grid.nav_down(), CardGridAction::Selected(0));
    }
}
