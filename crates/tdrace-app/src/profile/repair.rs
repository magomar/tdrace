//! Itemized post-race vehicle damage and garage repair economics (Spec 078 Section 6).

use serde::{Deserialize, Serialize};
use tdrace_core::{EnginePlacement, SuspensionArchetype};

use super::ModuleCareerProgress;

fn round_to_10(val: f64) -> u64 {
    if val <= 0.0 {
        0
    } else {
        ((val / 10.0).round() * 10.0) as u64
    }
}

/// Itemized post-race repair invoice detailing costs per vehicle subsystem (Spec 078).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemizedRepairInvoice {
    /// Cost to repair chassis monocoque to 100% integrity (Credits).
    pub chassis_cost: u64,
    /// Cost to overhaul engine block and cooling lines to 100% health (Credits).
    pub engine_cost: u64,
    /// 4-corner suspension assembly repair costs [FL, FR, RL, RR] (Credits).
    pub suspension_costs: [u64; 4],
    /// Sum of all component repair costs before subsidies and caps (Credits).
    pub total_raw_damage_cost: u64,
    /// Excess damage waived by sponsor subsidy when invoice exceeds 40% of purse (Credits).
    pub sponsor_subsidy: u64,
    /// Net credits deducted from player race purse / wallet (Credits).
    pub net_deduction: u64,
    /// Whether the sponsor mechanic patched components below 50% free of charge.
    pub sponsor_safety_net_applied: bool,
}

impl ItemizedRepairInvoice {
    /// Calculates the itemized post-race repair invoice according to Spec 078 Section 6.
    ///
    /// # Formulas:
    /// - `Cost_chassis = round_to_10(B_purse(tier) * 0.15 * (1.0 - H_chassis)^1.2)`
    /// - `Cost_engine  = round_to_10(B_purse(tier) * 0.25 * k_powertrain * (1.0 - H_engine)^1.4)`
    /// - `Cost_susp[i] = round_to_10(B_purse(tier) * 0.08 * k_part_cost(Archetype) * (1.0 - H_susp[i])^1.2)`
    ///
    /// Clamps net deduction to <= 40% of `earned_purse`, and applies free 50% repairs if
    /// `player_credits < 1_000 Cr`.
    pub fn calculate(
        tier: u32,
        earned_purse: u64,
        player_credits: u64,
        placement: EnginePlacement,
        front_archetype: SuspensionArchetype,
        rear_archetype: SuspensionArchetype,
        chassis_health: f32,
        engine_health: f32,
        suspension_health: [f32; 4],
    ) -> Self {
        let b_purse = ModuleCareerProgress::round_base_purse(tier) as f64;

        // Anti-bankruptcy sponsor safety net floor:
        // Free repairs up to 50% for all components if liquid balance is under $1,000 Cr
        let eligible_for_safety_net = player_credits < 1_000;
        let mut sponsor_safety_net_applied = false;

        let effective_chassis_h = if eligible_for_safety_net && chassis_health < 0.50 {
            sponsor_safety_net_applied = true;
            0.50
        } else {
            chassis_health.clamp(0.0, 1.0)
        };

        let effective_engine_h = if eligible_for_safety_net && engine_health < 0.50 {
            sponsor_safety_net_applied = true;
            0.50
        } else {
            engine_health.clamp(0.0, 1.0)
        };

        let mut effective_susp_h = [1.0f32; 4];
        for i in 0..4 {
            let h = suspension_health[i];
            effective_susp_h[i] = if eligible_for_safety_net && h < 0.50 {
                sponsor_safety_net_applied = true;
                0.50
            } else {
                h.clamp(0.0, 1.0)
            };
        }

        // 1. Chassis Repair Cost
        let chassis_cost = round_to_10(b_purse * 0.15 * (1.0 - effective_chassis_h as f64).powf(1.2));

        // 2. Engine Repair Cost
        let k_powertrain = placement.repair_cost_multiplier() as f64;
        let engine_cost = round_to_10(b_purse * 0.25 * k_powertrain * (1.0 - effective_engine_h as f64).powf(1.4));

        // 3. 4-Corner Suspension Repair Costs
        let mut suspension_costs = [0u64; 4];
        for i in 0..4 {
            let arch = if i < 2 { front_archetype } else { rear_archetype };
            let k_part = arch.part_cost_multiplier() as f64;
            suspension_costs[i] = round_to_10(b_purse * 0.08 * k_part * (1.0 - effective_susp_h[i] as f64).powf(1.2));
        }

        let total_raw_damage_cost = chassis_cost + engine_cost + suspension_costs.iter().sum::<u64>();

        // 4. Anti-Bankruptcy Sponsor Protections (Clamped to <= 40% of earned purse)
        let max_purse_deduction = ((earned_purse as f64) * 0.40).round() as u64;

        let (sponsor_subsidy, net_deduction) = if total_raw_damage_cost > max_purse_deduction {
            (total_raw_damage_cost - max_purse_deduction, max_purse_deduction)
        } else {
            (0, total_raw_damage_cost)
        };

        Self {
            chassis_cost,
            engine_cost,
            suspension_costs,
            total_raw_damage_cost,
            sponsor_subsidy,
            net_deduction,
            sponsor_safety_net_applied,
        }
    }
}
