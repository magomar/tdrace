//! Integration tests for Spec 137: Bot Behaviour Library, Provenance Registry,
//! and Multi-Scenario Driving Performance Evaluation Matrix.

use std::fs;
use std::path::Path;

use race_kit::ai::library::{
    BehaviourStatus, BotBehaviour, BotBehaviourId, BotBehaviourRegistry, BotQuery,
    CreationMethod, PerformanceSummaryMetrics,
};
use race_kit::ai::{DriverTier, DrivingStyle};
use tdrace_app::ai::bot_harness::{run_behaviour_pack_race, run_behaviour_time_trial};
use tdrace_app::ai::library_storage::{
    db, load_bot_library_from_dir, save_behaviour_to_file, HumanInteractionRecord,
};
use tdrace_app::catalog::CLASSIC_ARCADE_CARS;
use tdrace_core::catalog::official_track;

#[test]
fn test_scenario_register_and_retrieve_with_provenance() {
    // Scenario: Registering and retrieving a new behavior with complete creation provenance
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let behaviours_dir = manifest_dir.join("../../data/bot_library/behaviours");

    let registry = load_bot_library_from_dir(&behaviours_dir).expect("load official behaviours");
    assert!(registry.len() >= 10);

    let qss_id = BotBehaviourId::from("qss_mpc_legend_calculating_v1");
    let qss_b = registry.get(&qss_id).expect("qss behaviour must exist");

    assert_eq!(qss_b.persona.style, DrivingStyle::Calculating);
    assert_eq!(qss_b.persona.tier, DriverTier::Legend);
    assert_eq!(qss_b.status, BehaviourStatus::Candidate);

    match &qss_b.provenance {
        CreationMethod::PhysicsExactQssMpc {
            solver_version,
            horizon_seconds,
            rollout_candidates,
            ..
        } => {
            assert!(solver_version.contains("friction-ellipse"));
            assert!(*horizon_seconds > 0.5);
            assert_eq!(*rollout_candidates, 5);
        }
        other => panic!("expected PhysicsExactQssMpc provenance, got {:?}", other),
    }
}

#[test]
fn test_scenario_preserving_existing_behaviors_alongside_new_controllers() {
    // Scenario: Preserving existing behaviors when a new controller is introduced
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let behaviours_dir = manifest_dir.join("../../data/bot_library/behaviours");

    let registry = load_bot_library_from_dir(&behaviours_dir).expect("load library");

    let baseline_id = BotBehaviourId::from("canonical_balanced_t4");
    let qss_id = BotBehaviourId::from("qss_mpc_legend_calculating_v1");
    let evorl_id = BotBehaviourId::from("evorl_rx_aggressive_pro_v1");
    let imitation_id = BotBehaviourId::from("human_imitation_gt_smooth_pro_v1");

    assert!(registry.contains(&baseline_id));
    assert!(registry.contains(&qss_id));
    assert!(registry.contains(&evorl_id));
    assert!(registry.contains(&imitation_id));

    // Both controllers can be retrieved and instantiated simultaneously
    let baseline_b = registry.get(&baseline_id).unwrap();
    let qss_b = registry.get(&qss_id).unwrap();

    let driver1 = race_kit::ai::library::DriverControllerFactory::create_bot_driver(baseline_b);
    let driver2 = race_kit::ai::library::DriverControllerFactory::create_bot_driver(qss_b);

    assert_eq!(driver1.profile.aggression, baseline_b.to_bot_profile().aggression);
    assert_eq!(driver2.profile.aggression, qss_b.to_bot_profile().aggression);
}

#[test]
fn test_scenario_multi_scenario_simulation_matrix_execution() {
    // Scenario: Multi-scenario simulation matrix evaluates varied cars, tires, and grid sizes
    let track = official_track("classic", "gt_coastal_grand_prix");
    let car_cfg = CLASSIC_ARCADE_CARS[0].to_car_config();

    let subject = BotBehaviour::canonical_baseline(DrivingStyle::Balanced, DriverTier::Pro);
    let opponents = vec![
        BotBehaviour::canonical_baseline(DrivingStyle::Aggressive, DriverTier::Pro),
        BotBehaviour::canonical_baseline(DrivingStyle::Smooth, DriverTier::Pro),
        BotBehaviour::canonical_baseline(DrivingStyle::Tenacious, DriverTier::Pro),
    ];

    // 1. Time Trial Execution
    let tt_record = run_behaviour_time_trial(
        &track,
        "gt_coastal_grand_prix",
        car_cfg,
        "classic_gt",
        &subject,
        2,
        60.0,
    );
    assert_eq!(tt_record.behaviour_id, subject.id.0);
    assert_eq!(tt_record.track_slug, "gt_coastal_grand_prix");

    // 2. Pack Race Execution with 4 cars (subject + 3 opponents)
    let pack_record = run_behaviour_pack_race(
        &track,
        "gt_coastal_grand_prix",
        "gt_coastal_4car_sprint",
        car_cfg,
        "classic_gt",
        &subject,
        &opponents,
        1,
        2,
        60.0,
    );
    assert_eq!(pack_record.behaviour_id, subject.id.0);
    assert_eq!(pack_record.grid_size, 4);
    assert!(pack_record.finish_position >= 1 && pack_record.finish_position <= 4);
}

#[test]
fn test_scenario_query_by_non_pace_criteria_and_safety() {
    // Scenario: Querying behaviors by non-pace criteria (safety and racecraft etiquette)
    let mut reg = BotBehaviourRegistry::new();

    // Fast but aggressive/crash-prone bot
    let mut fast_wild_bot = BotBehaviour::canonical_baseline(DrivingStyle::Bold, DriverTier::LegendPlus);
    fast_wild_bot.id = BotBehaviourId::from("fast_wild_legend");
    fast_wild_bot.performance_summary = Some(PerformanceSummaryMetrics {
        best_flying_lap_s: Some(45.0),
        mean_flying_lap_s: Some(45.2),
        pace_delta_to_optimal_pct: Some(0.2),
        contacts_per_lap: Some(0.35), // high contact rate
        clean_pass_ratio: Some(0.65),
        human_believability_score: Some(0.60),
    });

    // Slightly slower, but extremely clean and courteous racer
    let mut courteous_bot = BotBehaviour::canonical_baseline(DrivingStyle::Smooth, DriverTier::Pro);
    courteous_bot.id = BotBehaviourId::from("courteous_smooth_pro");
    courteous_bot.performance_summary = Some(PerformanceSummaryMetrics {
        best_flying_lap_s: Some(46.8),
        mean_flying_lap_s: Some(47.1),
        pace_delta_to_optimal_pct: Some(2.2),
        contacts_per_lap: Some(0.01), // ultra-safe
        clean_pass_ratio: Some(0.98),
        human_believability_score: Some(0.95), // very believable
    });

    reg.register(fast_wild_bot);
    reg.register(courteous_bot);

    // Query for safe, human-friendly behavior
    let safe_query = BotQuery::builder()
        .max_contacts_per_lap(0.05)
        .min_human_believability(0.85)
        .build();

    let safe_results = reg.search(&safe_query);
    assert_eq!(safe_results.len(), 1);
    assert_eq!(safe_results[0].id.0, "courteous_smooth_pro");
}

#[test]
fn test_scenario_ingestion_and_logging_of_real_race_human_telemetry() {
    // Scenario: Ingestion and logging of real-race human player telemetry
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    db::init_benchmark_database(&conn).unwrap();

    let subject = BotBehaviour::canonical_baseline(DrivingStyle::Calculating, DriverTier::Pro);
    db::upsert_behaviour(&conn, &subject).unwrap();

    let human_log = HumanInteractionRecord {
        session_id: "session_user_race_101".to_string(),
        behaviour_id: subject.id.0.clone(),
        behaviour_version: subject.version.clone(),
        player_id: "mario".to_string(),
        track_slug: "gt_coastal_grand_prix".to_string(),
        car_model: "classic_gt".to_string(),
        player_contacts: 1,
        bot_fault_contacts: 0,
        player_fault_contacts: 1,
        courtesy_yield_count: 2,
        overtake_fairness_score: 0.95,
        player_restarted: false,
        recorded_at: "2026-10-22T20:30:00Z".to_string(),
    };

    db::record_human_interaction(&conn, &human_log).expect("record telemetry");

    let summary = db::query_performance_summary(&conn, &subject.id.0)
        .expect("query summary")
        .expect("summary must exist");

    assert_eq!(summary.human_believability_score, Some(0.95));
}

#[test]
fn test_scenario_graceful_fallback_on_corrupted_behavior_asset() {
    // Scenario: Graceful fallback on corrupted behavior asset
    let temp_dir = std::env::temp_dir().join("tdrace_fallback_test");
    fs::create_dir_all(&temp_dir).unwrap();

    let valid = BotBehaviour::canonical_baseline(DrivingStyle::Tenacious, DriverTier::Amateur);
    save_behaviour_to_file(&temp_dir.join("valid_tenacious.toml"), &valid).unwrap();

    // Malformed TOML file
    fs::write(temp_dir.join("corrupted.toml"), "INVALID TOML SYNTAX = [").unwrap();

    let reg = load_bot_library_from_dir(&temp_dir).expect("loading should gracefully succeed");
    assert_eq!(reg.len(), 1);
    assert!(reg.contains(&valid.id));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_scenario_behaviour_versioning_and_date_retention() {
    // Scenario: Versioning and date retention for behaviours/controllers
    let temp_dir = std::env::temp_dir().join("tdrace_versioning_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let bot_id = BotBehaviourId::from("qss_mpc_legend");

    let v1 = BotBehaviour::canonical_baseline(DrivingStyle::Calculating, DriverTier::Legend)
        .with_version("1.0.0")
        .with_date("2026-10-22T10:00:00Z");
    let mut v1 = v1;
    v1.id = bot_id.clone();
    save_behaviour_to_file(&temp_dir.join("qss_legend_v1.0.0.toml"), &v1).unwrap();

    let v2 = BotBehaviour::canonical_baseline(DrivingStyle::Calculating, DriverTier::Legend)
        .with_version("1.1.0")
        .with_date("2026-10-23T14:00:00Z")
        .with_updated_at("2026-10-23T15:30:00Z");
    let mut v2 = v2;
    v2.id = bot_id.clone();
    save_behaviour_to_file(&temp_dir.join("qss_legend_v1.1.0.toml"), &v2).unwrap();

    // 1. Loading preserves both versions concurrently without overwriting
    let reg = load_bot_library_from_dir(&temp_dir).expect("loading versioned library");
    assert_eq!(reg.len(), 2);
    assert_eq!(reg.unique_id_count(), 1);

    // Latest version is accessible via get()
    let active = reg.get(&bot_id).expect("should find active bot");
    assert_eq!(active.version, "1.1.0");
    assert_eq!(active.date(), "2026-10-23T15:30:00Z");

    // Historical version is accessible via get_version()
    let hist = reg.get_version(&bot_id, "1.0.0").expect("should find v1.0.0");
    assert_eq!(hist.version, "1.0.0");
    assert_eq!(hist.date(), "2026-10-22T10:00:00Z");

    // All versions enumerable via versions_of()
    let versions = reg.versions_of(&bot_id);
    assert_eq!(versions.len(), 2);

    // 2. Database stores both versions concurrently under composite primary key (id, version)
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    db::init_benchmark_database(&conn).unwrap();

    db::upsert_behaviour(&conn, &v1).unwrap();
    db::upsert_behaviour(&conn, &v2).unwrap();

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM behaviours WHERE id = ?1",
        [&bot_id.0],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 2, "Database must store both versions without replacing");

    // 3. Benchmarks are tracked per version
    let tt_v1 = tdrace_app::ai::library_storage::TimeTrialBenchmarkRecord {
        run_id: "tt_v1_run".to_string(),
        behaviour_id: bot_id.0.clone(),
        behaviour_version: "1.0.0".to_string(),
        track_slug: "classic_circuit_1".to_string(),
        car_model: "gt_classic".to_string(),
        tire_compound: "soft".to_string(),
        surface_condition: "dry_tarmac".to_string(),
        laps_completed: 5,
        fastest_lap_s: 48.0,
        mean_flying_lap_s: 48.5,
        optimal_lap_delta_pct: 1.0,
        lateral_spread_m: 0.1,
        brake_onset_spread_m: 0.5,
        spin_count: 0,
        off_track_count: 0,
        wall_impact_count: 0,
        executed_at: "2026-10-22T10:30:00Z".to_string(),
    };
    let tt_v2 = tdrace_app::ai::library_storage::TimeTrialBenchmarkRecord {
        run_id: "tt_v2_run".to_string(),
        behaviour_id: bot_id.0.clone(),
        behaviour_version: "1.1.0".to_string(),
        track_slug: "classic_circuit_1".to_string(),
        car_model: "gt_classic".to_string(),
        tire_compound: "soft".to_string(),
        surface_condition: "dry_tarmac".to_string(),
        laps_completed: 5,
        fastest_lap_s: 46.5, // Faster lap in v1.1.0!
        mean_flying_lap_s: 47.0,
        optimal_lap_delta_pct: 0.5,
        lateral_spread_m: 0.08,
        brake_onset_spread_m: 0.3,
        spin_count: 0,
        off_track_count: 0,
        wall_impact_count: 0,
        executed_at: "2026-10-23T14:30:00Z".to_string(),
    };

    db::record_time_trial(&conn, &tt_v1).unwrap();
    db::record_time_trial(&conn, &tt_v2).unwrap();

    let summary_v1 = db::query_performance_summary_for_version(&conn, &bot_id.0, Some("1.0.0"))
        .unwrap()
        .unwrap();
    assert_eq!(summary_v1.best_flying_lap_s, Some(48.0));

    let summary_v2 = db::query_performance_summary_for_version(&conn, &bot_id.0, Some("1.1.0"))
        .unwrap()
        .unwrap();
    assert_eq!(summary_v2.best_flying_lap_s, Some(46.5));

    let _ = fs::remove_dir_all(&temp_dir);
}
