use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::f1::F1GameModule;
use tdrace_app::module::{GameModule, VehicleVisualType};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_f1_game_module_drivers_and_preferred_car() {
    let f1 = F1GameModule::new();
    let drivers = f1.drivers();
    assert_eq!(drivers.len(), 7, "F1 module must have 7 predefined driver characters");

    for d in &drivers {
        assert_eq!(
            d.preferred_car,
            CarChoice::F1Car,
            "Driver '{}' must have preferred_car = CarChoice::F1Car",
            d.name
        );
        assert!(!d.name.is_empty());
        assert!(!d.alias.is_empty());
        assert!(!d.bio.is_empty());
        assert!(d.stats.speed >= 0.90, "F1 driver speed stat must be top tier");
    }
}

#[test]
fn test_f1_tracks_predefined_car_and_resolve_predefined_car() {
    let f1 = F1GameModule::new();
    let track_defs = f1.tracks();
    assert_eq!(track_defs.len(), 14);

    for t_def in &track_defs {
        if t_def.id != "classic_grand_prix" {
            let track = (t_def.generator)();
            assert_eq!(
                track.predefined_car.as_deref(),
                Some("f1_car"),
                "Track '{}' must define predefined_car = 'f1_car'",
                t_def.id
            );
        }
    }

    let mut session = RaceSession::new();
    session.switch_to_f1();

    assert_eq!(session.resolve_predefined_car(), CarChoice::F1Car);
    assert_eq!(session.active_player_car_choice(), CarChoice::F1Car);
}

#[test]
fn test_f1_race_roster_car_assignment_and_display_titles() {
    let mut session = RaceSession::new();
    session.switch_to_f1();
    session.num_bots = 7;
    session.init_race();

    // Roster / Starting grid state
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.grid_participants.len(), 8); // 1 Player + 7 F1 opponents

    // Player car title on roster screen
    assert_eq!(
        session.grid_participants[0].car_title,
        "1050 BHP Hybrid F1 Turbo",
        "Player car title must be '1050 BHP Hybrid F1 Turbo'"
    );

    // All AI opponents on roster screen must have '1050 BHP Hybrid F1 Turbo'
    for participant in &session.grid_participants {
        assert_eq!(
            participant.car_title,
            "1050 BHP Hybrid F1 Turbo",
            "Participant '{}' must be assigned '1050 BHP Hybrid F1 Turbo' on roster",
            participant.name
        );
    }

    // Verify visual archetype is OpenWheel (exposed wings, halo)
    match session.current_visual_type {
        VehicleVisualType::OpenWheel { front_wing_span, halo, .. } => {
            assert!(front_wing_span > 1.5);
            assert!(halo);
        }
        _ => panic!("Expected OpenWheel vehicle visual type in F1 race"),
    }

    // Verify all cars in session are tuned with F1 physics (> 340 km/h top speed, downforce > 3.0)
    for car in &session.cars {
        assert!(
            car.config.top_speed_mps * 3.6 > 340.0,
            "Car top speed must exceed 340 km/h for F1 spec"
        );
        assert!(
            car.config.downforce_coefficient > 3.0,
            "Car downforce must exceed 3.0 for F1 spec"
        );
    }
}

#[test]
fn test_f1_free_car_selection_toggle_in_roster() {
    let mut session = RaceSession::new();
    session.switch_to_f1();
    session.num_bots = 3;
    session.init_race();

    assert_eq!(session.resolve_predefined_car(), CarChoice::F1Car);
    assert_eq!(session.active_player_car_choice(), CarChoice::F1Car);

    // Enable free car selection and choose a SportsCar for player
    session.free_car_selection = true;
    session.car_choice = CarChoice::SportsCar;
    session.rebuild_roster_participants();

    // Player gets SportsCar
    assert_eq!(session.active_player_car_choice(), CarChoice::SportsCar);
    assert_eq!(
        session.grid_participants.iter().find(|p| p.is_player).unwrap().car_title,
        "GT Sports Coupe"
    );

    // F1 AI opponents retain their preferred F1 cars
    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        assert_eq!(
            p.car_title,
            "1050 BHP Hybrid F1 Turbo",
            "F1 bot '{}' should prefer F1 car even with free car selection enabled",
            p.name
        );
    }
}

#[test]
fn test_f1_championship_roster_and_car_assignment() {
    let mut session = RaceSession::new();
    session.start_f1_championship();

    assert!(session.championship_session.is_some());
    assert_eq!(session.active_module_id, "f1");
    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.grid_participants.len(), 8);

    for p in &session.grid_participants {
        assert_eq!(p.car_title, "1050 BHP Hybrid F1 Turbo");
    }
}

#[test]
fn test_all_disciplines_car_assignment_integrity() {
    let mut session = RaceSession::new();

    // 1. F1 Module
    session.switch_to_f1();
    assert_eq!(session.resolve_predefined_car(), CarChoice::F1Car);
    assert_eq!(session.active_player_car_choice(), CarChoice::F1Car);
    assert_eq!(session.active_player_car_choice().title(), "1050 BHP Hybrid F1 Turbo");

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
