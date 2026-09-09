use tdrace_app::audio::auxiliary_fx::{
    generate_blow_off_valve_sound, generate_exhaust_crackle_sound, generate_transmission_whine_loop,
    generate_turbo_spool_loop, AuxiliaryAudioLayer,
};
use tdrace_app::audio::backend::AudioBackend;
use tdrace_app::audio::manager::EngineSoundType;

#[test]
fn test_auxiliary_sound_generators_produce_valid_wav() {
    let sample_rate = 44100;

    let whine = generate_transmission_whine_loop(sample_rate);
    assert!(whine.starts_with(b"RIFF"));

    let spool = generate_turbo_spool_loop(sample_rate);
    assert!(spool.starts_with(b"RIFF"));

    let bov = generate_blow_off_valve_sound(sample_rate);
    assert!(bov.starts_with(b"RIFF"));

    let crackle = generate_exhaust_crackle_sound(sample_rate);
    assert!(crackle.starts_with(b"RIFF"));
}

#[test]
fn test_turbo_spool_dynamics_and_blow_off_valve() {
    let mut backend = AudioBackend::new();
    let mut aux = AuxiliaryAudioLayer::new(&mut backend);

    // 1. Initial spool should be zero
    assert_eq!(aux.turbo_spool_level, 0.0);

    // 2. Accelerating with full throttle at high RPM should spool up turbo
    for _ in 0..60 {
        aux.update(30.0, 3, 6000.0, 1.0, EngineSoundType::RallyTurbo, 0.016, 0.8, &mut backend);
    }
    assert!(aux.turbo_spool_level > 0.5, "Turbo should spool up under continuous boost");

    // 3. Sudden throttle drop from full throttle to zero should trigger BOV and dump spool pressure
    let pre_drop_spool = aux.turbo_spool_level;
    aux.update(30.0, 3, 6000.0, 0.0, EngineSoundType::RallyTurbo, 0.016, 0.8, &mut backend);

    assert!(
        aux.turbo_spool_level < pre_drop_spool * 0.5,
        "Blow-off valve should dump manifold pressure on sudden throttle release"
    );
}

#[test]
fn test_rev_limiter_stutter_at_redline() {
    let mut backend = AudioBackend::new();
    let mut aux = AuxiliaryAudioLayer::new(&mut backend);

    // Below redline: modifier is always 1.0
    let mod_normal = aux.update(40.0, 4, 6000.0, 1.0, EngineSoundType::SportGT, 0.016, 0.8, &mut backend);
    assert_eq!(mod_normal, 1.0);

    // At redline (>9000 RPM) with high throttle: limiter oscillation triggers
    let mut saw_cut = false;
    let mut saw_full = false;

    for _ in 0..30 {
        let m = aux.update(50.0, 4, 9150.0, 1.0, EngineSoundType::SportGT, 0.005, 0.8, &mut backend);
        if (m - 0.35).abs() < 1e-3 {
            saw_cut = true;
        } else if (m - 1.0).abs() < 1e-3 {
            saw_full = true;
        }
    }

    assert!(saw_cut, "Rev limiter must trigger ignition cut phase (0.35 volume)");
    assert!(saw_full, "Rev limiter must cycle between full and cut phases (15 Hz oscillation)");
}
