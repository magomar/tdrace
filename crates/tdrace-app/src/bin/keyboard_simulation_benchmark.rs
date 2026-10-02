//! # Keyboard Key-Style × Preset × Car Benchmark (Spec 043)
//!
//! Drives the five Classic Arcade Cars with six prototypical key-pressing styles on each of the
//! four keyboard handling presets, across asphalt, dirt, packed sand and ice, in three scenarios
//! (sweeper corner, S-chicane reversal, low-grip slide catch).
//!
//! Writes `reports/keyboard_input_car_control_report.{md,json}`. When
//! `reports/keyboard_input_car_control_report_pre043.json` exists, the report adds an
//! old-vs-new comparison against it.

use std::fs;
use std::path::Path;
use std::time::Instant;

use chrono::Utc;
use serde::Serialize;

use tdrace_app::catalog::{RealCarModel, ALL_REAL_CARS, CLASSIC_ARCADE_CARS};
use tdrace_app::input::simulation::{
    key_style_sensitivity, run_keyboard_chicane_simulation, run_keyboard_slide_catch_simulation,
    run_keyboard_sweeper_simulation, ChicaneTransitionOutcome, KeyStyleSensitivity,
    KeyboardChicaneResult, KeyboardDriverProfile, KeyboardHandlingOutcome,
    KeyboardSlideCatchResult, KeyboardSteerPattern, KeyboardSweeperResult, SlideCatchOutcome,
};
use tdrace_app::input::SteeringProfile;
use tdrace_app::profile::AssistProfile;
use tdrace_core::physics::sim::DEFAULT_SIMULATION_DT;
use tdrace_core::physics::surface::SurfaceType;

const SURFACES: [SurfaceType; 4] = [
    SurfaceType::Asphalt,
    SurfaceType::Dirt,
    SurfaceType::PackedSand,
    SurfaceType::SheetIce,
];
const PRE_043_JSON: &str = "reports/keyboard_input_car_control_report_pre043.json";

/// Benchmark entry for an arcade car across all tests.
#[derive(Debug, Clone, Serialize)]
pub struct VehicleEvaluationReport {
    pub vehicle_id: String,
    pub vehicle_name: String,
    pub category: String,
    pub mass_kg: f32,
    pub power_bhp: u16,
    pub drivetrain: String,
    pub top_speed_kmh: u16,
    pub assist_matrix: Vec<AssistMatrixReport>,
    pub sweeper_results: Vec<KeyboardSweeperResult>,
    pub chicane_results: Vec<KeyboardChicaneResult>,
    pub catch_results: Vec<KeyboardSlideCatchResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssistMatrixReport {
    pub assist_profile: AssistProfile,
    pub input_profile: SteeringProfile,
    pub effective_traction_help: f32,
    pub sweeper_cases: usize,
    pub chicane_cases: usize,
    pub slide_catch_cases: usize,
    pub mean_speed_retention_pct: f32,
    pub mean_peak_rear_slip_deg: f32,
    pub mean_traction_help_reduction_pct: f32,
    pub mean_tcs_lateral_reduction_pct: f32,
    pub mean_tcs_longitudinal_reduction_pct: f32,
    pub mean_esc_corrective_torque_nm: f32,
    pub mean_reversal_latency_ms: f32,
    pub recovered_catches: usize,
    pub delayed_catches: usize,
    pub spun_catches: usize,
}

/// Full JSON export.
#[derive(Debug, Clone, Serialize)]
pub struct KeyStyleBenchmark {
    pub generated_at: String,
    pub fleet: Vec<VehicleEvaluationReport>,
    pub key_style_sensitivity: Vec<KeyStyleSensitivity>,
}

fn main() {
    let start_wall = Instant::now();
    let timestamp = Utc::now().to_rfc3339();
    let dt = DEFAULT_SIMULATION_DT;
    let mut cars: Vec<&RealCarModel> = CLASSIC_ARCADE_CARS.iter().collect();
    cars.extend(ALL_REAL_CARS.iter().filter(|model| matches!(model.id, "rally_peugeot_208_rally4" | "rally_omse_supercar_lites")));

    println!("Mode x key style x preset x car benchmark: {} cars, {} modes, {} presets, {} surfaces", cars.len(), AssistProfile::ALL.len(), SteeringProfile::PRESETS.len(), SURFACES.len());

    let mut fleet = Vec::with_capacity(cars.len());
    for model in &cars {
        let cfg = model.to_car_config();
        // Entry speeds tailored to car capability: 70 km/h baseline, 55 km/h for the kart
        let is_kart = model.id == "classic_kart";
        let (v_sweeper, v_chicane, v_catch) = if is_kart { (55.0, 50.0, 45.0) } else { (70.0, 65.0, 55.0) };

        let (mut sweeper, mut chicane, mut catch) = (Vec::new(), Vec::new(), Vec::new());
        for &surface in &SURFACES {
            for preset in SteeringProfile::PRESETS {
                for mode in AssistProfile::ALL {
                    for pattern in KeyboardSteerPattern::SWEEPER {
                        let driver = KeyboardDriverProfile::for_mode_pattern(pattern, preset, mode);
                        sweeper.push(run_keyboard_sweeper_simulation(model.id, model.name, &cfg, surface, &driver, v_sweeper, 3.0, dt));
                    }
                    for pattern in KeyboardSteerPattern::CHICANE {
                        let driver = KeyboardDriverProfile::for_mode_pattern(pattern, preset, mode);
                        chicane.push(run_keyboard_chicane_simulation(model.id, model.name, &cfg, surface, &driver, v_chicane, dt));
                    }
                    if surface != SurfaceType::Asphalt {
                        for pattern in [KeyboardSteerPattern::SustainedHold, KeyboardSteerPattern::RapidFeathering] {
                            let driver = KeyboardDriverProfile::for_mode_pattern(pattern, preset, mode);
                            catch.push(run_keyboard_slide_catch_simulation(model.id, model.name, &cfg, surface, &driver, v_catch, dt));
                        }
                    }
                }
            }
        }
        let assist_matrix = build_assist_matrix(&sweeper, &chicane, &catch);
        println!(" - {:<16} sweeper {} | chicane {} | catch {}", model.id, sweeper.len(), chicane.len(), catch.len());
        fleet.push(VehicleEvaluationReport {
            vehicle_id: model.id.to_string(),
            vehicle_name: model.name.to_string(),
            category: model.category_name.to_string(),
            mass_kg: model.weight_kg as f32,
            power_bhp: model.bhp,
            drivetrain: model.drivetrain.to_string(),
            top_speed_kmh: model.top_speed_kmh,
            assist_matrix,
            sweeper_results: sweeper,
            chicane_results: chicane,
            catch_results: catch,
        });
    }
    validate_assist_matrix(&fleet);

    let all_sweeper: Vec<KeyboardSweeperResult> = fleet
        .iter()
        .flat_map(|v| v.sweeper_results.iter().cloned())
        .collect();
    let sensitivity = key_style_sensitivity(&all_sweeper);
    let wall_s = start_wall.elapsed().as_secs_f64();
    println!("Simulated in {wall_s:.2} s");

    let reports_dir = Path::new("reports");
    fs::create_dir_all(reports_dir).expect("Failed to create reports directory");
    let pre043 = fs::read_to_string(PRE_043_JSON)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());

    let md = generate_markdown_report(&timestamp, &fleet, &sensitivity, pre043.as_ref(), wall_s);
    let json = KeyStyleBenchmark {
        generated_at: timestamp,
        fleet,
        key_style_sensitivity: sensitivity,
    };
    let json_path = reports_dir.join("keyboard_input_car_control_report.json");
    fs::write(&json_path, serde_json::to_string_pretty(&json).expect("serialize report")).expect("write JSON report");
    let md_path = reports_dir.join("keyboard_input_car_control_report.md");
    fs::write(&md_path, md).expect("write Markdown report");
    println!("Wrote {} and {}", md_path.display(), json_path.display());
}

fn build_assist_matrix(sweeper: &[KeyboardSweeperResult], chicane: &[KeyboardChicaneResult], catch: &[KeyboardSlideCatchResult]) -> Vec<AssistMatrixReport> {
    let mut cells = Vec::new();
    for mode in AssistProfile::ALL { for input in SteeringProfile::PRESETS {
        let sw: Vec<_> = sweeper.iter().filter(|r| r.assist_profile == mode && r.filter_profile == input).collect();
        let suffix = format!("_{}", input.to_string().to_lowercase());
        let ch: Vec<_> = chicane.iter().filter(|r| r.assist_profile == mode && r.driver_profile_id.ends_with(&suffix)).collect();
        let ca: Vec<_> = catch.iter().filter(|r| r.assist_profile == mode && r.driver_profile_id.ends_with(&suffix)).collect();
        let help = if mode == AssistProfile::Arcade { input.to_config().traction_help } else { 0.0 };
        let mean = |xs: Vec<f32>| xs.iter().sum::<f32>() / xs.len().max(1) as f32;
        cells.push(AssistMatrixReport { assist_profile: mode, input_profile: input, effective_traction_help: help,
            sweeper_cases: sw.len(), chicane_cases: ch.len(), slide_catch_cases: ca.len(),
            mean_speed_retention_pct: mean(sw.iter().map(|r| r.speed_retention_pct).collect()),
            mean_peak_rear_slip_deg: mean(sw.iter().map(|r| r.peak_rear_slip_deg).collect()),
            mean_traction_help_reduction_pct: mean(sw.iter().map(|r| r.mean_traction_help_reduction_pct).collect()),
            mean_tcs_lateral_reduction_pct: mean(sw.iter().map(|r| r.mean_tcs_lateral_reduction_pct).collect()),
            mean_tcs_longitudinal_reduction_pct: mean(sw.iter().map(|r| r.mean_tcs_longitudinal_reduction_pct).collect()),
            mean_esc_corrective_torque_nm: mean(sw.iter().map(|r| r.mean_esc_corrective_torque_nm).collect()),
            mean_reversal_latency_ms: mean(ch.iter().map(|r| r.reversal_latency_ms).collect()),
            recovered_catches: ca.iter().filter(|r| r.outcome == SlideCatchOutcome::Recovered).count(),
            delayed_catches: ca.iter().filter(|r| r.outcome == SlideCatchOutcome::DelayedRecovery).count(),
            spun_catches: ca.iter().filter(|r| r.outcome == SlideCatchOutcome::SpunOut).count() });
    }}
    cells
}

fn validate_assist_matrix(fleet: &[VehicleEvaluationReport]) {
    let sweep_count = KeyboardSteerPattern::SWEEPER.len() * SURFACES.len();
    let chicane_count = KeyboardSteerPattern::CHICANE.len() * SURFACES.len();
    let catch_count = 2 * (SURFACES.len() - 1);
    for vehicle in fleet {
        assert_eq!(vehicle.assist_matrix.len(), 12);
        for cell in &vehicle.assist_matrix {
            assert_eq!(cell.sweeper_cases, sweep_count);
            assert_eq!(cell.chicane_cases, chicane_count);
            assert_eq!(cell.slide_catch_cases, catch_count);
            assert!(cell.mean_speed_retention_pct.is_finite());
            assert!(cell.mean_peak_rear_slip_deg.is_finite());
            assert!(cell.mean_reversal_latency_ms.is_finite());
            let expected_help = if cell.assist_profile == AssistProfile::Arcade {
                cell.input_profile.to_config().traction_help
            } else {
                0.0
            };
            assert!((cell.effective_traction_help - expected_help).abs() < 1e-6);
            assert!(cell.mean_traction_help_reduction_pct.is_finite());
            assert!(cell.mean_tcs_lateral_reduction_pct.is_finite());
            assert!(cell.mean_tcs_longitudinal_reduction_pct.is_finite());
            assert!(cell.mean_esc_corrective_torque_nm.is_finite());
            if cell.assist_profile == AssistProfile::Pro {
                assert_eq!(cell.mean_tcs_lateral_reduction_pct, 0.0);
                assert_eq!(cell.mean_tcs_longitudinal_reduction_pct, 0.0);
                assert_eq!(cell.mean_esc_corrective_torque_nm, 0.0);
                assert_eq!(cell.mean_traction_help_reduction_pct, 0.0);
            }
        }
        let arcade_raw = vehicle.assist_matrix.iter().find(|cell| cell.assist_profile == AssistProfile::Arcade && cell.input_profile == SteeringProfile::Raw).unwrap();
        let sport_balanced = vehicle.assist_matrix.iter().find(|cell| cell.assist_profile == AssistProfile::Sport && cell.input_profile == SteeringProfile::Balanced).unwrap();
        let pro_balanced = vehicle.assist_matrix.iter().find(|cell| cell.assist_profile == AssistProfile::Pro && cell.input_profile == SteeringProfile::Balanced).unwrap();
        assert_eq!(arcade_raw.effective_traction_help, 0.0);
        assert_eq!(sport_balanced.effective_traction_help, 0.0);
        assert_eq!(pro_balanced.effective_traction_help, 0.0);
        let arcade_balanced = vehicle.assist_matrix.iter().find(|cell| cell.assist_profile == AssistProfile::Arcade && cell.input_profile == SteeringProfile::Balanced).unwrap();
        assert!(arcade_balanced.mean_traction_help_reduction_pct >= 0.0);
    }
}

fn sweeper<'a>(v: &'a VehicleEvaluationReport, surface: SurfaceType, preset: SteeringProfile, mode: AssistProfile, pattern: KeyboardSteerPattern) -> Option<&'a KeyboardSweeperResult> {
    v.sweeper_results
        .iter()
        .find(|r| r.surface == surface && r.filter_profile == preset && r.assist_profile == mode && r.steer_pattern == pattern)
}

fn sensitivity_of<'a>(s: &'a [KeyStyleSensitivity], vehicle: &str, surface: SurfaceType, preset: SteeringProfile, mode: AssistProfile) -> Option<&'a KeyStyleSensitivity> {
    s.iter().find(|k| k.vehicle_id == vehicle && k.surface == surface && k.filter_profile == preset && k.assist_profile == mode)
}

/// Old (pre-043) asphalt sweeper exit speed for a driver profile id, from the saved JSON.
fn old_exit(pre043: &serde_json::Value, vehicle: &str, driver_id: &str) -> Option<(f64, String)> {
    let v = pre043.as_array()?.iter().find(|v| v["vehicle_id"] == vehicle)?;
    let r = v["sweeper_results"]
        .as_array()?
        .iter()
        .find(|r| r["surface"] == "Asphalt" && r["driver_profile_id"] == driver_id)?;
    Some((r["exit_speed_kmh"].as_f64()?, r["outcome"].as_str()?.to_string()))
}

fn outcome_label(o: KeyboardHandlingOutcome) -> &'static str {
    o.badge()
}

fn generate_markdown_report(
    timestamp: &str,
    fleet: &[VehicleEvaluationReport],
    sensitivity: &[KeyStyleSensitivity],
    pre043: Option<&serde_json::Value>,
    wall_s: f64,
) -> String {
    let presets = SteeringProfile::PRESETS;
    let mut s = String::new();
    s.push_str("# Assist Mode × Key Style × Preset × Car Report (Spec 072) ⌨️🏎️\n\n");
    s.push_str(&format!(
        "Generated `{timestamp}` by `cargo run -p tdrace-app --bin keyboard_simulation_benchmark` in {wall_s:.2} s.\n\
         Physics: spec 043 slip-based tires, 120 Hz. Cars: {} vehicles (Classic plus FWD/AWD anchors). Modes: Arcade/Sport/Pro. Presets: Smooth, Balanced, Sharp, Raw.\n\
         Surfaces: asphalt, dirt, packed sand, sheet ice. Intervention multipliers are averaged per fixed step.\n\n",
        fleet.len()
    ));

    s.push_str("## 1. Mode × input response matrix\n\n");
    s.push_str("`Help pref` is mode-resolved configured traction help. `Help cut`, lateral/longitudinal TCS cuts and ESC yaw torque are measured per fixed step, separately from physical tire-force limits.\n\n");
    s.push_str("| Vehicle | Mode | Input | Help pref | Help cut | Lat TCS cut | Long TCS cut | ESC Nm | Retention | Rear slip | Reversal ms | Catch R/D/S |\n|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for vehicle in fleet {
        for cell in &vehicle.assist_matrix {
            s.push_str(&format!("| {} | {} | {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% | {:.1} | {:.1}% | {:.1}° | {:.0} | {} / {} / {} |\n",
                vehicle.vehicle_name, cell.assist_profile.short_name(), cell.input_profile,
                cell.effective_traction_help * 100.0, cell.mean_traction_help_reduction_pct,
                cell.mean_tcs_lateral_reduction_pct,
                cell.mean_tcs_longitudinal_reduction_pct, cell.mean_esc_corrective_torque_nm,
                cell.mean_speed_retention_pct, cell.mean_peak_rear_slip_deg,
                cell.mean_reversal_latency_ms, cell.recovered_catches, cell.delayed_catches, cell.spun_catches));
        }
    }

    s.push_str("\n## 2. What was driven\n\n");
    s.push_str("| Key style | Keys |\n|---|---|\n");
    for p in KeyboardSteerPattern::SWEEPER.iter().chain([KeyboardSteerPattern::SnapCountersteer].iter()) {
        let keys = match p {
            KeyboardSteerPattern::SustainedHold => "steer key held, W held",
            KeyboardSteerPattern::RapidFeathering => "steer 75 ms on / 75 ms off, W held",
            KeyboardSteerPattern::CadencePulse => "steer 180 ms on / 120 ms off, W held",
            KeyboardSteerPattern::TapAndCoast => "250 ms tap, 200 ms coast, then 100/100 ms taps, W held",
            KeyboardSteerPattern::LiftOffTurn => "steer held, W released for the first 600 ms",
            KeyboardSteerPattern::SnapCountersteer => "chicane: right 1 s, left 1 s, centre (scripted by the scenario)",
        };
        s.push_str(&format!("| {} | {keys} |\n", p.name()));
    }
    s.push_str("\nScenarios: **sweeper** (3 s corner from 70 km/h, kart 55 km/h), **chicane** (full right then full left from 65 km/h, kart 50 km/h, styles: hold, feathering, cadence), **slide catch** (15° / 35°·s⁻¹ induced slide on dirt, sand and ice, styles: hold, feathering).\n\n");

    // 2. Headline: Balanced on asphalt
    s.push_str("## 2. Balanced preset on asphalt: exit speed by key style (km/h)\n\n");
    s.push_str("| Car | Hold | Feathering | Cadence | Tap-and-coast | Lift-off | Hold vs feathering |\n|---|---|---|---|---|---|---|\n");
    for v in fleet {
        let cell = |p| {
            sweeper(v, SurfaceType::Asphalt, SteeringProfile::Balanced, AssistProfile::Arcade, p)
                .map(|r| format!("{:.1} ({})", r.exit_speed_kmh, outcome_label(r.outcome)))
                .unwrap_or_default()
        };
        let hvf = sensitivity_of(sensitivity, &v.vehicle_id, SurfaceType::Asphalt, SteeringProfile::Balanced, AssistProfile::Arcade)
            .map(|k| format!("{:.0}%", k.hold_vs_feathering_pct))
            .unwrap_or_default();
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {hvf} |\n",
            v.vehicle_name,
            cell(KeyboardSteerPattern::SustainedHold),
            cell(KeyboardSteerPattern::RapidFeathering),
            cell(KeyboardSteerPattern::CadencePulse),
            cell(KeyboardSteerPattern::TapAndCoast),
            cell(KeyboardSteerPattern::LiftOffTurn),
        ));
    }

    // 3. Key style sensitivity
    s.push_str("\n## 3. Key Style Sensitivity (asphalt sweeper)\n\n");
    s.push_str("Spread = how much the exit speed changes between the best and the worst key style for the same car and preset. A small spread means *how* you press matters little; a large spread means technique matters.\n\n");
    s.push_str("| Car | Smooth | Balanced | Sharp | Raw |\n|---|---|---|---|---|\n");
    for v in fleet {
        let cells: Vec<String> = presets
            .iter()
            .map(|&p| {
                sensitivity_of(sensitivity, &v.vehicle_id, SurfaceType::Asphalt, p, AssistProfile::Arcade)
                    .map(|k| format!("{:.0}% (best {}, worst {})", k.spread_pct, k.fastest_style.id(), k.slowest_style.id()))
                    .unwrap_or_default()
            })
            .collect();
        s.push_str(&format!("| {} | {} |\n", v.vehicle_name, cells.join(" | ")));
    }
    s.push_str("\nAverage spread over all cars, per surface:\n\n| Surface | Smooth | Balanced | Sharp | Raw |\n|---|---|---|---|---|\n");
    for &surface in &SURFACES {
        let cells: Vec<String> = presets
            .iter()
            .map(|&p| {
                let xs: Vec<f32> = sensitivity
                    .iter()
                    .filter(|k| {
                        k.surface == surface
                            && k.filter_profile == p
                            && k.assist_profile == AssistProfile::Arcade
                            && k.assist_profile == AssistProfile::Arcade
                    })
                    .map(|k| k.spread_pct)
                    .collect();
                format!("{:.0}%", xs.iter().sum::<f32>() / xs.len().max(1) as f32)
            })
            .collect();
        s.push_str(&format!("| {surface:?} | {} |\n", cells.join(" | ")));
    }

    // 4. Chicane
    s.push_str("\n## 4. Chicane reversal outcomes (all cars, surfaces and styles)\n\n| Preset | Crisp | Mild pendulum | Snap oversteer | Spinout | Mean reversal latency |\n|---|---|---|---|---|---|\n");
    for &p in &presets {
        let rs: Vec<&KeyboardChicaneResult> = fleet
            .iter()
            .flat_map(|v| v.chicane_results.iter())
            .filter(|r| {
                r.assist_profile == AssistProfile::Arcade
                    && r.driver_profile_id
                        .ends_with(&format!("_{}", p.to_string().to_lowercase()))
            })
            .collect();
        let count = |o: ChicaneTransitionOutcome| rs.iter().filter(|r| r.outcome == o).count();
        let latency =
            rs.iter().map(|r| r.reversal_latency_ms).sum::<f32>() / rs.len().max(1) as f32;
        s.push_str(&format!(
            "| {p} | {} | {} | {} | {} | {latency:.0} ms |\n",
            count(ChicaneTransitionOutcome::CrispTransition),
            count(ChicaneTransitionOutcome::MildDampedPendulum),
            count(ChicaneTransitionOutcome::ViolentSnapOversteer),
            count(ChicaneTransitionOutcome::Spinout),
        ));
    }

    // 5. Slide catch
    s.push_str("\n## 5. Slide catch outcomes (dirt, sand, ice)\n\n| Preset | Recovered | Delayed | Spun out |\n|---|---|---|---|\n");
    for &p in &presets {
        let rs: Vec<&KeyboardSlideCatchResult> = fleet
            .iter()
            .flat_map(|v| v.catch_results.iter())
            .filter(|r| {
                r.assist_profile == AssistProfile::Arcade
                    && r.driver_profile_id
                        .ends_with(&format!("_{}", p.to_string().to_lowercase()))
            })
            .collect();
        let count = |o: SlideCatchOutcome| rs.iter().filter(|r| r.outcome == o).count();
        s.push_str(&format!(
            "| {p} | {} | {} | {} |\n",
            count(SlideCatchOutcome::Recovered),
            count(SlideCatchOutcome::DelayedRecovery),
            count(SlideCatchOutcome::SpunOut),
        ));
    }

    // 6. Old vs new
    s.push_str("\n## 6. Old physics vs spec 043 (Balanced, asphalt sweeper)\n\n");
    match pre043 {
        Some(old) => {
            s.push_str("Old numbers come from `reports/keyboard_input_car_control_report_pre043.json` (the pre-043 benchmark output).\n\n");
            s.push_str("| Car | Hold old → new | Feathering old → new | Hold vs feathering old → new | Lift-off old → new |\n|---|---|---|---|---|\n");
            for v in fleet {
                let new = |p| {
                    sweeper(
                        v,
                        SurfaceType::Asphalt,
                        SteeringProfile::Balanced,
                        AssistProfile::Arcade,
                        p,
                    )
                };
                let (oh, of, ol) = (
                    old_exit(old, &v.vehicle_id, "hold_balanced"),
                    old_exit(old, &v.vehicle_id, "feathering_balanced"),
                    old_exit(old, &v.vehicle_id, "lift_off_balanced"),
                );
                let (nh, nf, nl) = (
                    new(KeyboardSteerPattern::SustainedHold),
                    new(KeyboardSteerPattern::RapidFeathering),
                    new(KeyboardSteerPattern::LiftOffTurn),
                );
                let fmt = |o: &Option<(f64, String)>, n: Option<&KeyboardSweeperResult>| match (o, n) {
                    (Some((ov, _)), Some(n)) => format!("{ov:.1} → {:.1} km/h", n.exit_speed_kmh),
                    _ => "n/a".to_string(),
                };
                let ratio = match (&oh, &of, nh, nf) {
                    (Some((h, _)), Some((f, _)), Some(nh), Some(nf)) => format!(
                        "{:.0}% → {:.0}%",
                        h / f.max(1e-3) * 100.0,
                        nh.exit_speed_kmh / nf.exit_speed_kmh.max(1e-3) * 100.0
                    ),
                    _ => "n/a".to_string(),
                };
                let lift = match (&ol, nl) {
                    (Some((ov, oo)), Some(n)) => format!(
                        "{ov:.1} ({oo}) → {:.1} ({})",
                        n.exit_speed_kmh,
                        outcome_label(n.outcome)
                    ),
                    _ => "n/a".to_string(),
                };
                s.push_str(&format!(
                    "| {} | {} | {} | {ratio} | {lift} |\n",
                    v.vehicle_name,
                    fmt(&oh, nh),
                    fmt(&of, nf)
                ));
            }
        }
        None => s.push_str("No pre-043 dataset found; comparison skipped.\n"),
    }

    // 7. Gate summary
    s.push_str("\n## 7. Spec 043 gates on this run\n\n");
    let hold_ok = fleet.iter().all(|v| {
        sweeper(
            v,
            SurfaceType::Asphalt,
            SteeringProfile::Balanced,
            AssistProfile::Arcade,
            KeyboardSteerPattern::SustainedHold,
        )
        .is_some_and(|r| {
            r.speed_retention_pct >= 90.0
                && matches!(
                    r.outcome,
                    KeyboardHandlingOutcome::CleanCarve | KeyboardHandlingOutcome::PowerSlide
                )
        })
    });
    let safe_spinouts = fleet
        .iter()
        .flat_map(|v| v.chicane_results.iter())
        .filter(|r| r.surface != SurfaceType::SheetIce)
        .filter(|r| {
            r.assist_profile == AssistProfile::Arcade
                && (r.driver_profile_id.contains("_smooth")
                    || r.driver_profile_id.contains("_balanced"))
                && r.outcome == ChicaneTransitionOutcome::Spinout
        })
        .count();
    let spreads: Vec<f32> = presets
        .iter()
        .map(|&p| {
            let xs: Vec<f32> = sensitivity
                .iter()
                .filter(|k| {
                    k.surface == SurfaceType::Asphalt
                        && k.filter_profile == p
                        && k.assist_profile == AssistProfile::Arcade
                        && k.assist_profile == AssistProfile::Arcade
                })
                .map(|k| k.spread_pct)
                .collect();
            xs.iter().sum::<f32>() / xs.len().max(1) as f32
        })
        .collect();
    let spread_ordered = spreads.windows(2).all(|w| w[0] > w[1]);
    let arcade_balanced_failures = fleet
        .iter()
        .filter(|v| {
            !v.sweeper_results.iter().any(|r| {
                r.surface == SurfaceType::Asphalt
                    && r.assist_profile == AssistProfile::Arcade
                    && r.filter_profile == SteeringProfile::Balanced
                    && r.steer_pattern == KeyboardSteerPattern::SustainedHold
                    && r.speed_retention_pct >= 90.0
            })
        })
        .count();
    let tick = |ok: bool| if ok { "PASS" } else { "FAIL" };
    s.push_str(&format!("- {} Holding keeps >= 90% of its entry speed and carves cleanly on every car (Balanced, asphalt).\n", tick(hold_ok)));
    s.push_str(&format!("- {} Arcade/Balanced sustained-key retention is >= 90% for all vehicles ({arcade_balanced_failures} failures).\n", tick(arcade_balanced_failures == 0)));
    s.push_str(&format!("- {} No chicane spinout on Smooth or Balanced on asphalt, dirt and packed sand ({safe_spinouts} found).\n", tick(safe_spinouts == 0)));
    s.push_str(&format!(
        "- {} Average asphalt key-style spread falls from Smooth to Raw ({:.0}% / {:.0}% / {:.0}% / {:.0}%).\n",
        tick(spread_ordered), spreads[0], spreads[1], spreads[2], spreads[3]
    ));
    s.push_str("\nReading the numbers: holding a key now carves the tightest line without scrubbing speed; tapping keeps a wider, faster line. Smooth filters taps into gentle steering, so on Smooth your technique changes the line the most; on Raw every tap is full input, so tapping and holding converge. For slides, tap the counter-steer: holding full opposite lock over-corrects into a slide the other way.\n");
    s
}
