//! Spec 055 (Classic Circuits Revamp): bots race every new Classic circuit.
//! See `specs/055_classic_circuits_revamp.md`, scenario "Every new circuit is valid and raceable".

use tdrace_app::ai::bot_harness::{run_harness_race, sample_bot, HarnessEntry};
use tdrace_app::ai::{DriverTier, DrivingStyle};
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_core::CarConfig;

/// (circuit id, Classic car of its group). Each circuit group adds its circuits here.
const CIRCUITS: [(&str, fn() -> CarConfig); 3] = [
    ("kart_hangar_sprint", ClassicGameModule::car_classic_kart),
    ("kart_warehouse_twister", ClassicGameModule::car_classic_kart),
    ("kart_tower_labyrinth", ClassicGameModule::car_classic_kart),
];

const STYLES: [DrivingStyle; 4] = [
    DrivingStyle::Balanced,
    DrivingStyle::Aggressive,
    DrivingStyle::Smooth,
    DrivingStyle::Calculating,
];

/// Scenario: Every new circuit is valid and raceable
///
/// Given each new Classic circuit and its Classic car
/// When 4 bots race 2 laps in the bot harness
/// Then every bot finishes both laps
#[test]
fn test_bots_finish_two_laps_on_every_new_circuit() {
    let mut failures = Vec::new();
    for (id, car) in CIRCUITS {
        let track = tdrace_core::catalog::official_track("classic", id);
        let entries = STYLES
            .iter()
            .enumerate()
            .map(|(i, style)| HarnessEntry::bot(sample_bot(*style, DriverTier::Amateur, 55 + i as u64), car()))
            .collect();
        for (i, r) in run_harness_race(&track, entries, 2, 600.0).iter().enumerate() {
            if !r.finished || r.lap_times.len() < 2 {
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
