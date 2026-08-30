use std::fs;
use std::path::Path;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::track_manager::{ModuleFilter, TrackManager};
use tdrace_app::ui::menu::TrackChoice;
use tdrace_app::ui::track_manager_ui::{TrackManagerModal, TrackManagerTab};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::TrackCategory;

#[test]
fn test_track_categories_initial_presets() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_presets_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let manager = TrackManager::new(&temp_dir);

    // 1. Initial state: 21 main presets across modules (7 Classic + 13 unique F1 + 1 unique Rally), 0 drafts
    let main_tracks = manager.main_track_choices();
    let draft_tracks = manager.draft_track_choices();

    assert_eq!(main_tracks.len(), 21, "All 21 presets across modules should be Main tracks");
    assert_eq!(draft_tracks.len(), 0, "Initial draft tracks list should be empty");

    for choice in &main_tracks {
        assert!(!choice.description().is_empty(), "Preset must have a non-empty description");
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_draft_creation_and_isolation_from_main_menu() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_drafts_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a draft track
    let draft_name = "Experimental Superloop";
    let draft_desc = "High bank experimental turn with aggressive curbs.";
    let path = manager
        .create_new_draft_track(draft_name, draft_desc)
        .expect("Draft track creation must succeed");

    assert!(Path::new(&path).exists(), "Draft track file must exist on disk");

    // 2. Verify isolation: Should appear in drafts, NOT in main
    let main_tracks = manager.main_track_choices();
    let draft_tracks = manager.draft_track_choices();

    assert_eq!(main_tracks.len(), 21, "Main menu should only contain approved circuits");
    assert_eq!(draft_tracks.len(), 1, "Drafts list should contain the newly created draft");

    let draft_choice = &draft_tracks[0];
    assert_eq!(draft_choice.title(), draft_name);
    assert_eq!(draft_choice.description(), draft_desc);
    assert!(draft_choice.is_custom());

    // 3. Load track and verify category
    let loaded = manager.load_track(draft_choice).expect("Must load draft track");
    assert_eq!(loaded.name, draft_name);
    assert_eq!(loaded.description, draft_desc);
    assert_eq!(loaded.category, TrackCategory::Draft);

    // 4. Save a track that originally had TrackCategory::Main (e.g. preset clone)
    let mut main_preset_copy = classic_grand_prix();
    main_preset_copy.name = "My Modified GP".to_string();
    assert_eq!(main_preset_copy.category, TrackCategory::Main);
    manager.save_custom_track(&main_preset_copy, Some("my_modified_gp")).expect("Save preset copy");

    // Must still land in drafts by default!
    let draft_tracks_after = manager.draft_track_choices();
    assert_eq!(draft_tracks_after.len(), 2, "Saved custom track must land in Drafts");
    let loaded_modified = manager.load_track(&draft_tracks_after.iter().find(|t| t.title() == "My Modified GP").unwrap()).unwrap();
    assert_eq!(loaded_modified.category, TrackCategory::Draft, "Saved track must have Draft category");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_promotion_and_demotion_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_lifecycle_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Create a draft
    let track_id = "test_circuit_proto";
    let mut track = classic_grand_prix();
    track.name = "Proto Circuit".to_string();
    track.description = "Prototype for testing.".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).expect("Save proto");

    assert_eq!(manager.main_track_choices().len(), 21);
    assert_eq!(manager.draft_track_choices().len(), 1);

    // Promote to F1 Module
    manager.promote_track_to_module(track_id, "f1").expect("Must promote to F1");

    assert_eq!(manager.main_track_choices().len(), 22, "Promoted track must appear in Main");
    assert_eq!(manager.draft_track_choices().len(), 0, "Promoted track must be removed from Drafts");

    let promoted_choice = manager.main_track_choices().into_iter().find(|t| t.track_id() == track_id).unwrap();
    assert_eq!(promoted_choice.title(), "Proto Circuit");
    let loaded_promoted = manager.load_track(&promoted_choice).expect("Must load promoted");
    assert_eq!(loaded_promoted.category, TrackCategory::Main);
    assert_eq!(loaded_promoted.module_id, Some("f1".to_string()));

    // Verify module_custom_tracks
    let f1_customs = manager.module_custom_tracks("f1");
    assert_eq!(f1_customs.len(), 1);
    assert_eq!(f1_customs[0].title(), "Proto Circuit");

    let rally_customs = manager.module_custom_tracks("rally");
    assert_eq!(rally_customs.len(), 0);

    // Demote back to Draft (Under testing)
    manager.demote_track(track_id).expect("Must demote to Draft");

    assert_eq!(manager.main_track_choices().len(), 21, "Demoted track must be removed from Main");
    assert_eq!(manager.draft_track_choices().len(), 1, "Demoted track must reappear in Drafts");
    assert_eq!(manager.module_custom_tracks("f1").len(), 0);

    let loaded_demoted = manager.load_track(&manager.draft_track_choices()[0]).expect("Must load demoted");
    assert_eq!(loaded_demoted.category, TrackCategory::Draft);

    // Re-promote to Rally Module
    manager.promote_track_to_module(track_id, "rally").expect("Must promote to Rally");
    assert_eq!(manager.module_custom_tracks("rally").len(), 1);
    assert_eq!(manager.module_custom_tracks("f1").len(), 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_metadata_editing() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_meta_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    let track_id = "apex_circuit";
    let mut track = classic_grand_prix();
    track.name = "Original Name".to_string();
    track.description = "Original Desc".to_string();
    track.category = TrackCategory::Main;
    manager.save_custom_track(&track, Some(track_id)).expect("Save");

    // Edit Name and Description
    manager
        .update_track_metadata(
            track_id,
            "Apex Super Circuit".to_string(),
            "Ultra high grip asphalt with high G curves.".to_string(),
        )
        .expect("Update metadata");

    let loaded = manager.load_track(&manager.draft_track_choices()[0]).expect("Load");
    assert_eq!(loaded.name, "Apex Super Circuit");
    assert_eq!(loaded.description, "Ultra high grip asphalt with high G curves.");
    assert_eq!(loaded.category, TrackCategory::Draft);

    // Promote to Official Preset and verify
    manager.promote_track(track_id).expect("Promote");
    let loaded_promoted = manager.load_track(manager.main_track_choices().iter().find(|t| t.track_id() == track_id).unwrap()).expect("Load");
    assert_eq!(loaded_promoted.name, "Apex Super Circuit");
    assert_eq!(loaded_promoted.category, TrackCategory::Main);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_deletion() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_delete_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    let track_id = "doomed_track";
    let mut track = classic_grand_prix();
    track.name = "Doomed Track".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).expect("Save");

    assert_eq!(manager.draft_track_choices().len(), 1);

    let deleted = manager.delete_custom_track(track_id).expect("Delete must succeed");
    assert!(deleted, "Should report true when track is deleted");
    assert_eq!(manager.draft_track_choices().len(), 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_race_session_with_track_manager_flow() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_session_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    // 1. Initial State: ModuleSelect (First Screen)
    assert_eq!(session.state, GameState::ModuleSelect { selected_idx: 0 });

    // 2. Transition to Track Manager
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Main,
        module_filter: ModuleFilter::All,
        selected_idx: 0,
        modal: TrackManagerModal::None,
    };

    assert!(matches!(session.state, GameState::TrackManager { .. }));

    // 3. Create a draft in the session
    session
        .track_manager
        .create_new_draft_track("Session Draft", "Created in session test")
        .expect("Create draft");

    assert_eq!(session.track_manager.draft_track_choices().len(), 1);

    // 4. Start race on the draft track
    let draft_choice = session.track_manager.draft_track_choices()[0].clone();
    session.track_choice = draft_choice;
    session.init_race();

    // Verify session initialized
    assert_eq!(session.state, GameState::StartingGrid);
    assert_eq!(session.track.name, "Session Draft");
    assert_eq!(session.track.description, "Created in session test");
    assert_eq!(session.cars.len(), 8); // Starter layout has 8 slots

    // Run 10 physics steps
    for _ in 0..10 {
        session.physics_step(1.0 / 60.0);
    }
    assert!(session.cars[0].state.position.length() > 0.0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_confirm_delete_modal() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_confirm_del_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    session
        .track_manager
        .create_new_draft_track("Track To Delete", "Test delete modal")
        .expect("Create draft");

    assert_eq!(session.track_manager.draft_track_choices().len(), 1);
    let track_id = session.track_manager.draft_track_choices()[0].track_id().to_string();

    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Drafts,
        module_filter: ModuleFilter::All,
        selected_idx: 0,
        modal: TrackManagerModal::ConfirmDelete {
            track_id: track_id.clone(),
            track_title: "Track To Delete".to_string(),
        },
    };

    // Simulate modal deletion
    let deleted = session.track_manager.delete_custom_track(&track_id).unwrap();
    assert!(deleted);
    assert_eq!(session.track_manager.draft_track_choices().len(), 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_module_filter_filtering_and_presets_in_classic() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_filter_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial state: 21 tracks across all modules (7 Classic + 13 unique F1 + 1 unique Rally)
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::All).len(), 21);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), 7);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::F1).len(), 14);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Rally).len(), 3);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Kart).len(), 2);

    // Promote a new track to F1
    let mut track_f1 = classic_grand_prix();
    track_f1.name = "Monza Custom GP".to_string();
    manager.save_custom_track(&track_f1, Some("monza_custom")).unwrap();
    manager.promote_track_to_module("monza_custom", "f1").unwrap();

    // Promote a new track to Rally
    let mut track_rally = classic_grand_prix();
    track_rally.name = "Dune Safari".to_string();
    manager.save_custom_track(&track_rally, Some("dune_safari")).unwrap();
    manager.promote_track_to_module("dune_safari", "rally").unwrap();

    // Verify filtered counts
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::All).len(), 23);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), 7);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::F1).len(), 15);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Rally).len(), 4);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Kart).len(), 2);

    // Verify filter cycle (.next())
    assert_eq!(ModuleFilter::All.next(), ModuleFilter::Classic);
    assert_eq!(ModuleFilter::Classic.next(), ModuleFilter::Rally);
    assert_eq!(ModuleFilter::Rally.next(), ModuleFilter::Kart);
    assert_eq!(ModuleFilter::Kart.next(), ModuleFilter::F1);
    assert_eq!(ModuleFilter::F1.next(), ModuleFilter::All);

    // Verify filter cycle (.prev())
    assert_eq!(ModuleFilter::All.prev(), ModuleFilter::F1);
    assert_eq!(ModuleFilter::F1.prev(), ModuleFilter::Kart);
    assert_eq!(ModuleFilter::Kart.prev(), ModuleFilter::Rally);
    assert_eq!(ModuleFilter::Rally.prev(), ModuleFilter::Classic);
    assert_eq!(ModuleFilter::Classic.prev(), ModuleFilter::All);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_module_subdirectories_and_file_movement() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_subdirs_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a draft track -> should land in temp_dir/drafts/my_circuit.json
    let mut track = classic_grand_prix();
    track.name = "My Circuit".to_string();
    track.category = TrackCategory::Draft;
    let saved_path = manager.save_custom_track(&track, Some("my_circuit")).expect("Save draft");
    let draft_file = temp_dir.join("drafts").join("my_circuit.json");
    assert!(draft_file.exists(), "Draft file must be saved in drafts/ subdirectory");
    assert_eq!(Path::new(&saved_path), draft_file);

    // 2. Promote to F1 -> should move file to temp_dir/f1/my_circuit.json and remove from drafts/
    manager.promote_track_to_module("my_circuit", "f1").expect("Promote to F1");
    let f1_file = temp_dir.join("f1").join("my_circuit.json");
    assert!(f1_file.exists(), "Promoted file must exist in f1/ subdirectory");
    assert!(!draft_file.exists(), "Old draft file must be removed after promotion");

    // 3. Demote back to Draft -> should move file back to temp_dir/drafts/my_circuit.json
    manager.demote_track("my_circuit").expect("Demote to Draft");
    assert!(draft_file.exists(), "Demoted file must exist back in drafts/ subdirectory");
    assert!(!f1_file.exists(), "Old f1 file must be removed after demotion");

    // 4. Promote to Rally -> should move file to temp_dir/rally/my_circuit.json
    manager.promote_track_to_module("my_circuit", "rally").expect("Promote to Rally");
    let rally_file = temp_dir.join("rally").join("my_circuit.json");
    assert!(rally_file.exists(), "Promoted file must exist in rally/ subdirectory");
    assert!(!draft_file.exists(), "Old draft file must be removed");

    // 5. Test multi-directory scanner on fresh TrackManager instance
    let new_scanner = TrackManager::new(&temp_dir);
    assert_eq!(new_scanner.main_track_choices().len(), 22); // 21 presets + 1 custom
    assert_eq!(new_scanner.module_custom_tracks("rally").len(), 1);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_tab_and_module_cycling() {
    let mut active_tab = TrackManagerTab::Main;
    let mut module_filter = ModuleFilter::All;

    // Tab toggle
    active_tab = match active_tab {
        TrackManagerTab::Main => TrackManagerTab::Drafts,
        TrackManagerTab::Drafts => TrackManagerTab::Main,
    };
    assert_eq!(active_tab, TrackManagerTab::Drafts);

    active_tab = match active_tab {
        TrackManagerTab::Main => TrackManagerTab::Drafts,
        TrackManagerTab::Drafts => TrackManagerTab::Main,
    };
    assert_eq!(active_tab, TrackManagerTab::Main);

    // Module cycling forward (Right arrow)
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Classic);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Rally);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Kart);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::F1);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::All);

    // Module cycling backward (Left arrow)
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::F1);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Kart);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Rally);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Classic);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::All);
}

#[test]
fn test_track_manager_open_in_track_editor() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_open_editor_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    // 1. Create a draft track
    session
        .track_manager
        .create_new_draft_track("Track For Editor", "Draft circuit to edit")
        .expect("Create draft");

    let draft_choice = session.track_manager.draft_track_choices()[0].clone();

    // 2. Open selected track in Track Editor
    let file_path = match &draft_choice {
        TrackChoice::Custom { path, .. } => Some(path.clone()),
        _ => None,
    };
    let track = session.track_manager.load_track(&draft_choice).expect("Load draft");
    session.track_choice = draft_choice.clone();
    session.track = track.clone();
    session.enter_track_editor_with_path(track, file_path.clone());

    // 3. Verify session state is TrackEditor and contains the selected track
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(session.editor_state.is_some());
    let editor_state = session.editor_state.as_ref().unwrap();
    assert_eq!(editor_state.track.name, "Track For Editor");
    assert_eq!(editor_state.current_file_path, file_path);

    let _ = fs::remove_dir_all(&temp_dir);
}



