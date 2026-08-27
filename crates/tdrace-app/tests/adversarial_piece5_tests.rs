use glam::Vec2;
use std::f32::consts::PI;
use tdrace_app::input::touch::{
    RawTouchPhase, RawTouchPoint, TouchController, TouchLayout,
};
use tdrace_app::render::ghost::{lerp_angle, GhostFrame, GhostLap};
use tdrace_app::replay::{Replay, ReplayPlayer, ReplayRecorder};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::presets::{classic_grand_prix, drift_park, kart_arena, oval_speedway};
use tdrace_core::CarConfig;

// ==============================================================================
// 1. ADVERSARIAL MULTI-TOUCH STRESS TESTS
// ==============================================================================

#[test]
fn test_ten_finger_chaotic_multitouch_stress() {
    let mut tc = TouchController::new();
    tc.enabled = true;
    tc.layout = TouchLayout::VirtualJoystick;
    let sw = 1920.0;
    let sh = 1080.0;

    let (gas_rect, brake_rect, handbrake_rect, layout_rect) = tc.compute_pedal_rects(sw, sh);
    let joy_center = tc.compute_joystick_center(sw, sh);

    // 10 fingers touch simultaneously across different zones
    let mut touch_batch = Vec::new();
    for id in 1..=10 {
        let pos = match id {
            1 => joy_center + Vec2::new(-50.0, 10.0), // Joystick left
            2 => Vec2::new(gas_rect.x + 10.0, gas_rect.y + 10.0), // Gas pedal
            3 => Vec2::new(brake_rect.x + 5.0, brake_rect.y + 5.0), // Brake pedal
            4 => Vec2::new(handbrake_rect.x + 5.0, handbrake_rect.y + 5.0), // Handbrake
            5 => Vec2::new(layout_rect.x + 5.0, layout_rect.y + 5.0), // Layout toggle
            6 => Vec2::new(500.0, 500.0), // Middle of screen (no button)
            7 => Vec2::new(600.0, 300.0), // Middle of screen
            8 => Vec2::new(10.0, 10.0),   // Top left
            9 => Vec2::new(sw - 10.0, 10.0), // Top right
            10 => Vec2::new(sw / 2.0, sh - 10.0), // Bottom middle
            _ => unreachable!(),
        };
        touch_batch.push(RawTouchPoint {
            id,
            position: pos,
            phase: RawTouchPhase::Started,
        });
    }

    tc.process_touch_points(&touch_batch, sw, sh);

    // Ensure no panic, no NaN
    let ctrl = tc.poll_controls();
    assert!(!ctrl.steer.is_nan());
    assert!(!ctrl.throttle.is_nan());
    assert!(!ctrl.brake.is_nan());

    // Release all 10 touches in reverse order with mixed Ended/Cancelled
    let mut release_batch = Vec::new();
    for id in (1..=10).rev() {
        let phase = if id % 2 == 0 {
            RawTouchPhase::Cancelled
        } else {
            RawTouchPhase::Ended
        };
        release_batch.push(RawTouchPoint {
            id,
            position: Vec2::ZERO,
            phase,
        });
    }

    tc.process_touch_points(&release_batch, sw, sh);

    // All controls must cleanly reset to default zero
    let ctrl_after = tc.poll_controls();
    assert_eq!(ctrl_after.steer, 0.0);
    assert_eq!(ctrl_after.throttle, 0.0);
    assert_eq!(ctrl_after.brake, 0.0);
    assert!(!ctrl_after.handbrake);
    assert!(!ctrl_after.reverse);
}

#[test]
fn test_touch_sliding_off_buttons() {
    let mut tc = TouchController::new();
    tc.enabled = true;
    let sw = 1280.0;
    let sh = 720.0;

    let (gas_rect, _, _, _) = tc.compute_pedal_rects(sw, sh);
    let inside_gas = Vec2::new(gas_rect.x + gas_rect.w * 0.5, gas_rect.y + gas_rect.h * 0.5);
    let outside_gas = Vec2::new(gas_rect.x - 50.0, gas_rect.y); // Dragged to the left outside rect

    // Start touch inside Gas
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 42,
            position: inside_gas,
            phase: RawTouchPhase::Started,
        }],
        sw,
        sh,
    );
    assert!(tc.btn_gas.is_pressed);
    assert_eq!(tc.poll_controls().throttle, 1.0);

    // Slide finger outside Gas rect (Moved)
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 42,
            position: outside_gas,
            phase: RawTouchPhase::Moved,
        }],
        sw,
        sh,
    );
    // Button must deactivate while finger is held outside
    assert!(!tc.btn_gas.is_pressed);
    assert_eq!(tc.poll_controls().throttle, 0.0);

    // Slide finger back inside Gas rect (Moved)
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 42,
            position: inside_gas,
            phase: RawTouchPhase::Moved,
        }],
        sw,
        sh,
    );
    assert!(tc.btn_gas.is_pressed);
    assert_eq!(tc.poll_controls().throttle, 1.0);

    // Lift finger (Ended)
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 42,
            position: inside_gas,
            phase: RawTouchPhase::Ended,
        }],
        sw,
        sh,
    );
    assert!(!tc.btn_gas.is_pressed);
    assert_eq!(tc.poll_controls().throttle, 0.0);
}

#[test]
fn test_joystick_extreme_displacement_and_zero_delta() {
    let mut tc = TouchController::new();
    tc.enabled = true;
    let sw = 1280.0;
    let sh = 720.0;
    let center = tc.compute_joystick_center(sw, sh);

    // 1. Extreme 10,000px displacement far offscreen
    tc.process_touch_points(
        &[
            RawTouchPoint {
                id: 1,
                position: center,
                phase: RawTouchPhase::Started,
            },
            RawTouchPoint {
                id: 1,
                position: center + Vec2::new(10_000.0, 5_000.0),
                phase: RawTouchPhase::Moved,
            },
        ],
        sw,
        sh,
    );
    assert!(tc.joystick.deflection_x <= 1.0 && tc.joystick.deflection_x >= -1.0);
    assert!(tc.joystick.deflection_y <= 1.0 && tc.joystick.deflection_y >= -1.0);
    assert!(!tc.joystick.deflection_x.is_nan());
    assert!(!tc.joystick.deflection_y.is_nan());

    // 2. Touch exactly at center (delta = 0)
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 1,
            position: center,
            phase: RawTouchPhase::Moved,
        }],
        sw,
        sh,
    );
    assert_eq!(tc.joystick.deflection_x, 0.0);
    assert_eq!(tc.joystick.deflection_y, 0.0);
}

#[test]
fn test_touch_layout_toggle_during_active_touches() {
    let mut tc = TouchController::new();
    tc.enabled = true;
    let sw = 1280.0;
    let sh = 720.0;
    let center = tc.compute_joystick_center(sw, sh);

    // Active touch on joystick
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 77,
            position: center + Vec2::new(40.0, 0.0),
            phase: RawTouchPhase::Started,
        }],
        sw,
        sh,
    );
    assert_eq!(tc.layout, TouchLayout::VirtualJoystick);
    assert!(tc.joystick.is_active);

    // Toggle layout to SplitButtons -> must cleanly reset joystick
    tc.toggle_layout();
    assert_eq!(tc.layout, TouchLayout::SplitButtons);
    assert!(!tc.joystick.is_active);
    assert_eq!(tc.poll_controls().steer, 0.0);
}

// ==============================================================================
// 2. ADVERSARIAL REPLAY DETERMINISM & SCRUBBING TESTS
// ==============================================================================

#[test]
fn test_5000_step_fuzzed_replay_determinism_all_tracks_and_cars() {
    let tracks = [
        (TrackChoice::ClassicGrandPrix, classic_grand_prix()),
        (TrackChoice::OvalSpeedway, oval_speedway()),
        (TrackChoice::DriftPark, drift_park()),
        (TrackChoice::KartArena, kart_arena()),
    ];

    let cars = [
        (CarChoice::SportsCar, CarConfig::sports_car()),
        (CarChoice::DriftCar, CarConfig::drift_car()),
        (CarChoice::Kart, CarConfig::kart()),
        (CarChoice::RallyCar, CarConfig::rally_car()),
    ];

    let dt = 1.0 / 120.0;

    for (t_choice, track) in &tracks {
        for (c_choice, config) in &cars {
            let init_pose = track.grid_positions[0];
            let mut car = Car::new(*config).with_pose(init_pose.position, init_pose.angle);
            let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
            let mut recorder = ReplayRecorder::new(t_choice.clone(), *c_choice, 42, dt);

            // Pseudo-random deterministic fuzzer (LCG)
            let mut rng_state: u64 = 0xDEADBEEF;
            let mut next_f32 = || {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((rng_state >> 32) as u32 as f32) / (u32::MAX as f32)
            };

            // Run 500 dynamic steps per configuration combination
            for _step in 0..500 {
                let throttle = next_f32();
                let steer = (next_f32() * 2.0) - 1.0;
                let brake = if next_f32() > 0.85 { next_f32() } else { 0.0 };
                let handbrake = next_f32() > 0.90;

                let controls = CarControls {
                    throttle,
                    steer,
                    brake,
                    handbrake,
                    reverse: false,
                };

                recorder.record_frame(controls, &car, &tracker);

                let surfaces = track.sample_car_surfaces(&car);
                car.step_per_wheel(&controls, surfaces, dt);
                tracker.update(&car, &track.spline, &track.checkpoints, dt);
            }

            let replay = recorder.finish(tracker.best_lap_time);
            let bytes = replay.to_bytes().unwrap();
            let loaded = Replay::from_bytes(&bytes).unwrap();
            let player = ReplayPlayer::new(loaded);

            let max_err = player.verify_determinism().unwrap();
            assert!(
                max_err < 1e-4,
                "Trajectory mismatch on {:?} / {:?}: max error = {:.8}",
                t_choice,
                c_choice,
                max_err
            );
        }
    }
}

#[test]
fn test_replay_scrubbing_and_boundary_seeking() {
    let track_choice = TrackChoice::ClassicGrandPrix;
    let car_choice = CarChoice::SportsCar;
    let mut recorder = ReplayRecorder::new(track_choice, car_choice, 1, 1.0 / 120.0);
    let car = Car::new(CarConfig::sports_car());
    let tracker = TrackProgressTracker::new(4, 3);

    for i in 0..600 {
        recorder.record_frame(
            CarControls {
                throttle: (i as f32) / 600.0,
                steer: 0.0,
                brake: 0.0,
                handbrake: false,
                reverse: false,
            },
            &car,
            &tracker,
        );
    }

    let replay = recorder.finish(None);
    let mut player = ReplayPlayer::new(replay);

    // Scrub back and forth
    player.scrub_to_frame(300);
    assert_eq!(player.current_frame, 300);
    assert!(!player.is_finished);

    player.scrub_to_frame(0);
    assert_eq!(player.current_frame, 0);

    player.scrub_to_frame(599);
    assert_eq!(player.current_frame, 599);
    let ctrl = player.step(1.0 / 120.0);
    assert!(ctrl.is_some());
    assert!(player.is_finished);

    // Negative time scrubbing clamps to 0
    player.scrub_to_time(-5.0);
    assert_eq!(player.current_frame, 0);

    // Overflow time scrubbing clamps to total frames
    player.scrub_to_time(9999.0);
    assert_eq!(player.current_frame, 600);
    assert!(player.is_finished);
}

// ==============================================================================
// 3. ADVERSARIAL GHOST CAR INTERPOLATION TESTS
// ==============================================================================

#[test]
fn test_ghost_edge_cases_empty_single_and_identical_timestamps() {
    // 1. Empty ghost lap
    let empty_lap = GhostLap::new(TrackChoice::ClassicGrandPrix, CarChoice::SportsCar, 0.0, vec![]);
    assert!(empty_lap.sample_at_time(0.0).is_none());
    assert!(empty_lap.sample_at_time(10.0).is_none());
    assert!(empty_lap.sample_at_time(-5.0).is_none());

    // 2. Single frame ghost lap
    let single_frame = GhostFrame {
        time: 1.0,
        position: Vec2::new(15.0, 25.0),
        angle: 0.8,
        steer_angle: 0.1,
        speed: 20.0,
    };
    let single_lap = GhostLap::new(TrackChoice::ClassicGrandPrix, CarChoice::SportsCar, 1.0, vec![single_frame]);
    assert_eq!(single_lap.sample_at_time(-10.0).unwrap(), single_frame);
    assert_eq!(single_lap.sample_at_time(1.0).unwrap(), single_frame);
    assert_eq!(single_lap.sample_at_time(100.0).unwrap(), single_frame);

    // 3. Duplicate timestamps (dt = 0.0 between consecutive frames)
    let dup_frames = vec![
        GhostFrame {
            time: 0.0,
            position: Vec2::new(0.0, 0.0),
            angle: 0.0,
            steer_angle: 0.0,
            speed: 0.0,
        },
        GhostFrame {
            time: 1.0,
            position: Vec2::new(10.0, 10.0),
            angle: 0.5,
            steer_angle: 0.2,
            speed: 10.0,
        },
        GhostFrame {
            time: 1.0, // Identical time!
            position: Vec2::new(10.0, 10.0),
            angle: 0.5,
            steer_angle: 0.2,
            speed: 10.0,
        },
        GhostFrame {
            time: 2.0,
            position: Vec2::new(20.0, 20.0),
            angle: 1.0,
            steer_angle: 0.0,
            speed: 20.0,
        },
    ];
    let dup_lap = GhostLap::new(TrackChoice::ClassicGrandPrix, CarChoice::SportsCar, 2.0, dup_frames);
    let sample = dup_lap.sample_at_time(1.0).unwrap();
    assert!(!sample.position.x.is_nan());
    assert!(!sample.position.y.is_nan());
    assert_eq!(sample.position, Vec2::new(10.0, 10.0));
}

#[test]
fn test_lerp_angle_all_quadrants_and_extremes() {
    // Exact PI to -PI wrap
    let a = PI;
    let b = -PI;
    let mid = lerp_angle(a, b, 0.5);
    assert!((mid.abs() - PI).abs() < 1e-4);

    // 359 deg to 1 deg
    let deg359 = 359.0f32.to_radians(); // ~6.265 rad -> normalized to -0.017 rad
    let deg1 = 1.0f32.to_radians(); // ~0.017 rad
    let mid_deg = lerp_angle(deg359, deg1, 0.5);
    assert!(mid_deg.abs() < 1e-3); // Should cross through 0 deg
}
