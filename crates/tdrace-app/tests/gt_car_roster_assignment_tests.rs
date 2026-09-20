use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::module::{GameModule, VehicleVisualType};
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

#[test]
fn test_gt_game_module_drivers_and_preferred_car() {
    let gt = GtWorldChallengeModule::new();
    let drivers = gt.drivers();
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
fn test_gt_tracks_predefined_car_and_resolve_predefined_car() {
    let gt = GtWorldChallengeModule::new();
    let track_defs = gt.tracks();
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
fn test_gt_race_roster_car_assignment_and_display_titles() {
    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 7;
    session.init_race();

    // Roster / Starting grid state
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.grid_participants.len(), 8); // 1 Player + 7 GT opponents

    // Player car title on roster screen (Monza defaults to GT4 Clubsport)
    let player = session
        .grid_participants
        .iter()
        .find(|p| p.is_player)
        .expect("Player must exist in grid participants");
    assert_eq!(
        player.car_title,
        "420 BHP GT4 Clubsport",
        "Player car title must be '420 BHP GT4 Clubsport'"
    );

    // All AI opponents on roster screen must be assigned cars from the GT category pool
    let eligible = session.eligible_opponent_cars();
    let eligible_titles: Vec<&'static str> = eligible.iter().map(|c| c.title()).collect();
    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    for participant in &opponents {
        assert!(
            eligible_titles.contains(&participant.car_title.as_str()),
            "Participant '{}' vehicle '{}' must belong to the eligible GT category pool",
            participant.name,
            participant.car_title
        );
    }

    // Verify there is variety across opponents (not everyone has identical clones)
    let unique_opponent_cars: std::collections::HashSet<_> =
        opponents.iter().map(|p| &p.car_title).collect();
    assert!(
        unique_opponent_cars.len() > 1,
        "Opponents should have varied car models assigned, found: {:?}",
        unique_opponent_cars
    );

    // Verify visual archetype is TouringGT
    match session.current_visual_type {
        VehicleVisualType::TouringGT { gt_wing, .. } => {
            assert!(gt_wing);
        }
        _ => panic!("Expected TouringGT vehicle visual type in GT World Challenge race"),
    }

    // Verify all cars in session are tuned with GT physics (top speed > 260 km/h, downforce >= 0.85)
    for car in &session.cars {
        assert!(
            car.config.top_speed_mps * 3.6 > 260.0,
            "Car top speed must exceed 260 km/h for GT spec"
        );
        assert!(
            car.config.downforce_coefficient >= 0.85,
            "Car downforce must be at least 0.85 for GT spec"
        );
    }
}

#[test]
fn test_gt_free_car_selection_toggle_in_roster() {
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
fn test_gt_championship_roster_and_car_assignment() {
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

#[test]
fn test_player_elected_car_sets_authentic_title_in_roster() {
    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 3;

    // Elect BMW M4 GT4
    session.selected_car_model_id = Some("gt_bmw_m4_gt4");
    session.free_car_selection = true;
    session.rebuild_roster_participants();

    let player = session
        .grid_participants
        .iter()
        .find(|p| p.is_player)
        .expect("Player participant must exist");

    assert_eq!(
        player.car_title, "BMW M4 GT4 (G82)",
        "Player car title in roster must display authentic model name"
    );
    assert_eq!(
        player.model_id,
        Some("gt_bmw_m4_gt4"),
        "Player participant model_id must match elected vehicle"
    );
    assert_eq!(
        session.car_model_ids[0],
        Some("gt_bmw_m4_gt4"),
        "First car in session must have elected model ID"
    );
}

#[test]
fn test_bots_assigned_same_category_when_player_elects_car() {
    use tdrace_app::catalog::{find_model_by_id, get_models_for_category};

    let mut session = RaceSession::new();
    session.switch_to_gt();
    session.num_bots = 7;

    // Elect Porsche 718 Cayman GT4 (GT4 Clubsport category)
    session.selected_car_model_id = Some("gt_porsche_718_gt4");
    session.free_car_selection = true;
    session.rebuild_roster_participants();

    assert_eq!(session.grid_participants.len(), 8);
    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.car_model_ids.len(), 8);

    let gt4_models = get_models_for_category("gt", "GT4 Clubsport");
    let gt4_ids: Vec<&str> = gt4_models.iter().map(|m| m.id).collect();

    // Verify all bot participants in roster are in the same category
    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        let mid = p.model_id.expect("Bot participant must have a model_id");
        assert!(
            gt4_ids.contains(&mid),
            "Bot '{}' model '{}' must belong to GT4 Clubsport category, eligible: {:?}",
            p.name,
            mid,
            gt4_ids
        );

        let model = find_model_by_id(mid).unwrap();
        assert_eq!(
            p.car_title, model.name,
            "Bot '{}' car title must match its authentic model name",
            p.name
        );
    }

    // Verify all cars in session have corresponding model IDs
    for (i, &mid_opt) in session.car_model_ids.iter().enumerate() {
        let mid = mid_opt.expect("Every car must have an authentic model_id");
        assert!(
            gt4_ids.contains(&mid),
            "Car {} with model '{}' must belong to GT4 category",
            i,
            mid
        );
    }
}

#[test]
fn test_category_parity_across_multiple_disciplines() {
    use tdrace_app::catalog::get_models_for_category;

    let mut session = RaceSession::new();
    session.num_bots = 5;

    // 1. Rally WRC
    session.switch_to_rally();
    session.selected_car_model_id = Some("rally_hyundai_i20_rx");
    session.free_car_selection = true;
    session.rebuild_roster_participants();

    let player = session.grid_participants.iter().find(|p| p.is_player).unwrap();
    assert_eq!(player.car_title, "Hyundai i20 RX Supercar");

    let wrc_models = get_models_for_category("rally", "WRC / RX Supercar");
    let wrc_ids: Vec<&str> = wrc_models.iter().map(|m| m.id).collect();

    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        let mid = p.model_id.unwrap();
        assert!(
            wrc_ids.contains(&mid),
            "Rally bot model '{}' must belong to WRC category",
            mid
        );
    }

    // 2. NASCAR Trans-Am
    session.switch_to_nascar();
    session.selected_car_model_id = Some("nascar_mustang_ta1");
    session.free_car_selection = true;
    session.rebuild_roster_participants();

    let player = session.grid_participants.iter().find(|p| p.is_player).unwrap();
    assert_eq!(player.car_title, "Ford Mustang TA1");

    let ta_models = get_models_for_category("nascar", "Trans-Am TA1");
    let ta_ids: Vec<&str> = ta_models.iter().map(|m| m.id).collect();

    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        let mid = p.model_id.unwrap();
        assert!(
            ta_ids.contains(&mid),
            "NASCAR bot model '{}' must belong to Trans-Am TA1 category",
            mid
        );
    }

    // 3. Karting KZ2 Shifter
    session.switch_to_kart();
    session.selected_car_model_id = Some("kart_tony_kart_racer_kz");
    session.free_car_selection = true;
    session.rebuild_roster_participants();

    let player = session.grid_participants.iter().find(|p| p.is_player).unwrap();
    assert_eq!(player.car_title, "Tony Kart Racer 401 KZ");

    let kz_models = get_models_for_category("kart", "Shifter Kart 125cc KZ2");
    let kz_ids: Vec<&str> = kz_models.iter().map(|m| m.id).collect();

    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        let mid = p.model_id.unwrap();
        assert!(
            kz_ids.contains(&mid),
            "Kart bot model '{}' must belong to KZ2 category",
            mid
        );
    }
}

#[test]
fn test_classic_module_preserves_procedural_cars() {
    let mut session = RaceSession::new();
    session.switch_to_classic();
    session.num_bots = 3;
    session.init_race();

    assert_eq!(session.active_module_id, "classic");
    for p in &session.grid_participants {
        assert_eq!(
            p.model_id, None,
            "Classic participant '{}' must not have a model_id",
            p.name
        );
    }
    for (i, &mid) in session.car_model_ids.iter().enumerate() {
        assert_eq!(
            mid, None,
            "Car {} in classic session must not have a model_id",
            i
        );
    }
}


