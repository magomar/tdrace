//! Spec 087: the committed Codex data must match a fresh export of the game data.

use std::path::{Path, PathBuf};

use serde_json::Value;
use tdrace_app::catalog::{ALL_REAL_CARS, CLASSIC_ARCADE_CARS};
use tdrace_app::codex::{export, Scope, DEFAULT_OUT_DIR, LAUNCH_MODULES};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn items(files: &[(&str, String)], name: &str) -> Vec<Value> {
    let json = &files.iter().find(|(n, _)| *n == name).unwrap_or_else(|| panic!("{name} not exported")).1;
    let doc: Value = serde_json::from_str(json).unwrap();
    doc["items"].as_array().unwrap().clone()
}

#[test]
fn committed_codex_data_is_fresh() {
    let dir = repo_root().join(DEFAULT_OUT_DIR);
    let files = export(Scope::All, &repo_root()).unwrap();
    let stale: Vec<&str> = files
        .iter()
        .filter(|(name, json)| std::fs::read_to_string(dir.join(name)).ok().as_deref() != Some(json.as_str()))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        stale.is_empty(),
        "stale Codex data in {DEFAULT_OUT_DIR}: {}. Run: cargo run -p tdrace-app --bin export_codex",
        stale.join(", ")
    );
}

#[test]
fn all_scope_exports_every_playable_car_and_no_vault_content() {
    let files = export(Scope::All, &repo_root()).unwrap();
    let cars = items(&files, "cars.json");
    assert_eq!(cars.len(), CLASSIC_ARCADE_CARS.len() + ALL_REAL_CARS.len());
    for car in &cars {
        assert_ne!(car["module"], "vault");
        assert!(car["physics"]["mass"].as_f64().unwrap() > 0.0, "{} has no physics", car["id"]);
    }
    assert!(items(&files, "circuits.json").iter().all(|c| c["module"] != "vault"));
    assert_eq!(items(&files, "surfaces.json").len(), 15);
}

#[test]
fn classic_ax_brawler_power_comes_from_the_catalogue() {
    let files = export(Scope::All, &repo_root()).unwrap();
    let brawler = items(&files, "cars.json").into_iter().find(|c| c["id"] == "classic_ax_brawler").unwrap();
    let model = CLASSIC_ARCADE_CARS.iter().find(|c| c.id == "classic_ax_brawler").unwrap();
    assert_eq!(brawler["bhp"], u64::from(model.bhp));
}

#[test]
fn launch_scope_holds_only_launch_modules() {
    let files = export(Scope::Launch, &repo_root()).unwrap();
    for name in ["modules.json", "cars.json", "circuits.json"] {
        for item in items(&files, name) {
            let module = item.get("module").or_else(|| item.get("id")).and_then(Value::as_str).unwrap();
            assert!(LAUNCH_MODULES.contains(&module), "{name}: {module} is not a launch module");
        }
    }
    let modules = items(&files, "modules.json");
    assert_eq!(modules.len(), LAUNCH_MODULES.len());
}

/// The Codex shows chassis, wheels, suspension and drivetrain layout per base preset (`chassis.json`),
/// so every car built on one preset must share them. Only mass, power, grip and aero vary per car.
#[test]
fn cars_on_one_base_preset_share_their_platform() {
    let models: Vec<_> = CLASSIC_ARCADE_CARS.iter().chain(ALL_REAL_CARS.iter()).collect();
    for first in &models {
        let a = first.to_car_config();
        for other in models.iter().filter(|m| m.base_car_choice == first.base_car_choice) {
            let b = other.to_car_config();
            let who = format!("{} vs {} ({:?})", first.id, other.id, first.base_car_choice);
            assert_eq!(a.chassis, b.chassis, "chassis: {who}");
            assert_eq!(a.suspension, b.suspension, "suspension: {who}");
            assert_eq!(a.front_differential, b.front_differential, "front differential: {who}");
            assert_eq!(a.rear_differential, b.rear_differential, "rear differential: {who}");
            assert_eq!(a.engine_placement, b.engine_placement, "engine placement: {who}");
            assert_eq!(
                (a.wheelbase, a.track_width, a.cg_to_front, a.cg_to_rear, a.cg_height),
                (b.wheelbase, b.track_width, b.cg_to_front, b.cg_to_rear, b.cg_height),
                "geometry: {who}"
            );
            for (wa, wb) in a.wheels.iter().zip(b.wheels.iter()) {
                assert_eq!(
                    (wa.tire_radius, wa.tire_width, wa.rotational_inertia, wa.compound.id),
                    (wb.tire_radius, wb.tire_width, wb.rotational_inertia, wb.compound.id),
                    "wheels: {who}"
                );
            }
            let (ta, tb) = (a.tire, b.tire);
            assert_eq!(
                (ta.peak_slip_angle_deg, ta.peak_slip_ratio, ta.slide_grip, ta.falloff, ta.load_sensitivity, ta.power_slide),
                (tb.peak_slip_angle_deg, tb.peak_slip_ratio, tb.slide_grip, tb.falloff, tb.load_sensitivity, tb.power_slide),
                "tire shape: {who}"
            );
            assert_eq!(a.rear_axle.peak_slip_scale, b.rear_axle.peak_slip_scale, "rear axle: {who}");
        }
    }
}

#[test]
fn every_platform_part_points_at_an_exported_table_row() {
    let files = export(Scope::All, &repo_root()).unwrap();
    let ids = |name: &str| -> Vec<Value> { items(&files, name).iter().map(|i| i["id"].clone()).collect() };
    let (suspension, tyres, placements) = (ids("suspension.json"), ids("tyres.json"), ids("drivetrain.json"));
    let car_ids = ids("cars.json");
    let mut covered = 0;
    for p in items(&files, "chassis.json") {
        assert!(suspension.contains(&p["suspension"]["front"]["archetype"]), "{}", p["id"]);
        assert!(suspension.contains(&p["suspension"]["rear"]["archetype"]), "{}", p["id"]);
        assert!(placements.contains(&p["engine_placement"]), "{}", p["id"]);
        for w in p["wheels"].as_array().unwrap() {
            assert!(tyres.contains(&w["compound"]), "{}", p["id"]);
        }
        for car in p["cars"].as_array().unwrap() {
            assert!(car_ids.contains(car), "{} lists unknown car {car}", p["id"]);
            covered += 1;
        }
    }
    assert_eq!(covered, car_ids.len(), "every car belongs to exactly one platform");
}
