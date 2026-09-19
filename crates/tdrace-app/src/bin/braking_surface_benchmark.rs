//! # Multi-Surface Car Braking Dynamics & Stability Simulation Benchmark Runner
//!
//! Executes the multi-scenario braking simulation battery across representative motorsport
//! vehicle archetypes and all 12 surfaces:
//! 1. High-Speed Straight-Line Panic Braking (Nominal vs Early Yaw Perturbation)
//! 2. Dynamic EBD Rear Axle Lockup Assessment
//! 3. Split-Mu Asymmetric Surface Braking (ISO 14512)
//! 4. Cornering Trail-Braking (CBC Inside-Wheel Modulation)
//! 5. Cadence / Brake Pumping Recovery Latency
//!
//! Generates Markdown and JSON reports saved to `docs/experiments/`.

use std::fs;
use std::path::Path;
use std::time::Instant;

use chrono::Utc;
use tdrace_app::module::f1::GtWorldChallengeModule;
use tdrace_app::module::{
    ExtremeOffRoadModule, KartGameModule, NascarGameModule, RallyGameModule,
};
use tdrace_core::physics::sim::{
    generate_braking_simulation_markdown_report, run_braking_surface_battery,
    BrakingStabilityRating, BrakingSurfaceExperimentResult, CorneringBrakingBehavior,
    SplitMuStatus, DEFAULT_SIMULATION_DT,
};
use tdrace_core::physics::{CarConfig, SurfaceType};

struct VehicleTarget {
    id: String,
    name: String,
    category: String,
    v0_kmh: f32,
    config: CarConfig,
}

fn main() {
    println!("================================================================================");
    println!("🔬 TDRACE MULTI-SURFACE CAR BRAKING DYNAMICS & STABILITY BENCHMARK");
    println!("   Systematic Assessment of 7 Representative Vehicles across 12 Surfaces");
    println!("================================================================================");

    let start_wall = Instant::now();
    let timestamp = Utc::now().to_rfc3339();

    let targets = vec![
        VehicleTarget {
            id: "gt3_evo".to_string(),
            name: "AMG GT3 Evo".to_string(),
            category: "GT / GT3 Endurance".to_string(),
            v0_kmh: 150.0,
            config: GtWorldChallengeModule::car_gt3_evo(),
        },
        VehicleTarget {
            id: "f1_hybrid_26".to_string(),
            name: "1050 BHP Hybrid F1 Turbo".to_string(),
            category: "GT / Open-Wheel F1".to_string(),
            v0_kmh: 180.0,
            config: GtWorldChallengeModule::car_f1_hybrid(),
        },
        VehicleTarget {
            id: "nascar_cup_v8".to_string(),
            name: "NASCAR Cup V8 Stock Car".to_string(),
            category: "NASCAR / TA1 Cup".to_string(),
            v0_kmh: 140.0,
            config: NascarGameModule::car_stock_car(),
        },
        VehicleTarget {
            id: "wrc_turbo_rally".to_string(),
            name: "WRC AWD Turbo Rally".to_string(),
            category: "Rally / Group Rallycross".to_string(),
            v0_kmh: 140.0,
            config: RallyGameModule::car_wrc_rally(),
        },
        VehicleTarget {
            id: "sand_rail_buggy".to_string(),
            name: "300 BHP Sand Rail Buggy".to_string(),
            category: "Extreme Off-Road / Sand Rail".to_string(),
            v0_kmh: 120.0,
            config: ExtremeOffRoadModule::car_sand_rail(),
        },
        VehicleTarget {
            id: "shifter_kart_125".to_string(),
            name: "125cc Shifter Kart".to_string(),
            category: "Kart / Shifter Kart".to_string(),
            v0_kmh: 100.0,
            config: KartGameModule::car_shifter_kart(),
        },
        VehicleTarget {
            id: "classic_sports_car".to_string(),
            name: "GT Sports Coupe (Prototypical)".to_string(),
            category: "Classic Prototypical".to_string(),
            v0_kmh: 140.0,
            config: CarConfig::sports_car(),
        },
    ];

    println!(
        "Executing braking battery across {} vehicles and {} surfaces (dt = {:.4}s)...",
        targets.len(),
        SurfaceType::ALL.len(),
        DEFAULT_SIMULATION_DT
    );
    println!("--------------------------------------------------------------------------------");

    let mut results: Vec<BrakingSurfaceExperimentResult> = Vec::with_capacity(targets.len());

    for (idx, target) in targets.iter().enumerate() {
        let t0 = Instant::now();
        print!(
            "[{:02}/{:02}] {:<32} | {:<28} ... ",
            idx + 1,
            targets.len(),
            target.name,
            target.category
        );

        let res = run_braking_surface_battery(
            &target.id,
            &target.name,
            &target.category,
            &target.config,
            target.v0_kmh,
            DEFAULT_SIMULATION_DT,
        );
        println!("DONE ({:.1?})", t0.elapsed());
        results.push(res);
    }

    let elapsed = start_wall.elapsed();
    println!("--------------------------------------------------------------------------------");
    println!(
        "✅ All {} braking benchmark batteries completed in {:.2?} s!",
        results.len(),
        elapsed.as_secs_f64()
    );

    // Generate output reports: Markdown and JSON
    let experiment_name = "Multi-Surface Car Braking Dynamics & Stability Benchmark";
    let md_report = generate_braking_simulation_markdown_report(experiment_name, &timestamp, &results);
    let json_report = serde_json::to_string_pretty(&results)
        .expect("Failed to serialize braking experiment results to JSON");

    let out_dir = Path::new("docs/experiments");
    fs::create_dir_all(out_dir).expect("Failed to create docs/experiments directory");

    let md_path = out_dir.join("surface_braking_simulation_report.md");
    let json_path = out_dir.join("surface_braking_simulation_report.json");

    fs::write(&md_path, &md_report).expect("Failed to write Markdown report");
    fs::write(&json_path, &json_report).expect("Failed to write JSON telemetry");

    println!("📄 Saved Markdown Summary to:   {}", md_path.display());
    println!("💾 Saved Raw JSON Telemetry to: {}", json_path.display());

    // Physical Invariants & Realism Verification
    println!("\n================================================================================");
    println!("🔍 VERIFICATION OF CORE BRAKING INVARIANTS & STABILITY BOUNDS");
    println!("================================================================================");

    let asp_idx = SurfaceType::ALL.iter().position(|&s| s == SurfaceType::Asphalt).unwrap();
    let conc_idx = SurfaceType::ALL.iter().position(|&s| s == SurfaceType::Concrete).unwrap();
    let ice_idx = SurfaceType::ALL.iter().position(|&s| s == SurfaceType::Ice).unwrap();

    for r in &results {
        let asp_stop = r.straight_line_nominal[asp_idx].stopping_distance_m;
        let conc_stop = r.straight_line_nominal[conc_idx].stopping_distance_m;
        let ice_stop = r.straight_line_nominal[ice_idx].stopping_distance_m;

        // 1. Ice stopping distance must drastically exceed Asphalt
        assert!(
            ice_stop > asp_stop * 2.5,
            "{}: Ice stopping distance ({:.1}m) must exceed 2.5x Asphalt ({:.1}m)",
            r.vehicle_name,
            ice_stop,
            asp_stop
        );

        // 2. Concrete stopping distance must be close to Asphalt
        assert!(
            conc_stop >= asp_stop * 0.95 && conc_stop < ice_stop * 0.5,
            "{}: Concrete stopping distance ({:.1}m) failed relative to Asphalt ({:.1}m)",
            r.vehicle_name,
            conc_stop,
            asp_stop
        );

        // 3. EBD rear lockup constraint: Rear lockup duration should not exceed front lockup
        for s in &r.straight_line_nominal {
            assert!(
                s.rear_lockup_duration_s <= s.front_lockup_duration_s + 0.10,
                "{}: Rear lockup ({:.2}s) exceeded front lockup ({:.2}s) on {}",
                r.vehicle_name,
                s.rear_lockup_duration_s,
                s.front_lockup_duration_s,
                s.surface.name()
            );
        }

        // 4. Panic stop with yaw disturbance: On Asphalt and Concrete, stability must avoid spinout
        let asp_perturbed = &r.straight_line_perturbed[asp_idx];
        let conc_perturbed = &r.straight_line_perturbed[conc_idx];
        assert_ne!(
            asp_perturbed.stability_rating,
            BrakingStabilityRating::Spinout,
            "{}: Asphalt perturbed braking spun out (sideslip = {:.1}°)",
            r.vehicle_name,
            asp_perturbed.max_sideslip_deg
        );
        assert!(
            asp_perturbed.max_sideslip_deg < 25.0,
            "{}: Asphalt perturbed sideslip ({:.1}°) exceeded 25°",
            r.vehicle_name,
            asp_perturbed.max_sideslip_deg
        );
        assert_ne!(
            conc_perturbed.stability_rating,
            BrakingStabilityRating::Spinout,
            "{}: Concrete perturbed braking spun out (sideslip = {:.1}°)",
            r.vehicle_name,
            conc_perturbed.max_sideslip_deg
        );
        assert!(
            conc_perturbed.max_sideslip_deg < 25.0,
            "{}: Concrete perturbed sideslip ({:.1}°) exceeded 25°",
            r.vehicle_name,
            conc_perturbed.max_sideslip_deg
        );

        // 5. Cornering trail-braking: on Asphalt, car must not snap oversteer
        let asp_cornering = &r.cornering_trail_braking[asp_idx];
        assert_ne!(
            asp_cornering.behavior,
            CorneringBrakingBehavior::SnapOversteerSpin,
            "{}: Snapped into spin during trail-braking on Asphalt",
            r.vehicle_name
        );

        // 6. Split-mu (Asphalt vs Gravel): must not spin out
        if let Some(split_gravel) = r.split_mu.iter().find(|sm| sm.low_mu_surface == SurfaceType::Gravel) {
            assert_ne!(
                split_gravel.status,
                SplitMuStatus::SpunOut,
                "{}: Spun out on Asphalt vs Gravel split-mu test",
                r.vehicle_name
            );
        }
    }

    println!("✅ All {} vehicles passed physical braking invariants across all surfaces!", results.len());
    println!("================================================================================");
}
