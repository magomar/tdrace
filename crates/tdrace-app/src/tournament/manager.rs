use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::format::ChampionshipDefinition;

/// Default embedded presets compiled into binary to ensure zero-failure fallback
/// even if filesystem is missing, sandboxed, or running in WebAssembly.
pub const EMBEDDED_PRESETS: &[(&str, &str)] = &[
    ("gt4_clubman_sprint", include_str!("../../../../championships/gt/gt4_clubman_sprint.toml")),
    ("nascar_cup_tier5", include_str!("../../../../championships/nascar/nascar_cup_tier5.toml")),
    ("rally_world_cup", include_str!("../../../../championships/rally/rally_world_cup.toml")),
    ("kart_world_cup", include_str!("../../../../championships/kart/kart_world_cup.toml")),
    ("extreme_offroad_cup", include_str!("../../../../championships/extreme_offroad/extreme_offroad_cup.toml")),
];

/// Manages discovery, loading, saving, and cataloging of declarative championships.
#[derive(Debug, Clone)]
pub struct ChampionshipManager {
    pub user_dir: PathBuf,
    pub git_dir: Option<PathBuf>,
    pub championships: HashMap<String, ChampionshipDefinition>,
}

impl Default for ChampionshipManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChampionshipManager {
    /// Creates a new ChampionshipManager using standard platform storage directories.
    pub fn new() -> Self {
        let user_dir = crate::storage::resolve_user_championships_dir();
        let git_dir = crate::storage::resolve_git_championships_dir();
        let mut mgr = Self {
            user_dir,
            git_dir,
            championships: HashMap::new(),
        };
        mgr.scan_all();
        mgr
    }

    /// Creates a ChampionshipManager with explicit directories (useful for sandboxed testing).
    pub fn with_dirs(user_dir: PathBuf, git_dir: Option<PathBuf>) -> Self {
        let mut mgr = Self {
            user_dir,
            git_dir,
            championships: HashMap::new(),
        };
        mgr.scan_all();
        mgr
    }

    /// Rescans all championships from embedded presets, git repo directory, and user storage.
    pub fn scan_all(&mut self) {
        self.championships.clear();

        // 1. Load embedded presets
        for &(id, content) in EMBEDDED_PRESETS {
            if let Ok(def) = ChampionshipDefinition::from_toml(content) {
                self.championships.insert(id.to_string(), def);
            }
        }

        // 2. Scan repository presets (if available in dev mode)
        if let Some(git_dir) = self.git_dir.clone() {
            if git_dir.is_dir() {
                self.scan_directory_recursive(&git_dir);
            }
        }

        // 3. Scan user custom championships
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
                        if let Ok(def) = ChampionshipDefinition::from_toml(&content) {
                            self.championships.insert(def.championship.id.clone(), def);
                        }
                    }
                }
            }
        }
    }

    /// Retrieves a championship definition by its unique identifier slug.
    pub fn get(&self, id: &str) -> Option<&ChampionshipDefinition> {
        self.championships.get(id)
    }

    /// Retrieves all championships matching a motorsport module (e.g. "gt", "nascar").
    pub fn get_by_module(&self, module_id: &str) -> Vec<&ChampionshipDefinition> {
        self.championships
            .values()
            .filter(|c| c.championship.module_id.eq_ignore_ascii_case(module_id))
            .collect()
    }

    /// Retrieves a championship matching a module and specific career tier (1..=5).
    pub fn get_by_module_and_tier(&self, module_id: &str, tier: u32) -> Option<&ChampionshipDefinition> {
        self.championships
            .values()
            .find(|c| c.championship.module_id.eq_ignore_ascii_case(module_id) && c.championship.tier == tier)
    }

    /// Returns a list of all known championship definitions sorted by module, tier, and name.
    pub fn all_sorted(&self) -> Vec<&ChampionshipDefinition> {
        let mut list: Vec<&ChampionshipDefinition> = self.championships.values().collect();
        list.sort_by(|a, b| {
            a.championship
                .module_id
                .cmp(&b.championship.module_id)
                .then_with(|| a.championship.tier.cmp(&b.championship.tier))
                .then_with(|| a.championship.name.cmp(&b.championship.name))
        });
        list
    }

    /// Saves a championship definition to the user's custom championships folder.
    pub fn save_user_championship(&mut self, def: &ChampionshipDefinition) -> Result<PathBuf, std::io::Error> {
        if let Err(errs) = def.validate() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                errs.join("; "),
            ));
        }

        let _ = fs::create_dir_all(&self.user_dir);
        let filename = format!("{}.toml", def.championship.id);
        let target_path = self.user_dir.join(filename);

        let toml_str = def
            .to_toml()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&target_path, toml_str)?;
        self.championships.insert(def.championship.id.clone(), def.clone());

        Ok(target_path)
    }

    /// Saves a championship definition directly into the repository presets (dev mode only).
    pub fn save_preset_championship(&mut self, def: &ChampionshipDefinition) -> Result<PathBuf, std::io::Error> {
        if let Err(errs) = def.validate() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                errs.join("; "),
            ));
        }

        let git_dir = self.git_dir.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Git championships directory not available or not running in dev mode",
            )
        })?;

        let module_dir = git_dir.join(&def.championship.module_id);
        let _ = fs::create_dir_all(&module_dir);
        let filename = format!("{}.toml", def.championship.id);
        let target_path = module_dir.join(filename);

        let toml_str = def
            .to_toml()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&target_path, toml_str)?;
        self.championships.insert(def.championship.id.clone(), def.clone());

        Ok(target_path)
    }

    /// Deletes a custom championship file from user storage.
    pub fn delete_user_championship(&mut self, id: &str) -> Result<bool, std::io::Error> {
        let filename = format!("{}.toml", id);
        let target_path = self.user_dir.join(filename);
        if target_path.exists() {
            fs::remove_file(target_path)?;
            self.championships.remove(id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tournament::format::{ChampionshipMeta, DriverConfig, RoundConfig, ScoringConfig};

    #[test]
    fn test_manager_loads_embedded_presets() {
        let mgr = ChampionshipManager::new();
        assert!(mgr.get("gt4_clubman_sprint").is_some());
        assert!(mgr.get("nascar_cup_tier5").is_some());
        assert!(mgr.get("rally_world_cup").is_some());
        assert!(mgr.get("kart_world_cup").is_some());
        assert!(mgr.get("extreme_offroad_cup").is_some());

        let gt_cups = mgr.get_by_module("gt");
        assert!(!gt_cups.is_empty());
        assert_eq!(gt_cups[0].championship.module_id, "gt");

        let tier1_gt = mgr.get_by_module_and_tier("gt", 1);
        assert!(tier1_gt.is_some());
        assert_eq!(tier1_gt.unwrap().championship.id, "gt4_clubman_sprint");
    }

    #[test]
    fn test_manager_save_and_delete_user_championship() {
        let temp_dir = std::env::temp_dir().join("tdrace_champ_mgr_test");
        let _ = fs::create_dir_all(&temp_dir);

        let mut mgr = ChampionshipManager::with_dirs(temp_dir.clone(), None);

        let test_def = ChampionshipDefinition {
            championship: ChampionshipMeta {
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
