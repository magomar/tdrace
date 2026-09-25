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
