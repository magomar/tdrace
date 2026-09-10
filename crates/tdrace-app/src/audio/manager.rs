//! Central Audio Manager and Mixer coordinating SoundBank, Music, and Dynamic SFX.

use std::collections::HashMap;
use std::time::Duration;
use macroquad::audio::{PlaySoundParams, Sound};
#[cfg(target_arch = "wasm32")]
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound};

/// Safe wrapper around macroquad load_sound_from_bytes that avoids uninitialized macroquad window contexts on native desktop.
async fn safe_load_sound_from_bytes(data: &[u8]) -> Option<Sound> {
    #[cfg(target_arch = "wasm32")]
    {
        load_sound_from_bytes(data).await.ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = data;
        None
    }
}
use serde::{Deserialize, Serialize};

use crate::audio::sfx::{
    generate_car_hit_sound, generate_countdown_high, generate_countdown_low,
    generate_curb_rumble_sound, generate_engine_sound,
    generate_f1_v6_rpm_band, generate_gear_shift_pop, generate_generic_engine_rpm_band,
    generate_jump_launch_sound, generate_kart_125cc_rpm_band, generate_landing_sound,
    generate_lap_chime, generate_nascar_v8_rpm_band, generate_offroad_sound, generate_race_finish,
    generate_rally_turbo_rpm_band, generate_sector_ping, generate_skid_sound,
    generate_sport_gt_rpm_band, generate_ui_move, generate_ui_select, generate_wall_crash_sound,
    generate_water_splash_sound,
};
use crate::audio::synthwave::{generate_menu_theme, generate_nightcall_race_theme};
use crate::audio::dsp::DEFAULT_SAMPLE_RATE;
use crate::audio::auxiliary_fx::AuxiliaryAudioLayer;
use crate::audio::backend::{ActiveSoundHandle, AudioBackend, SoundData};
use crate::audio::engine_mixer::EngineAudioMixer;
use crate::audio::samples::ArchetypeSampleBank;

/// Vehicle engine audio synthesis archetype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineSoundType {
    /// Generic / Standard Sport engine sound (fallback default, 4-stroke 6-cyl balanced sport)
    Generic,
    /// High-displacement V8 / Touring GT (deep crossplane rumble, low-end torque growl, heavy block)
    SportGT,
    /// 125cc 2-Stroke Single-Cylinder Kart (screaming 2-stroke ring-a-ding buzz, expansion chamber resonance)
    Kart125cc,
    /// High-revving Formula 1 V6 Turbo Hybrid (screaming top end, aggressive turbo whine, sharp metallic pitch)
    F1V6Turbo,
    /// 4-Cylinder Rally Turbo (anti-lag pops, wastegate flutter, gravel-chewing mid-range rasp)
    RallyTurbo,
    /// Roaring 5.9L (358 cu in) Pushrod V8 Stock Car / Trans-Am TA1 (open boom-tube side pipes, thunderous roar)
    NascarV8,
}

impl Default for EngineSoundType {
    fn default() -> Self {
        Self::Generic
    }
}

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

/// Audio Volume & Mute Settings (re-exported from cabinet::audio).
pub use cabinet::audio::AudioSettings;

/// Music Track Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MusicTrack {
    NightcallRace,
    NeonMenu,
}

/// Sound Effect Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    JumpLaunch,
    Landing,
    WaterSplash,
}

/// Loaded Sound Handles Cache.
pub struct SoundBank {
    pub music_nightcall: Option<Sound>,
    pub music_menu: Option<Sound>,
    pub engine_rpm_bands: [Option<Sound>; NUM_RPM_BANDS],
    pub engine_generic: [Option<Sound>; NUM_RPM_BANDS],
    pub engine_sport_gt: [Option<Sound>; NUM_RPM_BANDS],
    pub engine_kart: [Option<Sound>; NUM_RPM_BANDS],
    pub engine_f1: [Option<Sound>; NUM_RPM_BANDS],
    pub engine_rally: [Option<Sound>; NUM_RPM_BANDS],
    pub engine_nascar: [Option<Sound>; NUM_RPM_BANDS],
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
    pub sfx_jump_launch: Option<Sound>,
    pub sfx_landing: Option<Sound>,
    pub sfx_water_splash: Option<Sound>,
    pub kira_sfx: HashMap<SfxType, SoundData>,
    pub kira_music: HashMap<MusicTrack, SoundData>,
}

impl SoundBank {
    pub fn empty() -> Self {
        Self {
            music_nightcall: None,
            music_menu: None,
            engine_rpm_bands: [const { None }; NUM_RPM_BANDS],
            engine_generic: [const { None }; NUM_RPM_BANDS],
            engine_sport_gt: [const { None }; NUM_RPM_BANDS],
            engine_kart: [const { None }; NUM_RPM_BANDS],
            engine_f1: [const { None }; NUM_RPM_BANDS],
            engine_rally: [const { None }; NUM_RPM_BANDS],
            engine_nascar: [const { None }; NUM_RPM_BANDS],
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
            sfx_jump_launch: None,
            sfx_landing: None,
            sfx_water_splash: None,
            kira_sfx: HashMap::new(),
            kira_music: HashMap::new(),
        }
    }

    /// Asynchronously loads and registers all procedural WAV sound assets into memory.
    pub async fn load_all() -> Self {
        let sample_rate = DEFAULT_SAMPLE_RATE;

        let nightcall_wav = generate_nightcall_race_theme(sample_rate);
        let menu_wav = generate_menu_theme(sample_rate);

        // Pre-generate 28 harmonic RPM frequency bands for each engine sound archetype
        let mut generic_bands = [const { None }; NUM_RPM_BANDS];
        let mut sport_gt_bands = [const { None }; NUM_RPM_BANDS];
        let mut kart_bands = [const { None }; NUM_RPM_BANDS];
        let mut f1_bands = [const { None }; NUM_RPM_BANDS];
        let mut rally_bands = [const { None }; NUM_RPM_BANDS];
        let mut nascar_bands = [const { None }; NUM_RPM_BANDS];

        for (idx, &freq) in RPM_BAND_FREQS.iter().enumerate() {
            let gen_wav = generate_generic_engine_rpm_band(sample_rate, freq);
            generic_bands[idx] = safe_load_sound_from_bytes(&gen_wav).await;

            let gt_wav = generate_sport_gt_rpm_band(sample_rate, freq);
            sport_gt_bands[idx] = safe_load_sound_from_bytes(&gt_wav).await;

            let kart_wav = generate_kart_125cc_rpm_band(sample_rate, freq);
            kart_bands[idx] = safe_load_sound_from_bytes(&kart_wav).await;

            let f1_wav = generate_f1_v6_rpm_band(sample_rate, freq);
            f1_bands[idx] = safe_load_sound_from_bytes(&f1_wav).await;

            let rally_wav = generate_rally_turbo_rpm_band(sample_rate, freq);
            rally_bands[idx] = safe_load_sound_from_bytes(&rally_wav).await;

            let nascar_wav = generate_nascar_v8_rpm_band(sample_rate, freq);
            nascar_bands[idx] = safe_load_sound_from_bytes(&nascar_wav).await;
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
        let jump_wav = generate_jump_launch_sound(sample_rate);
        let land_wav = generate_landing_sound(sample_rate);
        let water_wav = generate_water_splash_sound(sample_rate);

        let mut kira_sfx = HashMap::new();
        let mut kira_music = HashMap::new();

        if let Ok(sd) = SoundData::from_bytes(&nightcall_wav, true) {
            kira_music.insert(MusicTrack::NightcallRace, sd);
        }
        if let Ok(sd) = SoundData::from_bytes(&menu_wav, true) {
            kira_music.insert(MusicTrack::NeonMenu, sd);
        }

        let sfx_sources = [
            (SfxType::ShiftPop, &shift_pop_wav),
            (SfxType::Engine, &engine_wav),
            (SfxType::Skid, &skid_wav),
            (SfxType::WallCrash, &wall_crash_wav),
            (SfxType::CarHit, &car_hit_wav),
            (SfxType::Curb, &curb_wav),
            (SfxType::Offroad, &offroad_wav),
            (SfxType::CountdownLow, &cd_low_wav),
            (SfxType::CountdownHigh, &cd_high_wav),
            (SfxType::LapChime, &lap_wav),
            (SfxType::SectorPing, &sector_wav),
            (SfxType::UiSelect, &ui_sel_wav),
            (SfxType::UiMove, &ui_mov_wav),
            (SfxType::RaceFinish, &finish_wav),
            (SfxType::JumpLaunch, &jump_wav),
            (SfxType::Landing, &land_wav),
            (SfxType::WaterSplash, &water_wav),
        ];

        for (sfx_type, bytes) in sfx_sources {
            if let Ok(sd) = SoundData::from_bytes(bytes, false) {
                kira_sfx.insert(sfx_type, sd);
            }
        }

        Self {
            music_nightcall: safe_load_sound_from_bytes(&nightcall_wav).await,
            music_menu: safe_load_sound_from_bytes(&menu_wav).await,
            engine_rpm_bands: generic_bands.clone(),
            engine_generic: generic_bands,
            engine_sport_gt: sport_gt_bands,
            engine_kart: kart_bands,
            engine_f1: f1_bands,
            engine_rally: rally_bands,
            engine_nascar: nascar_bands,
            sfx_shift_pop: safe_load_sound_from_bytes(&shift_pop_wav).await,
            sfx_engine: safe_load_sound_from_bytes(&engine_wav).await,
            sfx_skid: safe_load_sound_from_bytes(&skid_wav).await,
            sfx_wall_crash: safe_load_sound_from_bytes(&wall_crash_wav).await,
            sfx_car_hit: safe_load_sound_from_bytes(&car_hit_wav).await,
            sfx_curb: safe_load_sound_from_bytes(&curb_wav).await,
            sfx_offroad: safe_load_sound_from_bytes(&offroad_wav).await,
            sfx_cd_low: safe_load_sound_from_bytes(&cd_low_wav).await,
            sfx_cd_high: safe_load_sound_from_bytes(&cd_high_wav).await,
            sfx_lap: safe_load_sound_from_bytes(&lap_wav).await,
            sfx_sector: safe_load_sound_from_bytes(&sector_wav).await,
            sfx_ui_select: safe_load_sound_from_bytes(&ui_sel_wav).await,
            sfx_ui_move: safe_load_sound_from_bytes(&ui_mov_wav).await,
            sfx_finish: safe_load_sound_from_bytes(&finish_wav).await,
            sfx_jump_launch: safe_load_sound_from_bytes(&jump_wav).await,
            sfx_landing: safe_load_sound_from_bytes(&land_wav).await,
            sfx_water_splash: safe_load_sound_from_bytes(&water_wav).await,
            kira_sfx,
            kira_music,
        }
    }

    /// Retrieves an engine RPM band sound handle for the given engine type,
    /// falling back automatically to the Generic default sound bank if unavailable.
    pub fn get_engine_band(&self, engine_type: EngineSoundType, idx: usize) -> Option<&Sound> {
        if idx >= NUM_RPM_BANDS {
            return None;
        }

        let specific = match engine_type {
            EngineSoundType::Generic => self.engine_generic[idx].as_ref(),
            EngineSoundType::SportGT => self.engine_sport_gt[idx].as_ref(),
            EngineSoundType::Kart125cc => self.engine_kart[idx].as_ref(),
            EngineSoundType::F1V6Turbo => self.engine_f1[idx].as_ref(),
            EngineSoundType::RallyTurbo => self.engine_rally[idx].as_ref(),
            EngineSoundType::NascarV8 => self.engine_nascar[idx].as_ref(),
        };

        specific
            .or_else(|| self.engine_generic[idx].as_ref())
            .or_else(|| self.engine_rpm_bands[idx].as_ref())
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
            SfxType::JumpLaunch => self.sfx_jump_launch.as_ref(),
            SfxType::Landing => self.sfx_landing.as_ref(),
            SfxType::WaterSplash => self.sfx_water_splash.as_ref(),
        }
    }

    pub fn get_kira_sfx(&self, sfx: SfxType) -> Option<&SoundData> {
        self.kira_sfx.get(&sfx)
    }

    pub fn get_kira_music(&self, track: MusicTrack) -> Option<&SoundData> {
        self.kira_music.get(&track)
    }
}

/// Master Audio System Coordinator.
pub struct AudioManager {
    pub settings: AudioSettings,
    pub bank: SoundBank,
    pub current_music: Option<MusicTrack>,
    pub active_engine_type: EngineSoundType,
    pub is_engine_active: bool,
    pub is_skid_active: bool,
    pub engine_active_bands: [bool; NUM_RPM_BANDS],
    pub shift_gap_timer: f32,
    limiter_timer: f32,
    pub backend: AudioBackend,
    pub sampled_engine: Option<EngineAudioMixer>,
    pub auxiliary_layer: Option<AuxiliaryAudioLayer>,
    pub use_sampled_engine: bool,
    pub active_music_handle: Option<ActiveSoundHandle>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Safe wrapper around macroquad play_sound that handles uninitialized headless contexts.
fn safe_play_sound(sound: &Sound, params: PlaySoundParams) {
    #[cfg(target_arch = "wasm32")]
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        play_sound(sound, params);
    }));
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (sound, params);
}

/// Safe wrapper around macroquad set_sound_volume.
fn safe_set_sound_volume(sound: &Sound, volume: f32) {
    #[cfg(target_arch = "wasm32")]
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        set_sound_volume(sound, volume);
    }));
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (sound, volume);
}

/// Safe wrapper around macroquad stop_sound.
fn safe_stop_sound(sound: &Sound) {
    #[cfg(target_arch = "wasm32")]
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        stop_sound(sound);
    }));
    #[cfg(not(target_arch = "wasm32"))]
    let _ = sound;
}

impl AudioManager {
    pub fn new() -> Self {
        let mut backend = AudioBackend::new();
        let (sampled_engine, auxiliary_layer) = if backend.is_available() {
            let bank = ArchetypeSampleBank::generate(EngineSoundType::Generic, DEFAULT_SAMPLE_RATE);
            (
                Some(EngineAudioMixer::new(bank)),
                Some(AuxiliaryAudioLayer::new(&mut backend)),
            )
        } else {
            (None, None)
        };

        Self {
            settings: AudioSettings::default(),
            bank: SoundBank::empty(),
            current_music: None,
            active_engine_type: EngineSoundType::Generic,
            is_engine_active: false,
            is_skid_active: false,
            engine_active_bands: [false; NUM_RPM_BANDS],
            shift_gap_timer: 0.0,
            limiter_timer: 0.0,
            backend,
            sampled_engine,
            auxiliary_layer,
            use_sampled_engine: true,
            active_music_handle: None,
        }
    }

    /// Sets the active vehicle engine sound archetype, stopping prior engine loops if switching.
    pub fn set_engine_type(&mut self, engine_type: EngineSoundType) {
        if self.active_engine_type != engine_type {
            self.stop_all_loops();
            self.active_engine_type = engine_type;
            if let Some(sampled) = self.sampled_engine.as_mut() {
                let new_bank = ArchetypeSampleBank::generate(engine_type, DEFAULT_SAMPLE_RATE);
                sampled.set_bank(new_bank, &mut self.backend);
            }
        }
    }

    /// Initializes and loads all audio banks asynchronously.
    pub async fn init_async(&mut self) {
        self.bank = SoundBank::load_all().await;
        if let Some(track) = self.current_music {
            let vol = self.settings.effective_music_volume();
            if self.backend.is_available() {
                if let Some(sd) = self.bank.get_kira_music(track) {
                    self.active_music_handle = Some(self.backend.play(sd, vol, 1.0));
                }
            } else {
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

    /// Adjusts UI sounds volume level.
    pub fn set_ui_volume(&mut self, vol: f32) {
        self.settings.ui_volume = vol.clamp(0.0, 1.0);
    }

    /// Plays a background music track with looping.
    pub fn play_music(&mut self, track: MusicTrack) {
        if self.current_music == Some(track) {
            return;
        }

        self.stop_music();
        self.current_music = Some(track);

        let vol = self.settings.effective_music_volume();
        if self.backend.is_available() {
            if let Some(sd) = self.bank.get_kira_music(track) {
                self.active_music_handle = Some(self.backend.play(sd, vol, 1.0));
                return;
            }
        }

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
    pub fn sync_music_volume(&mut self) {
        let vol = self.settings.effective_music_volume();
        if let Some(h) = self.active_music_handle.as_mut() {
            h.set_volume(vol, Duration::from_millis(50));
        }
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
        if let Some(mut h) = self.active_music_handle.take() {
            h.stop(Duration::from_millis(150));
        }
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
        let vol = match sfx {
            SfxType::UiMove | SfxType::UiSelect => self.settings.effective_ui_volume() * gain,
            _ => self.settings.effective_sfx_volume_scaled(gain),
        };
        if vol <= 0.001 {
            return;
        }

        if self.backend.is_available() {
            if let Some(sound_data) = self.bank.get_kira_sfx(sfx) {
                self.backend.play(sound_data, vol.clamp(0.0, 1.0), 1.0);
                return;
            }
        }

        if let Some(sound) = self.bank.get_sound(sfx) {
            safe_play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: vol.clamp(0.0, 1.0),
                },
            );
        }
    }

    /// Dynamically crossfades multi-harmonic engine RPM sound bands, reflects throttle load, clutch shift gap, and triggers shift pops.
    pub fn update_engine_rpm(&mut self, rpm: f32, throttle: f32, is_shift: bool) {
        self.update_engine_telemetry(rpm, throttle, is_shift, 0.0, 1, 0.016);
    }

    /// Full telemetry update feeding RPM, throttle, gear, speed, and delta into sampled engine and auxiliary layers.
    pub fn update_engine_telemetry(
        &mut self,
        rpm: f32,
        throttle: f32,
        is_shift: bool,
        speed: f32,
        gear: usize,
        dt: f32,
    ) {
        if self.settings.is_muted {
            self.stop_all_loops();
            return;
        }

        if is_shift {
            self.play_sfx_with_gain(SfxType::ShiftPop, 0.95);
            self.shift_gap_timer = 0.075; // 75ms clutch disengagement gap
        } else if self.shift_gap_timer > 0.0 {
            self.shift_gap_timer = (self.shift_gap_timer - dt).max(0.0);
        }

        let effective_throttle = if self.shift_gap_timer > 0.0 {
            0.0
        } else {
            throttle
        };

        // If modern sampled engine audio is active and hardware backend is available
        if self.use_sampled_engine && self.backend.is_available() && self.sampled_engine.is_some() {
            let limiter_mod = if let Some(aux) = self.auxiliary_layer.as_mut() {
                aux.update(
                    speed.abs(),
                    gear,
                    rpm,
                    effective_throttle,
                    self.active_engine_type,
                    dt,
                    self.settings.effective_sfx_volume(),
                    &mut self.backend,
                )
            } else {
                1.0
            };

            let effective_vol = self.settings.effective_sfx_volume() * limiter_mod;
            if let Some(sampled) = self.sampled_engine.as_mut() {
                sampled.update(rpm, effective_throttle, effective_vol, &mut self.backend);
            }
            self.is_engine_active = true;
            return;
        }

        // Fallback: procedural 28-band crossfade (e.g. headless/WASM)
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
                    let angle = u * std::f32::consts::FRAC_PI_2;
                    weights[i] = angle.cos();
                    weights[i + 1] = angle.sin();
                    break;
                }
            }
        }

        // Engine volume reflecting throttle demand: wide-open intake roar vs engine braking overrun
        let load_factor = effective_throttle.max(0.0);
        let base_gain = 0.42 + 0.58 * load_factor;

        // Rev limiter bounce stutter at redline with high throttle (16 Hz ignition cut oscillation)
        let limiter_mod = if clamped_rpm >= max_rpm - 350.0 && effective_throttle > 0.4 {
            self.limiter_timer = (self.limiter_timer + 0.16).fract();
            if self.limiter_timer < 0.5 { 1.0 } else { 0.45 }
        } else {
            1.0
        };

        // Overrun deceleration burble: subtle modulation when off-throttle at high RPM
        let overrun_mod = if effective_throttle < 0.05 && clamped_rpm > min_rpm + 1500.0 {
            let wobble = (clamped_rpm * 0.05).sin() * 0.08;
            1.0 + wobble
        } else {
            1.0
        };

        let total_vol = self.settings.effective_sfx_volume_scaled(base_gain * limiter_mod * overrun_mod);

        for idx in 0..NUM_RPM_BANDS {
            let band_vol = total_vol * weights[idx];
            let sound_opt = self.bank.get_engine_band(self.active_engine_type, idx);

            if let Some(sound) = sound_opt {
                if band_vol > 0.001 {
                    if !self.engine_active_bands[idx] {
                        safe_play_sound(
                            sound,
                            PlaySoundParams {
                                looped: true,
                                volume: band_vol.clamp(0.0, 1.0),
                            },
                        );
                        self.engine_active_bands[idx] = true;
                    } else {
                        safe_set_sound_volume(sound, band_vol.clamp(0.0, 1.0));
                    }
                } else if self.engine_active_bands[idx] {
                    safe_stop_sound(sound);
                    self.engine_active_bands[idx] = false;
                }
            }
        }
        self.is_engine_active = true;
    }

    /// Triggers tire drift sounds when breaking traction (disabled to keep pure engine audio).
    pub fn update_skid_chirp(&mut self, _slip_intensity: f32, _dt: f32) {
        // Disabled: Keep pure internal combustion engine audio without synthetic chirp overlays
    }

    /// Stops all continuous loops across all engine bands.
    pub fn stop_all_loops(&mut self) {
        if let Some(sampled) = self.sampled_engine.as_mut() {
            sampled.stop();
        }
        if let Some(aux) = self.auxiliary_layer.as_mut() {
            aux.stop();
        }

        for engine_type in [
            EngineSoundType::Generic,
            EngineSoundType::SportGT,
            EngineSoundType::Kart125cc,
            EngineSoundType::F1V6Turbo,
            EngineSoundType::RallyTurbo,
            EngineSoundType::NascarV8,
        ] {
            for idx in 0..NUM_RPM_BANDS {
                if let Some(sound) = self.bank.get_engine_band(engine_type, idx) {
                    safe_set_sound_volume(sound, 0.0);
                    safe_stop_sound(sound);
                }
            }
        }
        self.engine_active_bands = [false; NUM_RPM_BANDS];
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
            ui_volume: 0.95,
            is_muted: false,
        };

        assert!((settings.effective_music_volume() - 0.4).abs() < 0.01);
        assert!((settings.effective_sfx_volume_scaled(1.0) - 0.72).abs() < 0.01);
        assert!((settings.effective_ui_volume() - 0.76).abs() < 0.01);

        settings.toggle_mute();
        assert_eq!(settings.effective_music_volume(), 0.0);
        assert_eq!(settings.effective_sfx_volume_scaled(1.0), 0.0);
        assert_eq!(settings.effective_ui_volume(), 0.0);
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
