use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_session_initialization() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::Menu);

    session.init_race();
    let expected_cars = 1 + session.num_bots;
    assert_eq!(session.cars.len(), expected_cars);
    assert_eq!(session.trackers.len(), expected_cars);
    assert_eq!(session.ai_drivers.len(), session.num_bots);
    assert_eq!(session.color_schemes.len(), expected_cars);

    match session.state {
        GameState::StartingGrid => (),
        _ => panic!("Expected StartingGrid state after init_race vs AI"),
    }
}


#[test]
fn test_session_track_and_car_selection() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::KartArena;
    session.car_choice = CarChoice::Kart;
    session.num_bots = 5;
    session.init_race();

    assert_eq!(session.track.name, "Kart Arena");
    assert_eq!(session.cars.len(), 6);
    assert_eq!(session.cars[0].config.mass, 180.0); // Kart mass
}

#[test]
fn test_session_time_attack_mode() {
    let mut session = RaceSession::new();
    session.is_time_attack = true;
    session.init_race();

    assert_eq!(session.cars.len(), 1); // Solo player
    assert_eq!(session.ai_drivers.len(), 0);
}

#[test]
fn test_session_standings_computation() {
    let mut session = RaceSession::new();
    session.num_bots = 3;
    session.init_race();

    // Advance car 2 to lap 2
    session.trackers[2].current_lap = 2;
    session.trackers[2].normalized_progress = 0.35;

    // Advance car 0 to lap 1 with 0.80 progress
    session.trackers[0].current_lap = 1;
    session.trackers[0].normalized_progress = 0.80;

    // Car 1 on lap 1 with 0.40 progress
    session.trackers[1].current_lap = 1;
    session.trackers[1].normalized_progress = 0.40;

    let standings = session.compute_standings();
    assert_eq!(standings[0], 2); // Car 2 is P1
    assert_eq!(standings[1], 0); // Car 0 is P2
    assert_eq!(standings[2], 1); // Car 1 is P3
}

#[test]
fn test_player_lap_tracking_advancement_and_finish() {
    let mut session = RaceSession::new();
    session.init_race();

    assert_eq!(session.prev_player_lap, 1);
    assert_eq!(session.trackers[0].current_lap, 1);

    // Simulate crossing start/finish line for lap 2
    session.trackers[0].current_lap = 2;

    // Run physics step
    session.physics_step(1.0 / 60.0);

    // prev_player_lap must be synchronized to lap 2 so sound triggers only once
    assert_eq!(session.prev_player_lap, 2);

    // Subsequent steps must remain synchronized
    session.physics_step(1.0 / 60.0);
    assert_eq!(session.prev_player_lap, 2);
    assert_eq!(session.trackers[0].current_lap, 2);

    // Complete all laps (e.g., total_laps + 1)
    session.trackers[0].current_lap = session.total_laps + 1;
    session.physics_step(1.0 / 60.0);
    assert_eq!(session.prev_player_lap, session.total_laps + 1);

    // Check race finish transition
    session.check_race_finish();
    assert_eq!(
        session.state,
        GameState::Finished,
        "State should transition to Finished upon race completion"
    );
}

#[test]
fn test_session_update_state_preservation_and_race_start() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::Menu);

    // 1. Initializing race puts state into StartingGrid
    session.init_race();
    assert_eq!(session.state, GameState::StartingGrid);

    // 2. update() MUST NOT overwrite StartingGrid back to Menu
    session.update();
    assert_eq!(session.state, GameState::StartingGrid);

    // 3. Countdown state must be preserved during update()
    session.state = GameState::Countdown(3.0);
    session.update();
    match session.state {
        GameState::Countdown(rem) => assert!(rem < 3.0, "Countdown should progress with frame dt"),
        _ => panic!("Expected Countdown state to be preserved across update()"),
    }

    // 4. Racing state must be preserved during update()
    session.state = GameState::Racing;
    session.update();
    assert_eq!(session.state, GameState::Racing);

    // 5. Paused state must be preserved during update()
    session.state = GameState::Paused;
    session.update();
    assert_eq!(session.state, GameState::Paused);

    // 6. Finished state must be preserved during update()
    session.state = GameState::Finished;
    session.update();
    assert_eq!(session.state, GameState::Finished);

    // 7. ControlsHelp state must be preserved during update()
    session.state = GameState::ControlsHelp(false);
    session.update();
    assert_eq!(session.state, GameState::ControlsHelp(false));
}

