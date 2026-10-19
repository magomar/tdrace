use crate::ai::{DriverCharacter, DriverTier, DrivingStyle, LcgRng};
use serde::{Deserialize, Serialize};

/// Persistent career competitor entry tracking identity, driving style, and dynamic performance tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CareerRivalEntry {
    pub driver_id: String,
    pub driver_name: String,
    pub style: DrivingStyle,
    pub tier: DriverTier,
    /// Racing number kept across rounds (spec 092). `None` until the first grid is built.
    #[serde(default, deserialize_with = "crate::ai::racing_number::deserialize_stored_race_number")]
    pub race_number: Option<u8>,
}

impl CareerRivalEntry {
    /// Creates a career rival entry from a predefined DriverCharacter and tier.
    pub fn new(character: &DriverCharacter, tier: DriverTier) -> Self {
        Self {
            driver_id: character.id.to_string(),
            driver_name: character.name.to_string(),
            style: character.style,
            tier,
            race_number: None,
        }
    }

    /// Resolves this rival back to their static DriverCharacter definition across modules.
    pub fn resolve_character(&self) -> Option<DriverCharacter> {
        DriverCharacter::find_global(&self.driver_id)
    }
}

/// Skill progression outcome for a retained rival across tier unlock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillProgressionOutcome {
    StepUp,
    HoldSteady,
    StandoutLeap,
}

/// Detailed progression report for a retained rival.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetainedRivalReport {
    pub rival: CareerRivalEntry,
    pub previous_tier: DriverTier,
    pub new_tier: DriverTier,
    pub outcome: SkillProgressionOutcome,
}

/// Summary report generated during tier unlock roster evolution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RosterEvolutionReport {
    pub retained_rivals: Vec<RetainedRivalReport>,
    pub churned_out: Vec<CareerRivalEntry>,
    pub churned_in: Vec<CareerRivalEntry>,
}

/// Engine managing Career Mode rival roster lifecycle, persistence, 90/10 churn,
/// and probabilistic skill progression.
pub struct RosterEvolutionEngine;

impl RosterEvolutionEngine {
    /// Initializes a career rival roster of size `n` with uniform driving style distribution
    /// and bell-curve tier assignment centered on `target_tier`.
    pub fn initialize_career_roster(n: usize, target_tier: DriverTier, seed: u64) -> Vec<CareerRivalEntry> {
        Self::initialize_career_roster_from_pool(&[], n, target_tier, seed)
    }

    /// Initializes a career rival roster from an optional driver pool with uniform style distribution
    /// and bell-curve tier assignment centered on `target_tier`.
    pub fn initialize_career_roster_from_pool(
        pool: &[DriverCharacter],
        n: usize,
        target_tier: DriverTier,
        seed: u64,
    ) -> Vec<CareerRivalEntry> {
        Self::initialize_career_roster_from_pool_with_progression(pool, n, target_tier, 0.0, seed)
    }

    /// Initializes a career rival roster with experience-scaled `T4+`/`T5+` probability weights (Spec 131).
    pub fn initialize_career_roster_from_pool_with_progression(
        pool: &[DriverCharacter],
        n: usize,
        target_tier: DriverTier,
        experience_score: f32,
        seed: u64,
    ) -> Vec<CareerRivalEntry> {
        let opponents = if !pool.is_empty() {
            DriverCharacter::sample_casual_race_roster(pool, n, seed)
        } else {
            DriverCharacter::sample_casual_race_opponents(n, seed)
        };
        let tiers = if experience_score > 0.0 {
            DriverTier::sample_championship_grid_tiers(
                target_tier.car_tier_u8() as u32,
                experience_score,
                n,
                seed.wrapping_add(777),
            )
        } else {
            target_tier.sample_grid_tiers(n, seed.wrapping_add(777))
        };
        opponents
            .into_iter()
            .zip(tiers.into_iter())
            .map(|(char_def, tier)| CareerRivalEntry::new(&char_def, tier))
            .collect()
    }

    /// Advances the career roster to a newly unlocked tier `new_tier`:
    /// 1. Evaluates rank-dependent probabilistic skill progression on retained rivals
    ///    (higher rank = higher probability of advancement up to 80%, down to 70% for last place;
    ///    macro field average across symmetric ranks is 65% step up, 25% hold steady, 10% standout leap).
    /// 2. Retains 90% of rivals and churns 10% (min 1).
    /// 3. Departing rivals are replaced with fresh characters selected by sampling a driving style
    ///    uniformly across all 6 styles (P = 1/6) and picking an unused character fitting that style from the global 72 pool.
    /// 4. Incoming rivals are assigned tiers from the discrete bell curve of `new_tier`.
    pub fn evolve_roster(
        current_roster: &[CareerRivalEntry],
        new_tier: DriverTier,
        seed: u64,
    ) -> (Vec<CareerRivalEntry>, RosterEvolutionReport) {
        Self::evolve_roster_with_progression(current_roster, new_tier, 0.0, seed)
    }

    /// Advances the career roster to `new_tier` while scaling the promotion probability of `T4+` (`ProPlus`)
    /// and `T5+` (`LegendPlus`) error-free variants with the player's `experience_score` (`0.0..=1.0`, Spec 131).
    pub fn evolve_roster_with_progression(
        current_roster: &[CareerRivalEntry],
        new_tier: DriverTier,
        experience_score: f32,
        seed: u64,
    ) -> (Vec<CareerRivalEntry>, RosterEvolutionReport) {
        let exp = experience_score.clamp(0.0, 1.0);
        if current_roster.is_empty() {
            let initial = Self::initialize_career_roster_from_pool_with_progression(&[], 7, new_tier, exp, seed);
            let report = RosterEvolutionReport {
                retained_rivals: Vec::new(),
                churned_out: Vec::new(),
                churned_in: initial.clone(),
            };
            return (initial, report);
        }

        let n = current_roster.len();
        let mut rng = LcgRng(seed.wrapping_add(0xD15EA5E));

        // Calculate churn count: 10% turnover, at least 1 churn if n >= 2, otherwise 0
        let retained_target = ((n as f32) * 0.90).floor() as usize;
        let churn_count = if n <= 1 { 0 } else { (n - retained_target).max(1) };

        // Choose which indices to churn:
        // Protect podium finishers (first 3 indices if sorted by standings) from churn if possible,
        // or choose uniformly among non-podium finishers.
        let mut eligible_churn_indices: Vec<usize> = if n > 3 {
            (3..n).collect()
        } else {
            (0..n).collect()
        };
        rng.shuffle(&mut eligible_churn_indices);
        let churn_indices: Vec<usize> = eligible_churn_indices.into_iter().take(churn_count).collect();

        let mut retained_reports = Vec::new();
        let mut churned_out = Vec::new();
        let mut new_roster = Vec::new();

        // Step 1: Retain 90% and evaluate rank-dependent probabilistic skill progression
        for (idx, rival) in current_roster.iter().enumerate() {
            if churn_indices.contains(&idx) {
                churned_out.push(rival.clone());
            } else {
                let previous_tier = rival.tier;
                // Performance score: index 0 (1st place) -> 1.0, index n-1 (last place) -> 0.0
                let perf_score = if n > 1 {
                    1.0 - (idx as f32) / ((n - 1) as f32)
                } else {
                    0.5
                };
                let d = perf_score - 0.5; // [-0.5, +0.5]

                // Rank-modulated probabilities:
                // 1st place (d = +0.5): 67.5% StepUp, 20.0% HoldSteady, 12.5% StandoutLeap (80% advance)
                // Median (d = 0.0):    65.0% StepUp, 25.0% HoldSteady, 10.0% StandoutLeap (75% advance)
                // Last place (d = -0.5): 62.5% StepUp, 30.0% HoldSteady, 7.5% StandoutLeap (70% advance)
                let p_leap = 0.10 + 0.05 * d;
                let p_hold = 0.25 - 0.10 * d;
                let p_step = 1.0 - p_leap - p_hold;

                let roll = rng.next_f32();
                let (mut assigned_tier, outcome) = if roll < p_step {
                    (new_tier, SkillProgressionOutcome::StepUp)
                } else if roll < p_step + p_hold {
                    (previous_tier, SkillProgressionOutcome::HoldSteady)
                } else {
                    let leap_tier = DriverTier::from_u8((new_tier.car_tier_u8() + 1).min(5));
                    (leap_tier, SkillProgressionOutcome::StandoutLeap)
                };

                if exp > 0.0 {
                    let mut plus_rng = LcgRng::new(seed.wrapping_add(0x4117_E000 ^ ((idx as u64) * 0x9E37_79B9)));
                    let plus_roll = plus_rng.next_f32();
                    if previous_tier == DriverTier::LegendPlus {
                        assigned_tier = DriverTier::LegendPlus;
                    } else if assigned_tier == DriverTier::Legend {
                        let p_t5_plus = (0.10 + 0.55 * exp + 0.12 * perf_score
                            + if previous_tier == DriverTier::ProPlus { 0.15 } else { 0.0 })
                        .clamp(0.0, 0.85);
                        if plus_roll < p_t5_plus {
                            assigned_tier = DriverTier::LegendPlus;
                        }
                    } else if assigned_tier == DriverTier::Pro {
                        let p_t4_plus = (0.12 + 0.55 * exp + 0.10 * perf_score).clamp(0.0, 0.80);
                        if plus_roll < p_t4_plus {
                            assigned_tier = DriverTier::ProPlus;
                        }
                    } else if assigned_tier == DriverTier::Contender
                        && outcome == SkillProgressionOutcome::StandoutLeap
                        && exp >= 0.35
                        && plus_roll < 0.35 * exp
                    {
                        assigned_tier = DriverTier::ProPlus;
                    }
                }

                let mut updated_rival = rival.clone();
                updated_rival.tier = assigned_tier;
                retained_reports.push(RetainedRivalReport {
                    rival: updated_rival.clone(),
                    previous_tier,
                    new_tier: assigned_tier,
                    outcome,
                });
                new_roster.push(updated_rival);
            }
        }

        // Step 2: Churn 10% - Replace departed rivals with fresh characters drawn by sampling
        // a driving style uniformly at random (P = 1/6) and selecting an unused character fitting that style.
        let all_characters = DriverCharacter::all_across_modules();
        let mut churned_in = Vec::new();

        for departed in &churned_out {
            let style_idx = (rng.next_u32() as usize) % DrivingStyle::ALL.len();
            let target_style = DrivingStyle::ALL[style_idx];

            // Candidates: global characters with target_style who are NOT currently in the new roster
            let mut candidates: Vec<DriverCharacter> = all_characters
                .iter()
                .filter(|c| c.style == target_style && !new_roster.iter().any(|r| r.driver_id == c.id))
                .cloned()
                .collect();

            if candidates.is_empty() {
                // Fallback to any character not in new roster
                candidates = all_characters
                    .iter()
                    .filter(|c| !new_roster.iter().any(|r| r.driver_id == c.id))
                    .cloned()
                    .collect();
            }

            rng.shuffle(&mut candidates);
            let replacement_char = candidates.first().cloned().unwrap_or_else(|| {
                DriverCharacter::find_global(&departed.driver_id).unwrap_or(all_characters[0].clone())
            });

            // Sample incoming rival tier from bell curve of new_tier
            let incoming_seed = rng.next_u32() as u64;
            let incoming_tier = if exp > 0.0 {
                DriverTier::sample_championship_grid_tiers(
                    new_tier.car_tier_u8() as u32,
                    exp,
                    1,
                    incoming_seed,
                )[0]
            } else {
                new_tier.sample_grid_tiers(1, incoming_seed)[0]
            };
            let replacement_rival = CareerRivalEntry::new(&replacement_char, incoming_tier);

            churned_in.push(replacement_rival.clone());
            new_roster.push(replacement_rival);
        }

        let report = RosterEvolutionReport {
            retained_rivals: retained_reports,
            churned_out,
            churned_in,
        };

        (new_roster, report)
    }

    /// Post-season veteran progression pass: as a player completes additional championships,
    /// front-running rivals at Tier 3+ have a chance to graduate into error-free `T4+` (`ProPlus`)
    /// and `T5+` (`LegendPlus`) variants proportional to `experience_score` (Spec 131).
    pub fn evolve_post_season_veterans(
        roster: &mut [CareerRivalEntry],
        championship_tier: u32,
        experience_score: f32,
        seed: u64,
    ) {
        let exp = experience_score.clamp(0.0, 1.0);
        if exp <= 0.0 || championship_tier < 3 || roster.is_empty() {
            return;
        }
        let n = roster.len();
        let mut rng = LcgRng::new(seed.wrapping_add(0x5EA5_0126));
        for (idx, rival) in roster.iter_mut().enumerate() {
            let perf_score = if n > 1 {
                1.0 - (idx as f32) / ((n - 1) as f32)
            } else {
                0.5
            };
            let roll = rng.next_f32();
            match rival.tier {
                DriverTier::Pro => {
                    let p_promote = (0.15 + 0.45 * exp + 0.15 * perf_score).clamp(0.0, 0.75);
                    if roll < p_promote {
                        rival.tier = DriverTier::ProPlus;
                    }
                }
                DriverTier::ProPlus if championship_tier >= 4 => {
                    let p_promote = (0.10 + 0.40 * exp + 0.15 * perf_score).clamp(0.0, 0.65);
                    if roll < p_promote {
                        rival.tier = if championship_tier >= 5 {
                            DriverTier::LegendPlus
                        } else {
                            DriverTier::Legend
                        };
                    }
                }
                DriverTier::Legend if championship_tier >= 4 => {
                    let p_promote = (0.12 + 0.50 * exp + 0.15 * perf_score).clamp(0.0, 0.78);
                    if roll < p_promote {
                        rival.tier = DriverTier::LegendPlus;
                    }
                }
                _ => {}
            }
        }
    }
}
