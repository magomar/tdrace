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
