use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::f1::F1GameModule;
use tdrace_app::module::{GameModule, VehicleVisualType};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_f1_game_module_drivers_and_preferred_car() {
    let f1 = F1GameModule::new();
    let drivers = f1.drivers();
    assert_eq!(drivers.len(), 7, "GT module must have 7 predefined driver characters");

    for d in &drivers {
        assert_eq!(
            d.preferred_car,
            CarChoice::GT3Car,
            "Driver '{}' must have preferred_car = CarChoice::GT3Car",
            d.name
        );
        assert!(!d.name.is_empty());
        assert!(!d.alias.is_empty());
        assert!(!d.bio.is_empty());
        assert!(d.stats.speed >= 0.90, "GT driver speed stat must be top tier");
    }
}

#[test]
fn test_f1_tracks_predefined_car_and_resolve_predefined_car() {
    let f1 = F1GameModule::new();
    let track_defs = f1.tracks();
    assert_eq!(track_defs.len(), 15);

    for t_def in &track_defs {
        let track = (t_def.generator)();
        let expected_car = match t_def.id {
            "monza" | "red_bull_ring" | "nurburgring_gp" => "gt4_clubsport",
            "silverstone" | "catalunya" | "bathurst" => "gt3_evo",
            "spa" | "zandvoort" | "portimao_gp" => "gt2_biturbo",
            "suzuka" | "interlagos" | "le_mans_sarthe" => "gt1_legend",
            "monaco" | "madring" | "marina_bay" => "hypercar_prototype",
            other => panic!("Unexpected GT track ID: {}", other),
        };
        assert_eq!(
            track.predefined_car.as_deref(),
            Some(expected_car),
            "Track '{}' must define career level predefined_car = '{}'",
            t_def.id,
            expected_car
        );
    }

    let mut session = RaceSession::new();
    session.switch_to_gt();

    assert_eq!(session.resolve_predefined_car(), CarChoice::GT4Clubsport);
    assert_eq!(session.active_player_car_choice(), CarChoice::GT4Clubsport);
}

#[test]
fn test_f1_race_roster_car_assignment_and_display_titles() {
    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 7;
    session.init_race();

    // Roster / Starting grid state
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.grid_participants.len(), 8); // 1 Player + 7 GT opponents

    // Player car title on roster screen (Monza defaults to GT4 Clubsport)
    assert_eq!(
        session.grid_participants[0].car_title,
        "420 BHP GT4 Clubsport",
        "Player car title must be '420 BHP GT4 Clubsport'"
    );

    // All AI opponents on roster screen must have '420 BHP GT4 Clubsport'
    for participant in &session.grid_participants {
        assert_eq!(
            participant.car_title,
            "420 BHP GT4 Clubsport",
            "Participant '{}' must be assigned '420 BHP GT4 Clubsport' on roster",
            participant.name
        );
    }

    // Verify visual archetype is TouringGT
    match session.current_visual_type {
        VehicleVisualType::TouringGT { gt_wing, .. } => {
            assert!(gt_wing);
        }
        _ => panic!("Expected TouringGT vehicle visual type in GT World Challenge race"),
    }

    // Verify all cars in session are tuned with GT4 physics (~272 km/h top speed, downforce ~0.85)
    for car in &session.cars {
        assert!(
            car.config.top_speed_mps * 3.6 > 260.0,
            "Car top speed must exceed 260 km/h for GT4 spec"
        );
        assert!(
            car.config.downforce_coefficient >= 0.85,
            "Car downforce must be at least 0.85 for GT4 spec"
        );
    }
}

#[test]
fn test_f1_free_car_selection_toggle_in_roster() {
    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 3;
    session.init_race();

    assert_eq!(session.resolve_predefined_car(), CarChoice::GT4Clubsport);
    assert_eq!(session.active_player_car_choice(), CarChoice::GT4Clubsport);

    // Enable free car selection and choose experimental F1Car for player
    session.free_car_selection = true;
    session.car_choice = CarChoice::F1Car;
    session.rebuild_roster_participants();

    // Player gets experimental F1 car
    assert_eq!(session.active_player_car_choice(), CarChoice::F1Car);
    assert_eq!(
        session.grid_participants.iter().find(|p| p.is_player).unwrap().car_title,
        "1050 BHP Hybrid F1 Turbo (Experimental)"
    );

    // GT AI opponents retain their preferred GT3 cars
    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        assert_eq!(
            p.car_title,
            "600 BHP GT3 Evo Racer",
            "GT bot '{}' should prefer GT3 car even with free car selection enabled",
            p.name
        );
    }
}

#[test]
fn test_f1_championship_roster_and_car_assignment() {
    let mut session = RaceSession::new();
    session.start_gt_championship();

    assert!(session.championship_session.is_some());
    assert_eq!(session.active_module_id, "gt");
    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.grid_participants.len(), 8);

    for p in &session.grid_participants {
        assert_eq!(p.car_title, "420 BHP GT4 Clubsport");
    }
}

#[test]
fn test_all_disciplines_car_assignment_integrity() {
    let mut session = RaceSession::new();

    // 1. GT World Challenge Module
    session.switch_to_gt();
    assert_eq!(session.resolve_predefined_car(), CarChoice::GT4Clubsport);
    assert_eq!(session.active_player_car_choice(), CarChoice::GT4Clubsport);
    assert_eq!(session.active_player_car_choice().title(), "420 BHP GT4 Clubsport");

    // 2. Rally Module
    session.switch_to_rally();
    assert_eq!(session.resolve_predefined_car(), CarChoice::RallyCar);
    assert_eq!(session.active_player_car_choice(), CarChoice::RallyCar);
    assert_eq!(session.active_player_car_choice().title(), "AWD Turbo Rally");

    // 3. Kart Module
    session.switch_to_kart();
    assert_eq!(session.resolve_predefined_car(), CarChoice::Kart);
    assert_eq!(session.active_player_car_choice(), CarChoice::Kart);
    assert_eq!(session.active_player_car_choice().title(), "125cc Shifter Kart");

    // 4. Classic GP Preset
    session.switch_to_classic();
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.track = session.load_track_for_session(&session.track_choice);
    assert_eq!(session.resolve_predefined_car(), CarChoice::SportsCar);
    assert_eq!(session.active_player_car_choice(), CarChoice::SportsCar);
    assert_eq!(session.active_player_car_choice().title(), "GT Sports Coupe");

    // 5. Classic Drift Park Preset
    session.track_choice = TrackChoice::DriftPark;
    session.track = session.load_track_for_session(&session.track_choice);
    assert_eq!(session.resolve_predefined_car(), CarChoice::DriftCar);
    assert_eq!(session.active_player_car_choice(), CarChoice::DriftCar);
    assert_eq!(session.active_player_car_choice().title(), "Tuned Drift Spec");
}

