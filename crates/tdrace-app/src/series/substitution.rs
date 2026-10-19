use serde::{Deserialize, Serialize};
use crate::ai::{DriverCharacter, DriverTier, DrivingStyle, LcgRng};
use super::{SeriesSession, SeriesStandingEntry};

/// Duration of a driver replacement before a championship race (Spec 126).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplacementDuration {
    /// Temporary 1-race stand-in; the regular driver returns before the next championship race with prior points intact.
    OneRace,
    /// Permanent seat change; the departing driver returns to the global driver pool and may re-enter later.
    Permanent,
}

/// Probability channel that triggered a driver substitution (Spec 126).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubstitutionChannel {
    /// Rank-independent medical, illness, or personal/family emergency.
    MedicalPersonal,
    /// Rank-dependent contract dispute, sponsorship, visa/travel logistics, or sporting penalty.
    AdministrativeContract,
}

/// Static narrative explanation template for a pre-race driver substitution.
#[derive(Debug, Clone, Copy)]
pub struct SubstitutionReasonTemplate {
    pub id: &'static str,
    pub channel: SubstitutionChannel,
    pub duration: ReplacementDuration,
    pub headline: &'static str,
    pub template: &'static str,
}

impl SubstitutionReasonTemplate {
    pub fn format_reason(&self, outgoing: &str, incoming: &str, team: &str, car: &str) -> String {
        self.template
            .replace("{outgoing}", outgoing)
            .replace("{incoming}", incoming)
            .replace("{team}", team)
            .replace("{car}", car)
    }
}

/// Pool of 13 narrative explanation templates (8 rank-independent medical/personal, 5 rank-dependent admin/contract).
pub const SUBSTITUTION_REASON_TEMPLATES: &[SubstitutionReasonTemplate] = &[
    // --- Channel A: Medical & Personal Reasons (Rank-Independent) ---
    SubstitutionReasonTemplate {
        id: "acute_appendicitis",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::Permanent,
        headline: "MEDICAL WITHDRAWAL — APPENDICITIS SURGERY",
        template: "{outgoing} was admitted to the local hospital overnight for emergency appendicitis surgery. {team} has signed {incoming} to take over the {car} for the remainder of the championship.",
    },
    SubstitutionReasonTemplate {
        id: "viral_gastroenteritis",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::OneRace,
        headline: "MEDICAL STAND-IN — ACUTE FOOD POISONING",
        template: "{outgoing} has been sidelined for this round by the FIA medical delegate due to acute viral gastroenteritis. {incoming} steps into the {team} {car} as a one-race stand-in.",
    },
    SubstitutionReasonTemplate {
        id: "wrist_sprain_sim",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::OneRace,
        headline: "INJURY STAND-IN — WRIST SPRAIN",
        template: "{outgoing} suffered a minor ligament sprain during high-torque simulator preparation and was advised to rest for one round. {team} has called up reserve driver {incoming} to pilot the {car} today.",
    },
    SubstitutionReasonTemplate {
        id: "inner_ear_vertigo",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::OneRace,
        headline: "MEDICAL PRECAUTION — VESTIBULAR LABYRINTHITIS",
        template: "Paddock doctors have rested {outgoing} for this event after diagnosing temporary inner-ear vertigo. {incoming} will deputize for {team} in the {car} for this round.",
    },
    SubstitutionReasonTemplate {
        id: "family_paternity_leave",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::OneRace,
        headline: "PERSONAL LEAVE — FAMILY EMERGENCY",
        template: "{outgoing} has flown home for the birth of their first child with full support from {team}. {incoming} takes the wheel of the {car} for this single race.",
    },
    SubstitutionReasonTemplate {
        id: "pre_event_testing_fracture",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::Permanent,
        headline: "TESTING ACCIDENT — COLLARBONE FRACTURE",
        template: "Following a heavy high-G impact during private mid-week testing, {outgoing} sustained a collarbone fracture requiring multi-week rehabilitation. {team} has drafted {incoming} into the {car}.",
    },
    SubstitutionReasonTemplate {
        id: "seasonal_influenza",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::OneRace,
        headline: "ILLNESS STAND-IN — HIGH FEVER & FLU",
        template: "{outgoing} woke up with a high fever and severe seasonal influenza this morning. {team} has registered {incoming} as a temporary one-race replacement in the {car}.",
    },
    SubstitutionReasonTemplate {
        id: "karting_rib_bruising",
        channel: SubstitutionChannel::MedicalPersonal,
        duration: ReplacementDuration::Permanent,
        headline: "TRAINING INJURY — RIB CARTILAGE DAMAGE",
        template: "{outgoing} aggravated a rib cartilage injury during an exhibition charity race and has stepped down from the active roster. {incoming} joins {team} in the {car} starting at 0 points.",
    },
    // --- Channel B: Administrative, Contract, Visa & Sporting Reasons (Rank-Dependent) ---
    SubstitutionReasonTemplate {
        id: "visa_customs_delay",
        channel: SubstitutionChannel::AdministrativeContract,
        duration: ReplacementDuration::OneRace,
        headline: "TRAVEL LOGISTICS — VISA CLEARANCE DELAY",
        template: "Administrative delays at border customs prevented {outgoing} from arriving before close of scrutineering. {team} has drafted {incoming} into the {car} for this round.",
    },
    SubstitutionReasonTemplate {
        id: "connecting_flight_grounded",
        channel: SubstitutionChannel::AdministrativeContract,
        duration: ReplacementDuration::OneRace,
        headline: "TRAVEL DISRUPTION — GROUNDED CHARTER FLIGHT",
        template: "Severe air-traffic closures stranded {outgoing}'s connecting flight en route to the circuit. {team} has handed the {car} to reserve driver {incoming} for today's event.",
    },
    SubstitutionReasonTemplate {
        id: "sponsor_contract_termination",
        channel: SubstitutionChannel::AdministrativeContract,
        duration: ReplacementDuration::Permanent,
        headline: "CONTRACT DISPUTE — TITLE SPONSOR RESTRUCTURING",
        template: "A mid-season title sponsor restructuring at {team} led to an immediate parting of ways with {outgoing}. {incoming} has signed to campaign the {car} from 0 championship points.",
    },
    SubstitutionReasonTemplate {
        id: "stewards_penalty_points_ban",
        channel: SubstitutionChannel::AdministrativeContract,
        duration: ReplacementDuration::OneRace,
        headline: "STEWARDS BULLETIN — ONE-RACE SUSPENSION",
        template: "After accumulating 12 penalty points on their competition license, {outgoing} must serve a mandatory one-race suspension. {incoming} fills in for {team} aboard the {car}.",
    },
    SubstitutionReasonTemplate {
        id: "endurance_clash_release",
        channel: SubstitutionChannel::AdministrativeContract,
        duration: ReplacementDuration::Permanent,
        headline: "TEAM RESTRUCTURING — FACTORY SEAT REASSIGNMENT",
        template: "Following irreconcilable contract and simulator scheduling disputes with {team} management, {outgoing} has been released from the {car} in favor of {incoming}.",
    },
];

/// Record of a single driver substitution event to be presented in the pre-race modal (Spec 126).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverSubstitutionEvent {
    pub round_index: usize,
    pub outgoing_driver_id: String,
    pub outgoing_driver_name: String,
    pub outgoing_points: u32,
    pub incoming_driver_id: String,
    pub incoming_driver_name: String,
    pub team_name: String,
    pub car_model_id: Option<String>,
    pub car_name: String,
    pub duration: ReplacementDuration,
    pub channel: SubstitutionChannel,
    pub reason_headline: String,
    pub reason_text: String,
}

/// Tracks a regular championship driver who was temporarily replaced for a single race (`OneRace`)
/// so they can be restored to their seat with their prior points before the next race begins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveTemporalStandIn {
    pub stand_in_driver_id: String,
    pub original_entry: SeriesStandingEntry,
    pub team_name: String,
    pub round_substituted: usize,
}

/// Resolves a valid catalog `car_model_id` and authentic motorsport constructor `team_name`
/// for a given discipline `module_id`, `tier`, optional `DriverCharacter`, and grid `slot_idx`.
pub fn resolve_car_and_team_for_driver(
    module_id: &str,
    tier: u8,
    character: Option<&DriverCharacter>,
    slot_idx: usize,
) -> (Option<String>, String) {
    let effective_mod = if module_id.trim().is_empty() { "gt" } else { module_id };
    let effective_tier = tier.max(1);

    let mut tier_models = crate::catalog::get_models_for_module_and_tier(effective_mod, effective_tier);
    if tier_models.is_empty() {
        tier_models = crate::catalog::get_models_for_module(effective_mod);
    }

    let chosen_model = character
        .and_then(|c| c.favorite_car_for_discipline_and_tier(effective_mod, effective_tier))
        .and_then(|fav_id| tier_models.iter().copied().find(|m| m.id == fav_id))
        .or_else(|| {
            if !tier_models.is_empty() {
                Some(tier_models[slot_idx % tier_models.len()])
            } else {
                None
            }
        });

    let suffixes = match DriverCharacter::normalize_discipline(effective_mod) {
        "gt" => &["Factory GT", "Corse", "Motorsport", "Customer Racing", "Performance"][..],
        "nascar" => &["Competition", "Speedworks", "Racing", "Vanguard Motorsport", "Garage"][..],
        "rally" => &["World RX Team", "Rallycross Works", "RX Motorsport", "Factory RX", "Motorsport"][..],
        "kart" => &["Factory Karting", "Kart Team", "Racing Academy", "Corse Kart", "Motorsport"][..],
        "extreme_offroad" => &["Off-Road Works", "Trophy Racing", "Dune Dynamics", "4x4 Motorsport", "Factory Raid"][..],
        _ => &["Cross Works", "Buggy Racing", "Autocross Team", "Motorsport", "Factory XC"][..],
    };
    let suffix = suffixes[slot_idx % suffixes.len()];

    if let Some(model) = chosen_model {
        let team_name = format!("{} {}", model.manufacturer, suffix);
        (Some(model.id.to_string()), team_name)
    } else {
        let fallback_team = character
            .and_then(|c| c.name.split_whitespace().last())
            .map(|last| format!("{} {}", last, suffix))
            .unwrap_or_else(|| format!("Independent {}", suffix));
        (None, fallback_team)
    }
}

/// Returns true if a team name is empty or a generic `<LastName> Racing` placeholder.
pub fn is_generic_team_name(team_name: &str, driver_name: &str) -> bool {
    let trimmed = team_name.trim();
    if trimmed.is_empty() || trimmed == "Independent Racing" || trimmed == "Rival Racing" {
        return true;
    }
    if let Some(last) = driver_name.split_whitespace().last() {
        if trimmed == format!("{} Racing", last) {
            return true;
        }
    }
    false
}

impl SeriesSession {
    /// Dynamically synthesizes a complete championship roster of `total_drivers` (1 player + `total_drivers - 1` AI)
    /// using uniform 1/6 driving style quotas, discrete bell-curve skill tiers centered on `tier`,
    /// and 100% valid catalog `car_model_id` and constructor teams.
    pub fn generate_dynamic_roster(
        name: impl Into<String>,
        module_id: &str,
        tier: u32,
        point_system: super::PointSystem,
        track_ids: Vec<String>,
        laps_per_round: u32,
        total_drivers: usize,
        player_team: &str,
        player_car_model_id: Option<&str>,
        seed: u64,
    ) -> Self {
        let name_str = name.into();
        let total = total_drivers.max(2);
        let bot_count = total - 1;
        let tier_u8 = tier.clamp(1, 7) as u8;
        let bell_tier = DriverTier::from_u8(tier.clamp(1, 5) as u8);

        let all_drivers = DriverCharacter::all_across_modules();
        let sampled_chars = DriverCharacter::sample_casual_race_roster(&all_drivers, bot_count, seed);
        let sampled_tiers = bell_tier.sample_grid_tiers(bot_count, seed.wrapping_add(777));

        let mut standings = Vec::with_capacity(total);

        let (default_player_car, default_player_team) =
            resolve_car_and_team_for_driver(module_id, tier_u8, None, 0);
        let resolved_player_car = player_car_model_id
            .map(|s| s.to_string())
            .or(default_player_car);
        let resolved_player_team = if player_team.trim().is_empty() {
            default_player_team
        } else {
            player_team.to_string()
        };

        let mut player_entry = SeriesStandingEntry::new("player", "Player", resolved_player_team);
        player_entry.car_model_id = resolved_player_car;
        standings.push(player_entry);

        for (idx, (char_def, driver_tier)) in sampled_chars.into_iter().zip(sampled_tiers.into_iter()).enumerate() {
            let (car_model_id, team_name) =
                resolve_car_and_team_for_driver(module_id, tier_u8, Some(&char_def), idx + 1);
            let mut entry = SeriesStandingEntry::new(char_def.id, char_def.name, team_name)
                .with_ai_style_and_tier(
                    Some(char_def.style.as_str().to_string()),
                    Some(driver_tier.to_u8()),
                );
            entry.car_model_id = car_model_id;
            standings.push(entry);
        }

        let mut session = Self::new(name_str, point_system, track_ids, laps_per_round, &[])
            .with_tier(tier)
            .with_module_id(module_id);
        session.standings = standings;
        session
    }

    /// Ensures every participant in `self.standings` has a valid `car_model_id`, `ai_style`, `ai_tier`,
    /// and constructor `team_name` for the session's `module_id` and `tier`.
    pub fn ensure_valid_roster_cars_and_teams(&mut self) {
        let effective_mod = if self.module_id.trim().is_empty() {
            "gt".to_string()
        } else {
            self.module_id.clone()
        };
        let tier_u8 = self.tier.clamp(1, 7) as u8;
        let bell_center = DriverTier::from_u8(self.tier.clamp(1, 5) as u8);
        let seed = super::weekend_seed(&self.name, self.tier as usize);
        let sampled_tiers = bell_center.sample_grid_tiers(self.standings.len().max(1), seed.wrapping_add(313));

        for (idx, entry) in self.standings.iter_mut().enumerate() {
            let char_opt = if entry.driver_id == "player" {
                None
            } else {
                DriverCharacter::find_global(&entry.driver_id)
            };

            if entry.driver_id != "player" {
                if entry.ai_style.is_none() {
                    let style = char_opt
                        .as_ref()
                        .map(|c| c.style)
                        .unwrap_or(DrivingStyle::ALL[idx % DrivingStyle::ALL.len()]);
                    entry.ai_style = Some(style.as_str().to_string());
                }
                if entry.ai_tier.is_none() {
                    let assigned_tier = sampled_tiers.get(idx).copied().unwrap_or(bell_center);
                    entry.ai_tier = Some(assigned_tier.to_u8());
                }
            }

            let has_valid_car = entry
                .car_model_id
                .as_deref()
                .and_then(crate::catalog::find_model_by_id)
                .is_some();

            let (resolved_car, resolved_team) =
                resolve_car_and_team_for_driver(&effective_mod, tier_u8, char_opt.as_ref(), idx);

            if !has_valid_car {
                entry.car_model_id = resolved_car;
            }
            if entry.driver_id != "player" && is_generic_team_name(&entry.team_name, &entry.driver_name) {
                entry.team_name = resolved_team;
            }
        }
    }

    /// Restores any regular drivers who were temporarily replaced for 1 race (`ReplacementDuration::OneRace`)
    /// back into `self.standings` with their prior championship points intact.
    pub fn restore_expired_temporal_replacements(&mut self) -> Vec<SeriesStandingEntry> {
        if self.active_stand_ins.is_empty() {
            return Vec::new();
        }

        let stand_ins = std::mem::take(&mut self.active_stand_ins);
        let mut restored = Vec::with_capacity(stand_ins.len());

        for stand_in in stand_ins {
            if let Some(slot) = self
                .standings
                .iter_mut()
                .find(|s| s.driver_id == stand_in.stand_in_driver_id)
            {
                *slot = stand_in.original_entry.clone();
            } else if !self
                .standings
                .iter()
                .any(|s| s.driver_id == stand_in.original_entry.driver_id)
            {
                self.standings.push(stand_in.original_entry.clone());
            }
            restored.push(stand_in.original_entry);
        }

        if !self.is_new() {
            self.sort_standings();
        }
        restored
    }

    /// Evaluates pre-race driver substitutions across the entire active AI roster before a championship race begins.
    ///
    /// 1. Restores any `OneRace` temporal stand-ins from the previous race so the regular driver returns with prior points.
    /// 2. Traverses all eligible AI drivers in current standing rank order $r_i \in [0, M-1]$:
    ///    - Medical / personal reasons have rank-independent probability $p_{\text{med}}(i) = \frac{0.06}{M} \cdot \gamma^k$.
    ///    - Administrative / contract / visa reasons have rank-dependent probability $p_{\text{admin}}(i) = \frac{0.12}{M} \cdot (0.10 + 2.70 u_i^2) \cdot \gamma^k$,
    ///      making top-ranked drivers much less likely to be replaced than backmarkers.
    ///    - Multi-replacement damping $\gamma = 0.12$ keeps single replacements moderate (~16% per race) and multiple replacements very rare (< 0.2%).
    /// 3. Incoming replacement drivers start with 0 championship points and inherit the team and car model of the seat.
    /// 4. Permanently replaced drivers return to the global pool and can re-enter in future races on the same or a different team.
    pub fn evaluate_pre_race_substitutions(&mut self, seed: u64) -> Vec<DriverSubstitutionEvent> {
        // Never substitute mid-tournament (between heats/semis/finals of a single round)
        if self.tournament_in_progress() || self.is_completed {
            return Vec::new();
        }

        // 1. Restore any 1-race stand-ins from the prior race
        self.restore_expired_temporal_replacements();

        // Collect indices of eligible AI drivers (must be in the global 72-character pool, excluding "player")
        let ai_indices: Vec<usize> = self
            .standings
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.driver_id != "player"
                    && DriverCharacter::find_global(&s.driver_id).is_some()
            })
            .map(|(idx, _)| idx)
            .collect();

        let m = ai_indices.len();
        if m == 0 {
            return Vec::new();
        }

        let mut rng = LcgRng::new(seed.wrapping_add(0x5B57_1707_E551_CE01));
        let all_characters = DriverCharacter::all_across_modules();
        let bell_center = DriverTier::from_u8(self.tier.clamp(1, 5) as u8);

        let med_templates: Vec<&SubstitutionReasonTemplate> = SUBSTITUTION_REASON_TEMPLATES
            .iter()
            .filter(|t| t.channel == SubstitutionChannel::MedicalPersonal)
            .collect();
        let admin_templates: Vec<&SubstitutionReasonTemplate> = SUBSTITUTION_REASON_TEMPLATES
            .iter()
            .filter(|t| t.channel == SubstitutionChannel::AdministrativeContract)
            .collect();

        let lambda_med = 0.06_f32;
        let lambda_admin = 0.12_f32;
        let gamma = 0.12_f32;

        let mut k: u32 = 0;
        let mut events = Vec::new();
        let mut new_stand_ins = Vec::new();

        for (rank_idx, &standing_idx) in ai_indices.iter().enumerate() {
            let u_rank = if m > 1 {
                (rank_idx as f32) / ((m - 1) as f32)
            } else {
                0.5
            };

            let damp = gamma.powi(k as i32);
            let p_med = (lambda_med / (m as f32)) * damp;
            let p_admin = (lambda_admin / (m as f32)) * (0.10 + 2.70 * u_rank * u_rank) * damp;

            let roll = rng.next_f32();
            let triggered_channel = if roll < p_med {
                Some(SubstitutionChannel::MedicalPersonal)
            } else if roll < p_med + p_admin {
                Some(SubstitutionChannel::AdministrativeContract)
            } else {
                None
            };

            let Some(channel) = triggered_channel else {
                continue;
            };

            // Sample a target driving style uniformly across all 6 styles (P = 1/6)
            let style_idx = (rng.next_u32() as usize) % DrivingStyle::ALL.len();
            let target_style = DrivingStyle::ALL[style_idx];

            // Available pool: global characters not currently in standings and not currently in new_stand_ins
            let is_busy = |cid: &str, standings: &[SeriesStandingEntry], stand_ins: &[ActiveTemporalStandIn]| -> bool {
                standings.iter().any(|s| s.driver_id == cid)
                    || stand_ins.iter().any(|si| si.original_entry.driver_id == cid || si.stand_in_driver_id == cid)
            };

            let mut candidates: Vec<DriverCharacter> = all_characters
                .iter()
                .filter(|c| c.style == target_style && !is_busy(c.id, &self.standings, &new_stand_ins))
                .cloned()
                .collect();

            if candidates.is_empty() {
                candidates = all_characters
                    .iter()
                    .filter(|c| !is_busy(c.id, &self.standings, &new_stand_ins))
                    .cloned()
                    .collect();
            }

            if candidates.is_empty() {
                continue;
            }

            rng.shuffle(&mut candidates);
            let incoming_char = &candidates[0];
            let incoming_tier = bell_center.sample_grid_tiers(1, rng.next_u32() as u64)[0];

            let templates = match channel {
                SubstitutionChannel::MedicalPersonal => &med_templates,
                SubstitutionChannel::AdministrativeContract => &admin_templates,
            };
            let tpl = templates[(rng.next_u32() as usize) % templates.len()];

            let outgoing_entry = self.standings[standing_idx].clone();
            let car_name = outgoing_entry
                .car_model_id
                .as_deref()
                .and_then(crate::catalog::find_model_by_id)
                .map(|m| m.name.to_string())
                .unwrap_or_else(|| "Team Competition Car".to_string());

            let reason_text = tpl.format_reason(
                &outgoing_entry.driver_name,
                incoming_char.name,
                &outgoing_entry.team_name,
                &car_name,
            );

            if tpl.duration == ReplacementDuration::OneRace {
                new_stand_ins.push(ActiveTemporalStandIn {
                    stand_in_driver_id: incoming_char.id.to_string(),
                    original_entry: outgoing_entry.clone(),
                    team_name: outgoing_entry.team_name.clone(),
                    round_substituted: self.current_round,
                });
            }

            // Replacement driver enters with 0 points, inheriting the team and car model of the seat
            // while the team retains 100% of its constructor points (Spec 127)
            self.replace_driver_in_seat(
                standing_idx,
                incoming_char.id,
                incoming_char.name,
                Some(incoming_char.style.as_str().to_string()),
                Some(incoming_tier.to_u8()),
            );

            events.push(DriverSubstitutionEvent {
                round_index: self.current_round,
                outgoing_driver_id: outgoing_entry.driver_id,
                outgoing_driver_name: outgoing_entry.driver_name,
                outgoing_points: outgoing_entry.points,
                incoming_driver_id: incoming_char.id.to_string(),
                incoming_driver_name: incoming_char.name.to_string(),
                team_name: outgoing_entry.team_name,
                car_model_id: outgoing_entry.car_model_id,
                car_name,
                duration: tpl.duration,
                channel,
                reason_headline: tpl.headline.to_string(),
                reason_text,
            });

            k += 1;
        }

        self.active_stand_ins.extend(new_stand_ins);

        if !events.is_empty() && !self.is_new() {
            self.sort_standings();
        }

        self.pending_substitutions.extend(events.clone());
        events
    }
}
