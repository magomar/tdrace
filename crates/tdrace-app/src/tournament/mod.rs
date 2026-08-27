use serde::{Deserialize, Serialize};

/// Scoring system for championship tournaments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointSystem {
    /// Official FIA Formula 1 scoring: 25, 18, 15, 12, 10, 8, 6, 4, 2, 1 (plus optional fastest lap bonus)
    F1Standard { fastest_lap_bonus: bool },
    /// MotoGP scoring: 25, 20, 16, 13, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1
    MotoGp,
    /// Classic arcade 6-place scoring: 10, 6, 4, 3, 2, 1
    ClassicArcade,
    /// Custom points matrix
    Custom(Vec<u32>),
}

impl PointSystem {
    pub fn points_for_position(&self, position: usize, has_fastest_lap: bool) -> u32 {
        if position == 0 {
            return 0;
        }
        let base_pts = match self {
            Self::F1Standard { .. } => match position {
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
            Self::Custom(pts) => {
                if position <= pts.len() {
                    pts[position - 1]
                } else {
                    0
                }
            }
        };

        let bonus = match self {
            Self::F1Standard { fastest_lap_bonus: true } if has_fastest_lap && position <= 10 => 1,
            _ => 0,
        };

        base_pts + bonus
    }
}

/// Standings entry for a driver in a tournament.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TournamentStandingEntry {
    pub driver_id: String,
    pub driver_name: String,
    pub team_name: String,
    pub points: u32,
    pub wins: u32,
    pub podiums: u32,
    pub best_finish: usize,
    pub total_race_time: f32,
}

impl TournamentStandingEntry {
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
        }
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

/// Detailed results of a finished championship round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChampionshipRoundResult {
    pub round_index: usize,
    pub track_id: String,
    pub track_title: String,
    pub results: Vec<RoundDriverResult>,
}

/// Multi-round championship tournament manager.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChampionshipSession {
    pub name: String,
    pub point_system: PointSystem,
    pub track_ids: Vec<String>,
    pub laps_per_round: u32,
    pub current_round: usize,
    pub standings: Vec<TournamentStandingEntry>,
    pub history: Vec<ChampionshipRoundResult>,
    pub is_completed: bool,
}

impl ChampionshipSession {
    pub fn new(
        name: impl Into<String>,
        point_system: PointSystem,
        track_ids: Vec<String>,
        laps_per_round: u32,
        initial_drivers: &[(&str, &str, &str)], // (id, name, team)
    ) -> Self {
        let standings = initial_drivers
            .iter()
            .map(|&(id, name, team)| TournamentStandingEntry::new(id, name, team))
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

        self.history.push(ChampionshipRoundResult {
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

    pub fn sort_standings(&mut self) {
        self.standings.sort_by(|a, b| {
            b.points
                .cmp(&a.points)
                .then_with(|| b.wins.cmp(&a.wins))
                .then_with(|| a.best_finish.cmp(&b.best_finish))
                .then_with(|| a.total_race_time.partial_cmp(&b.total_race_time).unwrap_or(std::cmp::Ordering::Equal))
        });
    }

    pub fn leader(&self) -> Option<&TournamentStandingEntry> {
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

/// High-level tournament format descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TournamentFormat {
    QuickRace { default_laps: u32, default_bots: usize },
    TimeAttack,
    Championship { name: String, point_system: PointSystem, track_ids: Vec<String>, laps_per_round: u32 },
    QualifyingShootout { time_limit: f32 },
    StageRally { name: String, stage_track_ids: Vec<String> },
    EliminationCup { elimination_interval: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f1_point_system() {
        let pts = PointSystem::F1Standard { fastest_lap_bonus: true };
        assert_eq!(pts.points_for_position(1, false), 25);
        assert_eq!(pts.points_for_position(1, true), 26);
        assert_eq!(pts.points_for_position(10, true), 2);
        assert_eq!(pts.points_for_position(11, true), 0); // No bonus outside top 10
    }

    #[test]
    fn test_championship_standings() {
        let mut champ = ChampionshipSession::new(
            "Formula 1 2026",
            PointSystem::F1Standard { fastest_lap_bonus: true },
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
}
