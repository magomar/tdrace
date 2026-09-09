use std::time::Duration;
use tdrace_app::audio::backend::{amplitude_to_db, AudioBackend, SoundData};
use tdrace_app::audio::dsp::{encode_wav_16bit_mono, Oscillator, DEFAULT_SAMPLE_RATE};

#[test]
fn test_amplitude_to_db_scaling() {
    assert_eq!(amplitude_to_db(0.0), -60.0);
    assert_eq!(amplitude_to_db(0.0005), -60.0);
    assert!((amplitude_to_db(1.0) - 0.0).abs() < 1e-4);
    // 0.5 amplitude is approx -6.02 dB
    let half_db = amplitude_to_db(0.5);
    assert!((half_db - (-6.0206)).abs() < 0.05);
    // 0.1 amplitude is -20 dB
    let tenth_db = amplitude_to_db(0.1);
    assert!((tenth_db - (-20.0)).abs() < 0.05);
}

#[test]
fn test_audio_backend_lifecycle_and_mock_safety() {
    let mut backend = AudioBackend::new();
    // In headless environments, is_available may be false, but it must not panic
    let _ = backend.is_available();

    // Create a small 0.1s 440 Hz test tone
    let sample_rate = DEFAULT_SAMPLE_RATE;
    let n_samples = (0.1 * sample_rate as f32) as usize;
    let mut samples = vec![0.0f32; n_samples];
    for (i, s) in samples.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;
        *s = Oscillator::sine(t * 440.0) * 0.5;
    }
    let wav_bytes = encode_wav_16bit_mono(&samples, sample_rate);

    let sound_result = SoundData::from_bytes(&wav_bytes, true);
    assert!(sound_result.is_ok(), "SoundData should decode generated WAV bytes");

    let sound = sound_result.unwrap();
    let mut handle = backend.play(&sound, 0.75, 1.0);

    // Verify dynamic pitch-shifting and volume modulation controls
    handle.set_playback_rate(1.5, Duration::from_millis(10));
    handle.set_playback_rate(0.65, Duration::from_millis(10));
    handle.set_playback_rate(2.2, Duration::from_millis(10));

    handle.set_volume(0.3, Duration::from_millis(10));
    handle.set_volume(0.0, Duration::from_millis(10));

    handle.stop(Duration::from_millis(10));
}
