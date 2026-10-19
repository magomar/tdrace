use serde::{Deserialize, Serialize};

use crate::ai::{DriverCharacter, DriverStats, DriverTier, DrivingStyle, LcgRng};
use crate::profile::{ModuleCareerProgress, PlayerProfile};
use crate::series::substitution::resolve_car_and_team_for_driver;
use crate::series::{SeriesSession, SeriesStandingEntry, TeamStandingEntry};

/// Active contract terms and performance stats for a candidate or hired driver (Spec 127).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriverHireQuote {
    pub character_id: String,
    pub character_name: String,
    pub style: DrivingStyle,
    pub tier: DriverTier,
    pub stats: DriverStats,
    /// Per-race wage paid after each completed round (Credits). Proportional to tier.
    pub per_race_salary: u64,
}

/// Configuration and telemetry of a player-owned racing team (Spec 127).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerTeamConfig {
    pub team_name: String,
    /// Catalog car ID purchased/assigned for the bot companion.
    pub companion_car_id: String,
    /// Current contract terms of the hired bot teammate.
    pub contract: DriverHireQuote,
    /// Cumulative dividends earned by the player from teammate finishes (50% purse split).
    pub total_dividends_earned: u64,
    /// Cumulative salaries paid to the teammate.
    pub total_salaries_paid: u64,
}

impl SeriesSession {
    /// Generates a scouting market of 6 candidates spanning all 6 driving styles and
    /// sampled from the championship bell-curve tier distribution around `series_tier` (Spec 127).
    pub fn generate_scout_pool(series_tier: u32, seed: u64) -> Vec<DriverHireQuote> {
        let mut rng = LcgRng::new(seed ^ 0x5C00_71E5);
        let mut pool = Vec::with_capacity(6);
        let styles = DrivingStyle::ALL;

        let clamped_series = series_tier.clamp(1, 5) as u8;
        let min_tier = clamped_series.saturating_sub(1).max(1);
        let max_tier = (clamped_series + 1).min(5);

        let characters = DriverCharacter::all_across_modules();
        let mut used_ids = std::collections::HashSet::new();

        for &style in styles.iter() {
            let candidate_tier_val = if min_tier == max_tier {
                min_tier
            } else if rng.gen_range(0..100) < 60 {
                clamped_series
            } else if clamped_series == 1 || rng.gen_range(0..100) < 55 {
                max_tier
            } else {
                min_tier
            };
            let candidate_tier = DriverTier::from_u8(candidate_tier_val);

            let matching: Vec<&DriverCharacter> = characters
                .iter()
                .filter(|c| c.style == style && !used_ids.contains(c.id))
                .collect();
            if matching.is_empty() {
                continue;
            }
            let char_idx = (rng.gen_range(0..matching.len() as u32)) as usize;
            let character = matching[char_idx];
            used_ids.insert(character.id);

            let stats = character.resolve_stats(candidate_tier);
            let salary = Self::calculate_driver_salary(candidate_tier, stats.speed);

            pool.push(DriverHireQuote {
                character_id: character.id.to_string(),
                character_name: character.name.to_string(),
                style,
                tier: candidate_tier,
                stats,
                per_race_salary: salary,
            });
            if pool.len() >= 6 {
                break;
            }
        }

        // Ensure the shortlist includes both the base series tier and an adjacent tier (e.g. Tier 1 and Tier 2 in T1)
        if pool.len() >= 2 && min_tier != max_tier {
            let base_tier = DriverTier::from_u8(clamped_series);
            let adj_tier = if clamped_series < 5 {
                DriverTier::from_u8(max_tier)
            } else {
                DriverTier::from_u8(min_tier)
            };
            if !pool.iter().any(|q| q.tier == base_tier) {
                if let Some(first) = pool.first_mut() {
                    if let Some(ch) = DriverCharacter::find_global(&first.character_id) {
                        first.tier = base_tier;
                        first.stats = ch.resolve_stats(base_tier);
                        first.per_race_salary = Self::calculate_driver_salary(base_tier, first.stats.speed);
                    }
                }
            }
            if !pool.iter().any(|q| q.tier == adj_tier) {
                if let Some(last) = pool.last_mut() {
                    if let Some(ch) = DriverCharacter::find_global(&last.character_id) {
                        last.tier = adj_tier;
                        last.stats = ch.resolve_stats(adj_tier);
                        last.per_race_salary = Self::calculate_driver_salary(adj_tier, last.stats.speed);
                    }
                }
            }
        }

        pool
    }

    /// Calculates per-race driver salary proportional to performance tier and speed quality (Spec 127).
    pub fn calculate_driver_salary(tier: DriverTier, speed_stat: f32) -> u64 {
        let base_salary = match tier {
            DriverTier::Rookie => 350,      // Tier 1
            DriverTier::Amateur => 750,     // Tier 2 (~2.1x)
            DriverTier::Contender => 1_500, // Tier 3 (~4.3x)
            DriverTier::Pro | DriverTier::ProPlus => 3_200,       // Tier 4 (~9.1x)
            DriverTier::Legend | DriverTier::LegendPlus => 6_500,    // Tier 5 (~18.6x)
        };
        let speed_factor = 0.85 + (speed_stat - 0.70) * 0.75;
        let raw = (base_salary as f32) * speed_factor;
        // Round to nearest 50 Credits
        (((raw / 50.0).round() as u64) * 50).max(250)
    }

    /// Verifies whether a player profile has sufficient credits to found a custom racing team
    /// given the cheapest eligible companion car cost and initial race wage (Spec 127).
    pub fn can_afford_team_founding(
        available_credits: u64,
        companion_car_price: u64,
        initial_race_salary: u64,
    ) -> Result<(), String> {
        let required = companion_car_price.saturating_add(initial_race_salary);
        if available_credits < required {
            Err(format!(
                "INSUFFICIENT FUNDS (REQUIRES ~${} Cr, AVAILABLE ${} Cr)",
                required, available_credits
            ))
        } else {
            Ok(())
        }
    }

    /// Founds a player-owned constructor team for this championship session (Spec 127):
    /// 1. Verifies credit solvency (`profile.credits >= car_price + contract.per_race_salary` when buying a car,
    ///    or `profile.credits >= car_price`).
    /// 2. Deducts `car_price` from `profile.credits` and permanently unlocks & registers `companion_car_id` in the player's fleet.
    /// 3. Assigns both the human player and the hired bot teammate to `team_name` in `self.standings`.
    /// 4. Registers the player-owned team in `self.team_standings` with `is_player_owned = true`.
    pub fn found_player_team(
        &mut self,
        profile: &mut PlayerProfile,
        career_progress: &mut ModuleCareerProgress,
        team_name: impl Into<String>,
        companion_car_id: impl Into<String>,
        car_price: u64,
        contract: DriverHireQuote,
    ) -> Result<(), String> {
        let team_name = team_name.into();
        let companion_car_id = companion_car_id.into();

        if profile.credits < car_price.saturating_add(contract.per_race_salary) {
            return Err(format!(
                "INSUFFICIENT FUNDS (REQUIRES ~${} Cr)",
                car_price.saturating_add(contract.per_race_salary)
            ));
        }

        if car_price > 0 {
            profile.spend_credits(car_price)?;
        }

        // Permanently unlock and register companion car in player's fleet
        career_progress.ensure_car(&companion_car_id);
        if !profile.owned_cars.iter().any(|c| c == &companion_car_id) {
            profile.owned_cars.push(companion_car_id.clone());
        }

        // 1. Update player's team name in standings
        let player_name = if let Some(p_entry) = self.standings.iter_mut().find(|s| s.driver_id == "player") {
            p_entry.team_name = team_name.clone();
            p_entry.driver_name.clone()
        } else {
            let mut p_entry = SeriesStandingEntry::new("player", &profile.name, &team_name);
            let eff_mod = if self.module_id.trim().is_empty() {
                "gt"
            } else {
                &self.module_id
            };
            let (car_opt, _) = resolve_car_and_team_for_driver(eff_mod, self.tier.clamp(1, 7) as u8, None, 0);
            p_entry.car_model_id = car_opt;
            let p_name = p_entry.driver_name.clone();
            self.standings.insert(0, p_entry);
            p_name
        };

        // 2. Place or update the hired bot teammate in standings
        if let Some(bot_entry) = self
            .standings
            .iter_mut()
            .find(|s| s.driver_id == contract.character_id)
        {
            bot_entry.driver_name = contract.character_name.clone();
            bot_entry.team_name = team_name.clone();
            bot_entry.car_model_id = Some(companion_car_id.clone());
            bot_entry.ai_style = Some(contract.style.as_str().to_string());
            bot_entry.ai_tier = Some(contract.tier.to_u8());
        } else {
            let mut teammate_entry = SeriesStandingEntry::new(
                &contract.character_id,
                &contract.character_name,
                &team_name,
            )
            .with_ai_style_and_tier(
                Some(contract.style.as_str().to_string()),
                Some(contract.tier.to_u8()),
            );
            teammate_entry.car_model_id = Some(companion_car_id.clone());
            if let Some(ch) = DriverCharacter::find_global(&contract.character_id) {
                teammate_entry.ai_character = Some(ch.id.to_string());
            }

            // Replace the last non-player AI slot if roster is already at grid capacity, or push if small
            if self.standings.len() >= 2 {
                if let Some(replace_idx) = self.standings.iter().rposition(|s| s.driver_id != "player") {
                    self.standings[replace_idx] = teammate_entry;
                } else {
                    self.standings.push(teammate_entry);
                }
            } else {
                self.standings.push(teammate_entry);
            }
        }

        self.player_team = Some(PlayerTeamConfig {
            team_name: team_name.clone(),
            companion_car_id,
            contract: contract.clone(),
            total_dividends_earned: 0,
            total_salaries_paid: 0,
        });

        if self.current_round == 0 {
            self.team_standings.clear();
        }
        self.rebuild_initial_team_standings();

        // Ensure the player team entry is marked as player-owned and includes both drivers
        if let Some(team_entry) = self.team_standings.iter_mut().find(|t| t.team_name == team_name) {
            team_entry.is_player_owned = true;
            team_entry.register_driver("player", &player_name);
            team_entry.register_driver(&contract.character_id, &contract.character_name);
        } else {
            let mut team_entry = TeamStandingEntry::new(&team_name, true);
            team_entry.register_driver("player", &player_name);
            team_entry.register_driver(&contract.character_id, &contract.character_name);
            self.team_standings.insert(0, team_entry);
        }

        Ok(())
    }

    /// Settles post-race constructor team finances after a completed round (Spec 127):
    /// 1. Credits 50% of the hired bot teammate's gross prize purse (`teammate_gross_purse / 2`) to `profile`.
    /// 2. Deducts the teammate's per-race salary (`contract.per_race_salary`) from `profile.credits`
    ///    (applying the salary safety net if wallet balance drops to 0).
    /// Returns `(dividend_earned, salary_deducted, optional_deferred_warning)`.
    pub fn settle_post_race_team_finances(
        &mut self,
        profile: &mut PlayerProfile,
        _teammate_finish_pos: usize,
        teammate_gross_purse: u64,
    ) -> (u64, u64, Option<String>) {
        let Some(team_cfg) = &mut self.player_team else {
            return (0, 0, None);
        };

        let dividend = teammate_gross_purse / 2;
        if dividend > 0 {
            profile.add_credits(dividend);
            team_cfg.total_dividends_earned = team_cfg.total_dividends_earned.saturating_add(dividend);
        }

        let salary = team_cfg.contract.per_race_salary;
        let (salary_paid, warning) = if profile.credits >= salary {
            profile.credits -= salary;
            (salary, None)
        } else {
            let paid = profile.credits;
            profile.credits = 0;
            (
                paid,
                Some("TEAM WAGES DEFERRED: RACE RESULTS MUST IMPROVE NEXT ROUND".to_string()),
            )
        };
        team_cfg.total_salaries_paid = team_cfg.total_salaries_paid.saturating_add(salary_paid);

        (dividend, salary, warning)
    }
}
