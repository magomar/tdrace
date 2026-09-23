use serde::{Deserialize, Serialize};

use super::{SeriesSession, PointSystem};

fn default_module() -> String {
    "gt".to_string()
}

fn default_tier() -> u32 {
    1
}

fn default_laps() -> u32 {
    4
}

fn default_scoring_system() -> String {
    "fia".to_string()
}

fn default_true() -> bool {
    true
}

/// Metadata describing a multi-race series, championship, or cup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesMeta {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_module")]
    pub module_id: String,
    #[serde(default = "default_tier")]
    pub tier: u32,
    #[serde(default = "default_laps")]
    pub laps_per_round: u32,
    #[serde(default)]
    pub bot_count: Option<usize>,
    #[serde(default)]
    pub ai_difficulty: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

pub type ChampionshipMeta = SeriesMeta;

impl Default for SeriesMeta {
    fn default() -> Self {
        Self {
            id: "custom_series".to_string(),
            name: "Custom Series".to_string(),
            description: "Custom multi-round racing series".to_string(),
            module_id: default_module(),
            tier: default_tier(),
            laps_per_round: default_laps(),
            bot_count: None,
            ai_difficulty: None,
            icon: None,
        }
    }
}

/// Scoring regulations and bonus rules for tournament standings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_scoring_system")]
    pub system: String,
    #[serde(default = "default_true")]
    pub fastest_lap_bonus: bool,
    #[serde(default)]
    pub stage_win_bonus: bool,
    #[serde(default)]
    pub clean_race_bonus: bool,
    #[serde(default)]
    pub custom_points: Vec<u32>,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            system: default_scoring_system(),
            fastest_lap_bonus: true,
            stage_win_bonus: false,
            clean_race_bonus: false,
            custom_points: Vec::new(),
        }
    }
}

impl ScoringConfig {
    pub fn to_point_system(&self) -> PointSystem {
        match self.system.to_ascii_lowercase().as_str() {
            "fia" | "fia_standard" | "f1" => PointSystem::FiaStandard {
                fastest_lap_bonus: self.fastest_lap_bonus,
            },
            "motogp" | "moto_gp" => PointSystem::MotoGp,
            "arcade" | "classic_arcade" => PointSystem::ClassicArcade,
            "nascar" | "nascar_cup" => PointSystem::NascarCup {
                stage_win_bonus: self.stage_win_bonus,
            },
            "custom" if !self.custom_points.is_empty() => {
                PointSystem::Custom(self.custom_points.clone())
            }
            _ => PointSystem::FiaStandard {
                fastest_lap_bonus: self.fastest_lap_bonus,
            },
        }
    }

    pub fn from_point_system(sys: &PointSystem) -> Self {
        match sys {
            PointSystem::FiaStandard { fastest_lap_bonus } => Self {
                system: "fia".to_string(),
                fastest_lap_bonus: *fastest_lap_bonus,
                stage_win_bonus: false,
                clean_race_bonus: false,
                custom_points: Vec::new(),
            },
            PointSystem::MotoGp => Self {
                system: "motogp".to_string(),
                fastest_lap_bonus: false,
                stage_win_bonus: false,
                clean_race_bonus: false,
                custom_points: Vec::new(),
            },
            PointSystem::ClassicArcade => Self {
                system: "arcade".to_string(),
                fastest_lap_bonus: false,
                stage_win_bonus: false,
                clean_race_bonus: false,
                custom_points: Vec::new(),
            },
            PointSystem::NascarCup { stage_win_bonus } => Self {
                system: "nascar".to_string(),
                fastest_lap_bonus: false,
                stage_win_bonus: *stage_win_bonus,
                clean_race_bonus: false,
                custom_points: Vec::new(),
            },
            PointSystem::Custom(pts) => Self {
                system: "custom".to_string(),
                fastest_lap_bonus: false,
                stage_win_bonus: false,
                clean_race_bonus: false,
                custom_points: pts.clone(),
            },
        }
    }
}

/// Calendar round entry linking a circuit to the championship schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundConfig {
    #[serde(default)]
    pub order: usize,
    pub track_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub laps: Option<u32>,
    #[serde(default)]
    pub weather: Option<String>,
}

/// Roster participant definition for either the player or an AI competitor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverConfig {
    pub id: String,
    pub name: String,
    pub team: String,
    #[serde(default)]
    pub is_player: bool,
    #[serde(default)]
    pub car_model_id: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub ai_character: Option<String>,
    #[serde(default)]
    pub ai_style: Option<String>,
    #[serde(default)]
    pub ai_tier: Option<u8>,
    #[serde(default)]
    pub livery_idx: Option<u8>,
}

/// Full declarative multi-race series specification file document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesDefinition {
    #[serde(alias = "championship", alias = "cup")]
    pub series: SeriesMeta,
    #[serde(default)]
    pub scoring: ScoringConfig,
    pub rounds: Vec<RoundConfig>,
    pub drivers: Vec<DriverConfig>,
}

pub type ChampionshipDefinition = SeriesDefinition;

impl Default for SeriesDefinition {
    fn default() -> Self {
        Self {
            series: SeriesMeta::default(),
            scoring: ScoringConfig::default(),
            rounds: Vec::new(),
            drivers: Vec::new(),
        }
    }
}

impl SeriesDefinition {
    pub fn championship(&self) -> &SeriesMeta {
        &self.series
    }

    pub fn championship_mut(&mut self) -> &mut SeriesMeta {
        &mut self.series
    }

    pub fn cup(&self) -> &SeriesMeta {
        &self.series
    }

    pub fn cup_mut(&mut self) -> &mut SeriesMeta {
        &mut self.series
    }

    /// Deserializes a declarative series definition from TOML format.
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Serializes this series definition to a formatted TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Validates the structural integrity and logic of the series definition.
    ///
    /// Returns `Ok(())` if valid, or `Err(Vec<String>)` containing all diagnostic errors.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // 1. Series ID & Name checks
        let id_trimmed = self.series.id.trim();
        if id_trimmed.is_empty() {
            errors.push("Series ID cannot be empty".to_string());
        } else if !id_trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            errors.push("Series ID must contain only alphanumeric, hyphen, and underscore characters".to_string());
        }

        if self.series.name.trim().is_empty() {
            errors.push("Series name cannot be empty".to_string());
        }

        if self.series.laps_per_round == 0 {
            errors.push("Default laps per round must be greater than 0".to_string());
        }

        // 2. Rounds checks
        if self.rounds.is_empty() {
            errors.push("Series must contain at least 1 round".to_string());
        }
        let mut seen_orders = std::collections::HashSet::new();
        for (idx, r) in self.rounds.iter().enumerate() {
            if !seen_orders.insert(r.order) {
                errors.push(format!("Duplicate round order detected: {}", r.order));
            }
            if r.track_id.trim().is_empty() {
                errors.push(format!("Round {} track ID cannot be empty", idx + 1));
            }
            if let Some(laps) = r.laps {
                if laps == 0 {
                    errors.push(format!("Round {} custom laps cannot be 0", idx + 1));
                }
            }
        }

        // 3. Drivers checks
        if self.drivers.len() < 2 {
            errors.push("Series grid must contain at least 2 drivers".to_string());
        }

        let player_count = self.drivers.iter().filter(|d| d.is_player).count();
        if player_count == 0 {
            errors.push("Series must designate exactly 1 player driver (found 0)".to_string());
        } else if player_count > 1 {
            errors.push(format!("Series can only have 1 player driver (found {})", player_count));
        }

        let mut seen_ids = std::collections::HashSet::new();
        for d in &self.drivers {
            if d.id.trim().is_empty() {
                errors.push("Driver ID cannot be empty".to_string());
            } else if !seen_ids.insert(&d.id) {
                errors.push(format!("Duplicate driver ID detected: '{}'", d.id));
            }
            if d.name.trim().is_empty() {
                errors.push(format!("Driver '{}' name cannot be empty", d.id));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Converts this declarative series definition into an active runtime `SeriesSession`.
    pub fn to_session(&self) -> SeriesSession {
        let point_system = self.scoring.to_point_system();
        let track_ids: Vec<String> = self.rounds.iter().map(|r| r.track_id.clone()).collect();
        let round_laps: Vec<Option<u32>> = self.rounds.iter().map(|r| r.laps).collect();
        let initial_drivers: Vec<(&str, &str, &str)> = self
            .drivers
            .iter()
            .map(|d| (d.id.as_str(), d.name.as_str(), d.team.as_str()))
            .collect();

        let mut session = SeriesSession::new(
            &self.series.name,
            point_system,
            track_ids,
            self.series.laps_per_round,
            &initial_drivers,
        )
        .with_tier(self.series.tier)
        .with_round_laps(round_laps);

        for (standing, d) in session.standings.iter_mut().zip(&self.drivers) {
            standing.ai_character = d.ai_character.clone();
            standing.ai_style = d.ai_style.clone();
            standing.ai_tier = d.ai_tier;
        }

        session
    }

    /// Recovers a declarative series definition from an active runtime `SeriesSession`.
    pub fn from_session(session: &SeriesSession, module_id: &str, tier: u32) -> Self {
        let slug = session
            .name
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<String>();

        let series = SeriesMeta {
            id: slug,
            name: session.name.clone(),
            description: format!("{} series", session.name),
            module_id: module_id.to_string(),
            tier,
            laps_per_round: session.laps_per_round,
            bot_count: Some(session.standings.len().saturating_sub(1)),
            ai_difficulty: None,
            icon: None,
        };

        let scoring = ScoringConfig::from_point_system(&session.point_system);

        let rounds = session
            .track_ids
            .iter()
            .enumerate()
            .map(|(idx, tid)| RoundConfig {
                order: idx + 1,
                track_id: tid.clone(),
                name: None,
                laps: session.round_laps.get(idx).copied().flatten(),
                weather: None,
            })
            .collect();

        let drivers = session
            .standings
            .iter()
            .enumerate()
            .map(|(idx, s)| DriverConfig {
                id: s.driver_id.clone(),
                name: s.driver_name.clone(),
                team: s.team_name.clone(),
                is_player: idx == 0 || s.driver_id == "player",
                car_model_id: None,
                country: None,
                ai_character: s.ai_character.clone(),
                ai_style: s.ai_style.clone(),
                ai_tier: s.ai_tier,
                livery_idx: Some(idx as u8),
            })
            .collect();

        Self {
            series,
            scoring,
            rounds,
            drivers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_roundtrip() {
        let toml_str = r#"
[championship]
id = "gt4_sprint"
name = "GT4 European Sprint"
description = "Sprint cup for GT4 machinery"
module_id = "gt"
tier = 1
laps_per_round = 4

[scoring]
system = "fia"
fastest_lap_bonus = true

[[rounds]]
order = 1
track_id = "monza"
name = "Monza Autodromo"

[[rounds]]
order = 2
track_id = "spa"
laps = 5

[[drivers]]
id = "player"
name = "Player One"
team = "Apex Racing"
is_player = true
car_model_id = "gt_toyota_supra_gt4"

[[drivers]]
id = "max"
name = "Max Hunter"
team = "Red Bull GT"
car_model_id = "gt_porsche_718_cayman_gt4_rs"
"#;

        let def = ChampionshipDefinition::from_toml(toml_str).expect("Valid TOML should deserialize");
        assert_eq!(def.series.id, "gt4_sprint");
        assert_eq!(def.series.name, "GT4 European Sprint");
        assert_eq!(def.rounds.len(), 2);
        assert_eq!(def.rounds[0].track_id, "monza");
        assert_eq!(def.rounds[1].laps, Some(5));
        assert_eq!(def.drivers.len(), 2);
        assert!(def.drivers[0].is_player);

        assert!(def.validate().is_ok());

        let serialized = def.to_toml().expect("Serialization to TOML must succeed");
        let def2 = ChampionshipDefinition::from_toml(&serialized).expect("Deserialization back must succeed");
        assert_eq!(def, def2);
    }

    #[test]
    fn test_validation_catches_invalid_inputs() {
        let mut def = ChampionshipDefinition::default();
        def.series.id = "bad id with spaces!".to_string();
        def.series.name = "".to_string();
        def.series.laps_per_round = 0;
        def.rounds = vec![];
        def.drivers = vec![];

        let errs = def.validate().expect_err("Should fail validation");
        assert!(errs.iter().any(|e| e.contains("alphanumeric")));
        assert!(errs.iter().any(|e| e.contains("name cannot be empty")));
        assert!(errs.iter().any(|e| e.contains("at least 1 round")));
        assert!(errs.iter().any(|e| e.contains("at least 2 drivers")));

        // Test duplicate driver id and 0 player slots
        def.series.id = "valid_id".to_string();
        def.series.name = "Valid Cup".to_string();
        def.series.laps_per_round = 3;
        def.rounds = vec![RoundConfig {
            order: 1,
            track_id: "monza".to_string(),
            name: None,
            laps: None,
            weather: None,
        }];
        def.drivers = vec![
            DriverConfig {
                id: "bot1".to_string(),
                name: "Bot 1".to_string(),
                team: "Team A".to_string(),
                is_player: false,
                car_model_id: None,
                country: None,
                ai_character: None,
                ai_style: None,
                ai_tier: None,
                livery_idx: None,
            },
            DriverConfig {
                id: "bot1".to_string(), // Duplicate
                name: "Bot 2".to_string(),
                team: "Team B".to_string(),
                is_player: false,
                car_model_id: None,
                country: None,
                ai_character: None,
                ai_style: None,
                ai_tier: None,
                livery_idx: None,
            },
        ];
        let errs2 = def.validate().expect_err("Duplicate ID and no player should fail");
        assert!(errs2.iter().any(|e| e.contains("Duplicate driver ID")));
        assert!(errs2.iter().any(|e| e.contains("exactly 1 player driver")));
    }

    #[test]
    fn test_session_conversion() {
        let def = ChampionshipDefinition {
            series: SeriesMeta {
                id: "nascar_tier1".to_string(),
                name: "NASCAR Grassroots Cup".to_string(),
                description: "Oval racing".to_string(),
                module_id: "nascar".to_string(),
                tier: 1,
                laps_per_round: 10,
                bot_count: Some(1),
                ai_difficulty: None,
                icon: None,
            },
            scoring: ScoringConfig {
                system: "nascar".to_string(),
                fastest_lap_bonus: false,
                stage_win_bonus: true,
                clean_race_bonus: false,
                custom_points: vec![],
            },
            rounds: vec![
                RoundConfig {
                    order: 1,
                    track_id: "oval_speedway".to_string(),
                    name: None,
                    laps: None,
                    weather: None,
                },
            ],
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
                    id: "dale".to_string(),
                    name: "Dale Vance".to_string(),
                    team: "RCR".to_string(),
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

        let session = def.to_session();
        assert_eq!(session.name, "NASCAR Grassroots Cup");
        assert_eq!(session.track_ids, vec!["oval_speedway"]);
        assert_eq!(session.laps_per_round, 10);
        assert_eq!(session.standings.len(), 2);
        assert_eq!(session.standings[0].driver_id, "player");
        assert_eq!(session.standings[1].driver_id, "dale");
        assert_eq!(
            session.point_system,
            PointSystem::NascarCup { stage_win_bonus: true }
        );

        let recovered = ChampionshipDefinition::from_session(&session, "nascar", 1);
        assert_eq!(recovered.series.name, "NASCAR Grassroots Cup");
        assert_eq!(recovered.scoring.system, "nascar");
        assert_eq!(recovered.rounds.len(), 1);
        assert_eq!(recovered.rounds[0].track_id, "oval_speedway");
    }
}
