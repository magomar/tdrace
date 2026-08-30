use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_session_initialization() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

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
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

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

#[test]
fn test_main_menu_exit_confirmation_state() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });
    assert!(!session.show_exit_confirm);

    // Triggering exit confirmation modal
    session.show_exit_confirm = true;
    assert!(session.show_exit_confirm, "show_exit_confirm should be true when exit modal is open");
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 }, "State should remain GameState::ModuleSelect");

    // Dismissing exit confirmation modal
    session.show_exit_confirm = false;
    assert!(!session.show_exit_confirm, "show_exit_confirm should be false after dismissal");
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 }, "State should remain GameState::ModuleSelect");
}

#[test]
fn test_all_races_and_modules_default_to_eight_riders() {
    // 1. Default initialization
    let mut session = RaceSession::new();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Default session should have 8 riders (1 player + 7 bots)");
    assert_eq!(session.ai_drivers.len(), 7, "Default session should have 7 AI bots");
    assert_eq!(session.opponent_drivers.len(), 7);

    // 2. Switch to F1
    session.switch_to_f1();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "F1 module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 3. Switch to Rally
    session.switch_to_rally();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Rally module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 4. Switch to Kart
    session.switch_to_kart();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Kart module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 5. Switch to Classic
    session.switch_to_classic();
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Classic module should have 8 riders");
    assert_eq!(session.ai_drivers.len(), 7);

    // 6. Test all preset tracks in classic mode
    for track_choice in [
        TrackChoice::ClassicGrandPrix,
        TrackChoice::OvalSpeedway,
        TrackChoice::DriftPark,
        TrackChoice::KartArena,
        TrackChoice::RampRaceway,
        TrackChoice::OasisRally,
        TrackChoice::OutlawPass,
    ] {
        session.track_choice = track_choice;
        session.init_race();
        assert_eq!(
            session.cars.len(),
            8,
            "Track {:?} should have 8 riders by default",
            session.track_choice
        );
        assert_eq!(session.ai_drivers.len(), 7);
        assert_eq!(session.opponent_drivers.len(), 7);
    }
}

#[test]
fn test_grid_positioning_fast_lap_earns_pole() {
    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    // Set a fast player personal best on this track
    session.active_profile_stats.best_times.insert("classic_grand_prix".to_string(), 18.2);
    session.active_profile_stats.best_circuit_times.insert("classic_grand_prix".to_string(), 55.0);

    session.init_race();

    // Player should be at grid slot 0 (Pole Position)
    let player_slot = session.grid_participants.iter().position(|p| p.is_player).unwrap();
    assert_eq!(player_slot, 0, "Player with fastest lap time should start on Pole (slot 0)");

    // Verify car 0 is spawned at grid_positions[0]
    let player_car = &session.cars[0];
    let grid_pose_0 = &session.track.grid_positions[0];
    assert!((player_car.state.position.x - grid_pose_0.position.x).abs() < 1e-3);
    assert!((player_car.state.position.y - grid_pose_0.position.y).abs() < 1e-3);
}

#[test]
fn test_grid_positioning_slower_lap_placed_behind() {
    use tdrace_app::ai::DriverCharacter;
    use tdrace_app::db::HallOfFameEntry;

    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;

    // All possible opponents have 20.0s in HoF
    session.hof_entries = DriverCharacter::all()
        .iter()
        .map(|d| HallOfFameEntry {
            id: Some(1),
            track_id: "classic_grand_prix".to_string(),
            player_name: d.name.to_string(),
            car_name: "GT Sports Coupe".to_string(),
            total_time: 60.0,
            best_lap: Some(20.0),
            laps: 3,
            created_at: "2026-08-27 10:00".to_string(),
        })
        .collect();

    // Player has slower lap time 25.0s
    session.active_profile_stats.best_times.insert("classic_grand_prix".to_string(), 25.0);
    session.active_profile_stats.best_circuit_times.insert("classic_grand_prix".to_string(), 75.0);

    // Rebuild grid
    session.rebuild_roster_participants();

    let player_slot = session.grid_participants.iter().position(|p| p.is_player).unwrap();

    // Player with 25.0s should be placed behind all opponents with 20.0s
    assert_eq!(player_slot, session.grid_participants.len() - 1, "Player with 25.0s should be at back of grid");
}

#[test]
fn test_grid_positioning_tie_broken_by_circuit_time() {
    use tdrace_app::ai::DriverCharacter;
    use tdrace_app::db::HallOfFameEntry;

    let mut session = RaceSession::new();
    session.track_choice = TrackChoice::ClassicGrandPrix;

    // All opponents have 22.0s lap and 68.0s circuit time
    session.hof_entries = DriverCharacter::all()
        .iter()
        .map(|d| HallOfFameEntry {
            id: Some(1),
            track_id: "classic_grand_prix".to_string(),
            player_name: d.name.to_string(),
            car_name: "GT Sports Coupe".to_string(),
            total_time: 68.0,
            best_lap: Some(22.0),
            laps: 3,
            created_at: "2026-08-27 10:00".to_string(),
        })
        .collect();

    // Player has identical best lap (22.0s), but FASTER circuit time (64.0s)
    session.active_profile_stats.best_times.insert("classic_grand_prix".to_string(), 22.0);
    session.active_profile_stats.best_circuit_times.insert("classic_grand_prix".to_string(), 64.0);

    session.rebuild_roster_participants();

    let player_slot = session.grid_participants.iter().position(|p| p.is_player).unwrap();

    assert_eq!(player_slot, 0, "Player with faster circuit time on tied lap should earn P1");
}

#[test]
fn test_grid_positioning_all_slots_unique_and_valid() {
    let mut session = RaceSession::new();
    session.init_race();

    assert_eq!(session.grid_participants.len(), 8);
    // Ensure all cars are placed at valid unique positions
    let mut positions = Vec::new();
    for (i, p) in session.grid_participants.iter().enumerate() {
        let car_pose = if p.is_player {
            session.cars[0].state.position
        } else {
            let bot_idx = p.bot_index.unwrap();
            session.cars[bot_idx + 1].state.position
        };
        let expected_slot_pose = session.track.grid_positions[i].position;
        assert!((car_pose.x - expected_slot_pose.x).abs() < 1e-3);
        assert!((car_pose.y - expected_slot_pose.y).abs() < 1e-3);
        positions.push((car_pose.x.to_bits(), car_pose.y.to_bits()));
    }
    // Verify all 8 car positions are unique
    positions.sort();
    positions.dedup();
    assert_eq!(positions.len(), 8, "All 8 cars must spawn in distinct grid positions");
}

#[test]
fn test_race_session_camera_zoom_in_and_out() {
    let mut session = RaceSession::new();
    session.init_race();
    assert_eq!(session.camera.current_level_idx, 0);
    assert_eq!(session.camera.current_zoom_level().name, "Close");

    // Continuous progressive zoom in/out
    let init_max = session.camera.max_zoom_scale;
    session.camera.zoom_progressive(1.0, 1.0, 0.2);
    assert!(session.camera.max_zoom_scale > init_max);
    session.camera.zoom_progressive(-1.0, 1.0, 0.2);
    assert!((session.camera.max_zoom_scale - init_max).abs() < 0.1);

    // Zoom level cycling with Tab
    let lvl1 = session.camera.cycle_zoom_level();
    assert_eq!(lvl1.name, "Medium");
    assert_eq!(session.camera.current_level_idx, 1);

    let lvl2 = session.camera.cycle_zoom_level();
    assert_eq!(lvl2.name, "Far");
    assert_eq!(session.camera.current_level_idx, 2);

    let lvl3 = session.camera.cycle_zoom_level();
    assert_eq!(lvl3.name, "Overview");
    assert_eq!(session.camera.current_level_idx, 3);

    let lvl0 = session.camera.cycle_zoom_level();
    assert_eq!(lvl0.name, "Close");
    assert_eq!(session.camera.current_level_idx, 0);
}


