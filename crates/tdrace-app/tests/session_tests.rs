use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_session_initialization() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::Menu);

    session.init_race();
    assert_eq!(session.cars.len(), 4); // 1 player + 3 default bots
    assert_eq!(session.trackers.len(), 4);
    assert_eq!(session.ai_drivers.len(), 3);
    assert_eq!(session.color_schemes.len(), 4);

    match session.state {
        GameState::Countdown(t) => assert!(t > 0.0),
        _ => panic!("Expected Countdown state after init_race"),
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
