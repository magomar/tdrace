use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::extreme_offroad::ExtremeOffRoadModule;
use tdrace_app::module::{GameModule, VehicleVisualType};
use tdrace_app::tournament::{RoundDriverResult, TournamentFormat};
use tdrace_app::ui::menu::{resolve_predefined_car_for_track, CarChoice};
use tdrace_core::track::validation::{validate_track, ValidationSeverity};

#[test]
fn test_extreme_offroad_module_identity_and_vehicles() {
    let offroad = ExtremeOffRoadModule::new();
    assert_eq!(offroad.id(), "extreme_offroad");
    assert!(offroad.title().contains("EXTREME OFF-ROAD"));
    assert!(offroad.subtitle().contains("Baja"));

    let vehicles = offroad.vehicles();
    assert_eq!(vehicles.len(), 1, "Expected 1 vehicle configuration for Sand Rail Buggy");

    let buggy = &vehicles[0];
    assert_eq!(buggy.id, "sand_rail_buggy");
    assert_eq!(buggy.config.mass, 590.0);
    assert_eq!(buggy.config.drive_bias, 0.0);
    assert_eq!(buggy.config.max_engine_force, 8800.0);
    assert_eq!(buggy.config.max_brake_force, 11500.0);
    assert_eq!(buggy.config.handbrake_force, 8200.0);
    assert_eq!(buggy.config.wheelbase, 2.40);
    assert_eq!(buggy.config.track_width, 1.75);
    assert!(matches!(
        buggy.visual_type,
        VehicleVisualType::SandRail {
            lightbar: true,
            whip_antenna: true,
            paddle_tires: true
        }
    ));
}

#[test]
fn test_extreme_offroad_tracks_and_geometry_validation() {
    let offroad = ExtremeOffRoadModule::new();
    let tracks = offroad.tracks();
    assert_eq!(tracks.len(), 17, "Expected 17 Extreme Off-Road tracks & arenas");

    let expected_ids = [
        "sahara_dune_crossing",
        "dirt_figure_eight",
        "atacama_sand_basin",
        "red_rock_canyon",
        "mud_slough_arena",
        "baja_500_desert_scrub",
        "arctic_frozen_lake",
        "alpine_snow_ridge",
        "rovaniemi_ice_ring",
        "supercross_stadium_arena",
        "gravel_quarry_chasm",
        "louisiana_mud_swampland",
        "monster_colosseum",
        "glacier_crest_pass",
        "stunt_city_megastructure",
        "glamis_dunes",
        "crandon_short_course",
    ];

    let tracks_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tracks")
        .join("extreme_offroad");
    let _ = std::fs::create_dir_all(&tracks_dir);

    for (idx, def) in tracks.iter().enumerate() {
        assert_eq!(def.id, expected_ids[idx]);
        assert!(
            def.default_laps >= 2 && def.default_laps <= 5,
            "Track '{}' default laps ({}) must be between 2 and 5",
            def.id,
            def.default_laps
        );

        let track = (def.generator)();
        let target_file = tracks_dir.join(format!("{}.json", def.id));
        if !target_file.exists() {
            if let Ok(json_str) = serde_json::to_string_pretty(&track) {
                let _ = std::fs::write(&target_file, json_str);
            }
        }

        assert!(
            track.default_laps >= 2 && track.default_laps <= 5,
            "Track '{}' generator default laps ({}) must be between 2 and 5",
            def.id,
            track.default_laps
        );
        assert_eq!(
            def.default_laps,
            track.default_laps,
            "Track '{}' def.default_laps ({}) != track.default_laps ({})",
            def.id,
            def.default_laps,
            track.default_laps
        );
        let diags = validate_track(&track);
        let errors: Vec<_> = diags
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "Track '{}' generated geometry validation errors: {:?}",
            def.id,
            errors
        );
        assert!(
            track.grid_positions.len() >= 8,
            "Track '{}' must support at least 8 grid positions",
            def.id
        );
    }
}

#[test]
fn test_extreme_offroad_roster_integrity() {
    let offroad = ExtremeOffRoadModule::new();
    let drivers = offroad.drivers();
    assert_eq!(drivers.len(), 8, "Expected 8 off-road driver personalities");

    for driver in &drivers {
        assert_eq!(driver.preferred_car, CarChoice::SandRail);
        assert!(!driver.name.is_empty());
        assert!(!driver.alias.is_empty());
        assert!(!driver.bio.is_empty());
        assert!(driver.stats.speed >= 0.90);
        assert!(driver.stats.aggression >= 0.85);
    }

    assert!(drivers.iter().any(|d| d.name == "Wyatt Cole" && d.alias == "Dust Devil"));
    assert!(drivers.iter().any(|d| d.name == "Jaxson Rivera" && d.alias == "Baja King"));
    assert!(drivers.iter().any(|d| d.name == "Astrid Lindholm" && d.alias == "Ice Queen"));
    assert!(drivers.iter().any(|d| d.name == "Bubba Beauregard" && d.alias == "Mud Slinger"));
    assert!(drivers.iter().any(|d| d.name == "Travis McGrath" && d.alias == "Nitro"));
    assert!(drivers.iter().any(|d| d.name == "Roxie Vance" && d.alias == "Rock Hound"));
    assert!(drivers.iter().any(|d| d.name == "Sven Lindqvist" && d.alias == "Blizzard"));
    assert!(drivers.iter().any(|d| d.name == "Cruz Morales" && d.alias == "Chasm Jumper"));
}

#[test]
fn test_extreme_offroad_tournament_formats() {
    let offroad = ExtremeOffRoadModule::new();
    let tourneys = offroad.supported_game_modes();
    assert_eq!(tourneys.len(), 4);

    let champ = tourneys
        .iter()
        .find_map(|t| match t {
            TournamentFormat::Championship { name, track_ids, .. } => {
                Some((name.clone(), track_ids.clone()))
            }
            _ => None,
        })
        .expect("Championship format missing");
    assert_eq!(champ.0, "Extreme Off-Road World Series");
    assert_eq!(champ.1.len(), 15);

    let stage_rally = tourneys
        .iter()
        .find_map(|t| match t {
            TournamentFormat::StageRally { name, stage_track_ids } => {
                Some((name.clone(), stage_track_ids.clone()))
            }
            _ => None,
        })
        .expect("Stage Rally format missing");
    assert_eq!(stage_rally.0, "Desert & Ice Raid Tour");
    assert_eq!(stage_rally.1.len(), 9);
}

#[test]
fn test_extreme_offroad_session_switch_and_car_choice() {
    let mut session = RaceSession::new();
    session.switch_to_extreme_offroad();

    assert_eq!(session.active_module_id, "extreme_offroad");
    assert_eq!(session.car_choice, CarChoice::SandRail);
    assert_eq!(session.resolve_predefined_car(), CarChoice::SandRail);

    let (title, tag, _desc, (spd, acc, grp, dft)) = (
        CarChoice::SandRail.title(),
        CarChoice::SandRail.tag(),
        CarChoice::SandRail.description(),
        CarChoice::SandRail.stats(),
    );
    assert_eq!(title, "300 BHP Sand Rail Buggy");
    assert_eq!(tag, "300 BHP RWD ULTRALIGHT");
    assert!(spd > 0.8 && acc > 0.9 && grp > 0.7 && dft > 0.8);

    let track = tdrace_core::track::presets::sahara_dune_crossing();
    assert_eq!(
        resolve_predefined_car_for_track(Some(&track), "extreme_offroad"),
        CarChoice::SandRail
    );

    session.init_race();
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.cars[0].config.mass, 590.0);
    assert_eq!(session.cars[0].config.drive_bias, 0.0);
    assert!(matches!(
        session.current_visual_type,
        VehicleVisualType::SandRail { .. }
    ));
}

#[test]
fn test_extreme_offroad_championship_flow() {
    let mut session = RaceSession::new();
    session.start_extreme_offroad_championship();

    assert!(session.championship_session.is_some());
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.name, "Extreme Off-Road World Series 2026");
    assert_eq!(champ.track_ids.len(), 15);
    assert_eq!(champ.current_round, 0);
    assert_eq!(champ.current_track_id(), Some("sahara_dune_crossing"));

    let results = vec![
        RoundDriverResult {
            driver_id: "player".to_string(),
            driver_name: "Player".to_string(),
            team_name: "Sand Rail Dynamics".to_string(),
            finish_position: 1,
            total_time: 120.0,
            best_lap: Some(25.0),
            points_awarded: 25,
            has_fastest_lap: false,
        },
    ];
    session.championship_session.as_mut().unwrap().submit_round_results("Sahara Dune Crossing", results);

    session.advance_championship_round();
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.current_round, 1);
    assert_eq!(champ.current_track_id(), Some("dirt_figure_eight"));
}
