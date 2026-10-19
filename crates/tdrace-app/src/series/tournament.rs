//! Tournament sprint weekend (Spec 104, amended by Spec 128 for World RX): the knockout format of
//! FIA Autocross (32 drivers, 8-car 5-3 grids) and FIA World Rallycross (24 drivers, 6-car 2-2-2
//! grids, Stage 0 SuperPole, configurable Q1–Q4 Classificatory Heats ranked by overall total time,
//! Odd/Even Semifinals, Row-by-Row Final grid pairing, and Three-Stage cumulative scoring).
//!
//! The player always races once per stage (Option A): a driver who misses the cut moves to a
//! consolation race and ends in the B-Final. The other races of a stage are not driven, so they are
//! resolved here from the time the player's race produced, with a seeded model. The module has no
//! clock, no I/O and no random source except [`LcgRng`], so a weekend replays bit for bit from the
//! same seed and results.

use std::cmp::Ordering;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{RoundDriverResult, SeriesStandingEntry};
use crate::ai::LcgRng;

/// Cars in one Autocross race: 8-car 5-3 grid.
pub const GRID_SIZE: usize = 8;
/// Drivers each Autocross group sends to the next stage.
pub const ADVANCERS_PER_GROUP: usize = 4;
/// Cars in one World RX race: 6-car 2-2-2 staggered grid (Spec 128).
pub const RX_GRID_SIZE: usize = 6;
/// Drivers each World RX semifinal sends to the Grand Final (Spec 128).
pub const RX_ADVANCERS_PER_GROUP: usize = 3;
/// Top drivers in the World RX Intermediate Classification who advance to the Semifinals (Spec 128).
pub const RX_INTERMEDIATE_CUTOFF: usize = 12;
/// Cars in one FIA European Autocross race: 10-car 3-2-3-2 grid (Spec 130).
pub const AX_GRID_SIZE: usize = 10;
/// Drivers each FIA European Autocross semifinal sends to the Grand Final (Spec 130).
pub const AX_ADVANCERS_PER_GROUP: usize = 5;
/// Top drivers in the FIA European Autocross Intermediate Classification who advance to the Semifinals (Spec 130).
pub const AX_INTERMEDIATE_CUTOFF: usize = 20;
/// Number of laps in a World RX SuperPole / AX Timed Practice session: 1 out-lap from launch chute + 1 timed flying lap (Spec 128, 130).
pub const SUPERPOLE_LAPS: u32 = 2;
/// `total_time` of a driver who did not finish. It sorts after every real time.
pub const DNF_TIME: f32 = 9_999.0;
/// The 32-position scoring matrix of an Autocross weekend: Grand Final (1-8), B-Final (9-16), the rest (17-32).
pub const TOURNAMENT_POINTS_32: [u32; 32] = [
    25, 22, 20, 18, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 4, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1,
];
/// World RX Stage 1 (Intermediate Classification) points for ranks 1st through 12th (Spec 128).
pub const WORLD_RX_INTERMEDIATE_POINTS_12: [u32; 12] = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5];
/// World RX Stage 2 (Semifinal 1 & Semifinal 2) points for places 1st through 6th in each Semi (Spec 128).
pub const WORLD_RX_SEMI_POINTS_6: [u32; 6] = [6, 5, 4, 3, 2, 1];
/// World RX Stage 3 (Grand Final) points for places 1st through 6th (Spec 128).
pub const WORLD_RX_FINAL_POINTS_6: [u32; 6] = [8, 5, 4, 3, 2, 1];
/// FIA Autocross Stage 1 (Intermediate Classification) points for ranks 1st through 10th (Spec 130).
pub const FIA_AX_INTERMEDIATE_POINTS_10: [u32; 10] = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
/// FIA Autocross Stage 2 (Grand Final) points for places 1st through 10th (Spec 130).
pub const FIA_AX_FINAL_POINTS_10: [u32; 10] = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1];

const REFERENCE_LAP_FALLBACK: f32 = 60.0;
const SUPERPOLE_REFERENCE_LAP_DEFAULT: f32 = 40.0;
const SUPERPOLE_RNG_SALT: u64 = 0x5550_4F4C_4500_0000;

fn default_qualifying_rounds() -> usize {
    1
}

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

impl WeekendFormat {
    /// For `skip_serializing_if`: a single-race series keeps its TOML free of the weekend keys.
    pub fn is_single_race(&self) -> bool {
        *self == Self::StandardSingleRace
    }
}

/// A stage of a tournament weekend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TournamentStage {
    /// Solo timed flying lap from the launch chute (1 out-lap + 1 flying lap) determining Q1 grids in World RX (Spec 128).
    SuperPole,
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
            Self::SuperPole => "SUPERPOLE",
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
    /// Number of Qualifying Heat rounds (`Q1..=Qn`, default `1`, valid range `1..=4`, Spec 128).
    #[serde(default = "default_qualifying_rounds")]
    pub qualifying_rounds: usize,
}

impl Default for TournamentConfig {
    fn default() -> Self {
        Self {
            total_drivers: 32,
            heat_laps: 3,
            semi_laps: 4,
            final_laps: 5,
            qualifying_rounds: 1,
        }
    }
}

impl TournamentConfig {
    /// Default 24-driver, 6-car World RX weekend configuration (Spec 128).
    pub fn world_rx_default() -> Self {
        Self {
            total_drivers: 24,
            heat_laps: 3,
            semi_laps: 4,
            final_laps: 5,
            qualifying_rounds: 1,
        }
    }

    /// Default 30-driver, 10-car FIA European Autocross weekend configuration (Spec 130).
    pub fn autocross_default() -> Self {
        Self {
            total_drivers: 30,
            heat_laps: 4,
            semi_laps: 5,
            final_laps: 6,
            qualifying_rounds: 1,
        }
    }

    /// True when this weekend follows the 24-driver, 6-car FIA World RX format (Spec 128).
    pub fn is_world_rx(&self) -> bool {
        self.total_drivers == 24
    }

    /// True when this weekend follows the 30-driver, 10-car FIA European Autocross format (Spec 130).
    pub fn is_autocross_30(&self) -> bool {
        self.total_drivers == 30
    }

    /// Cars per race grid (`6` for World RX, `10` for Autocross 30, `8` for legacy Autocross).
    pub fn grid_size(&self) -> usize {
        if self.is_world_rx() {
            RX_GRID_SIZE
        } else if self.is_autocross_30() {
            AX_GRID_SIZE
        } else {
            GRID_SIZE
        }
    }

    /// Drivers advancing per group (`3` for World RX, `5` for Autocross 30, `4` for legacy Autocross).
    pub fn advancers_per_group(&self) -> usize {
        if self.is_world_rx() {
            RX_ADVANCERS_PER_GROUP
        } else if self.is_autocross_30() {
            AX_ADVANCERS_PER_GROUP
        } else {
            ADVANCERS_PER_GROUP
        }
    }

    /// True when the weekend begins with Stage 0 (`SuperPole` / Timed Qualifying Practice solo timed flying lap).
    pub fn has_superpole(&self) -> bool {
        self.is_world_rx() || self.is_autocross_30()
    }

    /// Checks that `total_drivers` is 16, 24, 30, 32 or 64, `qualifying_rounds` is 1..=4, and every distance is positive.
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.total_drivers, 16 | 24 | 30 | 32 | 64) {
            return Err(format!("tournament total_drivers must be 16, 24, 30, 32 or 64 (got {})", self.total_drivers));
        }
        if !(1..=4).contains(&self.qualifying_rounds) {
            return Err(format!("tournament qualifying_rounds must be 1..=4 (got {})", self.qualifying_rounds));
        }
        if self.heat_laps == 0 || self.semi_laps == 0 || self.final_laps == 0 {
            return Err("tournament heat_laps, semi_laps and final_laps must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn heat_count(&self) -> usize {
        self.total_drivers / self.grid_size()
    }

    /// Rounds of the main bracket, the final included: 3 for 24, 30 or 32 drivers.
    pub fn round_count(&self) -> usize {
        if self.is_autocross_30() {
            3
        } else {
            self.heat_count().trailing_zeros() as usize + 1
        }
    }

    /// Races of the main bracket in `round`.
    pub fn groups_in_round(&self, round: usize) -> usize {
        if self.is_autocross_30() {
            match round {
                0 => 3,
                1 => 2,
                _ => 1,
            }
        } else {
            self.heat_count() >> round
        }
    }

    pub fn stage_for_round(&self, round: usize) -> TournamentStage {
        if round == 0 {
            TournamentStage::QualifyingHeat
        } else if round == self.final_round() {
            TournamentStage::GrandFinal
        } else if self.groups_in_round(round) == 2 {
            TournamentStage::Semifinal
        } else {
            TournamentStage::Quarterfinal
        }
    }

    pub fn laps_for(&self, stage: TournamentStage) -> u32 {
        match stage {
            TournamentStage::SuperPole => 2,
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
    /// HUD badge: `SUPERPOLE`, `HEAT 2/4`, `SEMIFINAL 1/2`, `B-FINAL`, `GRAND FINAL`.
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
    /// Active Qualifying Heat round index (`0..config.qualifying_rounds`, Spec 128).
    #[serde(default)]
    pub current_qualifying_round: usize,
    /// Races of the active stage. A consolation race is scheduled only when the player is in it.
    pub active_races: Vec<TournamentRace>,
    /// Stage 0 SuperPole flying-lap classification (`P1–P24`, Spec 128).
    #[serde(default)]
    pub superpole_results: Vec<RoundDriverResult>,
    /// Optional circuit-specific reference flying lap time (seconds) for SuperPole AI simulation.
    #[serde(default)]
    pub superpole_reference_lap: Option<f32>,
    /// Qualifying heats of the latest completed Q round (`6` drivers each in RX, `8` in AX).
    pub heat_results: Vec<Vec<RoundDriverResult>>,
    /// All completed Qualifying Heat rounds (`Q1..=Qn`, Spec 128).
    #[serde(default)]
    pub qualifying_round_results: Vec<Vec<Vec<RoundDriverResult>>>,
    /// Overall Intermediate Classification across all completed Qualifying Heat rounds (`Q1..=Qn`, Spec 128).
    #[serde(default)]
    pub intermediate_standings: Vec<RoundDriverResult>,
    /// Quarterfinals of a 64-driver weekend.
    pub quarter_results: Vec<Vec<RoundDriverResult>>,
    /// Semifinals (`6` drivers each in RX, `8` in AX).
    pub semi_results: Vec<Vec<RoundDriverResult>>,
    /// Consolation races the player drove, with the round they ran in.
    pub consolation_results: Vec<(usize, Vec<RoundDriverResult>)>,
    /// Consolation B-Final (`positions 7-12` in RX, `9-16` in AX).
    pub b_final_results: Option<Vec<RoundDriverResult>>,
    /// Grand Final (`positions 1-6` in RX, `1-8` in AX).
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

/// Snake-seeds `ranked_ids` (`P1..=P24`) across `heat_count` Qualifying Heats:
/// Row 0 goes `0..heat_count`, Row 1 goes `(0..heat_count).rev()`, etc.
pub fn snake_seed_heats(ranked_ids: &[String], heat_count: usize, laps: u32) -> Vec<TournamentRace> {
    let mut heat_drivers: Vec<Vec<String>> = vec![Vec::new(); heat_count];
    for (idx, id) in ranked_ids.iter().enumerate() {
        let row = idx / heat_count;
        let col = idx % heat_count;
        let heat = if row % 2 == 0 { col } else { heat_count - 1 - col };
        heat_drivers[heat].push(id.clone());
    }
    heat_drivers
        .into_iter()
        .enumerate()
        .map(|(h, driver_ids)| TournamentRace {
            stage: TournamentStage::QualifyingHeat,
            group: h,
            groups: heat_count,
            laps,
            driver_ids,
        })
        .collect()
}

impl TournamentWeekendState {
    /// Draws the opening stage of the weekend (`SuperPole` for World RX, `QualifyingHeat` for Autocross).
    /// `entries` is the full field in standings order.
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

        if config.has_superpole() {
            let superpole_race = TournamentRace {
                stage: TournamentStage::SuperPole,
                group: 0,
                groups: 1,
                laps: config.laps_for(TournamentStage::SuperPole),
                driver_ids: vec!["player".to_string()],
            };
            return Ok(Self {
                config,
                seed,
                player_id: "player".to_string(),
                active_stage: TournamentStage::SuperPole,
                active_round: 0,
                current_qualifying_round: 0,
                active_races: vec![superpole_race],
                superpole_results: Vec::new(),
                superpole_reference_lap: None,
                heat_results: Vec::new(),
                qualifying_round_results: Vec::new(),
                intermediate_standings: Vec::new(),
                quarter_results: Vec::new(),
                semi_results: Vec::new(),
                consolation_results: Vec::new(),
                b_final_results: None,
                grand_final_results: None,
                is_complete: false,
            });
        }

        // Seeded draw for Autocross: shuffle the field, deal it into heats, then seat each heat in standings order.
        let grid_size = config.grid_size();
        let mut order: Vec<usize> = (0..entries.len()).collect();
        LcgRng::new(seed).shuffle(&mut order);
        let heats = (0..config.heat_count())
            .map(|h| {
                let mut members: Vec<usize> = order[h * grid_size..(h + 1) * grid_size].to_vec();
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
            current_qualifying_round: 0,
            active_races: heats,
            superpole_results: Vec::new(),
            superpole_reference_lap: None,
            heat_results: Vec::new(),
            qualifying_round_results: Vec::new(),
            intermediate_standings: Vec::new(),
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
        self.active_races.iter().find(|r| r.driver_ids.contains(&self.player_id))
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

    /// True when the player missed the cut of a main-bracket stage early enough to need consolation
    /// races: before the round that feeds the Grand Final and the B-Final.
    pub fn player_in_consolation(&self) -> bool {
        if self.config.is_world_rx() {
            return self.active_round >= 1
                && self
                    .intermediate_standings
                    .iter()
                    .any(|r| r.driver_id == self.player_id && r.finish_position > RX_INTERMEDIATE_CUTOFF);
        }
        if self.config.is_autocross_30() {
            return self.active_round >= 1
                && self
                    .intermediate_standings
                    .iter()
                    .any(|r| r.driver_id == self.player_id && r.finish_position > AX_INTERMEDIATE_CUTOFF);
        }
        let advancers = self.config.advancers_per_group();
        let last_before_b_final = self.config.final_round().saturating_sub(1);
        (0..last_before_b_final).any(|r| {
            self.round_results(r)
                .iter()
                .flatten()
                .any(|x| x.driver_id == self.player_id && x.finish_position > advancers)
        })
    }

    /// Returns `(intermediate_pts, semi_pts, final_pts)` earned by `driver_id` in a World RX (Three-Stage)
    /// or FIA Autocross 30 (Two-Phase: intermediate + final) weekend.
    pub fn driver_stage_points(&self, driver_id: &str) -> (u32, u32, u32) {
        if self.config.is_world_rx() {
            let inter_pts = self
                .intermediate_standings
                .iter()
                .position(|r| r.driver_id == driver_id)
                .and_then(|idx| WORLD_RX_INTERMEDIATE_POINTS_12.get(idx).copied())
                .unwrap_or(0);
            let semi_pts = self
                .semi_results
                .iter()
                .find_map(|semi| {
                    semi.iter()
                        .position(|r| r.driver_id == driver_id)
                        .and_then(|idx| WORLD_RX_SEMI_POINTS_6.get(idx).copied())
                })
                .unwrap_or(0);
            let final_pts = self
                .grand_final_results
                .as_ref()
                .and_then(|gf| {
                    gf.iter()
                        .position(|r| r.driver_id == driver_id)
                        .and_then(|idx| WORLD_RX_FINAL_POINTS_6.get(idx).copied())
                })
                .unwrap_or(0);
            (inter_pts, semi_pts, final_pts)
        } else if self.config.is_autocross_30() {
            let inter_pts = self
                .intermediate_standings
                .iter()
                .position(|r| r.driver_id == driver_id)
                .and_then(|idx| FIA_AX_INTERMEDIATE_POINTS_10.get(idx).copied())
                .unwrap_or(0);
            let final_pts = self
                .grand_final_results
                .as_ref()
                .and_then(|gf| {
                    gf.iter()
                        .position(|r| r.driver_id == driver_id)
                        .and_then(|idx| FIA_AX_FINAL_POINTS_10.get(idx).copied())
                })
                .unwrap_or(0);
            (inter_pts, 0, final_pts)
        } else {
            (0, 0, 0)
        }
    }

    /// Returns the cumulative championship points earned by `driver_id` in a World RX or Autocross 30 weekend.
    pub fn driver_total_weekend_points(&self, driver_id: &str) -> u32 {
        let (q, s, f) = self.driver_stage_points(driver_id);
        q + s + f
    }

    /// Records the race the player drove, resolves the other races of the stage and schedules the next
    /// stage. Fails closed: nothing changes unless `results` is valid for the player's active race.
    pub fn submit_player_race(&mut self, results: Vec<RoundDriverResult>, entries: &[SeriesStandingEntry]) -> Result<(), String> {
        if self.is_complete {
            return Err("the weekend is complete".to_string());
        }
        let player_race = self.player_race().cloned().ok_or_else(|| "the player has no scheduled race".to_string())?;

        if player_race.stage == TournamentStage::SuperPole {
            return self.submit_superpole(results, entries);
        }

        let player_results = Self::normalize(&player_race, results)?;

        // Reference pace: in World RX and Autocross 30, anchor other heats to the fastest AI in the player's race so
        // beating (or trailing) the AI in your own heat translates directly to the overall time sheet.
        let winner_time = if self.config.is_world_rx() || self.config.is_autocross_30() {
            player_results
                .iter()
                .filter(|r| r.driver_id != self.player_id && !is_dnf(r))
                .map(|r| r.total_time * 1.015)
                .fold(f32::MAX, f32::min)
        } else {
            player_results.iter().filter(|r| !is_dnf(r)).map(|r| r.total_time).fold(f32::MAX, f32::min)
        };
        let reference_lap = if winner_time < DNF_TIME { winner_time / player_race.laps as f32 } else { REFERENCE_LAP_FALLBACK };

        let races = std::mem::take(&mut self.active_races);
        let round = self.active_round;
        let q_round = self.current_qualifying_round;
        let mut main_results: Vec<(usize, Vec<RoundDriverResult>)> = Vec::new();
        let mut final_round_results: Vec<(TournamentStage, Vec<RoundDriverResult>)> = Vec::new();
        for race in &races {
            let is_player_race = race.driver_ids.contains(&self.player_id);
            let race_results = if is_player_race {
                player_results.clone()
            } else {
                let race_seed = self.seed ^ (((round as u64) << 32) | ((q_round as u64) << 16) | (race.group as u64));
                Self::simulate(race, reference_lap, entries, race_seed)
            };
            match race.stage {
                TournamentStage::ConsolationSemifinal => self.consolation_results.push((round, race_results)),
                TournamentStage::GrandFinal | TournamentStage::ConsolationBFinal => final_round_results.push((race.stage, race_results)),
                _ => main_results.push((race.group, race_results)),
            }
        }
        if !main_results.is_empty() {
            main_results.sort_by_key(|(group, _)| *group);
            let ordered_groups: Vec<Vec<RoundDriverResult>> = main_results.into_iter().map(|(_, r)| r).collect();
            if round == 0 {
                self.qualifying_round_results.push(ordered_groups.clone());
                self.heat_results = ordered_groups;
                self.recompute_intermediate_standings(entries);

                if self.current_qualifying_round + 1 < self.config.qualifying_rounds {
                    self.current_qualifying_round += 1;
                    // Rank drivers by the just-completed Q round's overall completion time and snake-seed the next Q round.
                    let mut latest_q: Vec<RoundDriverResult> = self.heat_results.iter().flatten().cloned().collect();
                    latest_q.sort_by(by_elapsed_time);
                    let ranked_ids: Vec<String> = latest_q.into_iter().map(|r| r.driver_id).collect();
                    self.active_races = snake_seed_heats(&ranked_ids, self.config.heat_count(), self.config.heat_laps);
                    self.active_stage = TournamentStage::QualifyingHeat;
                    return Ok(());
                }
            } else {
                *self.round_results_mut(round) = ordered_groups;
            }
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

    /// Resolves Stage 0 (`SuperPole` / Timed Qualifying Practice) from either the solo player flying-lap result (`len == 1`)
    /// or an explicit full-field classification (`len == total_drivers`), then snake-seeds Qualifying Round 1 (`Q1`).
    fn submit_superpole(&mut self, results: Vec<RoundDriverResult>, entries: &[SeriesStandingEntry]) -> Result<(), String> {
        let mut classification: Vec<RoundDriverResult> = if results.len() == 1 && results[0].driver_id == self.player_id {
            let mut player_res = results.into_iter().next().unwrap();
            let player_entry = entries.iter().find(|e| e.driver_id == self.player_id);
            if player_res.driver_name.trim().is_empty() {
                player_res.driver_name = player_entry.map(|e| e.driver_name.clone()).unwrap_or_else(|| self.player_id.clone());
            }
            if player_res.team_name.trim().is_empty() {
                player_res.team_name = player_entry.map(|e| e.team_name.clone()).unwrap_or_default();
            }
            // Prefer the single flying lap time (`best_lap`) when present and valid.
            if let Some(flying) = player_res.best_lap.filter(|&t| t > 0.0 && t < DNF_TIME) {
                player_res.total_time = flying;
            } else {
                player_res.best_lap = (player_res.total_time < DNF_TIME).then_some(player_res.total_time);
            }
            player_res.points_awarded = 0;
            player_res.has_fastest_lap = false;

            let ref_lap = self.superpole_reference_lap.unwrap_or(SUPERPOLE_REFERENCE_LAP_DEFAULT);
            let mut all = Vec::with_capacity(entries.len());
            all.push(player_res);
            for entry in entries.iter().filter(|e| e.driver_id != self.player_id) {
                let tier = entry.ai_tier.unwrap_or(1).clamp(1, 5) as f32;
                let mut rng = LcgRng::new(self.seed ^ SUPERPOLE_RNG_SALT ^ fnv1a(&entry.driver_id));
                let pace = 1.0 + 0.012 * (5.0 - tier) + (rng.next_f32() - 0.5) * 0.03;
                let flying_time = ref_lap * pace;
                all.push(RoundDriverResult {
                    driver_id: entry.driver_id.clone(),
                    driver_name: entry.driver_name.clone(),
                    team_name: entry.team_name.clone(),
                    finish_position: 0,
                    total_time: flying_time,
                    best_lap: Some(flying_time),
                    points_awarded: 0,
                    has_fastest_lap: false,
                });
            }
            all.sort_by(by_elapsed_time);
            all
        } else if results.len() == self.config.total_drivers {
            let expected: HashSet<&str> = entries.iter().map(|e| e.driver_id.as_str()).collect();
            let got: HashSet<&str> = results.iter().map(|r| r.driver_id.as_str()).collect();
            if got != expected {
                return Err(format!("SUPERPOLE results do not match the {}-driver field", self.config.total_drivers));
            }
            let mut all = results;
            all.sort_by(by_elapsed_time);
            for r in &mut all {
                r.points_awarded = 0;
                r.has_fastest_lap = false;
            }
            all
        } else {
            return Err(format!(
                "SUPERPOLE expects 1 solo player result or {} full-field results (got {})",
                self.config.total_drivers,
                results.len()
            ));
        };

        renumber(&mut classification);
        let ranked_ids: Vec<String> = classification.iter().map(|r| r.driver_id.clone()).collect();
        self.superpole_results = classification;
        self.current_qualifying_round = 0;
        self.active_round = 0;
        self.active_races = snake_seed_heats(&ranked_ids, self.config.heat_count(), self.config.heat_laps);
        self.active_stage = TournamentStage::QualifyingHeat;
        Ok(())
    }

    /// Recomputes `self.intermediate_standings` across all completed Qualifying Heat rounds (`Q1..=Qn`).
    /// In FIA Autocross 30 with 3 completed heats, each driver's score is their Best 2 of 3 heat results (Spec 130).
    fn recompute_intermediate_standings(&mut self, entries: &[SeriesStandingEntry]) {
        if self.qualifying_round_results.is_empty() {
            self.intermediate_standings.clear();
            return;
        }
        let is_ax_best_2_of_3 = self.config.is_autocross_30()
            && self.config.qualifying_rounds >= 3
            && self.qualifying_round_results.len() >= 3;

        let mut combined: Vec<RoundDriverResult> = entries
            .iter()
            .map(|entry| {
                let mut best_lap: Option<f32> = None;
                let mut best_heat_pos = usize::MAX;
                let mut heat_times: Vec<f32> = Vec::with_capacity(self.qualifying_round_results.len());

                for q_round in &self.qualifying_round_results {
                    if let Some(res) = q_round.iter().flatten().find(|r| r.driver_id == entry.driver_id) {
                        heat_times.push(res.total_time);
                        if let Some(bl) = res.best_lap {
                            best_lap = Some(best_lap.map_or(bl, |prev| prev.min(bl)));
                        }
                        best_heat_pos = best_heat_pos.min(res.finish_position);
                    } else {
                        heat_times.push(DNF_TIME);
                    }
                }

                let total_time = if is_ax_best_2_of_3 {
                    // Best 2 of 3: sort times ascending, sum the lowest 2
                    heat_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
                    heat_times.iter().take(2).sum()
                } else {
                    let mut sum = 0.0f32;
                    let mut any_dnf = false;
                    for &t in &heat_times {
                        if t >= DNF_TIME {
                            any_dnf = true;
                            sum += DNF_TIME;
                        } else {
                            sum += t;
                        }
                    }
                    if any_dnf && sum < DNF_TIME {
                        sum = DNF_TIME;
                    }
                    sum
                };

                RoundDriverResult {
                    driver_id: entry.driver_id.clone(),
                    driver_name: entry.driver_name.clone(),
                    team_name: entry.team_name.clone(),
                    finish_position: best_heat_pos,
                    total_time,
                    best_lap,
                    points_awarded: 0,
                    has_fastest_lap: false,
                }
            })
            .collect();

        combined.sort_by(by_elapsed_time);
        renumber(&mut combined);
        if self.qualifying_round_results.len() >= self.config.qualifying_rounds {
            if self.config.is_world_rx() {
                for (idx, r) in combined.iter_mut().enumerate() {
                    r.points_awarded = WORLD_RX_INTERMEDIATE_POINTS_12.get(idx).copied().unwrap_or(0);
                }
            } else if self.config.is_autocross_30() {
                for (idx, r) in combined.iter_mut().enumerate() {
                    r.points_awarded = FIA_AX_INTERMEDIATE_POINTS_10.get(idx).copied().unwrap_or(0);
                }
            }
        }
        self.intermediate_standings = combined;
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
        if self.config.is_world_rx() && round == 0 {
            let top12: Vec<&RoundDriverResult> = self.intermediate_standings.iter().take(RX_INTERMEDIATE_CUTOFF).collect();
            let odd: Vec<&RoundDriverResult> = top12.iter().step_by(2).copied().collect();
            let even: Vec<&RoundDriverResult> = top12.iter().skip(1).step_by(2).copied().collect();
            let out: Vec<&RoundDriverResult> = self.intermediate_standings.iter().skip(RX_INTERMEDIATE_CUTOFF).collect();
            return (vec![odd, even], out);
        }
        if self.config.is_autocross_30() && round == 0 {
            let top20: Vec<&RoundDriverResult> = self.intermediate_standings.iter().take(AX_INTERMEDIATE_CUTOFF).collect();
            let odd: Vec<&RoundDriverResult> = top20.iter().step_by(2).copied().collect();
            let even: Vec<&RoundDriverResult> = top20.iter().skip(1).step_by(2).copied().collect();
            let out: Vec<&RoundDriverResult> = self.intermediate_standings.iter().skip(AX_INTERMEDIATE_CUTOFF).collect();
            return (vec![odd, even], out);
        }
        let advancers = self.config.advancers_per_group();
        let mut advancing = Vec::new();
        let mut out = Vec::new();
        for group in self.round_results(round) {
            advancing.push(group.iter().filter(|r| r.finish_position <= advancers).collect());
            out.extend(group.iter().filter(|r| r.finish_position > advancers));
        }
        (advancing, out)
    }

    /// Pairs two Semifinal finisher slices row by row (`Row 1: pos 0`, `Row 2: pos 1`, etc.),
    /// ordering the two drivers within each row by their Intermediate Classification rank.
    fn pair_rows_by_intermediate_rank(&self, semi1: &[&RoundDriverResult], semi2: &[&RoundDriverResult]) -> Vec<String> {
        let inter_rank = |id: &str| -> usize {
            self.intermediate_standings
                .iter()
                .position(|r| r.driver_id == id)
                .unwrap_or(usize::MAX)
        };
        let rows = semi1.len().max(semi2.len());
        let mut grid = Vec::with_capacity(semi1.len() + semi2.len());
        for row in 0..rows {
            match (semi1.get(row), semi2.get(row)) {
                (Some(a), Some(b)) => {
                    if inter_rank(&a.driver_id) <= inter_rank(&b.driver_id) {
                        grid.push(a.driver_id.clone());
                        grid.push(b.driver_id.clone());
                    } else {
                        grid.push(b.driver_id.clone());
                        grid.push(a.driver_id.clone());
                    }
                }
                (Some(a), None) => grid.push(a.driver_id.clone()),
                (None, Some(b)) => grid.push(b.driver_id.clone()),
                (None, None) => {}
            }
        }
        grid
    }

    /// Schedules the races of `round` from the results of `round - 1`.
    fn schedule_round(&mut self, round: usize) {
        let config = self.config;
        let grid_size = config.grid_size();
        let groups = config.groups_in_round(round);
        let stage = config.stage_for_round(round);

        if config.is_world_rx() || config.is_autocross_30() {
            let mut races: Vec<TournamentRace> = if round == 1 {
                // Semifinals: Odd/Even split from the Top 12 (RX) or Top 20 (AX) of the Intermediate Classification.
                let (odd_even, _) = self.split_round(0);
                odd_even
                    .into_iter()
                    .enumerate()
                    .map(|(j, seeds)| TournamentRace {
                        stage: TournamentStage::Semifinal,
                        group: j,
                        groups: 2,
                        laps: config.semi_laps,
                        driver_ids: seeds.into_iter().map(|r| r.driver_id.clone()).collect(),
                    })
                    .collect()
            } else {
                // World RX Grand Final: Top 3 from Semi 1 and Semi 2 paired row by row (Winners on Row 1, 2nd on Row 2, 3rd on Row 3).
                let adv = config.advancers_per_group();
                let s1_top: Vec<&RoundDriverResult> = self.semi_results.first().map(|s| s.iter().take(adv).collect()).unwrap_or_default();
                let s2_top: Vec<&RoundDriverResult> = self.semi_results.get(1).map(|s| s.iter().take(adv).collect()).unwrap_or_default();
                vec![TournamentRace {
                    stage: TournamentStage::GrandFinal,
                    group: 0,
                    groups: 1,
                    laps: config.final_laps,
                    driver_ids: self.pair_rows_by_intermediate_rank(&s1_top, &s2_top),
                }]
            };

            let player_in_main = races.iter().any(|r| r.driver_ids.contains(&self.player_id));
            if round == config.final_round() {
                let adv = config.advancers_per_group();
                let s1_out: Vec<&RoundDriverResult> = self.semi_results.first().map(|s| s.iter().skip(adv).collect()).unwrap_or_default();
                let s2_out: Vec<&RoundDriverResult> = self.semi_results.get(1).map(|s| s.iter().skip(adv).collect()).unwrap_or_default();
                let mut field = self.pair_rows_by_intermediate_rank(&s1_out, &s2_out);
                if !player_in_main && !field.contains(&self.player_id) {
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
                let (_, out) = self.split_round(0);
                let mut field: Vec<(String, usize)> = out
                    .iter()
                    .filter(|r| r.driver_id != self.player_id)
                    .take(grid_size - 1)
                    .map(|r| (r.driver_id.clone(), r.finish_position))
                    .collect();
                let player_rank = out
                    .iter()
                    .find(|r| r.driver_id == self.player_id)
                    .map(|r| r.finish_position)
                    .unwrap_or(usize::MAX);
                field.push((self.player_id.clone(), player_rank));
                field.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
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
            return;
        }

        // Autocross (Spec 104): Group j takes the advancers of groups j and j + groups.
        let (advancing, mut out) = self.split_round(round - 1);
        out.sort_by(|a, b| by_elapsed_time(a, b));
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

        let player_in_main = races.iter().any(|r| r.driver_ids.contains(&self.player_id));
        if round == config.final_round() {
            // B-Final: the drivers who missed the Grand Final from the last semifinal round. A player who
            // came through consolation takes the place of the slowest of them and starts last.
            let mut field: Vec<String> = out.iter().map(|r| r.driver_id.clone()).collect();
            if !player_in_main && !field.contains(&self.player_id) {
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
            // The player is out of the bracket but keeps racing: the best (grid_size - 1) of the drivers who just missed
            // the cut line up with him.
            let mut field: Vec<(String, f32)> = out
                .iter()
                .filter(|r| r.driver_id != self.player_id)
                .take(grid_size - 1)
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
    /// B-Final, then the drivers each earlier stage dropped, the latest stage first.
    /// For World RX (`config.is_world_rx()`), each driver's `points_awarded` is populated with their
    /// Three-Stage cumulative score (`intermediate_pts + semi_pts + final_pts`).
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
        if self.config.is_world_rx() || self.config.is_autocross_30() {
            for r in &mut ranked {
                r.points_awarded = self.driver_total_weekend_points(&r.driver_id);
            }
        }
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
        let c24 = TournamentConfig::world_rx_default();
        assert!(c24.is_world_rx() && c24.has_superpole());
        assert_eq!((c24.grid_size(), c24.heat_count(), c24.round_count()), (6, 4, 3));
        let c30 = TournamentConfig::autocross_default();
        assert!(c30.is_autocross_30() && c30.has_superpole());
        assert_eq!((c30.grid_size(), c30.heat_count(), c30.round_count()), (10, 3, 3));
        assert_eq!(c30.stage_for_round(0), TournamentStage::QualifyingHeat);
        assert_eq!(c30.stage_for_round(1), TournamentStage::Semifinal);
        assert_eq!(c30.stage_for_round(2), TournamentStage::GrandFinal);
        assert!(c30.validate().is_ok());
        let c64 = TournamentConfig { total_drivers: 64, ..c32 };
        assert_eq!((c64.heat_count(), c64.round_count()), (8, 4));
        assert_eq!(c64.stage_for_round(1), TournamentStage::Quarterfinal);
        assert!(TournamentConfig { total_drivers: 20, ..c32 }.validate().is_err());
        assert!(TournamentConfig { qualifying_rounds: 0, ..c24 }.validate().is_err());
        assert!(TournamentConfig { qualifying_rounds: 5, ..c24 }.validate().is_err());
    }
}
