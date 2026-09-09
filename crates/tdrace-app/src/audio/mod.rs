//! Cross-platform Audio Engine for TDRace.

pub mod auxiliary_fx;
pub mod backend;
pub mod dsp;
pub mod engine_mixer;
pub mod manager;
pub mod samples;
pub mod sfx;
pub mod synthwave;

pub use auxiliary_fx::AuxiliaryAudioLayer;
pub use backend::{ActiveSoundHandle, AudioBackend, SoundData};
pub use cabinet::audio::{AudioMixer, SoundBus};
pub use engine_mixer::{ActiveVoiceGroup, EngineAudioMixer};
pub use manager::{AudioManager, AudioSettings, EngineSoundType, MusicTrack, SfxType, SoundBank};
pub use samples::{ArchetypeSampleBank, EngineSamplePoint, HIGH_RPM, IDLE_RPM, MID_RPM};
pub use sfx::EngineSoundConfig;
