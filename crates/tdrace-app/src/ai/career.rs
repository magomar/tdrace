use crate::ai::{DriverCharacter, DriverTier, DrivingStyle, LcgRng};
use serde::{Deserialize, Serialize};

/// Persistent career competitor entry tracking identity, driving style, and dynamic performance tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CareerRivalEntry {
    pub driver_id: String,
    pub driver_name: String,
    pub style: DrivingStyle,
    pub tier: DriverTier,
}

impl CareerRivalEntry {
    /// Creates a career rival entry from a predefined DriverCharacter and tier.
    pub fn new(character: &DriverCharacter, tier: DriverTier) -> Self {
        Self {
            driver_id: character.id.to_string(),
            driver_name: character.name.to_string(),
            style: character.style,
            tier,
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
        let opponents = if !pool.is_empty() {
            DriverCharacter::sample_casual_race_roster(pool, n, seed)
        } else {
            DriverCharacter::sample_casual_race_opponents(n, seed)
        };
        let tiers = target_tier.sample_grid_tiers(n, seed.wrapping_add(777));
        opponents
            .into_iter()
            .zip(tiers.into_iter())
            .map(|(char_def, tier)| CareerRivalEntry::new(&char_def, tier))
            .collect()
    }

    /// Advances the career roster to a newly unlocked tier `new_tier`:
    /// 1. Evaluates probabilistic skill progression on retained rivals (65% step up, 25% hold steady, 10% standout leap).
    /// 2. Retains 90% of rivals and churns 10% (min 1).
    /// 3. Departing rivals are replaced with fresh characters matching their driving style from the global 72 pool.
    /// 4. Incoming rivals are assigned tiers from the discrete bell curve of `new_tier`.
    pub fn evolve_roster(
        current_roster: &[CareerRivalEntry],
        new_tier: DriverTier,
        seed: u64,
    ) -> (Vec<CareerRivalEntry>, RosterEvolutionReport) {
        if current_roster.is_empty() {
            let initial = Self::initialize_career_roster(7, new_tier, seed);
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

        // Step 1: Retain 90% and evaluate probabilistic skill progression
        for (idx, rival) in current_roster.iter().enumerate() {
            if churn_indices.contains(&idx) {
                churned_out.push(rival.clone());
            } else {
                let previous_tier = rival.tier;
                // Skill progression roll: 65% step up, 25% hold steady, 10% standout leap
                let roll = (rng.next_f32() * 100.0) as u32;
                let (assigned_tier, outcome) = if roll < 65 {
                    (new_tier, SkillProgressionOutcome::StepUp)
                } else if roll < 90 {
                    (previous_tier, SkillProgressionOutcome::HoldSteady)
                } else {
                    let leap_tier = DriverTier::from_u8((new_tier.to_u8() + 1).min(5));
                    (leap_tier, SkillProgressionOutcome::StandoutLeap)
                };

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

        // Step 2: Churn 10% - Replace departed rivals with new characters matching the departed style
        let all_characters = DriverCharacter::all_across_modules();
        let mut churned_in = Vec::new();

        for departed in &churned_out {
            let departing_style = departed.style;
            // Candidates: global characters with same style who are NOT currently in the new roster
            let mut candidates: Vec<DriverCharacter> = all_characters
                .iter()
                .filter(|c| c.style == departing_style && !new_roster.iter().any(|r| r.driver_id == c.id))
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
            let incoming_tier = new_tier.sample_grid_tiers(1, rng.next_u32() as u64)[0];
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
}
