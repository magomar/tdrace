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
fn test_hall_of_fame_seed_defaults() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory SQLite should initialize");

    db.seed_defaults_if_empty("classic_grand_prix").unwrap();
    let seeded = db.get_top_10("classic_grand_prix").unwrap();
    assert!(!seeded.is_empty());
    assert_eq!(seeded[0].player_name, "Apex Tanaka");


    // Idempotency: seeding again does not duplicate
    db.seed_defaults_if_empty("classic_grand_prix").unwrap();
    let seeded_again = db.get_top_10("classic_grand_prix").unwrap();
    assert_eq!(seeded.len(), seeded_again.len());
}

#[test]
fn test_race_session_hof_integration_and_qualification() {
    let mut session = RaceSession::new();
    // Swap DB to in-memory for testing
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    assert_eq!(session.track_choice_id(), "classic_grand_prix");
    assert!(!session.hof_entries.is_empty(), "Defaults should be seeded");

    // Simulate qualifying total time
    let player_time = 45.0; // Very fast time, easily qualifies for #1
    let best_lap = Some(15.0);

    let is_top = session.hof_db.as_ref().unwrap().is_top_10("classic_grand_prix", player_time).unwrap();
    assert!(is_top);

    // Enter name state simulation
    session.state = GameState::NameEntry {
        player_time,
        best_lap,
        input_name: "TEST_RACER".to_string(),
        cursor_timer: 0.0,
    };

    if let GameState::NameEntry { player_time, best_lap, input_name, .. } = &session.state {
        let entry = HallOfFameEntry {
            id: None,
            track_id: session.track_choice_id().to_string(),
            player_name: input_name.clone(),
            car_name: session.car_choice.title().to_string(),
            total_time: *player_time,
            best_lap: *best_lap,
            laps: session.total_laps,
            created_at: String::new(),
        };
        let row_id = session.hof_db.as_ref().unwrap().insert_entry(&entry).unwrap();
        session.recent_hof_id = Some(row_id);
        session.refresh_hof_entries();
        session.state = GameState::Finished;
    }

    assert_eq!(session.state, GameState::Finished);
    assert_eq!(session.hof_entries[0].player_name, "TEST_RACER");
    assert_eq!(session.hof_entries[0].total_time, 45.0);
    assert!(session.recent_hof_id.is_some());
}
