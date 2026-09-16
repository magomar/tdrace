use std::fs;
use tdrace_app::tracks::{DevTrackStore, UserTrackStore};
use tdrace_core::track::presets::classic_grand_prix;
use tdrace_core::track::TrackCategory;

#[test]
fn test_user_track_store_lifecycle_and_auto_backup() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_user_store_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let store = UserTrackStore::new(&temp_dir);

    // 1. Initial save creates file and backup snapshot
    let mut track = classic_grand_prix();
    track.name = "Test Circuit".to_string();
    track.category = TrackCategory::Draft;

    let saved_path = store.save_track(&track, "test_circuit", false).expect("Save track");
    assert!(saved_path.exists());
    assert!(store.track_exists("test_circuit"));

    let backup_file = store.backup_dir().join("test_circuit.json");
    assert!(backup_file.exists(), "Auto-backup must exist upon save");

    // 2. Scan does not include .backup folder
    let scanned = store.scan_tracks();
    assert_eq!(scanned.len(), 1);
    assert_eq!(scanned[0].id, "test_circuit");

    // 3. Overwrite creates a new backup
    track.name = "Test Circuit Overwritten".to_string();
    let overwritten_path = store.save_track(&track, "test_circuit", true).expect("Overwrite track");
    assert_eq!(saved_path, overwritten_path);

    let loaded = store.load_track("test_circuit").expect("Load track");
    assert_eq!(loaded.name, "Test Circuit Overwritten");

    // 4. Delete track moves it to backup and removes from active scan
    let deleted = store.delete_track("test_circuit").expect("Delete track");
    assert!(deleted);
    assert!(!saved_path.exists(), "Active file must be removed");
    assert!(backup_file.exists(), "Backup copy must still exist");
    assert_eq!(store.scan_tracks().len(), 0, "Active scan must be empty");

    // 5. Fallback load from backup recovers the track
    let recovered = store.load_track("test_circuit").expect("Fallback load from backup");
    assert_eq!(recovered.name, "Test Circuit Overwritten");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dev_track_store_dual_persistence_and_standard_mode_safety() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tdrace_test_dev_store_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let _ = fs::remove_dir_all(&temp_dir);

    let user_dir = temp_dir.join("user_tracks");
    let store = UserTrackStore::new(&user_dir);
    let dev_store = DevTrackStore::new();

    let mut track = classic_grand_prix();
    track.name = "Alpine Sprint".to_string();
    track.category = TrackCategory::Draft;
    store.save_track(&track, "alpine_sprint", true).expect("Save initial draft");

    // 1. In standard mode (no TDRACE_DEV), promotion must fail
    std::env::remove_var("TDRACE_DEV");
    let err = dev_store.promote_to_git_preset(&store, "alpine_sprint", "classic");
    assert!(err.is_err(), "Standard mode must reject promotion");

    // 2. In Dev mode, promotion dual-persists to Git and user storage
    let mock_git = temp_dir.join("mock_git");
    let _ = fs::create_dir_all(mock_git.join("classic"));
    std::env::set_var("TDRACE_DEV", "1");
    std::env::set_var("TDRACE_GIT_TRACKS_DIR", &mock_git);

    let git_path = dev_store
        .promote_to_git_preset(&store, "alpine_sprint", "classic")
        .expect("Dev mode must promote track");
    assert!(git_path.exists(), "Git preset file must exist");

    // CRITICAL: User copy MUST still exist in user storage!
    let user_copy = store.path_for_slug("alpine_sprint");
    assert!(user_copy.exists(), "User storage copy MUST be retained (dual persistence)");

    // 3. Simulating external git deletion (e.g. rm tracks/classic/alpine_sprint.json)
    let _ = fs::remove_file(&git_path);
    assert!(!git_path.exists(), "Git preset deleted");

    // The track is still safe and accessible from user storage!
    let user_loaded = store.load_track("alpine_sprint").expect("User track survives git deletion");
    assert_eq!(user_loaded.name, "Alpine Sprint");

    std::env::remove_var("TDRACE_DEV");
    std::env::remove_var("TDRACE_GIT_TRACKS_DIR");
    let _ = fs::remove_dir_all(&temp_dir);
}
