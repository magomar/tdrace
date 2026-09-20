use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::profile::{CountryRegistry, ModuleCareerProgress, PlayerProfile, RaceHistoryEntry};
use tdrace_app::render::color::CarColorScheme;
use tdrace_app::ui::menu::TrackChoice;
use tdrace_core::physics::config::AssistProfile;

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
        last_mode: AssistProfile::Arcade,
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
    assert_eq!(stats.best_circuit_times.get("classic_grand_prix"), Some(&74.2));
    assert_eq!(stats.best_circuit_times.get("oval_speedway"), Some(&42.1));
    assert_eq!(stats.best_circuit_times.get("drift_park"), Some(&68.0));
    assert_eq!(stats.best_circuit_times.get("kart_arena"), Some(&56.4));
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
        last_mode: AssistProfile::Arcade,
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
    assert_eq!(session.cars.len(), 8); // 1 player + 7 bots default (8 pilots)

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

    // Verify automatic transition to Finished and HOF populated with active profile alias
    assert_eq!(session.state, GameState::Finished);
    assert_eq!(session.hof_entries[0].player_name, "Hammer Time");
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

#[test]
fn test_clear_profile_history_and_hall_of_fame() {
    use tdrace_app::db::HallOfFameEntry;

    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");

    // 1. Seed default profile and create a secondary profile
    let p1 = db.seed_default_profile_if_empty().expect("Seed default profile");
    let p1_id = p1.id.expect("Profile 1 ID");

    let p2 = PlayerProfile {
        id: None,
        name: "Carlos Sainz".to_string(),
        alias: "Smooth Operator".to_string(),
        country: Some("ESP".to_string()),
        color_scheme: CarColorScheme::from_index(3),
        is_active: false,
        created_at: "2026-09-01 10:00".to_string(),
        last_mode: AssistProfile::Arcade,
    };
    let p2_id = db.create_profile(&p2).expect("Insert profile 2");

    // 2. Insert race history logs for both profiles
    let race_p1 = RaceHistoryEntry {
        id: None,
        profile_id: p1_id,
        track_id: "classic_grand_prix".to_string(),
        car_name: "GT Sports Coupe".to_string(),
        position: 1,
        total_cars: 8,
        total_time: 75.0,
        best_lap: Some(24.0),
        laps: 3,
        is_time_attack: false,
        created_at: "2026-09-01 10:10".to_string(),
    };
    db.insert_race_history(&race_p1).unwrap();

    let race_p2 = RaceHistoryEntry {
        id: None,
        profile_id: p2_id,
        track_id: "classic_grand_prix".to_string(),
        car_name: "AWD Turbo Rally".to_string(),
        position: 2,
        total_cars: 8,
        total_time: 78.0,
        best_lap: Some(25.5),
        laps: 3,
        is_time_attack: false,
        created_at: "2026-09-01 10:15".to_string(),
    };
    db.insert_race_history(&race_p2).unwrap();

    // 3. Insert Hall of Fame records for P1 (solo + split-screen), P2, and a Bot
    db.insert_entry(&HallOfFameEntry {
        id: None,
        track_id: "classic_grand_prix".to_string(),
        player_name: "Apex Legend".to_string(),
        car_name: "GT Sports Coupe".to_string(),
        total_time: 75.0,
        best_lap: Some(24.0),
        laps: 3,
        created_at: "2026-09-01 10:10".to_string(),
    }).unwrap();
    db.insert_entry(&HallOfFameEntry {
        id: None,
        track_id: "classic_grand_prix".to_string(),
        player_name: "Apex Legend (P1)".to_string(),
        car_name: "GT Sports Coupe".to_string(),
        total_time: 76.5,
        best_lap: Some(24.5),
        laps: 3,
        created_at: "2026-09-01 10:20".to_string(),
    }).unwrap();
    db.insert_entry(&HallOfFameEntry {
        id: None,
        track_id: "classic_grand_prix".to_string(),
        player_name: "Smooth Operator".to_string(),
        car_name: "AWD Turbo Rally".to_string(),
        total_time: 78.0,
        best_lap: Some(25.5),
        laps: 3,
        created_at: "2026-09-01 10:15".to_string(),
    }).unwrap();
    db.insert_entry(&HallOfFameEntry {
        id: None,
        track_id: "classic_grand_prix".to_string(),
        player_name: "The Stig".to_string(),
        car_name: "GT Sports Coupe".to_string(),
        total_time: 80.0,
        best_lap: Some(26.0),
        laps: 3,
        created_at: "2026-09-01 10:10".to_string(),
    }).unwrap();

    // Verify stats before clearing
    let stats_before = db.get_stats_for_profile(p1_id).unwrap();
    assert_eq!(stats_before.total_races, 1);
    assert_eq!(stats_before.wins, 1);

    // 4. Clear historical data for Profile 1 only
    db.clear_profile_historical_data(p1_id, &p1.alias).unwrap();

    // Verify Profile 1 historical data is wiped
    let p1_history = db.get_history_for_profile(p1_id, 10).unwrap();
    assert!(p1_history.is_empty(), "Profile 1 history should be empty");
    let stats_after = db.get_stats_for_profile(p1_id).unwrap();
    assert_eq!(stats_after.total_races, 0);
    assert_eq!(stats_after.wins, 0);
    assert!(stats_after.best_times.is_empty());

    // Verify Profile 1 identity remains intact
    let p1_fetched = db.get_profile_by_id(p1_id).unwrap().unwrap();
    assert_eq!(p1_fetched.name, "Racer One");
    assert_eq!(p1_fetched.alias, "Apex Legend");

    // Verify Profile 2 history and HOF remain intact
    let p2_history = db.get_history_for_profile(p2_id, 10).unwrap();
    assert_eq!(p2_history.len(), 1, "Profile 2 history should remain intact");

    // Verify Hall of Fame: Apex Legend entries removed, Smooth Operator & The Stig preserved
    let top = db.get_top_10("classic_grand_prix").unwrap();
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].player_name, "Smooth Operator");
    assert_eq!(top[1].player_name, "The Stig");

    // 5. Verify RaceSession integration
    let mut session = RaceSession::new();
    session.hof_db = Some(db);
    session.refresh_profiles_and_stats();
    assert_eq!(session.active_profile.id, Some(p1_id));

    // Simulate winning race to populate in-memory session caches
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.init_race();
    session.trackers[0].current_lap = 4;
    session.trackers[0].best_lap_time = Some(23.0);
    session.session_time = 70.0;
    session.check_race_finish();

    assert_eq!(session.profile_history.len(), 1);
    assert_eq!(session.active_profile_stats.total_races, 1);
    assert!(session.hof_entries.iter().any(|e| e.player_name == "Apex Legend"));

    // Clear active profile history via session method
    session.clear_profile_history(p1_id);

    assert!(session.profile_history.is_empty(), "Session profile history should be empty");
    assert_eq!(session.active_profile_stats.total_races, 0, "Session career stats should reset");
    assert!(!session.hof_entries.iter().any(|e| e.player_name == "Apex Legend"), "Session HOF should not have Apex Legend");
    assert_eq!(session.active_profile.alias, "Apex Legend", "Active profile identity preserved");
}

#[test]
fn test_module_career_progress_persistence_and_xp_leveling() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");
    let profile = db.seed_default_profile_if_empty().expect("Seed default profile");
    let pid = profile.id.expect("Profile ID must exist");

    // 1. Initial get_or_create for GT should return Level 1 starter progress with 0 XP and Supra GT4 starter
    let mut progress: ModuleCareerProgress = db.get_or_create_module_progress(pid, "gt").expect("Query or create progress");
    assert_eq!(progress.profile_id, pid);
    assert_eq!(progress.module_id, "gt");
    assert_eq!(progress.level, 1);
    assert_eq!(progress.xp, 0);
    assert_eq!(progress.lifetime_xp, 0);
    assert!(progress.is_car_unlocked("gt_toyota_supra_gt4", false));
    assert!(progress.is_car_unlocked("gt4_clubsport", false));
    assert!(!progress.is_car_unlocked("gt3_evo", false));
    assert!(!progress.is_car_unlocked("gt2_biturbo", false));
    assert!(!progress.is_car_unlocked("gt1_legend", false));
    assert!(!progress.is_car_unlocked("hypercar_prototype", false));

    // In dev mode, everything is unlocked
    assert!(progress.is_car_unlocked("gt3_evo", true));
    assert!(progress.is_track_unlocked("monaco", true));

    // Check Starter circuits
    assert!(progress.is_track_unlocked("monza", false));
    assert!(progress.is_track_unlocked("red_bull_ring", false));
    assert!(progress.is_track_unlocked("nurburgring_gp", false));
    assert!(!progress.is_track_unlocked("silverstone", false));
    assert!(!progress.is_track_unlocked("spa", false));

    // 2. Add XP: increases spendable XP and lifetime XP, level remains 1 until advanced
    progress.add_xp(2500);
    assert_eq!(progress.xp, 2500);
    assert_eq!(progress.lifetime_xp, 2500);
    assert_eq!(progress.level, 1);

    // Cannot advance without a championship podium finish
    assert!(!progress.can_advance_tier(), "Cannot advance without championship podium");
    assert!(progress.advance_tier().is_err());

    // Award championship podium trophy (e.g. Gold)
    progress.trophies_gold = 1;
    assert!(progress.can_advance_tier(), "Eligible to advance: has podium and >= 2000 XP for Tier 2 car");

    // Advance tier to Tier 2
    let new_lvl = progress.advance_tier().expect("Advance tier");
    assert_eq!(new_lvl, 2);
    assert_eq!(progress.level, 2);
    assert!(progress.is_track_unlocked("silverstone", false));
    assert!(progress.is_track_unlocked("catalunya", false));
    assert!(progress.is_track_unlocked("bathurst", false));

    // In Tier 2, cars are NOT automatically unlocked; they must be purchased with spendable XP!
    assert!(!progress.is_car_unlocked("gt3_evo", false));
    assert!(progress.can_buy_car("gt3_evo", 2));
    progress.buy_car("gt3_evo", 2).expect("Buy Tier 2 GT3 car");
    assert!(progress.is_car_unlocked("gt3_evo", false));
    // XP reduced by 2,000 (2,500 - 2,000 = 500), but lifetime XP remains 2,500
    assert_eq!(progress.xp, 500);
    assert_eq!(progress.lifetime_xp, 2500);

    // Save and verify persistence in SQLite
    db.save_module_progress(&progress).expect("Save progress");
    let fetched = db.get_module_progress(pid, "gt").expect("Fetch progress").expect("Must exist");
    assert_eq!(fetched.level, 2);
    assert_eq!(fetched.xp, 500);
    assert_eq!(fetched.lifetime_xp, 2500);
    assert_eq!(fetched.trophies_gold, 1);
    assert!(fetched.is_car_unlocked("gt3_evo", false));
    assert!(fetched.is_track_unlocked("bathurst", false));

    // 3. Verify module independence: progress in rally is completely separate
    let rally_progress = db.get_or_create_module_progress(pid, "rally").expect("Rally progress");
    assert_eq!(rally_progress.module_id, "rally");
    assert_eq!(rally_progress.level, 1);
    assert_eq!(rally_progress.xp, 0);
}

#[test]
fn test_gt_career_session_gating_and_cup_launch() {
    use tdrace_app::ui::menu::{CarChoice, GameMode};

    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    let profile = mem_db.seed_default_profile_if_empty().unwrap();
    let pid = profile.id.unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    assert_eq!(session.active_profile.id, Some(pid));
    assert_eq!(session.active_career_progress.level, 1);
    assert_eq!(session.active_career_progress.xp, 0);

    // Switch to GT module
    session.switch_to_gt();
    assert_eq!(session.active_module_id, "gt");

    // Level 1 vehicle gating checks: GT4 unlocked, GT3/GT2/GT1/Hypercar locked
    assert!(session.is_car_unlocked(CarChoice::GT4Clubsport));
    assert!(!session.is_car_unlocked(CarChoice::GT3Car));
    assert!(!session.is_car_unlocked(CarChoice::GT2Biturbo));
    assert!(!session.is_car_unlocked(CarChoice::GT1Legend));
    assert!(!session.is_car_unlocked(CarChoice::HypercarPrototype));

    // Level 1 circuit gating checks
    assert!(session.is_track_unlocked("monza"));
    assert!(session.is_track_unlocked("red_bull_ring"));
    assert!(session.is_track_unlocked("nurburgring_gp"));
    assert!(!session.is_track_unlocked("silverstone"));
    assert!(!session.is_track_unlocked("bathurst"));
    assert!(!session.is_track_unlocked("le_mans_sarthe"));
    assert!(!session.is_track_unlocked("monaco"));

    // Dev mode bypass: all cars and circuits unlocked immediately
    session.config.gameplay.dev_mode = true;
    assert!(session.is_car_unlocked(CarChoice::GT3Car));
    assert!(session.is_car_unlocked(CarChoice::HypercarPrototype));
    assert!(session.is_track_unlocked("silverstone"));
    assert!(session.is_track_unlocked("bathurst"));
    assert!(session.is_track_unlocked("monaco"));

    // Reset dev mode
    session.config.gameplay.dev_mode = false;
    assert!(!session.is_car_unlocked(CarChoice::GT3Car));
    assert!(!session.is_track_unlocked("silverstone"));

    // Verify GT Career Tier launch with cumulative calendars (3, 6, 9, 12, 15)
    session.start_gt_career_tier(1);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::GT4Clubsport);
    assert!(session.championship_session.is_some());
    let champ1 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ1.track_ids.len(), 3);
    assert_eq!(champ1.track_ids, vec!["monza", "red_bull_ring", "nurburgring_gp"]);

    session.start_gt_career_tier(2);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::GT3Car);
    let champ2 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ2.track_ids.len(), 6);
    assert_eq!(
        champ2.track_ids,
        vec!["monza", "red_bull_ring", "nurburgring_gp", "silverstone", "catalunya", "bathurst"]
    );

    session.start_gt_career_tier(4);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::GT1Legend);
    let champ4 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ4.track_ids.len(), 12);

    session.start_gt_career_tier(5);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::HypercarPrototype);
    let champ5 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ5.track_ids.len(), 15);

    // Test race completion in GT awards metric distance XP, finish duplication, and first-time bonus
    session.start_gt_career_tier(1);
    session.total_laps = 3;
    session.trackers[0].current_lap = 4; // finished 3 laps
    session.trackers[0].best_lap_time = Some(21.0);
    session.session_time = 65.0;
    session.check_race_finish();

    assert!(session.active_career_progress.xp > 0);
    let receipt = session.last_xp_receipt.as_ref().expect("Receipt present");
    assert_eq!(receipt.completed_laps, 3);
    assert_eq!(receipt.completion_bonus, receipt.lap_xp, "Finish bonus duplicates lap XP");
    assert_eq!(receipt.first_time_bonus, 250, "Tier 1 first-time bonus is 250 XP");
    assert_eq!(receipt.total_xp, receipt.lap_xp + receipt.completion_bonus + 250);
    assert_eq!(session.active_career_progress.xp, receipt.total_xp);

    // Verify persisted to DB
    if let Some(db) = &session.hof_db {
        let saved = db.get_module_progress(pid, "gt").unwrap().unwrap();
        assert_eq!(saved.xp, session.active_career_progress.xp);
        assert!(saved.visited_tracks.contains(&"monza".to_string()));
    }
}

#[test]
fn test_circuit_defaults_unlocked_and_dev_mode_unblocks_all() {
    use tdrace_app::ui::menu::CarChoice;

    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    let _ = mem_db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    // At Level 1, switch to GT
    session.switch_to_gt();
    assert_eq!(session.active_career_progress.level, 1);

    // Initial default circuit Monza must have an UNBLOCKED predefined car (GT4 Clubsport)
    let monza_car = session.resolve_predefined_car();
    assert_eq!(monza_car, CarChoice::GT4Clubsport);
    assert_eq!(session.active_player_car_choice(), CarChoice::GT4Clubsport);
    assert!(
        session.is_car_unlocked(monza_car),
        "Initially open circuit Monza's predefined car must NOT be blocked at Level 1"
    );

    // Dev mode bypass verification via session.config.gameplay.dev_mode
    session.config.gameplay.dev_mode = false;
    assert!(!session.is_car_unlocked(CarChoice::GT3Car));
    assert!(!session.is_car_unlocked(CarChoice::HypercarPrototype));
    assert!(!session.is_track_unlocked("silverstone"));
    assert!(!session.is_track_unlocked("monaco"));

    session.config.gameplay.dev_mode = true;
    assert!(session.is_dev_mode());
    assert!(session.is_car_unlocked(CarChoice::GT3Car));
    assert!(session.is_car_unlocked(CarChoice::HypercarPrototype));
    assert!(session.is_track_unlocked("silverstone"));
    assert!(session.is_track_unlocked("monaco"));

    // Dev mode bypass verification via environment variable TDRACE_DEV
    session.config.gameplay.dev_mode = false;
    std::env::set_var("TDRACE_DEV", "1");
    assert!(session.is_dev_mode());
    assert!(session.is_car_unlocked(CarChoice::GT3Car));
    assert!(session.is_car_unlocked(CarChoice::HypercarPrototype));
    assert!(session.is_track_unlocked("silverstone"));
    assert!(session.is_track_unlocked("monaco"));
    std::env::remove_var("TDRACE_DEV");
    assert!(!session.is_dev_mode());
}

#[test]
fn test_car_purchasing_with_spendable_xp_and_deduction() {
    let mut progress = ModuleCareerProgress::default_for_gt(1);
    assert_eq!(progress.xp, 0);
    assert_eq!(progress.lifetime_xp, 0);
    assert_eq!(progress.level, 1);

    // Tier 1 cars cost 1,000 XP
    assert_eq!(ModuleCareerProgress::car_cost(1), 1000);
    // Tier 3 cars cost 3,000 XP
    assert_eq!(ModuleCareerProgress::car_cost(3), 3000);

    // Cannot buy without sufficient XP
    assert!(!progress.can_buy_car("gt4_cayman", 1));
    assert!(progress.buy_car("gt4_cayman", 1).is_err());

    // Add 1,500 XP
    progress.add_xp(1500);
    assert_eq!(progress.xp, 1500);
    assert_eq!(progress.lifetime_xp, 1500);
    assert!(progress.can_buy_car("gt4_cayman", 1));

    // Buy Tier 1 car
    progress.buy_car("gt4_cayman", 1).expect("Purchase car");
    assert!(progress.is_car_unlocked("gt4_cayman", false));
    // XP reduced by 1,000: 1500 - 1000 = 500
    assert_eq!(progress.xp, 500);
    // Lifetime XP remains 1500
    assert_eq!(progress.lifetime_xp, 1500);

    // Cannot buy again once unlocked
    assert!(!progress.can_buy_car("gt4_cayman", 1));

    // Cannot buy Tier 2 car while still at Level 1 even if player has XP
    progress.add_xp(5000);
    assert_eq!(progress.xp, 5500);
    assert_eq!(progress.level, 1);
    assert!(!progress.can_buy_car("gt3_evo", 2), "Cannot buy car above current driver tier");
}

#[test]
fn test_two_condition_tier_advancement_gates() {
    let mut progress = ModuleCareerProgress::default_for_gt(1);
    assert_eq!(progress.level, 1);

    // Target car cost for Tier 2: 1000 * 2 = 2000 XP
    assert_eq!(progress.next_tier_target_xp(), Some(2000));

    // Case 1: Neither condition met
    assert!(!progress.can_advance_tier());

    // Case 2: Only XP condition met (5,000 XP, 0 trophies)
    progress.add_xp(5000);
    assert_eq!(progress.xp, 5000);
    assert_eq!(progress.trophies_gold + progress.trophies_silver + progress.trophies_bronze, 0);
    assert!(!progress.can_advance_tier(), "Must not advance without championship podium");
    assert!(progress.advance_tier().is_err());

    // Case 3: Only Podium condition met (spend XP down below 2000)
    progress.xp = 1500;
    progress.trophies_bronze = 1;
    assert!(!progress.can_advance_tier(), "Must not advance without sufficient XP for next-tier car");
    assert!(progress.advance_tier().is_err());

    // Case 4: Both conditions met (Podium + >= 2000 XP)
    progress.xp = 2000;
    assert!(progress.can_advance_tier(), "Can advance with podium and sufficient XP");
    let next_lvl = progress.advance_tier().expect("Advance tier");
    assert_eq!(next_lvl, 2);
    assert_eq!(progress.level, 2);

    // Tier 2 now unlocked: silverstone, catalunya, bathurst
    assert!(progress.is_track_unlocked("silverstone", false));
    assert!(progress.is_track_unlocked("bathurst", false));

    // Next tier target is Tier 3 car: 1000 * 3 = 3000 XP
    assert_eq!(progress.next_tier_target_xp(), Some(3000));
}

#[test]
fn test_metric_distance_lap_xp_rounding_and_first_time_bonus() {
    // 1. Round to 10 helper
    assert_eq!(ModuleCareerProgress::round_to_10(0), 0);
    assert_eq!(ModuleCareerProgress::round_to_10(95), 100);
    assert_eq!(ModuleCareerProgress::round_to_10(104), 100);
    assert_eq!(ModuleCareerProgress::round_to_10(289), 290);
    assert_eq!(ModuleCareerProgress::round_to_10(1000), 1000);

    // 2. First-time circuit bonus: 250 XP x tier
    assert_eq!(ModuleCareerProgress::first_time_circuit_bonus(1), 250);
    assert_eq!(ModuleCareerProgress::first_time_circuit_bonus(2), 500);
    assert_eq!(ModuleCareerProgress::first_time_circuit_bonus(3), 750);
    assert_eq!(ModuleCareerProgress::first_time_circuit_bonus(4), 1000);
    assert_eq!(ModuleCareerProgress::first_time_circuit_bonus(5), 1250);

    // 3. User example: 1000m track gives 100 pts per lap; 5 laps = 500 pts; race completion duplicates = +500 pts
    let track_len_m = 1000.0f32;
    let per_lap_xp = ModuleCareerProgress::round_to_10((track_len_m / 10.0) as u64);
    assert_eq!(per_lap_xp, 100);
    let completed_laps = 5u64;
    let lap_xp = per_lap_xp * completed_laps;
    assert_eq!(lap_xp, 500);
    let completion_bonus = lap_xp;
    assert_eq!(completion_bonus, 500);
    let total_no_first_time = lap_xp + completion_bonus;
    assert_eq!(total_no_first_time, 1000);
}

#[test]
fn test_championship_completion_podium_trophy_awarded() {
    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    let _ = mem_db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    // Start GT Tier 1 Championship (3 rounds)
    session.start_gt_career_tier(1);
    assert!(session.championship_session.is_some());

    // Advance to final round (round 2 of 3)
    let champ = session.championship_session.as_mut().unwrap();
    champ.current_round = 2; // last round is index 2

    // Finish race as P1
    session.total_laps = 3;
    session.trackers[0].current_lap = 4;
    session.trackers[0].best_lap_time = Some(20.5);
    session.session_time = 62.0;
    session.check_race_finish();

    // Verify championship completed and Gold trophy awarded
    assert_eq!(session.active_career_progress.trophies_gold, 1);
    assert_eq!(session.state, GameState::ChampionshipStandings);
}


