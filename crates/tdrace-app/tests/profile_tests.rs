use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::profile::{CountryRegistry, PlayerProfile, RaceHistoryEntry};
use tdrace_app::render::color::CarColorScheme;
use tdrace_app::ui::menu::TrackChoice;

#[test]
fn test_profile_schema_and_crud() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");

    // 1. Initial table should be empty, seed default profile
    let default_prof = db.seed_default_profile_if_empty().expect("Seed default profile");
    assert!(default_prof.is_active);
    assert_eq!(default_prof.name, "Racer One");
    assert_eq!(default_prof.alias, "Apex Legend");
    assert_eq!(default_prof.country.as_deref(), Some("ESP"));

    // 2. Fetch all profiles
    let profiles = db.get_all_profiles().expect("Fetch all profiles");
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, default_prof.id);

    // 3. Create a second profile and make it active
    let p2 = PlayerProfile {
        id: None,
        name: "Carlos Sainz".to_string(),
        alias: "Smooth Operator".to_string(),
        country: Some("ESP".to_string()),
        color_scheme: CarColorScheme::from_index(4),
        is_active: true,
        created_at: "2026-08-27 10:00".to_string(),
    };
    let p2_id = db.create_profile(&p2).expect("Insert profile 2");

    let active = db.get_active_profile().expect("Get active profile");
    assert_eq!(active.id, Some(p2_id));
    assert_eq!(active.alias, "Smooth Operator");

    // Verify first profile is now inactive
    let all_now = db.get_all_profiles().expect("Fetch all");
    assert_eq!(all_now.len(), 2);
    let p1_updated = all_now.iter().find(|p| p.id == default_prof.id).unwrap();
    assert!(!p1_updated.is_active);

    // 4. Update profile 2
    let mut p2_modified = active;
    p2_modified.name = "Carlos Sainz Jr".to_string();
    db.update_profile(&p2_modified).expect("Update profile");

    let p2_fetched = db.get_profile_by_id(p2_id).expect("Fetch by id").expect("Must exist");
    assert_eq!(p2_fetched.name, "Carlos Sainz Jr");

    // 5. Delete profile 2 -> default profile should automatically reactivate
    db.delete_profile(p2_id).expect("Delete profile 2");
    let after_delete = db.get_all_profiles().expect("Fetch all");
    assert_eq!(after_delete.len(), 1);
    assert!(after_delete[0].is_active);
    assert_eq!(after_delete[0].id, default_prof.id);
}

#[test]
fn test_race_history_logging_and_career_stats() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");
    let profile = db.seed_default_profile_if_empty().expect("Seed default profile");
    let pid = profile.id.expect("Profile ID must exist");

    // Initial stats should be zeros
    let initial_stats = db.get_stats_for_profile(pid).expect("Query stats");
    assert_eq!(initial_stats.total_races, 0);
    assert_eq!(initial_stats.wins, 0);
    assert_eq!(initial_stats.podiums, 0);
    assert_eq!(initial_stats.win_rate, 0.0);

    // Insert 4 simulated race history records:
    // Race 1: P1 (Win) on classic_grand_prix, 3 laps, best lap 24.5s
    // Race 2: P2 (Podium) on oval_speedway, 3 laps, best lap 13.8s
    // Race 3: P1 (Win) on drift_park, 3 laps, best lap 22.1s
    // Race 4: P5 (No podium) on kart_arena, 3 laps, best lap 17.5s
    let races = [
        RaceHistoryEntry {
            id: None,
            profile_id: pid,
            track_id: "classic_grand_prix".to_string(),
            car_name: "GT Sports Coupe".to_string(),
            position: 1,
            total_cars: 6,
            total_time: 74.2,
            best_lap: Some(24.5),
            laps: 3,
            is_time_attack: false,
            created_at: "2026-08-27 10:00".to_string(),
        },
        RaceHistoryEntry {
            id: None,
            profile_id: pid,
            track_id: "oval_speedway".to_string(),
            car_name: "AWD Turbo Rally".to_string(),
            position: 2,
            total_cars: 6,
            total_time: 42.1,
            best_lap: Some(13.8),
            laps: 3,
            is_time_attack: false,
            created_at: "2026-08-27 10:15".to_string(),
        },
        RaceHistoryEntry {
            id: None,
            profile_id: pid,
            track_id: "drift_park".to_string(),
            car_name: "Tuned Drift Spec".to_string(),
            position: 1,
            total_cars: 6,
            total_time: 68.0,
            best_lap: Some(22.1),
            laps: 3,
            is_time_attack: false,
            created_at: "2026-08-27 10:30".to_string(),
        },
        RaceHistoryEntry {
            id: None,
            profile_id: pid,
            track_id: "kart_arena".to_string(),
            car_name: "125cc Shifter Kart".to_string(),
            position: 5,
            total_cars: 6,
            total_time: 56.4,
            best_lap: Some(17.5),
            laps: 3,
            is_time_attack: false,
            created_at: "2026-08-27 10:45".to_string(),
        },
    ];

    for r in &races {
        db.insert_race_history(r).expect("Insert history");
    }

    // Query history logs
    let logs = db.get_history_for_profile(pid, 10).expect("Get history");
    assert_eq!(logs.len(), 4);
    assert_eq!(logs[0].track_id, "kart_arena", "Should be sorted DESC by id");

    // Aggregate statistics
    let stats = db.get_stats_for_profile(pid).expect("Get aggregated stats");
    assert_eq!(stats.total_races, 4);
    assert_eq!(stats.wins, 2);
    assert_eq!(stats.podiums, 3);
    assert_eq!(stats.total_laps, 12);
    assert_eq!(stats.win_rate, 50.0);
    assert_eq!(stats.podium_rate, 75.0);
    assert_eq!(stats.best_times.get("classic_grand_prix"), Some(&24.5));
    assert_eq!(stats.best_times.get("oval_speedway"), Some(&13.8));
    assert_eq!(stats.best_times.get("drift_park"), Some(&22.1));
    assert_eq!(stats.best_times.get("kart_arena"), Some(&17.5));
}

#[test]
fn test_country_registry_and_banner_metadata() {
    let esp = CountryRegistry::find_by_code("ESP").expect("Must find Spain");
    assert_eq!(esp.name, "Spain");
    assert_eq!(esp.flag_emoji, "🇪🇸");

    let usa = CountryRegistry::find_by_code("usa").expect("Must find USA case-insensitively");
    assert_eq!(usa.code, "USA");
    assert_eq!(usa.flag_emoji, "🇺🇸");

    let jpn = CountryRegistry::find_by_code("JPN").expect("Must find Japan");
    assert_eq!(jpn.name, "Japan");

    assert!(CountryRegistry::find_by_code("XYZ").is_none());

    let profile_esp = PlayerProfile::new("Mario", "Gomez", Some("ESP"), CarColorScheme::default());
    assert_eq!(profile_esp.country_name(), "Spain");
    assert_eq!(profile_esp.country_emoji(), "🇪🇸");

    let profile_none = PlayerProfile::new("Anonymous", "Ghost", None, CarColorScheme::default());
    assert_eq!(profile_none.country_name(), "International");
    assert_eq!(profile_none.country_emoji(), "🏁");
}

#[test]
fn test_race_session_profile_integration_and_race_finish_logging() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);

    // Create a custom active profile
    let custom_livery = CarColorScheme::from_index(5);
    let custom_profile = PlayerProfile {
        id: None,
        name: "Lewis Hamilton".to_string(),
        alias: "Hammer Time".to_string(),
        country: Some("GBR".to_string()),
        color_scheme: custom_livery,
        is_active: true,
        created_at: "2026-08-27 11:00".to_string(),
    };

    if let Some(db) = &session.hof_db {
        let new_id = db.create_profile(&custom_profile).unwrap();
        db.set_active_profile(new_id).unwrap();
    }
    session.refresh_profiles_and_stats();

    assert_eq!(session.active_profile.alias, "Hammer Time");
    assert_eq!(session.active_profile.country.as_deref(), Some("GBR"));

    // Initialize race: player car should use custom livery
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();

    assert_eq!(session.color_schemes[0], session.active_profile.color_scheme);
    assert_eq!(session.color_schemes[0].to_hex_strings(), custom_livery.to_hex_strings());
    assert_eq!(session.cars.len(), 6); // 1 player + 5 bots default

    // Simulate winning race completion
    session.trackers[0].current_lap = 4; // Completed 3 laps
    session.trackers[0].best_lap_time = Some(23.4);
    session.session_time = 71.5;

    session.check_race_finish();

    // Verify race result was logged to history in DB
    let pid = session.active_profile.id.unwrap();
    if let Some(db) = &session.hof_db {
        let history = db.get_history_for_profile(pid, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].track_id, "classic_grand_prix");
        assert_eq!(history[0].position, 1);
        assert_eq!(history[0].best_lap, Some(23.4));

        let stats = db.get_stats_for_profile(pid).unwrap();
        assert_eq!(stats.total_races, 1);
        assert_eq!(stats.wins, 1);
    }

    // Verify NameEntry prefilled with active profile alias
    if let GameState::NameEntry { input_name, .. } = &session.state {
        assert_eq!(input_name, "Hammer Time");
    }
}

#[test]
fn test_profile_editing_workflow() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    let default_prof = session.active_profile.clone();
    let pid = default_prof.id.expect("Default profile should have an ID");

    // 1. Enter edit mode for default profile
    let country_idx = default_prof
        .country
        .as_deref()
        .and_then(|code| {
            CountryRegistry::ALL
                .iter()
                .position(|c| c.code.eq_ignore_ascii_case(code))
                .map(|pos| pos + 1)
        })
        .unwrap_or(0);

    session.state = GameState::ProfileCreate {
        editing_id: Some(pid),
        field_idx: 0,
        input_name: default_prof.name.clone(),
        input_alias: default_prof.alias.clone(),
        country_idx,
        livery_idx: 2,
        cursor_timer: 0.0,
    };

    // 2. Verify editing_id is set
    if let GameState::ProfileCreate { editing_id, .. } = session.state {
        assert_eq!(editing_id, Some(pid));
    } else {
        panic!("Expected ProfileCreate state");
    }

    // 3. Save modified fields
    let updated_scheme = CarColorScheme::from_index(2);
    let mut updated = PlayerProfile::new("Fernando Alonso", "El Nano", Some("ESP"), updated_scheme);
    updated.id = Some(pid);
    updated.is_active = true;

    if let Some(db) = &session.hof_db {
        db.update_profile(&updated).expect("Update profile in DB");
    }
    session.refresh_profiles_and_stats();

    // 4. Verify updated values
    assert_eq!(session.active_profile.id, Some(pid));
    assert_eq!(session.active_profile.name, "Fernando Alonso");
    assert_eq!(session.active_profile.alias, "El Nano");
    assert_eq!(session.active_profile.country.as_deref(), Some("ESP"));
    assert_eq!(
        session.active_profile.color_scheme.to_hex_strings(),
        updated_scheme.to_hex_strings()
    );
}
