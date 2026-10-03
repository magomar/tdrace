//! Tests for post-race itemized garage repair invoice and archetype pricing (Spec 078 Section 6).

use tdrace_app::profile::ItemizedRepairInvoice;
use tdrace_core::{EnginePlacement, SuspensionArchetype};

#[test]
fn test_itemized_garage_repair_invoice_archetype_pricing() {
    let tier = 1; // Base purse = 5,000 Cr
    let earned_purse = 5_000;
    let player_credits = 30_000; // Well above $1,000 Cr safety net floor

    // 1. Suspension archetype pricing differentiation
    // Identical suspension damage (50% health, H = 0.50) across all archetypes
    let inv_pushrod = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::FrontEngine,
        SuspensionArchetype::PushrodInboard,
        SuspensionArchetype::PushrodInboard,
        1.0,
        1.0,
        [0.50, 0.50, 0.50, 0.50],
    );

    let inv_double_wishbone = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::FrontEngine,
        SuspensionArchetype::DoubleWishbone,
        SuspensionArchetype::DoubleWishbone,
        1.0,
        1.0,
        [0.50, 0.50, 0.50, 0.50],
    );

    let inv_macpherson = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::FrontEngine,
        SuspensionArchetype::MacPhersonStrut,
        SuspensionArchetype::MacPhersonStrut,
        1.0,
        1.0,
        [0.50, 0.50, 0.50, 0.50],
    );

    let inv_kart = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::FrontEngine,
        SuspensionArchetype::RigidKart,
        SuspensionArchetype::RigidKart,
        1.0,
        1.0,
        [0.50, 0.50, 0.50, 0.50],
    );

    // Pushrod (k=2.20) must be more expensive than Double Wishbone (k=1.20)
    assert!(
        inv_pushrod.suspension_costs[0] > inv_double_wishbone.suspension_costs[0],
        "Pushrod ({}) must cost more to repair than Double Wishbone ({})",
        inv_pushrod.suspension_costs[0],
        inv_double_wishbone.suspension_costs[0]
    );

    // Double Wishbone (k=1.20) must be more expensive than MacPherson (k=0.80)
    assert!(
        inv_double_wishbone.suspension_costs[0] > inv_macpherson.suspension_costs[0],
        "Double Wishbone ({}) must cost more than MacPherson ({})",
        inv_double_wishbone.suspension_costs[0],
        inv_macpherson.suspension_costs[0]
    );

    // MacPherson (k=0.80) must be more expensive than Rigid Kart (k=0.50)
    assert!(
        inv_macpherson.suspension_costs[0] > inv_kart.suspension_costs[0],
        "MacPherson ({}) must cost more than Rigid Kart ({})",
        inv_macpherson.suspension_costs[0],
        inv_kart.suspension_costs[0]
    );

    // 2. Engine placement pricing differentiation
    // Identical engine damage (50% health, H = 0.50)
    let inv_front_eng = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::FrontEngine,
        SuspensionArchetype::DoubleWishbone,
        SuspensionArchetype::DoubleWishbone,
        1.0,
        0.50,
        [1.0, 1.0, 1.0, 1.0],
    );

    let inv_mid_eng = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::MidEngine,
        SuspensionArchetype::DoubleWishbone,
        SuspensionArchetype::DoubleWishbone,
        1.0,
        0.50,
        [1.0, 1.0, 1.0, 1.0],
    );

    let inv_rear_eng = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::RearEngine,
        SuspensionArchetype::DoubleWishbone,
        SuspensionArchetype::DoubleWishbone,
        1.0,
        0.50,
        [1.0, 1.0, 1.0, 1.0],
    );

    assert!(
        inv_rear_eng.engine_cost > inv_mid_eng.engine_cost,
        "RearEngine flat-6 labor ({}) must cost more than MidEngine ({})",
        inv_rear_eng.engine_cost,
        inv_mid_eng.engine_cost
    );
    assert!(
        inv_mid_eng.engine_cost > inv_front_eng.engine_cost,
        "MidEngine labor ({}) must cost more than FrontEngine ({})",
        inv_mid_eng.engine_cost,
        inv_front_eng.engine_cost
    );

    // 3. 40% Purse Cap Invariant
    // Totally destroyed vehicle (all health = 0.0)
    let inv_totaled = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::RearEngine,
        SuspensionArchetype::PushrodInboard,
        SuspensionArchetype::PushrodInboard,
        0.0,
        0.0,
        [0.0, 0.0, 0.0, 0.0],
    );

    let max_allowed_deduction = ((earned_purse as f64) * 0.40).round() as u64; // 2,000 Cr
    assert!(
        inv_totaled.total_raw_damage_cost > max_allowed_deduction,
        "Totaled car raw damage ({}) should exceed 40% purse cap ({})",
        inv_totaled.total_raw_damage_cost,
        max_allowed_deduction
    );
    assert_eq!(
        inv_totaled.net_deduction, max_allowed_deduction,
        "Net deduction must be strictly capped at 40% of earned purse"
    );
    assert_eq!(
        inv_totaled.sponsor_subsidy,
        inv_totaled.total_raw_damage_cost - max_allowed_deduction,
        "Sponsor subsidy must absorb all excess costs above 40% cap"
    );

    // 4. Anti-Bankruptcy Sponsor Floor (< $1,000 Cr bank balance)
    let low_credits = 450; // Bank balance < $1,000 Cr
    let inv_subsidized = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        low_credits,
        EnginePlacement::FrontEngine,
        SuspensionArchetype::DoubleWishbone,
        SuspensionArchetype::DoubleWishbone,
        0.10, // 10% health
        0.10,
        [0.10, 0.10, 0.10, 0.10],
    );

    assert!(
        inv_subsidized.sponsor_safety_net_applied,
        "Sponsor mechanic safety net must trigger for driver with < $1,000 Cr"
    );

    // Cost should match cost from 50% baseline instead of 10%
    let inv_baseline_50 = ItemizedRepairInvoice::calculate(
        tier,
        earned_purse,
        player_credits,
        EnginePlacement::FrontEngine,
        SuspensionArchetype::DoubleWishbone,
        SuspensionArchetype::DoubleWishbone,
        0.50,
        0.50,
        [0.50, 0.50, 0.50, 0.50],
    );

    assert_eq!(
        inv_subsidized.chassis_cost, inv_baseline_50.chassis_cost,
        "Sponsor mechanic must provide free repairs from 10% up to 50%"
    );
    assert_eq!(
        inv_subsidized.engine_cost, inv_baseline_50.engine_cost,
        "Sponsor mechanic must provide free repairs from 10% up to 50% for engine"
    );
}
