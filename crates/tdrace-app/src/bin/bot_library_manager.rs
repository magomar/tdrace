//! Bot Library Manager CLI tool (Spec 137).
//!
//! Provides CLI commands to inspect, list, query, and benchmark bot behaviors
//! stored in the Bot Behaviour Library (`data/bot_library/behaviours/`).

use std::path::Path;
use tdrace_app::ai::bot_harness::{run_behaviour_pack_race, run_behaviour_time_trial};
use tdrace_app::ai::library_storage::load_bot_library_from_dir;
use tdrace_app::catalog::CLASSIC_ARCADE_CARS;
use race_kit::ai::library::{BotBehaviourId, BotBehaviourRegistry};
use tdrace_core::catalog::official_track;

fn default_library_path() -> &'static Path {
    Path::new("data/bot_library/behaviours")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("list");

    let lib_dir = default_library_path();
    let registry = load_bot_library_from_dir(lib_dir).unwrap_or_else(|err| {
        eprintln!("[ERROR] Failed to load bot library from {:?}: {}", lib_dir, err);
        BotBehaviourRegistry::canonical_baseline_registry()
    });

    match cmd {
        "list" => {
            let width = 110;
            println!("╔{}╗", "═".repeat(width));
            println!("║ {:<width$} ║", "TDRace Bot Behaviour Library (Spec 137)", width = width - 2);
            println!("╠{}╣", "═".repeat(width));
            println!("║ {:<32} │ {:<7} │ {:<10} │ {:<9} │ {:<11} │ {:<5} │ {:<18} ║", "ID", "VERSION", "DATE", "STATUS", "STYLE", "TIER", "METHOD");
            println!("╟──────────────────────────────────┼─────────┼────────────┼───────────┼─────────────┼───────┼────────────────────╢");
            for b in registry.all() {
                let method_name = match &b.provenance {
                    race_kit::ai::library::CreationMethod::HandcraftedHeuristic { .. } => "Heuristic",
                    race_kit::ai::library::CreationMethod::EvolutionarySearch { .. } => "EvoRL Search",
                    race_kit::ai::library::CreationMethod::PhysicsExactQssMpc { .. } => "QSS+MPC (Spec 133)",
                    race_kit::ai::library::CreationMethod::HumanImitationCloning { .. } => "Human Imitation",
                    race_kit::ai::library::CreationMethod::ReinforcementLearningPolicy { .. } => "RL Policy",
                    race_kit::ai::library::CreationMethod::CompositeHybrid { .. } => "Composite Hybrid",
                };
                let date_str: String = b.date().chars().take(10).collect();
                println!(
                    "║ {:<32} │ {:<7} │ {:<10} │ {:<9?} │ {:<11?} │ {:<5} │ {:<18} ║",
                    b.id.0,
                    b.version,
                    date_str,
                    b.status,
                    b.persona.style,
                    b.persona.tier.tag(),
                    method_name
                );
            }
            println!("╚{}╝", "═".repeat(width));
            println!("Total behaviours: {} ({} unique controllers)\n", registry.len(), registry.unique_id_count());
        }
        "info" => {
            let id = match args.get(2) {
                Some(id) => BotBehaviourId::from(id.as_str()),
                None => {
                    eprintln!("Usage: bot_library_manager info <behaviour_id> [version]");
                    std::process::exit(1);
                }
            };
            let version_opt = args.get(3).map(|s| s.as_str());
            let b = match version_opt {
                Some(ver) => registry.get_version(&id, ver),
                None => registry.get(&id),
            };
            match b {
                Some(b) => {
                    let toml_str = toml::to_string_pretty(b).unwrap_or_default();
                    let versions = registry.versions_of(&id);
                    let vers_list: Vec<&str> = versions.iter().map(|v| v.version.as_str()).collect();
                    println!("--- Behaviour Details: {} (v{}, Date: {}) ---", b.id, b.version, b.date());
                    if vers_list.len() > 1 {
                        println!("All available versions: {:?}", vers_list);
                    }
                    println!("\n{}", toml_str);
                }
                None => {
                    eprintln!("Behaviour '{}' (version: {:?}) not found in library.", id, version_opt);
                    std::process::exit(1);
                }
            }
        }
        "benchmark" => {
            let id = match args.get(2) {
                Some(id) => BotBehaviourId::from(id.as_str()),
                None => {
                    eprintln!("Usage: bot_library_manager benchmark <behaviour_id> [version]");
                    std::process::exit(1);
                }
            };
            let version_opt = args.get(3).map(|s| s.as_str());
            let b = match version_opt {
                Some(ver) => match registry.get_version(&id, ver) {
                    Some(b) => b,
                    None => {
                        eprintln!("Behaviour '{}' version '{}' not found in library.", id, ver);
                        std::process::exit(1);
                    }
                },
                None => match registry.get(&id) {
                    Some(b) => b,
                    None => {
                        eprintln!("Behaviour '{}' not found in library.", id);
                        std::process::exit(1);
                    }
                },
            };

            println!("Running evaluation benchmark for '{}'...", b.id);
            let track = official_track("classic", "gt_coastal_grand_prix");
            let car_config = CLASSIC_ARCADE_CARS[0].to_car_config();

            // 1. Time Trial Benchmark
            println!("  [1/2] Executing 5-lap Time Trial on Coastal Grand Prix...");
            let tt = run_behaviour_time_trial(&track, "gt_coastal_grand_prix", car_config, "classic_gt", b, 5, 120.0);
            println!("        Fastest flying lap : {:.3} s", tt.fastest_lap_s);
            println!("        Mean flying lap    : {:.3} s", tt.mean_flying_lap_s);
            println!("        Lateral spread     : {:.3} m", tt.lateral_spread_m);
            println!("        Wall impacts       : {}", tt.wall_impact_count);

            // 2. Pack Race Benchmark
            println!("  [2/2] Executing 6-car Pack Race against canonical field...");
            let opponents: Vec<_> = registry
                .all()
                .filter(|o| o.id != b.id)
                .take(5)
                .cloned()
                .collect();
            let pack = run_behaviour_pack_race(
                &track,
                "gt_coastal_grand_prix",
                "gt_coastal_6car_pack",
                car_config,
                "classic_gt",
                b,
                &opponents,
                2,
                5,
                150.0,
            );
            println!("        Starting position  : P{}", pack.grid_starting_pos);
            println!("        Finish position    : P{}", pack.finish_position);
            println!("        Contacts per lap   : {:.2}", pack.contacts_per_lap);
            println!("        Clean passes       : {}", pack.clean_passes);
            println!("\nBenchmark complete for '{}'!\n", b.id);
        }
        _ => {
            eprintln!("Unknown command '{}'. Available: list, info <id>, benchmark <id>", cmd);
            std::process::exit(1);
        }
    }
}
