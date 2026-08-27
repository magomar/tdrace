use std::fs;
use std::path::Path;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::track_manager::TrackManager;
use tdrace_app::ui::track_manager_ui::{TrackManagerModal, TrackManagerTab};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::TrackCategory;

#[test]
fn test_track_categories_initial_presets() {
    let temp_dir = std::env::temp_dir().join("tdrace_test_tm_presets");
    let _ = fs::remove_dir_all(&temp_dir);

    let manager = TrackManager::new(&temp_dir);

    // 1. Initial state: 7 main presets, 0 drafts
    let main_tracks = manager.main_track_choices();
    let draft_tracks = manager.draft_track_choices();

    assert_eq!(main_tracks.len(), 7, "All 7 presets should be Main tracks");
    assert_eq!(draft_tracks.len(), 0, "Initial draft tracks list should be empty");

    for choice in &main_tracks {
        assert!(!choice.description().is_empty(), "Preset must have a non-empty description");
        assert!(!choice.is_custom(), "Built-in presets should not be flagged as custom");
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_draft_creation_and_isolation_from_main_menu() {
    let temp_dir = std::env::temp_dir().join("tdrace_test_tm_drafts");
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

    assert_eq!(main_tracks.len(), 7, "Main menu should only contain approved circuits");
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

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_promotion_and_demotion_lifecycle() {
    let temp_dir = std::env::temp_dir().join("tdrace_test_tm_lifecycle");
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Create a draft
    let track_id = "test_circuit_proto";
    let mut track = classic_grand_prix();
    track.name = "Proto Circuit".to_string();
    track.description = "Prototype for testing.".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(track_id)).expect("Save proto");

    assert_eq!(manager.main_track_choices().len(), 7);
    assert_eq!(manager.draft_track_choices().len(), 1);

    // Promote to Main (Approved circuit)
    manager.promote_track(track_id).expect("Must promote to Main");

    assert_eq!(manager.main_track_choices().len(), 8, "Promoted track must appear in Main");
    assert_eq!(manager.draft_track_choices().len(), 0, "Promoted track must be removed from Drafts");

    let promoted_choice = &manager.main_track_choices()[7];
    assert_eq!(promoted_choice.title(), "Proto Circuit");
    let loaded_promoted = manager.load_track(promoted_choice).expect("Must load promoted");
    assert_eq!(loaded_promoted.category, TrackCategory::Main);

    // Demote back to Draft (Under testing)
    manager.demote_track(track_id).expect("Must demote to Draft");

    assert_eq!(manager.main_track_choices().len(), 7, "Demoted track must be removed from Main");
    assert_eq!(manager.draft_track_choices().len(), 1, "Demoted track must reappear in Drafts");

    let loaded_demoted = manager.load_track(&manager.draft_track_choices()[0]).expect("Must load demoted");
    assert_eq!(loaded_demoted.category, TrackCategory::Draft);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_metadata_editing() {
    let temp_dir = std::env::temp_dir().join("tdrace_test_tm_meta");
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

    let loaded = manager.load_track(&manager.main_track_choices()[7]).expect("Load");
    assert_eq!(loaded.name, "Apex Super Circuit");
    assert_eq!(loaded.description, "Ultra high grip asphalt with high G curves.");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_track_deletion() {
    let temp_dir = std::env::temp_dir().join("tdrace_test_tm_delete");
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
    let temp_dir = std::env::temp_dir().join("tdrace_test_tm_session");
    let _ = fs::remove_dir_all(&temp_dir);

    let mut session = RaceSession::default();
    session.track_manager = TrackManager::new(&temp_dir);

    // 1. Initial State: Menu
    assert_eq!(session.state, GameState::Menu);

    // 2. Transition to Track Manager
    session.state = GameState::TrackManager {
        active_tab: TrackManagerTab::Main,
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
