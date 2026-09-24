use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::gt::GtWorldChallengeModule;
use tdrace_app::module::{GameModule, VehicleVisualType};
use tdrace_app::ui::menu::{CarChoice, GameMode, TrackChoice};

#[test]
fn test_gt_game_module_drivers_and_preferred_car() {
    let gt = GtWorldChallengeModule::new();
    let drivers = gt.drivers();
    assert_eq!(drivers.len(), 12, "GT module must have 12 predefined driver characters");

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
        assert!(d.default_stats().speed >= 0.80, "GT driver speed stat must be top tier");
    }
}

#[test]
fn test_gt_tracks_predefined_car_and_resolve_predefined_car() {
    let gt = GtWorldChallengeModule::new();
    let track_defs = gt.tracks();
    assert_eq!(track_defs.len(), 18);

    for t_def in &track_defs {
        let track = (t_def.generator)();
        assert_eq!(
            track.car_category,
            tdrace_core::CarCategory::Gt,
            "Track '{}' must define car_category = Gt",
            t_def.id
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

    // Player car title on roster screen (Monza defaults to GT4 Clubsport authentic starter car)
    let player = session
        .grid_participants
        .iter()
        .find(|p| p.is_player)
        .expect("Player must exist in grid participants");
    assert_eq!(
        player.car_title,
        "Toyota GR Supra GT4 EVO",
        "Player car title must be 'Toyota GR Supra GT4 EVO'"
    );
    assert_eq!(player.model_id, Some("gt_toyota_supra_gt4"));

    // All AI opponents on roster screen must be assigned cars from the GT4 category pool
    let gt4_models = tdrace_app::catalog::get_models_for_category("gt", "GT4 Clubsport");
    let gt4_names: Vec<&'static str> = gt4_models.iter().map(|m| m.name).collect();
    let opponents: Vec<_> = session.grid_participants.iter().filter(|p| !p.is_player).collect();
    for participant in &opponents {
        assert!(
            gt4_names.contains(&participant.car_title.as_str()),
            "Participant '{}' vehicle '{}' must belong to the eligible GT4 models pool",
            participant.name,
            participant.car_title
        );
        assert!(participant.model_id.is_some());
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

    // Verify all cars in session are tuned with GT physics (top speed > 260 km/h, downforce >= 0.60)
    for car in &session.cars {
        assert!(
            car.config.top_speed_mps * 3.6 > 260.0,
            "Car top speed must exceed 260 km/h for GT spec"
        );
        assert!(
            car.config.downforce_coefficient >= 0.60,
            "Car downforce must be at least 0.60 for GT spec"
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

    // Enable free car selection and choose cross-discipline DriftCar for player
    session.free_car_selection = true;
    session.car_choice = CarChoice::DriftCar;
    session.rebuild_roster_participants();

    // Player gets DriftCar
    assert_eq!(session.active_player_car_choice(), CarChoice::DriftCar);
    assert_eq!(
        session.grid_participants.iter().find(|p| p.is_player).unwrap().car_title,
        "Tuned Drift Spec"
    );

    // AI opponents retain their character's preferred cars
    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        let bot_idx = p.bot_index.unwrap();
        let character = &session.opponent_drivers[bot_idx];
        assert_eq!(
            p.car_title,
            character.preferred_car.title(),
            "Bot '{}' should retain their preferred car with free car selection enabled",
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
    assert_eq!(session.cars.len(), session.max_grid_participants());
    assert_eq!(session.grid_participants.len(), session.max_grid_participants());

    let gt4_models = tdrace_app::catalog::get_models_for_category("gt", "GT4 Clubsport");
    let gt4_names: Vec<&'static str> = gt4_models.iter().map(|m| m.name).collect();
    for p in &session.grid_participants {
        assert!(
            gt4_names.contains(&p.car_title.as_str()),
            "Championship participant '{}' vehicle '{}' must belong to GT4 authentic models",
            p.name,
            p.car_title
        );
        assert!(p.model_id.is_some());
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
    assert_eq!(session.resolve_predefined_car(), CarChoice::SportsCar);
    assert_eq!(session.active_player_car_choice(), CarChoice::SportsCar);
    assert_eq!(session.active_player_car_choice().title(), "GT Sports Coupe");
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
fn test_classic_module_assigns_fantasy_category_cars() {
    let mut session = RaceSession::new();
    session.switch_to_classic();
    session.track_choice = TrackChoice::OvalSpeedway;
    session.track = session.load_track_for_session(&session.track_choice);
    session.num_bots = 3;
    session.init_race();

    assert_eq!(session.active_module_id, "classic");
    for p in &session.grid_participants {
        assert_eq!(
            p.model_id, Some("classic_nascar"),
            "Classic participant '{}' must have classic_nascar model_id on Oval",
            p.name
        );
        assert_eq!(
            p.car_title, "Thunderbolt Stock V8",
            "Classic participant '{}' must have Thunderbolt Stock V8 title",
            p.name
        );
    }
    for (i, &mid) in session.car_model_ids.iter().enumerate() {
        assert_eq!(
            mid, Some("classic_nascar"),
            "Car {} in classic session must have classic_nascar model_id",
            i
        );
    }
}

#[test]
fn test_gt_career_tier_initializes_unlocked_real_car_and_diverse_roster() {
    let mut session = RaceSession::new();
    session.start_gt_career_tier(1);

    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.selected_car_model_id, Some("gt_toyota_supra_gt4"));
    assert_eq!(session.car_choice, CarChoice::GT4Clubsport);
    assert_eq!(session.grid_participants.len(), session.max_grid_participants());

    let player = session.grid_participants.iter().find(|p| p.is_player).unwrap();
    assert_eq!(player.car_title, "Toyota GR Supra GT4 EVO");
    assert_eq!(player.model_id, Some("gt_toyota_supra_gt4"));
    assert_eq!(session.car_model_ids[0], Some("gt_toyota_supra_gt4"));

    let gt4_models = tdrace_app::catalog::get_models_for_category("gt", "GT4 Clubsport");
    let gt4_ids: Vec<&str> = gt4_models.iter().map(|m| m.id).collect();

    // Bots must use diverse authentic GT4 models, not generic 420 BHP GT4 Clubsport
    for p in session.grid_participants.iter().filter(|p| !p.is_player) {
        let mid = p.model_id.expect("Bot must have authentic model_id");
        assert!(gt4_ids.contains(&mid), "Bot model '{}' must belong to GT4", mid);
        assert_ne!(p.car_title, "420 BHP GT4 Clubsport", "Bot must have real car title");
    }

    for (i, &mid) in session.car_model_ids.iter().enumerate() {
        assert!(mid.is_some(), "Car {} must have modern model_id sprite assigned", i);
    }
}

#[test]
fn test_quick_race_gt_circuit_selection_matches_garage_car() {
    let mut session = RaceSession::new();
    session.switch_to_gt();

    // 1. Enter Quick Race modality
    session.game_mode = GameMode::StandardRace;
    session.free_car_selection = false;
    session.is_time_attack = false;

    // 2. Select circuit from active module tracks (e.g. Monza)
    let tracks = session.active_module_tracks();
    let monza_choice = tracks
        .iter()
        .find(|t| t.track_id() == "monza")
        .cloned()
        .expect("Monza track must exist in GT module");
    session.track_choice = monza_choice;
    let loaded = tdrace_app::ui::menu::resolve_track_for_menu(&session.track_choice);
    session.car_choice = tdrace_app::ui::menu::resolve_predefined_car_for_track(loaded.as_ref(), session.active_module_id);

    // 3. Initialize race (enters GameState::StartingGrid)
    session.init_race();
    assert_eq!(session.state, GameState::StartingGrid);

    // 4. Verify player active car is an authentic car model, NOT the generic "420 BHP GT4 Clubsport"
    let (player_car_title, player_model_id) = {
        let player = session
            .grid_participants
            .iter()
            .find(|p| p.is_player)
            .expect("Player participant must exist");
        (player.car_title.clone(), player.model_id)
    };

    assert_ne!(player_car_title, "420 BHP GT4 Clubsport", "Active car must not be generic prototype");
    assert!(player_model_id.is_some(), "Player participant must have authentic model_id");

    let active_model_id = session.selected_car_model_id.expect("Selected car model ID must be assigned");
    assert_eq!(player_model_id, Some(active_model_id));

    // 5. Open Garage from Starting Grid (simulates clicking/pressing Garage card)
    session.open_garage_from_starting_grid();
    assert_eq!(session.state, GameState::Garage(tdrace_app::game::GarageOrigin::StartingGrid));
    assert_eq!(session.garage_tier, 1, "Garage tier must match GT4 tier 1");

    // 6. Verify the focused car in the garage matches the active car on starting grid
    let tier_models = tdrace_app::catalog::get_models_for_module_and_tier("gt", 1);
    let focused_garage_car = tier_models[session.garage_car_idx];

    assert_eq!(
        focused_garage_car.id, active_model_id,
        "Car in garage must match the active car on the starting grid"
    );
    assert_eq!(
        focused_garage_car.name, player_car_title,
        "Car title in garage must match the active car title on the starting grid"
    );

    // 7. Verify the generic prototype car does NOT exist anywhere in the garage catalog
    let all_garage_cars = tdrace_app::catalog::get_models_for_module("gt");
    assert!(
        all_garage_cars.iter().all(|m| m.name != "420 BHP GT4 Clubsport"),
        "Generic car must not exist in the GT garage catalog"
    );
}



