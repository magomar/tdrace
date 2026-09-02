use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::profile::data::PlayerProfile;

/// Multi-slot profile manager with disk persistence.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProfileManager {
    pub profiles: Vec<PlayerProfile>,
    pub active_index: usize,
}

impl ProfileManager {
    pub fn new() -> Self {
        Self {
            profiles: vec![PlayerProfile::default()],
            active_index: 0,
        }
    }

    /// Returns a reference to the currently active profile.
    pub fn active_profile(&self) -> &PlayerProfile {
        self.profiles.get(self.active_index).unwrap_or(&self.profiles[0])
    }

    /// Returns a mutable reference to the currently active profile.
    pub fn active_profile_mut(&mut self) -> &mut PlayerProfile {
        let idx = self.active_index;
        &mut self.profiles[idx]
    }

    /// Sets active profile by index.
    pub fn select_profile(&mut self, idx: usize) -> bool {
        if idx < self.profiles.len() {
            self.active_index = idx;
            for (i, p) in self.profiles.iter_mut().enumerate() {
                p.is_active = i == idx;
            }
            true
        } else {
            false
        }
    }

    /// Adds a new profile and selects it.
    pub fn add_profile(&mut self, profile: PlayerProfile) -> usize {
        self.profiles.push(profile);
        self.select_profile(self.profiles.len() - 1);
        self.active_index
    }

    /// Saves all profiles to JSON at path.
    pub fn save_to_disk(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Loads profiles from JSON at path, creating defaults if missing.
    pub fn load_from_disk(path: &Path) -> Self {
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(manager) = serde_json::from_str::<Self>(&data) {
                    return manager;
                }
            }
        }
        Self::new()
    }
}
