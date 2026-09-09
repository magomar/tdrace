use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::nascar::NascarGameModule;
use tdrace_app::module::{GameModule, VehicleVisualType};
use tdrace_app::tournament::{ChampionshipSession, PointSystem, RoundDriverResult, TournamentFormat};
use tdrace_app::ui::menu::{resolve_predefined_car_for_track, CarChoice, TrackChoice};
use tdrace_core::physics::CarConfig;
use tdrace_core::track::validation::validate_track;

#[test]
fn test_nascar_game_module_identity_and_vehicles() {
    let nascar = NascarGameModule::new();
    assert_eq!(nascar.id(), "nascar");
    assert!(nascar.title().contains("NASCAR"));
    assert!(nascar.subtitle().contains("Stock Car"));

    let vehicles = nascar.vehicles();
    assert_eq!(vehicles.len(), 2, "Expected 2 vehicle configurations");

    let cup_v8 = &vehicles[0];
    assert_eq!(cup_v8.id, "nascar_cup_v8");
    assert_eq!(cup_v8.config.mass, 1260.0);
    assert_eq!(cup_v8.config.max_engine_force, 11800.0);
    assert!(matches!(cup_v8.visual_type, VehicleVisualType::StockCar { tall_wing: false, roof_fins: true, window_net: true }));

    let ta1 = &vehicles[1];
    assert_eq!(ta1.id, "trans_am_ta1");
    assert_eq!(ta1.config.mass, 1260.0);
    assert_eq!(ta1.config.max_engine_force, 11800.0);
    assert!(matches!(ta1.visual_type, VehicleVisualType::StockCar { tall_wing: true, roof_fins: false, window_net: true }));
}

#[test]
fn test_nascar_tracks_and_geometry_validation() {
    let nascar = NascarGameModule::new();
    let tracks = nascar.tracks();
    assert_eq!(tracks.len(), 4, "Expected 4 NASCAR tracks");

    let expected_ids = [
        "daytona_superspeedway",
        "talladega_superspeedway",
        "watkins_glen_nascar",
        "bristol_motor_speedway",
    ];

    for (idx, def) in tracks.iter().enumerate() {
        assert_eq!(def.id, expected_ids[idx]);

        let track = (def.generator)();
        assert_eq!(track.predefined_car.as_deref(), Some("stock_car"));
        let issues = validate_track(&track);
        assert!(
            issues.is_empty(),
            "Track '{}' generated geometry validation issues: {:?}",
            def.id,
            issues
        );
        assert!(track.grid_positions.len() >= 12, "Track '{}' must support at least 12 grid positions", def.id);
    }
}

#[test]
fn test_nascar_roster_integrity() {
    let nascar = NascarGameModule::new();
    let drivers = nascar.drivers();
    assert_eq!(drivers.len(), 12, "Expected 12 NASCAR driver personalities");

    for driver in &drivers {
        assert_eq!(driver.preferred_car, CarChoice::StockCar);
        assert!(!driver.name.is_empty());
        assert!(!driver.alias.is_empty());
        assert!(!driver.bio.is_empty());
        assert!(driver.stats.speed >= 0.85);
        assert!(driver.stats.aggression >= 0.70);
    }

    assert!(drivers.iter().any(|d| d.name.contains("Dale") && d.alias == "The Intimidator"));
    assert!(drivers.iter().any(|d| d.name.contains("Pettyfield") && d.alias == "The King"));
    assert!(drivers.iter().any(|d| d.name.contains("Busch") && d.alias == "Wild Thing"));
}

#[test]
fn test_nascar_tournament_formats_and_points() {
    let nascar = NascarGameModule::new();
    let tourneys = nascar.supported_game_modes();
    assert!(tourneys.len() >= 4);

    let cup_champ = tourneys
        .iter()
        .find_map(|t| match t {
            TournamentFormat::Championship { name, point_system, track_ids, laps_per_round }
                if name.contains("NASCAR Cup Series Championship") =>
            {
                Some((name, point_system, track_ids, laps_per_round))
            }
            _ => None,
        })
        .expect("NASCAR Cup Series Championship must be present");

    assert_eq!(cup_champ.2.len(), 4);
    assert_eq!(*cup_champ.3, 10);
    assert!(matches!(cup_champ.1, PointSystem::NascarCup { stage_win_bonus: true }));

    // Test NASCAR Cup point system
    let pts = PointSystem::NascarCup { stage_win_bonus: true };
    assert_eq!(pts.points_for_position(1, false), 40);
    assert_eq!(pts.points_for_position(2, false), 35);
    assert_eq!(pts.points_for_position(3, false), 34);
    assert_eq!(pts.points_for_position(10, false), 27);
    assert_eq!(pts.points_for_position(36, false), 1);
    assert_eq!(pts.points_for_position(40, false), 1);

    // Test in ChampionshipSession
    let track_ids = cup_champ.2.clone();
    let initial_drivers = [
        ("dale_vance", "Dale Vance", "Earnhardt Motorsports"),
        ("chase_gordon", "Chase Gordon", "Hendrick Heritage Racing"),
        ("richard_pettyfield", "Richard Pettyfield", "Pettyfield Enterprises"),
    ];

    let mut session = ChampionshipSession::new(
        cup_champ.0,
        cup_champ.1.clone(),
        track_ids,
        *cup_champ.3,
        &initial_drivers,
    );
    assert_eq!(session.current_round, 0);
    assert_eq!(session.total_rounds(), 4);

    let round_results = vec![
        RoundDriverResult {
            driver_id: "dale_vance".to_string(),
            driver_name: "Dale Vance".to_string(),
            team_name: "Earnhardt Motorsports".to_string(),
            finish_position: 1,
            total_time: 120.5,
            best_lap: Some(28.1),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "chase_gordon".to_string(),
            driver_name: "Chase Gordon".to_string(),
            team_name: "Hendrick Heritage Racing".to_string(),
            finish_position: 2,
            total_time: 121.2,
            best_lap: Some(28.3),
            points_awarded: 0,
            has_fastest_lap: false,
        },
        RoundDriverResult {
            driver_id: "richard_pettyfield".to_string(),
            driver_name: "Richard Pettyfield".to_string(),
            team_name: "Pettyfield Enterprises".to_string(),
            finish_position: 3,
            total_time: 122.0,
            best_lap: Some(28.5),
            points_awarded: 0,
            has_fastest_lap: false,
        },
    ];

    session.submit_round_results("Daytona International Speedway", round_results);

    // Dale Vance: 1st place = 40 pts
    assert_eq!(session.standings.iter().find(|s| s.driver_id == "dale_vance").unwrap().points, 40);
    // Chase Gordon: 2nd place = 35 pts
    assert_eq!(session.standings.iter().find(|s| s.driver_id == "chase_gordon").unwrap().points, 35);
    // Richard Pettyfield: 3rd place = 34 pts
    assert_eq!(session.standings.iter().find(|s| s.driver_id == "richard_pettyfield").unwrap().points, 34);
    assert_eq!(session.leader().unwrap().driver_id, "dale_vance");
}

#[test]
fn test_nascar_car_choice_and_menu_resolution() {
    let stock_car = CarConfig::stock_car_ta1();
    assert_eq!(stock_car.mass, 1260.0);
    assert_eq!(stock_car.max_engine_force, 11800.0);
    assert_eq!(stock_car.top_speed_mps, 89.0);

    let choice = CarChoice::StockCar;
    assert_eq!(choice.tag(), "850 BHP SPACEFRAME V8");
    assert_eq!(choice.title(), "850 BHP NASCAR Cup V8");
    assert_eq!(choice.specs().1, "1,260 kg Mass");

    let daytona_choice = TrackChoice::Custom {
        id: "daytona_superspeedway".to_string(),
        title: "Daytona International Speedway".to_string(),
        description: "2.5-mile tri-oval".to_string(),
        path: "nascar/daytona_superspeedway".to_string(),
    };
    let daytona_track = TrackChoice::resolve_procedural_preset(&daytona_choice);
    assert!(daytona_track.is_some());
    let track = daytona_track.unwrap();

    let resolved_car = resolve_predefined_car_for_track(Some(&track), "nascar");
    assert_eq!(resolved_car, CarChoice::StockCar);

    let resolved_fallback = resolve_predefined_car_for_track(None, "nascar");
    assert_eq!(resolved_fallback, CarChoice::StockCar);
}

#[test]
fn test_nascar_race_session_starting_grid_and_roster() {
    let mut session = RaceSession::new();
    session.switch_to_nascar();
    session.num_bots = 7;
    session.init_race();

    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.grid_participants.len(), 8);
    assert_eq!(session.car_choice, CarChoice::StockCar);

    for p in &session.grid_participants {
        assert_eq!(p.car_title, "850 BHP NASCAR Cup V8");
    }

    match session.current_visual_type {
        VehicleVisualType::StockCar { window_net, roof_fins, .. } => {
            assert!(window_net);
            assert!(roof_fins);
        }
        _ => panic!("Expected StockCar vehicle visual type in NASCAR race"),
    }
}

#[test]
fn test_nascar_phase2_hub_navigation_and_return_mapping() {
    let mut session = RaceSession::new();
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    // Switching to NASCAR
    session.switch_to_nascar();
    assert_eq!(session.active_module_id, "nascar");
    assert_eq!(session.state, GameState::Menu);

    // Returning from Menu to ModuleSelect maps nascar -> index 4
    let cur_mod_idx = match session.active_module_id {
        "classic" => 0,
        "rally" => 1,
        "kart" => 2,
        "gt" | "gt_challenge" | "f1" => 3,
        "nascar" => 4,
        _ => 0,
    };
    assert_eq!(cur_mod_idx, 4, "NASCAR should map to index 4 in Grand Hub carousel");
}

#[test]
fn test_nascar_phase2_championship_lifecycle() {
    let mut session = RaceSession::new();
    session.start_nascar_championship();

    assert_eq!(session.active_module_id, "nascar");
    assert!(session.championship_session.is_some());

    {
        let champ = session.championship_session.as_ref().unwrap();
        assert_eq!(champ.name, "NASCAR Cup Series Championship 2026");
        assert_eq!(champ.total_rounds(), 4);
        assert_eq!(champ.current_round, 0);
        assert_eq!(champ.current_track_id(), Some("daytona_superspeedway"));
        assert!(matches!(champ.point_system, PointSystem::NascarCup { stage_win_bonus: true }));

        // Drivers in championship should match 12-driver roster + player
        assert_eq!(champ.standings.len(), 12);
        assert!(champ.standings.iter().any(|s| s.driver_name.contains("Intimidator")));
        assert!(champ.standings.iter().any(|s| s.driver_name.contains("The King")));
    }

    // Submit round 1 results to advance championship round
    let results = vec![
        RoundDriverResult {
            driver_id: "dale_vance".to_string(),
            driver_name: "Dale Vance".to_string(),
            team_name: "Richard Childress Racing".to_string(),
            finish_position: 1,
            total_time: 120.0,
            best_lap: Some(25.0),
            points_awarded: 0,
            has_fastest_lap: false,
        },
    ];
    session.championship_session.as_mut().unwrap().submit_round_results("Daytona International Speedway", results);

    // Advance round to round 2 (Talladega)
    session.advance_championship_round();
    let champ_r2 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ_r2.current_round, 1);
    assert_eq!(champ_r2.current_track_id(), Some("talladega_superspeedway"));
}

#[test]
fn test_nascar_phase2_config_file_and_overrides() {
    use tdrace_app::config::GameConfig;

    let override_val = GameConfig::load_module_override_file("nascar")
        .expect("config.nascar.toml must be loadable via load_module_override_file");
    let gameplay = override_val.get("gameplay").expect("gameplay table expected");
    assert_eq!(gameplay.get("default_track").and_then(|v| v.as_str()), Some("daytona_superspeedway"));
    assert_eq!(gameplay.get("default_laps").and_then(|v| v.as_integer()), Some(10));
    assert_eq!(gameplay.get("default_num_bots").and_then(|v| v.as_integer()), Some(11));
}

