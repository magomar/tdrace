//! # Cabinet LAN Multiplayer Arcade UI Module
//!
//! Exposes reusable arcade lobby screens, server browser, direct IP keypad,
//! and synchronized starting grid countdown widgets.

pub mod client_lobby_screen;
pub mod host_screen;
pub mod ip_keypad;
pub mod join_screen;

pub use client_lobby_screen::CabinetLanClientLobbyScreen;
pub use host_screen::CabinetLanHostScreen;
pub use ip_keypad::{IpKeypad, IpKeypadAction, KEYPAD_GRID};
pub use join_screen::CabinetLanJoinScreen;

use macroquad::color::Color;

use crate::ui::theme::Palette;

/// Request from a LAN lobby screen that the game open one of its own screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanLobbyRequest {
    /// Host asks for the game's circuit selector.
    PickCircuit,
    /// Local player asks for the game's car selector.
    PickCar,
}

/// Livery options shared by the host and client lobbies: (id, display_name, color).
pub const LAN_LIVERIES: [(&str, &str, Color); 9] = [
    ("corsa_red", "Rosso Corsa", Palette::NEON_RED),
    ("matte_cyan", "Matte Cyan", Palette::NEON_CYAN),
    ("viper_green", "Viper Green", Palette::NEON_GREEN),
    ("speed_yellow", "Speed Yellow", Palette::NEON_GOLD),
    ("sunset_orange", "Sunset Orange", Palette::NEON_ORANGE),
    ("synthwave_purple", "Synthwave Purple", Palette::NEON_MAGENTA),
    ("stealth_black", "Stealth Black", Color::new(0.20, 0.22, 0.28, 1.0)),
    ("glacier_white", "Glacier White", Palette::WHITE),
    ("cyber_magenta", "Cyber Magenta", Color::new(0.90, 0.15, 0.60, 1.0)),
];

/// Index of a livery id in `LAN_LIVERIES`, or 0 when unknown.
pub fn lan_livery_index(livery_id: &str) -> usize {
    LAN_LIVERIES.iter().position(|(id, _, _)| *id == livery_id).unwrap_or(0)
}

/// Display name of a livery id, or the id itself when unknown.
pub fn lan_livery_name(livery_id: &str) -> &str {
    LAN_LIVERIES
        .iter()
        .find(|(id, _, _)| *id == livery_id)
        .map(|(_, name, _)| *name)
        .unwrap_or(livery_id)
}

/// Default label for a car or track id: the id itself.
pub(crate) fn id_label(id: &str) -> String {
    id.to_string()
}
