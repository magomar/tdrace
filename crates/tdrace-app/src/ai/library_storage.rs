//! File-based TOML asset loading and SQLite benchmark storage for bot behaviors (Spec 137).

use std::fs;
use std::path::Path;

use race_kit::ai::library::{
    BotBehaviour, BotBehaviourRegistry, PerformanceSummaryMetrics,
};

/// Record of a completed time trial simulation benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeTrialBenchmarkRecord {
    pub run_id: String,
    pub behaviour_id: String,
    pub behaviour_version: String,
    pub track_slug: String,
    pub car_model: String,
    pub tire_compound: String,
    pub surface_condition: String,
    pub laps_completed: u32,
    pub fastest_lap_s: f32,
    pub mean_flying_lap_s: f32,
    pub optimal_lap_delta_pct: f32,
    pub lateral_spread_m: f32,
    pub brake_onset_spread_m: f32,
    pub spin_count: u32,
    pub off_track_count: u32,
    pub wall_impact_count: u32,
    pub executed_at: String,
}

/// Record of a completed multi-participant pack race simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct PackRaceBenchmarkRecord {
    pub run_id: String,
    pub behaviour_id: String,
    pub behaviour_version: String,
    pub scenario_name: String,
    pub track_slug: String,
    pub car_model: String,
    pub grid_size: u32,
    pub grid_starting_pos: u32,
    pub finish_position: u32,
    pub opponent_diversity_profile: String,
    pub contacts_per_lap: f32,
    pub mean_contact_impulse: f32,
    pub clean_passes: u32,
    pub divebomb_passes: u32,
    pub traffic_stall_time_s: f32,
    pub dnf_occurred: bool,
    pub executed_at: String,
}

/// Record of live telemetry captured from a real human player race.
#[derive(Debug, Clone, PartialEq)]
pub struct HumanInteractionRecord {
    pub session_id: String,
    pub behaviour_id: String,
    pub behaviour_version: String,
    pub player_id: String,
    pub track_slug: String,
    pub car_model: String,
    pub player_contacts: u32,
    pub bot_fault_contacts: u32,
    pub player_fault_contacts: u32,
    pub courtesy_yield_count: u32,
    pub overtake_fairness_score: f32,
    pub player_restarted: bool,
    pub recorded_at: String,
}

/// Scans `dir` for `.toml` files, parsing each into a [`BotBehaviour`] and registering it.
/// If any file is malformed, a warning is printed to stderr and the file is skipped gracefully.
pub fn load_bot_library_from_dir(dir: &Path) -> Result<BotBehaviourRegistry, Box<dyn std::error::Error>> {
    let mut registry = BotBehaviourRegistry::new();

    if !dir.exists() {
        return Ok(registry);
    }

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<BotBehaviour>(&content) {
                    Ok(behaviour) => {
                        registry.register(behaviour);
                    }
                    Err(err) => {
                        eprintln!(
                            "[WARN] Skipping invalid bot behaviour TOML file {:?}: {}",
                            path, err
                        );
                    }
                },
                Err(err) => {
                    eprintln!("[WARN] Failed to read bot behaviour file {:?}: {}", path, err);
                }
            }
        }
    }

    Ok(registry)
}

/// Serializes `behaviour` to pretty TOML and writes it to `path`.
pub fn save_behaviour_to_file(path: &Path, behaviour: &BotBehaviour) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let toml_str = toml::to_string_pretty(behaviour)?;
    fs::write(path, toml_str)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub mod db {
    use super::*;
    use rusqlite::{params, Connection, Result};

    /// Initializes SQLite tables for storing benchmark matrix runs and telemetry logs.
    pub fn init_benchmark_database(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS behaviours (
                id TEXT NOT NULL,
                version TEXT NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT,
                author TEXT NOT NULL,
                creation_method_type TEXT NOT NULL,
                driving_style TEXT NOT NULL,
                driver_tier TEXT NOT NULL,
                modality_specialization TEXT NOT NULL,
                controller_kind TEXT NOT NULL,
                raw_toml TEXT NOT NULL,
                PRIMARY KEY (id, version)
            );

            CREATE TABLE IF NOT EXISTS time_trial_benchmarks (
                run_id TEXT PRIMARY KEY,
                behaviour_id TEXT NOT NULL,
                behaviour_version TEXT NOT NULL DEFAULT '1.0.0',
                track_slug TEXT NOT NULL,
                car_model TEXT NOT NULL,
                tire_compound TEXT NOT NULL,
                surface_condition TEXT NOT NULL,
                laps_completed INTEGER NOT NULL,
                fastest_lap_s REAL NOT NULL,
                mean_flying_lap_s REAL NOT NULL,
                optimal_lap_delta_pct REAL NOT NULL,
                lateral_spread_m REAL NOT NULL,
                brake_onset_spread_m REAL NOT NULL,
                spin_count INTEGER NOT NULL,
                off_track_count INTEGER NOT NULL,
                wall_impact_count INTEGER NOT NULL,
                executed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pack_race_benchmarks (
                run_id TEXT PRIMARY KEY,
                behaviour_id TEXT NOT NULL,
                behaviour_version TEXT NOT NULL DEFAULT '1.0.0',
                scenario_name TEXT NOT NULL,
                track_slug TEXT NOT NULL,
                car_model TEXT NOT NULL,
                grid_size INTEGER NOT NULL,
                grid_starting_pos INTEGER NOT NULL,
                finish_position INTEGER NOT NULL,
                opponent_diversity_profile TEXT NOT NULL,
                contacts_per_lap REAL NOT NULL,
                mean_contact_impulse REAL NOT NULL,
                clean_passes INTEGER NOT NULL,
                divebomb_passes INTEGER NOT NULL,
                traffic_stall_time_s REAL NOT NULL,
                dnf_occurred INTEGER NOT NULL,
                executed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS human_interaction_telemetry (
                session_id TEXT PRIMARY KEY,
                behaviour_id TEXT NOT NULL,
                behaviour_version TEXT NOT NULL DEFAULT '1.0.0',
                player_id TEXT NOT NULL,
                track_slug TEXT NOT NULL,
                car_model TEXT NOT NULL,
                player_contacts INTEGER NOT NULL,
                bot_fault_contacts INTEGER NOT NULL,
                player_fault_contacts INTEGER NOT NULL,
                courtesy_yield_count INTEGER NOT NULL,
                overtake_fairness_score REAL NOT NULL,
                player_restarted INTEGER NOT NULL,
                recorded_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tt_behaviour ON time_trial_benchmarks(behaviour_id, behaviour_version, track_slug);
            CREATE INDEX IF NOT EXISTS idx_pack_behaviour ON pack_race_benchmarks(behaviour_id, behaviour_version, scenario_name);
            CREATE INDEX IF NOT EXISTS idx_human_behaviour ON human_interaction_telemetry(behaviour_id, behaviour_version);
            "#,
        )?;
        Ok(())
    }

    /// Stores a behavior record in the database.
    pub fn upsert_behaviour(conn: &Connection, b: &BotBehaviour) -> Result<()> {
        let raw_toml = toml::to_string_pretty(b).unwrap_or_default();
        let creation_type = match &b.provenance {
            race_kit::ai::library::CreationMethod::HandcraftedHeuristic { .. } => "handcrafted_heuristic",
            race_kit::ai::library::CreationMethod::EvolutionarySearch { .. } => "evolutionary_search",
            race_kit::ai::library::CreationMethod::PhysicsExactQssMpc { .. } => "physics_exact_qss_mpc",
            race_kit::ai::library::CreationMethod::HumanImitationCloning { .. } => "human_imitation_cloning",
            race_kit::ai::library::CreationMethod::ReinforcementLearningPolicy { .. } => "reinforcement_learning_policy",
            race_kit::ai::library::CreationMethod::CompositeHybrid { .. } => "composite_hybrid",
        };
        let controller_kind = match &b.controller {
            race_kit::ai::library::ControllerPayload::ClassicalLookahead { .. } => "classical_lookahead",
            race_kit::ai::library::ControllerPayload::OptimalLineFollower { .. } => "optimal_line_follower",
            race_kit::ai::library::ControllerPayload::QssMpcRollout { .. } => "qss_mpc_rollout",
        };

        conn.execute(
            r#"
            INSERT INTO behaviours (
                id, version, name, status, created_at, updated_at, author,
                creation_method_type, driving_style, driver_tier,
                modality_specialization, controller_kind, raw_toml
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id, version) DO UPDATE SET
                name = excluded.name,
                status = excluded.status,
                updated_at = excluded.updated_at,
                raw_toml = excluded.raw_toml
            "#,
            params![
                b.id.0,
                b.version,
                b.name,
                format!("{:?}", b.status).to_lowercase(),
                b.created_at,
                b.updated_at,
                b.author,
                creation_type,
                b.persona.style.as_str(),
                b.persona.tier.tag(),
                b.persona.modality_specialization,
                controller_kind,
                raw_toml
            ],
        )?;
        Ok(())
    }

    /// Records a time trial benchmark result.
    pub fn record_time_trial(conn: &Connection, r: &TimeTrialBenchmarkRecord) -> Result<()> {
        conn.execute(
            r#"
            INSERT INTO time_trial_benchmarks (
                run_id, behaviour_id, behaviour_version, track_slug, car_model, tire_compound,
                surface_condition, laps_completed, fastest_lap_s, mean_flying_lap_s,
                optimal_lap_delta_pct, lateral_spread_m, brake_onset_spread_m,
                spin_count, off_track_count, wall_impact_count, executed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
            params![
                r.run_id,
                r.behaviour_id,
                r.behaviour_version,
                r.track_slug,
                r.car_model,
                r.tire_compound,
                r.surface_condition,
                r.laps_completed,
                r.fastest_lap_s,
                r.mean_flying_lap_s,
                r.optimal_lap_delta_pct,
                r.lateral_spread_m,
                r.brake_onset_spread_m,
                r.spin_count,
                r.off_track_count,
                r.wall_impact_count,
                r.executed_at
            ],
        )?;
        Ok(())
    }

    /// Records a multi-participant pack race benchmark result.
    pub fn record_pack_race(conn: &Connection, r: &PackRaceBenchmarkRecord) -> Result<()> {
        conn.execute(
            r#"
            INSERT INTO pack_race_benchmarks (
                run_id, behaviour_id, behaviour_version, scenario_name, track_slug, car_model,
                grid_size, grid_starting_pos, finish_position, opponent_diversity_profile,
                contacts_per_lap, mean_contact_impulse, clean_passes, divebomb_passes,
                traffic_stall_time_s, dnf_occurred, executed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
            params![
                r.run_id,
                r.behaviour_id,
                r.behaviour_version,
                r.scenario_name,
                r.track_slug,
                r.car_model,
                r.grid_size,
                r.grid_starting_pos,
                r.finish_position,
                r.opponent_diversity_profile,
                r.contacts_per_lap,
                r.mean_contact_impulse,
                r.clean_passes,
                r.divebomb_passes,
                r.traffic_stall_time_s,
                r.dnf_occurred as i32,
                r.executed_at
            ],
        )?;
        Ok(())
    }

    /// Records telemetry from a real human race.
    pub fn record_human_interaction(conn: &Connection, r: &HumanInteractionRecord) -> Result<()> {
        conn.execute(
            r#"
            INSERT INTO human_interaction_telemetry (
                session_id, behaviour_id, behaviour_version, player_id, track_slug, car_model,
                player_contacts, bot_fault_contacts, player_fault_contacts,
                courtesy_yield_count, overtake_fairness_score, player_restarted,
                recorded_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                r.session_id,
                r.behaviour_id,
                r.behaviour_version,
                r.player_id,
                r.track_slug,
                r.car_model,
                r.player_contacts,
                r.bot_fault_contacts,
                r.player_fault_contacts,
                r.courtesy_yield_count,
                r.overtake_fairness_score,
                r.player_restarted as i32,
                r.recorded_at
            ],
        )?;
        Ok(())
    }

    /// Queries aggregate performance metrics for `behaviour_id` across all benchmarks.
    pub fn query_performance_summary(
        conn: &Connection,
        behaviour_id: &str,
    ) -> Result<Option<PerformanceSummaryMetrics>> {
        query_performance_summary_for_version(conn, behaviour_id, None)
    }

    /// Queries aggregate performance metrics for `behaviour_id` and an optional specific `version`.
    pub fn query_performance_summary_for_version(
        conn: &Connection,
        behaviour_id: &str,
        version: Option<&str>,
    ) -> Result<Option<PerformanceSummaryMetrics>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                (SELECT MIN(fastest_lap_s) FROM time_trial_benchmarks WHERE behaviour_id = ?1 AND (?2 IS NULL OR behaviour_version = ?2)),
                (SELECT AVG(mean_flying_lap_s) FROM time_trial_benchmarks WHERE behaviour_id = ?1 AND (?2 IS NULL OR behaviour_version = ?2)),
                (SELECT AVG(optimal_lap_delta_pct) FROM time_trial_benchmarks WHERE behaviour_id = ?1 AND (?2 IS NULL OR behaviour_version = ?2)),
                (SELECT AVG(contacts_per_lap) FROM pack_race_benchmarks WHERE behaviour_id = ?1 AND (?2 IS NULL OR behaviour_version = ?2)),
                (SELECT CAST(SUM(clean_passes) AS REAL) / MAX(1, SUM(clean_passes + divebomb_passes)) FROM pack_race_benchmarks WHERE behaviour_id = ?1 AND (?2 IS NULL OR behaviour_version = ?2)),
                (SELECT AVG(overtake_fairness_score) FROM human_interaction_telemetry WHERE behaviour_id = ?1 AND (?2 IS NULL OR behaviour_version = ?2))
            "#,
        )?;

        let row = stmt.query_row(params![behaviour_id, version], |row| {
            Ok(PerformanceSummaryMetrics {
                best_flying_lap_s: row.get(0)?,
                mean_flying_lap_s: row.get(1)?,
                pace_delta_to_optimal_pct: row.get(2)?,
                contacts_per_lap: row.get(3)?,
                clean_pass_ratio: row.get(4)?,
                human_believability_score: row.get(5)?,
            })
        })?;

        // If all fields are None, return None
        if row.best_flying_lap_s.is_none()
            && row.mean_flying_lap_s.is_none()
            && row.contacts_per_lap.is_none()
            && row.human_believability_score.is_none()
        {
            Ok(None)
        } else {
            Ok(Some(row))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use race_kit::ai::{DriverTier, DrivingStyle};

    #[test]
    fn test_save_and_load_behaviour_toml() {
        let temp_dir = std::env::temp_dir().join("tdrace_bot_library_test");
        let file_path = temp_dir.join("test_bot.toml");

        let b = BotBehaviour::canonical_baseline(DrivingStyle::Bold, DriverTier::Legend);
        save_behaviour_to_file(&file_path, &b).expect("save behaviour");

        let loaded = load_bot_library_from_dir(&temp_dir).expect("load library");
        assert!(loaded.contains(&b.id));

        let retrieved = loaded.get(&b.id).unwrap();
        assert_eq!(retrieved.name, b.name);
        assert_eq!(retrieved.persona.style, DrivingStyle::Bold);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_graceful_fallback_on_corrupted_file() {
        let temp_dir = std::env::temp_dir().join("tdrace_corrupt_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let valid_b = BotBehaviour::canonical_baseline(DrivingStyle::Smooth, DriverTier::Pro);
        save_behaviour_to_file(&temp_dir.join("valid.toml"), &valid_b).unwrap();

        // Write a garbage corrupted TOML file
        fs::write(temp_dir.join("corrupted.toml"), "INVALID TOML {{{ [[[[").unwrap();

        let registry = load_bot_library_from_dir(&temp_dir).expect("should not error out");
        assert_eq!(registry.len(), 1);
        assert!(registry.contains(&valid_b.id));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_sqlite_benchmark_db_lifecycle() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::init_benchmark_database(&conn).unwrap();

        let b = BotBehaviour::canonical_baseline(DrivingStyle::Calculating, DriverTier::Legend);
        db::upsert_behaviour(&conn, &b).unwrap();

        let tt = TimeTrialBenchmarkRecord {
            run_id: "tt_001".to_string(),
            behaviour_id: b.id.0.clone(),
            behaviour_version: b.version.clone(),
            track_slug: "classic_circuit_1".to_string(),
            car_model: "gt_classic".to_string(),
            tire_compound: "soft".to_string(),
            surface_condition: "dry_tarmac".to_string(),
            laps_completed: 10,
            fastest_lap_s: 48.5,
            mean_flying_lap_s: 48.9,
            optimal_lap_delta_pct: 1.2,
            lateral_spread_m: 0.15,
            brake_onset_spread_m: 0.8,
            spin_count: 0,
            off_track_count: 0,
            wall_impact_count: 0,
            executed_at: "2026-10-22T20:00:00Z".to_string(),
        };
        db::record_time_trial(&conn, &tt).unwrap();

        let pack = PackRaceBenchmarkRecord {
            run_id: "pack_001".to_string(),
            behaviour_id: b.id.0.clone(),
            behaviour_version: b.version.clone(),
            scenario_name: "grid_8_autocross".to_string(),
            track_slug: "autocross_track_1".to_string(),
            car_model: "buggy_1600".to_string(),
            grid_size: 8,
            grid_starting_pos: 4,
            finish_position: 2,
            opponent_diversity_profile: "uniform_styles".to_string(),
            contacts_per_lap: 0.05,
            mean_contact_impulse: 120.0,
            clean_passes: 3,
            divebomb_passes: 0,
            traffic_stall_time_s: 0.0,
            dnf_occurred: false,
            executed_at: "2026-10-22T20:05:00Z".to_string(),
        };
        db::record_pack_race(&conn, &pack).unwrap();

        let summary = db::query_performance_summary(&conn, &b.id.0).unwrap();
        assert!(summary.is_some());
        let s = summary.unwrap();
        assert_eq!(s.best_flying_lap_s, Some(48.5));
        assert_eq!(s.contacts_per_lap, Some(0.05));
        assert_eq!(s.clean_pass_ratio, Some(1.0));
    }

    #[test]
    fn test_load_official_behaviours_catalog() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let behaviours_dir = manifest_dir.join("../../data/bot_library/behaviours");
        if behaviours_dir.exists() {
            let registry = load_bot_library_from_dir(&behaviours_dir).expect("load official library");
            assert!(registry.len() >= 10, "expected at least 10 seeded behaviours, found {}", registry.len());
            assert!(registry.get(&race_kit::ai::library::BotBehaviourId::from("canonical_balanced_t4")).is_some());
            assert!(registry.get(&race_kit::ai::library::BotBehaviourId::from("qss_mpc_legend_calculating_v1")).is_some());
            assert!(registry.get(&race_kit::ai::library::BotBehaviourId::from("evorl_rx_aggressive_pro_v1")).is_some());
            assert!(registry.get(&race_kit::ai::library::BotBehaviourId::from("human_imitation_gt_smooth_pro_v1")).is_some());
        }
    }
}
