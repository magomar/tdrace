use crate::profile::AssistProfile;
use crate::render::surface_material::SurfaceTextureQuality;
use crate::ui::menu::CarChoice;
use cabinet::input::{DigitalInputConfig, SteeringProfile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tdrace_core::physics::config::PlayerHandling;
use tdrace_core::physics::CarConfig;

// Camera settings moved to race-ui (spec 058); re-exported so `crate::config::` paths keep working.
pub use race_ui::camera::{
    CameraConfig, ZoomLevelConfig, REFERENCE_SCREEN_HEIGHT, REFERENCE_SCREEN_WIDTH,
};

/// Keyboard handling settings (Spec 043): a preset plus five values.
///
/// Settings files from before Spec 043 still load: the old ten fields are ignored and the
/// five values come from the (renamed) preset, e.g. `"agile"` loads as Sharp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "InputConfigRaw")]
pub struct InputConfig {
    /// Active handling preset (Smooth, Balanced, Sharp, Raw, Custom).
    pub steering_profile: SteeringProfile,
    /// Steering Speed: center to full input, in ms.
    pub steer_time_ms: f32,
    /// Steering Authority: full input relative to the front grip limit (0.80-1.40).
    pub steer_authority: f32,
    /// Center Precision: response curve exponent (1.0-1.8).
    pub center_precision: f32,
    /// Pedal Speed: throttle and brake time to full, in ms (0 = instant).
    pub pedal_time_ms: f32,
    /// Traction Help [0, 1].
    pub traction_help: f32,
}

impl InputConfig {
    /// The keyboard filter configuration for these settings.
    /// The profile is re-derived from the values, so a module override of one value shows Custom.
    pub fn to_filter_config(&self) -> DigitalInputConfig {
        let mut cfg = DigitalInputConfig {
            profile: self.steering_profile,
            steer_time_ms: self.steer_time_ms,
            steer_authority: self.steer_authority,
            center_precision: self.center_precision,
            pedal_time_ms: self.pedal_time_ms,
            traction_help: self.traction_help,
        }
        .clamped();
        cfg.profile = cfg.matching_profile();
        cfg
    }

    /// Settings that persist the given keyboard filter configuration.
    pub fn from_filter_config(cfg: &DigitalInputConfig) -> Self {
        Self {
            steering_profile: cfg.profile,
            steer_time_ms: cfg.steer_time_ms,
            steer_authority: cfg.steer_authority,
            center_precision: cfg.center_precision,
            pedal_time_ms: cfg.pedal_time_ms,
            traction_help: cfg.traction_help,
        }
    }
}

/// Resolves the car-side human handling from the independent assist mode and input response.
/// Stored input settings remain unchanged; Arcade scales the stored traction-help preference,
/// while Sport and Pro disable this player aid.
pub fn player_handling_for(mode: AssistProfile, cfg: &DigitalInputConfig) -> PlayerHandling {
    let traction_help = match mode {
        AssistProfile::Arcade => cfg.traction_help.clamp(0.0, 1.0),
        AssistProfile::Sport | AssistProfile::Pro => 0.0,
    };
    let mut handling = PlayerHandling::human(cfg.steer_authority, traction_help);
    handling.low_speed_authority_enabled = true;
    handling
}

impl Default for InputConfig {
    fn default() -> Self {
        Self::from_filter_config(&DigitalInputConfig::default())
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct InputConfigRaw {
    steering_profile: Option<SteeringProfile>,
    steer_time_ms: Option<f32>,
    steer_authority: Option<f32>,
    center_precision: Option<f32>,
    pedal_time_ms: Option<f32>,
    traction_help: Option<f32>,
}

impl From<InputConfigRaw> for InputConfig {
    fn from(raw: InputConfigRaw) -> Self {
        let profile = raw.steering_profile.unwrap_or_default();
        let base = DigitalInputConfig::from_profile(profile);
        Self {
            steering_profile: profile,
            steer_time_ms: raw.steer_time_ms.unwrap_or(base.steer_time_ms),
            steer_authority: raw.steer_authority.unwrap_or(base.steer_authority),
            center_precision: raw.center_precision.unwrap_or(base.center_precision),
            pedal_time_ms: raw.pedal_time_ms.unwrap_or(base.pedal_time_ms),
            traction_help: raw.traction_help.unwrap_or(base.traction_help),
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
    /// Default track choice: "gt_coastal_grand_prix", "stock_tri_oval_speedway", "gt_ridge_ring", "kart_pine_grove".
    pub default_track: String,
    /// Default vehicle choice: "sports_car", "drift_car", "kart", "rally_car".
    pub default_car: String,
    /// Default number of laps for circuit races.
    pub default_laps: u32,
    /// Default number of AI opponents (7 AI bots for an 8-pilot race grid).
    pub default_num_bots: usize,
    /// Default driver assist profile: "arcade", "sport", "pro".
    pub default_assist_profile: String,
    /// Default cockpit telemetry HUD mode: "kinematic_damage", "dynamic_telemetry" (Spec 079).
    pub default_cockpit_telemetry_mode: String,
    /// Whether vehicle collision, bottom-out, and jump damage is enabled.
    pub car_damage: bool,
    /// When true, unlocks all circuits and vehicles for testing.
    pub dev_mode: bool,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            default_track: "gt_coastal_grand_prix".to_string(),
            default_car: "sports_car".to_string(),
            default_laps: 3,
            default_num_bots: 7,
            default_assist_profile: "arcade".to_string(),
            default_cockpit_telemetry_mode: "kinematic_damage".to_string(),
            car_damage: false,
            dev_mode: crate::storage::is_dev_mode(),
        }
    }
}

/// Display, window resolution, post-processing, and CRT scanline visual options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Desired window/render width in pixels.
    pub window_width: u32,
    /// Desired window/render height in pixels.
    pub window_height: u32,
    /// Fullscreen mode toggle.
    pub fullscreen: bool,
    /// UI scaling preference: "auto", "compact", "standard", or "large".
    pub ui_scale: String,
    /// Scanline intensity mode: "disabled", "subtle", "arcade_crt", or "retro_glow".
    pub scanline_mode: String,
    /// Vignette edge-darkening intensity (0.0 to 1.0).
    pub vignette_intensity: f32,
    /// Distance in pixels between scanlines.
    pub line_spacing: f32,
    /// Custom opacity override if specified.
    pub custom_opacity: Option<f32>,
    /// Whether realistic vehicle ground shadows are rendered under vehicles.
    pub vehicle_shadows: bool,
    /// Graphics quality tier for surface textures and terrain material shaders.
    pub surface_texture_quality: SurfaceTextureQuality,
    /// In-race floating bot nameplates enabled toggle.
    pub bot_nameplates: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            window_width: 1920,
            window_height: 1080,
            fullscreen: false,
            ui_scale: "auto".to_string(),
            scanline_mode: "disabled".to_string(),
            vignette_intensity: 0.0,
            line_spacing: 3.0,
            custom_opacity: None,
            vehicle_shadows: true,
            surface_texture_quality: SurfaceTextureQuality::High,
            bot_nameplates: true,
        }
    }
}

impl DisplayConfig {
    pub fn to_scanline_mode(&self) -> cabinet::fx::ScanlineMode {
        match self.scanline_mode.to_lowercase().as_str() {
            "subtle" => cabinet::fx::ScanlineMode::Subtle,
            "arcade_crt" | "arcade" | "crt" => cabinet::fx::ScanlineMode::ArcadeCrt,
            "retro_glow" | "retro" | "glow" => cabinet::fx::ScanlineMode::RetroGlow,
            _ => cabinet::fx::ScanlineMode::Disabled,
        }
    }

    pub fn to_crt_overlay(&self) -> cabinet::fx::CrtOverlay {
        let mode = self.to_scanline_mode();
        let mut overlay = cabinet::fx::CrtOverlay::with_mode(mode);
        overlay.config.vignette_intensity = if mode == cabinet::fx::ScanlineMode::Disabled {
            0.0
        } else if self.vignette_intensity > 0.0 {
            self.vignette_intensity
        } else {
            0.25
        };
        overlay.config.line_spacing = self.line_spacing;
        overlay.config.custom_opacity = self.custom_opacity;
        overlay
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

/// Configuration for player car locator helpers and visual aids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerHelpersConfig {
    /// Inverted overhead triangle chevron marker above the player car.
    pub overhead_chevron: bool,
    /// Size scale multiplier for the overhead chevron (0.5 to 2.0).
    pub overhead_chevron_scale: f32,
    /// Brightness multiplier for the overhead chevron (0.2 to 2.5).
    pub overhead_chevron_brightness: f32,
    /// Soft ground aura / glow disc projected under the player car.
    pub ground_aura: bool,
    /// Radius ratio multiplier for the ground aura disc (0.4 to 1.8).
    pub ground_aura_radius_ratio: f32,
    /// Brightness multiplier for the ground aura disc (0.2 to 2.5).
    pub ground_aura_brightness: f32,
    /// High-visibility flashing emergency roof beacon strobe.
    pub roof_beacon: bool,
    /// Brightness multiplier for the roof beacon (0.2 to 2.5).
    pub roof_beacon_brightness: f32,
    /// Ground-projected braking ribbon / curve warning chevrons.
    pub curve_helper: bool,
    /// Size scale multiplier for the curve indicator / braking ribbon (0.5 to 2.0).
    pub curve_helper_scale: f32,
    /// Brightness multiplier for the curve indicator / braking ribbon (0.2 to 2.5).
    pub curve_helper_brightness: f32,
    /// Color scheme for the curve helper: "traffic" (green/yellow/red) or "themed" (accent color).
    pub curve_color_scheme: String,
    /// Curve helper look: "chevrons" (severity arrows) or "pacenote" (rally icon of the curve's shape).
    pub curve_indicator_style: String,
    /// Distance and speed adaptive visibility scaling when zooming out or travelling fast.
    pub adaptive_visibility: bool,
    /// Expanding radar / sonar ping shockwaves on camera zoom changes and spin-outs.
    pub radar_sonar_ping: bool,
    /// Floating bot nameplates toggle.
    pub bot_nameplates: bool,
}

impl Default for PlayerHelpersConfig {
    fn default() -> Self {
        Self {
            overhead_chevron: true,
            overhead_chevron_scale: 1.0,
            overhead_chevron_brightness: 1.0,
            ground_aura: true,
            ground_aura_radius_ratio: 1.0,
            ground_aura_brightness: 1.0,
            roof_beacon: false,
            roof_beacon_brightness: 1.0,
            curve_helper: true,
            curve_helper_scale: 1.0,
            curve_helper_brightness: 1.0,
            curve_color_scheme: "traffic".to_string(),
            curve_indicator_style: "pacenote".to_string(),
            adaptive_visibility: true,
            radar_sonar_ping: true,
            bot_nameplates: true,
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
    pub display: DisplayConfig,
    #[serde(default)]
    pub player_helpers: PlayerHelpersConfig,
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
            display: DisplayConfig::default(),
            player_helpers: PlayerHelpersConfig::default(),
            cars,
            modules: BTreeMap::new(),
        }
    }
}

impl GameConfig {
    /// Locates the git-tracked default `config.toml` file in the repository or installation directory.
    pub fn resolve_default_config_path() -> Option<PathBuf> {
        if let Ok(val) = std::env::var("TDRACE_DEFAULT_CONFIG_PATH") {
            if !val.trim().is_empty() {
                let p = PathBuf::from(val);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        let candidates = [
            PathBuf::from("config.toml"),
            PathBuf::from("../config.toml"),
            PathBuf::from("../../config.toml"),
        ];
        for c in &candidates {
            if c.is_file() {
                return Some(c.clone());
            }
        }
        None
    }

    /// Ensures that an installed copy of `config.toml` exists in the user configuration directory.
    ///
    /// If `<user_config_dir>/config.toml` does not exist:
    /// - Copies the default `config.toml` from the repository / installation package.
    /// - If no template file exists, serializes and writes `GameConfig::default()`.
    pub fn ensure_user_config_installed() -> Result<PathBuf, String> {
        let user_path = crate::storage::resolve_user_config_path();
        if user_path.is_file() {
            return Ok(user_path);
        }

        if let Some(parent) = user_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Some(default_path) = Self::resolve_default_config_path() {
            match std::fs::copy(&default_path, &user_path) {
                Ok(_) => {
                    println!(
                        "[Config] Installed default config from {:?} to {:?}",
                        default_path, user_path
                    );
                    return Ok(user_path);
                }
                Err(e) => {
                    eprintln!(
                        "[Config] Warning: Failed to copy default config from {:?} to {:?}: {}. Falling back to default serialization.",
                        default_path, user_path, e
                    );
                }
            }
        }

        let default_config = Self::default();
        default_config.save_to_path(&user_path)?;
        println!(
            "[Config] Installed generated default config to {:?}",
            user_path
        );
        Ok(user_path)
    }

    /// Candidate search paths in priority order for loading `config.toml`.
    ///
    /// Priority order:
    /// 1. User installed config (`<user_config_dir>/config.toml`)
    /// 2. Git-tracked default templates (read-only fallback)
    pub fn candidate_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        // 1. User config path
        paths.push(crate::storage::resolve_user_config_path());
        // 2. Default template paths
        paths.push(PathBuf::from("config.toml"));
        paths.push(PathBuf::from("../config.toml"));
        paths.push(PathBuf::from("../../config.toml"));
        paths
    }

    /// Candidate search paths in priority order for loading a module-specific config file (e.g. `config.gt.toml`).
    pub fn candidate_module_paths(module_id: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        // 1. User config directory ~/.config/tdrace/ (overrides defaults)
        let user_dir = crate::storage::resolve_user_config_dir();
        paths.push(user_dir.join(format!("config.{}.toml", module_id)));
        paths.push(user_dir.join(format!("{}.toml", module_id)));

        // 2. Root / CWD: config.<module>.toml, configs/<module>.toml
        paths.push(PathBuf::from(format!("config.{}.toml", module_id)));
        paths.push(PathBuf::from(format!("configs/{}.toml", module_id)));
        // 3. Workspace root relative paths (when run from crate subdirectories)
        paths.push(PathBuf::from(format!("../../config.{}.toml", module_id)));
        paths.push(PathBuf::from(format!("../../configs/{}.toml", module_id)));
        paths.push(PathBuf::from(format!("../config.{}.toml", module_id)));
        paths.push(PathBuf::from(format!("../configs/{}.toml", module_id)));
        // 4. modules/<module>/config.toml, modules/<module>.toml
        paths.push(PathBuf::from(format!("modules/{}/config.toml", module_id)));
        paths.push(PathBuf::from(format!("modules/{}.toml", module_id)));

        paths
    }

    /// Loads the configuration from the installed user copy (installing defaults if missing),
    /// falling back to the default template if reading fails.
    pub fn load_or_default() -> Self {
        if let Ok(user_path) = Self::ensure_user_config_installed() {
            if user_path.is_file() {
                match Self::load_from_path(&user_path) {
                    Ok(config) => {
                        println!("[Config] Loaded configuration from {:?}", user_path);
                        return config;
                    }
                    Err(e) => {
                        eprintln!(
                            "[Config] Warning: Failed to load user config at {:?}: {}. Falling back to default template.",
                            user_path, e
                        );
                    }
                }
            }
        }

        // Fallback to git-tracked default template (read-only)
        if let Some(default_path) = Self::resolve_default_config_path() {
            if let Ok(config) = Self::load_from_path(&default_path) {
                println!(
                    "[Config] Loaded default configuration from {:?}",
                    default_path
                );
                return config;
            }
        }

        Self::default()
    }

    /// Saves the configuration to the installed user copy in `<user_config_dir>/config.toml`.
    ///
    /// Never modifies the git-tracked `config.toml` in the project root.
    pub fn save_user_config(&self) -> Result<(), String> {
        let user_path = crate::storage::resolve_user_config_path();
        self.save_to_path(&user_path)
    }

    /// Saves the configuration to the installed user copy.
    ///
    /// Preserved for backwards compatibility, ensuring callers never overwrite project files.
    pub fn save_to_first_existing_or_default(&self) -> Result<(), String> {
        self.save_user_config()
    }

    /// Looks for a module-specific config file from candidate paths and parses it as a TOML Value.
    pub fn load_module_override_file(module_id: &str) -> Option<toml::Value> {
        for path in Self::candidate_module_paths(module_id) {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(val) = toml::from_str::<toml::Value>(&content) {
                        println!(
                            "[Config] Loaded module override for '{}' from {:?}",
                            module_id, path
                        );
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

        let module_val_opt = self.modules.get(module_id);
        if let Some(module_val) = module_val_opt {
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
        let module_val_opt = self.modules.get(module_id);
        if let Some(module_val) = module_val_opt {
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
        // Enforce zero side-effects during tests: Never allow writing to the host user's live config directory
        if crate::storage::is_test_environment() {
            if let Ok(home) = std::env::var("HOME") {
                if !home.trim().is_empty() {
                    let live_user_config = PathBuf::from(home).join(".config").join("tdrace");
                    if path.starts_with(&live_user_config) {
                        return Err(format!(
                            "Safety violation: Test execution attempted to write to live user config directory {:?}",
                            path
                        ));
                    }
                }
            }
        }

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
            CarChoice::GT4Clubsport => "gt4_clubsport",
            CarChoice::GT3Car => "gt3_car",
            CarChoice::GT2Biturbo => "gt2_biturbo",
            CarChoice::GT1Legend => "gt1_legend",
            CarChoice::HypercarPrototype => "hypercar_prototype",
            CarChoice::StockCar => "stock_car",
            CarChoice::SandRail => "sand_rail_buggy",
            CarChoice::CrossCar => "cross_car",
        };

        if let Some(cfg) = self.cars.get(key) {
            *cfg
        } else {
            match choice {
                CarChoice::SportsCar => CarConfig::sports_car(),
                CarChoice::DriftCar => CarConfig::drift_car(),
                CarChoice::Kart => CarConfig::kart(),
                CarChoice::RallyCar => CarConfig::rally_car(),
                CarChoice::GT4Clubsport => {
                    crate::module::gt::GtWorldChallengeModule::car_gt4_clubsport()
                }
                CarChoice::GT3Car => crate::module::gt::GtWorldChallengeModule::car_gt3_evo(),
                CarChoice::GT2Biturbo => {
                    crate::module::gt::GtWorldChallengeModule::car_gt2_biturbo()
                }
                CarChoice::GT1Legend => crate::module::gt::GtWorldChallengeModule::car_gt1_legend(),
                CarChoice::HypercarPrototype => {
                    crate::module::gt::GtWorldChallengeModule::car_hypercar_prototype()
                }
                CarChoice::StockCar => CarConfig::stock_car_ta1(),
                CarChoice::SandRail | CarChoice::CrossCar => CarConfig::sand_rail(),
            }
        }
    }
}
