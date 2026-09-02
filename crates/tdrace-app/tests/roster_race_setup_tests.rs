use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::{CarChoice, TrackChoice};
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway,
    ramp_raceway,
};
use tdrace_core::track::Track;

#[test]
fn test_preset_tracks_predefined_cars_and_balanced_laps() {
    let gp = classic_grand_prix();
    assert_eq!(gp.default_laps, 3);
    assert_eq!(gp.predefined_car.as_deref(), Some("sports_car"));

    let oval = oval_speedway();
    assert_eq!(oval.default_laps, 5);
    assert_eq!(oval.predefined_car.as_deref(), Some("sports_car"));

    let drift = drift_park();
    assert_eq!(drift.default_laps, 3);
    assert_eq!(drift.predefined_car.as_deref(), Some("drift_car"));

    let kart = kart_arena();
    assert_eq!(kart.default_laps, 5);
    assert_eq!(kart.predefined_car.as_deref(), Some("kart"));

    let ramp = ramp_raceway();
    assert_eq!(ramp.default_laps, 3);
    assert_eq!(ramp.predefined_car.as_deref(), Some("sports_car"));

    let oasis = oasis_rally();
    assert_eq!(oasis.default_laps, 3);
    assert_eq!(oasis.predefined_car.as_deref(), Some("rally_car"));

    let outlaw = outlaw_pass();
    assert_eq!(outlaw.default_laps, 3);
    assert_eq!(outlaw.predefined_car.as_deref(), Some("sports_car"));
}

#[test]
fn test_enforced_predefined_car_in_race_session() {
    let mut session = RaceSession::new();

    // 1. Select Kart Arena (enforced car = Kart, laps = 5)
    session.track_choice = TrackChoice::KartArena;
    session.free_car_selection = false;
    session.num_bots = 4;
    session.init_race();

    assert_eq!(session.total_laps, 5);
    assert_eq!(session.resolve_predefined_car(), CarChoice::Kart);
    assert_eq!(session.active_player_car_choice(), CarChoice::Kart);
    assert_eq!(session.cars.len(), 5); // 1 player + 4 bots

    // Verify player car top speed matches Kart specs (~32 m/s)
    let player_car = &session.cars[0];
    assert!((player_car.config.top_speed_mps - 32.0).abs() < 1.0);

    // Verify all bot cars use Kart specs when free car selection is disabled
    for bot_car in &session.cars[1..] {
        assert!((bot_car.config.top_speed_mps - 32.0).abs() < 1.0);
    }

    // 2. Select Drift Park (enforced car = DriftCar, laps = 3)
    session.track_choice = TrackChoice::DriftPark;
    session.free_car_selection = false;
    session.init_race();

    assert_eq!(session.total_laps, 3);
    assert_eq!(session.resolve_predefined_car(), CarChoice::DriftCar);
    assert_eq!(session.active_player_car_choice(), CarChoice::DriftCar);

    // Verify drift car max steer lock (~0.78 rad)
    for car in &session.cars {
        assert!((car.config.max_steer_angle - 0.78).abs() < 0.05);
    }
}

#[test]
fn test_free_car_selection_toggle_in_race_session() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::KartArena;
    session.free_car_selection = false;
    session.car_choice = CarChoice::SportsCar;
    session.num_bots = 3;
    session.init_race();

    // Enforced: player car is Kart even if session.car_choice was SportsCar
    assert_eq!(session.active_player_car_choice(), CarChoice::Kart);

    // Enable free car selection
    session.free_car_selection = true;
    session.rebuild_roster_participants();

    // Now player gets SportsCar
    assert_eq!(session.active_player_car_choice(), CarChoice::SportsCar);
    assert!((session.cars[0].config.top_speed_mps - 58.0).abs() < 1.0);

    // AI bots use their distinct preferred vehicles
    let bot_choices: Vec<CarChoice> = session
        .opponent_drivers
        .iter()
        .map(|d| d.preferred_car)
        .collect();
    assert_eq!(bot_choices.len(), 3);
}

#[test]
fn test_roster_driver_count_modification() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.num_bots = 3;
    session.init_race();

    assert_eq!(session.cars.len(), 4);
    assert_eq!(session.opponent_drivers.len(), 3);

    // Modify driver count to 7 bots (8 racers)
    session.num_bots = 7;
    session.rebuild_roster_participants();

    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.opponent_drivers.len(), 7);
    assert_eq!(session.trackers.len(), 8);
    assert_eq!(session.ai_drivers.len(), 7);

    // Modify driver count to 1 bot (2 racers)
    session.num_bots = 1;
    session.rebuild_roster_participants();

    assert_eq!(session.cars.len(), 2);
    assert_eq!(session.opponent_drivers.len(), 1);
    assert_eq!(session.trackers.len(), 2);
    assert_eq!(session.ai_drivers.len(), 1);
}

#[test]
fn test_track_serde_default_laps_and_predefined_car_roundtrip() {
    let track = classic_grand_prix();
    let json = track.to_json_pretty().expect("Must serialize to JSON");

    let deserialized = Track::from_json(&json).expect("Must deserialize from JSON");
    assert_eq!(deserialized.default_laps, 3);
    assert_eq!(deserialized.predefined_car.as_deref(), Some("sports_car"));

    // Test backwards-compatibility when fields are missing from JSON
    let mut val: serde_json::Value = serde_json::from_str(&json).expect("Parse json value");
    if let Some(obj) = val.as_object_mut() {
        obj.remove("default_laps");
        obj.remove("predefined_car");
    }
    let legacy_json = serde_json::to_string(&val).expect("Serialize stripped json");

    let legacy_track = Track::from_json(&legacy_json).expect("Must deserialize legacy JSON");
    assert_eq!(legacy_track.default_laps, 3);
    assert_eq!(legacy_track.predefined_car, None);
}

#[test]
fn test_2d_navigation_focus_and_cursor_state() {
    use tdrace_app::ui::menu::MenuPanelFocus;
    use tdrace_app::ui::starting_grid::StartingGridFocus;

    let mut session = RaceSession::new();

    // 1. Verify default Track Select Menu column focus
    assert_eq!(session.menu_focused_panel, MenuPanelFocus::LeftTracks);
    assert_eq!(session.menu_track_idx, 0);
    assert_eq!(session.menu_car_idx, 0);

    // Switch column focus to Right Vehicle
    session.menu_focused_panel = MenuPanelFocus::RightVehicle;
    assert_eq!(session.menu_focused_panel, MenuPanelFocus::RightVehicle);

    // 2. Initialize Race and verify Starting Grid default focus
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    assert_eq!(session.starting_grid_focus, StartingGridFocus::LeftSetup);
    assert_eq!(session.starting_grid_card_idx, 0); // Game Mode card
    assert_eq!(session.starting_grid_roster_idx, 0);

    // 3. Starting Grid 2D navigation: panel switching & card cycling
    session.starting_grid_card_idx = 1; // Vehicle Selection card
    assert_eq!(session.starting_grid_card_idx, 1);

    session.starting_grid_card_idx = 2; // Bot Count card
    assert_eq!(session.starting_grid_card_idx, 2);

    session.starting_grid_card_idx = 3; // Launch Race button
    assert_eq!(session.starting_grid_card_idx, 3);

    // Verify Starting Grid Launch button rect geometry
    let (bx, by, bw, bh) = tdrace_app::ui::starting_grid_launch_button_rect(1280.0, 720.0);
    assert!(bw > 200.0);
    assert!(bh > 30.0);
    assert!(bx > 0.0);
    assert!(by > 300.0);

    // Switch panel focus to Right Starting Grid Roster
    session.starting_grid_focus = StartingGridFocus::RightRoster;
    session.starting_grid_roster_idx = 2;
    assert_eq!(session.starting_grid_focus, StartingGridFocus::RightRoster);
    assert_eq!(session.starting_grid_roster_idx, 2);

    // 4. Pause Menu button cursor
    assert_eq!(session.pause_selected_btn, 0); // Resume
    session.pause_selected_btn = 1; // Exit
    assert_eq!(session.pause_selected_btn, 1);
}

