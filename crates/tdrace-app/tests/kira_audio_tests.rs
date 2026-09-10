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
    let backend = AudioBackend::new();
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

fn block_on<F: std::future::Future>(mut fut: F) -> F::Output {
    use std::pin::Pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn dummy_raw_waker() -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            dummy_raw_waker()
        }
        let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
        RawWaker::new(std::ptr::null(), vtable)
    }

    let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    let mut pinned = unsafe { Pin::new_unchecked(&mut fut) };

    match pinned.as_mut().poll(&mut cx) {
        Poll::Ready(val) => val,
        Poll::Pending => panic!("Future did not complete synchronously"),
    }
}

#[test]
fn test_soundbank_kira_sfx_and_music_population() {
    use tdrace_app::audio::{MusicTrack, SfxType, SoundBank};

    let bank = block_on(SoundBank::load_all());

    // Verify all 17 SFX types have decoded Kira sound data
    let all_sfx = [
        SfxType::Engine,
        SfxType::ShiftPop,
        SfxType::Skid,
        SfxType::WallCrash,
        SfxType::CarHit,
        SfxType::Curb,
        SfxType::Offroad,
        SfxType::CountdownLow,
        SfxType::CountdownHigh,
        SfxType::LapChime,
        SfxType::SectorPing,
        SfxType::UiSelect,
        SfxType::UiMove,
        SfxType::RaceFinish,
        SfxType::JumpLaunch,
        SfxType::Landing,
        SfxType::WaterSplash,
    ];

    for sfx in all_sfx {
        assert!(
            bank.get_kira_sfx(sfx).is_some(),
            "SFX {:?} should be populated in SoundBank::kira_sfx",
            sfx
        );
    }

    // Verify music tracks have decoded Kira sound data
    assert!(bank.get_kira_music(MusicTrack::NightcallRace).is_some());
    assert!(bank.get_kira_music(MusicTrack::NeonMenu).is_some());
}

#[test]
fn test_audio_manager_kira_sfx_and_music_lifecycle() {
    use tdrace_app::audio::{AudioManager, MusicTrack, SfxType};

    let mut audio = AudioManager::new();
    block_on(audio.init_async());

    // SFX playback
    audio.play_sfx(SfxType::UiSelect);
    audio.play_sfx(SfxType::CountdownHigh);
    audio.play_sfx_with_gain(SfxType::LapChime, 0.8);

    // Music playback
    audio.play_music(MusicTrack::NightcallRace);
    assert_eq!(audio.current_music, Some(MusicTrack::NightcallRace));

    // Volume updates & sync
    audio.set_music_volume(0.6);
    audio.set_master_volume(0.85);
    audio.toggle_mute();
    assert!(audio.settings.is_muted);
    audio.toggle_mute();
    assert!(!audio.settings.is_muted);

    // Switch music track
    audio.play_music(MusicTrack::NeonMenu);
    assert_eq!(audio.current_music, Some(MusicTrack::NeonMenu));

    // Stop music
    audio.stop_music();
    assert_eq!(audio.current_music, None);
    assert!(audio.active_music_handle.is_none());
}
