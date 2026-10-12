//! Spec 083 (EvoRL bot performance calibration): regression gate for the tier pace ladder.
//! See `specs/083_evorl_bot_driving_style_and_performance_optimization.md`, scenario
//! "Regression gate in cargo test". Run with `--release`; the debug build is too slow.

use tdrace_app::ai::bot_harness::{run_style_grids, StyleGridRun, SAMPLE_TRACKS};
use tdrace_app::ai::DriverTier;

const TIERS: [DriverTier; 5] = [DriverTier::Rookie, DriverTier::Amateur, DriverTier::Contender, DriverTier::Pro, DriverTier::Legend];
const LAPS: u32 = 4;

/// Mean flying lap of a tier's six-car style grid on one track.
fn tier_mean(runs: &[StyleGridRun], track: &str, tier: DriverTier) -> f32 {
    let run = runs.iter().find(|r| r.track == track && r.tier == tier).expect("grid ran");
    let laps: Vec<f32> = run.results.iter().filter_map(|c| c.mean_flying_lap()).collect();
    assert_eq!(laps.len(), run.results.len(), "{track} T{}: every bot must set flying laps", tier.to_u8());
    laps.iter().sum::<f32>() / laps.len() as f32
}

/// Scenario: Regression gate in cargo test
///
/// Given the six-style grids of every tier on the 4 sample tracks
/// When they race 4 laps
/// Then mean flying laps are strictly ordered T1 > T2 > T3 > T4 > T5 on every track
#[test]
fn test_tier_pace_ladder_holds_on_every_sample_track() {
    let runs = run_style_grids(&TIERS, LAPS);
    let mut failures = Vec::new();
    for (track, _) in SAMPLE_TRACKS {
        let means: Vec<f32> = TIERS.iter().map(|&t| tier_mean(&runs, track, t)).collect();
        for (k, pair) in means.windows(2).enumerate() {
            if pair[0] <= pair[1] {
                failures.push(format!("{track}: T{} {:.2} s is not slower than T{} {:.2} s", k + 1, pair[0], k + 2, pair[1]));
            }
        }
    }
    assert!(failures.is_empty(), "tier ladder broken:\n{}", failures.join("\n"));
}
