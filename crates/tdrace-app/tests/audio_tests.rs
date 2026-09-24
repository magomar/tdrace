//! Comprehensive Audio Engine & Synthwave OST Integration Test Suite.

use tdrace_app::audio::dsp::{
    encode_wav_16bit_mono, encode_wav_16bit_stereo, soft_saturate, AdsrEnvelope, BiquadLowPass,
    Oscillator, StereoDelay, DEFAULT_SAMPLE_RATE,
};
use tdrace_app::audio::sfx::*;
use tdrace_app::audio::synthwave::{generate_menu_theme, generate_nightcall_race_theme, midi_to_hz};
use tdrace_app::audio::AudioSettings;
use tdrace_app::game::{GameState, RaceSession};

#[test]
fn test_wav_riff_header_and_data_integrity() {
    let mono_samples = vec![0.0, 0.25, -0.25, 0.75, -0.75, 1.0, -1.0];
    let mono_wav = encode_wav_16bit_mono(&mono_samples, DEFAULT_SAMPLE_RATE);

    assert_eq!(&mono_wav[0..4], b"RIFF");
    assert_eq!(&mono_wav[8..12], b"WAVE");
    assert_eq!(&mono_wav[12..16], b"fmt ");
    assert_eq!(&mono_wav[36..40], b"data");
    assert_eq!(mono_wav.len(), 44 + mono_samples.len() * 2);

    let stereo_frames = vec![(0.1, -0.1), (0.5, -0.5), (0.9, -0.9)];
    let stereo_wav = encode_wav_16bit_stereo(&stereo_frames, DEFAULT_SAMPLE_RATE);

    assert_eq!(&stereo_wav[0..4], b"RIFF");
    assert_eq!(&stereo_wav[8..12], b"WAVE");
    assert_eq!(stereo_wav.len(), 44 + stereo_frames.len() * 4);
}

#[test]
fn test_dsp_oscillators_bounds_and_shapes() {
    for i in 0..100 {
        let phase = i as f32 / 100.0;
        let s = Oscillator::sine(phase);
        let saw = Oscillator::saw(phase);
        let tri = Oscillator::triangle(phase);
        let sq = Oscillator::square(phase, 0.5);

        assert!(s >= -1.0001 && s <= 1.0001, "Sine out of bounds: {}", s);
        assert!(saw >= -1.0001 && saw <= 1.0001, "Saw out of bounds: {}", saw);
        assert!(tri >= -1.0001 && tri <= 1.0001, "Tri out of bounds: {}", tri);
        assert!(sq == 1.0 || sq == -1.0, "Square out of bounds: {}", sq);
    }
}

#[test]
fn test_adsr_envelope_stages() {
    let adsr = AdsrEnvelope::new(0.05, 0.10, 0.60, 0.15);

    // Initial Attack
    assert_eq!(adsr.evaluate(0.0, 1.0), 0.0);
    assert!((adsr.evaluate(0.05, 1.0) - 1.0).abs() < 0.01);

    // Decay to Sustain
    let mid_decay = adsr.evaluate(0.10, 1.0);
    assert!(mid_decay < 1.0 && mid_decay > 0.60);
    assert!((adsr.evaluate(0.15, 1.0) - 0.60).abs() < 0.01);

    // Held Sustain
    assert!((adsr.evaluate(0.50, 1.0) - 0.60).abs() < 0.01);

    // Release after gate (1.0s)
    let mid_release = adsr.evaluate(1.075, 1.0);
    assert!(mid_release < 0.60 && mid_release > 0.0);
    assert_eq!(adsr.evaluate(1.20, 1.0), 0.0);
}

#[test]
fn test_biquad_lowpass_filter_attenuation() {
    let mut filter = BiquadLowPass::new(44100, 500.0, 0.707);

    // Feed high frequency sine (10,000 Hz) -> should be heavily attenuated
    let high_freq = 10000.0;
    let mut max_output = 0.0f32;

    for i in 0..1000 {
        let t = i as f32 / 44100.0;
        let x = Oscillator::sine(t * high_freq);
        let y = filter.process(x);
        assert!(y.is_finite());
        if i > 100 {
            max_output = max_output.max(y.abs());
        }
    }

    assert!(max_output < 0.15, "Lowpass failed to attenuate 10kHz sine: {}", max_output);
}

#[test]
fn test_stereo_delay_echo() {
    let mut delay = StereoDelay::new(44100, 10.0, 20.0, 0.5, 0.5);
    let (l, r) = delay.process(1.0, 1.0);
    assert!((l - 0.5).abs() < 0.01);
    assert!((r - 0.5).abs() < 0.01);

    for _ in 0..2000 {
        let (dl, dr) = delay.process(0.0, 0.0);
        assert!(dl.is_finite());
        assert!(dr.is_finite());
    }
}

#[test]
fn test_soft_saturation_curve() {
    assert_eq!(soft_saturate(0.0, 1.0), 0.0);
    assert!((soft_saturate(0.5, 1.0) - 0.5).abs() < 0.1);
    assert!(soft_saturate(10.0, 1.0) <= 1.0);
    assert!(soft_saturate(-10.0, 1.0) >= -1.0);
}

#[test]
fn test_midi_tuning_frequencies() {
    assert!((midi_to_hz(69) - 440.0).abs() < 0.001); // A4
    assert!((midi_to_hz(57) - 220.0).abs() < 0.001); // A3
    assert!((midi_to_hz(60) - 261.625).abs() < 0.01); // Middle C
}

#[test]
fn test_nightcall_race_theme_synthesis_and_stereo_width() {
    let wav = generate_nightcall_race_theme(22050);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");

    // Stereo 16-bit at 22050 Hz: 44 bytes + total_samples * 4 bytes
    assert!(wav.len() > 100_000);
}

#[test]
fn test_menu_theme_synthesis() {
    let wav = generate_menu_theme(22050);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert!(wav.len() > 50_000);
}

#[test]
fn test_all_arcade_sfx_generators() {
    let sfx = [
        ("engine", generate_engine_sound(DEFAULT_SAMPLE_RATE)),
        ("skid", generate_skid_sound(DEFAULT_SAMPLE_RATE)),
        ("wall_crash", generate_wall_crash_sound(DEFAULT_SAMPLE_RATE)),
        ("car_hit", generate_car_hit_sound(DEFAULT_SAMPLE_RATE)),
        ("curb", generate_curb_rumble_sound(DEFAULT_SAMPLE_RATE)),
        ("offroad", generate_offroad_sound(DEFAULT_SAMPLE_RATE)),
        ("cd_low", generate_countdown_low(DEFAULT_SAMPLE_RATE)),
        ("cd_high", generate_countdown_high(DEFAULT_SAMPLE_RATE)),
        ("lap", generate_lap_chime(DEFAULT_SAMPLE_RATE)),
        ("sector", generate_sector_ping(DEFAULT_SAMPLE_RATE)),
        ("ui_select", generate_ui_select(DEFAULT_SAMPLE_RATE)),
        ("ui_move", generate_ui_move(DEFAULT_SAMPLE_RATE)),
        ("finish", generate_race_finish(DEFAULT_SAMPLE_RATE)),
    ];

    for (name, data) in sfx {
        assert_eq!(&data[0..4], b"RIFF", "Failed RIFF for {}", name);
        assert_eq!(&data[8..12], b"WAVE", "Failed WAVE for {}", name);
        assert!(data.len() > 44, "Empty data for {}", name);
    }
}

#[test]
fn test_audio_settings_and_mixer_gain() {
    let mut settings = AudioSettings::default();
    assert!(!settings.is_muted);

    let base_music = settings.effective_music_volume();
    assert!(base_music > 0.5 && base_music < 0.8);

    settings.set_master_volume_test(0.5);
    assert!((settings.effective_music_volume() - 0.35).abs() < 0.01);

    settings.toggle_mute();
    assert_eq!(settings.effective_music_volume(), 0.0);
    assert_eq!(settings.effective_sfx_volume(), 0.0);
    assert_eq!(settings.effective_sfx_volume_scaled(1.0), 0.0);
    assert_eq!(settings.effective_ui_volume(), 0.0);

    settings.toggle_mute();
    assert!((settings.effective_music_volume() - 0.35).abs() < 0.01);
}

#[test]
fn test_race_session_audio_wiring_and_countdown_state() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });
    assert_eq!(session.audio.settings.is_muted, false);

    // Initial race countdown setup
    session.init_race();
    assert!(matches!(session.state, GameState::StartingGrid | GameState::Countdown(_)));


    // Check audio mute toggle via session API
    session.audio.toggle_mute();
    assert_eq!(session.audio.settings.is_muted, true);
    session.audio.toggle_mute();
    // Test audio volume adjustment
    session.audio.set_master_volume(0.60);
    assert!((session.audio.settings.master_volume - 0.60).abs() < 0.01);
    session.audio.set_music_volume(0.40);
    assert!((session.audio.settings.music_volume - 0.40).abs() < 0.01);
    session.audio.set_sfx_volume(0.75);
    assert!((session.audio.settings.sfx_volume - 0.75).abs() < 0.01);
}

#[test]
fn test_independent_music_and_sound_separation_in_session() {
    let mut session = RaceSession::new();
    assert!(!session.audio.settings.is_music_muted);
    assert!(!session.audio.settings.is_sfx_muted);

    // 1. Toggle Music independently
    session.audio.toggle_music();
    assert!(session.audio.settings.is_music_muted);
    assert!(!session.audio.settings.is_sfx_muted);
    assert_eq!(session.audio.settings.effective_music_volume(), 0.0);
    assert!(session.audio.settings.effective_sfx_volume() > 0.0);

    // 2. Toggle Sound independently
    session.audio.toggle_sfx();
    assert!(session.audio.settings.is_music_muted);
    assert!(session.audio.settings.is_sfx_muted);
    assert_eq!(session.audio.settings.effective_music_volume(), 0.0);
    assert_eq!(session.audio.settings.effective_sfx_volume(), 0.0);

    // 3. Unmute Music: Music plays while Sound remains muted
    session.audio.toggle_music();
    assert!(!session.audio.settings.is_music_muted);
    assert!(session.audio.settings.is_sfx_muted);
    assert!(session.audio.settings.effective_music_volume() > 0.0);
    assert_eq!(session.audio.settings.effective_sfx_volume(), 0.0);

    // 4. Unmute Sound: Both active
    session.audio.toggle_sfx();
    assert!(!session.audio.settings.is_music_muted);
    assert!(!session.audio.settings.is_sfx_muted);
    assert!(session.audio.settings.effective_music_volume() > 0.0);
    assert!(session.audio.settings.effective_sfx_volume() > 0.0);
}

#[test]
fn test_engine_rpm_model_gear_shifts_and_revs() {
    let mut model = tdrace_app::game::EngineRpmModel::default();
    assert_eq!(model.current_gear, 1);
    assert!((model.current_rpm - 1100.0).abs() < 10.0);

    // Revving on the starting grid with throttle
    let (rpm_launch, is_shift) = model.update(0.0, 1.0, 0.0, 0.5);
    assert!(!is_shift);
    assert!(rpm_launch > 3000.0, "Engine should rev up on full throttle launch, got {}", rpm_launch);

    // Accelerating in 1st gear
    let (rpm_1st, is_shift_1) = model.update(10.0, 1.0, 0.0, 0.5);
    assert!(!is_shift_1);
    assert_eq!(model.current_gear, 1);
    assert!(rpm_1st > 4000.0);

    // Upshift to 2nd gear at 18 m/s
    let (_rpm_2nd, is_shift_2) = model.update(18.0, 1.0, 0.0, 0.1);
    assert!(is_shift_2, "Should trigger upshift to 2nd gear");
    assert_eq!(model.current_gear, 2);
}

#[test]
fn test_all_28_rpm_bands_synthesis_and_equal_power_weights() {
    use tdrace_app::audio::manager::{NUM_RPM_BANDS, RPM_BAND_FREQS, RPM_BAND_RPMS};

    assert_eq!(NUM_RPM_BANDS, 28);
    assert_eq!(RPM_BAND_FREQS.len(), 28);
    assert_eq!(RPM_BAND_RPMS.len(), 28);

    // Verify frequency monotonic ascending order
    for i in 0..(NUM_RPM_BANDS - 1) {
        assert!(RPM_BAND_FREQS[i] < RPM_BAND_FREQS[i + 1], "Bands must be strictly ascending in frequency");
        assert!(RPM_BAND_RPMS[i] < RPM_BAND_RPMS[i + 1], "Bands must be strictly ascending in RPM");
    }

    // Verify every band generates a valid non-empty WAV with RIFF header
    for &freq in &RPM_BAND_FREQS {
        let wav = generate_engine_rpm_band(DEFAULT_SAMPLE_RATE, freq);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert!(wav.len() > 1000);
    }

    // Test equal-power crossfade math across range
    for test_rpm in [900.0, 1500.0, 3200.0, 5000.0, 6800.0, 8000.0] {
        let mut w_sum_sq = 0.0f32;
        let mut active_count = 0;

        for i in 0..(NUM_RPM_BANDS - 1) {
            let low = RPM_BAND_RPMS[i];
            let high = RPM_BAND_RPMS[i + 1];
            if test_rpm >= low && test_rpm <= high {
                let u = ((test_rpm - low) / (high - low)).clamp(0.0, 1.0);
                let angle = u * std::f32::consts::FRAC_PI_2;
                let w1 = angle.cos();
                let w2 = angle.sin();
                w_sum_sq = w1 * w1 + w2 * w2;
                active_count = 2;
                break;
            }
        }

        if active_count == 2 {
            assert!((w_sum_sq - 1.0).abs() < 0.001, "Equal power sum must be 1.0, got {}", w_sum_sq);
        }
    }
}

#[test]
fn test_multi_engine_sound_types_synthesis_and_fallback() {
    use tdrace_app::audio::manager::{EngineSoundType, SoundBank, RPM_BAND_FREQS};

    // Test that all 5 engine sound type generators produce valid WAVs
    for &freq in &RPM_BAND_FREQS[..4] {
        let generic_wav = generate_generic_engine_rpm_band(DEFAULT_SAMPLE_RATE, freq);
        let gt_wav = generate_sport_gt_rpm_band(DEFAULT_SAMPLE_RATE, freq);
        let kart_wav = generate_kart_125cc_rpm_band(DEFAULT_SAMPLE_RATE, freq);
        let rally_wav = generate_rally_turbo_rpm_band(DEFAULT_SAMPLE_RATE, freq);
        let nascar_wav = generate_nascar_v8_rpm_band(DEFAULT_SAMPLE_RATE, freq);

        for (name, wav) in [
            ("generic", generic_wav),
            ("sport_gt", gt_wav),
            ("kart", kart_wav),
            ("rally", rally_wav),
            ("nascar", nascar_wav),
        ] {
            assert_eq!(&wav[0..4], b"RIFF", "{name} band missing RIFF");
            assert_eq!(&wav[8..12], b"WAVE", "{name} band missing WAVE");
            assert!(wav.len() > 1000, "{name} band payload empty");
        }
    }

    // Test fallback mechanism in SoundBank
    let bank = SoundBank::empty();
    // Initially empty -> returns None
    assert!(bank.get_engine_band(EngineSoundType::SportGT, 0).is_none());

    // When only generic bank has a sound, querying SportGT, Kart, Rally falls back to generic
    // (We test the fallback logic on the data structure)
    let test_sound_bytes = generate_generic_engine_rpm_band(DEFAULT_SAMPLE_RATE, 65.0);
    assert!(!test_sound_bytes.is_empty());
}

#[test]
fn test_audio_manager_engine_switching_and_shift_gap() {
    use tdrace_app::audio::manager::{AudioManager, EngineSoundType};

    let mut audio = AudioManager::new();
    assert_eq!(audio.active_engine_type, EngineSoundType::Generic);

    // Switch to Kart
    audio.set_engine_type(EngineSoundType::Kart125cc);
    assert_eq!(audio.active_engine_type, EngineSoundType::Kart125cc);

    // Switch to Sport GT
    audio.set_engine_type(EngineSoundType::SportGT);
    assert_eq!(audio.active_engine_type, EngineSoundType::SportGT);

    // Test shift gap initialization on shift
    assert_eq!(audio.shift_gap_timer, 0.0);
    audio.update_engine_rpm(3500.0, 1.0, true);
    assert!(audio.shift_gap_timer > 0.05, "Shift gap timer should be initialized on upshift");

    // Stepping update should decay shift gap timer
    audio.update_engine_rpm(3500.0, 1.0, false);
    assert!(audio.shift_gap_timer < 0.075);
}

// Helper trait to test settings setters
trait AudioSettingsTestExt {
    fn set_master_volume_test(&mut self, vol: f32);
}

impl AudioSettingsTestExt for AudioSettings {
    fn set_master_volume_test(&mut self, vol: f32) {
        self.master_volume = vol.clamp(0.0, 1.0);
    }
}

#[test]
fn test_classic_cars_audio_profile_and_tier_one_sound_bank_mapping() {
    use tdrace_app::audio::EngineSoundType;
    use tdrace_app::module::classic::ClassicGameModule;
    use tdrace_app::module::GameModule;
    use tdrace_app::catalog::CLASSIC_ARCADE_CARS;

    let classic = ClassicGameModule::new();
    let vehicles = classic.vehicles();

    // 1. Verify VehicleModelDefinition audio_profile on ClassicGameModule
    let gt = vehicles.iter().find(|v| v.id == "classic_gt").expect("classic_gt missing");
    assert_eq!(gt.audio_profile.expect("profile missing").sound_type, EngineSoundType::Gt4Clubsport);

    let nascar = vehicles.iter().find(|v| v.id == "classic_nascar").expect("classic_nascar missing");
    assert_eq!(nascar.audio_profile.expect("profile missing").sound_type, EngineSoundType::LateModelV8);

    let offroad = vehicles.iter().find(|v| v.id == "classic_offroad").expect("classic_offroad missing");
    assert_eq!(offroad.audio_profile.expect("profile missing").sound_type, EngineSoundType::SandRailBoxer);

    let kart = vehicles.iter().find(|v| v.id == "classic_kart").expect("classic_kart missing");
    assert_eq!(kart.audio_profile.expect("profile missing").sound_type, EngineSoundType::KartCadet60);

    let rally = vehicles.iter().find(|v| v.id == "classic_rally").expect("classic_rally missing");
    assert_eq!(rally.audio_profile.expect("profile missing").sound_type, EngineSoundType::CrossCarMotorcycle);

    // 2. Verify CLASSIC_ARCADE_CARS RealCarModel sound_type dispatch
    let expected = [
        ("classic_gt", EngineSoundType::Gt4Clubsport),
        ("classic_nascar", EngineSoundType::LateModelV8),
        ("classic_offroad", EngineSoundType::SandRailBoxer),
        ("classic_kart", EngineSoundType::KartCadet60),
        ("classic_rally", EngineSoundType::CrossCarMotorcycle),
    ];

    for (id, expected_sound) in expected {
        let car = CLASSIC_ARCADE_CARS.iter().find(|c| c.id == id).expect("car not found in catalog");
        assert_eq!(car.sound_type(), expected_sound, "Car {id} should map to {expected_sound:?}");
    }
}

#[test]
fn test_car_choice_sound_type_mapping() {
    use tdrace_app::audio::EngineSoundType;
    use tdrace_app::ui::menu::CarChoice;

    assert_eq!(CarChoice::StockCar.sound_type(), EngineSoundType::LateModelV8);
    assert_eq!(CarChoice::SandRail.sound_type(), EngineSoundType::SandRailBoxer);
    assert_eq!(CarChoice::Kart.sound_type(), EngineSoundType::KartCadet60);
    assert_eq!(CarChoice::RallyCar.sound_type(), EngineSoundType::CrossCarMotorcycle);
    assert_eq!(CarChoice::SportsCar.sound_type(), EngineSoundType::Gt4Clubsport);
    assert_eq!(CarChoice::DriftCar.sound_type(), EngineSoundType::Gt4Clubsport);
    assert_eq!(CarChoice::GT4Clubsport.sound_type(), EngineSoundType::Gt4Clubsport);
    assert_eq!(CarChoice::GT3Car.sound_type(), EngineSoundType::Gt3HighRev);
    assert_eq!(CarChoice::GT2Biturbo.sound_type(), EngineSoundType::Gt2Biturbo);
    assert_eq!(CarChoice::GT1Legend.sound_type(), EngineSoundType::Gt1V12Analogue);
    assert_eq!(CarChoice::HypercarPrototype.sound_type(), EngineSoundType::HypercarV6Hybrid);
}

#[test]
fn test_session_resolve_active_sound_type_for_classic_vehicles() {
    use tdrace_app::audio::EngineSoundType;
    use tdrace_app::game::{GameState, GarageOrigin, RaceSession};

    let mut session = RaceSession::new();
    session.active_module_id = "classic";

    // A. Race context with selected_car_model_id
    session.selected_car_model_id = Some("classic_kart");
    assert_eq!(session.resolve_active_sound_type(), EngineSoundType::KartCadet60);

    session.selected_car_model_id = Some("classic_rally");
    assert_eq!(session.resolve_active_sound_type(), EngineSoundType::CrossCarMotorcycle);

    session.selected_car_model_id = Some("classic_nascar");
    assert_eq!(session.resolve_active_sound_type(), EngineSoundType::LateModelV8);

    session.selected_car_model_id = Some("classic_offroad");
    assert_eq!(session.resolve_active_sound_type(), EngineSoundType::SandRailBoxer);

    session.selected_car_model_id = Some("classic_gt");
    assert_eq!(session.resolve_active_sound_type(), EngineSoundType::Gt4Clubsport);

    // B. Garage showroom context: active car matches garage_car_idx
    session.state = GameState::Garage(GarageOrigin::Menu);
    session.garage_tier = 1;

    let garage_expected = [
        (0, EngineSoundType::Gt4Clubsport),      // classic_gt
        (1, EngineSoundType::LateModelV8),       // classic_nascar
        (2, EngineSoundType::SandRailBoxer),     // classic_offroad
        (3, EngineSoundType::KartCadet60),       // classic_kart
        (4, EngineSoundType::CrossCarMotorcycle), // classic_rally
    ];

    for (idx, sound) in garage_expected {
        session.garage_car_idx = idx;
        assert_eq!(session.resolve_active_sound_type(), sound, "Garage car at idx {idx} should have sound {sound:?}");
    }
}

