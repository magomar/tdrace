use tdrace_app::db::{HallOfFameDb, HallOfFameEntry};
use tdrace_app::editor::EditorAction;
use tdrace_app::game::{FinishedScreenView, GameState, RaceSession};
use tdrace_app::profile::RaceHistoryEntry;
use tdrace_app::ui::menu::TrackChoice;

#[test]
fn test_hall_of_fame_in_memory_db_schema_and_empty_top10() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");
    let entries = db.get_top_10("classic_grand_prix").expect("Query should succeed");
    assert!(entries.is_empty(), "Initial entries should be empty");

    let qualifies = db.is_top_10("classic_grand_prix", 85.0).expect("Query should succeed");
    assert!(qualifies, "Any time qualifies when there are fewer than 10 records");
}

#[test]
fn test_hall_of_fame_insertion_and_ordering() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");

    let times = [78.5, 65.2, 92.1, 71.0, 60.5];
    for (i, &t) in times.iter().enumerate() {
        let entry = HallOfFameEntry {
            id: None,
            track_id: "classic_grand_prix".to_string(),
            player_name: format!("DRIVER_{}", i),
            car_name: "GT Sports Coupe".to_string(),
            total_time: t,
            best_lap: Some(t / 3.0),
            laps: 3,
            created_at: "2026-08-26 12:00".to_string(),
        };
        db.insert_entry(&entry).expect("Insert should succeed");
    }

    let top = db.get_top_10("classic_grand_prix").expect("Query top 10");
    assert_eq!(top.len(), 5);
    // Ascending order check: 60.5, 65.2, 71.0, 78.5, 92.1
    assert_eq!(top[0].player_name, "DRIVER_4");
    assert_eq!(top[0].total_time, 60.5);
    assert_eq!(top[1].player_name, "DRIVER_1");
    assert_eq!(top[1].total_time, 65.2);
    assert_eq!(top[4].player_name, "DRIVER_2");
    assert_eq!(top[4].total_time, 92.1);
}

#[test]
fn test_hall_of_fame_top10_cutoff_and_qualification() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");

    // Insert 10 records: 10.0, 20.0, 30.0, ..., 100.0
    for i in 1..=10 {
        let entry = HallOfFameEntry {
            id: None,
            track_id: "drift_park".to_string(),
            player_name: format!("BOT_{}", i),
            car_name: "Tuned Drift Spec".to_string(),
            total_time: (i * 10) as f32,
            best_lap: Some((i * 3) as f32),
            laps: 3,
            created_at: "2026-08-26 12:00".to_string(),
        };
        db.insert_entry(&entry).expect("Insert should succeed");
    }

    let entries = db.get_top_10("drift_park").expect("Query top 10");
    assert_eq!(entries.len(), 10);
    assert_eq!(entries[9].total_time, 100.0);

    // Faster than 10th record (100.0) -> qualifies
    assert!(db.is_top_10("drift_park", 95.0).unwrap());
    assert!(db.is_top_10("drift_park", 5.0).unwrap());

    // Slower or equal to 10th record -> does not qualify
    assert!(!db.is_top_10("drift_park", 100.0).unwrap());
    assert!(!db.is_top_10("drift_park", 105.0).unwrap());

    // Insert 11th record with 5.0s (new leader)
    let new_entry = HallOfFameEntry {
        id: None,
        track_id: "drift_park".to_string(),
        player_name: "HUMAN_HERO".to_string(),
        car_name: "Tuned Drift Spec".to_string(),
        total_time: 5.0,
        best_lap: Some(1.5),
        laps: 3,
        created_at: "2026-08-26 12:00".to_string(),
    };
    db.insert_entry(&new_entry).expect("Insert should succeed");

    let updated_top = db.get_top_10("drift_park").expect("Query top 10");
    assert_eq!(updated_top.len(), 10, "Top 10 query must cap at 10 results");
    assert_eq!(updated_top[0].player_name, "HUMAN_HERO");
    assert_eq!(updated_top[0].total_time, 5.0);
    assert_eq!(updated_top[9].total_time, 90.0, "100.0 should have been pushed out");
}

#[test]
fn test_hall_of_fame_track_isolation() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");

    let gp_entry = HallOfFameEntry {
        id: None,
        track_id: "classic_grand_prix".to_string(),
        player_name: "GP_CHAMP".to_string(),
        car_name: "GT Sports Coupe".to_string(),
        total_time: 72.0,
        best_lap: Some(24.0),
        laps: 3,
        created_at: "2026-08-26 12:00".to_string(),
    };
    db.insert_entry(&gp_entry).unwrap();

    let oval_entry = HallOfFameEntry {
        id: None,
        track_id: "oval_speedway".to_string(),
        player_name: "NASCAR_FAN".to_string(),
        car_name: "AWD Turbo Rally".to_string(),
        total_time: 40.0,
        best_lap: Some(13.0),
        laps: 3,
        created_at: "2026-08-26 12:00".to_string(),
    };
    db.insert_entry(&oval_entry).unwrap();

    let gp_list = db.get_top_10("classic_grand_prix").unwrap();
    assert_eq!(gp_list.len(), 1);
    assert_eq!(gp_list[0].player_name, "GP_CHAMP");

    let oval_list = db.get_top_10("oval_speedway").unwrap();
    assert_eq!(oval_list.len(), 1);
    assert_eq!(oval_list[0].player_name, "NASCAR_FAN");
}

#[test]
fn test_hall_of_fame_clean_start_and_clear() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");

    db.seed_defaults_if_empty("classic_grand_prix").unwrap();
    let initial = db.get_top_10("classic_grand_prix").unwrap();
    assert!(initial.is_empty(), "Hall of Fame should start clean with no fake benchmark entries");

    // Insert an actual race entry
    let entry = HallOfFameEntry {
        id: None,
        track_id: "classic_grand_prix".to_string(),
        player_name: "Real Racer".to_string(),
        car_name: "GT Sports Coupe".to_string(),
        total_time: 74.5,
        best_lap: Some(24.8),
        laps: 3,
        created_at: "2026-08-27 12:00".to_string(),
    };
    db.insert_entry(&entry).unwrap();
    assert_eq!(db.get_top_10("classic_grand_prix").unwrap().len(), 1);

    // Clear hall of fame
    db.clear_hall_of_fame().unwrap();
    assert!(db.get_top_10("classic_grand_prix").unwrap().is_empty());
}

#[test]
fn test_race_session_hof_automatic_logging_and_congratulations() {
    let mut session = RaceSession::new();
    // Swap DB to in-memory for testing
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    assert_eq!(session.track_choice_id(), "classic_grand_prix");
    assert!(session.hof_entries.is_empty(), "Hall of Fame should start empty");

    // Simulate player completing race in 1st place with a personal best
    session.session_time = 45.0;
    session.trackers[0].current_lap = session.total_laps + 1;
    session.trackers[0].best_lap_time = Some(15.0);

    // Check race finish transition
    session.check_race_finish();

    // Verify automatic transition to Finished state displaying Results first
    assert_eq!(session.state, GameState::Finished);
    assert_eq!(session.finished_view, FinishedScreenView::Results);
    assert!(!session.show_hall_of_fame);

    // Verify Hall of Fame table was populated with the actual race result
    assert!(!session.hof_entries.is_empty());
    assert_eq!(session.hof_entries[0].player_name, session.active_profile.alias);
    assert_eq!(session.hof_entries[0].total_time, 45.0);
    assert_eq!(session.hof_entries[0].best_lap, Some(15.0));
    assert!(session.recent_hof_id.is_some());

    // Verify congratulations metadata was computed
    let congrats = session.recent_congrats.as_ref().expect("Congratulations should be present");
    assert!(congrats.is_personal_best);
    assert_eq!(congrats.personal_best_lap, Some(15.0));
    assert_eq!(congrats.hof_rank, Some(1));
    assert_eq!(congrats.race_position, Some(1));
    assert!(congrats.has_achievements());
}

#[test]
fn test_hall_of_fame_clear_track_history_isolation() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");
    let profile = db.seed_default_profile_if_empty().unwrap();
    let pid = profile.id.unwrap();

    // 1. Insert records for two different tracks: "track_alpha" and "track_beta"
    let hof_a = HallOfFameEntry {
        id: None,
        track_id: "track_alpha".to_string(),
        player_name: "Racer A".to_string(),
        car_name: "Sports Coupe".to_string(),
        total_time: 60.0,
        best_lap: Some(20.0),
        laps: 3,
        created_at: "2026-09-01 10:00".to_string(),
    };
    let hof_b = HallOfFameEntry {
        id: None,
        track_id: "track_beta".to_string(),
        player_name: "Racer B".to_string(),
        car_name: "Sports Coupe".to_string(),
        total_time: 80.0,
        best_lap: Some(26.0),
        laps: 3,
        created_at: "2026-09-01 11:00".to_string(),
    };
    db.insert_entry(&hof_a).unwrap();
    db.insert_entry(&hof_b).unwrap();

    let race_a = RaceHistoryEntry {
        id: None,
        profile_id: pid,
        track_id: "track_alpha".to_string(),
        car_name: "Sports Coupe".to_string(),
        position: 1,
        total_cars: 4,
        total_time: 60.0,
        best_lap: Some(20.0),
        laps: 3,
        is_time_attack: false,
        created_at: "2026-09-01 10:00".to_string(),
        ..Default::default()
    };
    let race_b = RaceHistoryEntry {
        id: None,
        profile_id: pid,
        track_id: "track_beta".to_string(),
        car_name: "Sports Coupe".to_string(),
        position: 2,
        total_cars: 4,
        total_time: 80.0,
        best_lap: Some(26.0),
        laps: 3,
        is_time_attack: false,
        created_at: "2026-09-01 11:00".to_string(),
        ..Default::default()
    };
    db.insert_race_history(&race_a).unwrap();
    db.insert_race_history(&race_b).unwrap();

    // Verify both tracks exist
    assert_eq!(db.get_top_10("track_alpha").unwrap().len(), 1);
    assert_eq!(db.get_top_10("track_beta").unwrap().len(), 1);
    let stats_before = db.get_stats_for_profile(pid).unwrap();
    assert_eq!(stats_before.total_races, 2);
    assert!(stats_before.best_times.contains_key("track_alpha"));
    assert!(stats_before.best_times.contains_key("track_beta"));

    // 2. Clear history for track_alpha only
    db.clear_track_history("track_alpha").expect("Clear should succeed");

    // 3. Verify track_alpha is cleared but track_beta is untouched
    assert!(db.get_top_10("track_alpha").unwrap().is_empty());
    assert_eq!(db.get_top_10("track_beta").unwrap().len(), 1);

    let history_after = db.get_history_for_profile(pid, 10).unwrap();
    assert_eq!(history_after.len(), 1);
    assert_eq!(history_after[0].track_id, "track_beta");

    let stats_after = db.get_stats_for_profile(pid).unwrap();
    assert_eq!(stats_after.total_races, 1);
    assert!(!stats_after.best_times.contains_key("track_alpha"));
    assert_eq!(stats_after.best_times.get("track_beta"), Some(&26.0));
}

#[test]
fn test_race_session_circuit_history_cleared_on_editor_modify() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    // 1. Create a custom track in session and save it
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_hist_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&temp_dir);
    session.track_manager.tracks_dir = temp_dir.clone();

    let custom_track = tdrace_core::track::presets::oval_speedway();
    let saved_path = session.track_manager.save_custom_track(&custom_track, Some("my_oval")).expect("Save track");
    let track_id = "my_oval";

    session.track_choice = TrackChoice::Custom {
        id: track_id.to_string(),
        title: "My Oval".to_string(),
        description: "Custom oval".to_string(),
        path: saved_path.clone(),
    };
    session.init_race();
    assert_eq!(session.track_choice_id(), track_id);

    // 2. Complete race and record history
    session.session_time = 38.0;
    session.trackers[0].current_lap = session.total_laps + 1;
    session.trackers[0].best_lap_time = Some(12.5);
    session.check_race_finish();

    // Verify history and Hall of Fame exist
    assert_eq!(session.hof_entries.len(), session.cars.len());
    assert_eq!(session.active_profile_stats.best_times.get(track_id), Some(&12.5));
    assert!(session.profile_history.iter().any(|r| r.track_id == track_id));

    // 3. Load track into editor
    session.enter_track_editor_with_path(custom_track.clone(), Some(saved_path));

    // 4. Modify circuit and save with overwrite = true
    let mut modified_track = custom_track.clone();
    modified_track.name = "My Modified Oval".to_string();
    session.handle_editor_action(EditorAction::SaveTrack {
        name: modified_track.name.clone(),
        filename: "my_oval".to_string(),
        description: "Updated oval circuit".to_string(),
        overwrite: true,
        exit_after: false,
    });

    // 5. Verify that circuit history has been cleared!
    assert!(session.hof_entries.is_empty(), "Hall of Fame entries for modified circuit should be empty");
    assert!(!session.active_profile_stats.best_times.contains_key(track_id), "Best lap time for modified circuit should be removed");
    assert!(!session.active_profile_stats.best_circuit_times.contains_key(track_id), "Best circuit time for modified circuit should be removed");
    assert!(!session.profile_history.iter().any(|r| r.track_id == track_id), "Race history logs for modified circuit should be removed");

    // Clean up temporary directory
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_race_session_save_new_circuit_does_not_clear_other_tracks() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_new_track_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&temp_dir);
    session.track_manager.tracks_dir = temp_dir.clone();

    // 1. Establish records on track_existing
    let track_existing = tdrace_core::track::presets::oval_speedway();
    let saved_path_a = session.track_manager.save_custom_track(&track_existing, Some("track_existing")).expect("Save track");
    session.track_choice = TrackChoice::Custom {
        id: "track_existing".to_string(),
        title: "Track Existing".to_string(),
        description: "Existing track".to_string(),
        path: saved_path_a,
    };
    session.init_race();
    session.session_time = 40.0;
    session.trackers[0].current_lap = session.total_laps + 1;
    session.trackers[0].best_lap_time = Some(13.0);
    session.check_race_finish();

    assert_eq!(session.active_profile_stats.best_times.get("track_existing"), Some(&13.0));

    // 2. Open editor with a new track and save as "track_brand_new" (overwrite = false)
    let new_track = tdrace_core::track::presets::kart_arena();
    session.enter_track_editor_with_path(new_track.clone(), None);

    session.handle_editor_action(EditorAction::SaveTrack {
        name: "Brand New Track".to_string(),
        filename: "track_brand_new".to_string(),
        description: "Freshly minted circuit".to_string(),
        overwrite: false,
        exit_after: false,
    });

    // 3. Verify that track_existing's records remain intact
    assert_eq!(session.active_profile_stats.best_times.get("track_existing"), Some(&13.0));
    assert!(session.profile_history.iter().any(|r| r.track_id == "track_existing"));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_clear_bot_hall_of_fame_preserves_human_records() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");

    // 1. Seed human profile ("Racer One" / "Apex Legend")
    let profile = db.seed_default_profile_if_empty().expect("Seed profile");
    assert_eq!(profile.alias, "Apex Legend");

    // 2. Insert mix of human and AI bot records
    let entries = [
        ("Apex Legend", 65.0),
        ("Apex Legend (P1)", 66.2),
        ("Viper Frost", 67.0),
        ("Thunder Rossi", 68.5),
        ("Der Meister", 70.0),
    ];

    for (name, total_time) in entries {
        db.insert_entry(&HallOfFameEntry {
            id: None,
            track_id: "classic_grand_prix".to_string(),
            player_name: name.to_string(),
            car_name: "GT Sports Coupe".to_string(),
            total_time,
            best_lap: Some(total_time / 3.0),
            laps: 3,
            created_at: "2026-09-01 10:00".to_string(),
        }).unwrap();
    }

    let top_before = db.get_top_10("classic_grand_prix").unwrap();
    assert_eq!(top_before.len(), 5);

    // 3. Clear bot records
    db.clear_bot_hall_of_fame().unwrap();

    // 4. Verify only human records remain
    let top_after = db.get_top_10("classic_grand_prix").unwrap();
    assert_eq!(top_after.len(), 2);
    assert_eq!(top_after[0].player_name, "Apex Legend");
    assert_eq!(top_after[1].player_name, "Apex Legend (P1)");

    // 5. Test RaceSession integration
    let mut session = RaceSession::new();
    session.hof_db = Some(db);
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.refresh_profiles_and_stats();
    session.refresh_hof_entries();

    assert_eq!(session.hof_entries.len(), 2);

    // Add a bot entry to DB and refresh
    if let Some(db_ref) = &session.hof_db {
        db_ref.insert_entry(&HallOfFameEntry {
            id: None,
            track_id: "classic_grand_prix".to_string(),
            player_name: "Oversteer Reed".to_string(),
            car_name: "GT Sports Coupe".to_string(),
            total_time: 69.0,
            best_lap: Some(23.0),
            laps: 3,
            created_at: "2026-09-01 10:05".to_string(),
        }).unwrap();
    }
    session.refresh_hof_entries();
    assert_eq!(session.hof_entries.len(), 3);

    // Run clear_bot_history
    session.clear_bot_history();
    assert_eq!(session.hof_entries.len(), 2);
    assert!(session.hof_entries.iter().all(|e| e.player_name.starts_with("Apex Legend")));
}
