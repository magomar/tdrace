use tdrace_app::replay::{PlaybackSpeed, Replay, ReplayPlayer, ReplayRecorder};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::CarConfig;

#[test]
fn test_record_and_playback_1000_step_determinism() {
    let track = classic_grand_prix();
    let car_choice = CarChoice::SportsCar;
    let track_choice = TrackChoice::ClassicGrandPrix;
    let seed = 123456789;
    let dt = 1.0 / 120.0;

    let init_pose = track.grid_positions[0];
    let mut car = Car::new(CarConfig::sports_car()).with_pose(init_pose.position, init_pose.angle);
    let mut tracker = TrackProgressTracker::new(track.checkpoints.len(), 3);
    let mut recorder = ReplayRecorder::new(track_choice, car_choice, seed, dt);

    // Run 1,000 distinct, dynamic simulation steps
    for step in 0..1000 {
        let steer = if step < 200 {
            0.0
        } else if step < 450 {
            0.65 // Corner turn
        } else if step < 600 {
            -0.35 // Counter-steer
        } else if step < 750 {
            0.80 // Sharp hairpin
        } else {
            0.0
        };

        let throttle = if step < 500 {
            1.0
        } else if step < 650 {
            0.0
        } else {
            0.9
        };

        let brake = if step >= 450 && step < 550 { 0.8 } else { 0.0 };
        let handbrake = step >= 480 && step < 520; // Handbrake drift initiation

        let controls = CarControls {
            throttle,
            steer,
            brake,
            handbrake,
            reverse: false,
        };

        // Record frame
        recorder.record_frame(controls, &car, &tracker);

        // Step vehicle physics
        let surfaces = track.sample_car_surfaces(&car);
        car.step_per_wheel(&controls, surfaces, dt);
        tracker.update(&car, &track.spline, &track.checkpoints, dt);
    }

    let recorded_final_pos = car.state.position;
    let recorded_final_vel = car.state.velocity;
    let recorded_final_angle = car.state.angle;

    // Finalize recording
    let replay = recorder.finish(tracker.best_lap_time);
    assert_eq!(replay.frames.len(), 1000);
    assert!(!replay.keyframes.is_empty());
    assert_eq!(replay.header.total_frames, 1000);

    // 1. Test binary serialization & round-trip
    let bytes = replay.to_bytes().expect("Failed to serialize replay to bytes");
    assert!(!bytes.is_empty());
    let reloaded_from_bytes = Replay::from_bytes(&bytes).expect("Failed to deserialize replay from bytes");
    assert_eq!(reloaded_from_bytes.header, replay.header);
    assert_eq!(reloaded_from_bytes.frames.len(), replay.frames.len());

    // 2. Test JSON serialization & round-trip
    let json_str = replay.to_json().expect("Failed to serialize replay to JSON");
    let reloaded_from_json = Replay::from_json(&json_str).expect("Failed to deserialize replay from JSON");
    assert_eq!(reloaded_from_json.header, replay.header);
    assert_eq!(reloaded_from_json.keyframes.len(), replay.keyframes.len());

    // 3. Verify 100% trajectory match and bit-level determinism
    let player = ReplayPlayer::new(reloaded_from_bytes);
    let max_error = player.verify_determinism().expect("Replay determinism verification failed!");
    assert!(
        max_error < 1e-4,
        "Replay determinism error too high: {:.8}",
        max_error
    );

    // 4. Test exact final step trajectory match
    let mut replay_car = Car::new(CarConfig::sports_car()).with_pose(init_pose.position, init_pose.angle);
    for frame in &player.replay.frames {
        let surfaces = track.sample_car_surfaces(&replay_car);
        replay_car.step_per_wheel(&frame.controls, surfaces, dt);
    }

    let pos_diff = (replay_car.state.position - recorded_final_pos).length();
    let vel_diff = (replay_car.state.velocity - recorded_final_vel).length();
    let angle_diff = (replay_car.state.angle - recorded_final_angle).abs();

    assert!(pos_diff < 1e-4, "Final position mismatch: {}", pos_diff);
    assert!(vel_diff < 1e-4, "Final velocity mismatch: {}", vel_diff);
    assert!(angle_diff < 1e-4, "Final angle mismatch: {}", angle_diff);
}

#[test]
fn test_replay_playback_engine_controls() {
    let track_choice = TrackChoice::OvalSpeedway;
    let car_choice = CarChoice::DriftCar;
    let mut recorder = ReplayRecorder::new(track_choice, car_choice, 999, 1.0 / 120.0);

    let dummy_car = Car::new(CarConfig::drift_car());
    let dummy_tracker = TrackProgressTracker::new(4, 3);

    for i in 0..120 {
        let ctrl = CarControls {
            throttle: (i as f32) / 120.0,
            steer: 0.5,
            brake: 0.0,
            handbrake: false,
            reverse: false,
        };
        recorder.record_frame(ctrl, &dummy_car, &dummy_tracker);
    }

    let replay = recorder.finish(None);
    let mut player = ReplayPlayer::new(replay);

    // Initial state
    assert_eq!(player.current_frame, 0);
    assert_eq!(player.speed, PlaybackSpeed::Speed1x);
    assert!(!player.is_finished);

    // Step 1 tick forward
    let step_ctrl = player.step(1.0 / 120.0);
    assert!(step_ctrl.is_some());
    assert_eq!(player.current_frame, 1);

    // Test speed cycling: 1x -> 2x -> 4x -> 8x -> 1x
    player.cycle_speed();
    assert_eq!(player.speed, PlaybackSpeed::Speed2x);
    assert_eq!(player.speed.multiplier(), 2.0);

    player.cycle_speed();
    assert_eq!(player.speed, PlaybackSpeed::Speed4x);
    assert_eq!(player.speed.multiplier(), 4.0);

    player.cycle_speed();
    assert_eq!(player.speed, PlaybackSpeed::Speed8x);
    assert_eq!(player.speed.multiplier(), 8.0);

    player.cycle_speed();
    assert_eq!(player.speed, PlaybackSpeed::Speed1x);

    // Test pause toggle
    player.toggle_pause();
    assert_eq!(player.speed, PlaybackSpeed::Paused);
    assert_eq!(player.speed.multiplier(), 0.0);
    let paused_ctrl = player.step(1.0 / 60.0);
    assert!(paused_ctrl.is_none());
    assert_eq!(player.current_frame, 1); // stayed at frame 1

    player.toggle_pause();
    assert_eq!(player.speed, PlaybackSpeed::Speed1x);

    // Test scrubbing
    player.scrub_to_frame(60);
    assert_eq!(player.current_frame, 60);
    assert!((player.current_time() - 0.5).abs() < 1e-4);

    player.scrub_to_time(0.75); // frame 90
    assert_eq!(player.current_frame, 90);

    // Scrub past end
    player.scrub_to_frame(200);
    assert_eq!(player.current_frame, 120);
    assert!(player.is_finished);
}

#[test]
fn test_corrupted_replay_error_handling() {
    let empty_bytes: Vec<u8> = vec![];
    assert!(Replay::from_bytes(&empty_bytes).is_err());

    let invalid_magic = b"XYZ9some_random_payload_here";
    assert!(Replay::from_bytes(invalid_magic).is_err());

    let invalid_json = "{ invalid_json: true ";
    assert!(Replay::from_json(invalid_json).is_err());
}
