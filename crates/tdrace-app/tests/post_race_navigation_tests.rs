use glam::Vec2;
use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::{FinishedScreenView, GameState, LapTelemetry, RaceSession};
use tdrace_app::ui::menu::TrackChoice;

#[test]
fn test_post_race_initial_screen_is_results() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    assert_eq!(session.finished_view, FinishedScreenView::Results);
    assert!(!session.show_hall_of_fame);

    // Simulate finishing race
    session.session_time = 42.5;
    session.trackers[0].current_lap = session.total_laps + 1;
    session.trackers[0].best_lap_time = Some(14.0);

    session.check_race_finish();

    assert_eq!(session.state, GameState::Finished);
    assert_eq!(session.finished_view, FinishedScreenView::Results);
    assert!(!session.show_hall_of_fame);
    assert!(!session.player_race_stats.laps.is_empty());
}

#[test]
fn test_post_race_navigation_forward_and_backward_cycle() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.init_race();
    session.state = GameState::Finished;
    session.finished_view = FinishedScreenView::Results;
    session.show_hall_of_fame = false;

    // 1. In Results, confirm/Space advances to HallOfFame
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    assert_eq!(session.state, GameState::Finished);
    assert_eq!(session.finished_view, FinishedScreenView::HallOfFame);
    assert!(session.show_hall_of_fame);

    // 2. In HallOfFame, ESC moves back to Results
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_cancel_pressed = false;

    assert_eq!(session.state, GameState::Finished);
    assert_eq!(session.finished_view, FinishedScreenView::Results);
    assert!(!session.show_hall_of_fame);

    // 3. In Results, advance again to HallOfFame
    session.input.gamepad.snapshot.btn_a_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_a_pressed = false;

    assert_eq!(session.finished_view, FinishedScreenView::HallOfFame);
    assert!(session.show_hall_of_fame);

    // 4. In HallOfFame, confirm/Space advances to Main Menu (not restart!)
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    // Transitioning to Menu
    assert!(session.transition.is_some() || session.state == GameState::Menu);
    assert_ne!(session.state, GameState::Countdown(3.5));
}

#[test]
fn test_restart_exclusive_to_r_key_or_gamepad_y() {
    let mut session = RaceSession::new();
    session.init_race();
    session.state = GameState::Finished;
    session.finished_view = FinishedScreenView::Results;

    // Space/Confirm must NOT trigger restart
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;
    assert_eq!(session.state, GameState::Finished);
    assert_eq!(session.finished_view, FinishedScreenView::HallOfFame);

    // Trigger restart via gamepad Y (secondary action / R)
    session.input.gamepad.snapshot.btn_y_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_y_pressed = false;

    // Must trigger iris transition to countdown
    assert!(session.transition.is_some() || matches!(session.state, GameState::Countdown(_)));
}

#[test]
fn test_detailed_race_statistics_tab_navigation() {
    let mut session = RaceSession::new();
    session.init_race();
    session.state = GameState::Finished;
    session.finished_view = FinishedScreenView::Results;

    // Press TAB / gamepad X from Results
    session.input.gamepad.snapshot.btn_x_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_x_pressed = false;

    assert_eq!(session.finished_view, FinishedScreenView::Statistics);
    assert_eq!(session.finished_prev_view, FinishedScreenView::Results);

    // Press TAB / gamepad X from Statistics returns to Results
    session.input.gamepad.snapshot.btn_x_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_x_pressed = false;

    assert_eq!(session.finished_view, FinishedScreenView::Results);

    // Advance to HallOfFame
    session.finished_view = FinishedScreenView::HallOfFame;
    session.show_hall_of_fame = true;

    // Press TAB / gamepad X from HallOfFame opens Statistics
    session.input.gamepad.snapshot.btn_x_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_x_pressed = false;

    assert_eq!(session.finished_view, FinishedScreenView::Statistics);
    assert_eq!(session.finished_prev_view, FinishedScreenView::HallOfFame);

    // ESC from Statistics returns back to HallOfFame
    session.input.gamepad.snapshot.btn_cancel_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_cancel_pressed = false;

    assert_eq!(session.finished_view, FinishedScreenView::HallOfFame);
    assert!(session.show_hall_of_fame);
}

#[test]
fn test_player_race_telemetry_and_acrobatic_metrics() {
    let mut session = RaceSession::new();
    session.init_race();

    // Verify initial telemetry state
    assert!(session.player_race_stats.laps.is_empty());
    assert_eq!(session.player_race_stats.stunt_stats.total_stunt_score, 0);

    // Populate lap telemetry
    session.player_race_stats.laps.push(LapTelemetry {
        lap_number: 1,
        lap_time: 25.5,
        sector_times: vec![8.2, 8.5, 8.8],
        is_personal_best: true,
    });
    session.player_race_stats.laps.push(LapTelemetry {
        lap_number: 2,
        lap_time: 24.1,
        sector_times: vec![7.9, 8.1, 8.1],
        is_personal_best: true,
    });
    session.player_race_stats.best_lap_idx = Some(1);

    // Accumulate stunt metrics
    session.player_race_stats.stunt_stats.drift_count = 3;
    session.player_race_stats.stunt_stats.total_drift_points = 850;
    session.player_race_stats.stunt_stats.max_single_drift_score = 420.0;

    session.player_race_stats.stunt_stats.jump_count = 2;
    session.player_race_stats.stunt_stats.total_air_time = 1.45;
    session.player_race_stats.stunt_stats.longest_jump_time = 0.85;
    session.player_race_stats.stunt_stats.jump_points = 360;

    session.player_race_stats.stunt_stats.max_combo = 4;
    session.player_race_stats.stunt_stats.total_stunt_score = 1210;

    assert_eq!(session.player_race_stats.laps.len(), 2);
    assert_eq!(session.player_race_stats.best_lap_idx, Some(1));
    assert_eq!(session.player_race_stats.laps[1].lap_time, 24.1);
    assert_eq!(session.player_race_stats.stunt_stats.total_stunt_score, 1210);
}

#[test]
fn test_racing_simulation_tracks_jumps_and_drifts() {
    let mut session = RaceSession::new();
    session.init_race();
    assert_eq!(session.active_module_id, "classic");
    assert!(session.is_stunt_scoring_enabled());

    // Verify initial telemetry
    assert_eq!(session.player_race_stats.stunt_stats.jump_count, 0);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 0);

    // 1. Simulate jump landing
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.elevation = 0.01;
        player_car.state.vertical_velocity = -5.0;
        player_car.state.air_time = 0.85; // Standard jump (< 1.50s)
    }

    session.physics_step(0.016);
    assert_eq!(session.player_race_stats.stunt_stats.jump_count, 1);
    assert!((session.player_race_stats.stunt_stats.total_air_time - 0.85).abs() < 0.01);
    assert_eq!(session.player_race_stats.stunt_stats.jump_points, (0.85f32 * 250.0).round() as u32);
    assert!(session.player_race_stats.stunt_stats.total_stunt_score > 0);

    // 2. Simulate drift ending
    session.prev_player_drifting = true;
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.is_drifting = false;
        player_car.state.drift_score = 150.0;
    }

    session.physics_step(0.016);
    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 1);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 150);
}

#[test]
fn test_stunt_scoring_disabled_on_non_classic_modules() {
    let mut session = RaceSession::new();
    session.active_module_id = "gt";
    session.init_race();
    assert!(!session.is_stunt_scoring_enabled());

    // 1. Simulate jump landing on GT circuit
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.elevation = 0.01;
        player_car.state.vertical_velocity = -5.0;
        player_car.state.air_time = 1.60;
    }

    session.physics_step(0.016);
    // Stunt scoring must remain 0 on non-classic module
    assert_eq!(session.player_race_stats.stunt_stats.jump_count, 0);
    assert_eq!(session.player_race_stats.stunt_stats.jump_points, 0);
    assert_eq!(session.player_race_stats.stunt_stats.total_stunt_score, 0);

    // 2. Simulate drift ending on GT circuit
    session.prev_player_drifting = true;
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.is_drifting = false;
        player_car.state.drift_score = 300.0;
    }

    session.physics_step(0.016);
    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 0);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 0);
    assert_eq!(session.player_race_stats.stunt_stats.total_stunt_score, 0);

    // 3. Manual override allows re-enabling if desired
    session.set_stunt_scoring_enabled(Some(true));
    assert!(session.is_stunt_scoring_enabled());

    session.prev_player_drifting = true;
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.is_drifting = false;
        player_car.state.drift_score = 200.0;
    }

    session.physics_step(0.016);
    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 1);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 200);
}

#[test]
fn test_mega_jump_requires_one_point_five_seconds() {
    let mut session = RaceSession::new();
    session.init_race();
    assert!(session.is_stunt_scoring_enabled());

    // 1. Jump of 1.20s should award AIR TIME alert, NOT MEGA JUMP
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.elevation = 0.01;
        player_car.state.vertical_velocity = -5.0;
        player_car.state.air_time = 1.20;
    }
    session.physics_step(0.016);

    assert!(session.floating_text.items.iter().any(|item| item.text.contains("AIR TIME 1.20s")));
    assert!(!session.floating_text.items.iter().any(|item| item.text.contains("MEGA JUMP")));

    // Clear floating text for next jump test
    session.floating_text.clear();

    // 2. Jump of 1.60s (>= 1.50s) should award MEGA JUMP alert
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.elevation = 0.01;
        player_car.state.vertical_velocity = -5.0;
        player_car.state.air_time = 1.60;
    }
    session.physics_step(0.016);

    assert!(session.floating_text.items.iter().any(|item| item.text.contains("MEGA JUMP! 1.60s")));
}

#[test]
fn test_collision_voids_drift_and_breaks_combo() {
    let mut session = RaceSession::new();
    session.init_race();
    assert!(session.is_stunt_scoring_enabled());

    // 1. Establish an active combo streak and in-progress drift
    session.drift_combo_count = 3;
    session.drift_combo_timer = 3.5;
    session.prev_player_drifting = true;
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.is_drifting = true;
        player_car.state.drift_score = 160.0;
        player_car.state.position = Vec2::new(-1.0, 0.0);
        player_car.state.velocity = Vec2::new(10.0, 0.0);
        player_car.state.angle = 0.0;
    }

    // Place second car approaching head-on
    if let Some(car_b) = session.cars.get_mut(1) {
        car_b.state.position = Vec2::new(1.0, 0.0);
        car_b.state.velocity = Vec2::new(-10.0, 0.0);
        car_b.state.angle = std::f32::consts::PI;
    }

    session.physics_step(0.016);

    // Collision must void drift score, reset combo streak, and activate lockout
    assert_eq!(session.drift_combo_count, 0, "Combo must be broken by collision");
    assert_eq!(session.drift_combo_timer, 0.0);
    assert!(session.player_collision_stunt_lockout > 0.0, "Collision lockout must activate");
    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 0, "No drift should be awarded");
    assert_eq!(session.player_race_stats.stunt_stats.total_stunt_score, 0);
    assert_eq!(session.cars[0].state.drift_score, 0.0, "Drift score must be voided to 0");
    assert!(!session.cars[0].state.is_drifting);

    // Verify visual feedback for broken combo / voided drift
    assert!(
        session.floating_text.items.iter().any(|item| item.text.contains("COMBO BROKEN!") || item.text.contains("DRIFT VOIDED!")),
        "UI must display broken combo alert"
    );
}

#[test]
fn test_collision_lockout_prevents_spinout_stunt_points() {
    let mut session = RaceSession::new();
    session.init_race();
    assert!(session.is_stunt_scoring_enabled());

    // Activate collision recovery lockout
    session.player_collision_stunt_lockout = 1.0;

    // Simulate car spinning / sliding violently after impact
    session.prev_player_drifting = true;
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.is_drifting = false;
        player_car.state.drift_score = 250.0;
    }

    session.physics_step(0.016);

    // Lockout must suppress drift completion and wipe drift score
    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 0);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 0);
    assert_eq!(session.player_race_stats.stunt_stats.total_stunt_score, 0);
    assert_eq!(session.drift_combo_count, 0);
    assert_eq!(session.cars[0].state.drift_score, 0.0);
}

#[test]
fn test_clean_drift_resets_drift_score_after_banking() {
    let mut session = RaceSession::new();
    session.init_race();
    assert!(session.is_stunt_scoring_enabled());
    assert_eq!(session.player_collision_stunt_lockout, 0.0);

    // 1. First clean drift
    session.prev_player_drifting = true;
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.is_drifting = false;
        player_car.state.drift_score = 120.0;
    }

    session.physics_step(0.016);

    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 1);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 120);
    assert_eq!(session.cars[0].state.drift_score, 0.0, "Drift score must be reset to 0 after banking");

    // 2. A subsequent step without drifting must NOT re-bank points
    session.physics_step(0.016);
    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 1);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 120);
    assert_eq!(session.cars[0].state.drift_score, 0.0);

    // 3. Second clean drift accumulates from fresh 0 baseline
    session.prev_player_drifting = true;
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.is_drifting = false;
        player_car.state.drift_score = 90.0;
    }

    session.physics_step(0.016);

    assert_eq!(session.player_race_stats.stunt_stats.drift_count, 2);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 210);
    assert_eq!(session.cars[0].state.drift_score, 0.0);
}
