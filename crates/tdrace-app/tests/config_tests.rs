use std::io::Write;
use tdrace_app::config::{GameConfig, ZoomLevelConfig};
use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::CarChoice;
use tdrace_core::physics::config::{AssistProfile, CarConfig};

#[test]
fn test_default_config_roundtrip_toml() {
    let original = GameConfig::default();
    let toml_str = toml::to_string_pretty(&original).expect("Serialization to TOML should succeed");

    assert!(toml_str.contains("[camera]"));
    assert!(toml_str.contains("[[camera.levels]]"));
    assert!(toml_str.contains("[input]"));
    assert!(toml_str.contains("[audio]"));
    assert!(toml_str.contains("[gameplay]"));
    assert!(toml_str.contains("[cars.sports_car]"));
    assert!(toml_str.contains("[cars.drift_car]"));

    let restored: GameConfig = toml::from_str(&toml_str).expect("Deserialization from TOML should succeed");
    assert_eq!(original.camera, restored.camera);
    assert_eq!(original.input, restored.input);
    assert_eq!(original.audio, restored.audio);
    assert_eq!(original.gameplay, restored.gameplay);
    assert_eq!(original.cars.len(), restored.cars.len());
}

#[test]
fn test_custom_car_specs_override() {
    let mut config = GameConfig::default();
    let mut sports = CarConfig::sports_car();
    sports.top_speed_mps = 75.0; // Overclocked to ~270 km/h
    sports.mass = 850.0;
    sports.max_engine_force = 9999.0;
    config.cars.insert("sports_car".to_string(), sports);

    let toml_str = toml::to_string_pretty(&config).expect("Serialize custom car config");
    let loaded: GameConfig = toml::from_str(&toml_str).expect("Deserialize custom car config");

    let car_cfg = loaded.get_car_config(CarChoice::SportsCar);
    assert_eq!(car_cfg.top_speed_mps, 75.0);
    assert_eq!(car_cfg.mass, 850.0);
    assert_eq!(car_cfg.max_engine_force, 9999.0);

    // Other cars maintain their default specs
    let drift_cfg = loaded.get_car_config(CarChoice::DriftCar);
    assert_eq!(drift_cfg.mass, 980.0);
}

#[test]
fn test_custom_camera_zoom_levels_configuration() {
    let mut config = GameConfig::default();
    config.camera.levels = vec![
        ZoomLevelConfig {
            name: "Hyper-Close".to_string(),
            mode: "follow".to_string(),
            min_zoom: 20.0,
            max_zoom: 30.0,
        },
        ZoomLevelConfig {
            name: "Mid".to_string(),
            mode: "follow".to_string(),
            min_zoom: 12.0,
            max_zoom: 18.0,
        },
        ZoomLevelConfig {
            name: "Bird-Eye".to_string(),
            mode: "overview".to_string(),
            min_zoom: 4.0,
            max_zoom: 4.0,
        },
    ];
    config.camera.default_level_index = 1;

    let toml_str = toml::to_string_pretty(&config).unwrap();
    let loaded: GameConfig = toml::from_str(&toml_str).unwrap();

    let mut camera = tdrace_app::camera::RaceCamera::from_config(&loaded.camera);
    assert_eq!(camera.levels.len(), 3);
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(camera.current_zoom_level().name, "Mid");

    // Cycle to Bird-Eye (overview)
    let lvl = camera.cycle_zoom_level();
    assert_eq!(lvl.name, "Bird-Eye");
    assert_eq!(camera.mode, tdrace_app::camera::CameraMode::StaticOverview);

    // Cycle to Hyper-Close (follow)
    let lvl = camera.cycle_zoom_level();
    assert_eq!(lvl.name, "Hyper-Close");
    assert_eq!(camera.mode, tdrace_app::camera::CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 20.0);
    assert_eq!(camera.max_zoom_scale, 30.0);
}

#[test]
fn test_session_initialization_with_custom_config() {
    let mut config = GameConfig::default();
    config.gameplay.default_track = "kart_arena".to_string();
    config.gameplay.default_laps = 5;
    config.gameplay.default_num_bots = 7;
    config.audio.master_volume = 0.42;
    config.input.steer_rise_rate = 8.5;

    let session = RaceSession::new_with_config(config);
    assert_eq!(session.total_laps, 5);
    assert_eq!(session.num_bots, 7);
    assert!((session.audio.settings.master_volume - 0.42).abs() < 1e-4);
    assert_eq!(session.input.filter.config.steer_rise_rate, 8.5);
}

#[test]
fn test_config_save_and_load_from_path() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let config_path = temp_dir.join("test_config.toml");

    let mut config = GameConfig::default();
    config.gameplay.default_track = "kart_arena".to_string();
    config.save_to_path(&config_path).expect("Save to temp file should succeed");

    assert!(config_path.exists());
    let loaded = GameConfig::load_from_path(&config_path).expect("Load from temp file should succeed");
    assert_eq!(loaded.gameplay.default_track, "kart_arena");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_config_load_invalid_toml_fallback() {
    let temp_dir = std::env::temp_dir().join(format!("tdrace_invalid_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let bad_config_path = temp_dir.join("bad_config.toml");

    let mut file = std::fs::File::create(&bad_config_path).unwrap();
    writeln!(file, "This is not valid TOML syntax {{{{ [[]]").unwrap();

    let res = GameConfig::load_from_path(&bad_config_path);
    assert!(res.is_err());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_default_gameplay_pilot_count_and_toml_override() {
    // 1. Default config must specify 7 bots (yielding an 8-pilot race grid with 1 player)
    let default_cfg = GameConfig::default();
    assert_eq!(
        default_cfg.gameplay.default_num_bots, 7,
        "Default config must configure 7 bots for 8 pilots total"
    );

    let mut session = RaceSession::new_with_config(default_cfg);
    assert_eq!(session.num_bots, 7);
    session.init_race();
    assert_eq!(session.cars.len(), 8, "Must spawn 8 cars (1 player + 7 AI opponents)");
    assert_eq!(session.opponent_drivers.len(), 7);

    // 2. Custom TOML configuring bot count
    let custom_toml = r#"
[gameplay]
default_track = "oval_speedway"
default_car = "drift_car"
default_laps = 4
default_num_bots = 3
default_assist_profile = "sport"
"#;
    let loaded: GameConfig = toml::from_str(custom_toml).expect("Custom gameplay TOML parse");
    assert_eq!(loaded.gameplay.default_num_bots, 3);

    let mut custom_session = RaceSession::new_with_config(loaded);
    assert_eq!(custom_session.num_bots, 3);
    custom_session.init_race();
    assert_eq!(custom_session.cars.len(), 4, "Must spawn 4 cars (1 player + 3 AI opponents)");
}

#[test]
fn test_in_file_module_override_merging() {
    let toml_str = r#"
[audio]
master_volume = 0.85
sfx_volume = 0.90
music_volume = 0.70

[gameplay]
default_track = "classic_grand_prix"
default_laps = 3
default_num_bots = 7

[camera]
trauma_decay = 2.2

# F1 Module overrides
[modules.f1.audio]
master_volume = 0.50

[modules.f1.gameplay]
default_track = "monza"
default_laps = 15
default_assist_profile = "pro"

[modules.f1.camera]
velocity_lookahead_time = 0.65

# Rally Module overrides
[modules.rally.gameplay]
default_track = "oasis_rally"
default_assist_profile = "sport"

[modules.rally.camera]
trauma_decay = 1.2
"#;

    let base_cfg: GameConfig = toml::from_str(toml_str).expect("Parse base config with modules");

    // 1. Classic module (inherits general settings directly)
    let classic_cfg = base_cfg.for_module_table_only("classic");
    assert!((classic_cfg.audio.master_volume - 0.85).abs() < 1e-4);
    assert_eq!(classic_cfg.gameplay.default_track, "classic_grand_prix");
    assert_eq!(classic_cfg.gameplay.default_laps, 3);
    assert_eq!(classic_cfg.gameplay.default_assist_profile, "arcade");
    assert!((classic_cfg.camera.trauma_decay - 2.2).abs() < 1e-4);

    // 2. F1 module (specific overrides prevail, unmentioned fields inherit general)
    let f1_cfg = base_cfg.for_module_table_only("f1");
    assert!((f1_cfg.audio.master_volume - 0.50).abs() < 1e-4, "F1 specific audio override");
    assert!((f1_cfg.audio.sfx_volume - 0.90).abs() < 1e-4, "F1 inherits general sfx volume");
    assert_eq!(f1_cfg.gameplay.default_track, "monza", "F1 specific track override");
    assert_eq!(f1_cfg.gameplay.default_laps, 15, "F1 specific laps override");
    assert_eq!(f1_cfg.gameplay.default_assist_profile, "pro", "F1 specific assist override");
    assert_eq!(f1_cfg.gameplay.default_num_bots, 7, "F1 inherits general bot count");
    assert!((f1_cfg.camera.velocity_lookahead_time - 0.65).abs() < 1e-4, "F1 specific camera lookahead");
    assert!((f1_cfg.camera.trauma_decay - 2.2).abs() < 1e-4, "F1 inherits general trauma decay");

    // 3. Rally module
    let rally_cfg = base_cfg.for_module_table_only("rally");
    assert!((rally_cfg.audio.master_volume - 0.85).abs() < 1e-4, "Rally inherits general master volume");
    assert_eq!(rally_cfg.gameplay.default_track, "oasis_rally");
    assert_eq!(rally_cfg.gameplay.default_assist_profile, "sport");
    assert!((rally_cfg.camera.trauma_decay - 1.2).abs() < 1e-4, "Rally specific trauma decay");
}

#[test]
fn test_deep_merge_toml_partial_overrides() {
    use tdrace_app::config::deep_merge_toml;

    let mut base = toml::from_str::<toml::Value>(r#"
[camera]
position_smoothing = 8.5
trauma_decay = 2.2

[audio]
master_volume = 0.8
"#).unwrap();

    let overrides = toml::from_str::<toml::Value>(r#"
[camera]
trauma_decay = 1.0

[gameplay]
default_laps = 10
"#).unwrap();

    deep_merge_toml(&mut base, &overrides);

    assert_eq!(
        base.get("camera").unwrap().get("position_smoothing").unwrap().as_float().unwrap(),
        8.5
    );
    assert_eq!(
        base.get("camera").unwrap().get("trauma_decay").unwrap().as_float().unwrap(),
        1.0
    );
    assert_eq!(
        base.get("audio").unwrap().get("master_volume").unwrap().as_float().unwrap(),
        0.8
    );
    assert_eq!(
        base.get("gameplay").unwrap().get("default_laps").unwrap().as_integer().unwrap(),
        10
    );
}

#[test]
fn test_session_module_switching_applies_effective_config() {
    let toml_str = r#"
[audio]
master_volume = 0.80

[gameplay]
default_track = "classic_grand_prix"
default_laps = 3
default_num_bots = 7
"#;

    let base_cfg: GameConfig = toml::from_str(toml_str).unwrap();
    let mut session = RaceSession::new_with_config(base_cfg);

    // Initial state (classic)
    assert_eq!(session.active_module_id, "classic");
    assert!((session.audio.settings.master_volume - 0.80).abs() < 1e-4);
    assert_eq!(session.assist_profile, AssistProfile::Arcade);

    // Switch to F1 (loads config.f1.toml overrides)
    session.switch_to_f1();
    assert_eq!(session.active_module_id, "f1");
    assert_eq!(session.total_laps, 5, "F1 specific laps configured in config.f1.toml");
    assert_eq!(session.assist_profile, AssistProfile::Pro, "F1 specific assist configured");
    assert!((session.camera.velocity_lookahead_time - 0.50).abs() < 1e-4, "F1 specific camera lookahead configured");

    // Switch to Rally (loads config.rally.toml overrides)
    session.switch_to_rally();
    assert_eq!(session.active_module_id, "rally");
    assert_eq!(session.assist_profile, AssistProfile::Sport, "Rally specific assist profile configured");
    assert!((session.camera.trauma_decay - 1.8).abs() < 1e-4, "Rally specific trauma decay configured");
    assert!((session.audio.settings.master_volume - 0.80).abs() < 1e-4, "Rally inherits general master volume");

    // Switch back to Classic
    session.switch_to_classic();
    assert_eq!(session.active_module_id, "classic");
    assert!((session.audio.settings.master_volume - 0.80).abs() < 1e-4);
    assert_eq!(session.assist_profile, AssistProfile::Arcade);
}

#[test]
fn test_external_module_files_and_hierarchy_precedence() {
    let base_cfg = GameConfig::default();

    // 1. F1 Module loads config.f1.toml overrides
    let f1_cfg = base_cfg.for_module("f1");
    assert_eq!(f1_cfg.gameplay.default_track, "monza");
    assert_eq!(f1_cfg.gameplay.default_laps, 5);
    assert_eq!(f1_cfg.gameplay.default_assist_profile, "pro");
    assert!((f1_cfg.camera.velocity_lookahead_time - 0.50).abs() < 1e-4);
    assert!((f1_cfg.camera.position_smoothing - 9.5).abs() < 1e-4);

    // 2. Rally Module loads config.rally.toml overrides
    let rally_cfg = base_cfg.for_module("rally");
    assert_eq!(rally_cfg.gameplay.default_track, "oasis_rally");
    assert_eq!(rally_cfg.gameplay.default_assist_profile, "sport");
    assert!((rally_cfg.camera.trauma_decay - 1.8).abs() < 1e-4);
    assert!((rally_cfg.camera.max_shake_offset - 2.0).abs() < 1e-4);

    // 3. Kart Module loads config.kart.toml overrides
    let kart_cfg = base_cfg.for_module("kart");
    assert_eq!(kart_cfg.gameplay.default_track, "kart_arena");
    assert_eq!(kart_cfg.gameplay.default_laps, 6);
    assert!((kart_cfg.input.steer_rise_rate - 8.5).abs() < 1e-4);

    // 4. Precedence: Custom in-file override vs external merge override
    let mut custom_base = GameConfig::default();
    custom_base.gameplay.default_laps = 2; // general
    custom_base.modules.insert(
        "custom_mod".to_string(),
        toml::from_str(r#"
[gameplay]
default_laps = 8
"#).unwrap(),
    );

    let resolved_mod = custom_base.for_module("custom_mod");
    assert_eq!(resolved_mod.gameplay.default_laps, 8, "In-file module table overrides general config");

    let higher_override = toml::from_str(r#"
[gameplay]
default_laps = 20
"#).unwrap();
    let final_cfg = resolved_mod.merge_override(&higher_override);
    assert_eq!(final_cfg.gameplay.default_laps, 20, "Specific higher-level override prevails");
}


