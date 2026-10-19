use serde::{Deserialize, Serialize};

use crate::ai::{DriverCharacter, DriverTier, LcgRng};
use crate::profile::ModuleCareerProgress;
use crate::series::SeriesSession;

/// Progression outcome for a hired bot driver at season conclusion (Spec 024 / Spec 127).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotProgressionOutcome {
    StepUp(DriverTier),
    StandoutLeap(DriverTier),
    HoldSteady(DriverTier),
}

impl BotProgressionOutcome {
    /// Returns the resulting `DriverTier` after progression evaluation.
    pub const fn new_tier(self) -> DriverTier {
        match self {
            Self::StepUp(t) | Self::StandoutLeap(t) | Self::HoldSteady(t) => t,
        }
    }

    /// Returns true if the bot advanced at least one performance tier.
    pub fn is_promoted(self, previous_tier: DriverTier) -> bool {
        self.new_tier().to_u8() > previous_tier.to_u8()
    }
}

/// Summary report generated at the end of a championship season for a hired bot teammate (Spec 127).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeammateSeasonProgressionReport {
    pub driver_id: String,
    pub driver_name: String,
    pub final_rank: usize,
    pub previous_tier: DriverTier,
    pub new_tier: DriverTier,
    pub outcome: BotProgressionOutcome,
    pub previous_salary: u64,
    pub new_salary: u64,
    pub debrief_message: String,
}

impl SeriesSession {
    /// Evaluates season-end skill progression for a hired bot teammate (Spec 024 Section 5 / Spec 127).
    ///
    /// Note: `final_rank` may be 0-indexed (`0..total_drivers`) or 1-indexed (`1..=total_drivers`).
    /// For top finishers (e.g. 1st or 2nd place), advancement probability (`p_step + p_leap`) is ~80%.
    pub fn evaluate_hired_bot_progression(
        current_tier: DriverTier,
        final_rank: usize,
        total_drivers: usize,
        seed: u64,
    ) -> BotProgressionOutcome {
        let mut rng = LcgRng::new(seed ^ 0xB07_D41);
        let n = total_drivers.max(2);
        // Normalize final_rank to 0-based index [0..n-1]
        let rank_0 = if final_rank >= n {
            n - 1
        } else {
            final_rank
        };
        let normalized_score = 1.0 - (rank_0 as f32 / (n - 1) as f32);
        let delta = normalized_score - 0.5; // [-0.5, +0.5]

        let p_leap = 0.10 + 0.05 * delta;
        let p_hold = 0.25 - 0.10 * delta;
        let roll = rng.gen_range(0..10_000) as f32 / 10_000.0;

        let cur_val = current_tier.to_u8();
        if roll < p_leap {
            let next_val = (cur_val + 2).min(5);
            BotProgressionOutcome::StandoutLeap(DriverTier::from_u8(next_val))
        } else if roll < p_leap + p_hold {
            BotProgressionOutcome::HoldSteady(current_tier)
        } else {
            let next_val = (cur_val + 1).min(5);
            BotProgressionOutcome::StepUp(DriverTier::from_u8(next_val))
        }
    }

    /// Evaluates and applies end-of-season skill progression to the player's hired bot teammate,
    /// updating their contract tier, stats, and per-race salary and returning a debrief report (Spec 127).
    pub fn evaluate_and_update_teammate_contract(
        &mut self,
        seed: u64,
    ) -> Option<TeammateSeasonProgressionReport> {
        let team_cfg = self.player_team.as_ref()?;
        let driver_id = team_cfg.contract.character_id.clone();
        let driver_name = team_cfg.contract.character_name.clone();
        let previous_tier = team_cfg.contract.tier;
        let previous_salary = team_cfg.contract.per_race_salary;

        let total_drivers = self.standings.len().max(2);
        let final_rank_0 = self
            .standings
            .iter()
            .position(|s| s.driver_id == driver_id)
            .unwrap_or(total_drivers - 1);
        let display_pos = final_rank_0 + 1;

        let outcome = Self::evaluate_hired_bot_progression(
            previous_tier,
            final_rank_0,
            total_drivers,
            seed,
        );
        let new_tier = outcome.new_tier();

        let new_stats = DriverCharacter::find_global(&driver_id)
            .map(|ch| ch.resolve_stats(new_tier))
            .unwrap_or(team_cfg.contract.stats);
        let new_salary = Self::calculate_driver_salary(new_tier, new_stats.speed);

        let debrief_message = if new_tier.to_u8() > previous_tier.to_u8() {
            format!(
                "Contract Review: {} completed the season in P{}! Following their outstanding performance, they stepped up from {} to {}. Updated Terms: per-race salary revised from ${} Cr to ${} Cr.",
                driver_name,
                display_pos,
                previous_tier.title(),
                new_tier.title(),
                previous_salary,
                new_salary
            )
        } else {
            format!(
                "Contract Review: {} completed the season in P{} and held steady at {} (${} Cr/race).",
                driver_name,
                display_pos,
                new_tier.title(),
                new_salary
            )
        };

        if let Some(cfg) = &mut self.player_team {
            cfg.contract.tier = new_tier;
            cfg.contract.stats = new_stats;
            cfg.contract.per_race_salary = new_salary;
        }
        if let Some(st) = self.standings.iter_mut().find(|s| s.driver_id == driver_id) {
            st.ai_tier = Some(new_tier.to_u8());
        }

        Some(TeammateSeasonProgressionReport {
            driver_id,
            driver_name,
            final_rank: display_pos,
            previous_tier,
            new_tier,
            outcome,
            previous_salary,
            new_salary,
            debrief_message,
        })
    }

    /// Computes the End-of-Season Constructor Podium Bonus (Spec 127 Section 3):
    /// - 1st in Constructors: 150% of Tier 1st-place Championship Podium Bonus
    /// - 2nd in Constructors: 100% of Tier 2nd-place Championship Podium Bonus
    /// - 3rd in Constructors: 50% of Tier 3rd-place Championship Podium Bonus
    pub fn calculate_constructor_podium_bonus(&self) -> Option<(usize, u64)> {
        let player_team_name = self.player_team.as_ref().map(|t| t.team_name.as_str())?;
        let pos_0 = self
            .team_standings
            .iter()
            .position(|t| t.is_player_owned || t.team_name == player_team_name)?;
        let position = pos_0 + 1;
        let base_podium = ModuleCareerProgress::championship_podium_bonus(self.tier, position);
        let bonus = match position {
            1 => ((base_podium as f64) * 1.50).round() as u64,
            2 => base_podium,
            3 => ((base_podium as f64) * 0.50).round() as u64,
            _ => 0,
        };
        if bonus > 0 {
            Some((position, bonus))
        } else {
            None
        }
    }
}
