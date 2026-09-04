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

    // 1. Initial state: 34 main presets across modules (7 Classic + 13 unique F1 + 6 unique Rally + 8 famous Kart), 0 drafts
    let main_tracks = manager.main_track_choices();
    let draft_tracks = manager.draft_track_choices();

    assert_eq!(main_tracks.len(), 34, "All 34 presets across modules should be Main tracks");
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

    assert_eq!(main_tracks.len(), 34, "Main menu should only contain approved circuits");
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

    assert_eq!(manager.main_track_choices().len(), 34);
    assert_eq!(manager.draft_track_choices().len(), 1);

    // Promote to F1 Module
    manager.promote_track_to_module(track_id, "f1").expect("Must promote to F1");

    assert_eq!(manager.main_track_choices().len(), 35, "Promoted track must appear in Main");
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

    assert_eq!(manager.main_track_choices().len(), 34, "Demoted track must be removed from Main");
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

    // Initial state: 34 tracks across all modules (7 Classic + 13 unique F1 + 6 unique Rally + 8 famous Kart)
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::All).len(), 34);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), 7);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::F1).len(), 14);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Rally).len(), 8);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Kart).len(), 10);

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
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::All).len(), 36);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), 7);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::F1).len(), 15);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Rally).len(), 9);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Kart).len(), 10);

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
    assert_eq!(new_scanner.main_track_choices().len(), 35); // 34 presets + 1 custom
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

#[test]
fn test_track_manager_delete_with_backspace() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_backspace_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    // Create a draft track
    session
        .track_manager
        .create_new_draft_track("Track To Delete Backspace", "Test backspace key delete")
        .expect("Create draft");

    assert_eq!(session.track_manager.draft_track_choices().len(), 1);
    let track_id = session.track_manager.draft_track_choices()[0].track_id().to_string();

    // Confirm deletion modal state with track
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Drafts,
        module_filter: ModuleFilter::All,
        selected_idx: 0,
        modal: TrackManagerModal::ConfirmDelete {
            track_id: track_id.clone(),
            track_title: "Track To Delete Backspace".to_string(),
        },
    };

    // Perform deletion
    let deleted = session.track_manager.delete_custom_track(&track_id).unwrap();
    assert!(deleted);
    assert_eq!(session.track_manager.draft_track_choices().len(), 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_predefined_track_demote_promote_and_delete() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_predefined_ops_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial state: 7 Classic tracks, 29 total
    let init_classic_count = manager.filtered_main_track_choices(ModuleFilter::Classic).len();
    assert_eq!(init_classic_count, 7);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // 1. Demote built-in preset Classic Grand Prix (P key)
    manager.demote_track("classic_grand_prix").expect("Must demote predefined track");

    // Classic Grand Prix must now appear in Drafts and be removed from Main/Classic
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), init_classic_count - 1);
    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.draft_track_choices()[0].track_id(), "classic_grand_prix");

    // 2. Promote it back to Classic module (P key on Draft)
    manager.promote_track_to_module("classic_grand_prix", "classic").expect("Must promote track back");
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), init_classic_count);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // 3. Delete built-in preset Classic Grand Prix (Backspace / Delete)
    manager.delete_custom_track("classic_grand_prix").expect("Must delete predefined track");
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), init_classic_count - 1);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // 4. Test F1 predefined track (Monza) demotion and deletion
    let init_f1_count = manager.filtered_main_track_choices(ModuleFilter::F1).len();
    manager.demote_track("monza").expect("Must demote Monza");
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::F1).len(), init_f1_count - 1);
    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.draft_track_choices()[0].track_id(), "monza");

    manager.delete_custom_track("monza").expect("Must delete Monza");
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::F1).len(), init_f1_count - 1);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // 5. Verify persistence across new TrackManager instance
    let manager2 = TrackManager::new(&temp_dir);
    assert_eq!(manager2.filtered_main_track_choices(ModuleFilter::Classic).len(), init_classic_count - 1);
    assert_eq!(manager2.filtered_main_track_choices(ModuleFilter::F1).len(), init_f1_count - 1);
    assert_eq!(manager2.draft_track_choices().len(), 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_multi_module_promotion_and_distribution() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_multi_promo_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a draft circuit
    let track_id = "multi_spec_gp";
    let mut track = classic_grand_prix();
    track.name = "Multi Spec GP".to_string();
    track.description = "Circuit designed for multiple disciplines.".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).expect("Save proto");

    assert_eq!(manager.draft_track_choices().len(), 1);

    // 2. Promote simultaneously to Rally, F1, and Classic modules
    manager
        .promote_track_to_modules(track_id, &["rally", "f1", "classic"])
        .expect("Must promote to multiple modules");

    // 3. Verify files on disk in module folders
    assert!(!temp_dir.join("drafts").join(format!("{}.json", track_id)).exists(), "Draft file must be cleaned up");
    assert!(temp_dir.join("rally").join(format!("{}.json", track_id)).exists(), "Rally copy must exist");
    assert!(temp_dir.join("f1").join(format!("{}.json", track_id)).exists(), "F1 copy must exist");
    assert!(temp_dir.join("classic").join(format!("{}.json", track_id)).exists(), "Classic copy must exist");
    assert!(!temp_dir.join("kart").join(format!("{}.json", track_id)).exists(), "Kart copy must not exist");

    // 4. Verify catalog queries
    assert_eq!(manager.draft_track_choices().len(), 0, "No drafts remaining");

    let rally_customs = manager.module_custom_tracks("rally");
    assert_eq!(rally_customs.len(), 1);
    assert_eq!(rally_customs[0].title(), "Multi Spec GP");

    let f1_customs = manager.module_custom_tracks("f1");
    assert_eq!(f1_customs.len(), 1);
    assert_eq!(f1_customs[0].title(), "Multi Spec GP");

    let classic_customs = manager.module_custom_tracks("classic");
    assert_eq!(classic_customs.len(), 1);
    assert_eq!(classic_customs[0].title(), "Multi Spec GP");

    let kart_customs = manager.module_custom_tracks("kart");
    assert_eq!(kart_customs.len(), 0);

    // 5. Verify module filter choices
    let rally_choices = manager.filtered_main_track_choices(ModuleFilter::Rally);
    assert!(rally_choices.iter().any(|c| c.track_id() == track_id));

    let f1_choices = manager.filtered_main_track_choices(ModuleFilter::F1);
    assert!(f1_choices.iter().any(|c| c.track_id() == track_id));

    let classic_choices = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(classic_choices.iter().any(|c| c.track_id() == track_id));

    let kart_choices = manager.filtered_main_track_choices(ModuleFilter::Kart);
    assert!(!kart_choices.iter().any(|c| c.track_id() == track_id));

    // 6. Demote track back to Drafts
    manager.demote_track(track_id).expect("Must demote multi-module track");

    // 7. Verify all module copies are cleaned up and single draft restored
    assert!(temp_dir.join("drafts").join(format!("{}.json", track_id)).exists(), "Draft file must be restored");
    assert!(!temp_dir.join("rally").join(format!("{}.json", track_id)).exists(), "Rally copy removed");
    assert!(!temp_dir.join("f1").join(format!("{}.json", track_id)).exists(), "F1 copy removed");
    assert!(!temp_dir.join("classic").join(format!("{}.json", track_id)).exists(), "Classic copy removed");

    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.module_custom_tracks("rally").len(), 0);
    assert_eq!(manager.module_custom_tracks("f1").len(), 0);
    assert_eq!(manager.module_custom_tracks("classic").len(), 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_consistent_module_categorization_in_module_view() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_module_cat_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Promote a draft to Classic and Rally
    let mut track = classic_grand_prix();
    track.name = "Desert Speed".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some("desert_speed")).expect("Save");
    manager.promote_track_to_modules("desert_speed", &["classic", "rally"]).expect("Promote");

    // In Classic view, all tracks should belong to Classic
    let classic_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(!classic_tracks.is_empty());
    for t in &classic_tracks {
        let tag = t.tag_for_module("classic");
        assert!(
            tag == "CLASSIC MOTORSPORT" || tag == "FIA GP CIRCUIT" || tag == "SUPERSPEEDWAY" || tag == "TECHNICAL DRIFT" || tag == "AGILE SPRINT" || tag == "STUNT RAMPS & JUMPS" || tag == "DESERT DIRT RALLY" || tag == "NARROW MOUNTAIN PASS",
            "Track in classic view should have valid classic tag: {}", tag
        );
    }

    // In Rally view, all tracks should belong to Rally
    let rally_tracks = manager.filtered_main_track_choices(ModuleFilter::Rally);
    assert!(!rally_tracks.is_empty());
    for t in &rally_tracks {
        let tag = t.tag_for_module("rally");
        assert!(
            tag == "RALLY CROSS" || tag == "DESERT DIRT RALLY" || tag == "NARROW MOUNTAIN PASS",
            "Track in rally view should have rally tag: {}", tag
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_empty_module_tracks_resilience() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_empty_tracks_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::new();
    session.track_manager = TrackManager::new(&temp_dir);

    // Delete all known presets so module catalogs are completely empty
    session.track_manager.deleted_presets = vec![
        "monza".into(), "spa".into(), "silverstone".into(), "monaco".into(), "suzuka".into(),
        "interlagos".into(), "montreal".into(), "red_bull_ring".into(), "catalunya".into(),
        "zandvoort".into(), "bahrain".into(), "marina_bay".into(), "cota".into(),
        "classic_grand_prix".into(), "oval_speedway".into(), "drift_park".into(),
        "kart_arena".into(), "ramp_raceway".into(), "oasis_rally".into(), "outlaw_pass".into(),
        "sahara".into(), "dirt_figure_eight".into(), "holjes_rx".into(), "lydden_hill".into(),
        "hell_rx".into(), "loheac_rx".into(), "lonato".into(), "sarno".into(), "genk".into(), "pfi".into(),
        "zuera".into(), "le_mans_kart".into(), "portimao_kart".into(), "franciacorta".into(),
    ];

    assert_eq!(session.track_manager.module_catalog_tracks("f1").len(), 0);
    assert_eq!(session.track_manager.module_catalog_tracks("rally").len(), 0);
    assert_eq!(session.track_manager.module_catalog_tracks("kart").len(), 0);
    assert_eq!(session.track_manager.module_catalog_tracks("classic").len(), 0);

    // Ensure switching to each module succeeds gracefully without panicking
    session.switch_to_f1();
    assert_eq!(session.active_module_id, "f1");

    session.switch_to_rally();
    assert_eq!(session.active_module_id, "rally");

    session.switch_to_kart();
    assert_eq!(session.active_module_id, "kart");

    session.switch_to_classic();
    assert_eq!(session.active_module_id, "classic");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_preset_circuits_edit_overwrite_and_persistence_across_modules() {
    use tdrace_core::track::presets::oval_speedway;
    use tdrace_app::module::f1::F1GameModule;
    use tdrace_app::ui::menu::resolve_track_for_menu_with_dir;

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_preset_persistence_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Edit Classic preset (Oval Speedway)
    let mut oval = oval_speedway();
    oval.name = "Oval Speedway (Modified)".to_string();
    oval.description = "Customized banked oval.".to_string();
    let oval_path = manager
        .save_custom_track_with_options(&oval, Some("oval_speedway"), true)
        .expect("Save oval preset");
    assert!(oval_path.ends_with("classic/oval_speedway.json"), "Must save to classic/ subdirectory, got: {}", oval_path);

    // 2. Edit F1 preset (Monza)
    let mut monza = F1GameModule::track_monza();
    monza.name = "Monza Modified GP".to_string();
    monza.description = "Edited temple of speed.".to_string();
    let monza_path = manager
        .save_custom_track_with_options(&monza, Some("monza"), true)
        .expect("Save monza preset");
    assert!(monza_path.ends_with("f1/monza.json"), "Must save to f1/ subdirectory, got: {}", monza_path);

    // 3. Verify they remain Main category and are NOT demoted to drafts
    assert_eq!(manager.draft_track_choices().len(), 0, "No drafts should exist after editing presets");
    let main_tracks = manager.main_track_choices();
    assert_eq!(main_tracks.len(), 34, "All 34 presets must remain in main tracks list");

    let f1_catalog = manager.module_catalog_tracks("f1");
    let monza_choice = f1_catalog.iter().find(|t| t.track_id() == "monza").expect("Monza must be in F1 catalog");
    assert_eq!(monza_choice.title(), "Monza Modified GP");

    let classic_catalog = manager.module_catalog_tracks("classic");
    let oval_choice = classic_catalog.iter().find(|t| t.track_id() == "oval_speedway").expect("Oval must be in Classic catalog");
    assert_eq!(oval_choice.title(), "Oval Speedway (Modified)");

    // 4. Verify loading loads the edited version from disk
    let loaded_oval = manager.load_track(&TrackChoice::OvalSpeedway).unwrap();
    assert_eq!(loaded_oval.name, "Oval Speedway (Modified)");
    assert_eq!(loaded_oval.category, TrackCategory::Main);
    assert_eq!(loaded_oval.module_id, Some("classic".to_string()));

    let loaded_monza = manager.load_track_by_slug("monza").unwrap();
    assert_eq!(loaded_monza.name, "Monza Modified GP");
    assert_eq!(loaded_monza.category, TrackCategory::Main);
    assert_eq!(loaded_monza.module_id, Some("f1".to_string()));

    // 5. Verify resolve_track_for_menu
    let menu_oval = resolve_track_for_menu_with_dir(&TrackChoice::OvalSpeedway, &temp_dir).unwrap();
    assert_eq!(menu_oval.name, "Oval Speedway (Modified)");

    let menu_monza = resolve_track_for_menu_with_dir(monza_choice, &temp_dir).unwrap();
    assert_eq!(menu_monza.name, "Monza Modified GP");

    // 6. Verify Save as Copy lands in drafts
    let copy_path = manager
        .save_custom_track_with_options(&monza, Some("monza_custom_copy"), false)
        .expect("Save copy");
    assert!(copy_path.ends_with("drafts/monza_custom_copy.json"));
    assert_eq!(manager.draft_track_choices().len(), 1);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_re_promoting_already_promoted_track_to_different_modules() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_repromo_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a draft track
    let track_id = "dynamic_apex_gp";
    let mut track = classic_grand_prix();
    track.name = "Dynamic Apex GP".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).expect("Save draft");

    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.track_promoted_modules(track_id).len(), 0);
    assert!(!manager.is_track_in_module(track_id, "classic"));

    // 2. Initial Promotion to Classic module
    manager.promote_track_to_modules(track_id, &["classic"]).expect("Promote to classic");
    assert_eq!(manager.draft_track_choices().len(), 0);
    assert!(temp_dir.join("classic").join(format!("{}.json", track_id)).exists());
    assert!(manager.is_track_in_module(track_id, "classic"));
    assert_eq!(manager.track_promoted_modules(track_id), vec!["classic".to_string()]);

    // 3. Re-promote to different modules: change from Classic to Rally and Kart (removing Classic)
    manager
        .promote_track_to_modules(track_id, &["rally", "kart"])
        .expect("Re-promote to rally and kart");

    assert!(!temp_dir.join("classic").join(format!("{}.json", track_id)).exists(), "Classic file must be removed");
    assert!(temp_dir.join("rally").join(format!("{}.json", track_id)).exists(), "Rally file must exist");
    assert!(temp_dir.join("kart").join(format!("{}.json", track_id)).exists(), "Kart file must exist");

    assert!(!manager.is_track_in_module(track_id, "classic"));
    assert!(manager.is_track_in_module(track_id, "rally"));
    assert!(manager.is_track_in_module(track_id, "kart"));
    assert!(!manager.is_track_in_module(track_id, "f1"));

    let promoted_mods = manager.track_promoted_modules(track_id);
    assert_eq!(promoted_mods, vec!["rally".to_string(), "kart".to_string()]);

    // 4. Add F1 module without removing Rally or Kart (Rally, Kart, F1)
    manager
        .promote_track_to_modules(track_id, &["rally", "kart", "f1"])
        .expect("Add F1 module");

    assert!(temp_dir.join("rally").join(format!("{}.json", track_id)).exists());
    assert!(temp_dir.join("kart").join(format!("{}.json", track_id)).exists());
    assert!(temp_dir.join("f1").join(format!("{}.json", track_id)).exists());
    assert!(!temp_dir.join("classic").join(format!("{}.json", track_id)).exists());

    let promoted_mods_3 = manager.track_promoted_modules(track_id);
    assert_eq!(promoted_mods_3, vec!["rally".to_string(), "kart".to_string(), "f1".to_string()]);

    // 5. Promote with empty modules slice -> should demote track back to Drafts
    manager
        .promote_track_to_modules(track_id, &[])
        .expect("Empty promotion should demote");

    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.track_promoted_modules(track_id).len(), 0);
    assert!(temp_dir.join("drafts").join(format!("{}.json", track_id)).exists(), "Draft file must be restored");
    assert!(!temp_dir.join("rally").join(format!("{}.json", track_id)).exists());
    assert!(!temp_dir.join("kart").join(format!("{}.json", track_id)).exists());
    assert!(!temp_dir.join("f1").join(format!("{}.json", track_id)).exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_promotion_mask_resolution_for_promoted_track() {
    use tdrace_app::ui::track_manager_ui::PROMOTION_MODULES;

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_mask_res_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Create and promote to Kart and F1
    let track_id = "kart_f1_hybrid";
    let mut track = classic_grand_prix();
    track.name = "Hybrid Circuit".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).unwrap();
    manager.promote_track_to_modules(track_id, &["kart", "f1"]).unwrap();

    // Verify mask resolution logic matches PROMOTION_MODULES indices
    let mut selected_mask = [false; 4];
    for (idx, (mod_id, _, _, _)) in PROMOTION_MODULES.iter().enumerate() {
        if manager.is_track_in_module(track_id, mod_id) {
            selected_mask[idx] = true;
        }
    }

    // PROMOTION_MODULES order: 0: classic, 1: rally, 2: kart, 3: f1
    assert_eq!(selected_mask, [false, false, true, true]);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_create_new_draft_track_with_module_templates() {
    use std::fs;
    use tdrace_app::track_manager::TrackManager;
    use tdrace_core::physics::surface::SurfaceType;
    use tdrace_core::track::presets::{RaceDirection, TrackShape};
    use tdrace_core::track::Track;

    let temp_dir = std::env::temp_dir().join(format!("tdrace_mgr_template_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);
    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a rally draft track (Horizontal 8, Left)
    let rally_path = manager
        .create_new_draft_track_with_template(
            "Rally Draft Stage",
            "Loose dirt testing stage",
            "rally",
            TrackShape::HorizontalEight,
            RaceDirection::Left,
        )
        .expect("Rally draft creation should succeed");

    let rally_track = Track::load_from_file(&rally_path).expect("Load rally draft track");
    assert_eq!(rally_track.default_surface, SurfaceType::Dirt);
    assert_eq!(rally_track.spline.samples[0].surface, SurfaceType::Dirt);
    assert_eq!(rally_track.predefined_car.as_deref(), Some("rally_car"));

    // 2. Create a kart draft track (Oval, Right)
    let kart_path = manager
        .create_new_draft_track_with_template(
            "Karting Oval Arena",
            "Tight kart arena",
            "kart",
            TrackShape::Oval,
            RaceDirection::Right,
        )
        .expect("Kart draft creation should succeed");

    let kart_track = Track::load_from_file(&kart_path).expect("Load kart draft track");
    assert_eq!(kart_track.default_surface, SurfaceType::Asphalt);
    assert_eq!(kart_track.spline.samples[0].surface, SurfaceType::Asphalt);
    assert_eq!(kart_track.predefined_car.as_deref(), Some("kart"));

    // 3. Create an F1 draft track (Oval, Right)
    let f1_path = manager
        .create_new_draft_track_with_template(
            "Formula Super Speedway",
            "High speed DRS oval",
            "f1",
            TrackShape::Oval,
            RaceDirection::Right,
        )
        .expect("F1 draft creation should succeed");

    let f1_track = Track::load_from_file(&f1_path).expect("Load F1 draft track");
    assert_eq!(f1_track.default_surface, SurfaceType::Grass);
    assert_eq!(f1_track.spline.samples[0].surface, SurfaceType::Asphalt);
    assert_eq!(f1_track.predefined_car.as_deref(), Some("f1_car"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_clone_preset_to_drafts() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_clone_preset_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial check: 34 main tracks, 0 drafts
    assert_eq!(manager.main_track_choices().len(), 34);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // Clone Classic Grand Prix
    let gp_choice = TrackChoice::ClassicGrandPrix;
    let (cloned_track, saved_path) = manager.clone_track(&gp_choice).expect("Clone preset must succeed");

    assert_eq!(cloned_track.name, "Classic Grand Prix (clone)");
    assert_eq!(cloned_track.category, TrackCategory::Draft);
    assert!(cloned_track.module_id.is_none());
    assert!(cloned_track.modules.is_empty());
    assert!(Path::new(&saved_path).exists());
    assert!(saved_path.contains("drafts"));

    // Verify drafts list has 1 track, main still has 34
    assert_eq!(manager.main_track_choices().len(), 34);
    let drafts = manager.draft_track_choices();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].title(), "Classic Grand Prix (clone)");

    // Verify exact clone of geometry & properties
    let original = manager.load_track(&gp_choice).unwrap();
    assert_eq!(cloned_track.spline.waypoints.len(), original.spline.waypoints.len());
    assert_eq!(cloned_track.checkpoints.len(), original.checkpoints.len());
    assert_eq!(cloned_track.default_surface, original.default_surface);
    assert_eq!(cloned_track.default_laps, original.default_laps);
    assert_eq!(cloned_track.predefined_car, original.predefined_car);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_clone_and_open_in_track_editor() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_clone_editor_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    // Enter track manager
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Main,
        module_filter: ModuleFilter::All,
        selected_idx: 0,
        modal: TrackManagerModal::None,
    };

    let selected_track_choice = session.track_manager.main_track_choices()[0].clone();
    let original_title = selected_track_choice.title().to_string();

    // Execute cloning flow as triggered by C key
    let (cloned_track, file_path) = session
        .track_manager
        .clone_track(&selected_track_choice)
        .expect("Clone track");

    let file_stem = std::path::Path::new(&file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("cloned_track")
        .to_string();

    session.track_choice = TrackChoice::Custom {
        id: file_stem,
        title: cloned_track.name.clone(),
        description: cloned_track.description.clone(),
        path: file_path.clone(),
    };
    session.track = cloned_track.clone();
    session.enter_track_editor_with_path(cloned_track, Some(file_path.clone()));

    // Verify immediately opened in track editor
    assert_eq!(session.state, GameState::TrackEditor);
    assert!(session.editor_state.is_some());

    let editor_state = session.editor_state.as_ref().unwrap();
    assert_eq!(editor_state.track.name, format!("{} (clone)", original_title));
    assert_eq!(editor_state.track.category, TrackCategory::Draft);
    assert_eq!(editor_state.current_file_path, Some(file_path.clone()));
    assert!(Path::new(&file_path).exists());

    // Verify the clone exists in drafts group of track manager
    assert_eq!(session.track_manager.draft_track_choices().len(), 1);
    assert_eq!(session.track_manager.draft_track_choices()[0].title(), format!("{} (clone)", original_title));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_repeated_cloning_unique_slugs() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_clone_repeated_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Create a draft
    let initial_path = manager
        .create_new_draft_track("Loop Track", "Test loop track")
        .expect("Create draft");
    assert!(Path::new(&initial_path).exists());
    assert_eq!(manager.draft_track_choices().len(), 1);

    let draft_choice = manager.draft_track_choices()[0].clone();

    // 1st Clone
    let (clone1, path1) = manager.clone_track(&draft_choice).expect("Clone 1");
    assert_eq!(clone1.name, "Loop Track (clone)");
    assert!(path1.ends_with("loop_track_clone.json"));
    assert!(Path::new(&path1).exists());
    assert_eq!(manager.draft_track_choices().len(), 2);

    // 2nd Clone of the same original
    let (clone2, path2) = manager.clone_track(&draft_choice).expect("Clone 2");
    assert_eq!(clone2.name, "Loop Track (clone)");
    assert!(path2.ends_with("loop_track_clone_1.json"));
    assert!(Path::new(&path2).exists());
    assert_eq!(manager.draft_track_choices().len(), 3);

    // 3rd Clone of the first clone
    let clone1_choice = manager.draft_track_choices().into_iter().find(|t| t.title() == "Loop Track (clone)").unwrap();
    let (clone3, path3) = manager.clone_track(&clone1_choice).expect("Clone of clone");
    assert_eq!(clone3.name, "Loop Track (clone) (clone)");
    assert!(path3.ends_with("loop_track_clone_clone.json"));
    assert!(Path::new(&path3).exists());
    assert_eq!(manager.draft_track_choices().len(), 4);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_module_scoped_track_deletion_preserves_other_modules() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_scoped_del_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Promote a track to both classic and rally
    let track_id = "dual_discipline_gp";
    let mut track = classic_grand_prix();
    track.name = "Dual Discipline GP".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).expect("Save draft");
    manager.promote_track_to_modules(track_id, &["classic", "rally"]).expect("Promote to both");

    assert!(manager.filtered_main_track_choices(ModuleFilter::Classic).iter().any(|c| c.track_id() == track_id));
    assert!(manager.filtered_main_track_choices(ModuleFilter::Rally).iter().any(|c| c.track_id() == track_id));

    // 2. Delete track specifically from the Rally module
    let deleted = manager.delete_track_from_module(track_id, Some("rally")).expect("Delete from rally");
    assert!(deleted);

    // 3. Must be removed from Rally, but STILL present in Classic!
    assert!(!manager.filtered_main_track_choices(ModuleFilter::Rally).iter().any(|c| c.track_id() == track_id), "Must be removed from rally");
    assert!(manager.filtered_main_track_choices(ModuleFilter::Classic).iter().any(|c| c.track_id() == track_id), "Must remain in classic");

    // 4. Verify classic track file is intact on disk
    let classic_path = temp_dir.join("classic").join(format!("{}.json", track_id));
    assert!(classic_path.exists(), "Classic file must not be deleted");
    let rally_path = temp_dir.join("rally").join(format!("{}.json", track_id));
    assert!(!rally_path.exists(), "Rally file must be deleted");

    // 5. Test with built-in rally preset (dirt_figure_eight) promoted to classic
    let mut gp_track = tdrace_core::track::presets::dirt_figure_eight();
    gp_track.name = "Dirt Figure-8 Classic".to_string();
    gp_track.category = TrackCategory::Main;
    gp_track.module_id = Some("classic".to_string());
    gp_track.modules = vec!["classic".to_string()];
    let classic_dirt_path = temp_dir.join("classic").join("dirt_figure_eight.json");
    let _ = fs::create_dir_all(temp_dir.join("classic"));
    gp_track.save_to_file(&classic_dirt_path).expect("Save classic dirt");
    let _ = manager.scan_custom_tracks();

    assert!(manager.filtered_main_track_choices(ModuleFilter::Classic).iter().any(|c| c.track_id() == "dirt_figure_eight"));
    assert!(manager.filtered_main_track_choices(ModuleFilter::Rally).iter().any(|c| c.track_id() == "dirt_figure_eight"));

    // Delete dirt_figure_eight specifically from Rally
    manager.delete_track_from_module("dirt_figure_eight", Some("rally")).expect("Delete from rally");

    assert!(!manager.filtered_main_track_choices(ModuleFilter::Rally).iter().any(|c| c.track_id() == "dirt_figure_eight"), "dirt_figure_eight removed from rally");
    assert!(manager.filtered_main_track_choices(ModuleFilter::Classic).iter().any(|c| c.track_id() == "dirt_figure_eight"), "dirt_figure_eight must remain in classic");
    assert!(classic_dirt_path.exists(), "Classic dirt_figure_eight file must be intact");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_workspace_rally_deletion_preserves_classic() {
    let tracks_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tracks");
    if !tracks_dir.exists() {
        return;
    }
    let manager = TrackManager::new(&tracks_dir);

    let classic_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    let rally_tracks = manager.filtered_main_track_choices(ModuleFilter::Rally);

    assert!(classic_tracks.iter().any(|c| c.track_id() == "figure_eight"), "figure_eight must be in classic");
    assert!(classic_tracks.iter().any(|c| c.track_id() == "dirty_oval_speedway"), "dirty_oval_speedway must be in classic");
    assert!(classic_tracks.iter().any(|c| c.track_id() == "dirt_figure_eight"), "dirt_figure_eight must be in classic");

    assert!(!rally_tracks.iter().any(|c| c.track_id() == "dirt_figure_eight"), "dirt_figure_eight must NOT be in rally");
    assert!(!rally_tracks.iter().any(|c| c.track_id() == "dirty_oval_speedway"), "dirty_oval_speedway must NOT be in rally");
}
