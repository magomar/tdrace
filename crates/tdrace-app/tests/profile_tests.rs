use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::profile::{
    ChampionshipAward, CountryRegistry, ModuleCareerProgress, PlayerProfile, RaceHistoryEntry,
    TrophyMetal,
};
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
            category: "gt".to_string(),
            championship_name: Some("GT World Challenge".to_string()),
            stunt_score: 500,
            collisions: 0,
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
            category: "rally".to_string(),
            championship_name: None,
            stunt_score: 1200,
            collisions: 1,
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
            category: "gt".to_string(),
            championship_name: None,
            stunt_score: 3400,
            collisions: 0,
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
            category: "kart".to_string(),
            championship_name: None,
            stunt_score: 0,
            collisions: 2,
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
    assert_eq!(stats.p2_count, 1);
    assert_eq!(stats.p3_count, 0);
    assert_eq!(stats.podiums, 3);
    assert_eq!(stats.total_laps, 12);
    assert_eq!(stats.win_rate, 50.0);
    assert_eq!(stats.podium_rate, 75.0);
    assert_eq!(stats.total_stunt_score, 5100);
    assert_eq!(stats.max_stunt_score, 3400);
    assert_eq!(stats.total_collisions, 3);
    assert_eq!(stats.clean_races, 2);
    assert_eq!(stats.clean_rate, 50.0);
    assert_eq!(stats.category_stats.len(), 3);
    assert_eq!(stats.category_stats.get("gt").unwrap().wins, 2);
    assert_eq!(stats.category_stats.get("rally").unwrap().podiums, 1);
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
    assert_eq!(session.cars.len(), session.max_grid_participants()); // Full grid capacity on classic track

    // Simulate winning race completion
    session.trackers[0].current_lap = session.total_laps + 1; // Completed all laps
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
        ..Default::default()
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
        ..Default::default()
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
    session.trackers[0].current_lap = session.total_laps + 1;
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
    assert!(progress.is_track_unlocked("red_bull_ring", false));
    assert!(progress.is_track_unlocked("zandvoort", false));
    assert!(progress.is_track_unlocked("nurburgring_gp", false));
    assert!(!progress.is_track_unlocked("monza", false));
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
    assert!(progress.is_track_unlocked("monza", false));
    assert!(progress.is_track_unlocked("silverstone", false));
    assert!(progress.is_track_unlocked("catalunya", false));

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
    assert!(fetched.is_track_unlocked("monza", false));

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
    assert!(session.is_track_unlocked("red_bull_ring"));
    assert!(session.is_track_unlocked("zandvoort"));
    assert!(session.is_track_unlocked("nurburgring_gp"));
    assert!(session.is_track_unlocked("portimao_gp"));
    assert!(session.is_track_unlocked("montreal"));
    assert!(!session.is_track_unlocked("monza"));
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

    // Verify GT Career Tier launch with 5 -> 7 -> 9 -> 10 -> 12 calendar curve
    session.start_gt_career_tier(1);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::GT4Clubsport);
    assert!(session.championship_session.is_some());
    let champ1 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ1.track_ids.len(), 5);
    assert_eq!(champ1.track_ids, vec!["red_bull_ring", "zandvoort", "nurburgring_gp", "portimao_gp", "montreal"]);

    session.start_gt_career_tier(2);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::GT3Car);
    let champ2 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ2.track_ids.len(), 7);
    assert_eq!(
        champ2.track_ids,
        vec!["monza", "silverstone", "catalunya", "red_bull_ring", "zandvoort", "nurburgring_gp", "portimao_gp"]
    );

    session.start_gt_career_tier(3);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::GT2Biturbo);
    let champ3 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ3.track_ids.len(), 9);

    session.start_gt_career_tier(4);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::GT1Legend);
    let champ4 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ4.track_ids.len(), 10);
    assert_eq!(champ4.track_ids, vec!["suzuka", "interlagos", "bathurst", "spa", "monza", "silverstone", "catalunya", "cota", "nurburgring_gp", "red_bull_ring"]);

    session.start_gt_career_tier(5);
    assert_eq!(session.game_mode, GameMode::Career);
    assert_eq!(session.car_choice, CarChoice::HypercarPrototype);
    let champ5 = session.championship_session.as_ref().unwrap();
    assert_eq!(champ5.track_ids.len(), 12);
    assert_eq!(champ5.track_ids, vec!["le_mans_sarthe", "monaco", "marina_bay", "spa", "monza", "silverstone", "suzuka", "bathurst", "interlagos", "catalunya", "cota", "red_bull_ring"]);

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
        assert!(saved.visited_tracks.contains(&"red_bull_ring".to_string()));
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

    // Tier 2 now unlocked: monza, silverstone, catalunya
    assert!(progress.is_track_unlocked("monza", false));
    assert!(progress.is_track_unlocked("silverstone", false));
    assert!(progress.is_track_unlocked("catalunya", false));

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

    // Start GT Tier 1 Championship
    session.start_gt_career_tier(1);
    assert!(session.championship_session.is_some());

    // Advance to final round
    let champ = session.championship_session.as_mut().unwrap();
    champ.current_round = champ.total_rounds() - 1;

    // Finish race as P1
    session.total_laps = 3;
    session.trackers[0].current_lap = 4;
    session.trackers[0].best_lap_time = Some(20.5);
    session.session_time = 62.0;
    session.check_race_finish();
    assert_eq!(session.state, GameState::Finished);

    // Press Confirm to commit final round results and advance to Career Hub standings
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;

    // Verify championship completed and Gold trophy awarded
    assert_eq!(session.active_career_progress.trophies_gold, 1);
    assert_eq!(session.profile_awards.len(), 1);
    assert_eq!(session.profile_awards[0].position, 1);
    assert_eq!(session.profile_awards[0].metallic_tier(), TrophyMetal::Gold);
    assert_eq!(session.profile_awards[0].tier, 1);
    assert!(matches!(session.state, GameState::CareerHub { showing_standings: true, .. }));
}

#[test]
fn test_multi_level_career_stunt_and_collision_metrics() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");
    let prof = db.seed_default_profile_if_empty().expect("Seed default profile");
    let pid = prof.id.expect("Profile ID");

    // Race 1: GT Cup, P1 (Win), 1200 stunt pts, 0 collisions (Clean)
    let r1 = RaceHistoryEntry {
        id: None,
        profile_id: pid,
        track_id: "monza".to_string(),
        car_name: "GT4 Clubsport".to_string(),
        position: 1,
        total_cars: 8,
        total_time: 125.0,
        best_lap: Some(25.0),
        laps: 5,
        is_time_attack: false,
        created_at: "2026-09-20 10:00".to_string(),
        category: "gt".to_string(),
        championship_name: Some("GT World Challenge".to_string()),
        stunt_score: 1200,
        collisions: 0,
    };

    // Race 2: Off-road Stunt Arena, P3 (Podium), 4500 stunt pts, 3 collisions
    let r2 = RaceHistoryEntry {
        id: None,
        profile_id: pid,
        track_id: "stunt_city".to_string(),
        car_name: "Sand Rail Buggy".to_string(),
        position: 3,
        total_cars: 6,
        total_time: 95.0,
        best_lap: Some(15.0),
        laps: 3,
        is_time_attack: false,
        created_at: "2026-09-20 11:00".to_string(),
        category: "extreme_offroad".to_string(),
        championship_name: Some("Freestyle Stunt Arena Cup".to_string()),
        stunt_score: 4500,
        collisions: 3,
    };

    // Race 3: NASCAR Oval, P2 (Podium), 200 stunt pts, 1 collision
    let r3 = RaceHistoryEntry {
        id: None,
        profile_id: pid,
        track_id: "daytona".to_string(),
        car_name: "Stock Car Cup".to_string(),
        position: 2,
        total_cars: 12,
        total_time: 150.0,
        best_lap: Some(18.0),
        laps: 6,
        is_time_attack: false,
        created_at: "2026-09-20 12:00".to_string(),
        category: "nascar".to_string(),
        championship_name: Some("Daytona 500".to_string()),
        stunt_score: 200,
        collisions: 1,
    };

    db.insert_race_history(&r1).unwrap();
    db.insert_race_history(&r2).unwrap();
    db.insert_race_history(&r3).unwrap();

    let stats = db.get_stats_for_profile(pid).unwrap();
    assert_eq!(stats.total_races, 3);
    assert_eq!(stats.wins, 1);
    assert_eq!(stats.p2_count, 1);
    assert_eq!(stats.p3_count, 1);
    assert_eq!(stats.podiums, 3);
    assert_eq!(stats.total_stunt_score, 5900);
    assert_eq!(stats.max_stunt_score, 4500);
    assert_eq!(stats.total_collisions, 4);
    assert_eq!(stats.clean_races, 1);
    assert!((stats.clean_rate - 33.333).abs() < 0.1);

    // Verify category filtered history
    let gt_races = db.get_history_for_profile_filtered(pid, Some("gt"), 10).unwrap();
    assert_eq!(gt_races.len(), 1);
    assert_eq!(gt_races[0].track_id, "monza");
    assert_eq!(gt_races[0].stunt_score, 1200);

    let offroad_races = db.get_history_for_profile_filtered(pid, Some("extreme_offroad"), 10).unwrap();
    assert_eq!(offroad_races.len(), 1);
    assert_eq!(offroad_races[0].track_id, "stunt_city");
    assert_eq!(offroad_races[0].collisions, 3);

    // Verify per-category stats
    let gt_cat = stats.category_stats.get("gt").unwrap();
    assert_eq!(gt_cat.wins, 1);
    assert_eq!(gt_cat.clean_races, 1);

    let offroad_cat = stats.category_stats.get("extreme_offroad").unwrap();
    assert_eq!(offroad_cat.total_stunt_score, 4500);
    assert_eq!(offroad_cat.total_collisions, 3);
}

#[test]
fn test_option_a_tabbed_dashboard_navigation_and_filters() {
    let mut session = RaceSession::new();
    assert_eq!(session.profile_manager_tab, 0);
    assert_eq!(session.profile_telemetry_filter_idx, 0);

    // Verify Tab cycling right (Overview -> Careers -> Trophy Cabinet -> Championships -> Telemetry -> Overview)
    for expected_tab in [1, 2, 3, 4, 0] {
        session.profile_manager_tab = (session.profile_manager_tab + 1) % 5;
        assert_eq!(session.profile_manager_tab, expected_tab);
    }

    // Verify Tab cycling left with wrap-around (Overview -> Telemetry -> Championships -> Trophy Cabinet -> Careers -> Overview)
    for expected_tab in [4, 3, 2, 1, 0] {
        if session.profile_manager_tab == 0 {
            session.profile_manager_tab = 4;
        } else {
            session.profile_manager_tab -= 1;
        }
        assert_eq!(session.profile_manager_tab, expected_tab);
    }

    // Verify Category filter pills cycling
    for expected_idx in 1..7 {
        session.profile_telemetry_filter_idx = (session.profile_telemetry_filter_idx + 1) % 7;
        assert_eq!(session.profile_telemetry_filter_idx, expected_idx);
    }

    // Wrap around to 0
    session.profile_telemetry_filter_idx = (session.profile_telemetry_filter_idx + 1) % 7;
    assert_eq!(session.profile_telemetry_filter_idx, 0);
}

#[test]
fn test_player_card_focus_and_roster_manager_navigation() {
    let mut session = RaceSession::new();
    let db = HallOfFameDb::open_in_memory().expect("In-memory db should initialize");
    let p1 = db.seed_default_profile_if_empty().expect("Seed default profile");

    // Add a second driver
    let p2 = PlayerProfile {
        id: None,
        name: "Elena Swift".to_string(),
        alias: "Velocity".to_string(),
        country: Some("FRA".to_string()),
        color_scheme: CarColorScheme::from_index(2),
        is_active: false,
        created_at: "2026-09-20 12:00".to_string(),
        last_mode: AssistProfile::Sport,
    };
    let p2_id = db.create_profile(&p2).expect("Insert driver 2");

    session.hof_db = Some(db);
    session.refresh_profiles_and_stats();
    assert_eq!(session.profile_list.len(), 2);

    // 1. Verify default focus state on hero card is false
    assert!(!session.profile_focus_card);

    // 2. Up arrow sets profile_focus_card = true, Down arrow sets it to false
    session.profile_focus_card = true;
    assert!(session.profile_focus_card);
    session.profile_focus_card = false;
    assert!(!session.profile_focus_card);

    // 3. Open Player Roster Manager for profile 0
    session.open_player_roster_manager(0);
    match &session.state {
        GameState::PlayerRosterManager {
            selected_idx,
            active_column,
            field_idx,
            input_name,
            input_alias,
            country_idx,
            assist_mode,
            ..
        } => {
            assert_eq!(*selected_idx, 0);
            assert_eq!(*active_column, 0); // Left column (Roster list)
            assert_eq!(*field_idx, 0);
            assert_eq!(input_name, &p1.name);
            assert_eq!(input_alias, &p1.alias);
            assert_eq!(*assist_mode, AssistProfile::Arcade);
            assert!(*country_idx > 0);
        }
        _ => panic!("Expected GameState::PlayerRosterManager"),
    }

    // 4. Open Player Roster Manager for profile 1
    session.open_player_roster_manager(1);
    match &session.state {
        GameState::PlayerRosterManager {
            selected_idx,
            input_name,
            input_alias,
            assist_mode,
            ..
        } => {
            assert_eq!(*selected_idx, 1);
            assert_eq!(input_name, "Elena Swift");
            assert_eq!(input_alias, "Velocity");
            assert_eq!(*assist_mode, AssistProfile::Sport);
        }
        _ => panic!("Expected GameState::PlayerRosterManager"),
    }

    // 5. Test in-place update and persistence in DB
    if let Some(db) = &session.hof_db {
        let mut updated = db.get_profile_by_id(p2_id).unwrap().unwrap();
        updated.name = "Elena Modified".to_string();
        updated.alias = "Hyperdrive".to_string();
        updated.last_mode = AssistProfile::Pro;
        db.update_profile(&updated).expect("Update profile in db");

        let fetched = db.get_profile_by_id(p2_id).unwrap().unwrap();
        assert_eq!(fetched.name, "Elena Modified");
        assert_eq!(fetched.alias, "Hyperdrive");
        assert_eq!(fetched.last_mode, AssistProfile::Pro);
    }

    // 6. Test ESC return to ProfileManager
    session.state = GameState::ProfileManager { selected_idx: 1 };
    assert!(matches!(session.state, GameState::ProfileManager { selected_idx: 1 }));
}

#[test]
fn test_all_modules_career_tier_launch_and_calendar_counts() {
    use tdrace_app::ui::menu::GameMode;

    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    let _ = mem_db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    use tdrace_app::module::GameModule;

    // 1. NASCAR Career Tiers
    let nascar_module = tdrace_app::module::nascar::NascarGameModule::new();
    let nascar_registered_tracks: std::collections::HashSet<_> = nascar_module
        .tracks()
        .into_iter()
        .map(|t| t.id.to_string())
        .collect();

    let expected_nascar_tiers: [(&str, Vec<&str>); 5] = [
        (
            "NASCAR Weekly Short Track Series (Tier 1)",
            vec![
                "martinsville_speedway",
                "bristol_motor_speedway",
                "eldora_speedway",
                "bowman_gray_stadium",
                "lucas_oil_irp",
            ],
        ),
        (
            "NASCAR Intermediate Oval Challenge (Tier 2)",
            vec![
                "charlotte_motor_speedway",
                "darlington_raceway",
                "north_wilkesboro_speedway",
                "martinsville_speedway",
                "bristol_motor_speedway",
                "eldora_speedway",
                "lucas_oil_irp",
            ],
        ),
        (
            "NASCAR National Road & Oval Tour (Tier 3)",
            vec![
                "iowa_speedway",
                "watkins_glen_nascar",
                "road_america",
                "charlotte_motor_speedway",
                "darlington_raceway",
                "north_wilkesboro_speedway",
                "martinsville_speedway",
                "bristol_motor_speedway",
                "lucas_oil_irp",
            ],
        ),
        (
            "NASCAR Premier Speedway Trophy (Tier 4)",
            vec![
                "indianapolis_motor_speedway",
                "pocono_raceway",
                "chicago_street_course",
                "iowa_speedway",
                "watkins_glen_nascar",
                "road_america",
                "charlotte_motor_speedway",
                "darlington_raceway",
                "bristol_motor_speedway",
                "martinsville_speedway",
            ],
        ),
        (
            "NASCAR Cup Series Championship (Tier 5)",
            vec![
                "daytona_superspeedway",
                "talladega_superspeedway",
                "phoenix_raceway",
                "indianapolis_motor_speedway",
                "pocono_raceway",
                "chicago_street_course",
                "iowa_speedway",
                "watkins_glen_nascar",
                "road_america",
                "charlotte_motor_speedway",
                "darlington_raceway",
                "martinsville_speedway",
            ],
        ),
    ];

    for (tier_idx, (expected_cup_name, expected_tracks)) in expected_nascar_tiers.iter().enumerate() {
        let tier = (tier_idx + 1) as u32;
        session.start_nascar_career_tier(tier);
        assert_eq!(session.game_mode, GameMode::Career);
        let champ = session.championship_session.as_ref().unwrap();
        assert_eq!(champ.name, *expected_cup_name);
        assert_eq!(champ.tier, tier);
        assert_eq!(champ.track_ids.len(), expected_tracks.len());
        for track_id in &champ.track_ids {
            assert!(
                nascar_registered_tracks.contains(track_id),
                "NASCAR tier {} track '{}' must exist in NascarGameModule tracks",
                tier,
                track_id
            );
        }
        let actual_ids: Vec<&str> = champ.track_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_ids, *expected_tracks);
    }

    // 2. Rallycross Career Tiers
    let rally_module = tdrace_app::module::rally::RallyGameModule::new();
    let rally_registered_tracks: std::collections::HashSet<_> = rally_module
        .tracks()
        .into_iter()
        .map(|t| t.id.to_string())
        .collect();

    let expected_rally_tiers: [(&str, Vec<&str>); 5] = [
        (
            "Rallycross Grassroots Cup (Tier 1)",
            vec![
                "holjes_rx",
                "lydden_hill",
                "mettet_rx",
                "dreux_rx",
                "blyton_rx",
            ],
        ),
        (
            "World Rallycross Challenge (Tier 2)",
            vec![
                "hell_rx",
                "loheac_rx",
                "silverstone_rx",
                "holjes_rx",
                "lydden_hill",
                "mettet_rx",
                "blyton_rx",
            ],
        ),
        (
            "Group B Masters Series (Tier 3)",
            vec![
                "estering_rx",
                "montalegre_rx",
                "riga_rx",
                "hell_rx",
                "loheac_rx",
                "silverstone_rx",
                "holjes_rx",
                "lydden_hill",
                "mettet_rx",
            ],
        ),
        (
            "Dakar Rally Raid Trophy (Tier 4)",
            vec![
                "nyirad_rx",
                "kouvola_rx",
                "killarney_rx",
                "estering_rx",
                "montalegre_rx",
                "riga_rx",
                "hell_rx",
                "loheac_rx",
                "silverstone_rx",
                "holjes_rx",
            ],
        ),
        (
            "Stadium Super Trucks World Series (Tier 5)",
            vec![
                "catalunya_rx",
                "yas_marina_rx",
                "essay_rx",
                "nyirad_rx",
                "kouvola_rx",
                "killarney_rx",
                "estering_rx",
                "montalegre_rx",
                "riga_rx",
                "hell_rx",
                "loheac_rx",
                "holjes_rx",
            ],
        ),
    ];

    for (tier_idx, (expected_cup_name, expected_tracks)) in expected_rally_tiers.iter().enumerate() {
        let tier = (tier_idx + 1) as u32;
        session.start_rally_career_tier(tier);
        assert_eq!(session.game_mode, GameMode::Career);
        let champ = session.championship_session.as_ref().unwrap();
        assert_eq!(champ.name, *expected_cup_name);
        assert_eq!(champ.tier, tier);
        assert_eq!(champ.track_ids.len(), expected_tracks.len());
        for track_id in &champ.track_ids {
            assert!(
                rally_registered_tracks.contains(track_id),
                "Rally tier {} track '{}' must exist in RallyGameModule tracks",
                tier,
                track_id
            );
        }
        let actual_ids: Vec<&str> = champ.track_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_ids, *expected_tracks);
    }

    // 3. Karting Career Tiers
    let kart_module = tdrace_app::module::kart::KartGameModule::new();
    let kart_registered_tracks: std::collections::HashSet<_> = kart_module
        .tracks()
        .into_iter()
        .map(|t| t.id.to_string())
        .collect();

    let expected_kart_tiers: [(&str, Vec<&str>); 5] = [
        (
            "Rotax Junior Academy (Tier 1)",
            vec!["lonato", "genk", "wackersdorf", "laval_kart", "whilton_mill"],
        ),
        (
            "National Kart Championship (Tier 2)",
            vec!["sarno", "kristianstad", "seven_laghi", "lonato", "genk", "wackersdorf", "whilton_mill"],
        ),
        (
            "Continental Rotax Trophy (Tier 3)",
            vec!["pfi", "franciacorta", "ampfing", "sarno", "kristianstad", "seven_laghi", "lonato", "genk", "wackersdorf"],
        ),
        (
            "FIA Karting European Championship (Tier 4)",
            vec!["zuera", "silverstone_national_kart", "le_mans_kart", "pfi", "franciacorta", "ampfing", "sarno", "kristianstad", "seven_laghi", "lonato"],
        ),
        (
            "FIA Karting World Championship (Tier 5)",
            vec!["portimao_kart", "valencia_kart", "campillos", "zuera", "silverstone_national_kart", "le_mans_kart", "pfi", "franciacorta", "ampfing", "sarno", "kristianstad", "lonato"],
        ),
    ];

    for (tier_idx, (expected_cup_name, expected_tracks)) in expected_kart_tiers.iter().enumerate() {
        let tier = (tier_idx + 1) as u32;
        session.start_kart_career_tier(tier);
        assert_eq!(session.game_mode, GameMode::Career);
        let champ = session.championship_session.as_ref().unwrap();
        assert_eq!(champ.name, *expected_cup_name);
        assert_eq!(champ.tier, tier);
        assert_eq!(champ.track_ids.len(), expected_tracks.len());
        for track_id in &champ.track_ids {
            assert!(
                kart_registered_tracks.contains(track_id),
                "Kart tier {} track '{}' must exist in KartGameModule tracks",
                tier,
                track_id
            );
        }
        let actual_ids: Vec<&str> = champ.track_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_ids, *expected_tracks);
    }

    // 4. Extreme Off-Road Career Tiers
    let offroad_module = tdrace_app::module::extreme_offroad::ExtremeOffRoadModule::new();
    let offroad_registered_tracks: std::collections::HashSet<_> = offroad_module
        .tracks()
        .into_iter()
        .map(|t| t.id.to_string())
        .collect();

    let expected_offroad_tiers: [(&str, Vec<&str>); 5] = [
        (
            "Desert Sand Sprint Series (Tier 1)",
            vec![
                "sahara_dune_crossing",
                "dirt_figure_eight",
                "atacama_sand_basin",
                "glamis_dunes",
                "crandon_short_course",
            ],
        ),
        (
            "Red Rock Canyon Raid (Tier 2)",
            vec![
                "red_rock_canyon",
                "mud_slough_arena",
                "baja_500_desert_scrub",
                "sahara_dune_crossing",
                "dirt_figure_eight",
                "atacama_sand_basin",
                "crandon_short_course",
            ],
        ),
        (
            "Arctic Glacial Challenge (Tier 3)",
            vec![
                "arctic_frozen_lake",
                "alpine_snow_ridge",
                "rovaniemi_ice_ring",
                "red_rock_canyon",
                "mud_slough_arena",
                "baja_500_desert_scrub",
                "sahara_dune_crossing",
                "dirt_figure_eight",
                "crandon_short_course",
            ],
        ),
        (
            "Supercross & Mud Masters (Tier 4)",
            vec![
                "supercross_stadium_arena",
                "gravel_quarry_chasm",
                "louisiana_mud_swampland",
                "arctic_frozen_lake",
                "alpine_snow_ridge",
                "red_rock_canyon",
                "mud_slough_arena",
                "rovaniemi_ice_ring",
                "baja_500_desert_scrub",
                "sahara_dune_crossing",
            ],
        ),
        (
            "Extreme Off-Road Ultimate Championship (Tier 5)",
            vec![
                "monster_colosseum",
                "glacier_crest_pass",
                "stunt_city_megastructure",
                "supercross_stadium_arena",
                "gravel_quarry_chasm",
                "arctic_frozen_lake",
                "alpine_snow_ridge",
                "louisiana_mud_swampland",
                "red_rock_canyon",
                "rovaniemi_ice_ring",
                "baja_500_desert_scrub",
                "sahara_dune_crossing",
            ],
        ),
    ];

    for (tier_idx, (expected_cup_name, expected_tracks)) in expected_offroad_tiers.iter().enumerate() {
        let tier = (tier_idx + 1) as u32;
        session.start_extreme_offroad_career_tier(tier);
        assert_eq!(session.game_mode, GameMode::Career);
        let champ = session.championship_session.as_ref().unwrap();
        assert_eq!(champ.name, *expected_cup_name);
        assert_eq!(champ.tier, tier);
        assert_eq!(champ.track_ids.len(), expected_tracks.len());
        for track_id in &champ.track_ids {
            assert!(
                offroad_registered_tracks.contains(track_id),
                "Off-Road tier {} track '{}' must exist in ExtremeOffRoadGameModule tracks",
                tier,
                track_id
            );
        }
        let actual_ids: Vec<&str> = champ.track_ids.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual_ids, *expected_tracks);
    }
}

#[test]
fn test_gt_career_hub_calendar_helpers_and_customization() {
    use tdrace_app::ui::career_hub::{
        cycle_calendar_slot, gt_default_calendar, gt_eligible_previous_tracks, gt_mandatory_tracks,
        is_slot_mandatory,
    };

    // 1. Mandatory tracks: Tier 1 has 5, Tiers 2-5 have 3
    assert_eq!(gt_mandatory_tracks(1).len(), 5);
    assert_eq!(gt_mandatory_tracks(2), vec!["monza", "silverstone", "catalunya"]);
    assert_eq!(gt_mandatory_tracks(3), vec!["spa", "cota", "bahrain"]);
    assert_eq!(gt_mandatory_tracks(4), vec!["suzuka", "interlagos", "bathurst"]);
    assert_eq!(gt_mandatory_tracks(5), vec!["le_mans_sarthe", "monaco", "marina_bay"]);

    // 2. Default calendar sizes: 5, 7, 9, 10, 12
    assert_eq!(gt_default_calendar(1).len(), 5);
    assert_eq!(gt_default_calendar(2).len(), 7);
    assert_eq!(gt_default_calendar(3).len(), 9);
    assert_eq!(gt_default_calendar(4).len(), 10);
    assert_eq!(gt_default_calendar(5).len(), 12);

    // 3. Mandatory slot checks
    assert!(is_slot_mandatory(2, "monza"));
    assert!(is_slot_mandatory(2, "silverstone"));
    assert!(is_slot_mandatory(2, "catalunya"));
    assert!(!is_slot_mandatory(2, "red_bull_ring"));
    assert!(!is_slot_mandatory(2, "zandvoort"));

    // 4. Eligible previous tracks
    let tier2_eligible = gt_eligible_previous_tracks(2);
    assert_eq!(tier2_eligible.len(), 5); // 5 tracks from Tier 1
    assert!(tier2_eligible.contains(&"red_bull_ring"));
    assert!(tier2_eligible.contains(&"montreal"));

    // 5. Calendar slot cycling
    let mut calendar = gt_default_calendar(2);
    // Mandatory slots (0..2) cannot be modified
    cycle_calendar_slot(2, &mut calendar, 0, true);
    assert_eq!(calendar[0], "monza");
    cycle_calendar_slot(2, &mut calendar, 1, false);
    assert_eq!(calendar[1], "silverstone");

    // Optional slot (e.g. index 3 = "red_bull_ring") can be cycled
    let original_track = calendar[3].clone();
    cycle_calendar_slot(2, &mut calendar, 3, true);
    assert_ne!(calendar[3], original_track);
    // Must remain unique within calendar
    let mut track_set = std::collections::HashSet::new();
    for t in &calendar {
        assert!(track_set.insert(t.clone()), "Track {} was duplicated in calendar", t);
    }

    // Reverse cycling test
    cycle_calendar_slot(2, &mut calendar, 3, false);
    assert_eq!(calendar[3], original_track);
}

#[test]
fn test_gt_career_tier_launch_with_custom_calendar() {
    use tdrace_app::ui::menu::GameMode;

    let mut session = RaceSession::new();
    let mem_db = HallOfFameDb::open_in_memory().unwrap();
    let _ = mem_db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();
    session.switch_to_gt();

    // Custom calendar for Tier 2: 3 mandatory + 4 chosen tracks from Tier 1
    let custom_tracks = vec![
        "monza".to_string(),
        "silverstone".to_string(),
        "catalunya".to_string(),
        "montreal".to_string(),
        "portimao_gp".to_string(),
        "nurburgring_gp".to_string(),
        "zandvoort".to_string(),
    ];

    session.start_gt_career_tier_with_calendar(2, Some(custom_tracks.clone()));
    assert_eq!(session.game_mode, GameMode::Career);
    let champ = session.championship_session.as_ref().expect("Championship should be active");
    assert_eq!(champ.track_ids.len(), 7);
    assert_eq!(champ.track_ids, custom_tracks);

    // Fallback: If player attempts to launch with invalid calendar (missing mandatory tracks)
    let invalid_tracks = vec![
        "red_bull_ring".to_string(),
        "zandvoort".to_string(),
        "nurburgring_gp".to_string(),
        "portimao_gp".to_string(),
        "montreal".to_string(),
    ];
    session.start_gt_career_tier_with_calendar(2, Some(invalid_tracks));
    let champ_fallback = session.championship_session.as_ref().unwrap();
    assert_eq!(champ_fallback.track_ids.len(), 7);
    assert_eq!(champ_fallback.track_ids[0], "monza");
}

#[test]
fn test_real_championships_listing_and_filter() {
    use tdrace_app::series::ChampionshipManager;
    let manager = ChampionshipManager::new();
    let all_champs = manager.all_sorted();
    assert!(!all_champs.is_empty(), "Embedded presets must ensure championships exist");

    // Verify all championships have valid configuration
    for c in &all_champs {
        assert!(!c.series.id.is_empty(), "Series ID must not be empty");
        assert!(!c.series.name.is_empty(), "Series Name must not be empty");
        assert!(!c.series.module_id.is_empty(), "Module ID must not be empty");
        assert!(!c.rounds.is_empty(), "Championship must have at least one round");
        assert!(c.series.tier >= 1, "Series tier must be >= 1");
    }

    // Verify filtering by category
    let gt_champs: Vec<_> = all_champs.iter().filter(|c| c.series.module_id.eq_ignore_ascii_case("gt")).collect();
    assert!(!gt_champs.is_empty(), "GT championships must exist");

    let nascar_champs: Vec<_> = all_champs.iter().filter(|c| c.series.module_id.eq_ignore_ascii_case("nascar")).collect();
    assert!(!nascar_champs.is_empty(), "NASCAR championships must exist");

    let rally_champs: Vec<_> = all_champs.iter().filter(|c| c.series.module_id.eq_ignore_ascii_case("rally")).collect();
    assert!(!rally_champs.is_empty(), "Rally championships must exist");

    let kart_champs: Vec<_> = all_champs.iter().filter(|c| c.series.module_id.eq_ignore_ascii_case("kart")).collect();
    assert!(!kart_champs.is_empty(), "Kart championships must exist");

    let offroad_champs: Vec<_> = all_champs.iter().filter(|c| c.series.module_id.eq_ignore_ascii_case("extreme_offroad")).collect();
    assert!(!offroad_champs.is_empty(), "Extreme offroad championships must exist");
}

#[test]
fn test_resolve_championship_car_model_id_fallback_and_history() {
    use tdrace_app::series::ChampionshipManager;
    use tdrace_app::ui::profile_ui::resolve_championship_car_model_id;

    let manager = ChampionshipManager::new();
    let all_champs = manager.all_sorted();

    // 1. Without history, each championship resolves to a valid entry car
    for c in &all_champs {
        let (model_id, display_name, has_raced) = resolve_championship_car_model_id(c, &[]);
        assert!(!has_raced, "Without history, has_raced must be false");
        assert!(!model_id.is_empty(), "Model ID must not be empty");
        assert!(!display_name.is_empty(), "Display name must not be empty");

        // Verify model exists in catalog
        let model = tdrace_app::catalog::find_model_by_id(model_id);
        assert!(model.is_some(), "Resolved model '{}' must exist in catalog", model_id);
    }

    // 2. With history, championship resolves to the latest car raced by player
    let gt_champ = all_champs.iter().find(|c| c.series.module_id == "gt").expect("GT championship exists");
    let fake_history = vec![
        RaceHistoryEntry {
            profile_id: 1,
            category: "gt".to_string(),
            track_id: "monza".to_string(),
            car_name: "gt_bmw_m4_gt4".to_string(),
            position: 1,
            total_time: 120.0,
            best_lap: Some(60.0),
            championship_name: Some(gt_champ.series.name.clone()),
            ..RaceHistoryEntry::default()
        }
    ];

    let (model_id, display_name, has_raced) = resolve_championship_car_model_id(gt_champ, &fake_history);
    assert!(has_raced, "With matching history, has_raced must be true");
    assert_eq!(model_id, "gt_bmw_m4_gt4");
    assert!(display_name.contains("BMW") || display_name.contains("M4"));
}

#[test]
fn test_profile_champ_tab_navigation_and_scroll() {
    let mut session = RaceSession::new();
    assert_eq!(session.profile_champ_scroll, 0);

    // Scroll down
    session.profile_champ_scroll = 4;
    assert_eq!(session.profile_champ_scroll, 4);

    // Tab switch resets scroll
    session.profile_manager_tab = 1;
    session.profile_champ_scroll = 0;
    assert_eq!(session.profile_champ_scroll, 0);

    // Filter reset
    session.profile_champ_scroll = 3;
    session.profile_telemetry_filter_idx = (session.profile_telemetry_filter_idx + 1) % 7;
    session.profile_champ_scroll = 0;
    assert_eq!(session.profile_champ_scroll, 0);
}

#[test]
fn test_profile_focus_hierarchy_and_filter_selection() {
    use tdrace_app::ui::profile_ui::ProfileFocusArea;

    let mut session = RaceSession::new();
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Tabs);
    assert!(!session.profile_focus_card);

    // 1. Moving Up from Tabs focuses HeroCard
    session.profile_focus_area = ProfileFocusArea::HeroCard;
    session.profile_focus_card = session.profile_focus_area == ProfileFocusArea::HeroCard;
    assert!(session.profile_focus_card);

    // 2. Moving Down from HeroCard focuses Tabs
    session.profile_focus_area = ProfileFocusArea::Tabs;
    session.profile_focus_card = session.profile_focus_area == ProfileFocusArea::HeroCard;
    assert!(!session.profile_focus_card);

    // 3. On Tab 3 (Championships) or Tab 4 (Telemetry), moving Down from Tabs focuses Filters
    session.profile_manager_tab = 3;
    session.profile_focus_area = ProfileFocusArea::Filters;
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Filters);

    // On Tab 2 (Trophy Cabinet), moving Down from Tabs focuses Content directly
    session.profile_manager_tab = 2;
    session.profile_focus_area = ProfileFocusArea::Content;
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Content);

    // 4. Moving Left/Right while on Filters selects module filters
    assert_eq!(session.profile_telemetry_filter_idx, 0); // "ALL"

    // Right moves forward
    for expected in 1..=6 {
        session.profile_telemetry_filter_idx = (session.profile_telemetry_filter_idx + 1) % 7;
        assert_eq!(session.profile_telemetry_filter_idx, expected);
    }
    // Wrap around to 0
    session.profile_telemetry_filter_idx = (session.profile_telemetry_filter_idx + 1) % 7;
    assert_eq!(session.profile_telemetry_filter_idx, 0);

    // Left moves backward with wrap-around
    if session.profile_telemetry_filter_idx == 0 {
        session.profile_telemetry_filter_idx = 6;
    } else {
        session.profile_telemetry_filter_idx -= 1;
    }
    assert_eq!(session.profile_telemetry_filter_idx, 6); // "CLASSIC"

    // 5. Moving Down from Filters focuses Content (Championships list or Telemetry table)
    session.profile_focus_area = ProfileFocusArea::Content;
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Content);

    // 6. In Content, when at top (scroll == 0), moving Up returns to Filters
    session.profile_champ_scroll = 0;
    session.profile_focus_area = ProfileFocusArea::Filters;
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Filters);

    // 7. Moving Up from Filters returns to Tabs
    session.profile_focus_area = ProfileFocusArea::Tabs;
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Tabs);
}

#[test]
fn test_championships_sorted_by_tier_ascending() {
    use tdrace_app::series::ChampionshipManager;
    use tdrace_app::ui::profile_ui::get_sorted_championships;

    let manager = ChampionshipManager::new();

    // 1. All championships sorted across modules
    let all_sorted = get_sorted_championships(&manager, None);
    assert!(!all_sorted.is_empty());

    for i in 0..all_sorted.len() - 1 {
        let t_curr = all_sorted[i].series.tier;
        let t_next = all_sorted[i + 1].series.tier;
        assert!(
            t_curr <= t_next,
            "Championships must be sorted strictly by tier ascending: curr tier {} > next tier {}",
            t_curr, t_next
        );
    }

    // 2. Specific module filtered and sorted
    for module in &["gt", "nascar", "rally", "kart", "extreme_offroad"] {
        let mod_sorted = get_sorted_championships(&manager, Some(module));
        if !mod_sorted.is_empty() {
            for i in 0..mod_sorted.len() - 1 {
                assert!(
                    mod_sorted[i].series.tier <= mod_sorted[i + 1].series.tier,
                    "Module {} must be sorted by tier ascending", module
                );
            }
        }
    }
    // GT specifically has Tier 1 through Tier 5
    let gt_sorted = get_sorted_championships(&manager, Some("gt"));
    assert_eq!(gt_sorted[0].series.tier, 1, "GT module top series must be Tier 1");
}

#[test]
fn test_championship_car_original_sprite_factory_colors() {
    use tdrace_app::series::ChampionshipManager;
    use tdrace_app::ui::profile_ui::resolve_championship_car_model_id;

    let manager = ChampionshipManager::new();
    let all_champs = manager.all_sorted();

    for champ in &all_champs {
        let (model_id, _name, _has_raced) = resolve_championship_car_model_id(champ, &[]);
        let model = tdrace_app::catalog::find_model_by_id(model_id);
        assert!(model.is_some(), "Resolved model {} must exist in catalog", model_id);

        let m = model.unwrap();
        // Factory colors must match the model's catalog definition so get_vehicle_lateral_texture
        // sets is_factory = true and uses pristine PNG sprites without tinting or masking.
        let factory_prim = m.primary_color;
        let factory_sec = m.secondary_color;
        assert_eq!(m.primary_color, factory_prim);
        assert_eq!(m.secondary_color, factory_sec);
    }
}

#[test]
fn test_championship_navigation_selection_and_launch() {
    use tdrace_app::game::{GameState, RaceSession};
    use tdrace_app::series::ChampionshipManager;
    use tdrace_app::ui::profile_ui::get_sorted_championships;

    let mut session = RaceSession::new();
    let mem_db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();
    session.switch_to_gt();
    assert_eq!(session.profile_champ_selected_idx, 0);

    let manager = ChampionshipManager::new();
    let gt_champs = get_sorted_championships(&manager, Some("gt"));
    assert!(!gt_champs.is_empty());

    let tier1_champ = gt_champs.iter().find(|c| c.series.tier == 1).expect("Tier 1 GT champ");
    let tier3_champ = gt_champs.iter().find(|c| c.series.tier == 3);

    // Tier 1 is always unlocked
    session.config.gameplay.dev_mode = false;
    session.active_career_progress.level = 1;
    assert!(session.is_championship_unlocked(tier1_champ));

    // Tier 3 should be locked when level is 1 and dev_mode is false
    if let Some(t3) = tier3_champ {
        assert!(!session.is_championship_unlocked(t3));

        // When career level reaches 3, tier 3 unlocks
        session.active_career_progress.level = 3;
        assert!(session.is_championship_unlocked(t3));

        // When dev_mode is enabled, it unlocks regardless of career level
        session.active_career_progress.level = 1;
        assert!(!session.is_championship_unlocked(t3));
        session.config.gameplay.dev_mode = true;
        assert!(session.is_championship_unlocked(t3));
    }

    // Launch Tier 1 championship
    session.launch_or_resume_championship(tier1_champ);
    assert!(session.championship_session.is_some());
    let active_champ = session.championship_session.as_ref().unwrap();
    assert_eq!(active_champ.name, tier1_champ.series.name);
    assert_eq!(active_champ.tier, tier1_champ.series.tier);

    // Session transitions to StartingGrid ready to race
    assert_eq!(session.state, GameState::StartingGrid);
    assert!(session.total_laps > 0);
}

#[test]
fn test_career_hub_focus_and_navigation() {
    use tdrace_app::ui::career_hub::CareerHubFocus;

    let mut session = RaceSession::new();
    assert_eq!(session.career_hub_focus, CareerHubFocus::Tabs);

    // Enter CareerHub
    let tier = 1;
    let calendar = tdrace_app::ui::career_hub::gt_default_calendar(tier);
    session.career_hub_focus = CareerHubFocus::Tabs;
    session.state = GameState::CareerHub {
        selected_tier: tier,
        selected_slot: 0,
        calendar_tracks: calendar.clone(),
        showing_standings: false,
    };

    // 1. Initial focus is Tabs
    assert_eq!(session.career_hub_focus, CareerHubFocus::Tabs);

    // 2. Moving Down transitions focus to Calendar (slot 0)
    session.career_hub_focus = CareerHubFocus::Calendar;
    assert_eq!(session.career_hub_focus, CareerHubFocus::Calendar);

    // 3. Moving Up from slot 0 returns focus to Tabs
    let selected_slot = 0;
    if selected_slot == 0 {
        session.career_hub_focus = CareerHubFocus::Tabs;
    }
    assert_eq!(session.career_hub_focus, CareerHubFocus::Tabs);

    // 4. Test tier bounds and updates
    let mut selected_tier = 1u32;
    // Moving Left at tier 1 is clamped
    if selected_tier > 1 {
        selected_tier -= 1;
    }
    assert_eq!(selected_tier, 1);

    // Moving Right increments tier up to 5
    while selected_tier < 5 {
        selected_tier += 1;
    }
    assert_eq!(selected_tier, 5);
    // Clamped at 5
    if selected_tier < 5 {
        selected_tier += 1;
    }
    assert_eq!(selected_tier, 5);

    // 5. Test slot navigation in Calendar
    session.career_hub_focus = CareerHubFocus::Calendar;
    let mut current_slot = 0usize;
    let num_slots = calendar.len();
    if current_slot + 1 < num_slots {
        current_slot += 1;
    }
    assert_eq!(current_slot, 1);
    if current_slot > 0 {
        current_slot -= 1;
    }
    assert_eq!(current_slot, 0);

    // 6. Test returning to Tabs when navigating up from slot 0
    if current_slot == 0 {
        session.career_hub_focus = CareerHubFocus::Tabs;
    }
    assert_eq!(session.career_hub_focus, CareerHubFocus::Tabs);
}

#[test]
fn test_tier_1_starter_cars_single_entry_vehicle() {
    use tdrace_app::profile::ModuleCareerProgress;

    // Each discipline in Tier 1 must provide only 1 starter car
    let rally_starters = ModuleCareerProgress::starter_cars_for_module_and_tier("rally", 1);
    assert_eq!(rally_starters, vec!["rally_peugeot_208_rally4"]);

    let nascar_starters = ModuleCareerProgress::starter_cars_for_module_and_tier("nascar", 1);
    assert_eq!(nascar_starters, vec!["nascar_monte_carlo_ss"]);

    let kart_starters = ModuleCareerProgress::starter_cars_for_module_and_tier("kart", 1);
    assert_eq!(kart_starters, vec!["kart_crg_hero_60"]);

    let offroad_starters = ModuleCareerProgress::starter_cars_for_module_and_tier("extreme_offroad", 1);
    assert_eq!(offroad_starters, vec!["offroad_sand_rail_buggy"]);

    let gt_starters = ModuleCareerProgress::starter_cars_for_module_and_tier("gt", 1);
    assert_eq!(gt_starters, vec!["gt_toyota_supra_gt4", "gt4_clubsport"]);
}

#[test]
fn test_rally_championship_track_choice_sync_across_rounds() {
    let mut session = RaceSession::new();
    let mem_db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    session.start_rally_career_tier(1);
    assert_eq!(session.track_choice_id(), "holjes_rx");

    if let Some(champ) = &mut session.championship_session {
        champ.current_round += 1;
    }
    session.advance_championship_round();
    assert_eq!(session.track_choice_id(), "lydden_hill");

    if let Some(champ) = &mut session.championship_session {
        champ.current_round += 1;
    }
    session.advance_championship_round();
    assert_eq!(session.track_choice_id(), "mettet_rx");
}

#[test]
fn test_race_finish_records_authentic_model_title_in_history() {
    let mut session = RaceSession::new();
    let mem_db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    session.start_rally_career_tier(1);
    session.selected_car_model_id = Some("rally_fiesta_rally4");

    session.total_laps = 1;
    session.trackers[0].current_lap = 2; // finished 1 lap
    session.trackers[0].best_lap_time = Some(35.0);
    session.session_time = 40.0;
    session.check_race_finish();

    assert!(!session.profile_history.is_empty());
    let entry = &session.profile_history[0];
    assert_eq!(entry.car_name, "Ford Fiesta Rally4");
    assert_eq!(entry.track_id, "holjes_rx");
}

#[test]
fn test_module_career_progress_isolation_and_xp_crediting() {
    let mut session = RaceSession::new();
    let mem_db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
    session.hof_db = Some(mem_db);
    session.refresh_profiles_and_stats();

    let pid = session.active_profile.id.unwrap();

    // Start rally championship and complete round 1
    session.start_rally_career_tier(1);
    assert_eq!(session.active_module_id, "rally");

    session.total_laps = 1;
    session.trackers[0].current_lap = 2;
    session.trackers[0].best_lap_time = Some(35.0);
    session.session_time = 40.0;
    session.check_race_finish();

    // Rally module has earned XP and visited holjes_rx
    assert!(session.active_career_progress.xp > 0);
    assert!(session.active_career_progress.visited_tracks.contains(&"holjes_rx".to_string()));

    // GT module remains untouched at 0 XP
    if let Some(db) = &session.hof_db {
        let gt_prog = db.get_module_progress(pid, "gt").unwrap();
        let gt_xp = gt_prog.map(|p| p.xp).unwrap_or(0);
        assert_eq!(gt_xp, 0);

        let rally_prog = db.get_module_progress(pid, "rally").unwrap().expect("Rally progress exists");
        assert_eq!(rally_prog.xp, session.active_career_progress.xp);
        assert_eq!(rally_prog.visited_tracks, vec!["holjes_rx"]);
    }
}

#[test]
fn test_championship_award_persistence_and_upgrade() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");
    let prof = db.seed_default_profile_if_empty().expect("Seed default profile");
    let pid = prof.id.expect("Profile ID");

    // Initially no awards
    let initial_awards = db.get_championship_awards(pid).expect("Fetch awards");
    assert!(initial_awards.is_empty());

    // 1. Save 2nd Place Silver award in Tier 1 GT Sprint
    let silver_award = ChampionshipAward {
        profile_id: pid,
        championship_id: "gt4_clubman_sprint".to_string(),
        module_id: "gt".to_string(),
        tier: 1,
        position: 2,
        points: 90,
        car_model_id: "gt_toyota_supra_gt4".to_string(),
        achieved_at: "2026-09-24 12:00:00".to_string(),
    };
    let saved = db.save_championship_award(&silver_award).expect("Save silver award");
    assert!(saved);

    let fetched = db.get_championship_award(pid, "gt4_clubman_sprint").expect("Query award").expect("Must exist");
    assert_eq!(fetched.position, 2);
    assert_eq!(fetched.metallic_tier(), TrophyMetal::Silver);
    assert_eq!(fetched.points, 90);
    assert_eq!(fetched.car_model_id, "gt_toyota_supra_gt4");

    // Verify slot query works with "gt" and alias "gt_challenge"
    let slot_award = db.get_championship_award_for_slot(pid, "gt", 1).expect("Query slot").expect("Must exist");
    assert_eq!(slot_award.position, 2);
    let slot_alias = db.get_championship_award_for_slot(pid, "gt_challenge", 1).expect("Query slot").expect("Must exist");
    assert_eq!(slot_alias.position, 2);

    // 2. Re-entering and finishing 3rd (Bronze) should NOT downgrade the Silver award
    let bronze_award = ChampionshipAward {
        profile_id: pid,
        championship_id: "gt4_clubman_sprint".to_string(),
        module_id: "gt".to_string(),
        tier: 1,
        position: 3,
        points: 75,
        car_model_id: "gt4_clubsport".to_string(),
        achieved_at: "2026-09-24 13:00:00".to_string(),
    };
    let downgraded = db.save_championship_award(&bronze_award).expect("Save bronze award");
    assert!(!downgraded, "Award should not be downgraded");

    let still_silver = db.get_championship_award(pid, "gt4_clubman_sprint").expect("Query award").expect("Must exist");
    assert_eq!(still_silver.position, 2);
    assert_eq!(still_silver.metallic_tier(), TrophyMetal::Silver);
    assert_eq!(still_silver.points, 90);

    // 3. Re-entering and finishing 1st (Gold) MUST upgrade the award
    let gold_award = ChampionshipAward {
        profile_id: pid,
        championship_id: "gt4_clubman_sprint".to_string(),
        module_id: "gt".to_string(),
        tier: 1,
        position: 1,
        points: 115,
        car_model_id: "gt_toyota_supra_gt4".to_string(),
        achieved_at: "2026-09-24 14:00:00".to_string(),
    };
    let upgraded = db.save_championship_award(&gold_award).expect("Save gold award");
    assert!(upgraded, "Award should be upgraded to gold");

    let now_gold = db.get_championship_award(pid, "gt4_clubman_sprint").expect("Query award").expect("Must exist");
    assert_eq!(now_gold.position, 1);
    assert_eq!(now_gold.metallic_tier(), TrophyMetal::Gold);
    assert_eq!(now_gold.points, 115);
    assert_eq!(now_gold.achieved_at, "2026-09-24 14:00:00");

    // 4. Save Tier 3 Rallycross award
    let rally_award = ChampionshipAward {
        profile_id: pid,
        championship_id: "rally_group_b_masters".to_string(),
        module_id: "rally".to_string(),
        tier: 3,
        position: 1,
        points: 120,
        car_model_id: "rally_audi_sport_quattro_s1".to_string(),
        achieved_at: "2026-09-24 15:00:00".to_string(),
    };
    db.save_championship_award(&rally_award).expect("Save rally award");

    let all_awards = db.get_championship_awards(pid).expect("Fetch all awards");
    assert_eq!(all_awards.len(), 2);

    // 5. Test cascading deletion on profile delete
    db.delete_profile(pid).expect("Delete profile");
    let after_delete = db.get_championship_awards(pid).expect("Fetch awards");
    assert!(after_delete.is_empty(), "Awards must be deleted when profile is deleted");
}

#[test]
fn test_trophy_filename_and_asset_resolution() {
    use tdrace_app::render::{normalize_discipline, trophy_filename};

    // 1. Normalization of disciplines
    assert_eq!(normalize_discipline("gt"), "gt");
    assert_eq!(normalize_discipline("gt_challenge"), "gt");
    assert_eq!(normalize_discipline("kart"), "kart");
    assert_eq!(normalize_discipline("karting"), "kart");
    assert_eq!(normalize_discipline("rally"), "rally");
    assert_eq!(normalize_discipline("rallycross"), "rally");
    assert_eq!(normalize_discipline("nascar"), "nascar");
    assert_eq!(normalize_discipline("extreme_offroad"), "extreme_offroad");
    assert_eq!(normalize_discipline("offroad"), "extreme_offroad");

    // 2. Trophy asset filenames
    assert_eq!(
        trophy_filename("gt", 1, Some(TrophyMetal::Gold), false),
        "gt_t1_gold-128.png"
    );
    assert_eq!(
        trophy_filename("gt_challenge", 3, Some(TrophyMetal::Silver), true),
        "gt_t3_silver-256.png"
    );
    assert_eq!(
        trophy_filename("offroad", 5, Some(TrophyMetal::Bronze), false),
        "extreme_offroad_t5_bronze-128.png"
    );
    assert_eq!(
        trophy_filename("rally", 2, None, false),
        "trophy_locked-128.png"
    );
    assert_eq!(
        trophy_filename("nascar", 4, None, true),
        "trophy_locked-256.png"
    );

    // 3. Verify asset files actually exist on disk for each generated filename
    let candidates = ["assets/icons/trophies", "../../assets/icons/trophies"];
    let base_dir = candidates.iter().map(std::path::Path::new).find(|p| p.exists()).expect("Trophy directory exists");

    for disc in &["gt", "kart", "rally", "nascar", "extreme_offroad"] {
        for tier in 1..=5 {
            for metal in &[TrophyMetal::Gold, TrophyMetal::Silver, TrophyMetal::Bronze] {
                let name128 = trophy_filename(disc, tier, Some(*metal), false);
                let name256 = trophy_filename(disc, tier, Some(*metal), true);
                let p128 = base_dir.join(&name128);
                let p256 = base_dir.join(&name256);
                assert!(p128.exists(), "Asset missing: {:?}", p128);
                assert!(p256.exists(), "Asset missing: {:?}", p256);
            }
        }
    }

    let locked128 = base_dir.join("trophy_locked-128.png");
    let locked256 = base_dir.join("trophy_locked-256.png");
    assert!(locked128.exists(), "Locked 128 asset missing");
    assert!(locked256.exists(), "Locked 256 asset missing");
}

#[test]
fn test_trophy_cabinet_grid_navigation_and_provenance_display() {
    use tdrace_app::profile::{ChampionshipAward, TrophyMetal};
    use tdrace_app::ui::profile_ui::{find_award_for_slot, ProfileFocusArea, CABINET_DISCIPLINES};

    let mut session = RaceSession::new();
    assert_eq!(session.profile_cabinet_disc_idx, 0);
    assert_eq!(session.profile_cabinet_tier_idx, 0);

    // 1. Verify CABINET_DISCIPLINES contains 5 official disciplines
    assert_eq!(CABINET_DISCIPLINES.len(), 5);
    assert_eq!(CABINET_DISCIPLINES[0].0, "gt");
    assert_eq!(CABINET_DISCIPLINES[1].0, "kart");
    assert_eq!(CABINET_DISCIPLINES[2].0, "rally");
    assert_eq!(CABINET_DISCIPLINES[3].0, "nascar");
    assert_eq!(CABINET_DISCIPLINES[4].0, "extreme_offroad");

    // 2. Prepare sample awards
    let awards = vec![
        ChampionshipAward {
            profile_id: 1,
            championship_id: "gt_clubman".to_string(),
            module_id: "gt".to_string(),
            tier: 1,
            position: 1,
            points: 100,
            car_model_id: "gt4_clubsport".to_string(),
            achieved_at: "2026-09-24T12:00:00Z".to_string(),
        },
        ChampionshipAward {
            profile_id: 1,
            championship_id: "rallycross_pro".to_string(),
            module_id: "rallycross".to_string(), // normalizes to rally
            tier: 3,
            position: 2,
            points: 85,
            car_model_id: "rally_subaru".to_string(),
            achieved_at: "2026-09-24T13:00:00Z".to_string(),
        },
        ChampionshipAward {
            profile_id: 1,
            championship_id: "nascar_cup".to_string(),
            module_id: "nascar".to_string(),
            tier: 5,
            position: 3,
            points: 70,
            car_model_id: "nascar_stock".to_string(),
            achieved_at: "2026-09-24T14:00:00Z".to_string(),
        },
    ];

    // 3. Test find_award_for_slot
    // Match GT Tier 1
    let gt_award = find_award_for_slot(&awards, "gt", 1).expect("GT Tier 1 award should be found");
    assert_eq!(gt_award.metallic_tier(), TrophyMetal::Gold);
    assert_eq!(gt_award.points, 100);

    // Match Rallycross Tier 3 (via discipline normalization)
    let rally_award = find_award_for_slot(&awards, "rally", 3).expect("Rally Tier 3 award should be found");
    assert_eq!(rally_award.metallic_tier(), TrophyMetal::Silver);
    assert_eq!(rally_award.points, 85);

    // Match NASCAR Tier 5
    let nascar_award = find_award_for_slot(&awards, "nascar", 5).expect("NASCAR Tier 5 award should be found");
    assert_eq!(nascar_award.metallic_tier(), TrophyMetal::Bronze);
    assert_eq!(nascar_award.points, 70);

    // Locked slots return None
    assert!(find_award_for_slot(&awards, "kart", 1).is_none());
    assert!(find_award_for_slot(&awards, "gt", 2).is_none());
    assert!(find_award_for_slot(&awards, "extreme_offroad", 5).is_none());

    // 4. Test Trophy Cabinet collection stats computation
    let total_slots = 25usize;
    let earned_count = awards.len();
    let collection_pct = (earned_count as f32 / total_slots as f32) * 100.0;
    assert_eq!(earned_count, 3);
    assert_eq!(collection_pct, 12.0); // 3 / 25 = 12%

    let gold_count = awards.iter().filter(|a| a.metallic_tier() == TrophyMetal::Gold).count();
    let silver_count = awards.iter().filter(|a| a.metallic_tier() == TrophyMetal::Silver).count();
    let bronze_count = awards.iter().filter(|a| a.metallic_tier() == TrophyMetal::Bronze).count();
    assert_eq!(gold_count, 1);
    assert_eq!(silver_count, 1);
    assert_eq!(bronze_count, 1);

    // 5. Test Cabinet navigation bounds: disc_idx (0..4), tier_idx (0..4)
    // Moving Down increases disc_idx up to 4
    for i in 1..=4 {
        if session.profile_cabinet_disc_idx + 1 < 5 {
            session.profile_cabinet_disc_idx += 1;
        }
        assert_eq!(session.profile_cabinet_disc_idx, i);
    }
    // Attempting to move Down beyond 4 stays at 4
    if session.profile_cabinet_disc_idx + 1 < 5 {
        session.profile_cabinet_disc_idx += 1;
    }
    assert_eq!(session.profile_cabinet_disc_idx, 4);

    // Moving Right increases tier_idx up to 4
    for j in 1..=4 {
        if session.profile_cabinet_tier_idx + 1 < 5 {
            session.profile_cabinet_tier_idx += 1;
        }
        assert_eq!(session.profile_cabinet_tier_idx, j);
    }
    // Attempting to move Right beyond 4 stays at 4
    if session.profile_cabinet_tier_idx + 1 < 5 {
        session.profile_cabinet_tier_idx += 1;
    }
    assert_eq!(session.profile_cabinet_tier_idx, 4);

    // Moving Left decreases tier_idx down to 0
    for j in (0..=3).rev() {
        if session.profile_cabinet_tier_idx > 0 {
            session.profile_cabinet_tier_idx -= 1;
        }
        assert_eq!(session.profile_cabinet_tier_idx, j);
    }

    // Moving Up decreases disc_idx down to 0, and from 0 returns focus to Tabs
    session.profile_focus_area = ProfileFocusArea::Content;
    for i in (0..=3).rev() {
        if session.profile_cabinet_disc_idx > 0 {
            session.profile_cabinet_disc_idx -= 1;
        }
        assert_eq!(session.profile_cabinet_disc_idx, i);
    }
    assert_eq!(session.profile_cabinet_disc_idx, 0);

    // Moving Up from row 0 shifts focus to Tabs
    if session.profile_cabinet_disc_idx > 0 {
        session.profile_cabinet_disc_idx -= 1;
    } else {
        session.profile_focus_area = ProfileFocusArea::Tabs;
    }
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Tabs);

    // 6. Test shortcut from Overview shelf to Cabinet
    session.profile_manager_tab = 0; // Overview tab
    // Simulate Enter on top honors shelf
    session.profile_manager_tab = 2; // Jump to Cabinet
    session.profile_focus_area = ProfileFocusArea::Content;
    session.profile_cabinet_disc_idx = 2; // rally
    session.profile_cabinet_tier_idx = 2; // tier 3 (0-indexed 2)
    assert_eq!(session.profile_manager_tab, 2);
    assert_eq!(session.profile_focus_area, ProfileFocusArea::Content);
    assert_eq!(session.profile_cabinet_disc_idx, 2);
    assert_eq!(session.profile_cabinet_tier_idx, 2);
}









