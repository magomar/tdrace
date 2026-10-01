//! # Platform Virtual Keypad Widget
//!
//! Provides a responsive on-screen arcade keypad supporting both IPv4 address entry
//! and alphanumeric callsign/driver name entry with full gamepad, keyboard, and mouse support.

use macroquad::color::Color;
use macroquad::input::{is_key_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::shapes::draw_rectangle;

use crate::input::NavGrid2D;
use crate::ui::font::Fonts;
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
    std::panic::catch_unwind(|| macroquad::input::is_mouse_button_pressed(btn)).unwrap_or(false)
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

    /// Returns the current input buffer.
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// Sets the buffer contents directly, truncating to `max_len`.
    pub fn set_text(&mut self, text: impl Into<String>) {
        let mut text = text.into();
        text.truncate(self.max_len);
        self.buffer = text;
    }

    /// Processes keyboard, gamepad, and mouse events for navigation and typing.
    #[allow(clippy::too_many_arguments)]
    pub fn handle_input(
        &mut self,
        dt: f32,
        gamepad_left: bool,
        gamepad_right: bool,
        gamepad_up: bool,
        gamepad_down: bool,
        gamepad_confirm: bool,
        bounds_rect: (f32, f32, f32, f32),
        scaler: &UiScaler,
    ) -> VirtualKeypadAction {
        self.blink_timer += dt;

        if !self.is_focused {
            return VirtualKeypadAction::None;
        }

        // Direct hardware keyboard typing (character stream)
        let mut text_changed = false;
        while let Some(c) = std::panic::catch_unwind(macroquad::input::get_char_pressed)
            .ok()
            .flatten()
        {
            let allowed = match self.mode {
                VirtualKeypadMode::IpAddress => c.is_ascii_digit() || c == '.' || c == ':',
                VirtualKeypadMode::Alphanumeric => !c.is_control(),
            };
            if allowed && self.buffer.len() < self.max_len {
                self.buffer.push(c);
                text_changed = true;
            }
        }

        // Digit / period / colon key fallback (IP mode)
        if self.mode == VirtualKeypadMode::IpAddress && !text_changed {
            for num in 0..=9 {
                let key = match num {
                    0 => KeyCode::Key0,
                    1 => KeyCode::Key1,
                    2 => KeyCode::Key2,
                    3 => KeyCode::Key3,
                    4 => KeyCode::Key4,
                    5 => KeyCode::Key5,
                    6 => KeyCode::Key6,
                    7 => KeyCode::Key7,
                    8 => KeyCode::Key8,
                    _ => KeyCode::Key9,
                };
                let kp_key = match num {
                    0 => KeyCode::Kp0,
                    1 => KeyCode::Kp1,
                    2 => KeyCode::Kp2,
                    3 => KeyCode::Kp3,
                    4 => KeyCode::Kp4,
                    5 => KeyCode::Kp5,
                    6 => KeyCode::Kp6,
                    7 => KeyCode::Kp7,
                    8 => KeyCode::Kp8,
                    _ => KeyCode::Kp9,
                };
                if (safe_key_pressed(key) || safe_key_pressed(kp_key)) && self.buffer.len() < self.max_len {
                    self.buffer.push(char::from_digit(num, 10).unwrap_or('0'));
                    text_changed = true;
                }
            }
            if (safe_key_pressed(KeyCode::Period) || safe_key_pressed(KeyCode::KpDecimal))
                && self.buffer.len() < self.max_len
            {
                self.buffer.push('.');
                text_changed = true;
            }
            if safe_key_pressed(KeyCode::Semicolon) && self.buffer.len() < self.max_len {
                self.buffer.push(':');
                text_changed = true;
            }
        }

        if safe_key_pressed(KeyCode::Backspace) {
            text_changed |= self.buffer.pop().is_some();
        }
        if safe_key_pressed(KeyCode::Delete) {
            self.buffer.clear();
            return VirtualKeypadAction::Clear;
        }
        if (safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::KpEnter))
            && !self.buffer.trim().is_empty()
        {
            return VirtualKeypadAction::Submit(self.buffer.clone());
        }

        if text_changed {
            return VirtualKeypadAction::Changed(self.buffer.clone());
        }

        // 2D orthogonal directional navigation
        if safe_key_pressed(KeyCode::Left) || safe_key_pressed(KeyCode::A) || gamepad_left {
            self.nav.move_left();
        }
        if safe_key_pressed(KeyCode::Right) || safe_key_pressed(KeyCode::D) || gamepad_right {
            self.nav.move_right();
        }
        if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) || gamepad_up {
            self.nav.move_up();
        }
        if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) || gamepad_down {
            self.nav.move_down();
        }

        // Mouse hit testing
        let (bx, by, bw, bh) = bounds_rect;
        let pad = scaler.s(8.0);
        let header_h = scaler.s(42.0);
        let grid_y = by + header_h + pad;
        let grid_h = (bh - header_h - pad).max(scaler.s(100.0));
        let col_w = (bw - pad * (self.cols as f32 - 1.0)) / self.cols as f32;
        let row_h = (grid_h - pad * (self.rows as f32 - 1.0)) / self.rows as f32;

        let (mx, my) = safe_mouse_pos();
        let mouse_clicked = safe_mouse_pressed(MouseButton::Left);

        for col in 0..self.cols {
            for row in 0..self.rows {
                let kx = bx + col as f32 * (col_w + pad);
                let ky = grid_y + row as f32 * (row_h + pad);
                let hovered = mx >= kx && mx <= kx + col_w && my >= ky && my <= ky + row_h;
                if hovered {
                    self.nav.set_focus(col, row);
                    if mouse_clicked {
                        return self.activate_button(col, row);
                    }
                }
            }
        }

        // Confirm on focused button (gamepad A / Space)
        if self.nav.is_confirmed(gamepad_confirm || safe_key_pressed(KeyCode::Space)) {
            let (col, row) = self.nav.active_cell();
            return self.activate_button(col, row);
        }

        VirtualKeypadAction::None
    }

    /// Renders the keypad widget with display box and glowing button matrix.
    pub fn draw(&self, scaler: &UiScaler, fonts: &Fonts, x: f32, y: f32, w: f32, h: f32) {
        let pad = scaler.s(8.0);
        let header_h = scaler.s(42.0);

        // Header / text input display box
        scaler.draw_glass_card(
            x,
            y,
            w,
            header_h,
            Color::new(0.04, 0.06, 0.10, 0.95),
            if self.is_focused { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER },
            1.5,
        );

        let label_y = y + header_h * 0.65;
        let font_sz = scaler.font_s(15.0);

        if self.buffer.is_empty() {
            let placeholder = match self.mode {
                VirtualKeypadMode::IpAddress => "e.g. 192.168.1.105:7777",
                VirtualKeypadMode::Alphanumeric => "ENTER DRIVER NAME",
            };
            fonts.draw_ui_bold(placeholder, x + pad * 2.0, label_y, font_sz, Palette::UI_TEXT_MUTED);
        } else {
            fonts.draw_ui_bold(&self.buffer, x + pad * 2.0, label_y, font_sz, Palette::WHITE);

            let is_blink_on = (self.blink_timer % 0.8) < 0.4;
            if self.is_focused && is_blink_on {
                let dim = fonts.measure_ui_bold(&self.buffer, font_sz);
                let cursor_x = x + pad * 2.0 + dim.width + scaler.s(2.0);
                draw_rectangle(
                    cursor_x,
                    y + header_h * 0.25,
                    scaler.s(2.5),
                    header_h * 0.50,
                    Palette::NEON_CYAN,
                );
            }
        }

        // Button grid
        let grid_y = y + header_h + pad;
        let grid_h = (h - header_h - pad).max(scaler.s(100.0));
        let col_w = (w - pad * (self.cols as f32 - 1.0)) / self.cols as f32;
        let row_h = (grid_h - pad * (self.rows as f32 - 1.0)) / self.rows as f32;

        let (focus_col, focus_row) = self.nav.active_cell();
        let (mx, my) = safe_mouse_pos();

        for col in 0..self.cols {
            for row in 0..self.rows {
                let btn = match self.button_at(col, row) {
                    Some(b) => b,
                    None => continue,
                };
                let kx = x + col as f32 * (col_w + pad);
                let ky = grid_y + row as f32 * (row_h + pad);

                let is_focused = self.is_focused && focus_col == col && focus_row == row;
                let is_hovered = mx >= kx && mx <= kx + col_w && my >= ky && my <= ky + row_h;
                let accent = btn.accent_color();

                scaler.draw_button_card(kx, ky, col_w, row_h, is_focused, is_hovered, accent);

                let text_color = if is_focused || is_hovered {
                    Palette::WHITE
                } else if btn == VirtualKeypadButton::Submit {
                    Palette::NEON_GREEN
                } else {
                    accent
                };

                let btn_font_size = if btn.label().len() > 3 {
                    scaler.font_s(11.0)
                } else {
                    scaler.font_s(15.0)
                };

                fonts.draw_ui_bold_centered(
                    btn.label(),
                    kx + col_w * 0.5,
                    ky + row_h * 0.65,
                    btn_font_size,
                    text_color,
                );
            }
        }
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
