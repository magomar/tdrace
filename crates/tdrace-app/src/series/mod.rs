use serde::{Deserialize, Serialize};
use crate::ai::{CareerRivalEntry, DriverCharacter, DriverTier, DrivingStyle, RosterEvolutionEngine, RosterEvolutionReport};

pub mod format;
pub use format::*;
pub mod manager;
pub use manager::*;

/// Scoring system for championship tournaments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointSystem {
    /// Official FIA standard scoring: 25, 18, 15, 12, 10, 8, 6, 4, 2, 1 (plus optional fastest lap bonus)
    FiaStandard { fastest_lap_bonus: bool },
    /// MotoGP scoring: 25, 20, 16, 13, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1
    MotoGp,
    /// Classic arcade 6-place scoring: 10, 6, 4, 3, 2, 1
    ClassicArcade,
    /// Official NASCAR Cup Series scoring: 40 pts for 1st, 35 for 2nd, 34 for 3rd... down to 1 pt, plus stage win bonus (10 pts)
    NascarCup { stage_win_bonus: bool },
    /// Custom points matrix
    Custom(Vec<u32>),
}

impl PointSystem {
    pub fn points_for_position(&self, position: usize, has_fastest_lap: bool) -> u32 {
        if position == 0 {
            return 0;
        }
        let base_pts = match self {
            Self::FiaStandard { .. } => match position {
                1 => 25,
                2 => 18,
                3 => 15,
                4 => 12,
                5 => 10,
                6 => 8,
                7 => 6,
                8 => 4,
                9 => 2,
                10 => 1,
                _ => 0,
            },
            Self::MotoGp => match position {
                1 => 25,
                2 => 20,
                3 => 16,
                4 => 13,
                5 => 11,
                6 => 10,
                7 => 9,
                8 => 8,
                9 => 7,
                10 => 6,
                11 => 5,
                12 => 4,
                13 => 3,
                14 => 2,
                15 => 1,
                _ => 0,
            },
            Self::ClassicArcade => match position {
                1 => 10,
                2 => 6,
                3 => 4,
                4 => 3,
                5 => 2,
                6 => 1,
                _ => 0,
            },
            Self::NascarCup { .. } => match position {
                1 => 40,
                2 => 35,
                p if (3..=35).contains(&p) => (37 - p) as u32,
                _ => 1,
            },
            Self::Custom(pts) => {
                if position <= pts.len() {
                    pts[position - 1]
                } else {
                    0
                }
            }
        };

        let bonus = match self {
            Self::FiaStandard { fastest_lap_bonus: true } if has_fastest_lap && position <= 10 => 1,
            Self::NascarCup { stage_win_bonus: true } if has_fastest_lap => 10,
            _ => 0,
        };

        base_pts + bonus
    }
}

/// Standings entry for a driver in a series or tournament.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesStandingEntry {
    pub driver_id: String,
    pub driver_name: String,
    pub team_name: String,
    pub points: u32,
    pub wins: u32,
    pub podiums: u32,
    pub best_finish: usize,
    pub total_race_time: f32,
    #[serde(default)]
    pub ai_character: Option<String>,
    #[serde(default)]
    pub ai_style: Option<String>,
    #[serde(default)]
    pub ai_tier: Option<u8>,
}

pub type TournamentStandingEntry = SeriesStandingEntry;

impl SeriesStandingEntry {
    pub fn new(driver_id: impl Into<String>, driver_name: impl Into<String>, team_name: impl Into<String>) -> Self {
        Self {
            driver_id: driver_id.into(),
            driver_name: driver_name.into(),
            team_name: team_name.into(),
            points: 0,
            wins: 0,
            podiums: 0,
            best_finish: usize::MAX,
            total_race_time: 0.0,
            ai_character: None,
            ai_style: None,
            ai_tier: None,
        }
    }

    pub fn with_ai_character(mut self, ai_character: Option<String>) -> Self {
        self.ai_character = ai_character;
        self
    }

    pub fn with_ai_style_and_tier(mut self, ai_style: Option<String>, ai_tier: Option<u8>) -> Self {
        self.ai_style = ai_style;
        self.ai_tier = ai_tier;
        self
    }
}

/// Result of an individual driver in a single round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoundDriverResult {
    pub driver_id: String,
    pub driver_name: String,
    pub team_name: String,
    pub finish_position: usize,
    pub total_time: f32,
    pub best_lap: Option<f32>,
    pub points_awarded: u32,
    pub has_fastest_lap: bool,
}

/// Detailed results of a finished series round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesRoundResult {
    pub round_index: usize,
    pub track_id: String,
    pub track_title: String,
    pub results: Vec<RoundDriverResult>,
}

pub type ChampionshipRoundResult = SeriesRoundResult;

fn default_session_tier() -> u32 {
    1
}

/// Multi-round series / championship session manager.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesSession {
    pub name: String,
    pub point_system: PointSystem,
    pub track_ids: Vec<String>,
    pub laps_per_round: u32,
    pub current_round: usize,
    pub standings: Vec<SeriesStandingEntry>,
    pub history: Vec<SeriesRoundResult>,
    pub is_completed: bool,
    #[serde(default = "default_session_tier")]
    pub tier: u32,
    #[serde(default)]
    pub round_laps: Vec<Option<u32>>,
}

pub type ChampionshipSession = SeriesSession;

impl SeriesSession {
    pub fn new(
        name: impl Into<String>,
        point_system: PointSystem,
        track_ids: Vec<String>,
        laps_per_round: u32,
        initial_drivers: &[(&str, &str, &str)], // (id, name, team)
    ) -> Self {
        let standings = initial_drivers
            .iter()
            .map(|&(id, name, team)| SeriesStandingEntry::new(id, name, team))
            .collect();

        Self {
            name: name.into(),
            point_system,
            track_ids,
            laps_per_round,
            current_round: 0,
            standings,
            history: Vec::new(),
            is_completed: false,
            tier: 1,
            round_laps: Vec::new(),
        }
    }

    /// Creates a series session populated with a player and persistent career rivals.
    pub fn from_career_rivals(
        name: impl Into<String>,
        point_system: PointSystem,
        track_ids: Vec<String>,
        laps_per_round: u32,
        player_team: &str,
        rivals: &[CareerRivalEntry],
    ) -> Self {
        let mut standings = Vec::with_capacity(rivals.len() + 1);
        standings.push(SeriesStandingEntry::new("player", "Player", player_team));
        for rival in rivals {
            let team = format!("{} Racing", rival.driver_name.split_whitespace().last().unwrap_or("Rival"));
            let mut entry = SeriesStandingEntry::new(&rival.driver_id, &rival.driver_name, team);
            entry = entry.with_ai_style_and_tier(Some(rival.style.as_str().to_string()), Some(rival.tier.to_u8()));
            standings.push(entry);
        }
        Self {
            name: name.into(),
            point_system,
            track_ids,
            laps_per_round,
            current_round: 0,
            standings,
            history: Vec::new(),
            is_completed: false,
            tier: 1,
            round_laps: Vec::new(),
        }
    }

    /// Syncs series standings from a slice of career rivals and sets the current tier.
    pub fn update_from_career_rivals(&mut self, new_tier: u32, rivals: &[CareerRivalEntry]) {
        self.tier = new_tier;
        let player_entry = self.standings.iter().find(|s| s.driver_id == "player").cloned().unwrap_or_else(|| {
            SeriesStandingEntry::new("player", "Player", "Apex Racing")
        });
        let mut new_standings = vec![player_entry];
        for rival in rivals {
            let team = format!("{} Racing", rival.driver_name.split_whitespace().last().unwrap_or("Rival"));
            let mut entry = SeriesStandingEntry::new(&rival.driver_id, &rival.driver_name, team);
            entry = entry.with_ai_style_and_tier(Some(rival.style.as_str().to_string()), Some(rival.tier.to_u8()));
            new_standings.push(entry);
        }
        self.standings = new_standings;
    }

    /// Evolves the series standings using RosterEvolutionEngine upon tier unlock.
    pub fn trigger_roster_evolution(&mut self, new_tier: u32, seed: u64) -> RosterEvolutionReport {
        let current_rivals: Vec<CareerRivalEntry> = self
            .standings
            .iter()
            .filter(|s| s.driver_id != "player")
            .map(|s| {
                let style = s.ai_style.as_deref().map(DrivingStyle::from_str_lossy).unwrap_or(DrivingStyle::Balanced);
                let tier = s.ai_tier.map(DriverTier::from_u8).unwrap_or_else(|| DriverTier::from_u8(self.tier as u8));
                CareerRivalEntry {
                    driver_id: s.driver_id.clone(),
                    driver_name: s.driver_name.clone(),
                    style,
                    tier,
                }
            })
            .collect();

        let (new_roster, report) = RosterEvolutionEngine::evolve_roster(
            &current_rivals,
            DriverTier::from_u8(new_tier.clamp(1, 5) as u8),
            seed,
        );

        self.update_from_career_rivals(new_tier, &new_roster);
        report
    }

    /// Sets the motorsport category tier (1..=5) for this series or championship.
    pub fn with_tier(mut self, tier: u32) -> Self {
        self.tier = tier;
        self
    }

    /// Sets per-round lap counts for the championship schedule.
    pub fn with_round_laps(mut self, round_laps: Vec<Option<u32>>) -> Self {
        self.round_laps = round_laps;
        self
    }

    /// Returns the configured lap count for the current round, if specified.
    pub fn current_round_laps(&self) -> Option<u32> {
        self.round_laps.get(self.current_round).copied().flatten()
    }

    /// Returns true if this championship session is brand new (at round 0 with no finished rounds).
    pub fn is_new(&self) -> bool {
        self.current_round == 0 && self.history.is_empty()
    }

    /// Returns the drivers from `self.standings` who qualify to participate in a round with `max_slots` grid capacity.
    ///
    /// - The human player (`player`) always qualifies and takes 1 slot.
    /// - The remaining `max_slots - 1` slots are awarded to the highest-ranked non-player drivers in current standings.
    /// - Drivers who do not qualify are not permanently discarded from championship standings;
    ///   they simply do not participate in the round with fewer slots.
    pub fn qualified_drivers_for_round(&self, max_slots: usize) -> Vec<SeriesStandingEntry> {
        if max_slots == 0 {
            return Vec::new();
        }
        if self.standings.len() <= max_slots {
            return self.standings.clone();
        }

        let player = self.standings.iter().find(|s| s.driver_id == "player").cloned();
        let target_bots = if player.is_some() { max_slots.saturating_sub(1) } else { max_slots };

        let mut qualified: Vec<SeriesStandingEntry> = self
            .standings
            .iter()
            .filter(|s| s.driver_id != "player")
            .take(target_bots)
            .cloned()
            .collect();

        if let Some(p) = player {
            qualified.insert(0, p);
        }

        qualified
    }

    /// For a brand new championship (round 0 with no history), synchronizes the roster size
    /// to exactly match the starting grid slots of the circuit.
    ///
    /// If additional racers are needed, they are drawn from `fallback_drivers`.
    /// If fewer racers are needed, the roster is trimmed while always retaining the human player.
    pub fn sync_initial_grid_slots(
        &mut self,
        circuit_slots: usize,
        fallback_drivers: &[DriverCharacter],
    ) {
        if circuit_slots == 0 || !self.is_new() {
            return;
        }

        if self.standings.len() < circuit_slots {
            let needed = circuit_slots - self.standings.len();
            let mut added = 0;
            for character in fallback_drivers {
                if added >= needed {
                    break;
                }
                if character.id == "player" || self.standings.iter().any(|s| s.driver_id == character.id) {
                    continue;
                }
                let team = format!("{} Racing", character.name.split_whitespace().last().unwrap_or("Motorsport"));
                let mut entry = SeriesStandingEntry::new(character.id, character.name, team);
                entry = entry.with_ai_style_and_tier(
                    Some(character.style.as_str().to_string()),
                    Some(self.tier as u8),
                );
                self.standings.push(entry);
                added += 1;
            }
            if added < needed {
                for _ in added..needed {
                    let id = format!("driver_{}", self.standings.len() + 1);
                    let name = format!("Driver {}", self.standings.len() + 1);
                    let team = "Independent Racing".to_string();
                    let mut entry = SeriesStandingEntry::new(id, name, team);
                    entry = entry.with_ai_style_and_tier(None, Some(self.tier as u8));
                    self.standings.push(entry);
                }
            }
        } else if self.standings.len() > circuit_slots {
            let player_idx = self.standings.iter().position(|s| s.driver_id == "player");
            match player_idx {
                Some(p_idx) if p_idx < circuit_slots => {
                    self.standings.truncate(circuit_slots);
                }
                Some(p_idx) => {
                    let player_entry = self.standings.remove(p_idx);
                    self.standings.truncate(circuit_slots - 1);
                    self.standings.insert(0, player_entry);
                }
                None => {
                    self.standings.truncate(circuit_slots);
                }
            }
        }
    }

    pub fn current_track_id(&self) -> Option<&str> {
        self.track_ids.get(self.current_round).map(|s| s.as_str())
    }

    pub fn total_rounds(&self) -> usize {
        self.track_ids.len()
    }

    pub fn submit_round_results(&mut self, track_title: &str, mut results: Vec<RoundDriverResult>) {
        if self.current_round >= self.track_ids.len() {
            return;
        }

        let track_id = self.track_ids[self.current_round].clone();

        // Calculate points for each driver
        for res in &mut results {
            let pts = self.point_system.points_for_position(res.finish_position, res.has_fastest_lap);
            res.points_awarded = pts;

            if let Some(entry) = self.standings.iter_mut().find(|s| s.driver_id == res.driver_id) {
                entry.points += pts;
                entry.total_race_time += res.total_time;
                if res.finish_position == 1 {
                    entry.wins += 1;
                }
                if res.finish_position <= 3 {
                    entry.podiums += 1;
                }
                entry.best_finish = entry.best_finish.min(res.finish_position);
            }
        }

        // Sort standings by points (descending), then wins, then best finish, then total time
        self.sort_standings();

        self.history.push(SeriesRoundResult {
            round_index: self.current_round,
            track_id,
            track_title: track_title.to_string(),
            results,
        });

        self.current_round += 1;
        if self.current_round >= self.track_ids.len() {
            self.is_completed = true;
        }
    }

    /// Cancels and rolls back the results of the latest completed round,
    /// returning the track ID of that round so it can be re-run.
    pub fn cancel_latest_round(&mut self) -> Option<String> {
        let latest = self.history.pop()?;
        self.current_round = latest.round_index;
        self.is_completed = false;

        // Deduct points, time, wins, and podiums awarded in this round
        for res in &latest.results {
            if let Some(entry) = self.standings.iter_mut().find(|s| s.driver_id == res.driver_id) {
                entry.points = entry.points.saturating_sub(res.points_awarded);
                entry.total_race_time = (entry.total_race_time - res.total_time).max(0.0);
                if res.finish_position == 1 {
                    entry.wins = entry.wins.saturating_sub(1);
                }
                if res.finish_position <= 3 {
                    entry.podiums = entry.podiums.saturating_sub(1);
                }
            }
        }

        // Recompute best_finish for each driver from remaining history
        for entry in &mut self.standings {
            entry.best_finish = usize::MAX;
            for round in &self.history {
                if let Some(res) = round.results.iter().find(|r| r.driver_id == entry.driver_id) {
                    entry.best_finish = entry.best_finish.min(res.finish_position);
                }
            }
        }

        self.sort_standings();
        Some(latest.track_id)
    }

    pub fn sort_standings(&mut self) {
        self.standings.sort_by(|a, b| {
            b.points
                .cmp(&a.points)
                .then_with(|| b.wins.cmp(&a.wins))
                .then_with(|| a.best_finish.cmp(&b.best_finish))
                .then_with(|| a.total_race_time.partial_cmp(&b.total_race_time).unwrap_or(std::cmp::Ordering::Equal))
        });
    }

    pub fn leader(&self) -> Option<&SeriesStandingEntry> {
        self.standings.first()
    }
}

/// Qualifying hot-lap shootout session determining starting grid slots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualifyingResult {
    pub driver_id: String,
    pub driver_name: String,
    pub team_name: String,
    pub best_lap_time: f32,
    pub delta_to_pole: f32,
    pub grid_position: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualifyingSession {
    pub track_id: String,
    pub time_limit_seconds: f32,
    pub elapsed_time: f32,
    pub results: Vec<QualifyingResult>,
    pub is_completed: bool,
}

impl QualifyingSession {
    pub fn new(track_id: impl Into<String>, time_limit_seconds: f32) -> Self {
        Self {
            track_id: track_id.into(),
            time_limit_seconds,
            elapsed_time: 0.0,
            results: Vec::new(),
            is_completed: false,
        }
    }

    pub fn update_laptimes(&mut self, lap_times: &[(String, String, String, f32)]) {
        let mut sorted = lap_times.to_vec();
        sorted.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

        let pole_time = sorted.first().map(|s| s.3).unwrap_or(0.0);

        self.results = sorted
            .into_iter()
            .enumerate()
            .map(|(idx, (id, name, team, time))| QualifyingResult {
                driver_id: id,
                driver_name: name,
                team_name: team,
                best_lap_time: time,
                delta_to_pole: (time - pole_time).max(0.0),
                grid_position: idx + 1,
            })
            .collect();
    }
}

/// Stage Rally multi-stage time trial session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RallyStageResult {
    pub stage_index: usize,
    pub stage_name: String,
    pub driver_id: String,
    pub driver_name: String,
    pub raw_time: f32,
    pub penalty_seconds: f32,
    pub total_stage_time: f32,
    pub delta_to_stage_winner: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageRallySession {
    pub name: String,
    pub stage_track_ids: Vec<String>,
    pub current_stage: usize,
    pub stage_results: Vec<Vec<RallyStageResult>>,
    pub is_completed: bool,
}

impl StageRallySession {
    pub fn new(name: impl Into<String>, stage_track_ids: Vec<String>) -> Self {
        Self {
            name: name.into(),
            stage_track_ids,
            current_stage: 0,
            stage_results: Vec::new(),
            is_completed: false,
        }
    }
}

/// Sudden-Death Elimination tournament session (last car eliminated every N laps/seconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EliminationSession {
    pub elimination_lap_interval: u32,
    pub active_driver_ids: Vec<String>,
    pub eliminated_order: Vec<String>,
    pub current_target_lap: u32,
    pub is_completed: bool,
}

impl EliminationSession {
    pub fn new(driver_ids: Vec<String>, interval: u32) -> Self {
        Self {
            elimination_lap_interval: interval,
            active_driver_ids: driver_ids,
            eliminated_order: Vec::new(),
            current_target_lap: interval,
            is_completed: false,
        }
    }

    pub fn eliminate_last(&mut self, last_place_driver_id: &str) {
        if let Some(pos) = self.active_driver_ids.iter().position(|id| id == last_place_driver_id) {
            let id = self.active_driver_ids.remove(pos);
            self.eliminated_order.push(id);
        }
        self.current_target_lap += self.elimination_lap_interval;
        if self.active_driver_ids.len() <= 1 {
            self.is_completed = true;
        }
    }
}

/// High-level series / race format descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SeriesFormat {
    QuickRace { default_laps: u32, default_bots: usize },
    TimeAttack,
    Championship { name: String, point_system: PointSystem, track_ids: Vec<String>, laps_per_round: u32 },
    Series { name: String, point_system: PointSystem, track_ids: Vec<String>, laps_per_round: u32 },
    QualifyingShootout { time_limit: f32 },
    StageRally { name: String, stage_track_ids: Vec<String> },
    EliminationCup { elimination_interval: u32 },
}

pub type TournamentFormat = SeriesFormat;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fia_point_system() {
        let pts = PointSystem::FiaStandard { fastest_lap_bonus: true };
        assert_eq!(pts.points_for_position(1, false), 25);
        assert_eq!(pts.points_for_position(1, true), 26);
        assert_eq!(pts.points_for_position(10, true), 2);
        assert_eq!(pts.points_for_position(11, true), 0); // No bonus outside top 10
    }

    #[test]
    fn test_championship_standings() {
        let mut champ = ChampionshipSession::new(
            "GT World Challenge 2026",
            PointSystem::FiaStandard { fastest_lap_bonus: true },
            vec!["monza".to_string(), "spa".to_string()],
            5,
            &[
                ("max", "Max Hunter", "Red Bull"),
                ("lewis", "Lewis Hamilton", "Ferrari"),
            ],
        );

        champ.submit_round_results(
            "Monza Autodromo",
            vec![
                RoundDriverResult {
                    driver_id: "max".to_string(),
                    driver_name: "Max Hunter".to_string(),
                    team_name: "Red Bull".to_string(),
                    finish_position: 1,
                    total_time: 120.5,
                    best_lap: Some(24.1),
                    points_awarded: 25,
                    has_fastest_lap: true,
                },
                RoundDriverResult {
                    driver_id: "lewis".to_string(),
                    driver_name: "Lewis Hamilton".to_string(),
                    team_name: "Ferrari".to_string(),
                    finish_position: 2,
                    total_time: 122.0,
                    best_lap: Some(24.4),
                    points_awarded: 18,
                    has_fastest_lap: false,
                },
            ],
        );

        assert_eq!(champ.standings[0].driver_id, "max");
        assert_eq!(champ.standings[0].points, 26);
        assert_eq!(champ.standings[1].driver_id, "lewis");
        assert_eq!(champ.standings[1].points, 18);
        assert_eq!(champ.current_round, 1);
        assert!(!champ.is_completed);
    }

    #[test]
    fn test_nascar_point_system() {
        let pts = PointSystem::NascarCup { stage_win_bonus: true };
        assert_eq!(pts.points_for_position(1, false), 40);
        assert_eq!(pts.points_for_position(1, true), 50); // 40 win + 10 stage win bonus
        assert_eq!(pts.points_for_position(2, false), 35);
        assert_eq!(pts.points_for_position(3, false), 34);
        assert_eq!(pts.points_for_position(10, false), 27);
        assert_eq!(pts.points_for_position(35, false), 2);
        assert_eq!(pts.points_for_position(36, false), 1);
        assert_eq!(pts.points_for_position(40, false), 1);
    }
}
