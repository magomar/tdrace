use std::collections::HashMap;
use glam::Vec2;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};
use serde::{Deserialize, Serialize};

use crate::input::GamepadSnapshot;

/// Abstract gameplay and shell actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArcadeAction {
    Up,
    Down,
    Left,
    Right,
    Primary,
    Secondary,
    Action3,
    Action4,
    Pause,
    Menu,
    TabPrev,
    TabNext,
}

impl ArcadeAction {
    /// List of all standard arcade actions.
    pub const ALL: [ArcadeAction; 12] = [
        ArcadeAction::Up,
        ArcadeAction::Down,
        ArcadeAction::Left,
        ArcadeAction::Right,
        ArcadeAction::Primary,
        ArcadeAction::Secondary,
        ArcadeAction::Action3,
        ArcadeAction::Action4,
        ArcadeAction::Pause,
        ArcadeAction::Menu,
        ArcadeAction::TabPrev,
        ArcadeAction::TabNext,
    ];

    /// Human-friendly display name.
    pub fn label(&self) -> &'static str {
        match self {
            ArcadeAction::Up => "Move Up / Throttle",
            ArcadeAction::Down => "Move Down / Brake",
            ArcadeAction::Left => "Steer / Move Left",
            ArcadeAction::Right => "Steer / Move Right",
            ArcadeAction::Primary => "Primary Action / Fire",
            ArcadeAction::Secondary => "Secondary / Boost",
            ArcadeAction::Action3 => "Special / Drift",
            ArcadeAction::Action4 => "Utility / Horn",
            ArcadeAction::Pause => "Pause Game",
            ArcadeAction::Menu => "In-Game Menu",
            ArcadeAction::TabPrev => "Previous Tab (LB)",
            ArcadeAction::TabNext => "Next Tab (RB)",
        }
    }
}

/// Serializable representation of standard keyboard keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArcadeKey {
    Up,
    Down,
    Left,
    Right,
    W,
    A,
    S,
    D,
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    Z,
    X,
    C,
    V,
    Q,
    E,
    R,
    F,
    H,
    K,
    O,
    P,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Other(u32),
}

impl ArcadeKey {
    /// Converts this `ArcadeKey` to Macroquad's `KeyCode`.
    pub fn to_key_code(self) -> Option<KeyCode> {
        match self {
            ArcadeKey::Up => Some(KeyCode::Up),
            ArcadeKey::Down => Some(KeyCode::Down),
            ArcadeKey::Left => Some(KeyCode::Left),
            ArcadeKey::Right => Some(KeyCode::Right),
            ArcadeKey::W => Some(KeyCode::W),
            ArcadeKey::A => Some(KeyCode::A),
            ArcadeKey::S => Some(KeyCode::S),
            ArcadeKey::D => Some(KeyCode::D),
            ArcadeKey::Space => Some(KeyCode::Space),
            ArcadeKey::Enter => Some(KeyCode::Enter),
            ArcadeKey::Escape => Some(KeyCode::Escape),
            ArcadeKey::Tab => Some(KeyCode::Tab),
            ArcadeKey::Backspace => Some(KeyCode::Backspace),
            ArcadeKey::LeftShift => Some(KeyCode::LeftShift),
            ArcadeKey::RightShift => Some(KeyCode::RightShift),
            ArcadeKey::LeftControl => Some(KeyCode::LeftControl),
            ArcadeKey::RightControl => Some(KeyCode::RightControl),
            ArcadeKey::LeftAlt => Some(KeyCode::LeftAlt),
            ArcadeKey::RightAlt => Some(KeyCode::RightAlt),
            ArcadeKey::Z => Some(KeyCode::Z),
            ArcadeKey::X => Some(KeyCode::X),
            ArcadeKey::C => Some(KeyCode::C),
            ArcadeKey::V => Some(KeyCode::V),
            ArcadeKey::Q => Some(KeyCode::Q),
            ArcadeKey::E => Some(KeyCode::E),
            ArcadeKey::R => Some(KeyCode::R),
            ArcadeKey::F => Some(KeyCode::F),
            ArcadeKey::H => Some(KeyCode::H),
            ArcadeKey::K => Some(KeyCode::K),
            ArcadeKey::O => Some(KeyCode::O),
            ArcadeKey::P => Some(KeyCode::P),
            ArcadeKey::Num1 => Some(KeyCode::Key1),
            ArcadeKey::Num2 => Some(KeyCode::Key2),
            ArcadeKey::Num3 => Some(KeyCode::Key3),
            ArcadeKey::Num4 => Some(KeyCode::Key4),
            ArcadeKey::Num5 => Some(KeyCode::Key5),
            ArcadeKey::Other(_) => None,
        }
    }

    /// Converts Macroquad's `KeyCode` into `ArcadeKey`.
    pub fn from_key_code(code: KeyCode) -> Self {
        match code {
            KeyCode::Up => ArcadeKey::Up,
            KeyCode::Down => ArcadeKey::Down,
            KeyCode::Left => ArcadeKey::Left,
            KeyCode::Right => ArcadeKey::Right,
            KeyCode::W => ArcadeKey::W,
            KeyCode::A => ArcadeKey::A,
            KeyCode::S => ArcadeKey::S,
            KeyCode::D => ArcadeKey::D,
            KeyCode::Space => ArcadeKey::Space,
            KeyCode::Enter => ArcadeKey::Enter,
            KeyCode::Escape => ArcadeKey::Escape,
            KeyCode::Tab => ArcadeKey::Tab,
            KeyCode::Backspace => ArcadeKey::Backspace,
            KeyCode::LeftShift => ArcadeKey::LeftShift,
            KeyCode::RightShift => ArcadeKey::RightShift,
            KeyCode::LeftControl => ArcadeKey::LeftControl,
            KeyCode::RightControl => ArcadeKey::RightControl,
            KeyCode::LeftAlt => ArcadeKey::LeftAlt,
            KeyCode::RightAlt => ArcadeKey::RightAlt,
            KeyCode::Z => ArcadeKey::Z,
            KeyCode::X => ArcadeKey::X,
            KeyCode::C => ArcadeKey::C,
            KeyCode::V => ArcadeKey::V,
            KeyCode::Q => ArcadeKey::Q,
            KeyCode::E => ArcadeKey::E,
            KeyCode::R => ArcadeKey::R,
            KeyCode::F => ArcadeKey::F,
            KeyCode::H => ArcadeKey::H,
            KeyCode::K => ArcadeKey::K,
            KeyCode::O => ArcadeKey::O,
            KeyCode::P => ArcadeKey::P,
            KeyCode::Key1 => ArcadeKey::Num1,
            KeyCode::Key2 => ArcadeKey::Num2,
            KeyCode::Key3 => ArcadeKey::Num3,
            KeyCode::Key4 => ArcadeKey::Num4,
            KeyCode::Key5 => ArcadeKey::Num5,
            _ => ArcadeKey::Other(code as u32),
        }
    }

    /// Short label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            ArcadeKey::Up => "Up Arrow",
            ArcadeKey::Down => "Down Arrow",
            ArcadeKey::Left => "Left Arrow",
            ArcadeKey::Right => "Right Arrow",
            ArcadeKey::W => "W",
            ArcadeKey::A => "A",
            ArcadeKey::S => "S",
            ArcadeKey::D => "D",
            ArcadeKey::Space => "Space",
            ArcadeKey::Enter => "Enter",
            ArcadeKey::Escape => "Esc",
            ArcadeKey::Tab => "Tab",
            ArcadeKey::Backspace => "Backspace",
            ArcadeKey::LeftShift => "LShift",
            ArcadeKey::RightShift => "RShift",
            ArcadeKey::LeftControl => "LCtrl",
            ArcadeKey::RightControl => "RCtrl",
            ArcadeKey::LeftAlt => "LAlt",
            ArcadeKey::RightAlt => "RAlt",
            ArcadeKey::Z => "Z",
            ArcadeKey::X => "X",
            ArcadeKey::C => "C",
            ArcadeKey::V => "V",
            ArcadeKey::Q => "Q",
            ArcadeKey::E => "E",
            ArcadeKey::R => "R",
            ArcadeKey::F => "F",
            ArcadeKey::H => "H",
            ArcadeKey::K => "K",
            ArcadeKey::O => "O",
            ArcadeKey::P => "P",
            ArcadeKey::Num1 => "1",
            ArcadeKey::Num2 => "2",
            ArcadeKey::Num3 => "3",
            ArcadeKey::Num4 => "4",
            ArcadeKey::Num5 => "5",
            ArcadeKey::Other(_) => "Key",
        }
    }
}

/// Gamepad buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South, // A / Cross
    East,  // B / Circle
    West,  // X / Square
    North, // Y / Triangle
    LeftBumper,
    RightBumper,
    Start,
    Back,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
}

impl GamepadButton {
    pub fn label(&self) -> &'static str {
        match self {
            GamepadButton::South => "Button A / Cross",
            GamepadButton::East => "Button B / Circle",
            GamepadButton::West => "Button X / Square",
            GamepadButton::North => "Button Y / Triangle",
            GamepadButton::LeftBumper => "Left Bumper (LB)",
            GamepadButton::RightBumper => "Right Bumper (RB)",
            GamepadButton::Start => "Start / Options",
            GamepadButton::Back => "Back / Share",
            GamepadButton::DpadUp => "D-Pad Up",
            GamepadButton::DpadDown => "D-Pad Down",
            GamepadButton::DpadLeft => "D-Pad Left",
            GamepadButton::DpadRight => "D-Pad Right",
        }
    }
}

/// Analog stick or trigger axis direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    Throttle,
    Brake,
}

impl GamepadAxis {
    pub fn label(&self) -> &'static str {
        match self {
            GamepadAxis::LeftStickX => "Left Stick X",
            GamepadAxis::LeftStickY => "Left Stick Y",
            GamepadAxis::RightStickX => "Right Stick X",
            GamepadAxis::RightStickY => "Right Stick Y",
            GamepadAxis::Throttle => "RT / R2",
            GamepadAxis::Brake => "LT / L2",
        }
    }
}

/// Abstract input source that can bind to an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSource {
    Key(ArcadeKey),
    GamepadBtn(GamepadButton),
    GamepadAxisPos(GamepadAxis),
    GamepadAxisNeg(GamepadAxis),
}

#[inline]
fn safe_key_down(key: KeyCode) -> bool {
    std::panic::catch_unwind(|| is_key_down(key)).unwrap_or(false)
}

#[inline]
fn safe_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(|| is_key_pressed(key)).unwrap_or(false)
}

impl InputSource {
    pub fn key(code: KeyCode) -> Self {
        InputSource::Key(ArcadeKey::from_key_code(code))
    }

    /// Evaluates if this source is held down.
    pub fn is_down(&self, gp: &GamepadSnapshot) -> bool {
        match self {
            InputSource::Key(k) => {
                if let Some(kc) = k.to_key_code() {
                    safe_key_down(kc)
                } else {
                    false
                }
            }
            InputSource::GamepadBtn(btn) => match btn {
                GamepadButton::South => gp.handbrake || gp.btn_a_pressed,
                GamepadButton::East => gp.btn_b_pressed,
                GamepadButton::West => gp.btn_x_pressed,
                GamepadButton::North => gp.reverse || gp.btn_y_pressed,
                GamepadButton::Start => gp.btn_start_pressed,
                GamepadButton::Back => gp.btn_back_pressed,
                GamepadButton::DpadUp => gp.dpad_up_pressed || gp.nav_up,
                GamepadButton::DpadDown => gp.dpad_down_pressed || gp.nav_down,
                GamepadButton::DpadLeft => gp.dpad_left_pressed || gp.nav_left,
                GamepadButton::DpadRight => gp.dpad_right_pressed || gp.nav_right,
                GamepadButton::LeftBumper => false,
                GamepadButton::RightBumper => false,
            },
            InputSource::GamepadAxisPos(axis) => match axis {
                GamepadAxis::LeftStickX => gp.steer > 0.3,
                GamepadAxis::LeftStickY => false,
                GamepadAxis::RightStickX => false,
                GamepadAxis::RightStickY => false,
                GamepadAxis::Throttle => gp.throttle > 0.2,
                GamepadAxis::Brake => gp.brake > 0.2,
            },
            InputSource::GamepadAxisNeg(axis) => match axis {
                GamepadAxis::LeftStickX => gp.steer < -0.3,
                GamepadAxis::LeftStickY => false,
                GamepadAxis::RightStickX => false,
                GamepadAxis::RightStickY => false,
                GamepadAxis::Throttle => false,
                GamepadAxis::Brake => false,
            },
        }
    }

    /// Evaluates if this source was pressed this frame.
    pub fn is_pressed(&self, gp: &GamepadSnapshot) -> bool {
        match self {
            InputSource::Key(k) => {
                if let Some(kc) = k.to_key_code() {
                    safe_key_pressed(kc)
                } else {
                    false
                }
            }
            InputSource::GamepadBtn(btn) => match btn {
                GamepadButton::South => gp.btn_a_pressed || gp.btn_confirm_pressed,
                GamepadButton::East => gp.btn_b_pressed || gp.btn_cancel_pressed,
                GamepadButton::West => gp.btn_x_pressed,
                GamepadButton::North => gp.btn_y_pressed,
                GamepadButton::Start => gp.btn_start_pressed,
                GamepadButton::Back => gp.btn_back_pressed,
                GamepadButton::DpadUp => gp.dpad_up_pressed || gp.nav_up,
                GamepadButton::DpadDown => gp.dpad_down_pressed || gp.nav_down,
                GamepadButton::DpadLeft => gp.dpad_left_pressed || gp.nav_left,
                GamepadButton::DpadRight => gp.dpad_right_pressed || gp.nav_right,
                GamepadButton::LeftBumper => false,
                GamepadButton::RightBumper => false,
            },
            InputSource::GamepadAxisPos(axis) => match axis {
                GamepadAxis::LeftStickX => gp.nav_right,
                GamepadAxis::LeftStickY => gp.nav_down,
                _ => false,
            },
            InputSource::GamepadAxisNeg(axis) => match axis {
                GamepadAxis::LeftStickX => gp.nav_left,
                GamepadAxis::LeftStickY => gp.nav_up,
                _ => false,
            },
        }
    }

    /// Returns human-readable label for this input source.
    pub fn label(&self) -> String {
        match self {
            InputSource::Key(k) => k.label().to_string(),
            InputSource::GamepadBtn(b) => b.label().to_string(),
            InputSource::GamepadAxisPos(a) => format!("{}+", a.label()),
            InputSource::GamepadAxisNeg(a) => format!("{}-", a.label()),
        }
    }
}

/// Action-to-input binding map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputMap {
    pub bindings: HashMap<ArcadeAction, Vec<InputSource>>,
}

impl Default for InputMap {
    fn default() -> Self {
        Self::default_arcade()
    }
}

impl InputMap {
    /// Standard arcade layout mapping keyboard and gamepad inputs.
    pub fn default_arcade() -> Self {
        let mut bindings = HashMap::new();

        bindings.insert(
            ArcadeAction::Up,
            vec![
                InputSource::Key(ArcadeKey::Up),
                InputSource::Key(ArcadeKey::W),
                InputSource::GamepadBtn(GamepadButton::DpadUp),
                InputSource::GamepadAxisNeg(GamepadAxis::LeftStickY),
            ],
        );
        bindings.insert(
            ArcadeAction::Down,
            vec![
                InputSource::Key(ArcadeKey::Down),
                InputSource::Key(ArcadeKey::S),
                InputSource::GamepadBtn(GamepadButton::DpadDown),
                InputSource::GamepadAxisPos(GamepadAxis::LeftStickY),
            ],
        );
        bindings.insert(
            ArcadeAction::Left,
            vec![
                InputSource::Key(ArcadeKey::Left),
                InputSource::Key(ArcadeKey::A),
                InputSource::GamepadBtn(GamepadButton::DpadLeft),
                InputSource::GamepadAxisNeg(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Right,
            vec![
                InputSource::Key(ArcadeKey::Right),
                InputSource::Key(ArcadeKey::D),
                InputSource::GamepadBtn(GamepadButton::DpadRight),
                InputSource::GamepadAxisPos(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Primary,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::Key(ArcadeKey::Z),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Secondary,
            vec![
                InputSource::Key(ArcadeKey::LeftShift),
                InputSource::Key(ArcadeKey::X),
                InputSource::GamepadBtn(GamepadButton::West),
            ],
        );
        bindings.insert(
            ArcadeAction::Action3,
            vec![
                InputSource::Key(ArcadeKey::C),
                InputSource::GamepadBtn(GamepadButton::East),
            ],
        );
        bindings.insert(
            ArcadeAction::Action4,
            vec![
                InputSource::Key(ArcadeKey::V),
                InputSource::GamepadBtn(GamepadButton::North),
            ],
        );
        bindings.insert(
            ArcadeAction::Pause,
            vec![
                InputSource::Key(ArcadeKey::Escape),
                InputSource::GamepadBtn(GamepadButton::Start),
            ],
        );
        bindings.insert(
            ArcadeAction::Menu,
            vec![
                InputSource::Key(ArcadeKey::Tab),
                InputSource::GamepadBtn(GamepadButton::Back),
            ],
        );
        bindings.insert(
            ArcadeAction::TabPrev,
            vec![
                InputSource::Key(ArcadeKey::Q),
                InputSource::GamepadBtn(GamepadButton::LeftBumper),
            ],
        );
        bindings.insert(
            ArcadeAction::TabNext,
            vec![
                InputSource::Key(ArcadeKey::E),
                InputSource::GamepadBtn(GamepadButton::RightBumper),
            ],
        );

        Self { bindings }
    }

    /// Standard racing layout supporting Q/A/O/P, Arrow keys, WASD, and gamepad triggers/sticks.
    pub fn default_racing() -> Self {
        let mut bindings = HashMap::new();

        bindings.insert(
            ArcadeAction::Up,
            vec![
                InputSource::Key(ArcadeKey::Q),
                InputSource::Key(ArcadeKey::Up),
                InputSource::Key(ArcadeKey::W),
                InputSource::GamepadAxisPos(GamepadAxis::Throttle),
                InputSource::GamepadBtn(GamepadButton::DpadUp),
            ],
        );
        bindings.insert(
            ArcadeAction::Down,
            vec![
                InputSource::Key(ArcadeKey::A),
                InputSource::Key(ArcadeKey::Down),
                InputSource::Key(ArcadeKey::S),
                InputSource::GamepadAxisPos(GamepadAxis::Brake),
                InputSource::GamepadBtn(GamepadButton::DpadDown),
            ],
        );
        bindings.insert(
            ArcadeAction::Left,
            vec![
                InputSource::Key(ArcadeKey::O),
                InputSource::Key(ArcadeKey::Left),
                InputSource::Key(ArcadeKey::A),
                InputSource::GamepadAxisNeg(GamepadAxis::LeftStickX),
                InputSource::GamepadBtn(GamepadButton::DpadLeft),
            ],
        );
        bindings.insert(
            ArcadeAction::Right,
            vec![
                InputSource::Key(ArcadeKey::P),
                InputSource::Key(ArcadeKey::Right),
                InputSource::Key(ArcadeKey::D),
                InputSource::GamepadAxisPos(GamepadAxis::LeftStickX),
                InputSource::GamepadBtn(GamepadButton::DpadRight),
            ],
        );
        bindings.insert(
            ArcadeAction::Action3,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Primary,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Secondary,
            vec![
                InputSource::Key(ArcadeKey::LeftShift),
                InputSource::Key(ArcadeKey::E),
                InputSource::GamepadBtn(GamepadButton::West),
            ],
        );
        bindings.insert(
            ArcadeAction::Pause,
            vec![
                InputSource::Key(ArcadeKey::Escape),
                InputSource::GamepadBtn(GamepadButton::Start),
            ],
        );
        bindings.insert(
            ArcadeAction::Menu,
            vec![
                InputSource::Key(ArcadeKey::K),
                InputSource::Key(ArcadeKey::Tab),
                InputSource::GamepadBtn(GamepadButton::Back),
            ],
        );

        Self { bindings }
    }

    /// WASD racing preset (W=Gas, S=Brake, A=Left, D=Right, Space=Handbrake).
    pub fn wasd_racing() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(
            ArcadeAction::Up,
            vec![
                InputSource::Key(ArcadeKey::W),
                InputSource::GamepadAxisPos(GamepadAxis::Throttle),
            ],
        );
        bindings.insert(
            ArcadeAction::Down,
            vec![
                InputSource::Key(ArcadeKey::S),
                InputSource::GamepadAxisPos(GamepadAxis::Brake),
            ],
        );
        bindings.insert(
            ArcadeAction::Left,
            vec![
                InputSource::Key(ArcadeKey::A),
                InputSource::GamepadAxisNeg(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Right,
            vec![
                InputSource::Key(ArcadeKey::D),
                InputSource::GamepadAxisPos(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Action3,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Primary,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Secondary,
            vec![
                InputSource::Key(ArcadeKey::LeftShift),
                InputSource::GamepadBtn(GamepadButton::West),
            ],
        );
        bindings.insert(
            ArcadeAction::Pause,
            vec![
                InputSource::Key(ArcadeKey::Escape),
                InputSource::GamepadBtn(GamepadButton::Start),
            ],
        );
        bindings.insert(
            ArcadeAction::Menu,
            vec![
                InputSource::Key(ArcadeKey::K),
                InputSource::GamepadBtn(GamepadButton::Back),
            ],
        );
        Self { bindings }
    }

    /// Arrow keys racing preset.
    pub fn arrows_racing() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(
            ArcadeAction::Up,
            vec![
                InputSource::Key(ArcadeKey::Up),
                InputSource::GamepadAxisPos(GamepadAxis::Throttle),
            ],
        );
        bindings.insert(
            ArcadeAction::Down,
            vec![
                InputSource::Key(ArcadeKey::Down),
                InputSource::GamepadAxisPos(GamepadAxis::Brake),
            ],
        );
        bindings.insert(
            ArcadeAction::Left,
            vec![
                InputSource::Key(ArcadeKey::Left),
                InputSource::GamepadAxisNeg(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Right,
            vec![
                InputSource::Key(ArcadeKey::Right),
                InputSource::GamepadAxisPos(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Action3,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Primary,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Secondary,
            vec![
                InputSource::Key(ArcadeKey::RightControl),
                InputSource::GamepadBtn(GamepadButton::West),
            ],
        );
        bindings.insert(
            ArcadeAction::Pause,
            vec![
                InputSource::Key(ArcadeKey::Escape),
                InputSource::GamepadBtn(GamepadButton::Start),
            ],
        );
        bindings.insert(
            ArcadeAction::Menu,
            vec![
                InputSource::Key(ArcadeKey::K),
                InputSource::GamepadBtn(GamepadButton::Back),
            ],
        );
        Self { bindings }
    }

    /// Classic QAOP racing preset (Q=Gas, A=Brake, O=Left, P=Right, Space=Handbrake).
    pub fn classic_racing() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(
            ArcadeAction::Up,
            vec![
                InputSource::Key(ArcadeKey::Q),
                InputSource::GamepadAxisPos(GamepadAxis::Throttle),
            ],
        );
        bindings.insert(
            ArcadeAction::Down,
            vec![
                InputSource::Key(ArcadeKey::A),
                InputSource::GamepadAxisPos(GamepadAxis::Brake),
            ],
        );
        bindings.insert(
            ArcadeAction::Left,
            vec![
                InputSource::Key(ArcadeKey::O),
                InputSource::GamepadAxisNeg(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Right,
            vec![
                InputSource::Key(ArcadeKey::P),
                InputSource::GamepadAxisPos(GamepadAxis::LeftStickX),
            ],
        );
        bindings.insert(
            ArcadeAction::Action3,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Primary,
            vec![
                InputSource::Key(ArcadeKey::Space),
                InputSource::GamepadBtn(GamepadButton::South),
            ],
        );
        bindings.insert(
            ArcadeAction::Secondary,
            vec![
                InputSource::Key(ArcadeKey::LeftShift),
                InputSource::GamepadBtn(GamepadButton::West),
            ],
        );
        bindings.insert(
            ArcadeAction::Pause,
            vec![
                InputSource::Key(ArcadeKey::Escape),
                InputSource::GamepadBtn(GamepadButton::Start),
            ],
        );
        bindings.insert(
            ArcadeAction::Menu,
            vec![
                InputSource::Key(ArcadeKey::K),
                InputSource::GamepadBtn(GamepadButton::Back),
            ],
        );
        Self { bindings }
    }

    /// Queries if any bound input source for `action` is currently active.
    pub fn is_down(&self, action: ArcadeAction, gp: &GamepadSnapshot) -> bool {
        self.bindings
            .get(&action)
            .map_or(false, |sources| sources.iter().any(|s| s.is_down(gp)))
    }

    /// Queries if any bound keyboard key for `action` is currently held down.
    pub fn is_key_down(&self, action: ArcadeAction) -> bool {
        self.bindings
            .get(&action)
            .map_or(false, |sources| {
                sources.iter().any(|s| match s {
                    InputSource::Key(k) => k.to_key_code().map_or(false, safe_key_down),
                    _ => false,
                })
            })
    }

    /// Queries if any bound input source for `action` was triggered this frame.
    pub fn is_pressed(&self, action: ArcadeAction, gp: &GamepadSnapshot) -> bool {
        self.bindings
            .get(&action)
            .map_or(false, |sources| sources.iter().any(|s| s.is_pressed(gp)))
    }

    /// Queries if any bound keyboard key for `action` was pressed this frame.
    pub fn is_key_pressed(&self, action: ArcadeAction) -> bool {
        self.bindings
            .get(&action)
            .map_or(false, |sources| {
                sources.iter().any(|s| match s {
                    InputSource::Key(k) => k.to_key_code().map_or(false, safe_key_pressed),
                    _ => false,
                })
            })
    }

    /// Queries if any bound gamepad digital button for `action` is currently held down.
    pub fn is_gamepad_btn_down(&self, action: ArcadeAction, gp: &GamepadSnapshot) -> bool {
        self.bindings
            .get(&action)
            .map_or(false, |sources| {
                sources.iter().any(|s| match s {
                    InputSource::GamepadBtn(b) => InputSource::GamepadBtn(*b).is_down(gp),
                    _ => false,
                })
            })
    }

    /// Queries if any bound gamepad digital button for `action` was pressed this frame.
    pub fn is_gamepad_btn_pressed(&self, action: ArcadeAction, gp: &GamepadSnapshot) -> bool {
        self.bindings
            .get(&action)
            .map_or(false, |sources| {
                sources.iter().any(|s| match s {
                    InputSource::GamepadBtn(b) => InputSource::GamepadBtn(*b).is_pressed(gp),
                    _ => false,
                })
            })
    }

    /// Returns a human-friendly string representing the primary keyboard/gamepad key(s) bound to this action.
    pub fn primary_binding_label(&self, action: ArcadeAction) -> String {
        if let Some(sources) = self.bindings.get(&action) {
            let key_labels: Vec<_> = sources
                .iter()
                .filter_map(|s| match s {
                    InputSource::Key(k) => Some(k.label()),
                    _ => None,
                })
                .collect();
            if !key_labels.is_empty() {
                return key_labels.join(" / ");
            }
            if let Some(first) = sources.first() {
                return first.label();
            }
        }
        "Unbound".to_string()
    }

    /// Computes a composite 2D axis vector (e.g. for steering or character motion).
    pub fn axis_vector(
        &self,
        negative_x: ArcadeAction,
        positive_x: ArcadeAction,
        negative_y: ArcadeAction,
        positive_y: ArcadeAction,
        gp: &GamepadSnapshot,
    ) -> Vec2 {
        let mut x = 0.0;
        let mut y = 0.0;

        if self.is_down(negative_x, gp) {
            x -= 1.0;
        }
        if self.is_down(positive_x, gp) {
            x += 1.0;
        }
        if self.is_down(negative_y, gp) {
            y -= 1.0;
        }
        if self.is_down(positive_y, gp) {
            y += 1.0;
        }

        // Blend analog gamepad stick if active
        if gp.steer.abs() > 0.15 {
            x = gp.steer;
        }

        Vec2::new(x, y)
    }

    /// Returns the active sources bound to an action.
    pub fn get_bindings(&self, action: ArcadeAction) -> &[InputSource] {
        self.bindings
            .get(&action)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Rebinds an action to a new set of sources.
    pub fn set_bindings(&mut self, action: ArcadeAction, sources: Vec<InputSource>) {
        self.bindings.insert(action, sources);
    }

    /// Adds a binding source to an action.
    pub fn add_binding(&mut self, action: ArcadeAction, source: InputSource) {
        let list = self.bindings.entry(action).or_default();
        if !list.contains(&source) {
            list.push(source);
        }
    }

    /// Serializes configuration to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes configuration from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_map_defaults_and_serialization() {
        let map = InputMap::default_arcade();

        // Check that all standard actions have at least one binding
        for action in ArcadeAction::ALL {
            let bindings = map.get_bindings(action);
            assert!(
                !bindings.is_empty(),
                "Action {action:?} should have default bindings"
            );
        }

        // Check JSON serialization roundtrip
        let json = map.to_json().expect("Failed to serialize InputMap");
        let decoded = InputMap::from_json(&json).expect("Failed to deserialize InputMap");
        assert_eq!(map, decoded);
    }

    #[test]
    fn test_input_map_rebinding() {
        let mut map = InputMap::default_arcade();

        let new_sources = vec![InputSource::Key(ArcadeKey::F)];
        map.set_bindings(ArcadeAction::Primary, new_sources.clone());
        assert_eq!(map.get_bindings(ArcadeAction::Primary), new_sources.as_slice());

        map.add_binding(
            ArcadeAction::Primary,
            InputSource::GamepadBtn(GamepadButton::West),
        );
        assert_eq!(map.get_bindings(ArcadeAction::Primary).len(), 2);
    }

    #[test]
    fn test_input_map_axis_vector_with_gamepad() {
        let map = InputMap::default_arcade();
        let mut gp = GamepadSnapshot::default();

        let v0 = map.axis_vector(
            ArcadeAction::Left,
            ArcadeAction::Right,
            ArcadeAction::Up,
            ArcadeAction::Down,
            &gp,
        );
        assert_eq!(v0, Vec2::ZERO);

        gp.steer = 0.75;
        let v_gp = map.axis_vector(
            ArcadeAction::Left,
            ArcadeAction::Right,
            ArcadeAction::Up,
            ArcadeAction::Down,
            &gp,
        );
        assert!((v_gp.x - 0.75).abs() < 1e-4);
    }

    #[test]
    fn test_racing_presets_and_labels() {
        let racing = InputMap::default_racing();
        let wasd = InputMap::wasd_racing();
        let arrows = InputMap::arrows_racing();
        let classic = InputMap::classic_racing();

        assert!(!racing.get_bindings(ArcadeAction::Up).is_empty());
        assert_eq!(wasd.primary_binding_label(ArcadeAction::Up), "W");
        assert_eq!(arrows.primary_binding_label(ArcadeAction::Up), "Up Arrow");
        assert_eq!(classic.primary_binding_label(ArcadeAction::Up), "Q");

        // Verify headless query safety
        assert!(!racing.is_key_down(ArcadeAction::Up));
        assert!(!racing.is_key_pressed(ArcadeAction::Up));

        // Serialization roundtrip for racing map
        let json = racing.to_json().expect("Serialize racing map");
        let decoded = InputMap::from_json(&json).expect("Deserialize racing map");
        assert_eq!(racing, decoded);
    }
}
