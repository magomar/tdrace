use tdrace_app::db::{HallOfFameDb, HallOfFameEntry};
use tdrace_app::game::{GameState, RaceSession};
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

    // Verify automatic transition to Finished state without manual name entry
    assert_eq!(session.state, GameState::Finished);
    assert!(session.show_hall_of_fame);

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

