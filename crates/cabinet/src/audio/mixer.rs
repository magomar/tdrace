use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use crate::audio::bus::AudioSettings;

/// Sound category bus routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundBus {
    Music,
    Sfx,
    Ui,
}

/// Generic multi-channel audio mixer.
pub struct AudioMixer {
    pub settings: AudioSettings,
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            settings: AudioSettings::default(),
        }
    }

    /// Plays a sound effect through the appropriate volume bus.
    pub fn play(&self, sound: Option<&Sound>, bus: SoundBus, volume_scale: f32) {
        if let Some(snd) = sound {
            let bus_vol = match bus {
                SoundBus::Music => self.settings.effective_music_volume(),
                SoundBus::Sfx => self.settings.effective_sfx_volume(),
                SoundBus::Ui => self.settings.effective_ui_volume(),
            };
            let effective_vol = (bus_vol * volume_scale).clamp(0.0, 1.0);
            if effective_vol > 1e-4 {
                let _ = std::panic::catch_unwind(|| {
                    play_sound(
                        snd,
                        PlaySoundParams {
                            looped: false,
                            volume: effective_vol,
                        },
                    );
                });
            }
        }
    }
}
