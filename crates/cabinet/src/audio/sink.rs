//! Audio Sink abstraction and procedural arcade audio player for Cabinet.

use std::collections::HashMap;
use crate::audio::backend::{ActiveSoundHandle, AudioBackend, SoundData};
use crate::audio::bus::AudioSettings;
use crate::audio::sfx::{
    generate_car_hit_sound, generate_countdown_high, generate_countdown_low, generate_lap_chime,
    generate_race_finish, generate_sector_ping, generate_skid_sound, generate_ui_move,
    generate_ui_select, generate_wall_crash_sound, SoundCue,
};

/// Trait implemented by any audio subsystem capable of receiving high-level game & UI cues.
pub trait CabinetAudioSink {
    /// Plays a sound cue through the audio sink.
    fn play_cue(&self, cue: SoundCue);

    /// Convenience shortcut for UI confirm/select blip.
    fn play_ui_select(&self) {
        self.play_cue(SoundCue::UiSelect);
    }

    /// Convenience shortcut for UI navigation tick.
    fn play_ui_move(&self) {
        self.play_cue(SoundCue::UiMove);
    }

    /// Convenience shortcut for UI cancel / back action.
    fn play_ui_cancel(&self) {
        self.play_cue(SoundCue::UiCancel);
    }
}

/// Ready-to-use procedural arcade audio player for Cabinet games and shell menus.
pub struct CabinetAudioPlayer {
    pub backend: AudioBackend,
    pub settings: AudioSettings,
    sounds: HashMap<SoundCue, SoundData>,
}

impl Default for CabinetAudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl CabinetAudioPlayer {
    /// Creates and initializes the procedural arcade player with all default sound cues synthesized in-memory.
    pub fn new() -> Self {
        let backend = AudioBackend::new();
        let settings = AudioSettings::default();
        let mut sounds = HashMap::new();

        let sample_rate = 44100;
        let load_cue = |wav: Vec<u8>| SoundData::from_bytes(&wav, false).unwrap_or_else(|_| SoundData::empty());

        sounds.insert(SoundCue::UiSelect, load_cue(generate_ui_select(sample_rate)));
        sounds.insert(SoundCue::UiMove, load_cue(generate_ui_move(sample_rate)));
        sounds.insert(SoundCue::UiCancel, load_cue(generate_ui_move(sample_rate)));
        sounds.insert(SoundCue::CountdownLow, load_cue(generate_countdown_low(sample_rate)));
        sounds.insert(SoundCue::CountdownHigh, load_cue(generate_countdown_high(sample_rate)));
        sounds.insert(SoundCue::LapChime, load_cue(generate_lap_chime(sample_rate)));
        sounds.insert(SoundCue::SectorPing, load_cue(generate_sector_ping(sample_rate)));
        sounds.insert(SoundCue::RaceFinish, load_cue(generate_race_finish(sample_rate)));
        sounds.insert(SoundCue::Skid, load_cue(generate_skid_sound(sample_rate)));
        sounds.insert(SoundCue::ImpactLight, load_cue(generate_car_hit_sound(sample_rate)));
        sounds.insert(SoundCue::ImpactHeavy, load_cue(generate_wall_crash_sound(sample_rate)));

        Self {
            backend,
            settings,
            sounds,
        }
    }

    /// Plays a sound cue scaled by effective volume bus settings.
    pub fn play_cue(&self, cue: SoundCue) -> ActiveSoundHandle {
        if let Some(sound) = self.sounds.get(&cue) {
            let vol = match cue {
                SoundCue::UiSelect | SoundCue::UiMove | SoundCue::UiCancel => self.settings.effective_ui_volume(),
                _ => self.settings.effective_sfx_volume(),
            };
            self.backend.play(sound, vol, 1.0)
        } else {
            ActiveSoundHandle::new_empty()
        }
    }

    /// Registers a custom sound asset for a custom cue name.
    pub fn register_custom_cue(&mut self, name: &'static str, sound: SoundData) {
        self.sounds.insert(SoundCue::Custom(name), sound);
    }
}

impl CabinetAudioSink for CabinetAudioPlayer {
    fn play_cue(&self, cue: SoundCue) {
        let _ = CabinetAudioPlayer::play_cue(self, cue);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestMockSink {
        call_count: AtomicUsize,
    }

    impl CabinetAudioSink for TestMockSink {
        fn play_cue(&self, _cue: SoundCue) {
            self.call_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_mock_audio_sink() {
        let sink = TestMockSink {
            call_count: AtomicUsize::new(0),
        };
        sink.play_ui_select();
        sink.play_ui_move();
        sink.play_ui_cancel();
        assert_eq!(sink.call_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_cabinet_audio_player_initialization_and_cues() {
        let player = CabinetAudioPlayer::new();
        player.play_ui_select();
        player.play_ui_move();
        player.play_cue(SoundCue::LapChime);
        player.play_cue(SoundCue::RaceFinish);
    }
}
