use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::ai::{CareerRivalEntry, DriverTier, RosterEvolutionEngine, RosterEvolutionReport};
use crate::render::color::CarColorScheme;
pub use tdrace_core::physics::config::AssistProfile;
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
    #[serde(default = "PlayerProfile::default_starting_credits")]
    pub credits: u64,
    #[serde(default = "PlayerProfile::default_starting_credits")]
    pub lifetime_credits: u64,
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
            credits: Self::STARTING_CREDITS,
            lifetime_credits: Self::STARTING_CREDITS,
        }
    }
}

impl PlayerProfile {
    pub const STARTING_CREDITS: u64 = 25_000;

    pub fn default_starting_credits() -> u64 {
        Self::STARTING_CREDITS
    }

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
            credits: Self::STARTING_CREDITS,
            lifetime_credits: Self::STARTING_CREDITS,
        }
    }

    /// Adds credits to wallet balance and lifetime earnings.
    pub fn add_credits(&mut self, amount: u64) {
        self.credits = self.credits.saturating_add(amount);
        self.lifetime_credits = self.lifetime_credits.saturating_add(amount);
    }

    /// Spends credits from wallet balance.
    pub fn spend_credits(&mut self, amount: u64) -> Result<(), String> {
        if self.credits < amount {
            return Err(format!(
                "Insufficient credits: {} available, {} required",
                self.credits, amount
            ));
        }
        self.credits -= amount;
        Ok(())
    }

    /// Checks if wallet can afford given credit cost.
    pub fn can_afford(&self, amount: u64) -> bool {
        self.credits >= amount
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

    /// Returns default career progress for a given module associated with this profile.
    pub fn module_progress(&self, module_id: &str) -> ModuleCareerProgress {
        ModuleCareerProgress::default_for_module(self.id.unwrap_or(1), module_id)
    }
}

/// Category-specific aggregated career statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CategoryCareerStats {
    pub category: String,
    pub total_races: u32,
    pub wins: u32,
    pub p2_count: u32,
    pub p3_count: u32,
    pub podiums: u32,
    pub total_laps: u32,
    pub total_stunt_score: u32,
    pub total_collisions: u32,
    pub clean_races: u32,
    pub win_rate: f32,
    pub podium_rate: f32,
    pub clean_rate: f32,
}

/// Aggregated career statistics computed from persistent race history logs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProfileCareerStats {
    pub total_races: u32,
    pub wins: u32,
    pub p2_count: u32,
    pub p3_count: u32,
    pub podiums: u32,
    pub total_laps: u32,
    pub win_rate: f32,
    pub podium_rate: f32,
    pub total_stunt_score: u32,
    pub max_stunt_score: u32,
    pub total_collisions: u32,
    pub clean_races: u32,
    pub clean_rate: f32,
    pub best_times: BTreeMap<String, f32>,
    pub best_circuit_times: BTreeMap<String, f32>,
    pub category_stats: BTreeMap<String, CategoryCareerStats>,
}

pub type GlobalProfileStats = ProfileCareerStats;

impl ProfileCareerStats {
    pub fn compute(races: &[RaceHistoryEntry]) -> Self {
        let mut total_races = 0u32;
        let mut wins = 0u32;
        let mut p2_count = 0u32;
        let mut p3_count = 0u32;
        let mut podiums = 0u32;
        let mut total_laps = 0u32;
        let mut total_stunt_score = 0u32;
        let mut max_stunt_score = 0u32;
        let mut total_collisions = 0u32;
        let mut clean_races = 0u32;
        let mut best_times: BTreeMap<String, f32> = BTreeMap::new();
        let mut best_circuit_times: BTreeMap<String, f32> = BTreeMap::new();
        let mut category_stats: BTreeMap<String, CategoryCareerStats> = BTreeMap::new();

        for race in races {
            total_races += 1;
            total_laps += race.laps;
            total_stunt_score = total_stunt_score.saturating_add(race.stunt_score);
            max_stunt_score = max_stunt_score.max(race.stunt_score);
            total_collisions = total_collisions.saturating_add(race.collisions);

            if race.collisions == 0 {
                clean_races += 1;
            }

            if race.position == 1 {
                wins += 1;
            } else if race.position == 2 {
                p2_count += 1;
            } else if race.position == 3 {
                p3_count += 1;
            }
            if race.position >= 1 && race.position <= 3 {
                podiums += 1;
            }

            // Per-category aggregation
            let cat_key = if race.category.is_empty() { "gt" } else { &race.category };
            let cat_entry = category_stats.entry(cat_key.to_string()).or_insert_with(|| CategoryCareerStats {
                category: cat_key.to_string(),
                ..Default::default()
            });
            cat_entry.total_races += 1;
            cat_entry.total_laps += race.laps;
            cat_entry.total_stunt_score = cat_entry.total_stunt_score.saturating_add(race.stunt_score);
            cat_entry.total_collisions = cat_entry.total_collisions.saturating_add(race.collisions);
            if race.collisions == 0 {
                cat_entry.clean_races += 1;
            }
            if race.position == 1 {
                cat_entry.wins += 1;
            } else if race.position == 2 {
                cat_entry.p2_count += 1;
            } else if race.position == 3 {
                cat_entry.p3_count += 1;
            }
            if race.position >= 1 && race.position <= 3 {
                cat_entry.podiums += 1;
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

        // Finalize rates for each category
        for cat in category_stats.values_mut() {
            if cat.total_races > 0 {
                cat.win_rate = (cat.wins as f32 / cat.total_races as f32) * 100.0;
                cat.podium_rate = (cat.podiums as f32 / cat.total_races as f32) * 100.0;
                cat.clean_rate = (cat.clean_races as f32 / cat.total_races as f32) * 100.0;
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

        let clean_rate = if total_races > 0 {
            (clean_races as f32 / total_races as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_races,
            wins,
            p2_count,
            p3_count,
            podiums,
            total_laps,
            win_rate,
            podium_rate,
            total_stunt_score,
            max_stunt_score,
            total_collisions,
            clean_races,
            clean_rate,
            best_times,
            best_circuit_times,
            category_stats,
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
    // Enhanced Multi-Level Telemetry
    pub category: String,
    pub championship_name: Option<String>,
    pub stunt_score: u32,
    pub collisions: u32,
}

impl Default for RaceHistoryEntry {
    fn default() -> Self {
        Self {
            id: None,
            profile_id: 0,
            track_id: String::new(),
            car_name: String::new(),
            position: 1,
            total_cars: 1,
            total_time: 0.0,
            best_lap: None,
            laps: 1,
            is_time_attack: false,
            created_at: String::new(),
            category: "gt".to_string(),
            championship_name: None,
            stunt_score: 0,
            collisions: 0,
        }
    }
}

/// Metallic standing for championship podium finishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrophyMetal {
    Gold,
    Silver,
    Bronze,
}

impl TrophyMetal {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gold => "gold",
            Self::Silver => "silver",
            Self::Bronze => "bronze",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Gold => "Gold Champion",
            Self::Silver => "Silver Finalist",
            Self::Bronze => "Bronze Podium",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Gold => "🏆",
            Self::Silver => "🥈",
            Self::Bronze => "🥉",
        }
    }

    pub fn badge_color(&self) -> macroquad::color::Color {
        match self {
            Self::Gold => macroquad::color::Color::new(1.0, 0.85, 0.2, 1.0),
            Self::Silver => macroquad::color::Color::new(0.85, 0.9, 0.95, 1.0),
            Self::Bronze => macroquad::color::Color::new(0.85, 0.55, 0.35, 1.0),
        }
    }
}

/// Represents an authentic championship trophy won by a player profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChampionshipAward {
    pub profile_id: i64,
    pub championship_id: String,
    pub module_id: String,
    pub tier: u32,
    pub position: u32, // 1 = Gold, 2 = Silver, 3 = Bronze
    pub points: u32,
    pub car_model_id: String,
    pub achieved_at: String,
}

impl ChampionshipAward {
    /// Returns the trophy metallic tier enum (Gold, Silver, Bronze).
    pub fn metallic_tier(&self) -> TrophyMetal {
        match self.position {
            1 => TrophyMetal::Gold,
            2 => TrophyMetal::Silver,
            3 => TrophyMetal::Bronze,
            _ => TrophyMetal::Bronze,
        }
    }

    /// Returns the motorsport discipline identifier.
    pub fn discipline(&self) -> &str {
        &self.module_id
    }
}

/// Persistent summary record of a completed championship tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChampionshipRecord {
    pub championship_id: String,
    #[serde(default = "default_tier_one")]
    pub tier: u32,
    pub best_finish: u32, // 1 = Gold, 2 = Silver, 3 = Bronze, etc.
    pub times_completed: u32,
    pub highest_points: u32,
    pub last_completed_at: String,
}

fn default_tier_one() -> u32 {
    1
}

/// Persistent career progression record for a specific motorsport module (e.g. "gt", "rally", etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCareerProgress {
    pub profile_id: i64,
    pub module_id: String,
    pub xp: u64,
    pub lifetime_xp: u64,
    pub level: u32,
    pub unlocked_cars: Vec<String>,
    pub unlocked_tracks: Vec<String>,
    pub visited_tracks: Vec<String>,
    pub completed_events: Vec<String>,
    pub trophies_gold: u32,
    pub trophies_silver: u32,
    pub trophies_bronze: u32,
    pub updated_at: String,
    #[serde(default)]
    pub career_rivals: Vec<CareerRivalEntry>,
    #[serde(default)]
    pub active_championship: Option<crate::series::ChampionshipSession>,
    #[serde(default)]
    pub championships_completed: std::collections::HashMap<String, ChampionshipRecord>,
}

impl ModuleCareerProgress {
    /// Returns the authentic starter car model IDs for a given motorsport module and tier (1..=5).
    pub fn starter_cars_for_module_and_tier(module_id: &str, tier: u32) -> Vec<String> {
        match (module_id, tier) {
            ("gt" | "gt_challenge", 1) => vec!["gt_toyota_supra_gt4".to_string(), "gt4_clubsport".to_string()],
            ("gt" | "gt_challenge", _) => Vec::new(),

            ("nascar", 1) => vec!["nascar_monte_carlo_ss".to_string()],
            ("nascar", 2) => vec!["nascar_super_late_model".to_string()],
            ("nascar", 3) => vec!["nascar_arca_chevy_ss".to_string()],
            ("nascar", 4) => vec!["nascar_silverado_truck".to_string()],
            ("nascar", 5) => vec!["nascar_corvette_ta1".to_string()],

            ("rally", 1) => vec!["rally_peugeot_208_rally4".to_string()],
            ("rally", 2) => vec!["rally_omse_supercar_lites".to_string()],
            ("rally", 3) => vec!["rally_polo_rx".to_string()],
            ("rally", 4) => vec!["rally_peugeot_208_wrx".to_string()],
            ("rally", 5) => vec!["rally_peugeot_208_rx1e".to_string()],
            ("rally", 6) => vec!["rally_omse_fc1x".to_string()],
            ("rally", 0 | 7) => vec!["rally_audi_sport_quattro_s1".to_string()],

            ("kart", 1) => vec!["kart_crg_hero_60".to_string()],
            ("kart", 2) => vec!["kart_tony_kart_rookie_okj".to_string()],
            ("kart", 3) => vec!["kart_tony_kart_racer_ok".to_string()],
            ("kart", 4) => vec!["kart_birel_art_kz2".to_string()],
            ("kart", 5) => vec!["kart_anderson_maverick_mono".to_string()],
            ("kart", 6) => vec!["kart_anderson_cs250".to_string()],

            ("extreme_offroad" | "offroad", 1) => vec!["offroad_sand_rail_buggy".to_string()],
            ("extreme_offroad" | "offroad", 2) => vec!["offroad_ford_bronco_dr".to_string()],
            ("extreme_offroad" | "offroad", 3) => vec!["offroad_arctic_hilux_at44".to_string()],
            ("extreme_offroad" | "offroad", 4) => vec!["offroad_pro4_unlimited_chevy".to_string()],
            ("extreme_offroad" | "offroad", 5) => vec!["offroad_bigfoot_monster_truck".to_string()],

            _ => {
                if tier == 1 {
                    vec!["gt_toyota_supra_gt4".to_string(), "gt4_clubsport".to_string()]
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Returns the authentic starter car model IDs for a given motorsport module (Tier 1).
    pub fn starter_cars_for_module(module_id: &str) -> Vec<String> {
        Self::starter_cars_for_module_and_tier(module_id, 1)
    }

    /// Initial starter career record for a specific motorsport module.
    pub fn default_for_module(profile_id: i64, module_id: &str) -> Self {
        let mut progress = Self {
            profile_id,
            module_id: module_id.to_string(),
            xp: 0,
            lifetime_xp: 0,
            level: 1,
            unlocked_cars: Self::starter_cars_for_module(module_id),
            unlocked_tracks: Vec::new(),
            visited_tracks: Vec::new(),
            completed_events: Vec::new(),
            trophies_gold: 0,
            trophies_silver: 0,
            trophies_bronze: 0,
            updated_at: String::new(),
            career_rivals: Vec::new(),
            active_championship: None,
            championships_completed: std::collections::HashMap::new(),
        };
        progress.sync_unlocks_for_level();
        progress
    }

    /// Initial starter career record for Gran Turismo & Endurance GT module.
    pub fn default_for_gt(profile_id: i64) -> Self {
        Self::default_for_module(profile_id, "gt")
    }

    /// Purchasing cost for a vehicle of the specified tier in Credits (Spec 053).
    pub fn car_credit_cost(tier: u8) -> u64 {
        match tier {
            1 => 25_000,
            2 => 60_000,
            3 => 150_000,
            4 => 380_000,
            5 => 950_000,
            _ => (tier as u64) * 200_000,
        }
    }

    /// Vehicle credit cost (alias for `car_credit_cost`).
    pub fn car_cost(tier: u8) -> u64 {
        Self::car_credit_cost(tier)
    }

    /// Cumulative discipline XP threshold required to unlock each tier license (Spec 053).
    pub fn tier_license_xp(tier: u32) -> u64 {
        match tier {
            1 => 0,
            2 => 3_000,
            3 => 7_500,
            4 => 15_000,
            5 => 30_000,
            _ => u64::MAX,
        }
    }

    /// Cumulative discipline XP threshold required to unlock each tier license for a specific module (Spec 052, 053).
    pub fn tier_license_xp_for_module(module_id: &str, tier: u32) -> u64 {
        if module_id == "kart" {
            match tier {
                1 => 0,
                2 => 1_500,
                3 => 3_500,
                4 => 6_000,
                5 => 9_000,
                6 => 13_000,
                _ => u64::MAX,
            }
        } else {
            Self::tier_license_xp(tier)
        }
    }

    /// Base round prize purse (Credits) per career tier (Spec 053).
    pub fn round_base_purse(tier: u32) -> u64 {
        match tier {
            1 => 5_000,
            2 => 12_000,
            3 => 25_000,
            4 => 55_000,
            5 => 120_000,
            _ => (tier as u64) * 25_000,
        }
    }

    /// Prize purse finishing position multiplier.
    pub fn purse_position_multiplier(position: usize) -> f64 {
        match position {
            1 => 1.00,
            2 => 0.70,
            3 => 0.50,
            4 => 0.35,
            5 => 0.25,
            6..=10 => 0.15,
            _ => 0.08,
        }
    }

    /// Calculates finish purse prize and clean race bonus (Credits).
    pub fn calculate_round_purse(tier: u32, position: usize, is_clean: bool) -> (u64, u64) {
        let base = Self::round_base_purse(tier);
        let mult = Self::purse_position_multiplier(position);
        let finish_prize = ((base as f64) * mult).round() as u64;
        let clean_bonus = if is_clean {
            ((finish_prize as f64) * 0.20).round() as u64
        } else {
            0
        };
        (finish_prize, clean_bonus)
    }

    /// Championship overall podium bonus credits.
    pub fn championship_podium_bonus(tier: u32, position: usize) -> u64 {
        let base = Self::round_base_purse(tier);
        match position {
            1 => ((base as f64) * 4.0).round() as u64,
            2 => ((base as f64) * 2.5).round() as u64,
            3 => ((base as f64) * 1.5).round() as u64,
            _ => 0,
        }
    }

    /// Position multiplier for race XP awards.
    pub fn xp_position_multiplier(position: usize) -> f64 {
        match position {
            1 => 1.50,
            2 => 1.30,
            3 => 1.15,
            _ => 1.00,
        }
    }

    /// First-time exploration bonus awarded when playing a circuit for the first time (250 XP x tier, rounded to 10).
    pub fn first_time_circuit_bonus(tier: u32) -> u64 {
        Self::round_to_10((tier as u64) * 250)
    }

    /// Rounds an XP amount to the nearest multiple of 10 (so last digit is always 0).
    pub fn round_to_10(val: u64) -> u64 {
        ((val as f64 / 10.0).round() as u64) * 10
    }

    /// Returns the maximum career progression tier supported by this module.
    pub fn max_tier(&self) -> u32 {
        if self.module_id == "extreme_offroad" {
            7
        } else if self.module_id == "rally" || self.module_id == "kart" {
            6
        } else {
            5
        }
    }

    /// Target XP needed for next tier license threshold, or None if at max tier.
    pub fn next_tier_target_xp(&self) -> Option<u64> {
        if self.level >= self.max_tier() {
            None
        } else {
            Some(Self::tier_license_xp_for_module(&self.module_id, self.level + 1))
        }
    }

    /// Progress ratio [0.0..1.0] towards acquiring the next tier's license.
    pub fn level_progress_ratio(&self) -> f32 {
        if self.level >= self.max_tier() {
            return 1.0;
        }
        let target = self.next_tier_target_xp().unwrap_or(3_000);
        if target == 0 {
            1.0
        } else {
            (self.xp as f32 / target as f32).clamp(0.0, 1.0)
        }
    }

    /// Awards cumulative XP to discipline experience and cumulative lifetime XP.
    pub fn add_xp(&mut self, amount: u64) {
        self.xp = self.xp.saturating_add(amount);
        self.lifetime_xp = self.lifetime_xp.saturating_add(amount);
    }

    /// Checks if a vehicle can be purchased: must not already be unlocked, player must be at or above the car's tier,
    /// and player must have sufficient credits balance.
    pub fn can_buy_car(&self, car_id: &str, tier: u8, available_credits: u64) -> bool {
        !self.is_car_unlocked(car_id, false)
            && self.level >= (tier as u32)
            && available_credits >= Self::car_credit_cost(tier)
    }

    /// Purchases a vehicle, deducting its cost from the player's credit wallet and unlocking it.
    /// Note: Zero discipline XP is deducted.
    pub fn buy_car(&mut self, profile: &mut PlayerProfile, car_id: &str, tier: u8) -> Result<(), String> {
        let cost = Self::car_credit_cost(tier);
        if !self.can_buy_car(car_id, tier, profile.credits) {
            return Err(format!(
                "Cannot buy car '{}' (tier {}): insufficient credits ({}/{}) or insufficient tier ({})",
                car_id,
                tier,
                profile.credits,
                cost,
                self.level
            ));
        }
        profile.spend_credits(cost)?;
        self.ensure_car(car_id);
        Ok(())
    }

    /// Checks whether the driver has earned at least one podium finish (P1, P2, or P3) in any championship of the given tier.
    pub fn has_podium_in_tier(&self, tier: u32) -> bool {
        let has_champ_podium = self
            .championships_completed
            .values()
            .any(|c| c.tier == tier && c.best_finish <= 3);
        if has_champ_podium {
            return true;
        }
        // Fallback: If legacy trophies exist and requested tier matches current level, allow backwards compatibility
        if tier == self.level && (self.trophies_gold + self.trophies_silver + self.trophies_bronze) > 0 {
            return true;
        }
        false
    }

    /// Records a completed championship result, updating best finish, count, and legacy trophy counts.
    pub fn record_championship_finish(
        &mut self,
        championship_id: &str,
        tier: u32,
        position: u32,
        points: u32,
        timestamp: &str,
    ) {
        let entry = self
            .championships_completed
            .entry(championship_id.to_string())
            .or_insert_with(|| ChampionshipRecord {
                championship_id: championship_id.to_string(),
                tier,
                best_finish: position,
                times_completed: 0,
                highest_points: points,
                last_completed_at: timestamp.to_string(),
            });

        entry.times_completed += 1;
        if position < entry.best_finish {
            entry.best_finish = position;
        }
        if points > entry.highest_points {
            entry.highest_points = points;
        }
        entry.tier = tier;
        entry.last_completed_at = timestamp.to_string();

        match position {
            1 => self.trophies_gold += 1,
            2 => self.trophies_silver += 1,
            3 => self.trophies_bronze += 1,
            _ => {}
        }
    }

    /// Checks whether the driver satisfies both conditions to advance to the next tier:
    /// 1. Earned at least one championship podium in any championship of the current tier.
    /// 2. Accumulated cumulative discipline XP at or above the next tier's license threshold.
    pub fn can_advance_tier(&self) -> bool {
        if self.level >= self.max_tier() {
            return false;
        }
        let next_tier = self.level + 1;
        let has_podium = self.has_podium_in_tier(self.level);
        let has_xp = self.xp >= Self::tier_license_xp_for_module(&self.module_id, next_tier);
        has_podium && has_xp
    }

    /// Ensures career rivals are initialized for this module progress record.
    pub fn ensure_career_rivals(&mut self, opponent_count: usize, seed: u64) -> &[CareerRivalEntry] {
        self.ensure_career_rivals_with_pool(&[], opponent_count, seed)
    }

    /// Ensures career rivals are initialized from an optional driver pool.
    pub fn ensure_career_rivals_with_pool(
        &mut self,
        pool: &[crate::ai::DriverCharacter],
        opponent_count: usize,
        seed: u64,
    ) -> &[CareerRivalEntry] {
        if self.career_rivals.is_empty() {
            let tier = DriverTier::from_u8(self.level.clamp(1, 5) as u8);
            self.career_rivals = RosterEvolutionEngine::initialize_career_roster_from_pool(pool, opponent_count, tier, seed);
        } else if self.career_rivals.len() < opponent_count {
            let diff = opponent_count - self.career_rivals.len();
            let tier = DriverTier::from_u8(self.level.clamp(1, 5) as u8);
            let extra = RosterEvolutionEngine::initialize_career_roster_from_pool(pool, diff, tier, seed.wrapping_add(12345));
            for r in extra {
                if !self.career_rivals.iter().any(|existing| existing.driver_id == r.driver_id) {
                    self.career_rivals.push(r);
                }
            }
        }
        &self.career_rivals
    }

    /// Evolves the career rival roster upon unlocking a new tier.
    pub fn evolve_career_rivals(&mut self, new_tier: DriverTier, seed: u64) -> RosterEvolutionReport {
        let (updated, report) = RosterEvolutionEngine::evolve_roster(&self.career_rivals, new_tier, seed);
        self.career_rivals = updated;
        report
    }

    /// Advances to the next career tier using an explicit deterministic seed for roster evolution.
    pub fn advance_tier_with_seed(&mut self, seed: u64) -> Result<(u32, RosterEvolutionReport), String> {
        if !self.can_advance_tier() {
            let next_tier = self.level + 1;
            return Err(format!(
                "Cannot advance to Tier {}: Requires at least 1 championship podium in Tier {} and {} cumulative XP (current: {})",
                next_tier,
                self.level,
                Self::tier_license_xp_for_module(&self.module_id, next_tier),
                self.xp
            ));
        }
        self.level += 1;
        self.sync_unlocks_for_level();
        let report = self.evolve_career_rivals(DriverTier::from_u8(self.level.clamp(1, 5) as u8), seed);
        Ok((self.level, report))
    }

    /// Advances to the next career tier if all conditions are met and evolves rivals.
    pub fn advance_tier(&mut self) -> Result<u32, String> {
        let seed = (self.level as u64).wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(self.lifetime_xp);
        self.advance_tier_with_seed(seed).map(|(lvl, _)| lvl)
    }

    /// Ensures unlocked tracks and starter cars match or exceed current level.
    pub fn sync_unlocks_for_level(&mut self) {
        let max_tier = self.max_tier();
        let cap = self.level.clamp(1, max_tier);
        for t in 1..=cap {
            for car in Self::starter_cars_for_module_and_tier(&self.module_id, t) {
                self.ensure_car(&car);
            }
        }
        match self.module_id.as_str() {
            "gt" | "gt_challenge" => {
                // Tier 1 (5 circuits)
                self.ensure_track("red_bull_ring");
                self.ensure_track("zandvoort");
                self.ensure_track("nurburgring_gp");
                self.ensure_track("portimao_gp");
                self.ensure_track("montreal");

                // Tier 2 (3 circuits)
                if self.level >= 2 {
                    self.ensure_track("monza");
                    self.ensure_track("silverstone");
                    self.ensure_track("catalunya");
                }

                // Tier 3 (3 circuits)
                if self.level >= 3 {
                    self.ensure_track("spa");
                    self.ensure_track("cota");
                    self.ensure_track("bahrain");
                }

                // Tier 4 (3 circuits)
                if self.level >= 4 {
                    self.ensure_track("suzuka");
                    self.ensure_track("interlagos");
                    self.ensure_track("bathurst");
                }

                // Tier 5 (4 circuits)
                if self.level >= 5 {
                    self.ensure_track("le_mans_sarthe");
                    self.ensure_track("monaco");
                    self.ensure_track("marina_bay");
                    self.ensure_track("madring");
                }
            }
            "nascar" => {
                // Tier 1 (5 circuits)
                self.ensure_track("martinsville_speedway");
                self.ensure_track("bristol_motor_speedway");
                self.ensure_track("eldora_speedway");
                self.ensure_track("bowman_gray_stadium");
                self.ensure_track("lucas_oil_irp");

                // Tier 2 (3 circuits)
                if self.level >= 2 {
                    self.ensure_track("charlotte_motor_speedway");
                    self.ensure_track("darlington_raceway");
                    self.ensure_track("north_wilkesboro_speedway");
                }

                // Tier 3 (3 circuits)
                if self.level >= 3 {
                    self.ensure_track("iowa_speedway");
                    self.ensure_track("watkins_glen_nascar");
                    self.ensure_track("road_america");
                }

                // Tier 4 (3 circuits)
                if self.level >= 4 {
                    self.ensure_track("indianapolis_motor_speedway");
                    self.ensure_track("pocono_raceway");
                    self.ensure_track("chicago_street_course");
                }

                // Tier 5 (3 circuits)
                if self.level >= 5 {
                    self.ensure_track("daytona_superspeedway");
                    self.ensure_track("talladega_superspeedway");
                    self.ensure_track("phoenix_raceway");
                }
            }
            "rally" => {
                // Tier 1 (5 circuits)
                self.ensure_track("holjes_rx");
                self.ensure_track("lydden_hill");
                self.ensure_track("mettet_rx");
                self.ensure_track("dreux_rx");
                self.ensure_track("croft_rx");

                // Tier 2 (+3 circuits -> 8 total)
                if self.level >= 2 {
                    self.ensure_track("lessay_rx");
                    self.ensure_track("essay_rx");
                    self.ensure_track("lavare_rx");
                }

                // Tier 3 (+3 circuits -> 11 total)
                if self.level >= 3 {
                    self.ensure_track("kouvola_rx");
                    self.ensure_track("montalegre_rx");
                    self.ensure_track("nyirad_rx");
                }

                // Tier 4 (+3 circuits -> 14 total)
                if self.level >= 4 {
                    self.ensure_track("estering_rx");
                    self.ensure_track("hell_rx");
                    self.ensure_track("loheac_rx");
                }

                // Tier 5 (+3 circuits -> 17 total)
                if self.level >= 5 {
                    self.ensure_track("riga_rx");
                    self.ensure_track("killarney_rx");
                    self.ensure_track("catalunya_rx");
                }

                // Tier 6 (+3 NEW circuits -> 20 total)
                if self.level >= 6 {
                    self.ensure_track("spa_rx");
                    self.ensure_track("silverstone_rx");
                    self.ensure_track("erx_motor_park");
                }
            }
            "kart" => {
                // Tier 1 (5 circuits)
                self.ensure_track("lonato");
                self.ensure_track("genk");
                self.ensure_track("wackersdorf");
                self.ensure_track("laval_kart");
                self.ensure_track("whilton_mill");

                // Tier 2 (3 circuits)
                if self.level >= 2 {
                    self.ensure_track("sarno");
                    self.ensure_track("kristianstad");
                    self.ensure_track("seven_laghi");
                }

                // Tier 3 (3 circuits)
                if self.level >= 3 {
                    self.ensure_track("pfi");
                    self.ensure_track("franciacorta");
                    self.ensure_track("ampfing");
                }

                // Tier 4 (3 circuits)
                if self.level >= 4 {
                    self.ensure_track("zuera");
                    self.ensure_track("silverstone_national_kart");
                    self.ensure_track("aunay_kart");
                }

                // Tier 5 (3 circuits)
                if self.level >= 5 {
                    self.ensure_track("le_mans_kart");
                    self.ensure_track("campillos");
                    self.ensure_track("muelsen_kart");
                }

                // Tier 6 (3 circuits)
                if self.level >= 6 {
                    self.ensure_track("portimao_kart");
                    self.ensure_track("valencia_kart");
                    self.ensure_track("adria_kart");
                }
            }
            "extreme_offroad" => {
                // Tier 1 (8 circuits, with the three Mint 400 desert-race circuits of spec 048)
                self.ensure_track("sahara_dune_crossing");
                self.ensure_track("dirt_figure_eight");
                self.ensure_track("atacama_sand_basin");
                self.ensure_track("glamis_dunes");
                self.ensure_track("crandon_short_course");
                self.ensure_track("mint400_short_course");
                self.ensure_track("mint400_qualifying_loop");
                self.ensure_track("mint400_grand_loop");

                // Tier 2 (3 circuits)
                if self.level >= 2 {
                    self.ensure_track("red_rock_canyon");
                    self.ensure_track("mud_slough_arena");
                    self.ensure_track("baja_500_desert_scrub");
                }

                // Tier 3 (3 circuits)
                if self.level >= 3 {
                    self.ensure_track("arctic_frozen_lake");
                    self.ensure_track("alpine_snow_ridge");
                    self.ensure_track("rovaniemi_ice_ring");
                }

                // Tier 4 (3 circuits)
                if self.level >= 4 {
                    self.ensure_track("supercross_stadium_arena");
                    self.ensure_track("gravel_quarry_chasm");
                    self.ensure_track("louisiana_mud_swampland");
                }

                // Tier 5 (3 circuits)
                if self.level >= 5 {
                    self.ensure_track("monster_colosseum");
                    self.ensure_track("glacier_crest_pass");
                    self.ensure_track("stunt_city_megastructure");
                }
            }
            _ => {}
        }
    }

    pub fn ensure_car(&mut self, id: &str) {
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
        if car_id == "gt4_clubsport" && self.unlocked_cars.iter().any(|c| c == "gt_toyota_supra_gt4") {
            return true;
        }
        if car_id == "gt_toyota_supra_gt4" && self.unlocked_cars.iter().any(|c| c == "gt4_clubsport") {
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


