use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::render::color::CarColorScheme;
use tdrace_core::physics::config::AssistProfile;

pub use cabinet::profile::country::{draw_country_banner, CountryInfo, CountryRegistry};

/// Player Profile representing driver identity, livery customizations, nationality, and driving mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: Option<i64>,
    pub name: String,
    pub alias: String,
    pub country: Option<String>,
    pub color_scheme: CarColorScheme,
    pub is_active: bool,
    pub created_at: String,
    pub last_mode: AssistProfile,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            id: None,
            name: "Racer One".to_string(),
            alias: "Apex Hunter".to_string(),
            country: Some("ESP".to_string()),
            color_scheme: CarColorScheme::from_index(0),
            is_active: true,
            created_at: String::new(),
            last_mode: AssistProfile::Arcade,
        }
    }
}

impl PlayerProfile {
    pub fn new(name: &str, alias: &str, country: Option<&str>, color_scheme: CarColorScheme) -> Self {
        Self {
            id: None,
            name: name.trim().to_string(),
            alias: alias.trim().to_string(),
            country: country.map(|s| s.trim().to_uppercase()),
            color_scheme,
            is_active: false,
            created_at: String::new(),
            last_mode: AssistProfile::Arcade,
        }
    }

    /// Returns display country name or fallback.
    pub fn country_name(&self) -> &str {
        match &self.country {
            Some(code) => CountryRegistry::find_by_code(code)
                .map(|c| c.name)
                .unwrap_or("International"),
            None => "International",
        }
    }

    /// Returns country flag emoji or fallback icon.
    pub fn country_emoji(&self) -> &str {
        match &self.country {
            Some(code) => CountryRegistry::find_by_code(code)
                .map(|c| c.flag_emoji)
                .unwrap_or("🏁"),
            None => "🏁",
        }
    }
}

/// Aggregated career statistics computed from persistent race history logs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProfileCareerStats {
    pub total_races: u32,
    pub wins: u32,
    pub podiums: u32,
    pub total_laps: u32,
    pub win_rate: f32,
    pub podium_rate: f32,
    pub best_times: BTreeMap<String, f32>,
    pub best_circuit_times: BTreeMap<String, f32>,
}

impl ProfileCareerStats {
    pub fn compute(races: &[RaceHistoryEntry]) -> Self {
        let mut total_races = 0u32;
        let mut wins = 0u32;
        let mut podiums = 0u32;
        let mut total_laps = 0u32;
        let mut best_times: BTreeMap<String, f32> = BTreeMap::new();
        let mut best_circuit_times: BTreeMap<String, f32> = BTreeMap::new();

        for race in races {
            total_races += 1;
            total_laps += race.laps;

            if race.position == 1 {
                wins += 1;
            }
            if race.position >= 1 && race.position <= 3 {
                podiums += 1;
            }

            if let Some(lap) = race.best_lap {
                let current_best = best_times.entry(race.track_id.clone()).or_insert(lap);
                if lap < *current_best {
                    *current_best = lap;
                }
            }

            if race.total_time > 0.0 {
                let current_best_total = best_circuit_times
                    .entry(race.track_id.clone())
                    .or_insert(race.total_time);
                if race.total_time < *current_best_total {
                    *current_best_total = race.total_time;
                }
            }
        }

        let win_rate = if total_races > 0 {
            (wins as f32 / total_races as f32) * 100.0
        } else {
            0.0
        };

        let podium_rate = if total_races > 0 {
            (podiums as f32 / total_races as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_races,
            wins,
            podiums,
            total_laps,
            win_rate,
            podium_rate,
            best_times,
            best_circuit_times,
        }
    }
}

/// Individual race completion entry logged in the history database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceHistoryEntry {
    pub id: Option<i64>,
    pub profile_id: i64,
    pub track_id: String,
    pub car_name: String,
    pub position: usize,
    pub total_cars: usize,
    pub total_time: f32,
    pub best_lap: Option<f32>,
    pub laps: u32,
    pub is_time_attack: bool,
    pub created_at: String,
}

/// Persistent career progression record for a specific motorsport module (e.g. "gt", "rally", etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCareerProgress {
    pub profile_id: i64,
    pub module_id: String,
    pub xp: u64,
    pub level: u32,
    pub unlocked_cars: Vec<String>,
    pub unlocked_tracks: Vec<String>,
    pub completed_events: Vec<String>,
    pub trophies_gold: u32,
    pub trophies_silver: u32,
    pub trophies_bronze: u32,
    pub updated_at: String,
}

impl ModuleCareerProgress {
    /// Initial starter career record for Gran Turismo & Endurance GT module.
    pub fn default_for_gt(profile_id: i64) -> Self {
        let mut progress = Self {
            profile_id,
            module_id: "gt".to_string(),
            xp: 0,
            level: 1,
            unlocked_cars: Vec::new(),
            unlocked_tracks: Vec::new(),
            completed_events: Vec::new(),
            trophies_gold: 0,
            trophies_silver: 0,
            trophies_bronze: 0,
            updated_at: String::new(),
        };
        progress.sync_unlocks_for_level();
        progress
    }

    /// XP requirement thresholds for levels 1 through 5.
    pub fn xp_threshold_for_level(level: u32) -> u64 {
        match level {
            1 => 0,
            2 => 1500,
            3 => 3500,
            4 => 6500,
            _ => 10000,
        }
    }

    /// Base XP of the current level.
    pub fn current_level_base_xp(&self) -> u64 {
        Self::xp_threshold_for_level(self.level)
    }

    /// Target XP needed to reach next level, or None if at max level 5.
    pub fn next_level_target_xp(&self) -> Option<u64> {
        if self.level >= 5 {
            None
        } else {
            Some(Self::xp_threshold_for_level(self.level + 1))
        }
    }

    /// Progress ratio [0.0..1.0] towards next level.
    pub fn level_progress_ratio(&self) -> f32 {
        if self.level >= 5 {
            return 1.0;
        }
        let base = self.current_level_base_xp();
        let target = self.next_level_target_xp().unwrap_or(base);
        if target <= base {
            1.0
        } else {
            let cur = (self.xp.saturating_sub(base)) as f32;
            let total = (target - base) as f32;
            (cur / total).clamp(0.0, 1.0)
        }
    }

    /// Awards XP and calculates level up. Returns Some(new_level) if level increased.
    pub fn add_xp(&mut self, amount: u64) -> Option<u32> {
        self.xp = self.xp.saturating_add(amount);
        let old_level = self.level;
        let mut new_level = old_level;

        while new_level < 5 && self.xp >= Self::xp_threshold_for_level(new_level + 1) {
            new_level += 1;
        }

        if new_level > old_level {
            self.level = new_level;
            self.sync_unlocks_for_level();
            Some(new_level)
        } else {
            None
        }
    }

    /// Ensures unlocked cars and tracks match or exceed current level.
    pub fn sync_unlocks_for_level(&mut self) {
        if self.module_id == "gt" {
            // Level 1 Starter
            self.ensure_car("gt4_clubsport");
            self.ensure_track("monza");
            self.ensure_track("red_bull_ring");
            self.ensure_track("nurburgring_gp");

            // Level 2 (FIA GT3)
            if self.level >= 2 {
                self.ensure_car("gt3_evo");
                self.ensure_track("silverstone");
                self.ensure_track("catalunya");
                self.ensure_track("bathurst");
            }

            // Level 3 (SRO GT2)
            if self.level >= 3 {
                self.ensure_car("gt2_biturbo");
                self.ensure_track("spa");
                self.ensure_track("zandvoort");
                self.ensure_track("portimao_gp");
            }

            // Level 4 (90s Le Mans GT1)
            if self.level >= 4 {
                self.ensure_car("gt1_legend");
                self.ensure_track("suzuka");
                self.ensure_track("interlagos");
                self.ensure_track("le_mans_sarthe");
            }

            // Level 5 (LMH Hypercar Prototype)
            if self.level >= 5 {
                self.ensure_car("hypercar_prototype");
                self.ensure_track("monaco");
                self.ensure_track("madring");
                self.ensure_track("marina_bay");
            }
        }
    }

    fn ensure_car(&mut self, id: &str) {
        if !self.unlocked_cars.iter().any(|c| c == id) {
            self.unlocked_cars.push(id.to_string());
        }
    }

    fn ensure_track(&mut self, id: &str) {
        if !self.unlocked_tracks.iter().any(|t| t == id) {
            self.unlocked_tracks.push(id.to_string());
        }
    }

    /// Checks if a vehicle is unlocked for this profile.
    pub fn is_car_unlocked(&self, car_id: &str, dev_mode: bool) -> bool {
        if dev_mode {
            return true;
        }
        self.unlocked_cars.iter().any(|c| c == car_id)
    }

    /// Checks if a circuit is unlocked for this profile.
    pub fn is_track_unlocked(&self, track_id: &str, dev_mode: bool) -> bool {
        if dev_mode {
            return true;
        }
        self.unlocked_tracks.iter().any(|t| t == track_id)
    }
}


