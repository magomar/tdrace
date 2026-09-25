//! # 2D Arcade Virtual Numeric Keypad Widget
//!
//! Provides a responsive 4x4 on-screen keypad designed for gamepad, keyboard,
//! and mouse interaction to enter IPv4 addresses and port numbers on arcade hardware.

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

/// Action resulting from an interaction with the `IpKeypad`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpKeypadAction {
    /// No action or submission triggered this frame.
    None,
    /// The buffer text changed.
    Changed(String),
    /// User confirmed the input address string for connection.
    Submit(String),
    /// User cleared the buffer.
    Clear,
}

/// Keypad button definition in the 4x4 matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeypadButton {
    Char(char),
    Backspace,
    Clear,
    PresetSubnet,
    Submit,
}

impl KeypadButton {
    /// Text label rendered on the button.
    pub fn label(&self) -> &'static str {
        match self {
            KeypadButton::Char(c) => match c {
                '0' => "0",
                '1' => "1",
                '2' => "2",
                '3' => "3",
                '4' => "4",
                '5' => "5",
                '6' => "6",
                '7' => "7",
                '8' => "8",
                '9' => "9",
                '.' => ".",
                ':' => ":",
                _ => "?",
            },
            KeypadButton::Backspace => "⌫ BACK",
            KeypadButton::Clear => "CLEAR",
            KeypadButton::PresetSubnet => "192.168.1.",
            KeypadButton::Submit => "CONNECT",
        }
    }

    /// Primary accent color for button focus/borders.
    pub fn accent_color(&self) -> Color {
        match self {
            KeypadButton::Char(_) => Palette::NEON_CYAN,
            KeypadButton::Backspace => Palette::NEON_ORANGE,
            KeypadButton::Clear => Palette::NEON_RED,
            KeypadButton::PresetSubnet => Palette::NEON_GOLD,
            KeypadButton::Submit => Palette::NEON_GREEN,
        }
    }
}

/// 4x4 Matrix Layout:
/// - Col 0: '1', '4', '7', '.'
/// - Col 1: '2', '5', '8', '0'
/// - Col 2: '3', '6', '9', ':'
/// - Col 3: Backspace, Clear, Subnet Preset, Connect
pub const KEYPAD_GRID: [[KeypadButton; 4]; 4] = [
    [
        KeypadButton::Char('1'),
        KeypadButton::Char('4'),
        KeypadButton::Char('7'),
        KeypadButton::Char('.'),
    ],
    [
        KeypadButton::Char('2'),
        KeypadButton::Char('5'),
        KeypadButton::Char('8'),
        KeypadButton::Char('0'),
    ],
    [
        KeypadButton::Char('3'),
        KeypadButton::Char('6'),
        KeypadButton::Char('9'),
        KeypadButton::Char(':'),
    ],
    [
        KeypadButton::Backspace,
        KeypadButton::Clear,
        KeypadButton::PresetSubnet,
        KeypadButton::Submit,
    ],
];

/// Arcade virtual numeric keypad widget for gamepad and keyboard direct IP entry.
#[derive(Debug, Clone)]
pub struct IpKeypad {
    /// Active address string buffer (e.g. `"192.168.1.105:7777"`).
    pub buffer: String,
    /// Maximum allowed characters in the input buffer.
    pub max_len: usize,
    /// 2D orthogonal navigation router managing focus across the 4x4 matrix.
    pub nav: NavGrid2D,
    /// Whether the keypad is currently focused for user input.
    pub is_focused: bool,
    /// Cursor blink timer.
    blink_timer: f32,
}

impl Default for IpKeypad {
    fn default() -> Self {
        Self::new("192.168.1.")
    }
}

impl IpKeypad {
    /// Creates a new keypad widget with an initial buffer string.
    pub fn new(initial_text: impl Into<String>) -> Self {
        let mut buffer = initial_text.into();
        buffer.truncate(24);
        Self {
            buffer,
            max_len: 24,
            nav: NavGrid2D::new(vec![4, 4, 4, 4]),
            is_focused: true,
            blink_timer: 0.0,
        }
    }

    /// Returns the current input text.
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// Sets the buffer contents directly.
    pub fn set_text(&mut self, text: impl Into<String>) {
        let mut text = text.into();
        text.truncate(self.max_len);
        self.buffer = text;
    }

    /// Appends a single character if under max length.
    pub fn append_char(&mut self, c: char) -> bool {
        if self.buffer.len() < self.max_len {
            self.buffer.push(c);
            true
        } else {
            false
        }
    }

    /// Removes the last character from the buffer.
    pub fn backspace(&mut self) -> bool {
        self.buffer.pop().is_some()
    }

    /// Clears the input buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Triggers the action of the button at the specified grid cell.
    pub fn activate_button(&mut self, col: usize, row: usize) -> IpKeypadAction {
        if col >= 4 || row >= 4 {
            return IpKeypadAction::None;
        }

        let button = KEYPAD_GRID[col][row];
        match button {
            KeypadButton::Char(c) => {
                if self.append_char(c) {
                    IpKeypadAction::Changed(self.buffer.clone())
                } else {
                    IpKeypadAction::None
                }
            }
            KeypadButton::Backspace => {
                if self.backspace() {
                    IpKeypadAction::Changed(self.buffer.clone())
                } else {
                    IpKeypadAction::None
                }
            }
            KeypadButton::Clear => {
                self.clear();
                IpKeypadAction::Clear
            }
            KeypadButton::PresetSubnet => {
                if !self.buffer.starts_with("192.168.1.") {
                    self.set_text("192.168.1.");
                } else if !self.buffer.contains(':') && self.buffer.len() > 10 {
                    let _ = self.append_char(':');
                    self.buffer.push_str("7777");
                } else {
                    self.set_text("192.168.1.");
                }
                IpKeypadAction::Changed(self.buffer.clone())
            }
            KeypadButton::Submit => {
                if !self.buffer.trim().is_empty() {
                    IpKeypadAction::Submit(self.buffer.clone())
                } else {
                    IpKeypadAction::None
                }
            }
        }
    }

    /// Moves cursor left in the keypad matrix, preserving the current row.
    pub fn move_left(&mut self) -> bool {
        let cur_row = self.nav.active_row();
        if self.nav.move_left() {
            self.nav.cursor_rows[self.nav.focused_col] = cur_row;
            true
        } else {
            false
        }
    }

    /// Moves cursor right in the keypad matrix, preserving the current row.
    pub fn move_right(&mut self) -> bool {
        let cur_row = self.nav.active_row();
        if self.nav.move_right() {
            self.nav.cursor_rows[self.nav.focused_col] = cur_row;
            true
        } else {
            false
        }
    }

    /// Moves cursor up in the keypad matrix.
    pub fn move_up(&mut self) -> bool {
        self.nav.move_up()
    }

    /// Moves cursor down in the keypad matrix.
    pub fn move_down(&mut self) -> bool {
        self.nav.move_down()
    }

    /// Processes keyboard, gamepad, and mouse events for navigation and typing.
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
    ) -> IpKeypadAction {
        self.blink_timer += dt;

        if !self.is_focused {
            return IpKeypadAction::None;
        }

        // Direct hardware keyboard typing support
        let mut text_changed = false;
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
                9 => KeyCode::Key9,
                _ => unreachable!(),
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
                9 => KeyCode::Kp9,
                _ => unreachable!(),
            };
            if safe_key_pressed(key) || safe_key_pressed(kp_key) {
                let c = char::from_digit(num, 10).unwrap_or('0');
                text_changed |= self.append_char(c);
            }
        }

        if safe_key_pressed(KeyCode::Period) || safe_key_pressed(KeyCode::KpDecimal) {
            text_changed |= self.append_char('.');
        }
        if safe_key_pressed(KeyCode::Semicolon) {
            text_changed |= self.append_char(':');
        }
        if safe_key_pressed(KeyCode::Backspace) {
            text_changed |= self.backspace();
        }
        if safe_key_pressed(KeyCode::Delete) {
            self.clear();
            return IpKeypadAction::Clear;
        }
        if safe_key_pressed(KeyCode::Enter) || safe_key_pressed(KeyCode::KpEnter) {
            if !self.buffer.trim().is_empty() {
                return IpKeypadAction::Submit(self.buffer.clone());
            }
        }

        if text_changed {
            return IpKeypadAction::Changed(self.buffer.clone());
        }

        // 2D Orthogonal Directional Navigation
        if safe_key_pressed(KeyCode::Left) || safe_key_pressed(KeyCode::A) || gamepad_left {
            self.move_left();
        }
        if safe_key_pressed(KeyCode::Right) || safe_key_pressed(KeyCode::D) || gamepad_right {
            self.move_right();
        }
        if safe_key_pressed(KeyCode::Up) || safe_key_pressed(KeyCode::W) || gamepad_up {
            self.move_up();
        }
        if safe_key_pressed(KeyCode::Down) || safe_key_pressed(KeyCode::S) || gamepad_down {
            self.move_down();
        }

        // Calculate geometry for mouse hit testing
        let (bx, by, bw, bh) = bounds_rect;
        let pad = scaler.s(8.0);
        let header_h = scaler.s(42.0);
        let grid_y = by + header_h + pad;
        let grid_h = (bh - header_h - pad).max(scaler.s(100.0));
        let col_w = (bw - pad * 3.0) / 4.0;
        let row_h = (grid_h - pad * 3.0) / 4.0;

        let (mx, my) = safe_mouse_pos();
        let mouse_clicked = safe_mouse_pressed(MouseButton::Left);

        for col in 0..4 {
            for row in 0..4 {
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

        // Gamepad / Keyboard Confirm on focused button
        if self.nav.is_confirmed(gamepad_confirm) {
            let (col, row) = self.nav.active_cell();
            return self.activate_button(col, row);
        }

        IpKeypadAction::None
    }

    /// Renders the keypad widget with display box and 4x4 glowing button matrix.
    pub fn draw(
        &self,
        scaler: &UiScaler,
        fonts: &Fonts,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let pad = scaler.s(8.0);
        let header_h = scaler.s(42.0);

        // Header / Text input display box
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
            fonts.draw_ui_bold(
                "e.g. 192.168.1.105:7777",
                x + pad * 2.0,
                label_y,
                font_sz,
                Palette::UI_TEXT_MUTED,
            );
        } else {
            fonts.draw_ui_bold(
                &self.buffer,
                x + pad * 2.0,
                label_y,
                font_sz,
                Palette::WHITE,
            );

            // Blinking cursor
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

        // 4x4 Button Grid
        let grid_y = y + header_h + pad;
        let grid_h = (h - header_h - pad).max(scaler.s(100.0));
        let col_w = (w - pad * 3.0) / 4.0;
        let row_h = (grid_h - pad * 3.0) / 4.0;

        let (focus_col, focus_row) = self.nav.active_cell();
        let (mx, my) = safe_mouse_pos();

        for col in 0..4 {
            for row in 0..4 {
                let kx = x + col as f32 * (col_w + pad);
                let ky = grid_y + row as f32 * (row_h + pad);
                let btn = KEYPAD_GRID[col][row];

                let is_focused = self.is_focused && focus_col == col && focus_row == row;
                let is_hovered = mx >= kx && mx <= kx + col_w && my >= ky && my <= ky + row_h;
                let accent = btn.accent_color();

                scaler.draw_button_card(
                    kx,
                    ky,
                    col_w,
                    row_h,
                    is_focused,
                    is_hovered,
                    accent,
                );

                let text_color = if is_focused || is_hovered {
                    Palette::WHITE
                } else if btn == KeypadButton::Submit {
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
    fn test_ip_keypad_initialization_and_truncation() {
        let pad = IpKeypad::new("192.168.1.100");
        assert_eq!(pad.text(), "192.168.1.100");
        assert_eq!(pad.nav.active_cell(), (0, 0));

        let long_pad = IpKeypad::new("192.168.1.100:7777_extra_long_text");
        assert!(long_pad.text().len() <= 24);
    }

    #[test]
    fn test_ip_keypad_char_appending_and_backspace() {
        let mut pad = IpKeypad::new("");
        assert!(pad.append_char('1'));
        assert!(pad.append_char('0'));
        assert_eq!(pad.text(), "10");

        assert!(pad.backspace());
        assert_eq!(pad.text(), "1");

        pad.clear();
        assert_eq!(pad.text(), "");
        assert!(!pad.backspace());
    }

    #[test]
    fn test_ip_keypad_button_activations() {
        let mut pad = IpKeypad::new("");

        // Col 0, Row 0 is '1'
        let act = pad.activate_button(0, 0);
        assert_eq!(act, IpKeypadAction::Changed("1".to_string()));

        // Col 1, Row 3 is '0'
        let act = pad.activate_button(1, 3);
        assert_eq!(act, IpKeypadAction::Changed("10".to_string()));

        // Col 3, Row 0 is Backspace
        let act = pad.activate_button(3, 0);
        assert_eq!(act, IpKeypadAction::Changed("1".to_string()));

        // Col 3, Row 1 is Clear
        let act = pad.activate_button(3, 1);
        assert_eq!(act, IpKeypadAction::Clear);

        // Col 3, Row 2 is Preset
        let act = pad.activate_button(3, 2);
        assert_eq!(act, IpKeypadAction::Changed("192.168.1.".to_string()));

        // Col 3, Row 3 is Submit
        let act = pad.activate_button(3, 3);
        assert_eq!(act, IpKeypadAction::Submit("192.168.1.".to_string()));
    }

    #[test]
    fn test_ip_keypad_nav_movement() {
        let mut pad = IpKeypad::new("192.168.1.1");
        assert_eq!(pad.nav.active_cell(), (0, 0));

        assert!(pad.move_right());
        assert_eq!(pad.nav.active_cell(), (1, 0));

        assert!(pad.move_down());
        assert_eq!(pad.nav.active_cell(), (1, 1));

        assert!(pad.move_left());
        assert_eq!(pad.nav.active_cell(), (0, 1));

        assert!(pad.move_up());
        assert_eq!(pad.nav.active_cell(), (0, 0));
    }
}
