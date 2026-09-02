//! # Cabinet
//!
//! An opinionated, lightweight 2D Arcade Game Shell & Interaction Design System for Rust games.
//!
//! ## Subsystems
//! - [`ui`]: Resolution-independent `UiScaler`, TrueType `Fonts`, glassmorphism cards, design tokens, and swappable `CabinetTheme`.
//! - [`input`]: Universal 2D orthogonal navigation (`NavGrid2D`), digital input smoothing filter, and `GamepadManager`.
//! - [`fx`]: Game-feel juice primitives (`ScreenShake`, `ScreenFlash`, and `HitStop`).
//! - [`state`]: Modular `ScreenStack` & `CabinetScreen` lifecycle traits with built-in `UniversalPauseModal`.
//! - [`profile`]: `PlayerProfile`, multi-slot `ProfileManager`, country flag banners, and color presets.
//! - [`records`]: `HallOfFame`, `RecordDatabase`, and persistent leaderboards.
//! - [`audio`]: Multi-channel `AudioMixer` with 4 volume buses (`Master`, `Music`, `Sfx`, `Ui`).

pub mod audio;
pub mod fx;
pub mod input;
pub mod profile;
pub mod records;
pub mod state;
pub mod ui;

// Top-level convenient re-exports
pub use audio::{AudioMixer, AudioSettings, SoundBus};
pub use fx::{HitStop, ScreenFlash, ScreenShake};
pub use input::{DigitalInputConfig, DigitalInputFilter, GamepadConfig, GamepadManager, GamepadSnapshot, NavGrid2D};
pub use profile::{ColorScheme, CountryInfo, CountryRegistry, PlayerProfile, ProfileManager};
pub use records::{HallOfFame, RecordDatabase, RecordEntry, RecordMetric};
pub use state::{CabinetContext, CabinetScreen, ScreenAction, ScreenStack, UniversalPauseModal};
pub use ui::{CabinetTheme, Fonts, Palette, UiScaler};
