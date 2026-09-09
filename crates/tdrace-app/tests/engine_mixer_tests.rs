use tdrace_app::audio::backend::AudioBackend;
use tdrace_app::audio::engine_mixer::EngineAudioMixer;
use tdrace_app::audio::manager::EngineSoundType;
use tdrace_app::audio::samples::{ArchetypeSampleBank, HIGH_RPM, IDLE_RPM, MID_RPM};

#[test]
fn test_voice_parameter_pitch_matching_at_breakpoints() {
    let master_vol = 0.8;

    // 1. At exact Idle RPM (950 RPM) with 0 throttle
    let p_idle = EngineAudioMixer::compute_voice_parameters(IDLE_RPM, 0.0, master_vol);
    assert!((p_idle[0].0 - 1.0).abs() < 1e-3, "Idle sample should have 1.0x playback rate at Idle RPM");
    assert!(p_idle[0].1 > 0.3, "Idle voice should have high volume at Idle RPM");
    assert!(p_idle[3].1 < 1e-4 && p_idle[4].1 < 1e-4, "High RPM voices should be silent at Idle");

    // 2. At exact Mid RPM (3600 RPM) with 100% throttle
    let p_mid = EngineAudioMixer::compute_voice_parameters(MID_RPM, 1.0, master_vol);
    assert!((p_mid[1].0 - 1.0).abs() < 1e-3, "Mid-on sample should have 1.0x playback rate at Mid RPM");
    assert!(p_mid[1].1 > 0.4, "Mid-on voice should have high volume at full throttle");
    assert!(p_mid[2].1 < 1e-4, "Mid-off voice should be silent at full throttle");

    // 3. At exact High RPM (7200 RPM) with 100% throttle
    let p_high = EngineAudioMixer::compute_voice_parameters(HIGH_RPM, 1.0, master_vol);
    assert!((p_high[3].0 - 1.0).abs() < 1e-3, "High-on sample should have 1.0x playback rate at High RPM");
    assert!(p_high[3].1 > 0.4, "High-on voice should have high volume at redline");
}

#[test]
fn test_pitch_matching_in_transition_regions() {
    let master_vol = 0.8;

    // At 5400 RPM (midway between 3600 and 7200)
    let p_trans = EngineAudioMixer::compute_voice_parameters(5400.0, 1.0, master_vol);

    let rate_mid = p_trans[1].0;
    let rate_high = p_trans[3].0;

    // Both must match 5400 RPM:
    // rate_mid = 5400 / 3600 = 1.5
    // rate_high = 5400 / 7200 = 0.75
    assert!((rate_mid - 1.5).abs() < 1e-3, "Mid voice must pitch shift to 1.5x at 5400 RPM");
    assert!((rate_high - 0.75).abs() < 1e-3, "High voice must pitch shift to 0.75x at 5400 RPM");

    // Both bounding voices must be active simultaneously (smooth equal-power crossfade)
    assert!(p_trans[1].1 > 0.2, "Mid voice should be audible during transition");
    assert!(p_trans[3].1 > 0.2, "High voice should be audible during transition");
}

#[test]
fn test_throttle_load_crossfade_balance() {
    let master_vol = 0.8;

    // Full throttle at Mid RPM
    let p_full = EngineAudioMixer::compute_voice_parameters(MID_RPM, 1.0, master_vol);
    assert!(p_full[1].1 > p_full[2].1, "Full throttle must favour on-load over off-load voice");
    assert!(p_full[2].1 < 1e-4, "Off-load voice should be virtually silent at 100% throttle");

    // Zero throttle (engine braking overrun) at Mid RPM
    let p_off = EngineAudioMixer::compute_voice_parameters(MID_RPM, 0.0, master_vol);
    assert!(p_off[2].1 > p_off[1].1, "Zero throttle must favour off-load over on-load voice");
    assert!(p_off[1].1 < 1e-4, "On-load voice should be virtually silent at 0% throttle");

    // Partial throttle (50%)
    let p_half = EngineAudioMixer::compute_voice_parameters(MID_RPM, 0.5, master_vol);
    assert!(p_half[1].1 > 0.1 && p_half[2].1 > 0.1, "50% throttle should blend both load voices");
}

#[test]
fn test_engine_audio_mixer_lifecycle_and_switching() {
    let mut backend = AudioBackend::new();
    let bank_gt = ArchetypeSampleBank::generate(EngineSoundType::SportGT, 44100);
    let mut mixer = EngineAudioMixer::new(bank_gt);

    // Initial state
    assert!(!mixer.voices.is_active);

    // Update through RPM rev range
    mixer.update(1200.0, 0.2, 0.85, &mut backend);
    assert!(mixer.voices.is_active);

    mixer.update(3500.0, 0.8, 0.85, &mut backend);
    mixer.update(6800.0, 1.0, 0.85, &mut backend);
    mixer.update(7500.0, 0.0, 0.85, &mut backend);

    // Switch vehicle archetype
    let bank_nascar = ArchetypeSampleBank::generate(EngineSoundType::NascarV8, 44100);
    mixer.set_bank(bank_nascar, &mut backend);
    assert_eq!(mixer.bank.engine_type, EngineSoundType::NascarV8);

    // Stop playback
    mixer.stop();
    assert!(!mixer.voices.is_active);
}
