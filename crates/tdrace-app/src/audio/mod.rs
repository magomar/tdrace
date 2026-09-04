//! Cross-platform Audio Engine for TDRace.

pub mod dsp;
pub mod manager;
pub mod sfx;
pub mod synthwave;

pub use cabinet::audio::{AudioMixer, SoundBus};
pub use manager::{AudioManager, AudioSettings, EngineSoundType, MusicTrack, SfxType, SoundBank};
pub use sfx::EngineSoundConfig;
