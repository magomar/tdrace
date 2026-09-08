use std::fs;
use std::sync::Mutex;
use tdrace_app::track_manager::{ModuleFilter, TrackManager};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::TrackCategory;

static PERM_TEST_MUTEX: Mutex<()> = Mutex::new(());

struct DevModeGuard;
impl DevModeGuard {
    fn enter() -> Self {
        std::env::set_var("TDRACE_DEV", "1");
        Self
    }
}
impl Drop for DevModeGuard {
    fn drop(&mut self) {
        std::env::remove_var("TDRACE_DEV");
    }
}

#[test]
fn test_standard_mode_blocks_preset_modification() {
    let _lock = PERM_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_perm_std_blocks_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Assigning categories to an official preset must fail in standard mode
    let promo_err = manager.promote_track_to_module("classic_grand_prix", "f1").unwrap_err();
    assert!(promo_err.contains("official preset circuit and its categories cannot be modified in standard mode"));

    let promo_mods_err = manager.promote_track_to_modules("monza", &["classic", "rally"]).unwrap_err();
    assert!(promo_mods_err.contains("official preset circuit and its categories cannot be modified in standard mode"));

    // 2. Demoting an official preset must fail in standard mode
    let demote_err = manager.demote_track("classic_grand_prix").unwrap_err();
    assert!(demote_err.contains("official preset circuit and cannot be demoted in standard mode"));

    // 3. Updating metadata of an official preset must fail in standard mode
    let meta_err = manager
        .update_track_metadata("classic_grand_prix", "Hacked GP".to_string(), "Hacked desc".to_string())
        .unwrap_err();
    assert!(meta_err.contains("official preset circuit and its metadata cannot be modified in standard mode"));

    // 4. Deleting an official preset must fail in standard mode
    let del_err = manager.delete_custom_track("classic_grand_prix").unwrap_err();
    assert!(del_err.contains("official preset circuit and cannot be deleted in standard mode"));

    let del_mod_err = manager.delete_track_from_module("monza", Some("f1")).unwrap_err();
    assert!(del_mod_err.contains("official preset circuit and cannot be deleted in standard mode"));

    // 5. Official presets must remain in catalog unmodified
    let classic_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert_eq!(classic_tracks.len(), 10);
    assert!(classic_tracks.iter().any(|t| t.track_id() == "classic_grand_prix"));

    let f1_tracks = manager.filtered_main_track_choices(ModuleFilter::F1);
    assert!(f1_tracks.iter().any(|t| t.track_id() == "monza"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_standard_mode_allows_custom_circuit_management() {
    let _lock = PERM_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("TDRACE_DEV");

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_perm_std_custom_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a custom track
    let mut custom = classic_grand_prix();
    custom.name = "My Custom Speedway".to_string();
    custom.category = TrackCategory::Draft;
    manager.save_custom_track(&custom, Some("my_custom_speedway")).unwrap();

    // 2. Category assignment is permitted for custom tracks
    manager.promote_track_to_modules("my_custom_speedway", &["classic", "rally"]).unwrap();
    assert!(manager.is_track_in_module("my_custom_speedway", "classic"));
    assert!(manager.is_track_in_module("my_custom_speedway", "rally"));
    assert!(!manager.is_track_in_module("my_custom_speedway", "f1"));

    // 3. Metadata update is permitted for custom tracks
    manager
        .update_track_metadata(
            "my_custom_speedway",
            "My Renamed Speedway".to_string(),
            "Updated track description".to_string(),
        )
        .unwrap();

    let loaded = manager.load_track_by_slug("my_custom_speedway").unwrap();
    assert_eq!(loaded.name, "My Renamed Speedway");
    assert_eq!(loaded.description, "Updated track description");

    // 4. Deletion is permitted for custom tracks
    let deleted = manager.delete_custom_track("my_custom_speedway").unwrap();
    assert!(deleted);
    assert!(!manager.is_track_in_module("my_custom_speedway", "classic"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dev_mode_permits_preset_category_and_metadata_updates() {
    let _lock = PERM_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _dev_guard = DevModeGuard::enter();

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_perm_dev_preset_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // In dev mode, reassigning categories of an official preset succeeds
    let res = manager.promote_track_to_modules("classic_grand_prix", &["classic", "f1"]);
    assert!(res.is_ok(), "Dev mode must allow updating preset modules: {:?}", res);

    // In dev mode, updating metadata of an official preset succeeds
    let meta_res = manager.update_track_metadata(
        "classic_grand_prix",
        "Classic Grand Prix (Pro Edition)".to_string(),
        "Updated by circuit designer.".to_string(),
    );
    assert!(meta_res.is_ok(), "Dev mode must allow updating preset metadata: {:?}", meta_res);

    // Revert metadata back to canonical
    let _ = manager.update_track_metadata(
        "classic_grand_prix",
        "Classic Grand Prix".to_string(),
        "High-speed sweeping chicanes, hairpin sand traps & tactical pit lane.".to_string(),
    );
    let _ = manager.promote_track_to_modules("classic_grand_prix", &["classic", "f1"]);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dev_mode_promote_custom_track_and_demote_preset() {
    let _lock = PERM_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _dev_guard = DevModeGuard::enter();

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_perm_promo_demote_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // 1. Create a custom track
    let slug = "nordic_sprint";
    let mut track = classic_grand_prix();
    track.name = "Nordic Sprint".to_string();
    track.description = "Fast technical tarmac sprint.".to_string();
    track.category = TrackCategory::Draft;
    manager.save_custom_track(&track, Some(slug)).unwrap();

    assert_eq!(manager.draft_track_choices().len(), 1);

    // 2. Promote custom track to git preset via dev shortcut method
    let promo_path = manager
        .promote_custom_track_to_git_preset(slug)
        .expect("Dev mode must promote custom track to git preset");
    assert!(promo_path.exists());
    assert!(promo_path.to_string_lossy().contains("tracks/classic/nordic_sprint.json"));

    // User local copy must be cleaned up, and track must now appear in catalog
    assert_eq!(manager.draft_track_choices().len(), 0);
    let classic_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    assert!(classic_tracks.iter().any(|t| t.track_id() == slug));

    // 3. Demote preset back to custom track via dev shortcut method
    let custom_path = manager
        .demote_preset_to_custom_track(slug)
        .expect("Dev mode must demote preset to custom track");
    assert!(custom_path.exists());
    assert!(custom_path.to_string_lossy().contains("nordic_sprint.json"));
    assert!(!promo_path.exists(), "Git preset file must be removed upon demotion");

    // Must now appear in drafts
    assert_eq!(manager.draft_track_choices().len(), 1);
    assert_eq!(manager.draft_track_choices()[0].track_id(), slug);

    // 4. Clean up local test track
    let _ = manager.delete_custom_track(slug);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_demote_official_presets_visible_as_custom_tracks_in_track_manager() {
    let _lock = PERM_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _dev_guard = DevModeGuard::enter();

    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_perm_demote_visible_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let mut manager = TrackManager::new(&temp_dir);

    // Initial counts
    let init_classic = manager.filtered_main_track_choices(ModuleFilter::Classic).len();
    let init_rally = manager.filtered_main_track_choices(ModuleFilter::Rally).len();

    // Backup exact git preset files before demote so the repo working directory stays clean
    let git_backup: Vec<_> = tdrace_app::storage::resolve_git_tracks_dir()
        .map(|git_dir| {
            let paths = [
                git_dir.join("classic").join("outlaw_pass.json"),
                git_dir.join("rally").join("outlaw_pass.json"),
                git_dir.join("rally").join("sahara_dunes.json"),
            ];
            paths.into_iter().filter_map(|p| {
                fs::read(&p).ok().map(|content| (p, content))
            }).collect()
        })
        .unwrap_or_default();

    // Demote outlaw_pass and sahara_dunes
    let outlaw_path = manager.demote_preset_to_custom_track("outlaw_pass").expect("Demote outlaw_pass");
    let sahara_path = manager.demote_preset_to_custom_track("sahara_dunes").expect("Demote sahara_dunes");
    assert!(outlaw_path.exists());
    assert!(sahara_path.exists());

    // Both tracks must remain visible in Track Manager under their respective modules!
    let classic_tracks = manager.filtered_main_track_choices(ModuleFilter::Classic);
    let rally_tracks = manager.filtered_main_track_choices(ModuleFilter::Rally);

    assert_eq!(classic_tracks.len(), init_classic, "Classic track count must not decrease when demoting to custom");
    assert_eq!(rally_tracks.len(), init_rally, "Rally track count must not decrease when demoting to custom");

    // They must now appear as custom circuits, NOT immutable official presets
    let outlaw_choice = classic_tracks.iter().find(|t| t.track_id() == "outlaw_pass").expect("outlaw_pass in Classic");
    assert!(!outlaw_choice.is_official_preset(), "Demoted outlaw_pass must be a custom track");
    assert!(matches!(outlaw_choice, tdrace_app::ui::menu::TrackChoice::Custom { .. }));

    let sahara_choice = rally_tracks.iter().find(|t| t.track_id() == "sahara_dunes").expect("sahara_dunes in Rally");
    assert!(!sahara_choice.is_official_preset(), "Demoted sahara_dunes must be a custom track");
    assert!(matches!(sahara_choice, tdrace_app::ui::menu::TrackChoice::Custom { .. }));

    // Both must appear in custom_track_choices (for Main Menu CUSTOM tab)
    let custom_choices = manager.custom_track_choices();
    assert!(custom_choices.iter().any(|t| t.track_id() == "outlaw_pass"));
    assert!(custom_choices.iter().any(|t| t.track_id() == "sahara_dunes"));

    // Verify persistence when reloading manager from disk
    let reloaded_manager = TrackManager::new(&temp_dir);
    let reloaded_classic = reloaded_manager.filtered_main_track_choices(ModuleFilter::Classic);
    let reloaded_rally = reloaded_manager.filtered_main_track_choices(ModuleFilter::Rally);

    assert!(reloaded_classic.iter().any(|t| t.track_id() == "outlaw_pass" && !t.is_official_preset()));
    assert!(reloaded_rally.iter().any(|t| t.track_id() == "sahara_dunes" && !t.is_official_preset()));
    assert!(reloaded_rally.iter().any(|t| t.track_id() == "outlaw_pass" && !t.is_official_preset()));

    // Restore git preset files removed by demote during this test
    for (p, content) in git_backup {
        let _ = fs::write(p, content);
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

