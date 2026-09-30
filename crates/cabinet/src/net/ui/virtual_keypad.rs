//! # Platform Virtual Keypad Widget
//!
//! Provides a responsive on-screen arcade keypad supporting both IPv4 address entry
//! and alphanumeric callsign/driver name entry with full gamepad, keyboard, and mouse support.

use macroquad::color::Color;
use macroquad::input::{is_key_pressed, KeyCode};

use crate::input::NavGrid2D;
use crate::ui::theme::Palette;

#[inline]
fn safe_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(|| is_key_pressed(key)).unwrap_or(false)
}

/// Operational mode of the virtual keypad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualKeypadMode {
    /// 4x4 matrix for IPv4 address and port entry.
    IpAddress,
    /// 6x7 matrix for alphanumeric pilot names and callsigns.
    Alphanumeric,
}

/// Action resulting from an interaction with the `VirtualKeypad`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualKeypadAction {
    None,
    Changed(String),
    Submit(String),
    Clear,
    Cancel,
}

/// Virtual keypad button definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualKeypadButton {
    Char(char),
    Backspace,
    Clear,
    Space,
    PresetSubnet,
    Submit,
}

impl VirtualKeypadButton {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Char(c) => match c {
                '0' => "0", '1' => "1", '2' => "2", '3' => "3", '4' => "4",
                '5' => "5", '6' => "6", '7' => "7", '8' => "8", '9' => "9",
                'A' => "A", 'B' => "B", 'C' => "C", 'D' => "D", 'E' => "E",
                'F' => "F", 'G' => "G", 'H' => "H", 'I' => "I", 'J' => "J",
                'K' => "K", 'L' => "L", 'M' => "M", 'N' => "N", 'O' => "O",
                'P' => "P", 'Q' => "Q", 'R' => "R", 'S' => "S", 'T' => "T",
                'U' => "U", 'V' => "V", 'W' => "W", 'X' => "X", 'Y' => "Y",
                'Z' => "Z", '.' => ".", ':' => ":", '-' => "-", '_' => "_",
                _ => "?",
            },
            Self::Backspace => "⌫ BACK",
            Self::Clear => "CLEAR",
            Self::Space => "SPACE",
            Self::PresetSubnet => "192.168.1.",
            Self::Submit => "CONFIRM",
        }
    }

    pub fn accent_color(&self) -> Color {
        match self {
            Self::Char(_) => Palette::NEON_CYAN,
            Self::Space => Palette::NEON_CYAN,
            Self::Backspace => Palette::NEON_ORANGE,
            Self::Clear => Palette::NEON_RED,
            Self::PresetSubnet => Palette::NEON_GOLD,
            Self::Submit => Palette::NEON_GREEN,
        }
    }
}

/// Platform-wide virtual keypad widget.
#[derive(Debug, Clone)]
pub struct VirtualKeypad {
    pub mode: VirtualKeypadMode,
    pub buffer: String,
    pub max_len: usize,
    pub nav: NavGrid2D,
    pub is_focused: bool,
    blink_timer: f32,
    cols: usize,
    rows: usize,
    grid: Vec<Vec<VirtualKeypadButton>>,
}

impl VirtualKeypad {
    /// Creates a virtual keypad configured for IP entry.
    pub fn new_ip(initial: &str) -> Self {
        Self::new(VirtualKeypadMode::IpAddress, initial)
    }

    /// Creates a virtual keypad configured for callsign / driver name entry.
    pub fn new_callsign(initial: &str) -> Self {
        Self::new(VirtualKeypadMode::Alphanumeric, initial)
    }

    /// Creates a new virtual keypad in the given mode.
    pub fn new(mode: VirtualKeypadMode, initial: &str) -> Self {
        match mode {
            VirtualKeypadMode::IpAddress => {
                let cols = 4;
                let rows = 4;
                let grid = vec![
                    vec![
                        VirtualKeypadButton::Char('1'),
                        VirtualKeypadButton::Char('4'),
                        VirtualKeypadButton::Char('7'),
                        VirtualKeypadButton::Char('.'),
                    ],
                    vec![
                        VirtualKeypadButton::Char('2'),
                        VirtualKeypadButton::Char('5'),
                        VirtualKeypadButton::Char('8'),
                        VirtualKeypadButton::Char('0'),
                    ],
                    vec![
                        VirtualKeypadButton::Char('3'),
                        VirtualKeypadButton::Char('6'),
                        VirtualKeypadButton::Char('9'),
                        VirtualKeypadButton::Char(':'),
                    ],
                    vec![
                        VirtualKeypadButton::Backspace,
                        VirtualKeypadButton::Clear,
                        VirtualKeypadButton::PresetSubnet,
                        VirtualKeypadButton::Submit,
                    ],
                ];
                let mut buf = initial.to_string();
                buf.truncate(21);
                Self {
                    mode,
                    buffer: buf,
                    max_len: 21,
                    nav: NavGrid2D::new(vec![rows; cols]),
                    is_focused: true,
                    blink_timer: 0.0,
                    cols,
                    rows,
                    grid,
                }
            }
            VirtualKeypadMode::Alphanumeric => {
                let cols = 6;
                let rows = 7;
                // Grid indexed by grid[col][row]
                let mut grid = vec![vec![VirtualKeypadButton::Char(' '); rows]; cols];
                let layout_rows: [[VirtualKeypadButton; 6]; 7] = [
                    [
                        VirtualKeypadButton::Char('A'),
                        VirtualKeypadButton::Char('B'),
                        VirtualKeypadButton::Char('C'),
                        VirtualKeypadButton::Char('D'),
                        VirtualKeypadButton::Char('E'),
                        VirtualKeypadButton::Char('F'),
                    ],
                    [
                        VirtualKeypadButton::Char('G'),
                        VirtualKeypadButton::Char('H'),
                        VirtualKeypadButton::Char('I'),
                        VirtualKeypadButton::Char('J'),
                        VirtualKeypadButton::Char('K'),
                        VirtualKeypadButton::Char('L'),
                    ],
                    [
                        VirtualKeypadButton::Char('M'),
                        VirtualKeypadButton::Char('N'),
                        VirtualKeypadButton::Char('O'),
                        VirtualKeypadButton::Char('P'),
                        VirtualKeypadButton::Char('Q'),
                        VirtualKeypadButton::Char('R'),
                    ],
                    [
                        VirtualKeypadButton::Char('S'),
                        VirtualKeypadButton::Char('T'),
                        VirtualKeypadButton::Char('U'),
                        VirtualKeypadButton::Char('V'),
                        VirtualKeypadButton::Char('W'),
                        VirtualKeypadButton::Char('X'),
                    ],
                    [
                        VirtualKeypadButton::Char('Y'),
                        VirtualKeypadButton::Char('Z'),
                        VirtualKeypadButton::Char('0'),
                        VirtualKeypadButton::Char('1'),
                        VirtualKeypadButton::Char('2'),
                        VirtualKeypadButton::Char('3'),
                    ],
                    [
                        VirtualKeypadButton::Char('4'),
                        VirtualKeypadButton::Char('5'),
                        VirtualKeypadButton::Char('6'),
                        VirtualKeypadButton::Char('7'),
                        VirtualKeypadButton::Char('8'),
                        VirtualKeypadButton::Char('9'),
                    ],
                    [
                        VirtualKeypadButton::Space,
                        VirtualKeypadButton::Char('-'),
                        VirtualKeypadButton::Char('_'),
                        VirtualKeypadButton::Backspace,
                        VirtualKeypadButton::Clear,
                        VirtualKeypadButton::Submit,
                    ],
                ];

                for r in 0..rows {
                    for c in 0..cols {
                        grid[c][r] = layout_rows[r][c];
                    }
                }

                let mut buf = initial.to_string();
                buf.truncate(16);
                Self {
                    mode,
                    buffer: buf,
                    max_len: 16,
                    nav: NavGrid2D::new(vec![rows; cols]),
                    is_focused: true,
                    blink_timer: 0.0,
                    cols,
                    rows,
                    grid,
                }
            }
        }
    }

    pub fn blink_timer(&self) -> f32 {
        self.blink_timer
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn selected_col(&self) -> usize {
        self.nav.active_cell().0
    }

    pub fn selected_row(&self) -> usize {
        self.nav.active_cell().1
    }

    pub fn button_at(&self, col: usize, row: usize) -> Option<VirtualKeypadButton> {
        self.grid.get(col).and_then(|c| c.get(row)).copied()
    }

    pub fn selected_button(&self) -> Option<VirtualKeypadButton> {
        self.button_at(self.selected_col(), self.selected_row())
    }

    pub fn activate_button(&mut self, col: usize, row: usize) -> VirtualKeypadAction {
        let btn = match self.button_at(col, row) {
            Some(b) => b,
            None => return VirtualKeypadAction::None,
        };

        match btn {
            VirtualKeypadButton::Char(c) => {
                if self.buffer.len() < self.max_len {
                    self.buffer.push(c);
                    VirtualKeypadAction::Changed(self.buffer.clone())
                } else {
                    VirtualKeypadAction::None
                }
            }
            VirtualKeypadButton::Space => {
                if self.buffer.len() < self.max_len && !self.buffer.is_empty() {
                    self.buffer.push(' ');
                    VirtualKeypadAction::Changed(self.buffer.clone())
                } else {
                    VirtualKeypadAction::None
                }
            }
            VirtualKeypadButton::Backspace => {
                if !self.buffer.is_empty() {
                    self.buffer.pop();
                    VirtualKeypadAction::Changed(self.buffer.clone())
                } else {
                    VirtualKeypadAction::None
                }
            }
            VirtualKeypadButton::Clear => {
                self.buffer.clear();
                VirtualKeypadAction::Clear
            }
            VirtualKeypadButton::PresetSubnet => {
                self.buffer = "192.168.1.".to_string();
                VirtualKeypadAction::Changed(self.buffer.clone())
            }
            VirtualKeypadButton::Submit => {
                if !self.buffer.is_empty() {
                    VirtualKeypadAction::Submit(self.buffer.clone())
                } else {
                    VirtualKeypadAction::None
                }
            }
        }
    }

    pub fn activate_selected(&mut self) -> VirtualKeypadAction {
        self.activate_button(self.selected_col(), self.selected_row())
    }

    /// Handles keyboard / gamepad input and cursor animation.
    pub fn update(&mut self, dt: f32) -> VirtualKeypadAction {
        self.blink_timer += dt;
        if !self.is_focused {
            return VirtualKeypadAction::None;
        }

        // Arrow / WASD navigation
        if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) {
            self.nav.move_up();
        }
        if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) {
            self.nav.move_down();
        }
        if safe_key_pressed(KeyCode::Left) || safe_key_pressed(KeyCode::A) {
            self.nav.move_left();
        }
        if safe_key_pressed(KeyCode::Right) || safe_key_pressed(KeyCode::D) {
            self.nav.move_right();
        }

        // Selection confirmation on Enter or Space (if space is not hovered)
        if safe_key_pressed(KeyCode::Enter) {
            return self.activate_selected();
        }
        if safe_key_pressed(KeyCode::Space) {
            if self.mode == VirtualKeypadMode::Alphanumeric
                && self.selected_button() == Some(VirtualKeypadButton::Space)
            {
                return self.activate_selected();
            } else if self.mode == VirtualKeypadMode::IpAddress {
                return self.activate_selected();
            }
        }

        // Direct backspace shortcut
        if safe_key_pressed(KeyCode::Backspace) && !self.buffer.is_empty() {
            self.buffer.pop();
            return VirtualKeypadAction::Changed(self.buffer.clone());
        }

        VirtualKeypadAction::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_keypad_ip_mode() {
        let mut pad = VirtualKeypad::new_ip("");
        assert_eq!(pad.cols, 4);
        assert_eq!(pad.rows, 4);

        // Click '1' at col 0, row 0
        let act = pad.activate_button(0, 0);
        assert_eq!(act, VirtualKeypadAction::Changed("1".to_string()));

        // Click '0' at col 1, row 3
        let act = pad.activate_button(1, 3);
        assert_eq!(act, VirtualKeypadAction::Changed("10".to_string()));

        // Backspace at col 3, row 0
        let act = pad.activate_button(3, 0);
        assert_eq!(act, VirtualKeypadAction::Changed("1".to_string()));

        // Clear at col 3, row 1
        let act = pad.activate_button(3, 1);
        assert_eq!(act, VirtualKeypadAction::Clear);

        // Preset subnet at col 3, row 2
        let act = pad.activate_button(3, 2);
        assert_eq!(act, VirtualKeypadAction::Changed("192.168.1.".to_string()));

        // Submit at col 3, row 3
        let act = pad.activate_button(3, 3);
        assert_eq!(act, VirtualKeypadAction::Submit("192.168.1.".to_string()));
    }

    #[test]
    fn test_virtual_keypad_alphanumeric_mode() {
        let mut pad = VirtualKeypad::new_callsign("ACE");
        assert_eq!(pad.cols, 6);
        assert_eq!(pad.rows, 7);
        assert_eq!(pad.buffer, "ACE");

        // Click 'D' (row 0, col 3 -> layout_rows[0][3] = D)
        let act = pad.activate_button(3, 0);
        assert_eq!(act, VirtualKeypadAction::Changed("ACED".to_string()));

        // Backspace (row 6, col 3)
        let act = pad.activate_button(3, 6);
        assert_eq!(act, VirtualKeypadAction::Changed("ACE".to_string()));

        // Submit (row 6, col 5)
        let act = pad.activate_button(5, 6);
        assert_eq!(act, VirtualKeypadAction::Submit("ACE".to_string()));
    }
}
