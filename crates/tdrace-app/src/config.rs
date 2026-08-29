use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tdrace_core::physics::CarConfig;
use crate::ui::menu::CarChoice;

/// Configuration for a specific camera zoom level or mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoomLevelConfig {
    /// Friendly label displayed in HUD popups (e.g. "Close", "Medium", "Far", "Overview").
    pub name: String,
    /// Mode type: "follow" (dynamic speed follow) or "overview" (static full-circuit view).
    pub mode: String,
    /// Pixels per meter at high speed (for follow mode) or base scale (for overview).
    pub min_zoom: f32,
    /// Pixels per meter at zero speed / stationary (for follow mode).
    pub max_zoom: f32,
}

impl Default for ZoomLevelConfig {
    fn default() -> Self {
        Self {
            name: "Medium".to_string(),
            mode: "follow".to_string(),
            min_zoom: 10.0,
            max_zoom: 16.5,
        }
    }
}

impl ZoomLevelConfig {
    pub fn is_overview(&self) -> bool {
        self.mode.eq_ignore_ascii_case("overview")
    }
}

/// Global camera configuration and list of selectable zoom levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    /// Smooth follow position interpolation speed.
    pub position_smoothing: f32,
    /// Smooth follow zoom interpolation speed.
    pub zoom_smoothing: f32,
    /// Velocity lookahead projection time in seconds.
    pub velocity_lookahead_time: f32,
    /// Screen shake trauma decay rate per second.
    pub trauma_decay: f32,
    /// Maximum screen shake pixel displacement.
    pub max_shake_offset: f32,
    /// Initial active zoom level index in the `levels` list.
    pub default_level_index: usize,
    /// Ordered list of zoom levels cycled through during gameplay.
    pub levels: Vec<ZoomLevelConfig>,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            position_smoothing: 8.5,
            zoom_smoothing: 4.0,
            velocity_lookahead_time: 0.40,
            trauma_decay: 2.2,
            max_shake_offset: 1.5,
            default_level_index: 0,
            levels: vec![
                ZoomLevelConfig {
                    name: "Close".to_string(),
                    mode: "follow".to_string(),
                    min_zoom: 13.5,
                    max_zoom: 22.0,
                },
                ZoomLevelConfig {
                    name: "Medium".to_string(),
                    mode: "follow".to_string(),
                    min_zoom: 10.0,
                    max_zoom: 16.5,
                },
                ZoomLevelConfig {
                    name: "Far".to_string(),
                    mode: "follow".to_string(),
                    min_zoom: 7.0,
                    max_zoom: 11.5,
                },
                ZoomLevelConfig {
                    name: "Overview".to_string(),
                    mode: "overview".to_string(),
                    min_zoom: 3.5,
                    max_zoom: 3.5,
                },
            ],
        }
    }
}

/// Digital keyboard steering and throttle input filter settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    /// Steering rise rate in units/second.
    pub steer_rise_rate: f32,
    /// Steering return to center rate in units/second.
    pub steer_return_rate: f32,
    /// Non-linear steering exponent (e.g. 1.35 for fine micro-corrections near center).
    pub steer_exponent: f32,
    /// Speed-sensitive steering attenuation factor.
    pub speed_sensitive_factor: f32,
    /// Minimum steering lock allowed at maximum vehicle speed.
    pub min_speed_steer_limit: f32,
    /// Throttle rise rate in units/second.
    pub throttle_rise_rate: f32,
    /// Brake rise rate in units/second.
    pub brake_rise_rate: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            steer_rise_rate: 6.0,
            steer_return_rate: 10.0,
            steer_exponent: 1.35,
            speed_sensitive_factor: 0.018,
            min_speed_steer_limit: 0.38,
            throttle_rise_rate: 10.0,
            brake_rise_rate: 14.0,
        }
    }
}

/// Sound and music master/channel volume levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    /// Master volume [0.0, 1.0].
    pub master_volume: f32,
    /// Sound effects (SFX) volume [0.0, 1.0].
    pub sfx_volume: f32,
    /// Synthwave music volume [0.0, 1.0].
    pub music_volume: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.85,
            sfx_volume: 0.90,
            music_volume: 0.70,
        }
    }
}

/// Default session and gameplay parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameplayConfig {
    /// Default track choice: "classic_grand_prix", "oval_speedway", "drift_park", "kart_arena".
    pub default_track: String,
    /// Default vehicle choice: "sports_car", "drift_car", "kart", "rally_car".
    pub default_car: String,
    /// Default number of laps for circuit races.
    pub default_laps: u32,
    /// Default number of AI opponents (7 AI bots for an 8-pilot race grid).
    pub default_num_bots: usize,
    /// Default driver assist profile: "arcade", "sport", "pro".
    pub default_assist_profile: String,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            default_track: "classic_grand_prix".to_string(),
            default_car: "sports_car".to_string(),
            default_laps: 3,
            default_num_bots: 7,
            default_assist_profile: "arcade".to_string(),
        }
    }
}

/// Recursively merges `overrides` into `base`.
/// For tables, keys present in `overrides` are merged into `base` (existing sub-tables are recursively merged).
/// For all other values, `overrides` replaces `base`.
pub fn deep_merge_toml(base: &mut toml::Value, overrides: &toml::Value) {
    match (base, overrides) {
        (toml::Value::Table(base_map), toml::Value::Table(override_map)) => {
            for (k, v) in override_map {
                if let Some(existing) = base_map.get_mut(k) {
                    deep_merge_toml(existing, v);
                } else {
                    base_map.insert(k.clone(), v.clone());
                }
            }
        }
        (base_slot, override_val) => {
            *base_slot = override_val.clone();
        }
    }
}

/// Root application and game configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameConfig {
    #[serde(default)]
    pub camera: CameraConfig,
    #[serde(default)]
    pub input: InputConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub gameplay: GameplayConfig,
    #[serde(default)]
    pub cars: BTreeMap<String, CarConfig>,
    #[serde(default)]
    pub modules: BTreeMap<String, toml::Value>,
}

impl Default for GameConfig {
    fn default() -> Self {
        let mut cars = BTreeMap::new();
        cars.insert("sports_car".to_string(), CarConfig::sports_car());
        cars.insert("drift_car".to_string(), CarConfig::drift_car());
        cars.insert("kart".to_string(), CarConfig::kart());
        cars.insert("rally_car".to_string(), CarConfig::rally_car());

        Self {
            camera: CameraConfig::default(),
            input: InputConfig::default(),
            audio: AudioConfig::default(),
            gameplay: GameplayConfig::default(),
            cars,
            modules: BTreeMap::new(),
        }
    }
}

impl GameConfig {
    /// Candidate search paths in priority order for loading `config.toml`.
    pub fn candidate_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        // 1. Current working directory
        paths.push(PathBuf::from("config.toml"));
        // 2. Workspace root relative paths (when run from inside crate subdirectories)
        paths.push(PathBuf::from("../../config.toml"));
        paths.push(PathBuf::from("../config.toml"));
        // 3. User config ~/.config/tdrace/config.toml
        if let Some(home) = std::env::var_os("HOME") {
            let mut p = PathBuf::from(home);
            p.push(".config");
            p.push("tdrace");
            p.push("config.toml");
            paths.push(p);
        }
        paths
    }

    /// Candidate search paths in priority order for loading a module-specific config file (e.g. `config.f1.toml`).
    pub fn candidate_module_paths(module_id: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        // 1. Root / CWD: config.<module>.toml, configs/<module>.toml
        paths.push(PathBuf::from(format!("config.{}.toml", module_id)));
        paths.push(PathBuf::from(format!("configs/{}.toml", module_id)));
        // 2. Workspace root relative paths (when run from crate subdirectories)
        paths.push(PathBuf::from(format!("../../config.{}.toml", module_id)));
        paths.push(PathBuf::from(format!("../../configs/{}.toml", module_id)));
        paths.push(PathBuf::from(format!("../config.{}.toml", module_id)));
        paths.push(PathBuf::from(format!("../configs/{}.toml", module_id)));
        // 3. modules/<module>/config.toml, modules/<module>.toml
        paths.push(PathBuf::from(format!("modules/{}/config.toml", module_id)));
        paths.push(PathBuf::from(format!("modules/{}.toml", module_id)));
        // 4. User config directory ~/.config/tdrace/
        if let Some(home) = std::env::var_os("HOME") {
            let mut p1 = PathBuf::from(&home);
            p1.push(".config");
            p1.push("tdrace");
            p1.push(format!("config.{}.toml", module_id));
            paths.push(p1);

            let mut p2 = PathBuf::from(home);
            p2.push(".config");
            p2.push("tdrace");
            p2.push(format!("{}.toml", module_id));
            paths.push(p2);
        }
        paths
    }

    /// Loads the configuration from the first existing candidate file, or creates default.
    pub fn load_or_default() -> Self {
        for path in Self::candidate_paths() {
            if path.exists() {
                if let Ok(config) = Self::load_from_path(&path) {
                    println!("[Config] Loaded configuration from {:?}", path);
                    return config;
                }
            }
        }

        let default_config = Self::default();
        // Attempt to persist default config to ./config.toml for easy user editing
        let _ = default_config.save_to_path(Path::new("config.toml"));
        default_config
    }

    /// Looks for a module-specific config file from candidate paths and parses it as a TOML Value.
    pub fn load_module_override_file(module_id: &str) -> Option<toml::Value> {
        for path in Self::candidate_module_paths(module_id) {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(val) = toml::from_str::<toml::Value>(&content) {
                        println!("[Config] Loaded module override for '{}' from {:?}", module_id, path);
                        return Some(val);
                    }
                }
            }
        }
        None
    }

    /// Returns the effective `GameConfig` for a specific module by merging `self` (general config)
    /// with only the in-file `[modules.<module_id>]` table, without filesystem lookups.
    pub fn for_module_table_only(&self, module_id: &str) -> Self {
        let mut base_value = match toml::Value::try_from(self) {
            Ok(v) => v,
            Err(_) => return self.clone(),
        };

        if let Some(module_val) = self.modules.get(module_id) {
            deep_merge_toml(&mut base_value, module_val);
        }

        match base_value.try_into::<GameConfig>() {
            Ok(mut resolved) => {
                resolved.modules = self.modules.clone();
                resolved
            }
            Err(e) => {
                eprintln!("[Config] Warning: Failed to parse in-file module config for '{}': {}. Falling back to base config.", module_id, e);
                self.clone()
            }
        }
    }

    /// Returns the effective `GameConfig` for a specific module by merging `self` (general config)
    /// with any in-file `[modules.<module_id>]` table, and then with any external module file
    /// (`config.<module_id>.toml`, `configs/<module_id>.toml`, etc.). Specific values prevail over general values.
    pub fn for_module(&self, module_id: &str) -> Self {
        // 1. Serialize base config to toml::Value
        let mut base_value = match toml::Value::try_from(self) {
            Ok(v) => v,
            Err(_) => return self.clone(),
        };

        // 2. Apply in-file [modules.<module_id>] override if present
        if let Some(module_val) = self.modules.get(module_id) {
            deep_merge_toml(&mut base_value, module_val);
        }

        // 3. Apply external module file override if present (highest priority)
        if let Some(file_val) = Self::load_module_override_file(module_id) {
            deep_merge_toml(&mut base_value, &file_val);
        }

        // 4. Deserialize back to GameConfig
        match base_value.try_into::<GameConfig>() {
            Ok(mut resolved) => {
                resolved.modules = self.modules.clone();
                resolved
            }
            Err(e) => {
                eprintln!("[Config] Warning: Failed to parse merged module config for '{}': {}. Falling back to base config.", module_id, e);
                self.clone()
            }
        }
    }

    /// Merges an arbitrary TOML Value override into this config and returns the resulting GameConfig.
    pub fn merge_override(&self, override_val: &toml::Value) -> Self {
        let mut base_value = match toml::Value::try_from(self) {
            Ok(v) => v,
            Err(_) => return self.clone(),
        };
        deep_merge_toml(&mut base_value, override_val);
        match base_value.try_into::<GameConfig>() {
            Ok(mut resolved) => {
                resolved.modules = self.modules.clone();
                resolved
            }
            Err(e) => {
                eprintln!("[Config] Warning: Failed to merge config override: {}", e);
                self.clone()
            }
        }
    }

    /// Loads and parses configuration from a specific file path.
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config at {:?}: {}", path, e))?;
        toml::from_str::<GameConfig>(&content)
            .map_err(|e| format!("Failed to parse TOML config at {:?}: {}", path, e))
    }

    /// Serializes and writes configuration to a given file path.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config to TOML: {}", e))?;
        std::fs::write(path, toml_str)
            .map_err(|e| format!("Failed to write config file to {:?}: {}", path, e))?;
        Ok(())
    }

    /// Retrieves the vehicle physics specification for a given `CarChoice`,
    /// falling back to the built-in preset if not explicitly customized in TOML.
    pub fn get_car_config(&self, choice: CarChoice) -> CarConfig {
        let key = match choice {
            CarChoice::SportsCar => "sports_car",
            CarChoice::DriftCar => "drift_car",
            CarChoice::Kart => "kart",
            CarChoice::RallyCar => "rally_car",
        };

        if let Some(cfg) = self.cars.get(key) {
            *cfg
        } else {
            match choice {
                CarChoice::SportsCar => CarConfig::sports_car(),
                CarChoice::DriftCar => CarConfig::drift_car(),
                CarChoice::Kart => CarConfig::kart(),
                CarChoice::RallyCar => CarConfig::rally_car(),
            }
        }
    }
}
