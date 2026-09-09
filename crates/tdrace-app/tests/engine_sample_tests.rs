use tdrace_app::audio::manager::EngineSoundType;
use tdrace_app::audio::samples::{
    generate_steady_engine_loop, ArchetypeSampleBank, HIGH_RPM, IDLE_RPM, MID_RPM,
};
use tdrace_app::audio::sfx::EngineSoundConfig;

#[test]
fn test_generate_steady_engine_loop_waveform_integrity() {
    let sample_rate = 44100;
    let config = EngineSoundConfig::sport_gt();

    let idle_wav = generate_steady_engine_loop(sample_rate, IDLE_RPM, false, &config);
    assert!(idle_wav.starts_with(b"RIFF"));
    assert!(idle_wav.len() > 44, "WAV must contain audio payload");

    let mid_on_wav = generate_steady_engine_loop(sample_rate, MID_RPM, true, &config);
    let mid_off_wav = generate_steady_engine_loop(sample_rate, MID_RPM, false, &config);

    // On-load should have higher root-mean-square energy than off-load
    let parse_pcm_i16 = |wav: &[u8]| -> Vec<i16> {
        wav[44..]
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect()
    };

    let on_samples = parse_pcm_i16(&mid_on_wav);
    let off_samples = parse_pcm_i16(&mid_off_wav);

    let rms_on: f64 = (on_samples.iter().map(|&s| (s as f64).powi(2)).sum::<f64>()
        / on_samples.len() as f64)
        .sqrt();
    let rms_off: f64 = (off_samples.iter().map(|&s| (s as f64).powi(2)).sum::<f64>()
        / off_samples.len() as f64)
        .sqrt();

    assert!(
        rms_on > rms_off,
        "On-throttle combustion should have higher acoustic energy than off-throttle overrun (on={rms_on}, off={rms_off})"
    );
}

#[test]
fn test_all_five_archetypes_sample_banks_generate_valid_audio() {
    let archetypes = [
        EngineSoundType::SportGT,
        EngineSoundType::NascarV8,
        EngineSoundType::Kart125cc,
        EngineSoundType::F1V6Turbo,
        EngineSoundType::RallyTurbo,
    ];

    for archetype in archetypes {
        let bank = ArchetypeSampleBank::generate(archetype, 44100);

        assert_eq!(bank.idle.rpm, IDLE_RPM);
        assert!(!bank.idle.is_load);
        assert!(!bank.idle.wav_bytes.is_empty());

        assert_eq!(bank.mid_on.rpm, MID_RPM);
        assert!(bank.mid_on.is_load);

        assert_eq!(bank.mid_off.rpm, MID_RPM);
        assert!(!bank.mid_off.is_load);

        assert_eq!(bank.high_on.rpm, HIGH_RPM);
        assert!(bank.high_on.is_load);

        assert_eq!(bank.high_off.rpm, HIGH_RPM);
        assert!(!bank.high_off.is_load);
    }
}

#[test]
fn test_sample_bank_disk_serialization_and_reloading() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_samples_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);

    let bank = ArchetypeSampleBank::generate(EngineSoundType::NascarV8, 44100);
    bank.save_to_dir(&temp_dir).expect("Should write sample bank files to disk");

    let nascar_dir = temp_dir.join("nascar_v8");
    assert!(nascar_dir.join("idle.wav").exists());
    assert!(nascar_dir.join("mid_on.wav").exists());
    assert!(nascar_dir.join("mid_off.wav").exists());
    assert!(nascar_dir.join("high_on.wav").exists());
    assert!(nascar_dir.join("high_off.wav").exists());

    let loaded_bank = ArchetypeSampleBank::load_or_generate(EngineSoundType::NascarV8, &temp_dir);
    assert_eq!(loaded_bank.idle.wav_bytes.len(), bank.idle.wav_bytes.len());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_save_canonical_engine_assets_to_repo() {
    let repo_assets = if std::path::Path::new("assets/audio/engines").exists() {
        std::path::Path::new("assets/audio/engines")
    } else if std::path::Path::new("../../assets/audio/engines").exists() {
        std::path::Path::new("../../assets/audio/engines")
    } else {
        return;
    };

    for archetype in [
        EngineSoundType::SportGT,
        EngineSoundType::NascarV8,
        EngineSoundType::Kart125cc,
        EngineSoundType::F1V6Turbo,
        EngineSoundType::RallyTurbo,
    ] {
        let bank = ArchetypeSampleBank::generate(archetype, 44100);
        let _ = bank.save_to_dir(repo_assets);
    }
}
