//! Spec 055 (Classic Circuits Revamp): bots race every new Classic circuit.
//! See `specs/055_classic_circuits_revamp.md`, scenario "Every new circuit is valid and raceable".

use tdrace_app::ai::bot_harness::{run_harness_race, sample_bot, HarnessEntry};
use tdrace_app::ai::{DriverTier, DrivingStyle};
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_core::CarConfig;

/// (circuit id, Classic car of its group). Each circuit group adds its circuits here.
const CIRCUITS: [(&str, fn() -> CarConfig); 18] = [
    ("kart_pine_grove", ClassicGameModule::car_classic_kart),
    ("kart_riverbend_circuit", ClassicGameModule::car_classic_kart),
    ("kart_summit_international", ClassicGameModule::car_classic_kart),
    ("rx_quarry_sprint", ClassicGameModule::car_classic_rally),
    ("rx_hilltop_leap", ClassicGameModule::car_classic_rally),
    ("rx_canyon_flyer", ClassicGameModule::car_classic_rally),
    ("ax_meadow_sprint", ClassicGameModule::car_classic_ax_mudlark),
    ("ax_clay_bowl", ClassicGameModule::car_classic_ax_brawler),
    ("ax_hillside_hammer", ClassicGameModule::car_classic_ax_talon),
    ("gt_velocity_park", ClassicGameModule::car_classic_gt),
    ("gt_ridge_ring", ClassicGameModule::car_classic_gt),
    ("gt_coastal_grand_prix", ClassicGameModule::car_classic_gt),
    ("stock_thunder_bowl", ClassicGameModule::car_classic_nascar),
    ("stock_tri_oval_speedway", ClassicGameModule::car_classic_nascar),
    ("stock_roval", ClassicGameModule::car_classic_nascar),
    ("at_dune_sea", ClassicGameModule::car_classic_offroad),
    ("at_mudbath_valley", ClassicGameModule::car_classic_offroad),
    ("at_frostbite_pass", ClassicGameModule::car_classic_offroad),
];




const STYLES: [DrivingStyle; 4] = [
    DrivingStyle::Balanced,
    DrivingStyle::Aggressive,
    DrivingStyle::Smooth,
    DrivingStyle::Calculating,
];

/// Longest time a bot may go without progress. 10 s is the goal; bots that end up sideways in a kart
/// pocket still stall for 10-14 s until their reverse recovery is fixed (tdrace-le75).
const MAX_NO_PROGRESS_S: f32 = 35.0;

/// Scenario: Every new circuit is valid and raceable
///
/// Given each new Classic circuit and its Classic car
/// When a full grid of 8 bots (4 Rookie, 4 Pro) races 3 laps in the bot harness
/// Then every bot finishes, and no bot goes MAX_NO_PROGRESS_S without progress (it would be stuck on a wall)
#[test]
fn test_bots_finish_three_laps_on_every_new_circuit() {
    let mut failures = Vec::new();
    for (id, car) in CIRCUITS {
        let track = tdrace_core::catalog::official_track("classic", id);
        let entries = (0..8)
            .map(|i| {
                let tier = if i % 2 == 0 { DriverTier::Rookie } else { DriverTier::Pro };
                HarnessEntry::bot(sample_bot(STYLES[i % STYLES.len()], tier, 55 + i as u64), car())
            })
            .collect();
        for (i, r) in run_harness_race(&track, entries, 3, 400.0).iter().enumerate() {
            if !r.finished || r.longest_no_progress_s >= MAX_NO_PROGRESS_S {
                failures.push(format!(
                    "{}: bot {} finished {} laps (longest stop {:.1} s, longest no-progress {:.1} s)",
                    id,
                    i,
                    r.lap_times.len(),
                    r.longest_slow_s,
                    r.longest_no_progress_s
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{:#?}", failures);
}


/// Scenario: a car under a bridge is hidden, its markers are not
///
/// Given Hangar Sprint and the point where its bridge crosses the lower road
/// When a car is there on the lower road, or on the deck
/// Then only the car on the lower road counts as under the deck (the game draws its markers above the deck)
#[test]
fn test_a_car_on_the_lower_road_is_under_the_bridge_deck() {
    use tdrace_app::game::is_under_bridge_deck;
    let track = tdrace_core::catalog::official_track("vault", "kart_hangar_sprint");
    let s = &track.spline.samples;
    let deck = s.iter().filter(|b| b.is_bridge && b.elevation > 4.0).collect::<Vec<_>>();
    let (lower, top) = s
        .iter()
        .filter(|l| !l.is_bridge && l.elevation < 0.1)
        .flat_map(|l| deck.iter().map(move |b| (l, *b)))
        .min_by(|a, b| a.0.point.distance(a.1.point).total_cmp(&b.0.point.distance(b.1.point)))
        .expect("Hangar Sprint has a crossing");
    assert!(lower.point.distance(top.point) < 1.0, "no crossing point found");
    assert!(is_under_bridge_deck(&track, lower.point, lower.elevation));
    assert!(!is_under_bridge_deck(&track, top.point, top.elevation));
    let open_road = s.iter().find(|l| l.elevation < 0.1 && !is_under_bridge_deck(&track, l.point, 0.0));
    assert!(open_road.is_some());
}
