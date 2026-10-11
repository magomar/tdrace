//! Spec 098: GT tyre compound choice in the garage, race build side.

use tdrace_app::catalog::find_model_by_id;
use tdrace_app::game::RaceSession;
use tdrace_app::profile::{is_gt_garage_module, GT_GARAGE_COMPOUNDS};
use tdrace_app::ui::menu::GameMode;
use tdrace_core::physics::surface::CompoundId;

const GT_CAR: &str = "gt_yamato_hayate_t1";

/// Compound of every wheel of a car, both in its config and in the runtime wheel assemblies that
/// the cockpit badge and the wheel borders read (spec 089).
fn wheel_compounds(car: &tdrace_core::physics::Car) -> (Vec<CompoundId>, Vec<CompoundId>) {
    (
        car.config.wheels.iter().map(|w| w.compound.id).collect(),
        car.state.wheel_assemblies.iter().map(|a| a.config.compound.id).collect(),
    )
}

fn preset_compound(model_id: &str) -> CompoundId {
    find_model_by_id(model_id).expect("catalog model").to_car_config().wheels[0].compound.id
}

/// A session on its own in-memory database. `RaceSession::new()` opens the shared
/// `tdrace_records.db` of the working directory, and the garage saves its choice there.
fn fresh_session() -> RaceSession {
    let mut session = RaceSession::new();
    session.hof_db = Some(HallOfFameDb::open_in_memory().unwrap());
    session.refresh_profiles_and_stats();
    session
}

fn gt_session(num_bots: usize) -> RaceSession {
    let mut session = fresh_session();
    session.switch_to_gt();
    session.num_bots = num_bots;
    session
}

#[test]
fn a_gt_player_races_on_the_compound_chosen_in_the_garage() {
    let mut session = gt_session(3);
    assert_eq!(preset_compound(GT_CAR), CompoundId::MediumSlick, "GT presets are Medium");
    session.active_profile.car_compounds.insert(GT_CAR.to_string(), CompoundId::SoftSlick);
    session.init_race();

    assert_eq!(session.selected_car_model_id, Some(GT_CAR));
    let (config, assemblies) = wheel_compounds(&session.world.vehicles[0]);
    assert_eq!(config, vec![CompoundId::SoftSlick; 4], "Soft Slick on all four wheels");
    assert_eq!(assemblies, vec![CompoundId::SoftSlick; 4], "the badge and wheel borders read these");
    let tire = session.world.vehicles[0].config.wheels[0].tire_model;
    let medium_tire = find_model_by_id(GT_CAR).unwrap().to_car_config().wheels[0].tire_model;
    assert!(
        tire.peak_slip_angle_deg < medium_tire.peak_slip_angle_deg,
        "the Soft tyre peaks at a smaller slip angle than the Medium preset"
    );
}

#[test]
fn a_gt_player_without_a_choice_keeps_the_preset_compound() {
    let mut session = gt_session(2);
    session.init_race();
    let (config, assemblies) = wheel_compounds(&session.world.vehicles[0]);
    assert_eq!(config, vec![CompoundId::MediumSlick; 4]);
    assert_eq!(assemblies, vec![CompoundId::MediumSlick; 4]);
}

#[test]
fn bots_keep_their_preset_compound_when_the_player_chooses_soft() {
    let mut session = gt_session(5);
    session.active_profile.car_compounds.insert(GT_CAR.to_string(), CompoundId::SoftSlick);
    session.init_race();

    assert!(session.world.vehicles.len() >= 6);
    for (idx, car) in session.world.vehicles.iter().enumerate().skip(1) {
        let model_id = session.car_model_ids[idx].expect("bot catalog model");
        let (config, assemblies) = wheel_compounds(car);
        assert_eq!(config, vec![preset_compound(model_id); 4], "bot {idx} ({model_id}) config");
        assert_eq!(assemblies, vec![preset_compound(model_id); 4], "bot {idx} ({model_id}) assemblies");
    }
}

#[test]
fn player_two_keeps_the_preset_compound_in_split_screen() {
    let mut session = gt_session(3);
    session.game_mode = GameMode::SplitScreen;
    session.active_profile.car_compounds.insert(GT_CAR.to_string(), CompoundId::SoftSlick);
    session.init_race();

    assert!(session.world.vehicles.len() >= 2);
    let (p1, _) = wheel_compounds(&session.world.vehicles[0]);
    let (p2_config, p2_assemblies) = wheel_compounds(&session.world.vehicles[1]);
    assert_eq!(p1, vec![CompoundId::SoftSlick; 4]);
    assert_eq!(p2_config, vec![CompoundId::MediumSlick; 4], "player 2 does not inherit player 1's choice");
    assert_eq!(p2_assemblies, vec![CompoundId::MediumSlick; 4]);
    for (idx, car) in session.world.vehicles.iter().enumerate().skip(2) {
        let model_id = session.car_model_ids[idx].expect("bot catalog model");
        assert_eq!(wheel_compounds(car).0, vec![preset_compound(model_id); 4], "bot {idx}");
    }
}

#[test]
fn a_stored_choice_never_changes_a_stock_car() {
    let stock = "nascar_crossbow_montego_t1";
    let mut session = fresh_session();
    session.switch_to_nascar();
    session.num_bots = 2;
    // A stray entry (an old build or a hand-edited database) must not apply to a stock car.
    session.active_profile.car_compounds.insert(stock.to_string(), CompoundId::SoftSlick);
    session.init_race();

    assert_eq!(session.selected_car_model_id, Some(stock));
    let expected = preset_compound(stock);
    assert_eq!(expected, CompoundId::HardSlick, "stock cars race the series compound");
    let (config, assemblies) = wheel_compounds(&session.world.vehicles[0]);
    assert_eq!(config, vec![expected; 4]);
    assert_eq!(assemblies, vec![expected; 4]);
}

#[test]
fn garage_compound_applies_to_gt_cars_and_slicks_only() {
    let mut session = fresh_session();
    let profile = &mut session.active_profile;
    profile.car_compounds.insert("gt_a".to_string(), CompoundId::HardSlick);
    profile.car_compounds.insert("gt_b".to_string(), CompoundId::MonsoonWet);

    assert_eq!(profile.garage_compound("gt", "gt_a"), Some(CompoundId::HardSlick));
    assert_eq!(profile.garage_compound("gt_challenge", "gt_a"), Some(CompoundId::HardSlick));
    assert_eq!(profile.garage_compound("gt", "gt_b"), None, "a wet tyre is not a garage option");
    assert_eq!(profile.garage_compound("gt", "gt_untouched"), None);
    for module in ["nascar", "kart", "rally", "autocross", "extreme_offroad", "classic"] {
        assert_eq!(profile.garage_compound(module, "gt_a"), None, "{module} has no tyre choice");
        assert!(!is_gt_garage_module(module));
    }
    assert_eq!(GT_GARAGE_COMPOUNDS.len(), 3);
}

#[test]
fn cycling_the_players_car_on_the_starting_grid_fits_that_cars_choice() {
    let mut session = gt_session(3);
    session.game_mode = GameMode::ExperimentalRace;
    session.init_race();
    let player_roster_idx = session.grid_participants.iter().position(|p| p.is_player).unwrap();
    session.starting_grid_roster_idx = player_roster_idx;
    let first = session.grid_participants[player_roster_idx].model_id.unwrap();

    // Learn which car comes next, then come back.
    session.cycle_starting_grid_participant_model(true);
    let second = session.grid_participants[player_roster_idx].model_id.unwrap();
    assert_ne!(first, second);
    session.cycle_starting_grid_participant_model(false);

    session.active_profile.car_compounds.insert(second.to_string(), CompoundId::HardSlick);
    session.cycle_starting_grid_participant_model(true);
    assert_eq!(session.selected_car_model_id, Some(second));
    let (config, assemblies) = wheel_compounds(&session.world.vehicles[0]);
    assert_eq!(config, vec![CompoundId::HardSlick; 4]);
    assert_eq!(assemblies, vec![CompoundId::HardSlick; 4]);

    // Back on a car without a choice, the preset returns.
    session.cycle_starting_grid_participant_model(false);
    assert_eq!(session.selected_car_model_id, Some(first));
    assert_eq!(wheel_compounds(&session.world.vehicles[0]).1, vec![preset_compound(first); 4]);
}

// --- Garage side: TYRES row, input, persistence ---

use macroquad::input::KeyCode;
use tdrace_app::catalog::{get_all_models, get_models_for_module, get_models_for_module_and_tier};
use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::{inject_key_presses_for_tests, GameState, GarageOrigin};
use tdrace_app::ui::garage::{
    garage_compound_label, garage_compound_tradeoff, garage_next_compound, garage_select_button_rect,
    garage_tyre_pill_rect, garage_tyres, GarageTyres,
};

/// Runs one Garage frame with `keys` pressed.
fn garage_press(session: &mut RaceSession, origin: GarageOrigin, keys: &[KeyCode]) {
    inject_key_presses_for_tests(keys);
    session.update_garage(origin, 0.016);
    inject_key_presses_for_tests(&[]);
}

/// Opens the GT garage on the tier 1 car at `idx`, unlocked.
fn open_gt_garage(session: &mut RaceSession, origin: GarageOrigin, idx: usize) -> &'static str {
    session.switch_to_gt();
    session.garage_tier = 1;
    session.garage_car_idx = idx;
    let car = get_models_for_module_and_tier("gt", 1)[idx];
    session.active_career_progress.ensure_car(car.id);
    session.state = GameState::Garage(origin);
    car.id
}

fn tyres_of(session: &RaceSession, car_id: &str) -> GarageTyres {
    garage_tyres(find_model_by_id(car_id), &session.active_profile, true)
}

#[test]
fn the_tyres_row_offers_the_three_slicks_to_a_gt_car() {
    let mut session = fresh_session();
    let car = open_gt_garage(&mut session, GarageOrigin::Menu, 0);
    assert_eq!(tyres_of(&session, car), GarageTyres::Select(CompoundId::MediumSlick), "no choice: the preset");

    let labels: Vec<String> = GT_GARAGE_COMPOUNDS.iter().map(|c| garage_compound_label(*c)).collect();
    assert_eq!(labels, ["SOFT", "MEDIUM", "HARD"]);
    assert_eq!(garage_compound_tradeoff(CompoundId::SoftSlick), "More grip, wears faster when you slide.");
    assert_ne!(garage_compound_tradeoff(CompoundId::SoftSlick), garage_compound_tradeoff(CompoundId::MediumSlick));
    assert_ne!(garage_compound_tradeoff(CompoundId::HardSlick), garage_compound_tradeoff(CompoundId::MediumSlick));

    // Left to right: Soft, Medium, Hard. The cycle wraps both ways.
    assert_eq!(garage_next_compound(CompoundId::SoftSlick, true), CompoundId::MediumSlick);
    assert_eq!(garage_next_compound(CompoundId::MediumSlick, true), CompoundId::HardSlick);
    assert_eq!(garage_next_compound(CompoundId::HardSlick, true), CompoundId::SoftSlick);
    assert_eq!(garage_next_compound(CompoundId::SoftSlick, false), CompoundId::HardSlick);
    assert_eq!(garage_next_compound(CompoundId::HardSlick, false), CompoundId::MediumSlick);

    // The three options sit side by side inside the specification card, clear of the select button.
    let (sw, sh) = (1920.0, 1080.0);
    let (bx, by, bw, bh) = garage_select_button_rect(sw, sh);
    let rects: Vec<_> = (0..3).map(|i| garage_tyre_pill_rect(sw, sh, i)).collect();
    for (i, (x, y, w, h)) in rects.iter().enumerate() {
        assert!(*w > 40.0 && *h > 10.0);
        assert!(*x >= bx - 1.0 && x + w <= bx + bw + 1.0, "option {i} inside the right column");
        assert!(y + h < by, "option {i} above the select button ({} vs {})", y + h, by);
        assert!(y + h <= by + bh);
        if i > 0 {
            let (px, _, pw, _) = rects[i - 1];
            assert!(*x > px + pw, "option {i} does not overlap option {}", i - 1);
        }
    }
}

#[test]
fn changing_the_tyres_in_the_garage_and_confirming_races_on_that_compound() {
    let mut session = fresh_session();
    session.game_mode = GameMode::ExperimentalRace;
    session.num_bots = 3;
    session.switch_to_gt();
    session.init_race(); // the Starting Grid, where the garage is opened from
    let car = open_gt_garage(&mut session, GarageOrigin::StartingGrid, 0);

    // MEDIUM -> (T) HARD -> (T) SOFT
    garage_press(&mut session, GarageOrigin::StartingGrid, &[KeyCode::T]);
    assert_eq!(tyres_of(&session, car), GarageTyres::Select(CompoundId::HardSlick));
    garage_press(&mut session, GarageOrigin::StartingGrid, &[KeyCode::T]);
    assert_eq!(tyres_of(&session, car), GarageTyres::Select(CompoundId::SoftSlick));
    assert_eq!(session.active_profile.car_compounds.get(car), Some(&CompoundId::SoftSlick));

    garage_press(&mut session, GarageOrigin::StartingGrid, &[KeyCode::Enter]);
    assert_eq!(session.state, GameState::StartingGrid, "the car is confirmed");
    assert_eq!(session.selected_car_model_id, Some(car));

    let player = &session.world.vehicles[session.player_car_index()];
    let (config, assemblies) = wheel_compounds(player);
    assert_eq!(config, vec![CompoundId::SoftSlick; 4]);
    assert_eq!(assemblies, vec![CompoundId::SoftSlick; 4]);
    // The cockpit badge and wheel borders read the assemblies (spec 089).
    assert_eq!(player.state.wheel_assemblies[0].config.compound.id.badge_code(), "S");
    for (idx, bot) in session.world.vehicles.iter().enumerate().skip(1) {
        let model_id = session.car_model_ids[idx].expect("bot catalog model");
        assert_eq!(wheel_compounds(bot).0, vec![preset_compound(model_id); 4]);
    }
}

#[test]
fn a_choice_is_kept_per_car_across_a_restart() {
    let dir = std::env::temp_dir().join(format!("tdrace_spec098_garage_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("records.db");

    let (first, second);
    {
        let mut session = fresh_session();
        session.hof_db = Some(HallOfFameDb::open(&path).unwrap());
        session.refresh_profiles_and_stats();
        assert!(session.active_profile.id.is_some());
        first = open_gt_garage(&mut session, GarageOrigin::Menu, 0);
        second = get_models_for_module_and_tier("gt", 1)[1].id;
        assert_ne!(first, second);
        // HARD for the first car (MEDIUM -> T), the second one untouched.
        garage_press(&mut session, GarageOrigin::Menu, &[KeyCode::T]);
        assert_eq!(tyres_of(&session, first), GarageTyres::Select(CompoundId::HardSlick));
    }

    // Game restart: a new session on the same database file.
    let mut restarted = fresh_session();
    restarted.hof_db = Some(HallOfFameDb::open(&path).unwrap());
    restarted.refresh_profiles_and_stats();
    assert_eq!(tyres_of(&restarted, first), GarageTyres::Select(CompoundId::HardSlick), "the first car shows HARD");
    assert_eq!(tyres_of(&restarted, second), GarageTyres::Select(CompoundId::MediumSlick), "the second car shows MEDIUM");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn stock_cars_show_the_series_compound_and_offer_no_selector() {
    let nascar: Vec<_> = get_all_models().iter().filter(|m| m.module_id == "nascar").collect();
    assert!(!nascar.is_empty());
    for model in &nascar {
        let profile = tdrace_app::profile::PlayerProfile::default();
        assert_eq!(
            garage_tyres(Some(model), &profile, true),
            GarageTyres::ReadOnly { compound: preset_compound(model.id), note: "series rule" },
            "{}",
            model.id
        );
    }
    assert_eq!(preset_compound(nascar[0].id), CompoundId::HardSlick);

    // T does nothing on a stock car.
    let mut session = fresh_session();
    session.switch_to_nascar();
    session.garage_tier = 1;
    session.garage_car_idx = 0;
    session.state = GameState::Garage(GarageOrigin::Menu);
    garage_press(&mut session, GarageOrigin::Menu, &[KeyCode::T]);
    assert!(session.active_profile.car_compounds.is_empty());
}

#[test]
fn other_categories_have_no_tyres_row_and_keep_their_preset() {
    for module in ["kart", "rally", "autocross", "extreme_offroad", "classic", "vault"] {
        let models = get_models_for_module(module);
        assert!(!models.is_empty(), "{module} has cars");
        for model in models {
            let profile = tdrace_app::profile::PlayerProfile::default();
            assert_eq!(garage_tyres(Some(model), &profile, true), GarageTyres::Hidden, "{}", model.id);
        }
    }

    // T does nothing there, even on a car the profile (wrongly) has an entry for.
    let mut session = fresh_session();
    session.switch_to_kart();
    session.garage_tier = 1;
    session.garage_car_idx = 0;
    session.state = GameState::Garage(GarageOrigin::Menu);
    garage_press(&mut session, GarageOrigin::Menu, &[KeyCode::T]);
    assert!(session.active_profile.car_compounds.is_empty());

    let kart = get_models_for_module_and_tier("kart", 1)[0].id;
    session.active_profile.car_compounds.insert(kart.to_string(), CompoundId::HardSlick);
    garage_press(&mut session, GarageOrigin::Menu, &[KeyCode::T]);
    assert_eq!(session.active_profile.car_compounds.get(kart), Some(&CompoundId::HardSlick));
    assert_eq!(session.active_profile.car_compounds.len(), 1);
}

#[test]
fn a_locked_vehicle_shows_its_compound_read_only() {
    let mut session = fresh_session();
    session.game_mode = GameMode::StandardRace; // Quick Race locks the car at the Starting Grid
    session.switch_to_gt();
    session.init_race();
    let car = open_gt_garage(&mut session, GarageOrigin::StartingGrid, 0);
    assert_eq!(
        garage_tyres(find_model_by_id(car), &session.active_profile, false),
        GarageTyres::ReadOnly { compound: CompoundId::MediumSlick, note: "vehicle locked" }
    );
    garage_press(&mut session, GarageOrigin::StartingGrid, &[KeyCode::T]);
    assert!(session.active_profile.car_compounds.is_empty(), "no change while the vehicle is locked");
}

fn bind_lan_host(car: &str) -> cabinet::net::LanHost {
    cabinet::net::LanHost::bind(
        "Compound Test Room",
        "HostDriver",
        "ESP",
        car,
        "corsa_red",
        0, // ephemeral port
        4,
        "classic_grand_prix",
        "classic",
        3,
    )
    .expect("failed to bind host")
}

#[test]
fn a_lan_owner_fits_its_own_compound_on_its_own_car() {
    let car = "gt_valente_corsa_t2";
    let mut session = fresh_session();
    session.active_profile.car_compounds.insert(car.to_string(), CompoundId::HardSlick);
    session.launch_lan_race_session(Some(bind_lan_host(car)), None, 0);

    assert!(session.is_lan_multiplayer);
    assert_eq!(session.car_model_ids[0], Some(car));
    let (config, assemblies) = wheel_compounds(&session.world.vehicles[0]);
    assert_eq!(config, vec![CompoundId::HardSlick; 4]);
    assert_eq!(assemblies, vec![CompoundId::HardSlick; 4]);
    session.exit_lan_session();

    // Without a choice the LAN car keeps its preset.
    let mut plain = fresh_session();
    plain.launch_lan_race_session(Some(bind_lan_host(car)), None, 0);
    assert_eq!(wheel_compounds(&plain.world.vehicles[0]).1, vec![preset_compound(car); 4]);
    plain.exit_lan_session();
}
