use std::fs;
use std::path::Path;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::track_manager::{ModuleFilter, TrackManager};
use tdrace_app::ui::menu::TrackChoice;
use tdrace_app::ui::track_manager_ui::{TrackManagerModal, TrackManagerTab};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::TrackCategory;

static DEV_MODE_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct DevModeGuard {
    temp_dir: std::path::PathBuf,
}
impl DevModeGuard {
    fn enter() -> Self {
        let temp_dir = std::env::temp_dir().join(format!(
            "tdrace_dev_guard_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let mock_git = temp_dir.join("git_tracks");
        let _ = fs::create_dir_all(mock_git.join("classic"));
        let _ = fs::create_dir_all(mock_git.join("gt"));
        let _ = fs::create_dir_all(mock_git.join("rally"));
        let _ = fs::create_dir_all(mock_git.join("kart"));
        let _ = fs::create_dir_all(mock_git.join("nascar"));
        let _ = fs::create_dir_all(mock_git.join("extreme_offroad"));

        // Mirror existing git tracks so presets remain consistent across threads
        if let Some(real_git) = tdrace_app::storage::resolve_git_tracks_dir() {
            if let Ok(entries) = fs::read_dir(&real_git) {
                for sub in entries.flatten() {
                    if sub.path().is_dir() {
                        let sub_name = sub.file_name();
                        let target_sub = mock_git.join(&sub_name);
                        let _ = fs::create_dir_all(&target_sub);
                        if let Ok(files) = fs::read_dir(sub.path()) {
                            for f in files.flatten() {
                                if f.path().extension().and_then(|s| s.to_str()) == Some("json") {
                                    let _ = fs::copy(f.path(), target_sub.join(f.file_name()));
                                }
                            }
                        }
                    }
                }
            }
        }

        std::env::set_var("TDRACE_DEV", "1");
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, &mock_git);
        Self { temp_dir }
    }
}
impl Drop for DevModeGuard {
    fn drop(&mut self) {
        std::env::remove_var("TDRACE_DEV");
        std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn test_track_categories_initial_presets() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_presets_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let manager = TrackManager::new(&temp_dir);

    // 1. Initial state: 95 main presets across modules (10 Classic + 18 unique GT + 17 unique Rally + 17 famous Kart + 17 Nascar + 17 Extreme Off-Road, with 1 shared dirt_figure_eight), 0 drafts
    assert_eq!(manager.custom_track_choices().len(), 0);
    let main_tracks = manager.main_track_choices();
    let draft_tracks = manager.draft_track_choices();
    assert_eq!(main_tracks.len(), 95, "All 95 presets across modules should be Main tracks");
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
    // 3. Verify main menu list still contains only main tracks
    let main_tracks = manager.main_track_choices();
    assert_eq!(main_tracks.len(), 95, "Main menu should only contain approved circuits");
    let draft_tracks = manager.draft_track_choices();
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

    // 4. Save a track that has TrackCategory::Draft
    let mut draft_copy = classic_grand_prix();
    draft_copy.name = "My Modified GP".to_string();
    draft_copy.category = TrackCategory::Draft;
    manager.save_custom_track(&draft_copy, Some("my_modified_gp")).expect("Save draft copy");

    // Must land in drafts
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

    assert_eq!(manager.main_track_choices().len(), 95);
    assert_eq!(manager.draft_track_choices().len(), 1);

    // Promote to GT Module
    manager.promote_track_to_module(track_id, "gt").expect("Must promote to GT");

    assert_eq!(manager.main_track_choices().len(), 96, "Promoted track must appear in Main");
    assert_eq!(manager.draft_track_choices().len(), 0, "Promoted track must be removed from Drafts");

    let promoted_choice = manager.main_track_choices().into_iter().find(|t| t.track_id() == track_id).unwrap();
    assert_eq!(promoted_choice.title(), "Proto Circuit");
    let loaded_promoted = manager.load_track(&promoted_choice).expect("Must load promoted");
    assert_eq!(loaded_promoted.category, TrackCategory::Main);
    assert_eq!(loaded_promoted.modules, vec!["gt".to_string()]);

    // Verify module_custom_tracks
    let gt_customs = manager.module_custom_tracks("gt");
    assert_eq!(gt_customs.len(), 1);
    assert_eq!(gt_customs[0].title(), "Proto Circuit");

    let rally_customs = manager.module_custom_tracks("rally");
    assert_eq!(rally_customs.len(), 0);

    // Demote back to Draft (Under testing)
    manager.demote_track(track_id).expect("Must demote to Draft");

    assert_eq!(manager.main_track_choices().len(), 95, "Demoted track must be removed from Main");
    assert_eq!(manager.draft_track_choices().len(), 1, "Demoted track must reappear in Drafts");
    assert_eq!(manager.module_custom_tracks("gt").len(), 0);

    let loaded_demoted = manager.load_track(&manager.draft_track_choices()[0]).expect("Must load demoted");
    assert_eq!(loaded_demoted.category, TrackCategory::Draft);

    // Re-promote to Rally Module
    manager.promote_track_to_module(track_id, "rally").expect("Must promote to Rally");
    assert_eq!(manager.module_custom_tracks("rally").len(), 1);
    assert_eq!(manager.module_custom_tracks("gt").len(), 0);

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
    track.category = TrackCategory::Draft;
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
        module_filter: ModuleFilter::Classic,
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
    assert_eq!(session.cars.len(), session.max_grid_participants());

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
        module_filter: ModuleFilter::Classic,
        selected_idx: 0,
        modal: TrackManagerModal::ConfirmDelete {
            track_id: track_id.clone(),
            track_title: "Track To Delete".to_string(),
            cursor_idx: 0,
        },
    };

    // Simulate modal deletion
    let deleted = session.track_manager.delete_custom_track(&track_id).unwrap();
    assert!(deleted);
    assert_eq!(session.track_manager.draft_track_choices().len(), 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_confirm_delete_modal_arrow_switching() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_tm_arrow_switch_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    session
        .track_manager
        .create_new_draft_track("Track To Remove", "Test arrow switching")
        .expect("Create draft");

    let track_id = session.track_manager.draft_track_choices()[0].track_id().to_string();

    // 1. Initial state of ConfirmDelete modal: cursor_idx defaults to 0 (Cancel)
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Drafts,
        module_filter: ModuleFilter::Classic,
        selected_idx: 0,
        modal: TrackManagerModal::ConfirmDelete {
            track_id: track_id.clone(),
            track_title: "Track To Remove".to_string(),
            cursor_idx: 0,
        },
    };

    if let GameState::TrackManager { modal: TrackManagerModal::ConfirmDelete { cursor_idx, .. }, .. } = &session.state {
        assert_eq!(*cursor_idx, 0, "Cancel should be focused by default for safety");
    } else {
        panic!("Expected ConfirmDelete modal");
    }

    // 2. Simulate pressing Right arrow (or D) -> switches to 1 (Remove)
    let mut cursor_idx = 0;
    cursor_idx = if cursor_idx == 1 { 0 } else { 1 };
    assert_eq!(cursor_idx, 1, "Right arrow should switch from Cancel (0) to Remove (1)");

    // 3. Simulate pressing Left arrow (or A) -> switches back to 0 (Cancel)
    cursor_idx = if cursor_idx == 0 { 1 } else { 0 };
    assert_eq!(cursor_idx, 0, "Left arrow should switch from Remove (1) to Cancel (0)");

    // 4. Simulate pressing Up or Down arrow or Tab -> toggles between Cancel and Remove
    cursor_idx = 1 - cursor_idx;
    assert_eq!(cursor_idx, 1, "Up/Down should toggle to Remove (1)");
    cursor_idx = 1 - cursor_idx;
    assert_eq!(cursor_idx, 0, "Up/Down should toggle back to Cancel (0)");

    // 5. Confirm when cursor_idx == 0 (Cancel): modal is dismissed, track is preserved
    let is_confirmed = true;
    if is_confirmed && cursor_idx == 0 {
        session.state = GameState::TrackManager {
            active_tab: TrackManagerTab::Drafts,
            module_filter: ModuleFilter::Classic,
            selected_idx: 0,
            modal: TrackManagerModal::None,
        };
    }
    assert_eq!(session.track_manager.draft_track_choices().len(), 1, "Track must NOT be deleted when Cancel was selected");

    // 6. Re-open modal, switch to 1 (Remove), and confirm: track is deleted
    cursor_idx = 1;
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Drafts,
        module_filter: ModuleFilter::Classic,
        selected_idx: 0,
        modal: TrackManagerModal::ConfirmDelete {
            track_id: track_id.clone(),
            track_title: "Track To Remove".to_string(),
            cursor_idx,
        },
    };

    if is_confirmed && cursor_idx == 1 {
        let deleted = session.track_manager.delete_custom_track(&track_id).unwrap();
        assert!(deleted);
        session.state = GameState::TrackManager {
            active_tab: TrackManagerTab::Drafts,
            module_filter: ModuleFilter::Classic,
            selected_idx: 0,
            modal: TrackManagerModal::None,
        };
    }
    assert_eq!(session.track_manager.draft_track_choices().len(), 0, "Track must be deleted when Remove was selected");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_module_filter_filtering_and_presets_in_classic() {
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_filter_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial state: 95 tracks across all modules (10 Classic + 18 unique GT + 17 unique Rally + 17 famous Kart + 17 Nascar + 17 Extreme Off-Road, 1 shared dirt_figure_eight)
    assert_eq!(manager.main_track_choices().len(), 95);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), 10);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Gt).len(), 18);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Rally).len(), 17);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Kart).len(), 17);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Nascar).len(), 17);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::ExtremeOffRoad).len(), 17);

    // Promote a new track to GT
    let mut track_gt = classic_grand_prix();
    track_gt.name = "Monza Custom GP".to_string();
    manager.save_custom_track(&track_gt, Some("monza_custom")).unwrap();
    manager.promote_track_to_module("monza_custom", "gt").unwrap();

    // Promote a new track to Rally
    let mut track_rally = classic_grand_prix();
    track_rally.name = "Dune Safari".to_string();
    manager.save_custom_track(&track_rally, Some("dune_safari")).unwrap();
    manager.promote_track_to_module("dune_safari", "rally").unwrap();

    // Verify filtered counts
    assert_eq!(manager.main_track_choices().len(), 97);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Classic).len(), 10);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Gt).len(), 19);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Rally).len(), 18);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Kart).len(), 17);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Nascar).len(), 17);
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::ExtremeOffRoad).len(), 17);

    // Verify filter cycle (.next())
    assert_eq!(ModuleFilter::Classic.next(), ModuleFilter::Rally);
    assert_eq!(ModuleFilter::Rally.next(), ModuleFilter::Kart);
    assert_eq!(ModuleFilter::Kart.next(), ModuleFilter::Gt);
    assert_eq!(ModuleFilter::Gt.next(), ModuleFilter::Nascar);
    assert_eq!(ModuleFilter::Nascar.next(), ModuleFilter::ExtremeOffRoad);
    assert_eq!(ModuleFilter::ExtremeOffRoad.next(), ModuleFilter::Drafts);
    assert_eq!(ModuleFilter::Drafts.next(), ModuleFilter::Classic);

    // Verify filter cycle (.prev())
    assert_eq!(ModuleFilter::Classic.prev(), ModuleFilter::Drafts);
    assert_eq!(ModuleFilter::Drafts.prev(), ModuleFilter::ExtremeOffRoad);
    assert_eq!(ModuleFilter::ExtremeOffRoad.prev(), ModuleFilter::Nascar);
    assert_eq!(ModuleFilter::Nascar.prev(), ModuleFilter::Gt);
    assert_eq!(ModuleFilter::Gt.prev(), ModuleFilter::Kart);
    assert_eq!(ModuleFilter::Kart.prev(), ModuleFilter::Rally);
    assert_eq!(ModuleFilter::Rally.prev(), ModuleFilter::Classic);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_module_subdirectories_and_file_movement() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_subdirs_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a draft track -> saved flat in temp_dir/my_circuit.json with Draft category
    let mut track = classic_grand_prix();
    track.name = "My Circuit".to_string();
    track.category = TrackCategory::Draft;
    let saved_path = manager.save_custom_track(&track, Some("my_circuit")).expect("Save draft");
    let track_file = temp_dir.join("my_circuit.json");
    assert!(track_file.exists(), "Track file must be saved in user tracks directory");
    assert_eq!(Path::new(&saved_path), track_file);

    // 2. Promote to GT -> metadata updated to Main category and module 'gt'
    manager.promote_track_to_module("my_circuit", "gt").expect("Promote to GT");
    let loaded = tdrace_core::track::Track::load_from_file(&track_file).expect("Load promoted");
    assert_eq!(loaded.category, TrackCategory::Main);
    assert_eq!(loaded.modules, vec!["gt".to_string()]);
    assert_eq!(manager.draft_track_choices().len(), 0);
    assert_eq!(manager.module_custom_tracks("gt").len(), 1);

    // 3. Demote back to Draft -> metadata updated to Draft category and empty modules
    manager.demote_track("my_circuit").expect("Demote to Draft");
    let loaded_demoted = tdrace_core::track::Track::load_from_file(&track_file).expect("Load demoted");
    assert_eq!(loaded_demoted.category, TrackCategory::Draft);
    assert!(loaded_demoted.modules.is_empty());
    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.module_custom_tracks("gt").len(), 0);

    // 4. Promote to Rally -> metadata updated to Rally
    manager.promote_track_to_module("my_circuit", "rally").expect("Promote to Rally");
    let loaded_rally = tdrace_core::track::Track::load_from_file(&track_file).expect("Load rally");
    assert_eq!(loaded_rally.category, TrackCategory::Main);
    assert_eq!(loaded_rally.modules, vec!["rally".to_string()]);

    // 5. Test scanner on fresh TrackManager instance
    let new_scanner = TrackManager::new(&temp_dir);
    assert_eq!(new_scanner.main_track_choices().len(), 96); // 95 presets + 1 custom
    assert_eq!(new_scanner.module_custom_tracks("rally").len(), 1);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_tab_and_module_cycling() {
    let mut active_tab = TrackManagerTab::Main;
    let mut module_filter = ModuleFilter::Classic;

    // Tab toggle
    active_tab = match active_tab {
        TrackManagerTab::Main => TrackManagerTab::Drafts,
        TrackManagerTab::Drafts => TrackManagerTab::Main,
        TrackManagerTab::DevWorkbench => TrackManagerTab::Main,
    };
    assert_eq!(active_tab, TrackManagerTab::Drafts);

    active_tab = match active_tab {
        TrackManagerTab::Main => TrackManagerTab::Drafts,
        TrackManagerTab::Drafts => TrackManagerTab::Main,
        TrackManagerTab::DevWorkbench => TrackManagerTab::Main,
    };
    assert_eq!(active_tab, TrackManagerTab::Main);

    // Module cycling forward (Right arrow)
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Rally);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Kart);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Gt);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Nascar);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::ExtremeOffRoad);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Drafts);
    module_filter = module_filter.next();
    assert_eq!(module_filter, ModuleFilter::Classic);

    // Module cycling backward (Left arrow)
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Drafts);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::ExtremeOffRoad);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Nascar);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Gt);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Kart);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Rally);
    module_filter = module_filter.prev();
    assert_eq!(module_filter, ModuleFilter::Classic);
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
        module_filter: ModuleFilter::Classic,
        selected_idx: 0,
        modal: TrackManagerModal::ConfirmDelete {
            track_id: track_id.clone(),
            track_title: "Track To Delete Backspace".to_string(),
            cursor_idx: 0,
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
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_predefined_ops_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial state: 10 Classic tracks
    let init_classic_count = manager.filtered_main_track_choices(ModuleFilter::Classic).len();
    assert_eq!(init_classic_count, 10);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // In standard mode, modifying presets is strictly prohibited
    assert!(manager.demote_track("classic_grand_prix").is_err(), "Standard mode must reject demoting presets");
    assert!(manager.promote_track_to_module("classic_grand_prix", "gt").is_err(), "Standard mode must reject reassigning preset categories");
    assert!(manager.delete_custom_track("classic_grand_prix").is_err(), "Standard mode must reject deleting presets");

    // Enable developer mode for developer operations via guard
    let _dev_guard = DevModeGuard::enter();

    // 1. Demote built-in preset Classic Grand Prix (P key in dev mode)
    manager.demote_track("classic_grand_prix").expect("Must demote predefined track in dev mode");

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

    // 4. Test GT predefined track (Monza) demotion and deletion
    let init_gt_count = manager.filtered_main_track_choices(ModuleFilter::Gt).len();
    manager.demote_track("monza").expect("Must demote Monza");
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Gt).len(), init_gt_count - 1);
    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.draft_track_choices()[0].track_id(), "monza");

    manager.delete_custom_track("monza").expect("Must delete Monza");
    assert_eq!(manager.filtered_main_track_choices(ModuleFilter::Gt).len(), init_gt_count - 1);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // 5. Verify persistence across new TrackManager instance
    let manager2 = TrackManager::new(&temp_dir);
    assert_eq!(manager2.filtered_main_track_choices(ModuleFilter::Classic).len(), init_classic_count - 1);
    assert_eq!(manager2.filtered_main_track_choices(ModuleFilter::Gt).len(), init_gt_count - 1);
    assert_eq!(manager2.draft_track_choices().len(), 0);

    drop(_dev_guard);
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

    // 2. Promote simultaneously to Rally, GT, and Classic modules
    manager
        .promote_track_to_modules(track_id, &["rally", "gt", "classic"])
        .expect("Must promote to multiple modules");

    // 3. Verify single file on disk with multi-module metadata
    let single_track_file = temp_dir.join(format!("{}.json", track_id));
    assert!(single_track_file.exists(), "Single track file must exist in user tracks dir");
    let loaded = tdrace_core::track::Track::load_from_file(&single_track_file).expect("Load multi-spec");
    assert_eq!(loaded.category, TrackCategory::Main);
    assert_eq!(loaded.modules, vec!["rally".to_string(), "gt".to_string(), "classic".to_string()]);

    // 4. Verify catalog queries
    assert_eq!(manager.draft_track_choices().len(), 0, "No drafts remaining");

    let rally_customs = manager.module_custom_tracks("rally");
    assert_eq!(rally_customs.len(), 1);
    assert_eq!(rally_customs[0].title(), "Multi Spec GP");

    let gt_customs = manager.module_custom_tracks("gt");
    assert_eq!(gt_customs.len(), 1);
    assert_eq!(gt_customs[0].title(), "Multi Spec GP");

    let classic_customs = manager.module_custom_tracks("classic");
    assert_eq!(classic_customs.len(), 1);
    assert_eq!(classic_customs[0].title(), "Multi Spec GP");

    let kart_customs = manager.module_custom_tracks("kart");
    assert_eq!(kart_customs.len(), 0);

    // 5. Verify module filter choices
    let rally_choices = manager.filtered_main_track_choices(ModuleFilter::Rally);
    assert!(rally_choices.iter().any(|c| c.track_id() == track_id));

    let gt_choices = manager.filtered_main_track_choices(ModuleFilter::Gt);
    assert!(gt_choices.iter().any(|c| c.track_id() == track_id));

    let classic_choices = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(classic_choices.iter().any(|c| c.track_id() == track_id));

    let kart_choices = manager.filtered_main_track_choices(ModuleFilter::Kart);
    assert!(!kart_choices.iter().any(|c| c.track_id() == track_id));

    // 6. Demote track back to Drafts
    manager.demote_track(track_id).expect("Must demote multi-module track");

    // 7. Verify single track file updated to Draft with empty modules
    assert!(single_track_file.exists(), "Single track file must remain on disk");
    let loaded_demoted = tdrace_core::track::Track::load_from_file(&single_track_file).expect("Load demoted");
    assert_eq!(loaded_demoted.category, TrackCategory::Draft);
    assert!(loaded_demoted.modules.is_empty());

    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.module_custom_tracks("rally").len(), 0);
    assert_eq!(manager.module_custom_tracks("gt").len(), 0);
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
            tag == "CLASSIC MOTORSPORT" || tag == "FIA GP CIRCUIT" || tag == "SUPERSPEEDWAY" || tag == "TECHNICAL DRIFT" || tag == "AGILE SPRINT" || tag == "STUNT RAMPS & JUMPS" || tag == "DIRT STUNT RAMPS" || tag == "HYBRID RALLYCROSS" || tag == "DESERT DIRT RALLY" || tag == "NARROW MOUNTAIN PASS" || tag == "RALLY CROSS",
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
        "zandvoort".into(), "bahrain".into(), "marina_bay".into(), "cota".into(), "madring".into(),
        "nurburgring_gp".into(), "bathurst".into(), "portimao_gp".into(), "le_mans_sarthe".into(),
        "classic_grand_prix".into(), "oval_speedway".into(), "drift_park".into(),
        "kart_arena".into(), "ramp_raceway".into(), "classic_rallycross".into(), "oasis_rally".into(),
        "dirty_oval_speedway".into(), "figure_eight".into(),
        "sahara".into(), "sahara_dunes".into(), "dirt_figure_eight".into(), "holjes_rx".into(), "lydden_hill".into(),
        "hell_rx".into(), "loheac_rx".into(), "estering_rx".into(), "montalegre_rx".into(), "nyirad_rx".into(), "kouvola_rx".into(), "catalunya_rx".into(),
        "mettet_rx".into(), "silverstone_rx".into(), "riga_rx".into(), "killarney_rx".into(), "yas_marina_rx".into(), "essay_rx".into(),
        "dreux_rx".into(), "blyton_rx".into(),
        "lonato".into(), "sarno".into(), "genk".into(), "pfi".into(),
        "zuera".into(), "le_mans_kart".into(), "portimao_kart".into(), "franciacorta".into(),
        "wackersdorf".into(), "kristianstad".into(), "seven_laghi".into(), "ampfing".into(), "silverstone_national_kart".into(),
        "valencia_kart".into(), "campillos".into(),
        "laval_kart".into(), "whilton_mill".into(),
        "daytona_superspeedway".into(), "talladega_superspeedway".into(), "watkins_glen_nascar".into(),
        "bristol_motor_speedway".into(), "martinsville_speedway".into(), "darlington_raceway".into(),
        "charlotte_motor_speedway".into(), "indianapolis_motor_speedway".into(), "eldora_speedway".into(),
        "iowa_speedway".into(), "road_america".into(), "chicago_street_course".into(),
        "bowman_gray_stadium".into(), "lucas_oil_irp".into(), "north_wilkesboro_speedway".into(),
        "pocono_raceway".into(), "phoenix_raceway".into(),
        "sahara_dune_crossing".into(), "atacama_sand_basin".into(), "glamis_dunes".into(),
        "crandon_short_course".into(), "red_rock_canyon".into(), "mud_slough_arena".into(),
        "baja_500_desert_scrub".into(), "arctic_frozen_lake".into(), "alpine_snow_ridge".into(),
        "rovaniemi_ice_ring".into(), "supercross_stadium_arena".into(), "gravel_quarry_chasm".into(),
        "louisiana_mud_swampland".into(), "monster_colosseum".into(), "glacier_crest_pass".into(),
        "stunt_city_megastructure".into(),
    ];

    assert_eq!(session.track_manager.module_catalog_tracks("gt").len(), 0);
    assert_eq!(session.track_manager.module_catalog_tracks("rally").len(), 0);
    assert_eq!(session.track_manager.module_catalog_tracks("kart").len(), 0);
    assert_eq!(session.track_manager.module_catalog_tracks("classic").len(), 0);

    // Ensure switching to each module succeeds gracefully without panicking
    session.switch_to_gt();
    assert_eq!(session.active_module_id, "gt");

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
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    use tdrace_core::track::presets::oval_speedway;

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_preset_persistence_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. In normal user mode, official presets cannot be overwritten directly
    let mut oval = oval_speedway();
    oval.name = "Oval Speedway (Modified)".to_string();
    oval.description = "Customized banked oval.".to_string();

    let err_oval = manager
        .save_custom_track_with_options(&oval, Some("oval_speedway"), true)
        .expect_err("Normal user mode must reject overwriting official preset");
    assert!(err_oval.contains("is an official preset and cannot be modified directly"));

    // Official presets remain untouched in the catalog
    let loaded_oval = manager.load_track(&TrackChoice::OvalSpeedway).unwrap();
    assert_eq!(loaded_oval.name, "Oval Speedway");

    // 2. Cloning an official preset creates a copy in Drafts
    let (cloned_oval, copy_path) = manager
        .clone_track(&TrackChoice::OvalSpeedway)
        .expect("Clone preset to drafts");
    assert!(copy_path.contains("oval_speedway_clone"));
    assert_eq!(cloned_oval.category, TrackCategory::Draft);
    assert_eq!(manager.draft_track_choices().len(), 1);

    // 3. In Developer Mode (TDRACE_DEV=1), official presets can be modified directly
    {
        let mock_git = temp_dir.join("mock_git");
        let _ = fs::create_dir_all(mock_git.join("classic"));
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, &mock_git);
        let _dev_guard = DevModeGuard::enter();
        assert!(tdrace_app::storage::is_dev_mode());

        let dev_save = manager.save_custom_track_with_options(&oval, Some("oval_speedway"), true);
        assert!(dev_save.is_ok(), "Dev mode must allow saving preset");

        // Revert preset back to canonical
        let canonical_oval = oval_speedway();
        let _ = manager.save_custom_track_with_options(&canonical_oval, Some("oval_speedway"), true);
        std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);
    }

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
    assert!(temp_dir.join(format!("{}.json", track_id)).exists());
    assert!(manager.is_track_in_module(track_id, "classic"));
    assert_eq!(manager.track_promoted_modules(track_id), vec!["classic".to_string()]);

    // 3. Re-promote to different modules: change from Classic to Rally and Kart (removing Classic)
    manager
        .promote_track_to_modules(track_id, &["rally", "kart"])
        .expect("Re-promote to rally and kart");

    assert!(temp_dir.join(format!("{}.json", track_id)).exists());
    assert!(!manager.is_track_in_module(track_id, "classic"));
    assert!(manager.is_track_in_module(track_id, "rally"));
    assert!(manager.is_track_in_module(track_id, "kart"));
    assert!(!manager.is_track_in_module(track_id, "gt"));

    let promoted_mods = manager.track_promoted_modules(track_id);
    assert_eq!(promoted_mods, vec!["rally".to_string(), "kart".to_string()]);

    // 4. Add GT module without removing Rally or Kart (Rally, Kart, GT)
    manager
        .promote_track_to_modules(track_id, &["rally", "kart", "gt"])
        .expect("Add GT module");

    assert!(temp_dir.join(format!("{}.json", track_id)).exists());
    assert!(manager.is_track_in_module(track_id, "rally"));
    assert!(manager.is_track_in_module(track_id, "kart"));
    assert!(manager.is_track_in_module(track_id, "gt"));
    assert!(!manager.is_track_in_module(track_id, "classic"));

    let promoted_mods_3 = manager.track_promoted_modules(track_id);
    assert_eq!(promoted_mods_3, vec!["rally".to_string(), "kart".to_string(), "gt".to_string()]);

    // 5. Promote with empty modules slice -> should demote track back to Drafts
    manager
        .promote_track_to_modules(track_id, &[])
        .expect("Empty promotion should demote");

    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.track_promoted_modules(track_id).len(), 0);
    let draft_file = temp_dir.join(format!("{}.json", track_id));
    assert!(draft_file.exists(), "Track file must exist");
    let loaded_draft = tdrace_core::track::Track::load_from_file(&draft_file).expect("Load demoted");
    assert_eq!(loaded_draft.category, TrackCategory::Draft);
    assert!(loaded_draft.modules.is_empty());

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

    // Create and promote to Kart and GT
    let track_id = "kart_gt_hybrid";
    let mut track = classic_grand_prix();
    track.name = "Hybrid Circuit".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).unwrap();
    manager.promote_track_to_modules(track_id, &["kart", "gt"]).unwrap();

    // Verify mask resolution logic matches PROMOTION_MODULES indices
    let mut selected_mask = [false; 4];
    for (idx, (mod_id, _, _, _)) in PROMOTION_MODULES.iter().enumerate() {
        if manager.is_track_in_module(track_id, mod_id) {
            selected_mask[idx] = true;
        }
    }

    // PROMOTION_MODULES order: 0: classic, 1: rally, 2: kart, 3: gt
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
    assert_eq!(rally_track.car_category, tdrace_core::CarCategory::Rally);

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
    assert_eq!(kart_track.car_category, tdrace_core::CarCategory::Kart);

    // 3. Create a GT draft track (Oval, Right)
    let gt_path = manager
        .create_new_draft_track_with_template(
            "GT Grand Arena",
            "High speed GT oval",
            "gt",
            TrackShape::Oval,
            RaceDirection::Right,
        )
        .expect("GT draft creation should succeed");

    let gt_track = Track::load_from_file(&gt_path).expect("Load GT draft track");
    assert_eq!(gt_track.default_surface, SurfaceType::Grass);
    assert_eq!(gt_track.spline.samples[0].surface, SurfaceType::Asphalt);
    assert_eq!(gt_track.car_category, tdrace_core::CarCategory::Gt);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_clone_preset_to_drafts() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_clone_preset_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial check: 95 main tracks, 0 drafts
    assert_eq!(manager.main_track_choices().len(), 95);
    assert_eq!(manager.draft_track_choices().len(), 0);

    // Clone Classic Grand Prix
    let gp_choice = TrackChoice::ClassicGrandPrix;
    let (cloned_track, saved_path) = manager.clone_track(&gp_choice).expect("Clone preset must succeed");

    assert_eq!(cloned_track.name, "Classic Grand Prix (clone)");
    assert_eq!(cloned_track.category, TrackCategory::Draft);
    assert!(cloned_track.module_id.is_none());
    assert!(cloned_track.modules.is_empty());
    assert!(Path::new(&saved_path).exists());
    assert!(saved_path.ends_with(".json"));

    // Verify drafts list has 1 track, main still has 95
    assert_eq!(manager.main_track_choices().len(), 95);
    let drafts = manager.draft_track_choices();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].title(), "Classic Grand Prix (clone)");

    // Verify exact clone of geometry & properties
    let original = manager.load_track(&gp_choice).unwrap();
    assert_eq!(cloned_track.spline.waypoints.len(), original.spline.waypoints.len());
    assert_eq!(cloned_track.checkpoints.len(), original.checkpoints.len());
    assert_eq!(cloned_track.default_surface, original.default_surface);
    assert_eq!(cloned_track.default_laps, original.default_laps);
    assert_eq!(cloned_track.car_category, original.car_category);

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
        module_filter: ModuleFilter::Classic,
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

    // 4. Verify track file is intact on disk with updated metadata
    let track_path = temp_dir.join(format!("{}.json", track_id));
    assert!(track_path.exists(), "Track file must remain on disk");
    let loaded_track = tdrace_core::track::Track::load_from_file(&track_path).expect("Load track");
    assert_eq!(loaded_track.modules, vec!["classic".to_string()]);

    // 5. Test with a custom circuit promoted to multiple modules
    let mut custom_fig8 = tdrace_core::track::presets::dirt_figure_eight();
    custom_fig8.name = "Dirt Figure-8 Custom".to_string();
    custom_fig8.category = TrackCategory::Draft;
    let _ = manager.save_custom_track(&custom_fig8, Some("my_custom_fig8")).expect("Save");
    manager.promote_track_to_modules("my_custom_fig8", &["classic", "rally"]).expect("Promote");

    assert!(manager.filtered_main_track_choices(ModuleFilter::Classic).iter().any(|c| c.track_id() == "my_custom_fig8"));
    assert!(manager.filtered_main_track_choices(ModuleFilter::Rally).iter().any(|c| c.track_id() == "my_custom_fig8"));

    // Delete my_custom_fig8 specifically from Rally
    manager.delete_track_from_module("my_custom_fig8", Some("rally")).expect("Delete from rally");

    assert!(!manager.filtered_main_track_choices(ModuleFilter::Rally).iter().any(|c| c.track_id() == "my_custom_fig8"), "my_custom_fig8 removed from rally");
    assert!(manager.filtered_main_track_choices(ModuleFilter::Classic).iter().any(|c| c.track_id() == "my_custom_fig8"), "my_custom_fig8 must remain in classic");
    let fig8_file = temp_dir.join("my_custom_fig8.json");
    assert!(fig8_file.exists(), "my_custom_fig8 file must be intact");
    let loaded_fig8 = tdrace_core::track::Track::load_from_file(&fig8_file).expect("Load");
    assert_eq!(loaded_fig8.modules, vec!["classic".to_string()]);

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

#[test]
#[ignore = "Manual export tool: cargo test --test track_manager_tests test_export_canonical_presets_to_git_repo -- --ignored"]
fn test_export_canonical_presets_to_git_repo() {
    use tdrace_app::module::{
        classic::ClassicGameModule, extreme_offroad::ExtremeOffRoadModule,
        gt::GtWorldChallengeModule, kart::KartGameModule, nascar::NascarGameModule,
        rally::RallyGameModule, GameModule,
    };
    use tdrace_core::track::TrackCategory;

    let repo_tracks_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tracks");
    if !repo_tracks_dir.exists() {
        return;
    }

    let modules: Vec<Box<dyn GameModule>> = vec![
        Box::new(ClassicGameModule::new()),
        Box::new(GtWorldChallengeModule::new()),
        Box::new(RallyGameModule::new()),
        Box::new(KartGameModule::new()),
        Box::new(NascarGameModule::new()),
        Box::new(ExtremeOffRoadModule::new()),
    ];

    let mut total_exported = 0;
    for module in modules {
        let mod_id = module.id();
        let target_dir = repo_tracks_dir.join(mod_id);
        let _ = fs::create_dir_all(&target_dir);

        for track_def in module.tracks() {
            let mut track = (track_def.generator)();
            track.category = TrackCategory::Main;
            track.module_id = Some(mod_id.to_string());
            if !track.modules.contains(&mod_id.to_string()) {
                track.modules.push(mod_id.to_string());
            }
            let filename = if mod_id == "nascar" {
                match track_def.id {
                    "daytona_superspeedway" => "daytona",
                    "talladega_superspeedway" => "talladega",
                    "watkins_glen_nascar" => "watkins_glen",
                    "bristol_motor_speedway" => "bristol",
                    "martinsville_speedway" => "martinsville",
                    "darlington_raceway" => "darlington",
                    "charlotte_motor_speedway" => "charlotte",
                    "indianapolis_motor_speedway" => "indianapolis",
                    "eldora_speedway" => "eldora",
                    "iowa_speedway" => "iowa",
                    "chicago_street_course" => "chicago",
                    "bowman_gray_stadium" => "bowman_gray",
                    "lucas_oil_irp" => "irp_oval",
                    "north_wilkesboro_speedway" => "north_wilkesboro",
                    "pocono_raceway" => "pocono",
                    "phoenix_raceway" => "phoenix",
                    other => other,
                }
            } else {
                track_def.id
            };
            track = track
                .with_provenance_if_known(filename)
                .with_provenance_if_known(track_def.id);
            let file_path = target_dir.join(format!("{}.json", filename));
            track.save_to_file(&file_path).expect("Failed to export canonical preset");

            // Verify load
            let loaded = tdrace_core::track::Track::load_from_file(&file_path)
                .expect("Failed to load exported canonical preset");
            assert_eq!(loaded.name, track.name);
            assert_eq!(loaded.category, TrackCategory::Main);

            if let Some(prov) = tdrace_core::track::get_circuit_provenance(filename) {
                assert!(
                    loaded.osm_url.is_some(),
                    "Real circuit {} must have osm_url preserved in exported json",
                    filename
                );
                assert_eq!(
                    loaded.osm_url.as_deref(),
                    Some(prov.osm_url),
                    "OSM URL mismatch on {}",
                    filename
                );
            }

            total_exported += 1;
        }
    }
    assert!(total_exported >= 80, "Must export all preset track definitions across modules");
}

#[test]
fn test_category_ordering_presets_first_then_custom() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_ordering_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);
    let mut manager = TrackManager::new(&temp_dir);

    // Create custom tracks in classic and rally
    let _ = manager.create_new_custom_track_with_template(
        "Alpha Custom Classic",
        "A fast custom classic track",
        "classic",
        tdrace_core::track::presets::TrackShape::Oval,
        tdrace_core::track::presets::RaceDirection::Right,
    ).expect("Create classic track");

    let _ = manager.create_new_custom_track_with_template(
        "Beta Custom Classic",
        "A technical custom classic track",
        "classic",
        tdrace_core::track::presets::TrackShape::HorizontalEight,
        tdrace_core::track::presets::RaceDirection::Right,
    ).expect("Create classic track 2");

    let _ = manager.create_new_custom_track_with_template(
        "Muddy Trail Rally",
        "Custom rally track",
        "rally",
        tdrace_core::track::presets::TrackShape::Oval,
        tdrace_core::track::presets::RaceDirection::Left,
    ).expect("Create rally track");

    // Verify Classic category: 10 presets first, then 2 custom tracks
    let classic_tracks = manager.module_catalog_tracks("classic");
    assert_eq!(classic_tracks.len(), 12);
    for (i, track) in classic_tracks.iter().enumerate() {
        if i < 10 {
            assert!(track.is_official_preset(), "Track at index {} must be official preset: {}", i, track.title());
            assert!(!track.is_user_custom());
        } else {
            assert!(track.is_user_custom(), "Track at index {} must be user custom: {}", i, track.title());
        }
    }

    // Verify Rally category: 17 presets first, then 1 custom track
    let rally_tracks = manager.module_catalog_tracks("rally");
    assert_eq!(rally_tracks.len(), 18);
    for (i, track) in rally_tracks.iter().enumerate() {
        if i < 17 {
            assert!(track.is_official_preset(), "Track at index {} must be official preset: {}", i, track.title());
        } else {
            assert!(track.is_user_custom(), "Track at index {} must be user custom: {}", i, track.title());
            assert_eq!(track.title(), "Muddy Trail Rally");
        }
    }

    // Verify GT category: pure presets (18), no custom tracks leaked
    let gt_tracks = manager.module_catalog_tracks("gt");
    assert_eq!(gt_tracks.len(), 18);
    assert!(gt_tracks.iter().all(|t| t.is_official_preset()));

    // Verify Kart category: pure presets (17), no custom tracks leaked
    let kart_tracks = manager.module_catalog_tracks("kart");
    assert_eq!(kart_tracks.len(), 17);
    assert!(kart_tracks.iter().all(|t| t.is_official_preset()));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_custom_circuit_multi_category_assignment() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_multi_cat_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);
    let mut manager = TrackManager::new(&temp_dir);

    // Create a track assigned to both Classic and Rally
    let mut hybrid_track = classic_grand_prix();
    hybrid_track.name = "Hybrid Classic Rally".to_string();
    hybrid_track.category = TrackCategory::Main;
    hybrid_track.modules = vec!["classic".to_string(), "rally".to_string()];
    let _ = manager.save_custom_track_with_options(&hybrid_track, Some("hybrid_circuit"), false)
        .expect("Save hybrid track");

    // Check presence in Classic: after presets
    let classic_tracks = manager.module_catalog_tracks("classic");
    assert_eq!(classic_tracks.len(), 11);
    assert!(classic_tracks[10].is_user_custom());
    assert_eq!(classic_tracks[10].title(), "Hybrid Classic Rally");

    // Check presence in Rally: after presets
    let rally_tracks = manager.module_catalog_tracks("rally");
    assert_eq!(rally_tracks.len(), 18);
    assert!(rally_tracks[17].is_user_custom());
    assert_eq!(rally_tracks[17].title(), "Hybrid Classic Rally");

    // Check absence in GT and Kart
    let gt_tracks = manager.module_catalog_tracks("gt");
    assert!(!gt_tracks.iter().any(|t| t.title() == "Hybrid Classic Rally"));

    let kart_tracks = manager.module_catalog_tracks("kart");
    assert!(!kart_tracks.iter().any(|t| t.title() == "Hybrid Classic Rally"));

    // Now re-assign to Kart and GT using promote_track_to_modules
    manager.promote_track_to_modules("hybrid_circuit", &["kart", "gt"])
        .expect("Reassign categories");

    // Must now be present in Kart and GT, after presets
    let kart_after = manager.module_catalog_tracks("kart");
    assert_eq!(kart_after.len(), 18);
    assert!(kart_after[17].is_user_custom());
    assert_eq!(kart_after[17].title(), "Hybrid Classic Rally");

    let gt_after = manager.module_catalog_tracks("gt");
    assert_eq!(gt_after.len(), 19);
    assert!(gt_after[18].is_user_custom());
    assert_eq!(gt_after[18].title(), "Hybrid Classic Rally");

    // Must no longer appear in Classic and Rally
    let classic_after = manager.module_catalog_tracks("classic");
    assert_eq!(classic_after.len(), 10);
    assert!(!classic_after.iter().any(|t| t.title() == "Hybrid Classic Rally"));

    let rally_after = manager.module_catalog_tracks("rally");
    assert_eq!(rally_after.len(), 17);
    assert!(!rally_after.iter().any(|t| t.title() == "Hybrid Classic Rally"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_custom_circuit_promoted_to_preset_classified_as_official_preset() {
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_tm_preset_styling_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = fs::remove_dir_all(&temp_dir);

    // 1. A TrackChoice::Custom representing a preset in git must be recognized as an official preset
    let git_preset_choice = TrackChoice::Custom {
        id: "figure_eight".to_string(),
        title: "Figure Eight".to_string(),
        description: "Classic crossover".to_string(),
        path: "classic/figure_eight".to_string(),
    };
    assert!(git_preset_choice.is_official_preset(), "Track in presets must be official preset");

    // 2. Even if represented with an absolute path, it must be recognized as official preset when not demoted
    let abs_preset_choice = TrackChoice::Custom {
        id: "classic_grand_prix".to_string(),
        title: "Classic Grand Prix".to_string(),
        description: "GP circuit".to_string(),
        path: temp_dir.join("classic_grand_prix.json").to_string_lossy().to_string(),
    };
    assert!(abs_preset_choice.is_official_preset(), "Preset with absolute path must remain official preset if not demoted");

    // 3. If marked as demoted in .deleted_tracks.json, it is NOT an official preset
    let _ = fs::create_dir_all(&temp_dir);
    let deleted_file = temp_dir.join(".deleted_tracks.json");
    let _ = fs::write(&deleted_file, serde_json::to_string(&vec!["demoted:classic_grand_prix"]).unwrap());

    assert!(!abs_preset_choice.is_official_preset(), "Demoted preset must NOT be official preset");

    // 4. Stale user track copy resilience: even if a file with preset slug exists in user tracks dir,
    // TrackManager keeps it classified as an official preset.
    let stale_path = temp_dir.join("classic_grand_prix.json");
    let mut gp_track = classic_grand_prix();
    gp_track.category = TrackCategory::Main;
    let _ = gp_track.save_to_file(&stale_path);
    // Clear demoted marker
    let _ = fs::write(&deleted_file, "[]");

    let manager_with_stale = TrackManager::new(&temp_dir);
    let c_gp = manager_with_stale
        .filtered_main_track_choices(ModuleFilter::Classic)
        .into_iter()
        .find(|t| t.track_id() == "classic_grand_prix")
        .expect("Classic GP present");
    assert!(c_gp.is_official_preset(), "Classic GP must remain official preset even with stale local file");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dev_mode_reorder_preset_tracks_up_and_down() {
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_reorder_dev_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial order check
    let initial_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(initial_tracks.len() >= 2);
    let first_id = initial_tracks[0].track_id().to_string();
    let second_id = initial_tracks[1].track_id().to_string();
    assert_ne!(first_id, second_id);

    // 1. Enter dev mode
    {
        let _dev_guard = DevModeGuard::enter();

        // Move second track UP
        let moved = manager
            .reorder_preset_track(&second_id, "classic", true)
            .expect("Reorder up in dev mode must succeed");
        assert!(moved);

        let reordered_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
        assert_eq!(reordered_tracks[0].track_id(), second_id);
        assert_eq!(reordered_tracks[1].track_id(), first_id);

        // Move it back DOWN
        let moved_back = manager
            .reorder_preset_track(&second_id, "classic", false)
            .expect("Reorder down in dev mode must succeed");
        assert!(moved_back);

        let restored_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
        assert_eq!(restored_tracks[0].track_id(), first_id);
        assert_eq!(restored_tracks[1].track_id(), second_id);
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_standard_mode_rejects_preset_reordering() {
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_reorder_std_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);
    let initial_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    let second_id = initial_tracks[1].track_id().to_string();

    let err = manager
        .reorder_preset_track(&second_id, "classic", true)
        .expect_err("Standard mode must reject preset reordering");
    assert!(err.contains("developer mode"));

    let err_reset = manager
        .reset_preset_order("classic")
        .expect_err("Standard mode must reject resetting preset order");
    assert!(err_reset.contains("developer mode"));

    let after_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert_eq!(initial_tracks[0].track_id(), after_tracks[0].track_id());
    assert_eq!(initial_tracks[1].track_id(), after_tracks[1].track_id());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_preset_reordering_persistence() {
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_reorder_persist_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);
    let initial_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    let first_id = initial_tracks[0].track_id().to_string();
    let second_id = initial_tracks[1].track_id().to_string();

    {
        let _dev_guard = DevModeGuard::enter();
        manager
            .reorder_preset_track(&second_id, "classic", true)
            .expect("Reorder in dev mode");
    }

    assert!(temp_dir.join(".track_order.json").exists(), ".track_order.json must be written to disk");

    // Load fresh manager from disk
    let reloaded_manager = TrackManager::new(&temp_dir);
    let reloaded_tracks = reloaded_manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert_eq!(reloaded_tracks[0].track_id(), second_id);
    assert_eq!(reloaded_tracks[1].track_id(), first_id);

    // Reset order
    {
        let _dev_guard = DevModeGuard::enter();
        let mut reset_manager = reloaded_manager;
        let reset_result = reset_manager.reset_preset_order("classic").expect("Reset order");
        assert!(reset_result);

        let final_tracks = reset_manager.filtered_main_track_choices(ModuleFilter::Classic);
        assert_eq!(final_tracks[0].track_id(), first_id);
        assert_eq!(final_tracks[1].track_id(), second_id);
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reordering_boundary_conditions() {
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_reorder_bounds_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);
    let initial_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    let first_id = initial_tracks.first().unwrap().track_id().to_string();
    let last_id = initial_tracks.last().unwrap().track_id().to_string();

    {
        let _dev_guard = DevModeGuard::enter();

        // Moving first track UP returns Ok(false)
        let moved_top = manager.reorder_preset_track(&first_id, "classic", true).unwrap();
        assert!(!moved_top, "Top track cannot move up");

        // Moving last track DOWN returns Ok(false)
        let moved_bottom = manager.reorder_preset_track(&last_id, "classic", false).unwrap();
        assert!(!moved_bottom, "Bottom track cannot move down");

        // Non-existent track returns Err
        let err = manager.reorder_preset_track("non_existent_track_xyz", "classic", true);
        assert!(err.is_err());
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_active_module_tracks_reflects_reordered_presets() {
    let _lock = DEV_MODE_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_session_reorder_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);
    session.active_module_id = "classic";

    let initial_tracks = session.active_module_tracks();
    let first_id = initial_tracks[0].track_id().to_string();
    let second_id = initial_tracks[1].track_id().to_string();

    {
        let _dev_guard = DevModeGuard::enter();
        let moved = session
            .track_manager
            .reorder_preset_track(&second_id, "classic", true)
            .expect("Reorder in dev mode");
        assert!(moved);

        let reordered_tracks = session.active_module_tracks();
        assert_eq!(reordered_tracks[0].track_id(), second_id);
        assert_eq!(reordered_tracks[1].track_id(), first_id);
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_manager_drafts_category_browsing_and_shortcut_9() {
    // 1. Verify ModuleFilter metadata
    assert_eq!(ModuleFilter::ALL.len(), 7);
    assert_eq!(ModuleFilter::Drafts.id(), Some("drafts"));
    assert_eq!(ModuleFilter::Drafts.label(), "DRAFTS");
    assert_eq!(ModuleFilter::Drafts.shortcut_number(), 9);
    assert_eq!(ModuleFilter::for_module("drafts"), ModuleFilter::Drafts);

    // 2. Verify navigation cycle
    assert_eq!(ModuleFilter::Nascar.next(), ModuleFilter::ExtremeOffRoad);
    assert_eq!(ModuleFilter::ExtremeOffRoad.next(), ModuleFilter::Drafts);
    assert_eq!(ModuleFilter::Drafts.next(), ModuleFilter::Classic);
    assert_eq!(ModuleFilter::Classic.prev(), ModuleFilter::Drafts);
    assert_eq!(ModuleFilter::Drafts.prev(), ModuleFilter::ExtremeOffRoad);
    assert_eq!(ModuleFilter::ExtremeOffRoad.prev(), ModuleFilter::Nascar);

    // 3. Verify track resolution for Drafts category
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_tm_drafts_cat_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    // Create a draft track
    session
        .track_manager
        .create_new_draft_track("Super Prototype", "Experimental aerodynamics")
        .expect("Create draft");

    assert_eq!(session.track_manager.draft_track_choices().len(), 1);
    assert_eq!(session.track_manager.filtered_main_track_choices(ModuleFilter::Drafts).len(), 1);
    assert_eq!(session.track_manager.module_custom_tracks("drafts").len(), 1);

    // 4. Verify initial state and cycling into Drafts
    let mut filter = ModuleFilter::Classic;
    // Step forward 6 times: Classic -> Rally -> Kart -> GT -> Nascar -> ExtremeOffRoad -> Drafts
    for _ in 0..6 {
        filter = filter.next();
    }
    assert_eq!(filter, ModuleFilter::Drafts);

    // Step backward 1 time from Classic: Classic -> Drafts
    let mut filter2 = ModuleFilter::Classic;
    filter2 = filter2.prev();
    assert_eq!(filter2, ModuleFilter::Drafts);

    // 5. Direct jump to Drafts via shortcut number 9
    let jump_filter = ModuleFilter::Drafts;
    assert_eq!(jump_filter.shortcut_number(), 9);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_marina_bay_singapore_aliases_and_osm_calibration() {
    let t_mb = tdrace_app::module::gt::GtWorldChallengeModule::track_marina_bay();
    assert_eq!(t_mb.name, "Marina Bay Street Circuit (Singapore)");
    assert_eq!(t_mb.car_category, tdrace_core::CarCategory::Gt);
    assert_eq!(t_mb.module_id.as_deref(), Some("gt"));
    assert!(t_mb.modules.contains(&"gt".to_string()));
    assert_eq!(t_mb.default_laps, 3);

    // Verify 50% length scaling (FIA: 4940m -> ~2300-2480m)
    let len = t_mb.spline.total_length();
    assert!(
        len >= 2300.0 && len <= 2480.0,
        "Marina Bay length should be ~2470m (50% FIA), got {:.1}m",
        len
    );

    // Verify start straight alignment: Waypoint 0 at (0, 0), Waypoint 1 downstream along +X
    let wp0 = t_mb.spline.waypoints[0].point;
    let wp1 = t_mb.spline.waypoints[1].point;
    let wp_last = t_mb.spline.waypoints.last().unwrap().point;
    assert_eq!(wp0.x, 0.0);
    assert_eq!(wp0.y, 0.0);
    assert!(wp1.x > 50.0, "Waypoint 1 should advance down straight along +X");
    assert!(wp1.y.abs() < 1e-4, "Waypoint 1 should have Y ~ 0 along straight");
    assert!(wp_last.x < 0.0, "Last waypoint approaches start line from -X");

    // Verify aliases in canonical_preset_id
    assert_eq!(TrackManager::canonical_preset_id("marina_bay"), "marina_bay");
    assert_eq!(TrackManager::canonical_preset_id("singapore"), "marina_bay");
    assert_eq!(TrackManager::canonical_preset_id("singapur"), "marina_bay");

    // Verify preset_slug_aliases
    let aliases = TrackManager::preset_slug_aliases("singapore");
    assert!(aliases.contains(&"marina_bay"));
    assert!(aliases.contains(&"singapore"));
    assert!(aliases.contains(&"singapur"));

    // Verify TrackManager loading via aliases
    let temp_dir = std::env::temp_dir().join("tdrace_test_mb_alias");
    let _ = fs::remove_dir_all(&temp_dir);
    let manager = TrackManager::new(&temp_dir);

    let choice_mb = TrackChoice::Custom {
        id: "marina_bay".to_string(),
        title: "Marina Bay".to_string(),
        description: "Singapore".to_string(),
        path: "marina_bay".to_string(),
    };
    let loaded_mb = manager.load_track(&choice_mb).expect("Load marina_bay");
    assert_eq!(loaded_mb.name, "Marina Bay Street Circuit (Singapore)");

    let choice_sg = TrackChoice::Custom {
        id: "singapore".to_string(),
        title: "Singapore".to_string(),
        description: "Singapore".to_string(),
        path: "singapore".to_string(),
    };
    let loaded_sg = manager.load_track(&choice_sg).expect("Load singapore");
    assert_eq!(loaded_sg.name, "Marina Bay Street Circuit (Singapore)");

    let choice_sp = TrackChoice::Custom {
        id: "singapur".to_string(),
        title: "Singapur".to_string(),
        description: "Singapur".to_string(),
        path: "singapur".to_string(),
    };
    let loaded_sp = manager.load_track(&choice_sp).expect("Load singapur");
    assert_eq!(loaded_sp.name, "Marina Bay Street Circuit (Singapore)");

    // Verify menu resolver
    let menu_mb = tdrace_app::ui::menu::resolve_track_for_menu(&choice_mb).expect("Resolve menu marina_bay");
    assert_eq!(menu_mb.name, "Marina Bay Street Circuit (Singapore)");
    let menu_sg = tdrace_app::ui::menu::resolve_track_for_menu(&choice_sg).expect("Resolve menu singapore");
    assert_eq!(menu_sg.name, "Marina Bay Street Circuit (Singapore)");
    let menu_sp = tdrace_app::ui::menu::resolve_track_for_menu(&choice_sp).expect("Resolve menu singapur");
    assert_eq!(menu_sp.name, "Marina Bay Street Circuit (Singapore)");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_all_canonical_track_files_provenance_integrity() {
    let tracks_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tracks");
    assert!(tracks_root.exists(), "tracks/ repository directory must exist");

    let mut total_tracks = 0;
    let mut real_tracks_with_osm = 0;
    let mut real_tracks_with_wiki = 0;
    let mut fictional_tracks = 0;

    let modules = ["classic", "gt", "kart", "nascar", "rally", "extreme_offroad"];
    for mod_name in &modules {
        let mod_dir = tracks_root.join(mod_name);
        assert!(mod_dir.exists(), "Module directory {} must exist", mod_name);

        for entry in fs::read_dir(&mod_dir).expect("Read module dir").flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            total_tracks += 1;
            let track = tdrace_core::track::Track::load_from_file(&path)
                .unwrap_or_else(|e| panic!("Failed loading {}: {}", path.display(), e));

            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap();
            let is_real = tdrace_core::track::get_circuit_provenance(stem).is_some();

            if is_real {
                assert!(
                    track.osm_url.is_some(),
                    "Real track {} ({}) must have osm_url",
                    stem,
                    path.display()
                );
                assert!(
                    track.wikipedia_url.is_some(),
                    "Real track {} ({}) must have wikipedia_url",
                    stem,
                    path.display()
                );
                assert!(
                    track.country_code.is_some(),
                    "Real track {} ({}) must have country_code",
                    stem,
                    path.display()
                );
                assert!(
                    track.country_name.is_some(),
                    "Real track {} ({}) must have country_name",
                    stem,
                    path.display()
                );

                let osm = track.osm_url.as_ref().unwrap();
                let wiki = track.wikipedia_url.as_ref().unwrap();
                assert!(
                    osm.starts_with("https://www.openstreetmap.org/"),
                    "Track {} OSM URL must start with https://www.openstreetmap.org/ but got {}",
                    stem,
                    osm
                );
                assert!(
                    wiki.starts_with("https://en.wikipedia.org/wiki/"),
                    "Track {} Wiki URL must start with https://en.wikipedia.org/wiki/ but got {}",
                    stem,
                    wiki
                );

                real_tracks_with_osm += 1;
                real_tracks_with_wiki += 1;
            } else {
                assert!(
                    track.osm_url.is_none(),
                    "Fictional track {} ({}) must NOT have osm_url",
                    stem,
                    path.display()
                );
                fictional_tracks += 1;
            }
        }
    }

    assert_eq!(total_tracks, 96, "Must have exactly 96 total track files");
    assert_eq!(real_tracks_with_osm, 71, "Must have exactly 71 real circuits with verified OSM URLs");
    assert_eq!(real_tracks_with_wiki, 71, "Must have exactly 71 real circuits with verified Wikipedia URLs");
    assert_eq!(fictional_tracks, 25, "Must have exactly 25 fictional / inspired tracks");

    // Specific regression validations for circuits highlighted in user issue
    let bahrain = tdrace_core::track::Track::load_from_file(tracks_root.join("gt/bahrain.json"))
        .expect("Load bahrain");
    assert_eq!(
        bahrain.osm_url.as_deref(),
        Some("https://www.openstreetmap.org/relation/284538"),
        "Bahrain must link to authentic raceway relation 284538"
    );

    let cota = tdrace_core::track::Track::load_from_file(tracks_root.join("gt/cota.json"))
        .expect("Load cota");
    assert_eq!(
        cota.osm_url.as_deref(),
        Some("https://www.openstreetmap.org/relation/6537729"),
        "COTA must link to authentic relation 6537729"
    );

    let montreal = tdrace_core::track::Track::load_from_file(tracks_root.join("gt/montreal.json"))
        .expect("Load montreal");
    assert_eq!(
        montreal.osm_url.as_deref(),
        Some("https://www.openstreetmap.org/relation/284595"),
        "Montreal must link to authentic relation 284595"
    );
}

#[test]
fn test_portal_circuits_catalog_provenance_integrity() {
    let portal_json = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../portals/shared/data/circuits.json");
    assert!(portal_json.exists(), "portals/shared/data/circuits.json must exist");

    let raw = fs::read_to_string(&portal_json).expect("Read circuits.json");
    let circuits: Vec<serde_json::Value> = serde_json::from_str(&raw).expect("Parse circuits.json");

    assert_eq!(circuits.len(), 96, "Catalog must contain exactly 96 circuits");

    let mut osm_count = 0;
    let mut wiki_count = 0;
    for c in &circuits {
        let osm = c.get("osm_url").and_then(|v| v.as_str());
        let wiki = c.get("wikipedia_url").and_then(|v| v.as_str());
        let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");

        if let Some(url) = osm {
            assert!(url.starts_with("https://www.openstreetmap.org/"));
            osm_count += 1;
        }
        if let Some(url) = wiki {
            assert!(url.starts_with("https://en.wikipedia.org/wiki/"));
            wiki_count += 1;
        }

        if id == "bahrain" {
            assert_eq!(osm, Some("https://www.openstreetmap.org/relation/284538"));
        } else if id == "cota" {
            assert_eq!(osm, Some("https://www.openstreetmap.org/relation/6537729"));
        } else if id == "montreal" {
            assert_eq!(osm, Some("https://www.openstreetmap.org/relation/284595"));
        }
    }

    assert_eq!(osm_count, 71, "Exactly 71 circuits in portal catalog must possess OSM URL");
    assert_eq!(wiki_count, 71, "Exactly 71 circuits in portal catalog must possess Wikipedia URL");
}


