//! Tournament sprint weekend (Spec 104): the knockout format of FIA Autocross and Rallycross.
//!
//! A weekend takes 32 drivers through 4 qualifying heats, 2 semifinals and a Grand Final plus a
//! B-Final, with at most 8 cars per race. The bracket is binary, so the same code runs 16 and 64
//! drivers (64: 8 heats, 4 quarterfinals, 2 semifinals, the finals).
//!
//! The player always races once per stage (Option A): a driver who misses the top 4 of a group
//! moves to a consolation race and ends in the B-Final. The other races of a stage are not driven,
//! so they are resolved here from the time the player's race produced, with a seeded model. The
//! module has no clock, no I/O and no random source except [`LcgRng`], so a weekend replays bit for
//! bit from the same seed and results.

use std::cmp::Ordering;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{RoundDriverResult, SeriesStandingEntry};
use crate::ai::LcgRng;

/// Cars in one race: dirt tracks are narrow, so a grid never goes past 8.
pub const GRID_SIZE: usize = 8;
/// Drivers each group sends to the next stage.
pub const ADVANCERS_PER_GROUP: usize = 4;
/// `total_time` of a driver who did not finish. It sorts after every real time.
pub const DNF_TIME: f32 = 9_999.0;
/// The 32-position scoring matrix of a weekend: Grand Final (1-8), B-Final (9-16), the rest (17-32).
pub const TOURNAMENT_POINTS_32: [u32; 32] = [
    25, 22, 20, 18, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 4, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1,
];

const REFERENCE_LAP_FALLBACK: f32 = 60.0;

/// How a round of a series is run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeekendFormat {
    /// One race per circuit round (GT, NASCAR, Karting).
    #[default]
    StandardSingleRace,
    /// Heats, semifinals and finals (Autocross, Rallycross).
    TournamentSprint,
}

/// A stage of a tournament weekend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TournamentStage {
    QualifyingHeat,
    /// Only in a 64-driver weekend.
    Quarterfinal,
    Semifinal,
    /// The race of a driver who missed the cut, between the cut and the B-Final.
    ConsolationSemifinal,
    ConsolationBFinal,
    GrandFinal,
}

impl TournamentStage {
    pub fn title(self) -> &'static str {
        match self {
            Self::QualifyingHeat => "QUALIFYING HEAT",
            Self::Quarterfinal => "QUARTERFINAL",
            Self::Semifinal => "SEMIFINAL",
            Self::ConsolationSemifinal => "CONSOLATION SEMI",
            Self::ConsolationBFinal => "B-FINAL",
            Self::GrandFinal => "GRAND FINAL",
        }
    }
}

/// Size and distances of a weekend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TournamentConfig {
    pub total_drivers: usize,
    pub heat_laps: u32,
    pub semi_laps: u32,
    pub final_laps: u32,
}

impl Default for TournamentConfig {
    fn default() -> Self {
        Self { total_drivers: 32, heat_laps: 4, semi_laps: 5, final_laps: 6 }
    }
}

impl TournamentConfig {
    /// Checks that the bracket is binary (16, 32 or 64 drivers) and every distance is positive.
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.total_drivers, 16 | 32 | 64) {
            return Err(format!("tournament total_drivers must be 16, 32 or 64 (got {})", self.total_drivers));
        }
        if self.heat_laps == 0 || self.semi_laps == 0 || self.final_laps == 0 {
            return Err("tournament heat_laps, semi_laps and final_laps must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn heat_count(&self) -> usize {
        self.total_drivers / GRID_SIZE
    }

    /// Rounds of the main bracket, the final included: 3 for 32 drivers.
    pub fn round_count(&self) -> usize {
        self.heat_count().trailing_zeros() as usize + 1
    }

    /// Races of the main bracket in `round`.
    pub fn groups_in_round(&self, round: usize) -> usize {
        self.heat_count() >> round
    }

    pub fn stage_for_round(&self, round: usize) -> TournamentStage {
        match self.groups_in_round(round) {
            _ if round == 0 => TournamentStage::QualifyingHeat,
            1 => TournamentStage::GrandFinal,
            2 => TournamentStage::Semifinal,
            _ => TournamentStage::Quarterfinal,
        }
    }

    pub fn laps_for(&self, stage: TournamentStage) -> u32 {
        match stage {
            TournamentStage::QualifyingHeat => self.heat_laps,
            TournamentStage::GrandFinal => self.final_laps,
            _ => self.semi_laps,
        }
    }

    fn final_round(&self) -> usize {
        self.round_count() - 1
    }
}

/// One scheduled race of a weekend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TournamentRace {
    pub stage: TournamentStage,
    /// Index of the race within its stage (heat 0 is "HEAT 1").
    pub group: usize,
    /// Races of this stage in the main bracket.
    pub groups: usize,
    pub laps: u32,
    /// Driver ids in grid order, pole first.
    pub driver_ids: Vec<String>,
}

impl TournamentRace {
    /// HUD badge: `HEAT 2/4`, `SEMIFINAL 1/2`, `B-FINAL`, `GRAND FINAL`.
    pub fn badge(&self) -> String {
        match self.stage {
            TournamentStage::QualifyingHeat | TournamentStage::Quarterfinal | TournamentStage::Semifinal => {
                format!("{} {}/{}", self.stage.title().replace("QUALIFYING ", ""), self.group + 1, self.groups)
            }
            other => other.title().to_string(),
        }
    }
}

/// The state of one weekend: schedule of the active stage and the results so far.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TournamentWeekendState {
    pub config: TournamentConfig,
    pub seed: u64,
    pub player_id: String,
    /// Stage of the race the player drives next.
    pub active_stage: TournamentStage,
    pub active_round: usize,
    /// Races of the active stage. A consolation race is scheduled only when the player is in it.
    pub active_races: Vec<TournamentRace>,
    /// Qualifying heats (8 drivers each).
    pub heat_results: Vec<Vec<RoundDriverResult>>,
    /// Quarterfinals of a 64-driver weekend.
    pub quarter_results: Vec<Vec<RoundDriverResult>>,
    /// Semifinals (8 drivers each).
    pub semi_results: Vec<Vec<RoundDriverResult>>,
    /// Consolation races the player drove, with the round they ran in.
    pub consolation_results: Vec<(usize, Vec<RoundDriverResult>)>,
    /// Consolation B-Final (positions 9-16).
    pub b_final_results: Option<Vec<RoundDriverResult>>,
    /// Grand Final (positions 1-8).
    pub grand_final_results: Option<Vec<RoundDriverResult>>,
    pub is_complete: bool,
}

fn fnv1a(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

/// Seed of the weekend of round `round_index` of the series `name`.
pub fn weekend_seed(name: &str, round_index: usize) -> u64 {
    fnv1a(name) ^ (round_index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn is_dnf(r: &RoundDriverResult) -> bool {
    r.total_time >= DNF_TIME
}

/// Orders by position, then by time.
fn by_position_then_time(a: &RoundDriverResult, b: &RoundDriverResult) -> Ordering {
    a.finish_position
        .cmp(&b.finish_position)
        .then_with(|| a.total_time.partial_cmp(&b.total_time).unwrap_or(Ordering::Equal))
        .then_with(|| a.driver_id.cmp(&b.driver_id))
}

/// Orders by elapsed time with non-finishers last: the grid order of the next stage.
fn by_elapsed_time(a: &RoundDriverResult, b: &RoundDriverResult) -> Ordering {
    is_dnf(a)
        .cmp(&is_dnf(b))
        .then_with(|| a.total_time.partial_cmp(&b.total_time).unwrap_or(Ordering::Equal))
        .then_with(|| a.finish_position.cmp(&b.finish_position))
        .then_with(|| a.driver_id.cmp(&b.driver_id))
}

fn renumber(results: &mut [RoundDriverResult]) {
    for (i, r) in results.iter_mut().enumerate() {
        r.finish_position = i + 1;
    }
}

impl TournamentWeekendState {
    /// Draws the heats and schedules stage 1. `entries` is the full field in standings order.
    pub fn new(config: TournamentConfig, entries: &[SeriesStandingEntry], seed: u64) -> Result<Self, String> {
        config.validate()?;
        if entries.len() != config.total_drivers {
            return Err(format!("a {}-driver weekend needs {} drivers (got {})", config.total_drivers, config.total_drivers, entries.len()));
        }
        let mut seen = HashSet::new();
        if let Some(dup) = entries.iter().find(|e| !seen.insert(e.driver_id.as_str())) {
            return Err(format!("duplicate driver id '{}' in the weekend field", dup.driver_id));
        }
        if !entries.iter().any(|e| e.driver_id == "player") {
            return Err("the weekend field must contain the player".to_string());
        }

        // Seeded draw: shuffle the field, deal it into heats, then seat each heat in standings order.
        let mut order: Vec<usize> = (0..entries.len()).collect();
        LcgRng::new(seed).shuffle(&mut order);
        let heats = (0..config.heat_count())
            .map(|h| {
                let mut members: Vec<usize> = order[h * GRID_SIZE..(h + 1) * GRID_SIZE].to_vec();
                members.sort_unstable();
                TournamentRace {
                    stage: TournamentStage::QualifyingHeat,
                    group: h,
                    groups: config.heat_count(),
                    laps: config.heat_laps,
                    driver_ids: members.into_iter().map(|i| entries[i].driver_id.clone()).collect(),
                }
            })
            .collect();

        let mut state = Self {
            config,
            seed,
            player_id: "player".to_string(),
            active_stage: TournamentStage::QualifyingHeat,
            active_round: 0,
            active_races: heats,
            heat_results: Vec::new(),
            quarter_results: Vec::new(),
            semi_results: Vec::new(),
            consolation_results: Vec::new(),
            b_final_results: None,
            grand_final_results: None,
            is_complete: false,
        };
        state.active_stage = state.player_race().map(|r| r.stage).unwrap_or(TournamentStage::QualifyingHeat);
        Ok(state)
    }

    /// The scheduled race the player drives next. `None` once the weekend is complete.
    pub fn player_race(&self) -> Option<&TournamentRace> {
        self.active_races.iter().find(|r| r.driver_ids.iter().any(|id| *id == self.player_id))
    }

    /// Results of the main-bracket races of `round`.
    pub fn round_results(&self, round: usize) -> &[Vec<RoundDriverResult>] {
        match self.config.stage_for_round(round) {
            TournamentStage::QualifyingHeat => &self.heat_results,
            TournamentStage::Quarterfinal => &self.quarter_results,
            TournamentStage::Semifinal => &self.semi_results,
            _ => &[],
        }
    }

    fn round_results_mut(&mut self, round: usize) -> &mut Vec<Vec<RoundDriverResult>> {
        match self.config.stage_for_round(round) {
            TournamentStage::QualifyingHeat => &mut self.heat_results,
            TournamentStage::Quarterfinal => &mut self.quarter_results,
            _ => &mut self.semi_results,
        }
    }

    /// True when the player missed the top 4 of a main-bracket race early enough to need consolation
    /// races: before the round that feeds the Grand Final and the B-Final.
    pub fn player_in_consolation(&self) -> bool {
        let last_before_b_final = self.config.final_round().saturating_sub(1);
        (0..last_before_b_final).any(|r| {
            self.round_results(r).iter().flatten().any(|x| x.driver_id == self.player_id && x.finish_position > ADVANCERS_PER_GROUP)
        })
    }

    /// Records the race the player drove, resolves the other races of the stage and schedules the next
    /// stage. Fails closed: nothing changes unless `results` is exactly the field of the player's race.
    pub fn submit_player_race(&mut self, results: Vec<RoundDriverResult>, entries: &[SeriesStandingEntry]) -> Result<(), String> {
        if self.is_complete {
            return Err("the weekend is complete".to_string());
        }
        let player_race = self.player_race().cloned().ok_or_else(|| "the player has no scheduled race".to_string())?;
        let player_results = Self::normalize(&player_race, results)?;

        // Reference pace: the winner of the race the player drove, per lap.
        let winner_time = player_results.iter().filter(|r| !is_dnf(r)).map(|r| r.total_time).fold(f32::MAX, f32::min);
        let reference_lap = if winner_time < DNF_TIME { winner_time / player_race.laps as f32 } else { REFERENCE_LAP_FALLBACK };

        let races = std::mem::take(&mut self.active_races);
        let round = self.active_round;
        let mut main_results: Vec<(usize, Vec<RoundDriverResult>)> = Vec::new();
        let mut final_round_results: Vec<(TournamentStage, Vec<RoundDriverResult>)> = Vec::new();
        for race in &races {
            let is_player_race = race.driver_ids.iter().any(|id| *id == self.player_id);
            let race_results = if is_player_race {
                player_results.clone()
            } else {
                Self::simulate(race, reference_lap, entries, self.seed ^ ((round as u64) << 32 | race.group as u64))
            };
            match race.stage {
                TournamentStage::ConsolationSemifinal => self.consolation_results.push((round, race_results)),
                TournamentStage::GrandFinal | TournamentStage::ConsolationBFinal => final_round_results.push((race.stage, race_results)),
                _ => main_results.push((race.group, race_results)),
            }
        }
        if !main_results.is_empty() {
            main_results.sort_by_key(|(group, _)| *group);
            *self.round_results_mut(round) = main_results.into_iter().map(|(_, r)| r).collect();
        }
        for (stage, race_results) in final_round_results {
            if stage == TournamentStage::GrandFinal {
                self.grand_final_results = Some(race_results);
            } else {
                self.b_final_results = Some(race_results);
            }
        }

        if round == self.config.final_round() {
            self.is_complete = true;
            return Ok(());
        }
        self.schedule_round(round + 1);
        Ok(())
    }

    /// Checks `results` against the race field and puts it in finishing order, non-finishers last.
    fn normalize(race: &TournamentRace, mut results: Vec<RoundDriverResult>) -> Result<Vec<RoundDriverResult>, String> {
        let field: HashSet<&str> = race.driver_ids.iter().map(|s| s.as_str()).collect();
        let got: HashSet<&str> = results.iter().map(|r| r.driver_id.as_str()).collect();
        if results.len() != race.driver_ids.len() || got != field {
            return Err(format!("{} results do not match the field of the race ({} drivers)", race.badge(), race.driver_ids.len()));
        }
        results.sort_by(|a, b| is_dnf(a).cmp(&is_dnf(b)).then_with(|| by_position_then_time(a, b)));
        renumber(&mut results);
        for r in &mut results {
            r.points_awarded = 0;
            r.has_fastest_lap = false;
        }
        Ok(results)
    }

    /// Resolves a race nobody drove: pace from the player's race, a tier edge and a seeded variation.
    fn simulate(race: &TournamentRace, reference_lap: f32, entries: &[SeriesStandingEntry], seed: u64) -> Vec<RoundDriverResult> {
        let mut results: Vec<RoundDriverResult> = race
            .driver_ids
            .iter()
            .map(|id| {
                let entry = entries.iter().find(|e| e.driver_id == *id);
                let tier = entry.and_then(|e| e.ai_tier).unwrap_or(1).clamp(1, 5) as f32;
                let mut rng = LcgRng::new(seed ^ fnv1a(id));
                let pace = 1.0 + 0.012 * (5.0 - tier) + (rng.next_f32() - 0.5) * 0.03;
                RoundDriverResult {
                    driver_id: id.clone(),
                    driver_name: entry.map(|e| e.driver_name.clone()).unwrap_or_else(|| id.clone()),
                    team_name: entry.map(|e| e.team_name.clone()).unwrap_or_default(),
                    finish_position: 0,
                    total_time: reference_lap * race.laps as f32 * pace,
                    best_lap: Some(reference_lap * pace * 0.98),
                    points_awarded: 0,
                    has_fastest_lap: false,
                }
            })
            .collect();
        results.sort_by(by_elapsed_time);
        renumber(&mut results);
        results
    }

    /// The drivers a group of `round` sends on, and the ones it does not, each in finishing order.
    fn split_round(&self, round: usize) -> (Vec<Vec<&RoundDriverResult>>, Vec<&RoundDriverResult>) {
        let mut advancing = Vec::new();
        let mut out = Vec::new();
        for group in self.round_results(round) {
            advancing.push(group.iter().filter(|r| r.finish_position <= ADVANCERS_PER_GROUP).collect());
            out.extend(group.iter().filter(|r| r.finish_position > ADVANCERS_PER_GROUP));
        }
        (advancing, out)
    }

    /// Schedules the races of `round` from the results of `round - 1`.
    fn schedule_round(&mut self, round: usize) {
        let config = self.config;
        let (advancing, mut out) = self.split_round(round - 1);
        out.sort_by(|a, b| by_elapsed_time(a, b));
        let groups = config.groups_in_round(round);
        let stage = config.stage_for_round(round);

        // Group j takes the advancers of groups j and j + groups: heats 1 and 3 meet in semifinal 1.
        let mut races: Vec<TournamentRace> = (0..groups)
            .map(|j| {
                let mut seeds: Vec<&RoundDriverResult> = advancing[j].iter().chain(advancing[j + groups].iter()).copied().collect();
                seeds.sort_by(|a, b| by_elapsed_time(a, b));
                TournamentRace {
                    stage,
                    group: j,
                    groups,
                    laps: config.laps_for(stage),
                    driver_ids: seeds.iter().map(|r| r.driver_id.clone()).collect(),
                }
            })
            .collect();

        let player_in_main = races.iter().any(|r| r.driver_ids.iter().any(|id| *id == self.player_id));
        if round == config.final_round() {
            // B-Final: the drivers who missed the Grand Final from the last semifinal round. A player who
            // came through consolation takes the place of the slowest of them and starts last.
            let mut field: Vec<String> = out.iter().map(|r| r.driver_id.clone()).collect();
            if !player_in_main && !field.iter().any(|id| *id == self.player_id) {
                field.pop();
                field.push(self.player_id.clone());
            }
            races.push(TournamentRace {
                stage: TournamentStage::ConsolationBFinal,
                group: 0,
                groups: 1,
                laps: config.semi_laps,
                driver_ids: field,
            });
        } else if !player_in_main {
            // The player is out of the bracket but keeps racing: the best 7 of the drivers who just missed
            // the cut line up with him.
            let mut field: Vec<(String, f32)> = out
                .iter()
                .filter(|r| r.driver_id != self.player_id)
                .take(GRID_SIZE - 1)
                .map(|r| (r.driver_id.clone(), r.total_time))
                .collect();
            let player_time = out.iter().find(|r| r.driver_id == self.player_id).map(|r| r.total_time).unwrap_or(f32::MAX);
            field.push((self.player_id.clone(), player_time));
            field.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal).then_with(|| a.0.cmp(&b.0)));
            races.push(TournamentRace {
                stage: TournamentStage::ConsolationSemifinal,
                group: 0,
                groups: 1,
                laps: config.semi_laps,
                driver_ids: field.into_iter().map(|(id, _)| id).collect(),
            });
        }
        self.active_round = round;
        self.active_races = races;
        self.active_stage = self.player_race().map(|r| r.stage).unwrap_or(stage);
    }

    /// The whole field in finishing order, 1st to last, once both finals are run: the Grand Final, the
    /// B-Final, then the drivers each earlier stage dropped, the latest stage first. Points are not set.
    pub fn final_standings(&self) -> Option<Vec<RoundDriverResult>> {
        if !self.is_complete {
            return None;
        }
        let mut ranked: Vec<RoundDriverResult> = Vec::with_capacity(self.config.total_drivers);
        let mut seen: HashSet<String> = HashSet::new();
        for finals in [self.grand_final_results.as_ref()?, self.b_final_results.as_ref()?] {
            for r in finals {
                if seen.insert(r.driver_id.clone()) {
                    ranked.push(r.clone());
                }
            }
        }
        for round in (0..self.config.final_round()).rev() {
            let consolation = self.consolation_results.iter().find(|(r, _)| *r == round + 1).map(|(_, res)| res);
            let (_, mut out) = self.split_round(round);
            out.retain(|r| !seen.contains(&r.driver_id));
            out.sort_by(|a, b| {
                let pos = |r: &RoundDriverResult| {
                    consolation
                        .and_then(|c| c.iter().find(|x| x.driver_id == r.driver_id))
                        .map(|x| x.finish_position)
                        .unwrap_or(usize::MAX)
                };
                pos(a).cmp(&pos(b)).then_with(|| by_position_then_time(a, b))
            });
            for r in out {
                seen.insert(r.driver_id.clone());
                ranked.push(r.clone());
            }
        }
        renumber(&mut ranked);
        Some(ranked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_runs_from_25_to_1_over_32_places() {
        assert_eq!(TOURNAMENT_POINTS_32.len(), 32);
        assert_eq!((TOURNAMENT_POINTS_32[0], TOURNAMENT_POINTS_32[31]), (25, 1));
        assert!(TOURNAMENT_POINTS_32.windows(2).all(|w| w[0] >= w[1]));
    }

    #[test]
    fn config_sizes_the_bracket() {
        let c32 = TournamentConfig::default();
        assert_eq!((c32.heat_count(), c32.round_count()), (4, 3));
        assert_eq!(c32.stage_for_round(0), TournamentStage::QualifyingHeat);
        assert_eq!(c32.stage_for_round(1), TournamentStage::Semifinal);
        assert_eq!(c32.stage_for_round(2), TournamentStage::GrandFinal);
        let c64 = TournamentConfig { total_drivers: 64, ..c32 };
        assert_eq!((c64.heat_count(), c64.round_count()), (8, 4));
        assert_eq!(c64.stage_for_round(1), TournamentStage::Quarterfinal);
        assert!(TournamentConfig { total_drivers: 24, ..c32 }.validate().is_err());
    }
}
