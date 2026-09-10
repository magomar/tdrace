//! High-performance Audio Backend powered by Kira with dynamic pitch-shifting and tweening.

use std::time::Duration;

#[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
use std::io::Cursor;

#[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
use kira::{
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween,
};

/// Converts linear amplitude [0.0..1.0] to Kira Decibels.
pub fn amplitude_to_db(amp: f32) -> f32 {
    if amp <= 0.001 {
        -60.0
    } else {
        (20.0 * amp.min(2.0).log10()).max(-60.0)
    }
}

/// Handle to an actively playing sound instance supporting real-time pitch and volume modulation.
pub struct ActiveSoundHandle {
    #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
    handle: Option<StaticSoundHandle>,
}

impl ActiveSoundHandle {
    pub fn new_empty() -> Self {
        Self {
            #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
            handle: None,
        }
    }

    #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
    pub fn from_handle(handle: StaticSoundHandle) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    /// Dynamically alters playback rate / pitch with smooth tweening.
    /// 1.0 = original pitch, 2.0 = 1 octave up (double frequency), 0.5 = 1 octave down.
    pub fn set_playback_rate(&mut self, rate: f32, tween_duration: Duration) {
        #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
        if let Some(h) = self.handle.as_mut() {
            h.set_playback_rate(
                rate as f64,
                Tween {
                    duration: tween_duration,
                    ..Default::default()
                },
            );
        }
        #[cfg(not(all(feature = "kira", not(target_arch = "wasm32"))))]
        let _ = (rate, tween_duration);
    }

    /// Dynamically alters volume in linear amplitude [0.0..1.0] with smooth tweening.
    pub fn set_volume(&mut self, volume_linear: f32, tween_duration: Duration) {
        #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
        if let Some(h) = self.handle.as_mut() {
            let db = amplitude_to_db(volume_linear);
            h.set_volume(
                Decibels(db),
                Tween {
                    duration: tween_duration,
                    ..Default::default()
                },
            );
        }

        #[cfg(not(all(feature = "kira", not(target_arch = "wasm32"))))]
        let _ = (volume_linear, tween_duration);
    }

    /// Stops the sound with an optional fade-out tween.
    pub fn stop(&mut self, fade_duration: Duration) {
        #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
        if let Some(mut h) = self.handle.take() {
            h.stop(Tween {
                duration: fade_duration,
                ..Default::default()
            });
        }

        #[cfg(not(all(feature = "kira", not(target_arch = "wasm32"))))]
        let _ = fade_duration;
    }
}

/// Loaded sound asset ready to be played.
#[derive(Clone)]
pub struct SoundData {
    #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
    pub(crate) data: Option<StaticSoundData>,
}

impl SoundData {
    /// Decodes audio from raw WAV, OGG, or MP3 bytes.
    pub fn from_bytes(bytes: &[u8], looped: bool) -> Result<Self, String> {
        #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
        {
            let mut sound_data = StaticSoundData::from_cursor(Cursor::new(bytes))
                .map_err(|e| format!("Failed to decode audio bytes: {e}"))?;
            if looped {
                sound_data = sound_data.loop_region(0.0..);
            }
            Ok(Self {
                data: Some(sound_data),
            })
        }
        #[cfg(not(all(feature = "kira", not(target_arch = "wasm32"))))]
        {
            let _ = (bytes, looped);
            Ok(Self {})
        }
    }

    pub fn empty() -> Self {
        Self {
            #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
            data: None,
        }
    }
}

/// Master audio backend coordinating device audio output.
pub struct AudioBackend {
    #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
    manager: Option<std::sync::Mutex<AudioManager<DefaultBackend>>>,
}

impl Default for AudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend {
    /// Creates and initializes the audio backend.
    /// Gracefully handles absent audio hardware (e.g. headless CI environments).
    pub fn new() -> Self {
        #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
        {
            let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .ok()
                .map(std::sync::Mutex::new);
            Self { manager }
        }
        #[cfg(not(all(feature = "kira", not(target_arch = "wasm32"))))]
        {
            Self {}
        }
    }

    /// Checks whether an active hardware audio device is connected.
    pub fn is_available(&self) -> bool {
        #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
        {
            self.manager.is_some()
        }
        #[cfg(not(all(feature = "kira", not(target_arch = "wasm32"))))]
        {
            false
        }
    }

    /// Plays a sound asset with specified initial volume and playback rate.
    pub fn play(&self, sound: &SoundData, initial_volume: f32, initial_pitch: f32) -> ActiveSoundHandle {
        #[cfg(all(feature = "kira", not(target_arch = "wasm32")))]
        if let (Some(mutex), Some(data)) = (self.manager.as_ref(), sound.data.as_ref()) {
            if let Ok(mut mgr) = mutex.lock() {
                let mut sound_copy = data.clone();
                let db = amplitude_to_db(initial_volume);
                sound_copy.settings.volume = Decibels(db).into();
                sound_copy.settings.playback_rate = (initial_pitch as f64).into();

                if let Ok(handle) = mgr.play(sound_copy) {
                    return ActiveSoundHandle::from_handle(handle);
                }
            }
        }
        #[cfg(not(all(feature = "kira", not(target_arch = "wasm32"))))]
        let _ = (sound, initial_volume, initial_pitch);
        ActiveSoundHandle::new_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::sfx::generate_ui_select;

    #[test]
    fn test_amplitude_to_db() {
        assert_eq!(amplitude_to_db(0.0), -60.0);
        assert_eq!(amplitude_to_db(0.0001), -60.0);
        assert!((amplitude_to_db(1.0) - 0.0).abs() < 0.01);
        assert!((amplitude_to_db(0.5) - (-6.02)).abs() < 0.1);
    }

    #[test]
    fn test_sound_data_decoding_and_active_handle() {
        let wav = generate_ui_select(44100);
        let sound = SoundData::from_bytes(&wav, false).expect("WAV decode failed");

        let backend = AudioBackend::new();
        let mut handle = backend.play(&sound, 0.8, 1.0);
        handle.set_volume(0.5, Duration::from_millis(50));
        handle.set_playback_rate(1.2, Duration::from_millis(50));
        handle.stop(Duration::from_millis(100));
    }
}
