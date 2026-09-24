//! # Surface-Car Dynamics Computational Simulation Benchmark Runner
//!
//! Executes the full combinatorial experiment matrix:
//! - All 5 levels/tiers across the 5 specific modules (GT, NASCAR, Rally, Extreme Off-Road, Kart)
//! - Plus the 4 Classic Prototypical cars (Sports Car, Drift Car, Shifter Kart, Rally Car)
//! - Across all 12 surfaces (including Concrete) and 5 dynamic testing protocols.
//!
//! Generates both full detailed HTML and Markdown reports as well as JSON telemetry.

use std::fs;
use std::path::Path;
use std::time::Instant;

use chrono::Utc;
use tdrace_app::catalog::{get_models_for_module_and_tier, get_tier_name};
use tdrace_core::physics::sim::{
    generate_html_report, generate_markdown_report, ExperimentDataset, VehicleBenchmarkResult,
    DEFAULT_SIMULATION_DT,
};
use tdrace_core::physics::{CarConfig, SurfaceType};

struct VehicleTestTarget {
    id: String,
    name: String,
    category: String,
    module: String,
    tier: u8,
    drivetrain: String,
    mass_kg: f32,
    power_bhp: u16,
    top_speed_kmh: u16,
    config: CarConfig,
}

fn main() {
    println!("================================================================================");
    println!("🔬 TDRACE SURFACE-CAR DYNAMICS SYSTEMATIC COMPUTATIONAL BENCHMARK");
    println!("   Covering the 5 Levels of Specific Modules + Classic Prototypical Cars");
    println!("================================================================================");

    let start_wall = Instant::now();
    let timestamp = Utc::now().to_rfc3339();

    let mut dataset = ExperimentDataset::new(
        "Full 5-Tier Multi-Module & Classic Surface Dynamics Simulation",
        timestamp,
    );

    let mut targets: Vec<VehicleTestTarget> = Vec::new();

    // 1. Five specific modules, each covering Levels 1 through 5
    let specific_modules = [
        ("gt", "GT World Challenge"),
        ("nascar", "NASCAR"),
        ("rally", "Rally"),
        ("extreme_offroad", "Extreme Off-Road"),
        ("kart", "Kart"),
    ];

    for (mod_id, mod_title) in specific_modules {
        for tier in 1..=5 {
            let models = get_models_for_module_and_tier(mod_id, tier);
            if let Some(model) = models.first() {
                let tier_title = get_tier_name(mod_id, tier);
                targets.push(VehicleTestTarget {
                    id: model.id.to_string(),
                    name: model.name.to_string(),
                    category: tier_title.to_string(),
                    module: mod_title.to_string(),
                    tier,
                    drivetrain: model.drivetrain.to_string(),
                    mass_kg: model.weight_kg as f32,
                    power_bhp: model.bhp,
                    top_speed_kmh: model.top_speed_kmh,
                    config: model.to_car_config(),
                });
            }
        }
    }

    // 2. Classic Prototypical Cars
    targets.push(VehicleTestTarget {
        id: "classic_sports_car".to_string(),
        name: "GT Sports Coupe (Prototypical)".to_string(),
        category: "Classic Prototypical".to_string(),
        module: "Classic".to_string(),
        tier: 1,
        drivetrain: "RWD".to_string(),
        mass_kg: 1050.0,
        power_bhp: 320,
        top_speed_kmh: 208,
        config: CarConfig::sports_car(),
    });

    targets.push(VehicleTestTarget {
        id: "classic_drift_car".to_string(),
        name: "Tuned Drift Spec (Prototypical)".to_string(),
        category: "Classic Prototypical".to_string(),
        module: "Classic".to_string(),
        tier: 2,
        drivetrain: "RWD".to_string(),
        mass_kg: 980.0,
        power_bhp: 400,
        top_speed_kmh: 208,
        config: CarConfig::drift_car(),
    });

    targets.push(VehicleTestTarget {
        id: "classic_kart".to_string(),
        name: "125cc Shifter Kart (Prototypical)".to_string(),
        category: "Classic Prototypical".to_string(),
        module: "Classic".to_string(),
        tier: 3,
        drivetrain: "RWD".to_string(),
        mass_kg: 180.0,
        power_bhp: 48,
        top_speed_kmh: 115,
        config: CarConfig::kart(),
    });

    targets.push(VehicleTestTarget {
        id: "classic_rally_car".to_string(),
        name: "AWD Turbo Rally (Prototypical)".to_string(),
        category: "Classic Prototypical".to_string(),
        module: "Classic".to_string(),
        tier: 2,
        drivetrain: "AWD".to_string(),
        mass_kg: 1050.0,
        power_bhp: 380,
        top_speed_kmh: 208,
        config: CarConfig::rally_car(),
    });

    let total_runs = targets.len() * SurfaceType::ALL.len() * 5;
    println!(
        "Executing benchmark across {} vehicles and {} surfaces (Total: {} runs, dt = {:.4}s)...",
        targets.len(),
        SurfaceType::ALL.len(),
        total_runs,
        DEFAULT_SIMULATION_DT
    );
    println!("--------------------------------------------------------------------------------");

    for (idx, target) in targets.iter().enumerate() {
        let t0 = Instant::now();
        print!(
            "[{:02}/{:02}] {:<16} | T{:<1} | {:<38} ... ",
            idx + 1,
            targets.len(),
            target.module,
            target.tier,
            target.name
        );
        let res = VehicleBenchmarkResult::run(
            &target.id,
            &target.name,
            &target.category,
            &target.module,
            &target.config,
            DEFAULT_SIMULATION_DT,
        )
        .with_specs(
            target.tier,
            &target.drivetrain,
            target.mass_kg,
            target.power_bhp,
            target.top_speed_kmh,
        );
        println!("DONE ({:.1?})", t0.elapsed());
        dataset.vehicles.push(res);
    }

    let elapsed = start_wall.elapsed();
    println!("--------------------------------------------------------------------------------");
    println!(
        "✅ All {} vehicle benchmark batteries ({} total runs) finished in {:.2?} s!",
        dataset.vehicles.len(),
        total_runs,
        elapsed.as_secs_f64()
    );

    // Generate output reports: HTML, Markdown, and JSON
    let html_report = generate_html_report(&dataset);
    let markdown_report = generate_markdown_report(&dataset);
    let json_report = dataset.to_json().expect("Failed to serialize dataset to JSON");

    let out_dir = Path::new("docs/experiments");
    fs::create_dir_all(out_dir).expect("Failed to create docs/experiments directory");

    let html_path = out_dir.join("full_surface_simulation_report.html");
    let md_path = out_dir.join("full_surface_simulation_report.md");
    let json_path = out_dir.join("full_surface_simulation_report.json");

    fs::write(&html_path, &html_report).expect("Failed to write HTML report");
    fs::write(&md_path, &markdown_report).expect("Failed to write Markdown report");
    fs::write(&json_path, &json_report).expect("Failed to write JSON telemetry");

    println!("🌐 Saved Detailed HTML Report to: {}", html_path.display());
    println!("📄 Saved Markdown Summary to:    {}", md_path.display());
    println!("💾 Saved Raw JSON Telemetry to:  {}", json_path.display());

    // Validation verification
    println!("\n================================================================================");
    println!("🔍 VERIFICATION OF CORE PHYSICAL INVARIANTS");
    println!("================================================================================");

    let ice_idx = SurfaceType::ALL.iter().position(|&s| s == SurfaceType::SheetIce).expect("SheetIce missing");
    let sand_idx = SurfaceType::ALL.iter().position(|&s| s == SurfaceType::DeepSand).expect("DeepSand missing");
    let conc_idx = SurfaceType::ALL.iter().position(|&s| s == SurfaceType::Concrete).expect("Concrete missing");

    for v in &dataset.vehicles {
        let asp_stop = v.protocol_b[0].stopping_distance_m;
        let conc_stop = v.protocol_b[conc_idx].stopping_distance_m;
        let ice_stop = v.protocol_b[ice_idx].stopping_distance_m;

        // Ice stopping distance must drastically exceed Asphalt
        assert!(
            ice_stop > asp_stop * 2.0,
            "Vehicle {} stopping distance failed: Ice {:.1}m vs Asphalt {:.1}m",
            v.vehicle_name,
            ice_stop,
            asp_stop
        );

        // Concrete (mu = 0.95) stopping distance must be close to Asphalt (mu = 1.00)
        assert!(
            conc_stop >= asp_stop * 0.95 && conc_stop < ice_stop,
            "Vehicle {} stopping distance failed: Concrete {:.1}m vs Asphalt {:.1}m",
            v.vehicle_name,
            conc_stop,
            asp_stop
        );

        let asp_coast = v.protocol_e[0].coast_distance_m;
        let sand_coast = v.protocol_e[sand_idx].coast_distance_m;
        assert!(
            asp_coast > sand_coast,
            "Vehicle {} coast distance failed: Asphalt {:.1}m vs Sand {:.1}m",
            v.vehicle_name,
            asp_coast,
            sand_coast
        );

        // Protocol C Skidpad: Asphalt lateral grip must realistically exceed Ice (> 5.0x) and baseline >= 0.85g
        let asp_lat_g = v.protocol_c[0].peak_lateral_accel_g;
        let ice_lat_g = v.protocol_c[ice_idx].peak_lateral_accel_g;
        assert!(
            asp_lat_g > ice_lat_g * 5.0,
            "Vehicle {} skidpad grip failed: Asphalt {:.2}g vs Ice {:.2}g",
            v.vehicle_name,
            asp_lat_g,
            ice_lat_g
        );
        assert!(
            asp_lat_g >= 0.85,
            "Vehicle {} skidpad grip failed: Asphalt {:.2}g is below 0.85g threshold",
            v.vehicle_name,
            asp_lat_g
        );
    }

    println!("✅ All 30 vehicles passed physical invariants across all 12 surfaces (including Concrete)!");
    println!("================================================================================");
}
