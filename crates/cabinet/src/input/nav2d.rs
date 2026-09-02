use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use serde::{Deserialize, Serialize};

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

/// 2D Orthogonal Navigation Router managing panel focus and item selection.
/// Standard Rule:
/// - Horizontal Axis (Left/Right, A/D, D-pad X): Moves between Panels, Columns, Tabs, or Button Rows.
/// - Vertical Axis (Up/Down, W/S, D-pad Y): Moves within the active Panel, Column, or Card list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavGrid2D {
    /// Total number of columns / panels.
    pub num_columns: usize,
    /// Number of items / cards in each column.
    pub column_lengths: Vec<usize>,
    /// Currently focused column index.
    pub focused_col: usize,
    /// Currently selected row/item index within each column.
    pub cursor_rows: Vec<usize>,
    /// Whether column switching wraps around at edges.
    pub wrap_horizontal: bool,
    /// Whether row switching wraps around at edges.
    pub wrap_vertical: bool,
}

impl NavGrid2D {
    /// Creates a new 2D navigation grid with specified item counts per column.
    pub fn new(column_lengths: Vec<usize>) -> Self {
        let num_columns = column_lengths.len().max(1);
        let cursor_rows = vec![0; num_columns];
        Self {
            num_columns,
            column_lengths,
            focused_col: 0,
            cursor_rows,
            wrap_horizontal: true,
            wrap_vertical: true,
        }
    }

    /// Sets the length of a specific column dynamically (e.g. roster size changing).
    pub fn set_column_len(&mut self, col: usize, len: usize) {
        if col < self.column_lengths.len() {
            self.column_lengths[col] = len.max(1);
            if self.cursor_rows[col] >= self.column_lengths[col] {
                self.cursor_rows[col] = self.column_lengths[col].saturating_sub(1);
            }
        }
    }

    /// Returns the currently active (column, row) coordinate.
    #[inline]
    pub fn active_cell(&self) -> (usize, usize) {
        let row = self.cursor_rows.get(self.focused_col).copied().unwrap_or(0);
        (self.focused_col, row)
    }

    /// Returns the active row in the currently focused column.
    #[inline]
    pub fn active_row(&self) -> usize {
        self.cursor_rows.get(self.focused_col).copied().unwrap_or(0)
    }

    /// Manually sets the active column and row.
    pub fn set_focus(&mut self, col: usize, row: usize) {
        if col < self.num_columns {
            self.focused_col = col;
            let max_row = self.column_lengths.get(col).copied().unwrap_or(1).max(1);
            self.cursor_rows[col] = row.min(max_row - 1);
        }
    }

    /// Moves focus horizontally to the left.
    pub fn move_left(&mut self) -> bool {
        if self.num_columns <= 1 {
            return false;
        }
        if self.focused_col == 0 {
            if self.wrap_horizontal {
                self.focused_col = self.num_columns - 1;
                true
            } else {
                false
            }
        } else {
            self.focused_col -= 1;
            true
        }
    }

    /// Moves focus horizontally to the right.
    pub fn move_right(&mut self) -> bool {
        if self.num_columns <= 1 {
            return false;
        }
        if self.focused_col + 1 >= self.num_columns {
            if self.wrap_horizontal {
                self.focused_col = 0;
                true
            } else {
                false
            }
        } else {
            self.focused_col += 1;
            true
        }
    }

    /// Moves selection vertically up within the active column.
    pub fn move_up(&mut self) -> bool {
        let max_rows = self.column_lengths.get(self.focused_col).copied().unwrap_or(1);
        if max_rows <= 1 {
            return false;
        }
        let current_row = &mut self.cursor_rows[self.focused_col];
        if *current_row == 0 {
            if self.wrap_vertical {
                *current_row = max_rows - 1;
                true
            } else {
                false
            }
        } else {
            *current_row -= 1;
            true
        }
    }

    /// Moves selection vertically down within the active column.
    pub fn move_down(&mut self) -> bool {
        let max_rows = self.column_lengths.get(self.focused_col).copied().unwrap_or(1);
        if max_rows <= 1 {
            return false;
        }
        let current_row = &mut self.cursor_rows[self.focused_col];
        if *current_row + 1 >= max_rows {
            if self.wrap_vertical {
                *current_row = 0;
                true
            } else {
                false
            }
        } else {
            *current_row += 1;
            true
        }
    }

    /// Processes keyboard and optional gamepad directional triggers.
    /// Returns `true` if any navigation focus changed.
    pub fn handle_standard_inputs(
        &mut self,
        gamepad_nav_left: bool,
        gamepad_nav_right: bool,
        gamepad_nav_up: bool,
        gamepad_nav_down: bool,
    ) -> bool {
        let mut changed = false;

        // Horizontal navigation
        if safe_key_pressed(KeyCode::Left)
            || safe_key_pressed(KeyCode::A)
            || gamepad_nav_left
        {
            changed |= self.move_left();
        }
        if safe_key_pressed(KeyCode::Right)
            || safe_key_pressed(KeyCode::D)
            || gamepad_nav_right
        {
            changed |= self.move_right();
        }

        // Vertical navigation
        if safe_key_pressed(KeyCode::Up)
            || safe_key_pressed(KeyCode::W)
            || gamepad_nav_up
        {
            changed |= self.move_up();
        }
        if safe_key_pressed(KeyCode::Down)
            || safe_key_pressed(KeyCode::S)
            || gamepad_nav_down
        {
            changed |= self.move_down();
        }

        changed
    }

    /// Checks if universal confirm / selection input was triggered this frame.
    pub fn is_confirmed(&self, gamepad_confirm: bool) -> bool {
        safe_key_pressed(KeyCode::Enter)
            || safe_key_pressed(KeyCode::KpEnter)
            || safe_key_pressed(KeyCode::Space)
            || gamepad_confirm
    }

    /// Checks if universal cancel / back input was triggered this frame.
    pub fn is_cancelled(&self, gamepad_cancel: bool) -> bool {
        safe_key_pressed(KeyCode::Escape)
            || gamepad_cancel
    }

    /// Hit-tests mouse cursor against an interactive button rectangle and checks for clicks.
    pub fn check_mouse_click(rect: (f32, f32, f32, f32)) -> bool {
        let (x, y, w, h) = rect;
        let (mx, my) = safe_mouse_pos();
        let is_hovered = mx >= x && mx <= x + w && my >= y && my <= y + h;
        is_hovered && safe_mouse_pressed(MouseButton::Left)
    }

    /// Hit-tests mouse cursor against an interactive button rectangle for hover status.
    pub fn check_mouse_hover(rect: (f32, f32, f32, f32)) -> bool {
        let (x, y, w, h) = rect;
        let (mx, my) = safe_mouse_pos();
        mx >= x && mx <= x + w && my >= y && my <= y + h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_grid_2d_creation_and_bounds() {
        let mut grid = NavGrid2D::new(vec![3, 5]);
        assert_eq!(grid.active_cell(), (0, 0));

        // Move right into Column 1
        assert!(grid.move_right());
        assert_eq!(grid.active_cell(), (1, 0));

        // Move down within Column 1
        assert!(grid.move_down());
        assert_eq!(grid.active_cell(), (1, 1));

        // Move left back into Column 0 (remembers Column 0 cursor position 0)
        assert!(grid.move_left());
        assert_eq!(grid.active_cell(), (0, 0));

        // Move up wrapping to bottom of Column 0
        assert!(grid.move_up());
        assert_eq!(grid.active_cell(), (0, 2));
    }
}
