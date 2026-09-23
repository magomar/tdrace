use std::fs;
use tdrace_app::series::manager::EMBEDDED_PRESETS;
use tdrace_app::ui::menu::{ModalityCategory, ModalityItem};
use tdrace_app::{
    ChampionshipDefinition, ChampionshipManager, GameState, PointSystem, RaceSession,
    SeriesDefinition, SeriesManager, SeriesSession,
};

const SAMPLE_CHAMPIONSHIP_TOML: &str = r#"
[championship]
id = "gt3_apex_masters"
name = "GT3 Apex Masters"
description = "High speed sprint cup for GT3 class machines"
module = "gt"
tier = 3
laps_per_round = 5
bot_count = 7
ai_difficulty = "pro"

[scoring]
system = "fia"
fastest_lap_bonus = true
clean_race_bonus = false

[[rounds]]
order = 1
track_id = "spa_francorchamps"
name = "Belgian Grand Prix"
laps = 6

[[rounds]]
order = 2
track_id = "monza"
name = "Italian Speed Ring"

[[rounds]]
order = 3
track_id = "suzuka"
name = "Japanese Technical Challenge"

[[drivers]]
id = "player"
name = "Apex Predator"
team = "Scuderia Neon"
is_player = true
car_model_id = "gt_ferrari_296_gt3"
country = "ITA"

[[drivers]]
id = "bot_1"
name = "Marcus Steel"
team = "Apex Racing"
car_model_id = "gt_porsche_911_gt3_r"
country = "GER"

[[drivers]]
id = "bot_2"
name = "Lucas Moreau"
team = "Alpine Velocity"
car_model_id = "gt_bmw_m4_gt3"
country = "FRA"
"#;

#[test]
fn test_toml_deserialization_and_validation() {
    let def = ChampionshipDefinition::from_toml(SAMPLE_CHAMPIONSHIP_TOML)
        .expect("Failed to parse valid championship TOML");

    assert_eq!(def.series.id, "gt3_apex_masters");
    assert_eq!(def.series.name, "GT3 Apex Masters");
    assert_eq!(def.series.module_id, "gt");
    assert_eq!(def.series.tier, 3);
    assert_eq!(def.series.laps_per_round, 5);
    assert_eq!(def.series.bot_count, Some(7));
    assert_eq!(def.series.ai_difficulty.as_deref(), Some("pro"));

    assert_eq!(def.scoring.system, "fia");
    assert!(def.scoring.fastest_lap_bonus);
    assert!(!def.scoring.clean_race_bonus);

    assert_eq!(def.rounds.len(), 3);
    assert_eq!(def.rounds[0].order, 1);
    assert_eq!(def.rounds[0].track_id, "spa_francorchamps");
    assert_eq!(def.rounds[0].laps, Some(6));
    assert_eq!(def.rounds[1].order, 2);
    assert_eq!(def.rounds[1].track_id, "monza");
    assert_eq!(def.rounds[1].laps, None);

    assert_eq!(def.drivers.len(), 3);
    assert!(def.drivers[0].is_player);
    assert_eq!(def.drivers[0].id, "player");
    assert_eq!(def.drivers[0].name, "Apex Predator");
    assert_eq!(def.drivers[0].car_model_id.as_deref(), Some("gt_ferrari_296_gt3"));
    assert_eq!(def.drivers[0].country.as_deref(), Some("ITA"));

    assert!(!def.drivers[1].is_player);
    assert_eq!(def.drivers[1].id, "bot_1");

    // Structural validation must succeed
    assert!(def.validate().is_ok());
}

#[test]
fn test_toml_roundtrip_fidelity() {
    let def = ChampionshipDefinition::from_toml(SAMPLE_CHAMPIONSHIP_TOML)
        .expect("Failed to parse original TOML");

    let serialized = def.to_toml().expect("Failed to serialize definition to TOML");
    let def_roundtrip = ChampionshipDefinition::from_toml(&serialized)
        .expect("Failed to parse serialized TOML");

    assert_eq!(def, def_roundtrip);
}

#[test]
fn test_validation_rejects_invalid() {
    let mut def = ChampionshipDefinition::from_toml(SAMPLE_CHAMPIONSHIP_TOML)
        .expect("Failed to parse base definition");

    // 1. Empty ID
    def.series.id = "".to_string();
    assert!(def.validate().is_err());
    def.series.id = "valid_id".to_string();

    // 2. Empty Name
    def.series.name = "".to_string();
    assert!(def.validate().is_err());
    def.series.name = "Valid Name".to_string();

    // 3. Zero laps
    def.series.laps_per_round = 0;
    assert!(def.validate().is_err());
    def.series.laps_per_round = 3;

    // 4. Empty rounds
    let saved_rounds = std::mem::take(&mut def.rounds);
    assert!(def.validate().is_err());
    def.rounds = saved_rounds;

    // 5. Empty drivers
    let saved_drivers = std::mem::take(&mut def.drivers);
    assert!(def.validate().is_err());
    def.drivers = saved_drivers;

    // 6. Zero player drivers
    for d in &mut def.drivers {
        d.is_player = false;
    }
    assert!(def.validate().is_err());

    // 7. Multiple player drivers
    for d in &mut def.drivers {
        d.is_player = true;
    }
    assert!(def.validate().is_err());

    // Restore single player
    def.drivers[0].is_player = true;
    for d in def.drivers.iter_mut().skip(1) {
        d.is_player = false;
    }
    assert!(def.validate().is_ok());

    // 8. Duplicate driver IDs
    def.drivers[1].id = def.drivers[0].id.clone();
    assert!(def.validate().is_err());
    def.drivers[1].id = "bot_unique".to_string();

    // 9. Duplicate round orders
    def.rounds[1].order = def.rounds[0].order;
    assert!(def.validate().is_err());
}

#[test]
fn test_to_session_runtime_bridging() {
    let def = ChampionshipDefinition::from_toml(SAMPLE_CHAMPIONSHIP_TOML)
        .expect("Failed to parse definition");

    let session = def.to_session();
    assert_eq!(session.name, "GT3 Apex Masters");
    assert_eq!(session.laps_per_round, 5);
    assert_eq!(session.total_rounds(), 3);
    assert_eq!(session.current_round, 0);
    assert_eq!(session.current_track_id(), Some("spa_francorchamps"));
    assert_eq!(session.standings.len(), 3);

    // Verify point system mapped correctly
    assert!(matches!(session.point_system, PointSystem::FiaStandard { .. }));
    assert_eq!(session.point_system.points_for_position(1, true), 26); // 25 + 1 fastest lap bonus

    // Verify recovery from runtime session
    let recovered = ChampionshipDefinition::from_session(&session, "gt", 3);
    assert_eq!(recovered.series.name, "GT3 Apex Masters");
    assert_eq!(recovered.series.module_id, "gt");
    assert_eq!(recovered.series.tier, 3);
    assert_eq!(recovered.rounds.len(), 3);
    assert_eq!(recovered.rounds[0].track_id, "spa_francorchamps");
    assert_eq!(recovered.drivers.len(), 3);
}

#[test]
fn test_championship_manager_discovery_and_saving() {
    let temp_base = std::env::temp_dir().join(format!(
        "tdrace_test_champ_mgr_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let user_dir = temp_base.join("user_championships");
    let git_dir = temp_base.join("git_championships");
    let _ = fs::create_dir_all(&user_dir);
    let _ = fs::create_dir_all(&git_dir);

    let mut mgr = SeriesManager::with_dirs(user_dir.clone(), Some(git_dir.clone()));

    // 1. Embedded presets are present
    assert!(!mgr.series.is_empty());
    assert!(mgr.get("gt4_clubman_sprint").is_some());
    assert!(mgr.get("nascar_cup_tier5").is_some());
    assert!(mgr.get("rally_world_cup").is_some());
    assert!(mgr.get("kart_world_cup").is_some());
    assert!(mgr.get("extreme_offroad_cup").is_some());

    // 2. Save new user championship
    let mut custom_def = ChampionshipDefinition::from_toml(SAMPLE_CHAMPIONSHIP_TOML).unwrap();
    custom_def.series.id = "my_custom_user_cup".to_string();
    custom_def.series.name = "My Custom User Cup".to_string();

    let saved_path = mgr
        .save_user_championship(&custom_def)
        .expect("Save user championship");
    assert!(saved_path.exists());
    assert!(mgr.get("my_custom_user_cup").is_some());

    // 3. Rescan discovers the saved user cup
    mgr.scan_all();
    assert!(mgr.get("my_custom_user_cup").is_some());
    assert_eq!(
        mgr.get("my_custom_user_cup").unwrap().series.name,
        "My Custom User Cup"
    );

    // 4. Delete user championship
    let deleted = mgr
        .delete_user_championship("my_custom_user_cup")
        .expect("Delete user championship");
    assert!(deleted);
    assert!(!saved_path.exists());
    assert!(mgr.get("my_custom_user_cup").is_none());

    // Clean up temporary directory
    let _ = fs::remove_dir_all(&temp_base);
}

#[test]
fn test_all_embedded_presets_are_valid() {
    assert!(
        !EMBEDDED_PRESETS.is_empty(),
        "Embedded presets list must not be empty"
    );

    for (slug, toml_str) in EMBEDDED_PRESETS {
        let def = ChampionshipDefinition::from_toml(toml_str)
            .unwrap_or_else(|e| panic!("Embedded preset '{}' failed to parse: {}", slug, e));

        assert_eq!(
            &def.series.id, slug,
            "Embedded preset slug must match championship ID"
        );

        def.validate().unwrap_or_else(|errs| {
            panic!(
                "Embedded preset '{}' failed validation: {:?}",
                slug, errs
            )
        });

        let session = def.to_session();
        assert!(
            session.total_rounds() > 0,
            "Embedded preset '{}' must have at least 1 round",
            slug
        );
        assert!(
            !session.standings.is_empty(),
            "Embedded preset '{}' must have at least 1 driver",
            slug
        );
    }
}

#[test]
fn test_modality_select_championship_editor_option() {
    let options_items = ModalityCategory::Options.items();
    assert!(
        options_items.contains(&ModalityItem::SeriesEditor),
        "ModalityCategory::Options must contain SeriesEditor"
    );

    assert_eq!(
        ModalityItem::SeriesEditor.title(),
        "Series Editor"
    );
    assert_eq!(
        ModalityItem::SeriesEditor.tag(),
        "CUSTOM SERIES & CUPS • TOML WORKBENCH"
    );
    assert!(ModalityItem::SeriesEditor.is_available());
}

#[test]
fn test_game_session_championship_editor_integration() {
    let mut session = RaceSession::new();

    // Verify championship manager is initialized
    assert!(!session.championship_manager.series.is_empty());
    assert!(session.championship_editor_state.is_none());

    // Enter Championship Editor
    session.enter_championship_editor(None);
    assert_eq!(session.state, GameState::ChampionshipEditor);
    assert!(session.championship_editor_state.is_some());

    // Verify initial pre-existing championship definition in editor (defaults to active module Tier 1)
    let state = session.championship_editor_state.as_ref().unwrap();
    assert_eq!(state.def.series.module_id, "gt");
    assert_eq!(state.def.series.id, "gt4_clubman_sprint");
    assert_eq!(state.def.rounds.len(), 5);
    assert!(!state.def.drivers.is_empty());

    // Test LaunchTestCup action
    let test_def = ChampionshipDefinition::from_toml(SAMPLE_CHAMPIONSHIP_TOML).unwrap();
    session.launch_championship_test_cup(test_def);

    assert!(session.championship_editor_state.is_none());
    assert!(session.championship_session.is_some());
    let active_champ = session.championship_session.as_ref().unwrap();
    assert_eq!(active_champ.name, "GT3 Apex Masters");
    assert_eq!(session.active_module_id, "gt");
    assert_eq!(session.state, GameState::StartingGrid);
}

#[test]
fn test_gt_tiers_1_to_5_specifications() {
    let mgr = ChampionshipManager::new();

    let gt_champs: Vec<_> = mgr
        .all_sorted()
        .into_iter()
        .filter(|c| c.series.module_id == "gt")
        .collect();

    assert_eq!(gt_champs.len(), 5, "All 5 GT tier championships must be registered");

    // Expected tier metadata: (tier, id, min_rounds, starter_car)
    let expected_tiers = [
        (1, "gt4_clubman_sprint", 5, "gt_toyota_supra_gt4"),
        (2, "gt3_european_challenge", 7, "gt_porsche_911_gt3r"),
        (3, "gt2_power_masters", 9, "gt_porsche_911_gt2_rs"),
        (4, "gt1_heritage_trophy", 10, "gt_porsche_911_gt1_98"),
        (5, "hypercar_world_gp", 12, "gt_ferrari_499p"),
    ];

    for (tier, id, rounds_len, expected_car) in expected_tiers {
        let def = mgr.get(id).unwrap_or_else(|| panic!("Missing preset {}", id));
        assert_eq!(def.series.tier, tier);
        assert_eq!(def.rounds.len(), rounds_len);
        assert_eq!(def.drivers.len(), 8);
        assert_eq!(def.scoring.system, "fia");

        // Verify player driver
        let player = def.drivers.iter().find(|d| d.is_player).expect("Player driver required");
        assert_eq!(player.car_model_id.as_deref(), Some(expected_car));

        // Verify all car models in the grid exist in vehicle catalog
        for driver in &def.drivers {
            let model_id = driver.car_model_id.as_deref().expect("Driver car model required");
            let model = tdrace_app::catalog::find_model_by_id(model_id);
            assert!(model.is_some(), "Car model '{}' in '{}' must exist in ALL_REAL_CARS", model_id, id);
        }

        // Verify runtime conversion
        let session = def.to_session();
        assert_eq!(session.total_rounds(), rounds_len);
    }
}

#[test]
fn test_all_motorsport_modules_tiers_1_to_5_specifications() {
    let mgr = ChampionshipManager::new();

    let modules_and_expected = [
        (
            "gt",
            vec![
                (1, "gt4_clubman_sprint", 5, "gt_toyota_supra_gt4"),
                (2, "gt3_european_challenge", 7, "gt_porsche_911_gt3r"),
                (3, "gt2_power_masters", 9, "gt_porsche_911_gt2_rs"),
                (4, "gt1_heritage_trophy", 10, "gt_porsche_911_gt1_98"),
                (5, "hypercar_world_gp", 12, "gt_ferrari_499p"),
            ],
        ),
        (
            "nascar",
            vec![
                (1, "nascar_short_track_series", 5, "nascar_monte_carlo_ss"),
                (2, "nascar_intermediate_oval_challenge", 7, "nascar_super_late_model"),
                (3, "nascar_national_tour", 9, "nascar_arca_chevy_ss"),
                (4, "nascar_premier_speedway_trophy", 10, "nascar_silverado_truck"),
                (5, "nascar_cup_tier5", 12, "nascar_corvette_ta1"),
            ],
        ),
        (
            "rally",
            vec![
                (1, "rally_grassroots_cup", 5, "rally_peugeot_208_rally4"),
                (2, "rally_world_cup", 7, "rally_polo_rx"),
                (3, "rally_group_b_masters", 9, "rally_audi_sport_quattro_s1"),
                (4, "rally_dakar_raid_trophy", 10, "rally_toyota_hilux_t1_plus"),
                (5, "rally_super_trucks_series", 12, "rally_sst_super_truck"),
            ],
        ),
        (
            "kart",
            vec![
                (1, "kart_world_cup", 5, "kart_crg_hero_60"),
                (2, "kart_national_championship", 7, "kart_tony_kart_racer_ok"),
                (3, "kart_continental_trophy", 9, "kart_birel_art_kz2"),
                (4, "kart_european_championship", 10, "kart_honda_mean_mower"),
                (5, "kart_superkart_world_series", 12, "kart_anderson_cs250"),
            ],
        ),
        (
            "extreme_offroad",
            vec![
                (1, "extreme_desert_sand_sprint", 5, "offroad_sand_rail_buggy"),
                (2, "extreme_canyon_raid", 7, "offroad_baja_trophy_truck"),
                (3, "extreme_offroad_cup", 9, "offroad_subaru_ice_racer"),
                (4, "extreme_mud_masters", 10, "offroad_mega_mud_truck"),
                (5, "extreme_ultimate_championship", 12, "offroad_grave_crusher"),
            ],
        ),
    ];

    for (module_id, expected_tiers) in modules_and_expected {
        let module_champs: Vec<_> = mgr
            .all_sorted()
            .into_iter()
            .filter(|c| c.series.module_id == module_id)
            .collect();

        assert_eq!(
            module_champs.len(),
            5,
            "Module '{}' must have exactly 5 tier championships registered",
            module_id
        );

        for (tier, id, rounds_len, expected_car) in expected_tiers {
            let def = mgr
                .get(id)
                .unwrap_or_else(|| panic!("Module '{}' missing preset '{}'", module_id, id));

            assert_eq!(def.series.module_id, module_id);
            assert_eq!(def.series.tier, tier);
            assert_eq!(
                def.rounds.len(),
                rounds_len,
                "Championship '{}' in module '{}' tier {} must have {} rounds",
                id,
                module_id,
                tier,
                rounds_len
            );
            assert_eq!(
                def.drivers.len(),
                8,
                "Championship '{}' must have exactly 8 drivers",
                id
            );

            // Verify player driver exists with valid starter car
            let player = def
                .drivers
                .iter()
                .find(|d| d.is_player)
                .unwrap_or_else(|| panic!("Championship '{}' must have a player driver", id));
            assert_eq!(
                player.car_model_id.as_deref(),
                Some(expected_car),
                "Player car in '{}' tier {} must be {}",
                id,
                tier,
                expected_car
            );

            // Verify all driver car models exist in vehicle catalog
            for driver in &def.drivers {
                let model_id = driver
                    .car_model_id
                    .as_deref()
                    .unwrap_or_else(|| panic!("Driver '{}' in '{}' missing car model", driver.id, id));
                let model = tdrace_app::catalog::find_model_by_id(model_id);
                assert!(
                    model.is_some(),
                    "Car model '{}' in '{}' must exist in ALL_REAL_CARS",
                    model_id,
                    id
                );
            }

            // Verify round ordering is strictly 1..=rounds_len
            for (idx, round) in def.rounds.iter().enumerate() {
                assert_eq!(
                    round.order,
                    idx + 1,
                    "Round order mismatch in '{}'",
                    id
                );
            }

            // Verify runtime conversion
            let session = def.to_session();
            assert_eq!(session.total_rounds(), rounds_len);
            assert_eq!(session.tier, tier);
        }
    }
}

#[test]
fn test_championship_editor_open_and_load_modal() {
    use tdrace_app::ui::championship_editor::{
        ChampionshipEditorAction, ChampionshipEditorModal, ChampionshipEditorState,
    };

    let mgr = ChampionshipManager::new();
    let all_champs = mgr.all_sorted();

    let mut state = ChampionshipEditorState::new(None);
    state.modal = ChampionshipEditorModal::OpenChampionship { selected_idx: 1 };

    if let ChampionshipEditorModal::OpenChampionship { selected_idx } = state.modal {
        let selected_cup = all_champs[selected_idx];
        let action = ChampionshipEditorAction::LoadChampionship((*selected_cup).clone());

        if let ChampionshipEditorAction::LoadChampionship(loaded) = action {
            state.def = loaded;
            state.modal = ChampionshipEditorModal::None;
        }
    }

    assert_eq!(state.modal, ChampionshipEditorModal::None);
    assert!(!state.def.rounds.is_empty());
}

#[test]
fn test_series_toml_syntax_and_session() {
    let toml_content = r#"
[series]
id = "custom_supercar_challenge"
name = "Custom Supercar Challenge"
description = "A 4-round custom championship cup"
module_id = "gt"
tier = 2
laps_per_round = 4

[scoring]
system = "fia"
fastest_lap_bonus = true

[[rounds]]
order = 1
track_id = "spa_francorchamps"

[[rounds]]
order = 2
track_id = "monza"

[[drivers]]
id = "player"
name = "Player 1"
team = "Apex"
is_player = true

[[drivers]]
id = "bot_1"
name = "Bot Rival"
team = "Rivalry"
is_player = false
"#;

    let def = SeriesDefinition::from_toml(toml_content).expect("Must parse [series] header");
    assert_eq!(def.series.id, "custom_supercar_challenge");
    assert_eq!(def.series.name, "Custom Supercar Challenge");
    assert_eq!(def.series.tier, 2);
    assert_eq!(def.championship().id, "custom_supercar_challenge");
    assert_eq!(def.cup().name, "Custom Supercar Challenge");
    assert_eq!(def.rounds.len(), 2);
    assert!(def.validate().is_ok());

    let session: SeriesSession = def.to_session();
    assert_eq!(session.name, "Custom Supercar Challenge");
    assert_eq!(session.total_rounds(), 2);
    assert_eq!(session.tier, 2);
}

#[test]
fn test_rally_championship_points_awarded_to_all_drivers_and_persisted_across_rounds() {
    let mut session = RaceSession::new();
    session.start_rally_career_tier(1);

    // 1. Verify championship session initialization
    assert!(session.championship_session.is_some());
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.standings.len(), 8);
    let standing_ids: Vec<String> = champ.standings.iter().map(|s| s.driver_id.clone()).collect();
    assert!(standing_ids.contains(&"player".to_string()));
    assert!(standing_ids.contains(&"johan_vance".to_string()));
    assert!(standing_ids.contains(&"mattias_storm".to_string()));

    // 2. Init race for Round 1
    session.init_race();
    assert_eq!(session.cars.len(), 8);
    assert_eq!(session.opponent_drivers.len(), 7);
    for opp in &session.opponent_drivers {
        assert!(
            standing_ids.contains(&opp.id.to_string()),
            "Opponent driver '{}' must exist in championship standings",
            opp.id
        );
    }

    // 3. Simulate race finish where:
    // Car 1 (Johan Vance) finishes 1st (with fastest lap)
    // Car 2 (Mattias Storm) finishes 2nd
    // Car 0 (Player) finishes 3rd
    // Cars 3..7 finish 4th..8th
    for (idx, tracker) in session.trackers.iter_mut().enumerate() {
        tracker.current_lap = session.total_laps + 1;
        tracker.normalized_progress = match idx {
            1 => 0.99, // 1st
            2 => 0.90, // 2nd
            0 => 0.80, // 3rd (Player)
            3 => 0.70, // 4th
            4 => 0.60, // 5th
            5 => 0.50, // 6th
            6 => 0.40, // 7th
            7 => 0.30, // 8th
            _ => 0.10,
        };
        tracker.best_lap_time = Some(40.0 + idx as f32);
    }
    session.trackers[1].best_lap_time = Some(38.5);

    session.check_race_finish();
    assert_eq!(session.state, GameState::ChampionshipStandings);

    // 4. Verify Round 1 standings
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.current_round, 1);

    // Player finished 3rd -> 15 points (FIA standard)
    let player_standing = champ.standings.iter().find(|s| s.driver_id == "player").unwrap();
    assert_eq!(player_standing.points, 15, "Player finishing 3rd must receive 15 FIA points");
    assert_eq!(player_standing.podiums, 1);

    // Other AI drivers MUST have points awarded, not 0!
    for standing in &champ.standings {
        assert!(
            standing.points > 0,
            "Driver '{}' finished the race and must receive points (got 0)",
            standing.driver_name
        );
    }

    // Car 1 driver was 1st with fastest lap: 25 + 1 = 26 points
    let car1_driver_id = session.opponent_drivers[0].id;
    let winner_standing = champ.standings.iter().find(|s| s.driver_id == car1_driver_id).unwrap();
    assert_eq!(winner_standing.points, 26, "1st place with fastest lap gets 25 + 1 = 26 points");
    assert_eq!(winner_standing.wins, 1);

    // 5. Advance to Round 2
    session.advance_championship_round();
    assert_eq!(session.opponent_drivers.len(), 7);
    for opp in &session.opponent_drivers {
        assert!(
            standing_ids.contains(&opp.id.to_string()),
            "Opponent driver '{}' in Round 2 must still match championship roster",
            opp.id
        );
    }

    // 6. Simulate Round 2 finish: Player wins (1st with fastest lap), Car 1 finishes 2nd
    for (idx, tracker) in session.trackers.iter_mut().enumerate() {
        tracker.current_lap = session.total_laps + 1;
        tracker.normalized_progress = match idx {
            0 => 0.99, // 1st (Player)
            1 => 0.90, // 2nd
            2 => 0.80, // 3rd
            3 => 0.70, // 4th
            4 => 0.60, // 5th
            5 => 0.50, // 6th
            6 => 0.40, // 7th
            7 => 0.30, // 8th
            _ => 0.10,
        };
        tracker.best_lap_time = Some(42.0);
    }
    session.trackers[0].best_lap_time = Some(39.0);

    session.check_race_finish();
    assert_eq!(session.state, GameState::ChampionshipStandings);

    // 7. Verify accumulated points after Round 2
    let champ = session.championship_session.as_ref().unwrap();
    assert_eq!(champ.current_round, 2);

    let player_after_r2 = champ.standings.iter().find(|s| s.driver_id == "player").unwrap();
    // 15 from R1 + (25 + 1) from R2 = 41 points
    assert_eq!(player_after_r2.points, 41, "Player points must accumulate across rounds");
    assert_eq!(player_after_r2.wins, 1);
    assert_eq!(player_after_r2.podiums, 2);

    let winner_after_r2 = champ.standings.iter().find(|s| s.driver_id == car1_driver_id).unwrap();
    // 26 from R1 + 18 from R2 = 44 points
    assert_eq!(winner_after_r2.points, 44, "AI driver points must accumulate across rounds");
}

#[test]
fn test_kart_and_gt_championship_rosters_match_modules() {
    let mut session = RaceSession::new();
    session.start_kart_career_tier(1);
    session.init_race();
    assert_eq!(session.opponent_drivers.len(), 7);
    let kart_champ = session.championship_session.as_ref().unwrap();
    for opp in &session.opponent_drivers {
        assert!(
            kart_champ.standings.iter().any(|s| s.driver_id == opp.id),
            "Kart driver '{}' must match championship standings",
            opp.id
        );
    }

    session.start_gt_championship();
    session.init_race();
    assert_eq!(session.opponent_drivers.len(), 7);
    let gt_champ = session.championship_session.as_ref().unwrap();
    for opp in &session.opponent_drivers {
        assert!(
            gt_champ.standings.iter().any(|s| s.driver_id == opp.id),
            "GT driver '{}' must match championship standings",
            opp.id
        );
    }
}

#[test]
fn test_reset_championship_clears_session_and_database_history() {
    let mut session = RaceSession::new();
    let mem_db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
    let _ = mem_db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    session.start_rally_career_tier(1);
    session.init_race();
    assert!(session.championship_session.is_some());

    // Complete round 1
    session.trackers[0].current_lap = session.total_laps + 1;
    session.check_race_finish();
    assert_eq!(session.state, GameState::ChampionshipStandings);

    // Verify race history has logged this championship round
    let history = session.hof_db.as_ref().unwrap().get_history_for_profile(1, 10).unwrap();
    assert!(
        history.iter().any(|r| r.championship_name.as_deref() == Some("Rallycross Grassroots Cup (Tier 1)")),
        "Race history must have logged championship round"
    );

    // Now reset the championship
    session.reset_championship("Rallycross Grassroots Cup (Tier 1)", "rallycross_grassroots_cup");
    assert!(session.championship_session.is_none(), "Active championship session must be cleared");

    // Verify database history for this championship has been wiped
    let history_after = session.hof_db.as_ref().unwrap().get_history_for_profile(1, 10).unwrap();
    assert!(
        history_after.iter().all(|r| r.championship_name.as_deref() != Some("Rallycross Grassroots Cup (Tier 1)")),
        "Race history for this championship must be cleared"
    );

    // Restart championship and confirm it begins at Round 0 with clean 0 points
    session.start_rally_career_tier(1);
    let fresh_champ = session.championship_session.as_ref().unwrap();
    assert_eq!(fresh_champ.current_round, 0);
    for s in &fresh_champ.standings {
        assert_eq!(s.points, 0, "All drivers must have 0 points on fresh restart");
    }
}

#[test]
fn test_rally_tier_1_championship_starting_grid_eligibility() {
    let mut session = RaceSession::new();
    let mem_db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
    let _ = mem_db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    let mgr = tdrace_app::series::ChampionshipManager::new();
    let def = mgr.get("rally_grassroots_cup").expect("Rally Grassroots Cup preset must exist");

    session.launch_or_resume_championship(&def);

    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.selected_car_model_id, Some("rally_peugeot_208_rally4"));
    assert_eq!(session.active_player_car_tier(), 1, "Peugeot 208 Rally 4 must be Tier 1");
    let req_tier = session.current_race_required_tier();
    assert_eq!(req_tier, 1, "Rally Grassroots Cup requires Tier 1");
    assert!(
        session.is_active_player_car_eligible(req_tier),
        "Peugeot 208 Rally 4 must be eligible for Tier 1 Rally Championship"
    );
    assert!(
        session.is_active_player_car_unlocked(),
        "Peugeot 208 Rally 4 must be unlocked as Tier 1 starter"
    );

    // Verify StartingGrid launch condition succeeds
    let launch_allowed = session.is_active_player_car_eligible(req_tier) && session.is_active_player_car_unlocked();
    assert!(launch_allowed, "Launch must be allowed for Peugeot 208 Rally 4 in StartingGrid");
}

#[test]
fn test_all_modules_tier_1_championship_starters_are_eligible_and_unlocked() {
    let presets = [
        ("gt", "gt4_clubman_sprint", 1),
        ("rally", "rally_grassroots_cup", 1),
        ("nascar", "nascar_short_track_series", 1),
        ("kart", "kart_world_cup", 1),
        ("extreme_offroad", "extreme_desert_sand_sprint", 1),
    ];

    let mgr = tdrace_app::series::ChampionshipManager::new();

    for (mod_id, series_id, expected_tier) in presets {
        let mut session = RaceSession::new();
        let mem_db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
        let _ = mem_db.seed_default_profile_if_empty().unwrap();
        session.hof_db = Some(mem_db);
        session.refresh_profiles_and_stats();

        let def = mgr.get(series_id).unwrap_or_else(|| panic!("Preset '{}' must exist", series_id));
        session.launch_or_resume_championship(&def);

        assert_eq!(session.state, GameState::StartingGrid, "Module {} must transition to StartingGrid", mod_id);
        let req_tier = session.current_race_required_tier();
        assert_eq!(req_tier, expected_tier, "Module {} required tier mismatch", mod_id);
        assert_eq!(
            session.active_player_car_tier(),
            expected_tier,
            "Module {} starter vehicle tier mismatch",
            mod_id
        );
        assert!(
            session.is_active_player_car_eligible(req_tier),
            "Module {} starter vehicle must be eligible for Tier {}",
            mod_id,
            req_tier
        );
        assert!(
            session.is_active_player_car_unlocked(),
            "Module {} starter vehicle must be unlocked",
            mod_id
        );
    }
}



