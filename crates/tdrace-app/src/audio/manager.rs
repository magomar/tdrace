//! Central Audio Manager and Mixer coordinating SoundBank, Music, and Dynamic SFX.

use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
    PlaySoundParams, Sound,
};
use serde::{Deserialize, Serialize};

use crate::audio::sfx::{
    generate_car_hit_sound, generate_countdown_high, generate_countdown_low,
    generate_curb_rumble_sound, generate_engine_rpm_band, generate_engine_sound,
    generate_gear_shift_pop, generate_lap_chime, generate_offroad_sound,
    generate_race_finish, generate_sector_ping, generate_skid_sound,
    generate_ui_move, generate_ui_select, generate_wall_crash_sound,
};
use crate::audio::synthwave::{generate_menu_theme, generate_nightcall_race_theme};
use crate::audio::dsp::DEFAULT_SAMPLE_RATE;

/// Number of discrete RPM harmonic frequency bands for fine microtonal engine simulation.
pub const NUM_RPM_BANDS: usize = 28;

/// Engine RPM band center values from 850 RPM (idle) to 9,300 RPM (redline).
pub const RPM_BAND_RPMS: [f32; NUM_RPM_BANDS] = [
    850.0, 1000.0, 1150.0, 1350.0, 1550.0, 1800.0, 2050.0, 2350.0,
    2650.0, 3000.0, 3350.0, 3700.0, 4100.0, 4500.0, 4900.0, 5300.0,
    5700.0, 6100.0, 6500.0, 6850.0, 7200.0, 7500.0, 7800.0, 8100.0,
    8400.0, 8700.0, 9000.0, 9300.0,
];

/// Engine cylinder firing fundamental frequencies (Hz) corresponding to RPM bands.
/// For a 6-cylinder 4-stroke engine: Freq (Hz) = RPM * 3 / 60 = RPM / 20.
pub const RPM_BAND_FREQS: [f32; NUM_RPM_BANDS] = [
    42.5, 50.0, 57.5, 67.5, 77.5, 90.0, 102.5, 117.5,
    132.5, 150.0, 167.5, 185.0, 205.0, 225.0, 245.0, 265.0,
    285.0, 305.0, 325.0, 342.5, 360.0, 375.0, 390.0, 405.0,
    420.0, 435.0, 450.0, 465.0,
];

/// Audio Volume & Mute Settings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub is_muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.85,
            music_volume: 0.70,
            sfx_volume: 0.90,
            is_muted: false,
        }
    }
}

impl AudioSettings {
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    #[inline]
    pub fn effective_music_volume(&self) -> f32 {
        if self.is_muted {
            0.0
        } else {
            (self.master_volume * self.music_volume).clamp(0.0, 1.0)
        }
    }

    #[inline]
    pub fn effective_sfx_volume(&self, sfx_gain: f32) -> f32 {
        if self.is_muted {
            0.0
        } else {
            (self.master_volume * self.sfx_volume * sfx_gain).clamp(0.0, 1.0)
        }
    }
}

/// Music Track Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicTrack {
    NightcallRace,
    NeonMenu,
}

/// Sound Effect Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfxType {
    Engine,
    ShiftPop,
    Skid,
    WallCrash,
    CarHit,
    Curb,
    Offroad,
    CountdownLow,
    CountdownHigh,
    LapChime,
    SectorPing,
    UiSelect,
    UiMove,
    RaceFinish,
}

/// Loaded Sound Handles Cache.
pub struct SoundBank {
    pub music_nightcall: Option<Sound>,
    pub music_menu: Option<Sound>,
    pub engine_rpm_bands: [Option<Sound>; NUM_RPM_BANDS],
    pub sfx_shift_pop: Option<Sound>,
    pub sfx_engine: Option<Sound>,
    pub sfx_skid: Option<Sound>,
    pub sfx_wall_crash: Option<Sound>,
    pub sfx_car_hit: Option<Sound>,
    pub sfx_curb: Option<Sound>,
    pub sfx_offroad: Option<Sound>,
    pub sfx_cd_low: Option<Sound>,
    pub sfx_cd_high: Option<Sound>,
    pub sfx_lap: Option<Sound>,
    pub sfx_sector: Option<Sound>,
    pub sfx_ui_select: Option<Sound>,
    pub sfx_ui_move: Option<Sound>,
    pub sfx_finish: Option<Sound>,
}

impl SoundBank {
    pub fn empty() -> Self {
        Self {
            music_nightcall: None,
            music_menu: None,
            engine_rpm_bands: [const { None }; NUM_RPM_BANDS],
            sfx_shift_pop: None,
            sfx_engine: None,
            sfx_skid: None,
            sfx_wall_crash: None,
            sfx_car_hit: None,
            sfx_curb: None,
            sfx_offroad: None,
            sfx_cd_low: None,
            sfx_cd_high: None,
            sfx_lap: None,
            sfx_sector: None,
            sfx_ui_select: None,
            sfx_ui_move: None,
            sfx_finish: None,
        }
    }

    /// Asynchronously loads and registers all procedural WAV sound assets into memory.
    pub async fn load_all() -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE;

        let nightcall_wav = generate_nightcall_race_theme(sample_rate);
        let menu_wav = generate_menu_theme(sample_rate);

        // Pre-generate 28 harmonic RPM frequency bands for continuous pitch crossfading
        let mut rpm_bands = [const { None }; NUM_RPM_BANDS];
        for (idx, &freq) in RPM_BAND_FREQS.iter().enumerate() {
            let wav = generate_engine_rpm_band(sample_rate, freq);
            rpm_bands[idx] = load_sound_from_bytes(&wav).await.ok();
        }

        let shift_pop_wav = generate_gear_shift_pop(sample_rate);
        let engine_wav = generate_engine_sound(sample_rate);
        let skid_wav = generate_skid_sound(sample_rate);
        let wall_crash_wav = generate_wall_crash_sound(sample_rate);
        let car_hit_wav = generate_car_hit_sound(sample_rate);
        let curb_wav = generate_curb_rumble_sound(sample_rate);
        let offroad_wav = generate_offroad_sound(sample_rate);
        let cd_low_wav = generate_countdown_low(sample_rate);
        let cd_high_wav = generate_countdown_high(sample_rate);
        let lap_wav = generate_lap_chime(sample_rate);
        let sector_wav = generate_sector_ping(sample_rate);
        let ui_sel_wav = generate_ui_select(sample_rate);
        let ui_mov_wav = generate_ui_move(sample_rate);
        let finish_wav = generate_race_finish(sample_rate);

        Self {
            music_nightcall: load_sound_from_bytes(&nightcall_wav).await.ok(),
            music_menu: load_sound_from_bytes(&menu_wav).await.ok(),
            engine_rpm_bands: rpm_bands,
            sfx_shift_pop: load_sound_from_bytes(&shift_pop_wav).await.ok(),
            sfx_engine: load_sound_from_bytes(&engine_wav).await.ok(),
            sfx_skid: load_sound_from_bytes(&skid_wav).await.ok(),
            sfx_wall_crash: load_sound_from_bytes(&wall_crash_wav).await.ok(),
            sfx_car_hit: load_sound_from_bytes(&car_hit_wav).await.ok(),
            sfx_curb: load_sound_from_bytes(&curb_wav).await.ok(),
            sfx_offroad: load_sound_from_bytes(&offroad_wav).await.ok(),
            sfx_cd_low: load_sound_from_bytes(&cd_low_wav).await.ok(),
            sfx_cd_high: load_sound_from_bytes(&cd_high_wav).await.ok(),
            sfx_lap: load_sound_from_bytes(&lap_wav).await.ok(),
            sfx_sector: load_sound_from_bytes(&sector_wav).await.ok(),
            sfx_ui_select: load_sound_from_bytes(&ui_sel_wav).await.ok(),
            sfx_ui_move: load_sound_from_bytes(&ui_mov_wav).await.ok(),
            sfx_finish: load_sound_from_bytes(&finish_wav).await.ok(),
        }
    }

    pub fn get_sound(&self, sfx: SfxType) -> Option<&Sound> {
        match sfx {
            SfxType::Engine => self.sfx_engine.as_ref(),
            SfxType::ShiftPop => self.sfx_shift_pop.as_ref(),
            SfxType::Skid => self.sfx_skid.as_ref(),
            SfxType::WallCrash => self.sfx_wall_crash.as_ref(),
            SfxType::CarHit => self.sfx_car_hit.as_ref(),
            SfxType::Curb => self.sfx_curb.as_ref(),
            SfxType::Offroad => self.sfx_offroad.as_ref(),
            SfxType::CountdownLow => self.sfx_cd_low.as_ref(),
            SfxType::CountdownHigh => self.sfx_cd_high.as_ref(),
            SfxType::LapChime => self.sfx_lap.as_ref(),
            SfxType::SectorPing => self.sfx_sector.as_ref(),
            SfxType::UiSelect => self.sfx_ui_select.as_ref(),
            SfxType::UiMove => self.sfx_ui_move.as_ref(),
            SfxType::RaceFinish => self.sfx_finish.as_ref(),
        }
    }
}

/// Master Audio System Coordinator.
pub struct AudioManager {
    pub settings: AudioSettings,
    pub bank: SoundBank,
    pub current_music: Option<MusicTrack>,
    pub is_engine_active: bool,
    pub is_skid_active: bool,
    pub engine_active_bands: [bool; NUM_RPM_BANDS],
    prev_skid_vol: f32,
    limiter_timer: f32,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Safe wrapper around macroquad play_sound that handles uninitialized headless contexts.
fn safe_play_sound(sound: &Sound, params: PlaySoundParams) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        play_sound(sound, params);
    }));
}

/// Safe wrapper around macroquad set_sound_volume.
fn safe_set_sound_volume(sound: &Sound, volume: f32) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        set_sound_volume(sound, volume);
    }));
}

/// Safe wrapper around macroquad stop_sound.
fn safe_stop_sound(sound: &Sound) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        stop_sound(sound);
    }));
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            settings: AudioSettings::default(),
            bank: SoundBank::empty(),
            current_music: None,
            is_engine_active: false,
            is_skid_active: false,
            engine_active_bands: [false; NUM_RPM_BANDS],
            prev_skid_vol: 0.0,
            limiter_timer: 0.0,
        }
    }

    /// Initializes and loads all audio banks asynchronously.
    pub async fn init_async(&mut self) {
        self.bank = SoundBank::load_all().await;
        if let Some(track) = self.current_music {
            let vol = self.settings.effective_music_volume();
            let sound = match track {
                MusicTrack::NightcallRace => self.bank.music_nightcall.as_ref(),
                MusicTrack::NeonMenu => self.bank.music_menu.as_ref(),
            };
            if let Some(s) = sound {
                safe_play_sound(
                    s,
                    PlaySoundParams {
                        looped: true,
                        volume: vol,
                    },
                );
            }
        }
    }

    /// Toggles audio mute state and applies immediate volume updates to active channels.
    pub fn toggle_mute(&mut self) {
        self.settings.toggle_mute();
        self.sync_music_volume();
        if self.settings.is_muted {
            self.stop_all_loops();
        }
    }

    /// Adjusts master volume level.
    pub fn set_master_volume(&mut self, vol: f32) {
        self.settings.master_volume = vol.clamp(0.0, 1.0);
        self.sync_music_volume();
    }

    /// Adjusts music volume level.
    pub fn set_music_volume(&mut self, vol: f32) {
        self.settings.music_volume = vol.clamp(0.0, 1.0);
        self.sync_music_volume();
    }

    /// Adjusts SFX volume level.
    pub fn set_sfx_volume(&mut self, vol: f32) {
        self.settings.sfx_volume = vol.clamp(0.0, 1.0);
    }

    /// Plays a background music track with looping.
    pub fn play_music(&mut self, track: MusicTrack) {
        if self.current_music == Some(track) {
            return;
        }

        self.stop_music();
        self.current_music = Some(track);

        let vol = self.settings.effective_music_volume();
        let sound = match track {
            MusicTrack::NightcallRace => self.bank.music_nightcall.as_ref(),
            MusicTrack::NeonMenu => self.bank.music_menu.as_ref(),
        };

        if let Some(s) = sound {
            safe_play_sound(
                s,
                PlaySoundParams {
                    looped: true,
                    volume: vol,
                },
            );
        }
    }

    /// Synchronizes current music channel volume with settings.
    pub fn sync_music_volume(&self) {
        let vol = self.settings.effective_music_volume();
        if let Some(track) = self.current_music {
            let sound = match track {
                MusicTrack::NightcallRace => self.bank.music_nightcall.as_ref(),
                MusicTrack::NeonMenu => self.bank.music_menu.as_ref(),
            };
            if let Some(s) = sound {
                safe_set_sound_volume(s, vol);
            }
        }
    }

    /// Stops currently playing music.
    pub fn stop_music(&mut self) {
        if let Some(track) = self.current_music {
            let sound = match track {
                MusicTrack::NightcallRace => self.bank.music_nightcall.as_ref(),
                MusicTrack::NeonMenu => self.bank.music_menu.as_ref(),
            };
            if let Some(s) = sound {
                safe_stop_sound(s);
            }
            self.current_music = None;
        }
    }

    /// Plays a one-shot sound effect at standard gain.
    pub fn play_sfx(&self, sfx: SfxType) {
        self.play_sfx_with_gain(sfx, 1.0);
    }

    /// Plays a one-shot sound effect with custom volume multiplier.
    pub fn play_sfx_with_gain(&self, sfx: SfxType, gain: f32) {
        let vol = self.settings.effective_sfx_volume(gain);
        if vol <= 0.001 {
            return;
        }

        if let Some(sound) = self.bank.get_sound(sfx) {
            safe_play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: vol,
                },
            );
        }
    }

    /// Dynamically crossfades multi-harmonic engine RPM sound bands, reflects throttle load, and triggers shift pops.
    pub fn update_engine_rpm(&mut self, rpm: f32, throttle: f32, is_shift: bool) {
        if self.settings.is_muted {
            self.stop_all_loops();
            return;
        }

        if is_shift {
            self.play_sfx_with_gain(SfxType::ShiftPop, 0.95);
        }

        let min_rpm = RPM_BAND_RPMS[0];
        let max_rpm = RPM_BAND_RPMS[NUM_RPM_BANDS - 1];
        let clamped_rpm = rpm.clamp(min_rpm, max_rpm);

        // Equal-power crossfade across fine microtonal RPM bands
        let mut weights = [0.0f32; NUM_RPM_BANDS];
        if clamped_rpm <= min_rpm {
            weights[0] = 1.0;
        } else if clamped_rpm >= max_rpm {
            weights[NUM_RPM_BANDS - 1] = 1.0;
        } else {
            for i in 0..(NUM_RPM_BANDS - 1) {
                let low_rpm = RPM_BAND_RPMS[i];
                let high_rpm = RPM_BAND_RPMS[i + 1];
                if clamped_rpm >= low_rpm && clamped_rpm <= high_rpm {
                    let u = ((clamped_rpm - low_rpm) / (high_rpm - low_rpm)).clamp(0.0, 1.0);
                    // Equal-power crossfade maintaining constant acoustic energy
                    let angle = u * std::f32::consts::FRAC_PI_2;
                    weights[i] = angle.cos();
                    weights[i + 1] = angle.sin();
                    break;
                }
            }
        }

        // Engine volume reflecting throttle demand: wide-open intake roar vs engine braking overrun
        let load_factor = throttle.max(0.0);
        let base_gain = 0.42 + 0.58 * load_factor;

        // Rev limiter bounce stutter at redline with high throttle
        let limiter_mod = if clamped_rpm >= 7700.0 && throttle > 0.5 {
            self.limiter_timer = (self.limiter_timer + 0.12).fract();
            if self.limiter_timer < 0.5 { 1.0 } else { 0.60 }
        } else {
            1.0
        };

        let total_vol = self.settings.effective_sfx_volume(base_gain * limiter_mod);

        for (idx, sound_opt) in self.bank.engine_rpm_bands.iter().enumerate() {
            if let Some(sound) = sound_opt {
                let band_vol = total_vol * weights[idx];
                if band_vol > 0.001 {
                    if !self.engine_active_bands[idx] {
                        safe_play_sound(
                            sound,
                            PlaySoundParams {
                                looped: true,
                                volume: band_vol,
                            },
                        );
                        self.engine_active_bands[idx] = true;
                    } else {
                        safe_set_sound_volume(sound, band_vol);
                    }
                } else if self.engine_active_bands[idx] {
                    safe_stop_sound(sound);
                    self.engine_active_bands[idx] = false;
                }
            }
        }
        self.is_engine_active = true;
    }

    /// Triggers crisp arcade tire drift chirp when breaking traction.
    pub fn update_skid_chirp(&mut self, slip_intensity: f32, dt: f32) {
        if self.settings.is_muted {
            return;
        }

        self.prev_skid_vol = (self.prev_skid_vol - dt).max(0.0);
        if slip_intensity > 0.35 && self.prev_skid_vol <= 0.0 {
            let gain = (slip_intensity * 0.8).clamp(0.3, 0.85);
            self.play_sfx_with_gain(SfxType::Skid, gain);
            self.prev_skid_vol = 0.18; // Cooldown between chirps
        }
    }

    /// Stops all continuous loops (engine bands).
    pub fn stop_all_loops(&mut self) {
        for (idx, sound_opt) in self.bank.engine_rpm_bands.iter().enumerate() {
            if let Some(sound) = sound_opt {
                if self.engine_active_bands[idx] {
                    safe_set_sound_volume(sound, 0.0);
                    safe_stop_sound(sound);
                    self.engine_active_bands[idx] = false;
                }
            }
        }
        self.is_engine_active = false;
        self.is_skid_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_settings_volume_math() {
        let mut settings = AudioSettings {
            master_volume: 0.8,
            music_volume: 0.5,
            sfx_volume: 0.9,
            is_muted: false,
        };

        assert!((settings.effective_music_volume() - 0.4).abs() < 0.01);
        assert!((settings.effective_sfx_volume(1.0) - 0.72).abs() < 0.01);

        settings.toggle_mute();
        assert_eq!(settings.effective_music_volume(), 0.0);
        assert_eq!(settings.effective_sfx_volume(1.0), 0.0);
    }

    #[test]
    fn test_audio_manager_initialization_defaults() {
        let mgr = AudioManager::new();
        assert_eq!(mgr.settings.is_muted, false);
        assert_eq!(mgr.current_music, None);
        assert_eq!(mgr.is_engine_active, false);
        assert_eq!(mgr.is_skid_active, false);
    }
}
