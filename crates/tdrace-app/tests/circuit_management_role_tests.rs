use std::fs;
use std::sync::Mutex;
use tdrace_app::track_manager::{ModuleFilter, TrackManager};
use tdrace_app::tracks::{DevTrackStore, UserTrackStore};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::TrackCategory;

static ROLE_TEST_MUTEX: Mutex<()> = Mutex::new(());

struct DevEnvGuard {
    temp_dir: std::path::PathBuf,
    #[allow(dead_code)]
    mock_git: std::path::PathBuf,
}

impl DevEnvGuard {
    fn new() -> Self {
        let temp_dir = std::env::temp_dir().join(format!(
            "tdrace_role_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mock_git = temp_dir.join("git_tracks");
        for m in &["classic", "gt", "rally", "kart", "nascar"] {
            let _ = fs::create_dir_all(mock_git.join(m));
        }

        std::env::set_var("TDRACE_DEV", "1");
        std::env::set_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR, &mock_git);
        Self { temp_dir, mock_git }
    }
}

impl Drop for DevEnvGuard {
    fn drop(&mut self) {
        std::env::remove_var("TDRACE_DEV");
        std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn test_player_role_isolation_and_immutable_presets() {
    let _lock = ROLE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");
    std::env::remove_var(tdrace_app::storage::ENV_GIT_TRACKS_DIR);

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_player_role_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. In standard Player mode, official presets cannot be mutated
    let preset_id = "classic_grand_prix";
    let del_res = manager.delete_custom_track(preset_id);
    assert!(del_res.is_err(), "Player mode must reject preset deletion");
    let err_msg = del_res.unwrap_err();
    assert!(
        err_msg.contains("official preset circuit and cannot be deleted in standard mode"),
        "Error message must be English: {}",
        err_msg
    );

    // 2. Reordering presets is forbidden for player
    let reorder_up = manager.reorder_preset_track(preset_id, "classic", true);
    assert!(reorder_up.is_err(), "Player mode must reject preset reordering");
    let reorder_down = manager.reorder_preset_track(preset_id, "classic", false);
    assert!(reorder_down.is_err(), "Player mode must reject preset reordering");

    // 3. Promotion to git preset is forbidden for player
    let promo_res = manager.promote_custom_track_to_git_preset("some_circuit");
    assert!(promo_res.is_err(), "Player mode must reject git preset promotion");

    // 4. "My Circuits" library (module_custom_tracks) strictly lists user tracks, never presets
    let custom_list = manager.module_custom_tracks("classic");
    assert!(
        custom_list.is_empty(),
        "My Circuits must be empty for classic when no user tracks exist"
    );
    assert!(
        !custom_list.iter().any(|t| t.track_id() == preset_id),
        "Official presets must NEVER leak into My Circuits view"
    );

    // 5. Creating a custom track in Player mode
    let mut custom = classic_grand_prix();
    custom.name = "Player Special".to_string();
    custom.category = TrackCategory::Main;
    custom.modules = vec!["classic".to_string()];
    manager.save_custom_track(&custom, Some("player_special")).expect("Save custom track");

    let updated_custom_list = manager.module_custom_tracks("classic");
    assert_eq!(updated_custom_list.len(), 1);
    assert_eq!(updated_custom_list[0].track_id(), "player_special");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_data_loss_immunity_and_dual_persistence() {
    let _lock = ROLE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let guard = DevEnvGuard::new();

    let user_dir = guard.temp_dir.join("user_tracks");
    let mut manager = TrackManager::new(&user_dir);
    let dev_store = DevTrackStore::new();
    let user_store = UserTrackStore::new(&user_dir);

    // 1. Player creates a custom circuit (e.g. Ramp Raceway redesign)
    let slug = "ramp_raceway_v2";
    let mut custom_track = classic_grand_prix();
    custom_track.name = "Ramp Raceway V2".to_string();
    custom_track.description = "Revamped with massive tabletop jumps.".to_string();
    custom_track.category = TrackCategory::Draft;

    manager.save_custom_track(&custom_track, Some(slug)).expect("Save user custom circuit");

    let user_file = user_store.path_for_slug(slug);
    assert!(user_file.exists(), "User circuit file must exist before promotion");

    // 2. Dev promotes custom circuit to official git preset
    let git_file = dev_store
        .promote_to_git_preset(&user_store, slug, "classic")
        .expect("Promote custom track to git preset");
    assert!(git_file.exists(), "Preset file must exist in git directory");

    // CRITICAL GUARANTEE: User file MUST NOT be deleted upon promotion!
    assert!(
        user_file.exists(),
        "User file must remain intact in user storage after promotion (Dual Persistence)"
    );

    // Verify auto-backup was created
    let backup_file = user_store.backup_dir().join(format!("{}.json", slug));
    assert!(
        backup_file.exists(),
        "Auto-backup snapshot must exist in .backup/ directory"
    );

    // 3. Simulate accidental Git wipe / git clean -fdx / branch switch removing repository preset
    let _ = fs::remove_file(&git_file);
    assert!(!git_file.exists(), "Git preset file was wiped");

    // ZERO DATA LOSS: The user's track is STILL SAFE and fully loadable!
    let recovered_track = user_store
        .load_track(slug)
        .expect("User track must survive external Git wipe");
    assert_eq!(recovered_track.name, "Ramp Raceway V2");
    assert_eq!(recovered_track.description, "Revamped with massive tabletop jumps.");

    // Rescanning tracks still finds the user circuit
    manager.scan_custom_tracks().expect("Rescan tracks");
    let choices = manager.module_custom_tracks("classic");
    assert!(
        choices.iter().any(|c| c.track_id() == slug),
        "Custom track must remain available in My Circuits even after Git file deletion"
    );
}

#[test]
fn test_safe_deletion_with_auto_backup_archive() {
    let _lock = ROLE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_safe_delete_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = UserTrackStore::new(&temp_dir);
    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create two distinct custom tracks
    let mut t1 = classic_grand_prix();
    t1.name = "Track Alpha".to_string();
    manager.save_custom_track(&t1, Some("track_alpha")).unwrap();

    let mut t2 = classic_grand_prix();
    t2.name = "Track Beta".to_string();
    manager.save_custom_track(&t2, Some("track_beta")).unwrap();

    let alpha_path = store.path_for_slug("track_alpha");
    let beta_path = store.path_for_slug("track_beta");
    assert!(alpha_path.exists());
    assert!(beta_path.exists());

    // 2. Delete Track Alpha
    let deleted = manager.delete_custom_track("track_alpha").expect("Delete custom track");
    assert!(deleted);

    // 3. Track Alpha active file is deleted, but backup exists
    assert!(!alpha_path.exists(), "Active alpha file must be deleted");
    let backup_alpha = store.backup_dir().join("track_alpha.json");
    assert!(backup_alpha.exists(), "Backup snapshot of alpha must exist");

    // 4. Track Beta is completely unaffected
    assert!(beta_path.exists(), "Beta track file must remain untouched");
    let loaded_beta = manager.load_track_by_slug("track_beta").expect("Load beta track");
    assert_eq!(loaded_beta.name, "Track Beta");

    // 5. Fallback load from backup recovers Track Alpha
    let recovered_alpha = store.load_track("track_alpha").expect("Recover alpha from backup");
    assert_eq!(recovered_alpha.name, "Track Alpha");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dev_workbench_reordering_and_persistence() {
    let _lock = ROLE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let guard = DevEnvGuard::new();

    let user_dir = guard.temp_dir.join("user_tracks");
    let mut manager = TrackManager::new(&user_dir);

    // Initial preset order in classic
    let init_choices = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(init_choices.len() >= 2);
    let first_id = init_choices[0].track_id().to_string();
    let second_id = init_choices[1].track_id().to_string();

    // In dev mode, move the second track UP
    manager
        .reorder_preset_track(&second_id, "classic", true)
        .expect("Dev mode reorder_preset_track up");

    let reordered_choices = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert_eq!(reordered_choices[0].track_id(), second_id);
    assert_eq!(reordered_choices[1].track_id(), first_id);

    // Reload manager from disk to verify preset_order.json persistence
    let reloaded_manager = TrackManager::new(&user_dir);
    let persisted_choices = reloaded_manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert_eq!(persisted_choices[0].track_id(), second_id);
    assert_eq!(persisted_choices[1].track_id(), first_id);
}

#[test]
fn test_ui_copy_and_error_messages_are_strictly_in_english() {
    let _lock = ROLE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_ui_copy_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut manager = TrackManager::new(&temp_dir);

    // Test error message on unauthorized preset mutation
    let err = manager.delete_custom_track("classic_grand_prix").unwrap_err();
    assert!(
        !err.contains("circuito") && !err.contains("eliminar"),
        "Error message must not contain Spanish words: {}",
        err
    );
    assert!(
        err.contains("official preset circuit and cannot be deleted in standard mode"),
        "Expected clear English error message: {}",
        err
    );

    // Check ModuleFilter labels are in English
    assert_eq!(ModuleFilter::Classic.label(), "CLASSIC");
    assert_eq!(ModuleFilter::Rally.label(), "RALLY");
    assert_eq!(ModuleFilter::Kart.label(), "KARTING");
    assert_eq!(ModuleFilter::Gt.label(), "GT WORLD CHALLENGE");
    assert_eq!(ModuleFilter::Nascar.label(), "NASCAR");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_circuit_manager_preset_visibility_and_cloning_to_drafts() {
    let _lock = ROLE_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_cm_visibility_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. In Circuit Manager, filtered_main_track_choices returns presets for the module
    let classic_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(!classic_tracks.is_empty(), "Classic module must have presets visible in Circuit Manager");
    assert!(classic_tracks.iter().any(|t| t.track_id() == "classic_grand_prix"));
    assert!(classic_tracks.iter().any(|t| t.is_official_preset()));

    // Add a custom circuit for Classic
    let mut custom = classic_grand_prix();
    custom.name = "Custom Speed Ring".to_string();
    custom.category = TrackCategory::Main;
    custom.modules = vec!["classic".to_string()];
    manager.save_custom_track(&custom, Some("custom_speed_ring")).expect("Save custom circuit");

    // Both preset and custom are visible in Circuit Manager
    let updated_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(updated_tracks.iter().any(|t| t.track_id() == "classic_grand_prix"));
    assert!(updated_tracks.iter().any(|t| t.track_id() == "custom_speed_ring"));

    // 2. Cloning an official preset creates a Draft in Drafts category
    let preset_choice = classic_tracks.iter().find(|t| t.track_id() == "classic_grand_prix").unwrap();
    let (cloned_preset, preset_path) = manager.clone_track(preset_choice).expect("Clone preset must succeed");
    assert_eq!(cloned_preset.category, TrackCategory::Draft, "Cloned preset must have category Draft");
    assert!(cloned_preset.modules.is_empty(), "Cloned preset modules must be cleared for Drafts");
    assert!(std::path::Path::new(&preset_path).exists());

    // 3. Cloning a custom circuit also creates a Draft in Drafts category
    let custom_choice = updated_tracks.iter().find(|t| t.track_id() == "custom_speed_ring").unwrap();
    let (cloned_custom, custom_path) = manager.clone_track(custom_choice).expect("Clone custom track must succeed");
    assert_eq!(cloned_custom.category, TrackCategory::Draft, "Cloned custom track must have category Draft");
    assert!(std::path::Path::new(&custom_path).exists());

    // 4. Drafts list contains both clones, and neither leaks into module_custom_tracks
    let drafts = manager.draft_track_choices();
    assert_eq!(drafts.len(), 2, "Both cloned circuits must be in Drafts category");
    assert!(drafts.iter().any(|t| t.title() == "Classic Grand Prix (clone)"));
    assert!(drafts.iter().any(|t| t.title() == "Custom Speed Ring (clone)"));

    let module_approved = manager.module_custom_tracks("classic");
    assert_eq!(module_approved.len(), 1, "Only approved custom track must be in module_custom_tracks");
    assert_eq!(module_approved[0].track_id(), "custom_speed_ring");

    let _ = fs::remove_dir_all(&temp_dir);
}
