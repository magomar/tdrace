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

fn gt_session(num_bots: usize) -> RaceSession {
    let mut session = RaceSession::new();
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
    let mut session = RaceSession::new();
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
    let mut session = RaceSession::new();
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
    let mut session = RaceSession::new();
    session.active_profile.car_compounds.insert(car.to_string(), CompoundId::HardSlick);
    session.launch_lan_race_session(Some(bind_lan_host(car)), None, 0);

    assert!(session.is_lan_multiplayer);
    assert_eq!(session.car_model_ids[0], Some(car));
    let (config, assemblies) = wheel_compounds(&session.world.vehicles[0]);
    assert_eq!(config, vec![CompoundId::HardSlick; 4]);
    assert_eq!(assemblies, vec![CompoundId::HardSlick; 4]);
    session.exit_lan_session();

    // Without a choice the LAN car keeps its preset.
    let mut plain = RaceSession::new();
    plain.launch_lan_race_session(Some(bind_lan_host(car)), None, 0);
    assert_eq!(wheel_compounds(&plain.world.vehicles[0]).1, vec![preset_compound(car); 4]);
    plain.exit_lan_session();
}
