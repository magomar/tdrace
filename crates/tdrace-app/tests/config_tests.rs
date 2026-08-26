use std::io::Write;
use tdrace_app::config::{GameConfig, ZoomLevelConfig};
use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::CarChoice;
use tdrace_core::physics::CarConfig;

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
