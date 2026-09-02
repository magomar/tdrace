use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::render::color::CarColorScheme;

pub use cabinet::profile::country::{draw_country_banner, CountryInfo, CountryRegistry};

/// Player Profile representing driver identity, livery customizations, and nationality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: Option<i64>,
    pub name: String,
    pub alias: String,
    pub country: Option<String>,
    pub color_scheme: CarColorScheme,
    pub is_active: bool,
    pub created_at: String,
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


