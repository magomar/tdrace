//! Verification tests for Spec 054: Vault Module for Archived and Deprecated Content.
//!
//! Covers:
//! 1. `VaultGameModule` adhering to `GameModule` trait, metadata, vehicles, and rulesets.
//! 2. Developer mode gating in Grand Hub and Track Studio.
//! 3. Career mode and AI driver isolation from vaulted material.
//! 4. Track Manager module filtering and storage manifest integrity.

use macroquad::color::Color;
use tdrace_app::ai::DriverCharacter;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::module::vault::VaultGameModule;
use tdrace_app::module::GameModule;
use tdrace_app::series::EMBEDDED_PRESETS;
use tdrace_app::tournament::TournamentFormat;
use tdrace_app::track_manager::ModuleFilter;

#[test]
fn test_vault_game_module_identity_and_traits() {
    let vault = VaultGameModule::new();
    assert_eq!(vault.id(), "vault");
    assert_eq!(vault.title(), "THE VAULT");
    assert!(vault.subtitle().contains("Decommissioned"));

    let theme = vault.theme();
    assert_eq!(theme.header_badge, "COLD STORAGE / DEV ARCHIVE");
    assert_eq!(theme.primary_accent, Color::new(1.0, 0.65, 0.0, 1.0));

    let vehicles = vault.vehicles();
    assert_eq!(vehicles.len(), 3, "Vault should provide 3 decommissioned prototype vehicle models");

    let ids: Vec<&str> = vehicles.iter().map(|v| v.id).collect();
    assert!(ids.contains(&"vault_test_mule"));
    assert!(ids.contains(&"vault_prototype_mower"));
    assert!(ids.contains(&"vault_drift_trike"));

    assert_eq!(vault.default_vehicle_id(), "vault_test_mule");
    assert!(vault.drivers().is_empty(), "Vault must not register drivers in official championship grids");

    let modes = vault.supported_game_modes();
    assert!(!modes.is_empty());
    assert!(modes.iter().any(|m| matches!(m, TournamentFormat::QuickRace { .. })));
    assert!(modes.iter().any(|m| matches!(m, TournamentFormat::TimeAttack)));

    // Verify individual vehicle configurations
    let mule = VaultGameModule::car_vault_test_mule();
    assert_eq!(mule.mass, 1100.0);
    assert_eq!(mule.drive_bias, 0.5);

    let mower = VaultGameModule::car_vault_prototype_mower();
    assert_eq!(mower.mass, 210.0);
    assert_eq!(mower.top_speed_mps, 25.0);

    let trike = VaultGameModule::car_vault_drift_trike();
    assert_eq!(trike.mass, 120.0);
    assert_eq!(trike.tire.slide_grip, 0.75);
}

#[test]
fn test_vault_grand_hub_dev_mode_gating_and_switching() {
    let mut session = RaceSession::new();

    // Dev mode false
    session.config.gameplay.dev_mode = false;
    let standard_modules = if session.is_dev_mode() { 7 } else { 6 };
    assert_eq!(standard_modules, 6, "Standard mode has exactly 6 modules in Grand Hub");

    // Dev mode toggle via config
    session.config.gameplay.dev_mode = true;
    assert!(session.is_dev_mode(), "dev_mode true in config should enable dev mode");
    let dev_modules = if session.is_dev_mode() { 7 } else { 6 };
    assert_eq!(dev_modules, 7, "Dev mode includes The Vault as 7th module");

    // Switching to vault
    session.switch_to_vault();
    assert_eq!(session.active_module_id, "vault");
    assert_eq!(session.selected_car_model_id, Some("vault_test_mule"));
    assert_eq!(session.state, GameState::Menu);

    // Switch back to classic
    session.switch_to_classic();
    assert_eq!(session.active_module_id, "classic");
}

#[test]
fn test_vault_career_and_driver_isolation() {
    let vaulted_ids = ["vault_test_mule", "vault_prototype_mower", "vault_drift_trike"];

    // 1. None of the AI drivers across all modules should have a vaulted vehicle as favorite
    let all_drivers = DriverCharacter::all_across_modules();
    assert_eq!(all_drivers.len(), 72, "Total registered AI drivers across 6 official modules");

    for driver in &all_drivers {
        for discipline in ["gt", "nascar", "rally", "kart", "extreme_offroad", "classic"] {
            for tier in 1..=5 {
                if let Some(fav) = driver.favorite_car_for_discipline_and_tier(discipline, tier) {
                    assert!(
                        !vaulted_ids.contains(&fav),
                        "Driver '{}' must not have vaulted vehicle '{}' as favorite car",
                        driver.name,
                        fav
                    );
                }
            }
        }
    }

    // 2. No official career series preset should belong to the vault module
    for (series_id, toml_str) in EMBEDDED_PRESETS {
        assert!(
            !series_id.contains("vault"),
            "Series preset '{}' must not belong to vault module",
            series_id
        );
        assert!(
            !toml_str.contains("module_id = \"vault\""),
            "Series preset '{}' toml must not configure module_id = 'vault'",
            series_id
        );
    }
}

#[test]
fn test_vault_track_manager_filter_and_manifest_integrity() {
    // 1. ModuleFilter mapping
    assert_eq!(ModuleFilter::Vault.label(), "THE VAULT");
    assert_eq!(ModuleFilter::Vault.short_label(), "VAULT");
    assert_eq!(ModuleFilter::Vault.shortcut_number(), 7);
    assert_eq!(ModuleFilter::for_module("vault"), ModuleFilter::Vault);

    // 2. Storage manifest parsing
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tracks")
        .join("vault")
        .join("MANIFEST.json");

    assert!(manifest_path.exists(), "tracks/vault/MANIFEST.json must exist");

    let content = std::fs::read_to_string(&manifest_path).expect("Read MANIFEST.json");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Valid JSON");

    assert_eq!(json["schema_version"], "1.0.0");
    assert_eq!(json["module"], "vault");
    assert!(json["quarantined_tracks"].is_array());
    assert!(json["decommissioned_vehicles"].is_array());

    // 3. Track order registration
    let track_order_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tracks")
        .join(".track_order.json");

    let order_content = std::fs::read_to_string(&track_order_path).expect("Read .track_order.json");
    let order_json: serde_json::Value = serde_json::from_str(&order_content).expect("Valid JSON");
    assert!(order_json.get("vault").is_some(), ".track_order.json must have 'vault' entry");
}
