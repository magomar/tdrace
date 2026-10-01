use macroquad::color::Color;
use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};

use crate::audio::CabinetAudioSink;
use crate::ui::layout::LayoutRect;
use crate::ui::{Fonts, UiScaler};
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

/// Horizontal text alignment for a table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColumnAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Definition of a single column in a DataTable.
#[derive(Clone)]
pub struct DataColumn<T> {
    pub id: String,
    pub header: String,
    pub width: f32,
    pub align: ColumnAlign,
    pub extractor: fn(&T) -> String,
}

impl<T> DataColumn<T> {
    pub fn new(
        id: impl Into<String>,
        header: impl Into<String>,
        width: f32,
        align: ColumnAlign,
        extractor: fn(&T) -> String,
    ) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width,
            align,
            extractor,
        }
    }
}

/// A row in a DataTable with optional rank badge and player identification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataRow<T> {
    pub id: String,
    pub is_player: bool,
    pub rank: Option<usize>,
    pub data: T,
}

impl<T> DataRow<T> {
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Self {
            id: id.into(),
            is_player: false,
            rank: None,
            data,
        }
    }

    pub fn with_rank(mut self, rank: usize) -> Self {
        self.rank = Some(rank);
        self
    }

    pub fn with_player(mut self, is_player: bool) -> Self {
        self.is_player = is_player;
        self
    }
}

/// Action resulting from user input on a DataTable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAction {
    None,
    RowSelected(usize),
    RowConfirmed(usize),
    SortChanged(usize),
    ExitTop,
    ExitBottom,
}

/// Reusable multi-column data and leaderboard table component.
pub struct DataTable<T> {
    pub columns: Vec<DataColumn<T>>,
    pub rows: Vec<DataRow<T>>,
    pub selected_row: usize,
    pub is_focused: bool,
    pub sort_col_idx: Option<usize>,
    pub sort_ascending: bool,
    pub base_x: f32,
    pub base_y: f32,
    pub total_width: f32,
    pub row_height: f32,
    pub header_height: f32,
    pub gap: f32,
}

impl<T> DataTable<T> {
    /// Creates a new DataTable.
    pub fn new(
        base_x: f32,
        base_y: f32,
        total_width: f32,
        row_height: f32,
        header_height: f32,
    ) -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            selected_row: 0,
            is_focused: false,
            sort_col_idx: None,
            sort_ascending: true,
            base_x,
            base_y,
            total_width,
            row_height,
            header_height,
            gap: 2.0,
        }
    }

    /// Adds a column definition to the table.
    pub fn add_column(&mut self, col: DataColumn<T>) {
        self.columns.push(col);
    }

    /// Sets the rows collection and clamps current selection.
    pub fn set_rows(&mut self, rows: Vec<DataRow<T>>) {
        self.rows = rows;
        self.clamp_selection();
    }

    /// Number of rows in the table.
    #[inline]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns true if there are no rows.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Clamps selection within row bounds.
    pub fn clamp_selection(&mut self) {
        if self.rows.is_empty() {
            self.selected_row = 0;
        } else if self.selected_row >= self.rows.len() {
            self.selected_row = self.rows.len() - 1;
        }
    }

    /// Returns the bounding rectangle of the header row.
    pub fn header_rect(&self) -> LayoutRect {
        LayoutRect::new(self.base_x, self.base_y, self.total_width, self.header_height)
    }

    /// Returns the bounding rectangle of the row at `idx`.
    pub fn row_rect(&self, idx: usize) -> LayoutRect {
        let y = self.base_y + self.header_height + self.gap + (idx as f32) * (self.row_height + self.gap);
        LayoutRect::new(self.base_x, y, self.total_width, self.row_height)
    }

    /// Returns the bounding rectangle for windowed rendering relative to `start_idx`.
    pub fn row_rect_windowed(&self, idx: usize, start_idx: usize) -> LayoutRect {
        let rel_idx = idx.saturating_sub(start_idx);
        let y = self.base_y + self.header_height + self.gap + (rel_idx as f32) * (self.row_height + self.gap);
        LayoutRect::new(self.base_x, y, self.total_width, self.row_height)
    }

    /// Computes visible row indices `(start_idx, end_idx)` centered around `selected_row`.
    pub fn visible_range(&self, visible_count: usize) -> (usize, usize) {
        let total = self.rows.len();
        if total <= visible_count {
            (0, total)
        } else {
            let half = visible_count / 2;
            let start = self.selected_row.saturating_sub(half).min(total - visible_count);
            let end = (start + visible_count).min(total);
            (start, end)
        }
    }

    /// Hit-tests a point against rows up to count.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        if px < self.base_x || px > self.base_x + self.total_width {
            return None;
        }
        for i in 0..self.rows.len() {
            if self.row_rect(i).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Navigates up. At row 0, returns `TableAction::ExitTop` to allow clean focus exit.
    pub fn nav_up(&mut self) -> TableAction {
        if self.rows.is_empty() || self.selected_row == 0 {
            TableAction::ExitTop
        } else {
            self.selected_row -= 1;
            TableAction::RowSelected(self.selected_row)
        }
    }

    /// Navigates down. At last row, returns `TableAction::ExitBottom` to allow clean focus exit.
    pub fn nav_down(&mut self) -> TableAction {
        if self.rows.is_empty() || self.selected_row + 1 >= self.rows.len() {
            TableAction::ExitBottom
        } else {
            self.selected_row += 1;
            TableAction::RowSelected(self.selected_row)
        }
    }

    /// Processes keyboard, mouse, and directional events when focused.
    pub fn handle_input(&mut self, audio: Option<&dyn CabinetAudioSink>) -> TableAction {
        if !self.is_focused || self.rows.is_empty() {
            return TableAction::None;
        }

        let mut action = TableAction::None;

        if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) {
            action = self.nav_up();
        } else if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) {
            action = self.nav_down();
        } else if safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::Space) {
            action = TableAction::RowConfirmed(self.selected_row);
        }

        // Mouse hit test
        let (mx, my) = safe_mouse_pos();
        if let Some(hovered) = self.hit_test(mx, my) {
            if hovered != self.selected_row {
                self.selected_row = hovered;
                action = TableAction::RowSelected(hovered);
            }
            if safe_mouse_pressed(MouseButton::Left) {
                action = TableAction::RowConfirmed(hovered);
            }
        }

        if let Some(audio) = audio {
            match action {
                TableAction::RowSelected(_) => audio.play_ui_move(),
                TableAction::RowConfirmed(_) => audio.play_ui_select(),
                _ => {}
            }
        }

        action
    }

    /// Color token for rank badge.
    pub fn rank_badge_color(rank: usize) -> Color {
        match rank {
            1 => Palette::NEON_GOLD,
            2 => Color::new(0.85, 0.88, 0.92, 1.0), // Silver
            3 => Color::new(0.80, 0.50, 0.20, 1.0), // Bronze
            _ => Palette::UI_TEXT_MUTED,
        }
    }

    /// Standard rendering helper for row background and focus accent.
    pub fn render_row_frame(&self, rect: &LayoutRect, is_selected: bool, is_player: bool) {
        let (bg, border, thickness) = if is_selected && self.is_focused {
            (
                Color::new(0.25, 0.20, 0.05, 0.95),
                Palette::NEON_GOLD,
                2.4,
            )
        } else if is_player {
            (
                Color::new(0.08, 0.18, 0.28, 0.90),
                Palette::NEON_CYAN,
                1.5,
            )
        } else if is_selected {
            (
                Palette::UI_CARD_BG_HOVER,
                Palette::UI_CARD_BORDER_GLOW,
                1.2,
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

    /// Renders a read-only table within a bounded viewport. Does not reorder rows or
    /// highlight selected_row unless is_focused is explicitly enabled by the caller.
    /// Row heights shrink to fit the supplied viewport; callers own scroll state.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts, bounds: LayoutRect) {
        if self.columns.is_empty() || bounds.w <= 0.0 || bounds.h <= 0.0 {
            return;
        }
        let (header_h, row_h) = self.display_heights(bounds.h);
        let header = LayoutRect::new(bounds.x, bounds.y, bounds.w, header_h);
        draw_rectangle(header.x, header.y, header.w, header.h, Palette::UI_CARD_BG_HOVER);
        let size = scaler.font_s(14.0).min(row_h * 0.55).min(header_h * 0.55);
        if size <= 0.0 {
            return;
        }
        self.draw_cells(fonts, header, size, None);
        for (idx, row) in self.rows.iter().enumerate() {
            let rect = LayoutRect::new(bounds.x, bounds.y + header_h + idx as f32 * row_h, bounds.w, row_h);
            self.render_row_frame(&rect, self.is_focused && idx == self.selected_row, row.is_player);
            self.draw_cells(fonts, rect, size, Some(row));
        }
    }

    fn display_heights(&self, height: f32) -> (f32, f32) {
        let header_h = self.header_height.max(0.0).min(height.max(0.0));
        let row_h = self.row_height.max(0.0).min((height - header_h).max(0.0) / self.rows.len().max(1) as f32);
        (header_h, row_h)
    }

    fn draw_cells(&self, fonts: &Fonts, rect: LayoutRect, size: f32, row: Option<&DataRow<T>>) {
        let total: f32 = self.columns.iter().map(|col| col.width.max(0.0)).sum();
        if total <= 0.0 {
            return;
        }
        let mut x = rect.x;
        for col in &self.columns {
            let width = rect.w * col.width.max(0.0) / total;
            let pad = (size * 0.5).min(width * 0.25);
            let text = match row {
                Some(row) => (col.extractor)(&row.data),
                None => col.header.clone(),
            };
            let text = fonts.fit_ui_bold(&text, size, (width - 2.0 * pad).max(0.0));
            let text_w = fonts.measure_ui_bold(&text, size).width;
            let text_x = match col.align {
                ColumnAlign::Left => x + pad,
                ColumnAlign::Center => x + (width - text_w) * 0.5,
                ColumnAlign::Right => x + width - pad - text_w,
            };
            let color = match row {
                Some(row) if col.id == "pos" => row.rank.map(Self::rank_badge_color).unwrap_or(Palette::WHITE),
                Some(row) if row.is_player => Palette::NEON_GOLD,
                _ => Palette::WHITE,
            };
            fonts.draw_ui_bold(&text, text_x, rect.y + rect.h * 0.5 + size * 0.35, size, color);
            x += width;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Participant {
        name: String,
        lap_time: String,
    }

    #[test]
    fn display_rows_fit_bounded_viewport_without_reordering() {
        let mut table = DataTable::new(0.0, 0.0, 600.0, 32.0, 28.0);
        table.set_rows((0..32).map(|i| DataRow::new(i.to_string(), i)).collect());
        for height in [160.0, 240.0, 480.0] {
            let (header, row) = table.display_heights(height);
            assert!(header + row * table.len() as f32 <= height + 0.001);
            assert!(row > 0.0 && row <= 32.0);
        }
        assert_eq!(table.rows[0].data, 0);
        assert_eq!(table.rows[31].data, 31);
        assert!(!table.is_focused);
        assert_eq!(table.display_heights(0.0), (0.0, 0.0));
        table.set_rows(Vec::new());
        assert_eq!(table.display_heights(100.0), (28.0, 32.0));
    }

    #[test]
    fn test_data_table_layout_and_rects() {
        let mut table = DataTable::<Participant>::new(100.0, 50.0, 600.0, 30.0, 25.0);
        table.add_column(DataColumn::new("pos", "POS", 60.0, ColumnAlign::Center, |_| "1".to_string()));
        table.add_column(DataColumn::new("name", "DRIVER", 300.0, ColumnAlign::Left, |p| p.name.clone()));
        table.add_column(DataColumn::new("time", "BEST LAP", 240.0, ColumnAlign::Right, |p| p.lap_time.clone()));

        table.set_rows(vec![
            DataRow::new("p1", Participant { name: "Ayrton".into(), lap_time: "1:12.345".into() }).with_rank(1),
            DataRow::new("p2", Participant { name: "Alain".into(), lap_time: "1:12.512".into() }).with_rank(2),
            DataRow::new("p3", Participant { name: "Nigel".into(), lap_time: "1:13.001".into() }).with_rank(3),
        ]);

        assert_eq!(table.len(), 3);
        assert_eq!(table.header_rect(), LayoutRect::new(100.0, 50.0, 600.0, 25.0));

        let r0 = table.row_rect(0);
        assert_eq!(r0, LayoutRect::new(100.0, 77.0, 600.0, 30.0));

        let r1 = table.row_rect(1);
        assert_eq!(r1, LayoutRect::new(100.0, 109.0, 600.0, 30.0));

        assert_eq!(table.hit_test(150.0, 80.0), Some(0));
        assert_eq!(table.hit_test(150.0, 110.0), Some(1));
        assert_eq!(table.hit_test(50.0, 80.0), None);
    }

    #[test]
    fn test_data_table_navigation_and_boundary_exits() {
        let mut table = DataTable::<String>::new(0.0, 0.0, 400.0, 20.0, 20.0);
        table.set_rows(vec![
            DataRow::new("1", "First".into()),
            DataRow::new("2", "Second".into()),
            DataRow::new("3", "Third".into()),
        ]);

        // Start at 0, nav_up emits ExitTop
        assert_eq!(table.nav_up(), TableAction::ExitTop);
        assert_eq!(table.selected_row, 0);

        // Move down to 1, then 2
        assert_eq!(table.nav_down(), TableAction::RowSelected(1));
        assert_eq!(table.nav_down(), TableAction::RowSelected(2));
        assert_eq!(table.selected_row, 2);

        // Down from last row emits ExitBottom
        assert_eq!(table.nav_down(), TableAction::ExitBottom);
        assert_eq!(table.selected_row, 2);

        // Up from 2 returns to 1
        assert_eq!(table.nav_up(), TableAction::RowSelected(1));
        assert_eq!(table.selected_row, 1);
    }

    #[test]
    fn test_rank_badge_colors() {
        assert_eq!(DataTable::<()>::rank_badge_color(1), Palette::NEON_GOLD);
        assert_eq!(DataTable::<()>::rank_badge_color(2), Color::new(0.85, 0.88, 0.92, 1.0));
        assert_eq!(DataTable::<()>::rank_badge_color(3), Color::new(0.80, 0.50, 0.20, 1.0));
        assert_eq!(DataTable::<()>::rank_badge_color(4), Palette::UI_TEXT_MUTED);
    }
}
