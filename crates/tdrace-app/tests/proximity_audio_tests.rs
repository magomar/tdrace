//! Integration tests for Proximity Engine Audio and Doppler Shift calculations (Spec 061).

use glam::Vec2;
use tdrace_app::audio::proximity::{
    calculate_distance_attenuation, calculate_doppler_factor, calculate_spatial_audio,
    calculate_stereo_pan, select_top_k_audible_sources, DopplerConfig, VehicleAudioSource,
    DEFAULT_MAX_DISTANCE, DEFAULT_PAN_RADIUS, MAX_PROXIMITY_VOICES,
};
use tdrace_app::audio::EngineSoundType;

#[test]
fn test_distance_attenuation_saturation_and_cutoff() {
    // Wheel-to-wheel saturation (<= 2.5m)
    assert_eq!(calculate_distance_attenuation(0.0, 2.5, 75.0), 1.0);
    assert_eq!(calculate_distance_attenuation(1.5, 2.5, 75.0), 1.0);
    assert_eq!(calculate_distance_attenuation(2.5, 2.5, 75.0), 1.0);

    // Beyond audible horizon (>= 75.0m)
    assert_eq!(calculate_distance_attenuation(75.0, 2.5, 75.0), 0.0);
    assert_eq!(calculate_distance_attenuation(80.0, 2.5, 75.0), 0.0);
    assert_eq!(calculate_distance_attenuation(200.0, 2.5, 75.0), 0.0);

    // Monotonic quadratic roll-off in between
    let g10 = calculate_distance_attenuation(10.0, 2.5, 75.0);
    let g25 = calculate_distance_attenuation(25.0, 2.5, 75.0);
    let g50 = calculate_distance_attenuation(50.0, 2.5, 75.0);

    assert!(g10 > g25);
    assert!(g25 > g50);
    assert!(g50 > 0.0);
    assert!(g10 < 1.0);

    // Exact quadratic formula check at midpoint (38.75m)
    let mid_dist = (2.5 + 75.0) / 2.0;
    let expected_mid = 0.5f32.powi(2); // 0.25
    let actual_mid = calculate_distance_attenuation(mid_dist, 2.5, 75.0);
    assert!((actual_mid - expected_mid).abs() < 1e-4);
}

#[test]
fn test_stereo_panning_ranges_and_clamps() {
    let r_pan = DEFAULT_PAN_RADIUS; // 35.0m

    // Center
    assert_eq!(calculate_stereo_pan(0.0, r_pan), 0.0);

    // Left side
    let pan_left_half = calculate_stereo_pan(-17.5, r_pan);
    assert!((pan_left_half - (-0.5)).abs() < 1e-4);

    let pan_left_full = calculate_stereo_pan(-35.0, r_pan);
    assert_eq!(pan_left_full, -1.0);

    let pan_left_clamped = calculate_stereo_pan(-60.0, r_pan);
    assert_eq!(pan_left_clamped, -1.0);

    // Right side
    let pan_right_half = calculate_stereo_pan(17.5, r_pan);
    assert!((pan_right_half - 0.5).abs() < 1e-4);

    let pan_right_full = calculate_stereo_pan(35.0, r_pan);
    assert_eq!(pan_right_full, 1.0);

    let pan_right_clamped = calculate_stereo_pan(60.0, r_pan);
    assert_eq!(pan_right_clamped, 1.0);
}

#[test]
fn test_select_top_k_audible_sources_budgeting() {
    let listener = Vec2::new(0.0, 0.0);
    let mut sources = Vec::new();

    // Generate 6 vehicles at varying distances
    let distances = [15.0, 85.0, 4.0, 60.0, 76.0, 30.0];
    for (i, &d) in distances.iter().enumerate() {
        sources.push(VehicleAudioSource {
            vehicle_id: i + 1,
            engine_type: EngineSoundType::Generic,
            position: Vec2::new(0.0, d),
            velocity: Vec2::ZERO,
            forward_speed: 0.0,
            throttle: 0.0,
            slip_ratio: 0.0,
        });
    }

    let top_k = select_top_k_audible_sources(&sources, listener, DEFAULT_MAX_DISTANCE, MAX_PROXIMITY_VOICES);

    // Must return at most 3 items
    assert_eq!(top_k.len(), 3);

    // Distances within 75m: 4.0 (idx 2), 15.0 (idx 0), 30.0 (idx 5), 60.0 (idx 3)
    // 85.0 (idx 1) and 76.0 (idx 4) are outside 75m
    // Top 3 closest must be: idx 2 (4m), idx 0 (15m), idx 5 (30m)
    assert_eq!(top_k, vec![2, 0, 5]);
}

#[test]
fn test_doppler_disabled_returns_unity() {
    let config = DopplerConfig {
        enabled: false,
        ..Default::default()
    };

    let p_s = Vec2::new(0.0, 50.0);
    let v_s = Vec2::new(0.0, -50.0); // Moving towards listener
    let p_l = Vec2::ZERO;
    let v_l = Vec2::ZERO;

    let factor = calculate_doppler_factor(p_s, v_s, p_l, v_l, &config);
    assert_eq!(factor, 1.0);
}

#[test]
fn test_doppler_head_on_approach_pitch_rise() {
    let config = DopplerConfig::default(); // c = 160.0 m/s

    let p_s = Vec2::new(0.0, 50.0);
    let v_s = Vec2::new(0.0, -40.0); // Source moving south towards listener at 40 m/s
    let p_l = Vec2::ZERO;
    let v_l = Vec2::ZERO; // Listener stationary

    let factor = calculate_doppler_factor(p_s, v_s, p_l, v_l, &config);

    // Expected: c / (c - v_approach) = 160 / (160 - 40) = 160 / 120 = 1.3333
    let expected = 160.0 / 120.0;
    assert!((factor - expected).abs() < 1e-3, "Expected ~1.333, got {}", factor);
}

#[test]
fn test_doppler_receding_pitch_drop() {
    let config = DopplerConfig::default(); // c = 160.0 m/s

    let p_s = Vec2::new(0.0, 50.0);
    let v_s = Vec2::new(0.0, 30.0); // Source moving north away from listener at 30 m/s
    let p_l = Vec2::ZERO;
    let v_l = Vec2::ZERO;

    let factor = calculate_doppler_factor(p_s, v_s, p_l, v_l, &config);

    // Expected: c / (c - (-30)) = 160 / (160 + 30) = 160 / 190 = 0.8421
    let expected = 160.0 / 190.0;
    assert!((factor - expected).abs() < 1e-3, "Expected ~0.842, got {}", factor);
}

#[test]
fn test_doppler_perpendicular_closest_approach_unity() {
    let config = DopplerConfig::default();

    // Source passing horizontally at closest approach (x = 0, y = 10)
    let p_s = Vec2::new(0.0, 10.0);
    let v_s = Vec2::new(50.0, 0.0); // Pure horizontal velocity
    let p_l = Vec2::ZERO;
    let v_l = Vec2::ZERO;

    let factor = calculate_doppler_factor(p_s, v_s, p_l, v_l, &config);
    // Radial component along line of sight (0, 1) is 0.0, so Doppler is exactly 1.0
    assert!((factor - 1.0).abs() < 1e-4, "Expected 1.0 at closest approach, got {}", factor);
}

#[test]
fn test_doppler_extreme_collision_clamp_safety() {
    let config = DopplerConfig::default(); // max = 1.45, min = 0.65

    // Relative speed 150 m/s towards each other
    let p_s = Vec2::new(0.0, 10.0);
    let v_s = Vec2::new(0.0, -100.0);
    let p_l = Vec2::ZERO;
    let v_l = Vec2::new(0.0, 50.0);

    let factor = calculate_doppler_factor(p_s, v_s, p_l, v_l, &config);
    assert_eq!(factor, config.max_multiplier);
    assert!(!factor.is_nan());
    assert!(!factor.is_infinite());

    // Relative speed 200 m/s away from each other
    let v_s_receding = Vec2::new(0.0, 150.0);
    let v_l_receding = Vec2::new(0.0, -50.0);
    let factor_receding = calculate_doppler_factor(p_s, v_s_receding, p_l, v_l_receding, &config);
    assert_eq!(factor_receding, config.min_multiplier);
}

#[test]
fn test_spatial_audio_evaluator_composite() {
    let source = VehicleAudioSource {
        vehicle_id: 42,
        engine_type: EngineSoundType::Gt3HighRev,
        position: Vec2::new(17.5, 20.0),
        velocity: Vec2::new(0.0, -20.0),
        forward_speed: 20.0,
        throttle: 0.8,
        slip_ratio: 0.05,
    };

    let listener_pos = Vec2::ZERO;
    let listener_vel = Vec2::ZERO;
    let doppler_cfg = DopplerConfig::default();

    let res = calculate_spatial_audio(&source, listener_pos, listener_vel, &doppler_cfg);

    let expected_dist = (17.5f32.powi(2) + 20.0f32.powi(2)).sqrt(); // ~26.57m
    assert!((res.distance - expected_dist).abs() < 1e-3);
    assert!(res.gain > 0.0 && res.gain < 1.0);
    assert!((res.pan - (17.5 / DEFAULT_PAN_RADIUS)).abs() < 1e-3);
    assert!(res.doppler_factor > 1.0); // Approaching
}

#[test]
fn test_audio_manager_proximity_voice_allocation_and_hysteresis() {
    use tdrace_app::audio::AudioManager;

    let mut audio = AudioManager::new();
    if !audio.backend.is_available() {
        return; // Skip voice loop checks in headless environments lacking hardware audio
    }

    let listener_pos = Vec2::ZERO;
    let listener_vel = Vec2::ZERO;

    // 4 sources with ascending distances
    let mut sources = vec![
        VehicleAudioSource {
            vehicle_id: 1,
            engine_type: EngineSoundType::Generic,
            position: Vec2::new(0.0, 10.0),
            velocity: Vec2::ZERO,
            forward_speed: 10.0,
            throttle: 0.8,
            slip_ratio: 0.0,
        },
        VehicleAudioSource {
            vehicle_id: 2,
            engine_type: EngineSoundType::SportGT,
            position: Vec2::new(0.0, 20.0),
            velocity: Vec2::ZERO,
            forward_speed: 15.0,
            throttle: 0.7,
            slip_ratio: 0.0,
        },
        VehicleAudioSource {
            vehicle_id: 3,
            engine_type: EngineSoundType::Kart125cc,
            position: Vec2::new(0.0, 30.0),
            velocity: Vec2::ZERO,
            forward_speed: 20.0,
            throttle: 0.9,
            slip_ratio: 0.0,
        },
        VehicleAudioSource {
            vehicle_id: 4,
            engine_type: EngineSoundType::LateModelV8,
            position: Vec2::new(0.0, 40.0),
            velocity: Vec2::ZERO,
            forward_speed: 25.0,
            throttle: 0.6,
            slip_ratio: 0.0,
        },
    ];

    // Frame 1: Top 3 should be 1, 2, 3
    audio.update_proximity_engines(&sources, listener_pos, listener_vel, 0.016);
    let mut active_vids: Vec<usize> = audio
        .proximity_voices
        .iter()
        .filter_map(|s| s.vehicle_id)
        .collect();
    active_vids.sort();
    assert_eq!(active_vids, vec![1, 2, 3]);

    // Frame 2: Vehicle 4 moves to 28m (closer than vehicle 3 at 30m, but within 3m hysteresis buffer)
    sources[3].position = Vec2::new(0.0, 28.0);
    audio.update_proximity_engines(&sources, listener_pos, listener_vel, 0.016);
    let mut active_vids_hysteresis: Vec<usize> = audio
        .proximity_voices
        .iter()
        .filter_map(|s| s.vehicle_id)
        .collect();
    active_vids_hysteresis.sort();
    // Vehicle 3 should still be retained because 28.0 + 3.0 > 30.0
    assert_eq!(active_vids_hysteresis, vec![1, 2, 3]);

    // Frame 3: Vehicle 4 moves to 22m (more than 3m closer than vehicle 3 at 30m)
    sources[3].position = Vec2::new(0.0, 22.0);
    audio.update_proximity_engines(&sources, listener_pos, listener_vel, 0.016);
    let mut active_vids_displaced: Vec<usize> = audio
        .proximity_voices
        .iter()
        .filter_map(|s| s.vehicle_id)
        .collect();
    active_vids_displaced.sort();
    // Vehicle 3 displaced by vehicle 4
    assert_eq!(active_vids_displaced, vec![1, 2, 4]);

    // Stop all loops clears all proximity voice allocations
    audio.stop_all_loops();
    assert!(audio.proximity_voices.iter().all(|s| s.vehicle_id.is_none()));
}
