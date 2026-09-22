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
