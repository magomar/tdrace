use std::fs;
use tdrace_app::tournament::manager::EMBEDDED_PRESETS;
use tdrace_app::ui::menu::{ModalityCategory, ModalityItem};
use tdrace_app::{
    ChampionshipDefinition, ChampionshipManager, GameState, PointSystem, RaceSession,
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

    assert_eq!(def.championship.id, "gt3_apex_masters");
    assert_eq!(def.championship.name, "GT3 Apex Masters");
    assert_eq!(def.championship.module_id, "gt");
    assert_eq!(def.championship.tier, 3);
    assert_eq!(def.championship.laps_per_round, 5);
    assert_eq!(def.championship.bot_count, Some(7));
    assert_eq!(def.championship.ai_difficulty.as_deref(), Some("pro"));

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
    def.championship.id = "".to_string();
    assert!(def.validate().is_err());
    def.championship.id = "valid_id".to_string();

    // 2. Empty Name
    def.championship.name = "".to_string();
    assert!(def.validate().is_err());
    def.championship.name = "Valid Name".to_string();

    // 3. Zero laps
    def.championship.laps_per_round = 0;
    assert!(def.validate().is_err());
    def.championship.laps_per_round = 3;

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
    assert_eq!(recovered.championship.name, "GT3 Apex Masters");
    assert_eq!(recovered.championship.module_id, "gt");
    assert_eq!(recovered.championship.tier, 3);
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

    let mut mgr = ChampionshipManager::with_dirs(user_dir.clone(), Some(git_dir.clone()));

    // 1. Embedded presets are present
    assert!(!mgr.championships.is_empty());
    assert!(mgr.get("gt4_clubman_sprint").is_some());
    assert!(mgr.get("nascar_cup_tier5").is_some());
    assert!(mgr.get("rally_world_cup").is_some());
    assert!(mgr.get("kart_world_cup").is_some());
    assert!(mgr.get("extreme_offroad_cup").is_some());

    // 2. Save new user championship
    let mut custom_def = ChampionshipDefinition::from_toml(SAMPLE_CHAMPIONSHIP_TOML).unwrap();
    custom_def.championship.id = "my_custom_user_cup".to_string();
    custom_def.championship.name = "My Custom User Cup".to_string();

    let saved_path = mgr
        .save_user_championship(&custom_def)
        .expect("Save user championship");
    assert!(saved_path.exists());
    assert!(mgr.get("my_custom_user_cup").is_some());

    // 3. Rescan discovers the saved user cup
    mgr.scan_all();
    assert!(mgr.get("my_custom_user_cup").is_some());
    assert_eq!(
        mgr.get("my_custom_user_cup").unwrap().championship.name,
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
            &def.championship.id, slug,
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
        options_items.contains(&ModalityItem::ChampionshipEditor),
        "ModalityCategory::Options must contain ChampionshipEditor"
    );

    assert_eq!(
        ModalityItem::ChampionshipEditor.title(),
        "Championship Editor"
    );
    assert_eq!(
        ModalityItem::ChampionshipEditor.tag(),
        "CUSTOM CUP CREATOR • TOML WORKBENCH"
    );
    assert!(ModalityItem::ChampionshipEditor.is_available());
}

#[test]
fn test_game_session_championship_editor_integration() {
    let mut session = RaceSession::new();

    // Verify championship manager is initialized
    assert!(!session.championship_manager.championships.is_empty());
    assert!(session.championship_editor_state.is_none());

    // Enter Championship Editor
    session.enter_championship_editor(None);
    assert_eq!(session.state, GameState::ChampionshipEditor);
    assert!(session.championship_editor_state.is_some());

    // Verify initial default definition in editor
    let state = session.championship_editor_state.as_ref().unwrap();
    assert_eq!(state.def.championship.module_id, "gt");
    assert!(state.def.rounds.is_empty());
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
