//! Cross-platform Audio Engine for TDRace.

pub mod dsp;
pub mod manager;
pub mod sfx;
pub mod synthwave;

pub use manager::{AudioManager, AudioSettings, MusicTrack, SfxType, SoundBank};
