use tdrace_app::config::GameConfig;
use tdrace_app::game::RaceSession;
use tdrace_core::physics::car::ImpactZone;

#[test]
fn test_car_damage_default_disabled_in_config() {
    let config = GameConfig::default();
    assert!(!config.gameplay.car_damage);

    let toml_str = toml::to_string_pretty(&config).expect("Serialize default config");
    let loaded: GameConfig = toml::from_str(&toml_str).expect("Deserialize default config");
    assert!(!loaded.gameplay.car_damage);
}

#[test]
fn test_car_damage_config_toml_roundtrip() {
    let toml_enabled = r#"
[gameplay]
default_track = "gt_coastal_grand_prix"
default_car = "sports_car"
default_laps = 3
default_num_bots = 7
default_assist_profile = "arcade"
car_damage = true
"#;
    let loaded: GameConfig = toml::from_str(toml_enabled).expect("Deserialize enabled config");
    assert!(loaded.gameplay.car_damage);

    let toml_disabled = r#"
[gameplay]
default_track = "gt_coastal_grand_prix"
default_car = "sports_car"
default_laps = 3
default_num_bots = 7
default_assist_profile = "arcade"
car_damage = false
"#;
    let loaded_dis: GameConfig = toml::from_str(toml_disabled).expect("Deserialize disabled config");
    assert!(!loaded_dis.gameplay.car_damage);
}

#[test]
fn test_session_initializes_with_car_damage_disabled_by_default() {
    let mut session = RaceSession::new();
    session.hof_db = Some(tdrace_app::db::HallOfFameDb::open_in_memory().unwrap());
    session.init_race();

    // Verify gameplay config default
    assert!(!session.config.gameplay.car_damage);

    // Verify race rules damage_enabled
    assert!(!session.world.rules.damage_enabled);

    // Verify all vehicles have damage_enabled = false
    assert!(!session.world.vehicles.is_empty());
    for car in &session.world.vehicles {
        assert!(!car.config.damage_enabled);
    }
}

#[test]
fn test_disabled_car_damage_ignores_collision_damage() {
    let mut session = RaceSession::new();
    session.hof_db = Some(tdrace_app::db::HallOfFameDb::open_in_memory().unwrap());
    session.init_race();

    assert!(!session.world.vehicles[0].config.damage_enabled);
    assert_eq!(session.world.vehicles[0].state.health, 1.0);

    // Attempt direct impact damage
    session.world.vehicles[0].apply_collision_damage_to_zone(ImpactZone::FrontNose, 20_000.0);
    assert_eq!(session.world.vehicles[0].state.health, 1.0);
    assert_eq!(session.world.vehicles[0].state.chassis_health, 1.0);
    assert_eq!(session.world.vehicles[0].state.engine_health, 1.0);
    assert_eq!(session.world.vehicles[0].state.suspension_health, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn test_settings_modal_enable_and_disable_car_damage_lifecycle() {
    let mut session = RaceSession::new();
    session.hof_db = Some(tdrace_app::db::HallOfFameDb::open_in_memory().unwrap());
    session.init_race();

    // 1. Initial state: disabled
    assert!(!session.config.gameplay.car_damage);
    assert!(!session.world.rules.damage_enabled);
    assert!(!session.world.vehicles[0].config.damage_enabled);

    // 2. Open settings modal, verify pre-population
    session.open_settings_modal();
    assert!(session.is_settings_modal_open());
    {
        let modal = session.settings_modal.as_ref().unwrap();
        assert!(!modal.car_damage());
    }

    // 3. Mutate to Enabled, close and save
    if let Some(ref mut modal) = session.settings_modal {
        modal.set_car_damage(true);
        assert!(modal.car_damage());
    }
    session.close_settings_modal(true);
    assert!(!session.is_settings_modal_open());

    // 4. Verify applied to session config, race rules, and vehicles
    assert!(session.config.gameplay.car_damage);
    assert!(session.base_config.gameplay.car_damage);
    assert!(session.world.rules.damage_enabled);
    for car in &session.world.vehicles {
        assert!(car.config.damage_enabled);
    }

    // 5. Apply severe damage while enabled
    session.world.vehicles[0].apply_collision_damage_to_zone(ImpactZone::FrontNose, 15_000.0);
    assert!(session.world.vehicles[0].state.health < 1.0);
    assert!(session.world.vehicles[0].state.chassis_health < 1.0);

    // 6. Open settings modal, toggle back to Disabled, close and save
    session.open_settings_modal();
    {
        let modal = session.settings_modal.as_ref().unwrap();
        assert!(modal.car_damage());
    }
    if let Some(ref mut modal) = session.settings_modal {
        modal.set_car_damage(false);
    }
    session.close_settings_modal(true);

    // 7. Verify disabled AND all vehicle components repaired to 100% health
    assert!(!session.config.gameplay.car_damage);
    assert!(!session.base_config.gameplay.car_damage);
    assert!(!session.world.rules.damage_enabled);
    for car in &session.world.vehicles {
        assert!(!car.config.damage_enabled);
        assert_eq!(car.state.health, 1.0);
        assert_eq!(car.state.chassis_health, 1.0);
        assert_eq!(car.state.engine_health, 1.0);
        assert_eq!(car.state.suspension_health, [1.0, 1.0, 1.0, 1.0]);
    }
}

#[test]
fn test_settings_modal_car_damage_cancel_discards_changes() {
    let mut session = RaceSession::new();
    session.hof_db = Some(tdrace_app::db::HallOfFameDb::open_in_memory().unwrap());
    session.init_race();

    assert!(!session.config.gameplay.car_damage);

    // Open, toggle to enabled, cancel without saving
    session.open_settings_modal();
    if let Some(ref mut modal) = session.settings_modal {
        modal.set_car_damage(true);
    }
    session.close_settings_modal(false);

    // Should remain unchanged (false)
    assert!(!session.config.gameplay.car_damage);
    assert!(!session.world.rules.damage_enabled);
    for car in &session.world.vehicles {
        assert!(!car.config.damage_enabled);
    }
}
