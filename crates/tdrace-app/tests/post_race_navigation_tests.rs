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

    // Verify initial telemetry
    assert_eq!(session.player_race_stats.stunt_stats.jump_count, 0);
    assert_eq!(session.player_race_stats.stunt_stats.total_drift_points, 0);

    // 1. Simulate jump landing
    if let Some(player_car) = session.cars.first_mut() {
        player_car.state.elevation = 0.01;
        player_car.state.vertical_velocity = -5.0;
        player_car.state.air_time = 0.85; // 0.85s mega jump
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
