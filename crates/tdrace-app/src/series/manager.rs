use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::format::SeriesDefinition;

/// Default embedded presets compiled into binary to ensure zero-failure fallback
/// even if filesystem is missing, sandboxed, or running in WebAssembly.
pub const EMBEDDED_PRESETS: &[(&str, &str)] = &[
    // GT Championships (Tiers 1-5)
    ("gt4_clubman_sprint", include_str!("../../../../series/gt/gt4_clubman_sprint.toml")),
    ("gt3_european_challenge", include_str!("../../../../series/gt/gt3_european_challenge.toml")),
    ("gt2_power_masters", include_str!("../../../../series/gt/gt2_power_masters.toml")),
    ("gt1_heritage_trophy", include_str!("../../../../series/gt/gt1_heritage_trophy.toml")),
    ("hypercar_world_gp", include_str!("../../../../series/gt/hypercar_world_gp.toml")),
    // NASCAR Championships (Tiers 1-5)
    ("nascar_short_track_series", include_str!("../../../../series/nascar/nascar_short_track_series.toml")),
    ("nascar_intermediate_oval_challenge", include_str!("../../../../series/nascar/nascar_intermediate_oval_challenge.toml")),
    ("nascar_national_tour", include_str!("../../../../series/nascar/nascar_national_tour.toml")),
    ("nascar_premier_speedway_trophy", include_str!("../../../../series/nascar/nascar_premier_speedway_trophy.toml")),
    ("nascar_cup_tier5", include_str!("../../../../series/nascar/nascar_cup_tier5.toml")),
    // Rallycross Championships (Tiers 1-5)
    ("rally_grassroots_cup", include_str!("../../../../series/rally/rally_grassroots_cup.toml")),
    ("rally_world_cup", include_str!("../../../../series/rally/rally_world_cup.toml")),
    ("rally_group_b_masters", include_str!("../../../../series/rally/rally_group_b_masters.toml")),
    ("rally_dakar_raid_trophy", include_str!("../../../../series/rally/rally_dakar_raid_trophy.toml")),
    ("rally_super_trucks_series", include_str!("../../../../series/rally/rally_super_trucks_series.toml")),
    // Karting Championships (Tiers 1-5)
    ("kart_world_cup", include_str!("../../../../series/kart/kart_world_cup.toml")),
    ("kart_national_championship", include_str!("../../../../series/kart/kart_national_championship.toml")),
    ("kart_continental_trophy", include_str!("../../../../series/kart/kart_continental_trophy.toml")),
    ("kart_european_championship", include_str!("../../../../series/kart/kart_european_championship.toml")),
    ("kart_superkart_world_series", include_str!("../../../../series/kart/kart_superkart_world_series.toml")),
    // Extreme Off-Road Championships (Tiers 1-5)
    ("extreme_desert_sand_sprint", include_str!("../../../../series/extreme_offroad/extreme_desert_sand_sprint.toml")),
    ("extreme_canyon_raid", include_str!("../../../../series/extreme_offroad/extreme_canyon_raid.toml")),
    ("extreme_offroad_cup", include_str!("../../../../series/extreme_offroad/extreme_offroad_cup.toml")),
    ("extreme_mud_masters", include_str!("../../../../series/extreme_offroad/extreme_mud_masters.toml")),
    ("extreme_ultimate_championship", include_str!("../../../../series/extreme_offroad/extreme_ultimate_championship.toml")),
];

/// Manages discovery, loading, saving, and cataloging of declarative racing series and championships.
#[derive(Debug, Clone)]
pub struct SeriesManager {
    pub user_dir: PathBuf,
    pub git_dir: Option<PathBuf>,
    pub series: HashMap<String, SeriesDefinition>,
}

pub type ChampionshipManager = SeriesManager;

impl Default for SeriesManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SeriesManager {
    /// Creates a new SeriesManager using standard platform storage directories.
    pub fn new() -> Self {
        let user_dir = crate::storage::resolve_user_series_dir();
        let git_dir = crate::storage::resolve_git_series_dir();
        let mut mgr = Self {
            user_dir,
            git_dir,
            series: HashMap::new(),
        };
        mgr.scan_all();
        mgr
    }

    /// Creates a SeriesManager with explicit directories (useful for sandboxed testing).
    pub fn with_dirs(user_dir: PathBuf, git_dir: Option<PathBuf>) -> Self {
        let mut mgr = Self {
            user_dir,
            git_dir,
            series: HashMap::new(),
        };
        mgr.scan_all();
        mgr
    }

    /// Rescans all series from embedded presets, git repo directory, and user storage.
    pub fn scan_all(&mut self) {
        self.series.clear();

        // 1. Load embedded presets
        for &(id, content) in EMBEDDED_PRESETS {
            if let Ok(def) = SeriesDefinition::from_toml(content) {
                self.series.insert(id.to_string(), def);
            }
        }

        // 2. Scan repository presets (if available in dev mode)
        if let Some(git_dir) = self.git_dir.clone() {
            if git_dir.is_dir() {
                self.scan_directory_recursive(&git_dir);
            }
        }

        // 3. Scan user custom series
        let user_dir = self.user_dir.clone();
        if user_dir.is_dir() {
            self.scan_directory_recursive(&user_dir);
        }
    }

    fn scan_directory_recursive(&mut self, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory_recursive(&path);
                } else if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(def) = SeriesDefinition::from_toml(&content) {
                            self.series.insert(def.series.id.clone(), def);
                        }
                    }
                }
            }
        }
    }

    /// Retrieves a series definition by its unique identifier slug.
    pub fn get(&self, id: &str) -> Option<&SeriesDefinition> {
        self.series.get(id)
    }

    /// Retrieves all series matching a motorsport module (e.g. "gt", "nascar").
    pub fn get_by_module(&self, module_id: &str) -> Vec<&SeriesDefinition> {
        self.series
            .values()
            .filter(|c| c.series.module_id.eq_ignore_ascii_case(module_id))
            .collect()
    }

    /// Retrieves a series matching a module and specific career tier (1..=5).
    pub fn get_by_module_and_tier(&self, module_id: &str, tier: u32) -> Option<&SeriesDefinition> {
        self.series
            .values()
            .find(|c| c.series.module_id.eq_ignore_ascii_case(module_id) && c.series.tier == tier)
    }

    /// Returns a list of all known series definitions sorted by module, tier, and name.
    pub fn all_sorted(&self) -> Vec<&SeriesDefinition> {
        let mut list: Vec<&SeriesDefinition> = self.series.values().collect();
        list.sort_by(|a, b| {
            a.series
                .module_id
                .cmp(&b.series.module_id)
                .then_with(|| a.series.tier.cmp(&b.series.tier))
                .then_with(|| a.series.name.cmp(&b.series.name))
        });
        list
    }

    /// Saves a series definition to the user's custom series folder.
    pub fn save_user_series(&mut self, def: &SeriesDefinition) -> Result<PathBuf, std::io::Error> {
        if let Err(errs) = def.validate() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                errs.join("; "),
            ));
        }

        let _ = fs::create_dir_all(&self.user_dir);
        let filename = format!("{}.toml", def.series.id);
        let target_path = self.user_dir.join(filename);

        let toml_str = def
            .to_toml()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&target_path, toml_str)?;
        self.series.insert(def.series.id.clone(), def.clone());

        Ok(target_path)
    }

    /// Backwards compatibility alias for `save_user_series`.
    pub fn save_user_championship(&mut self, def: &SeriesDefinition) -> Result<PathBuf, std::io::Error> {
        self.save_user_series(def)
    }

    /// Saves a series definition directly into the repository presets (dev mode only).
    pub fn save_preset_series(&mut self, def: &SeriesDefinition) -> Result<PathBuf, std::io::Error> {
        if let Err(errs) = def.validate() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                errs.join("; "),
            ));
        }

        let git_dir = self.git_dir.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Git series directory not available or not running in dev mode",
            )
        })?;

        let module_dir = git_dir.join(&def.series.module_id);
        let _ = fs::create_dir_all(&module_dir);
        let filename = format!("{}.toml", def.series.id);
        let target_path = module_dir.join(filename);

        let toml_str = def
            .to_toml()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&target_path, toml_str)?;
        self.series.insert(def.series.id.clone(), def.clone());

        Ok(target_path)
    }

    /// Backwards compatibility alias for `save_preset_series`.
    pub fn save_preset_championship(&mut self, def: &SeriesDefinition) -> Result<PathBuf, std::io::Error> {
        self.save_preset_series(def)
    }

    /// Deletes a custom series file from user storage.
    pub fn delete_user_series(&mut self, id: &str) -> Result<bool, std::io::Error> {
        let filename = format!("{}.toml", id);
        let target_path = self.user_dir.join(filename);
        if target_path.exists() {
            fs::remove_file(target_path)?;
            self.series.remove(id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Backwards compatibility alias for `delete_user_series`.
    pub fn delete_user_championship(&mut self, id: &str) -> Result<bool, std::io::Error> {
        self.delete_user_series(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::format::{DriverConfig, RoundConfig, ScoringConfig, SeriesMeta};

    #[test]
    fn test_manager_loads_embedded_presets() {
        let mgr = SeriesManager::new();
        assert!(mgr.get("gt4_clubman_sprint").is_some());
        assert!(mgr.get("nascar_cup_tier5").is_some());
        assert!(mgr.get("rally_world_cup").is_some());
        assert!(mgr.get("kart_world_cup").is_some());
        assert!(mgr.get("extreme_offroad_cup").is_some());

        let gt_cups = mgr.get_by_module("gt");
        assert!(!gt_cups.is_empty());
        assert_eq!(gt_cups[0].series.module_id, "gt");

        let tier1_gt = mgr.get_by_module_and_tier("gt", 1);
        assert!(tier1_gt.is_some());
        assert_eq!(tier1_gt.unwrap().series.id, "gt4_clubman_sprint");
    }

    #[test]
    fn test_manager_save_and_delete_user_championship() {
        let temp_dir = std::env::temp_dir().join("tdrace_series_mgr_test");
        let _ = fs::create_dir_all(&temp_dir);

        let mut mgr = SeriesManager::with_dirs(temp_dir.clone(), None);

        let test_def = SeriesDefinition {
            series: SeriesMeta {
                id: "test_custom_sprint".to_string(),
                name: "Test Custom Sprint".to_string(),
                description: "Sandbox test".to_string(),
                module_id: "classic".to_string(),
                tier: 1,
                laps_per_round: 3,
                bot_count: Some(1),
                ai_difficulty: None,
                icon: None,
            },
            scoring: ScoringConfig::default(),
            rounds: vec![RoundConfig {
                order: 1,
                track_id: "classic_grand_prix".to_string(),
                name: None,
                laps: None,
                weather: None,
            }],
            drivers: vec![
                DriverConfig {
                    id: "player".to_string(),
                    name: "Player".to_string(),
                    team: "Apex".to_string(),
                    is_player: true,
                    car_model_id: None,
                    country: None,
                    ai_character: None,
                    ai_style: None,
                    ai_tier: None,
                    livery_idx: None,
                },
                DriverConfig {
                    id: "rival".to_string(),
                    name: "Rival".to_string(),
                    team: "Rivalry".to_string(),
                    is_player: false,
                    car_model_id: None,
                    country: None,
                    ai_character: None,
                    ai_style: None,
                    ai_tier: None,
                    livery_idx: None,
                },
            ],
        };

        // Save
        let saved_path = mgr.save_user_championship(&test_def).expect("Save must succeed");
        assert!(saved_path.exists());
        assert!(mgr.get("test_custom_sprint").is_some());

        // Rescan
        mgr.scan_all();
        assert!(mgr.get("test_custom_sprint").is_some());

        // Delete
        let deleted = mgr.delete_user_championship("test_custom_sprint").expect("Delete must succeed");
        assert!(deleted);
        assert!(!saved_path.exists());
        assert!(mgr.get("test_custom_sprint").is_none());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
