//! Cross-platform Audio Engine for TDRace.

pub mod auxiliary_fx;
pub mod backend;
pub mod dsp;
pub mod engine_mixer;
pub mod manager;
pub mod proximity;
pub mod samples;
pub mod sfx;
pub mod synthwave;

pub use auxiliary_fx::AuxiliaryAudioLayer;
pub use backend::{ActiveSoundHandle, AudioBackend, SoundData};
pub use cabinet::audio::{AudioMixer, SoundBus};
pub use engine_mixer::{ActiveVoiceGroup, EngineAudioMixer};
pub use manager::{AudioManager, AudioSettings, EngineSoundType, MusicTrack, SfxType, SoundBank};
pub use proximity::{
    calculate_countdown_warmup_throttle, calculate_distance_attenuation, calculate_doppler_factor,
    calculate_spatial_audio, calculate_stereo_pan, select_top_k_audible_sources, DopplerConfig,
    EngineRpmModel, ProximityVoiceSlot, SpatialAudioResult, VehicleAudioSource, DEFAULT_MAX_DISTANCE,
    DEFAULT_MIN_DISTANCE, DEFAULT_PAN_RADIUS, MAX_PROXIMITY_VOICES,
};
pub use samples::{ArchetypeSampleBank, EngineSamplePoint, HIGH_RPM, IDLE_RPM, MID_RPM};
pub use sfx::EngineSoundConfig;
