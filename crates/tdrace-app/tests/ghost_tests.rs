use glam::Vec2;
use std::f32::consts::PI;
use tdrace_app::render::ghost::{lerp_angle, GhostFrame, GhostLap, GhostRecorder};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};
use tdrace_core::physics::car::Car;
use tdrace_core::CarConfig;

#[test]
fn test_lerp_angle_shortest_path_and_wraparound() {
    // 1. Standard interpolation
    let mid = lerp_angle(0.2, 0.8, 0.5);
    assert!((mid - 0.5).abs() < 1e-4);

    // 2. Wrap-around crossing PI / -PI boundary:
    // Angle A = 3.10 rad (~177.6 deg), Angle B = -3.10 rad (~-177.6 deg)
    // Shortest path goes through PI (total angular distance = 0.083 rad)
    let mid_wrap = lerp_angle(3.10, -3.10, 0.5);
    // At t = 0.5, it should be at PI or -PI (both represent 180 degrees)
    assert!(
        (mid_wrap - PI).abs() < 1e-3 || (mid_wrap - (-PI)).abs() < 1e-3,
        "Expected PI or -PI, got {}",
        mid_wrap
    );

    // 3. Negative to positive wrap-around:
    let mid_neg_pos = lerp_angle(-3.10, 3.10, 0.5);
    assert!(
        (mid_neg_pos - PI).abs() < 1e-3 || (mid_neg_pos - (-PI)).abs() < 1e-3,
        "Expected PI or -PI, got {}",
        mid_neg_pos
    );

    // 4. Boundary cases (t = 0.0, t = 1.0)
    assert!((lerp_angle(0.5, 2.0, 0.0) - 0.5).abs() < 1e-4);
    assert!((lerp_angle(0.5, 2.0, 1.0) - 2.0).abs() < 1e-4);
}

#[test]
fn test_ghost_telemetry_interpolation() {
    let frames = vec![
        GhostFrame {
            time: 0.0,
            position: Vec2::new(0.0, 0.0),
            angle: 0.0,
            steer_angle: 0.0,
            speed: 0.0,
        },
        GhostFrame {
            time: 1.0,
            position: Vec2::new(10.0, 20.0),
            angle: 0.5,
            steer_angle: 0.2,
            speed: 25.0,
        },
        GhostFrame {
            time: 2.0,
            position: Vec2::new(30.0, 50.0),
            angle: 1.2,
            steer_angle: -0.1,
            speed: 40.0,
        },
    ];

    let ghost_lap = GhostLap::new(TrackChoice::ClassicGrandPrix, CarChoice::SportsCar, 2.0, frames);

    // Sample before start (clamp to frame 0)
    let s_before = ghost_lap.sample_at_time(-1.0).unwrap();
    assert_eq!(s_before.position, Vec2::new(0.0, 0.0));
    assert_eq!(s_before.speed, 0.0);

    // Sample exact frame 1
    let s_f1 = ghost_lap.sample_at_time(1.0).unwrap();
    assert_eq!(s_f1.position, Vec2::new(10.0, 20.0));
    assert_eq!(s_f1.speed, 25.0);

    // Sample midpoint between frame 0 and frame 1 (t = 0.5s)
    let s_mid = ghost_lap.sample_at_time(0.5).unwrap();
    assert_eq!(s_mid.position, Vec2::new(5.0, 10.0));
    assert!((s_mid.angle - 0.25).abs() < 1e-4);
    assert!((s_mid.steer_angle - 0.10).abs() < 1e-4);
    assert!((s_mid.speed - 12.5).abs() < 1e-4);

    // Sample midpoint between frame 1 and frame 2 (t = 1.5s)
    let s_mid2 = ghost_lap.sample_at_time(1.5).unwrap();
    assert_eq!(s_mid2.position, Vec2::new(20.0, 35.0));
    assert!((s_mid2.angle - 0.85).abs() < 1e-4);
    assert!((s_mid2.steer_angle - 0.05).abs() < 1e-4);
    assert!((s_mid2.speed - 32.5).abs() < 1e-4);

    // Sample past end (clamp to last frame)
    let s_after = ghost_lap.sample_at_time(5.0).unwrap();
    assert_eq!(s_after.position, Vec2::new(30.0, 50.0));
    assert_eq!(s_after.speed, 40.0);
}

#[test]
fn test_ghost_recorder_lap_lifecycle_and_best_lap_updates() {
    let mut recorder = GhostRecorder::new();
    assert!(recorder.best_ghost_lap.is_none());

    let mut car = Car::new(CarConfig::sports_car());
    car.state.position = Vec2::new(10.0, 10.0);

    let dt = 1.0 / 60.0;

    // --- Lap 1: 45.0 seconds (First lap sets initial best) ---
    for step in 0..120 {
        let t = step as f32 * 0.375;
        car.state.position += Vec2::new(1.0, 0.5);
        recorder.record_frame(t, &car, dt);
    }

    let is_new_best_1 = recorder.on_lap_completed(45.0, TrackChoice::DriftPark, CarChoice::DriftCar);
    assert!(is_new_best_1);
    assert!(recorder.best_ghost_lap.is_some());
    assert_eq!(recorder.best_ghost_lap.as_ref().unwrap().lap_time, 45.0);

    // --- Lap 2: 48.5 seconds (Slower lap -> must be discarded) ---
    for step in 0..120 {
        let t = step as f32 * 0.40;
        car.state.position += Vec2::new(0.5, 0.2);
        recorder.record_frame(t, &car, dt);
    }

    let is_new_best_2 = recorder.on_lap_completed(48.5, TrackChoice::DriftPark, CarChoice::DriftCar);
    assert!(!is_new_best_2);
    // Best lap remains 45.0s
    assert_eq!(recorder.best_ghost_lap.as_ref().unwrap().lap_time, 45.0);

    // --- Lap 3: 41.2 seconds (Faster lap -> updates personal best) ---
    for step in 0..120 {
        let t = step as f32 * 0.34;
        car.state.position += Vec2::new(1.5, 0.8);
        recorder.record_frame(t, &car, dt);
    }

    let is_new_best_3 = recorder.on_lap_completed(41.2, TrackChoice::DriftPark, CarChoice::DriftCar);
    assert!(is_new_best_3);
    assert_eq!(recorder.best_ghost_lap.as_ref().unwrap().lap_time, 41.2);

    // --- Lap 4: Invalidated lap (e.g. reset/wrong way) ---
    recorder.record_frame(5.0, &car, dt);
    assert!(!recorder.current_lap_frames.is_empty());
    recorder.on_lap_invalidated();
    assert!(recorder.current_lap_frames.is_empty());
    // Best ghost lap still preserved
    assert_eq!(recorder.best_ghost_lap.as_ref().unwrap().lap_time, 41.2);
}
