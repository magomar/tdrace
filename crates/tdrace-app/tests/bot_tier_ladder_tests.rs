//! Spec 083 (EvoRL bot performance calibration): regression gate for the tier pace ladder.
//! See `specs/083_evorl_bot_driving_style_and_performance_optimization.md`, scenario
//! "Regression gate in cargo test". Run with `--release`; the debug build is too slow.

use tdrace_app::ai::bot_harness::{run_keyboard_reference_race, run_style_grids, StyleGridRun, KEYBOARD_REFERENCE_TRACKS, SAMPLE_TRACKS};
use tdrace_app::ai::DriverTier;

const TIERS: [DriverTier; 5] = [DriverTier::Rookie, DriverTier::Amateur, DriverTier::Contender, DriverTier::Pro, DriverTier::Legend];
const PLUS_TIERS: [DriverTier; 4] = [DriverTier::Pro, DriverTier::ProPlus, DriverTier::Legend, DriverTier::LegendPlus];
const LAPS: u32 = 4;

/// Mean flying lap of a tier's six-car style grid on one track.
fn tier_mean(runs: &[StyleGridRun], track: &str, tier: DriverTier) -> f32 {
    let run = runs.iter().find(|r| r.track == track && r.tier == tier).expect("grid ran");
    let laps: Vec<f32> = run.results.iter().filter_map(|c| c.mean_flying_lap()).collect();
    assert_eq!(laps.len(), run.results.len(), "{track} {}: every bot must set flying laps", tier.short_name());
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

/// Spec 131: T4+ and T5+ have zero human mistakes, outperform T4/T5, and T5+ beats the keyboard reference driver.
#[test]
fn test_spec131_error_free_t4_plus_and_t5_plus_pace_and_zero_mistakes() {
    let runs = run_style_grids(&PLUS_TIERS, LAPS);
    let mut failures = Vec::new();
    for (track, _) in SAMPLE_TRACKS {
        let t4 = tier_mean(&runs, track, DriverTier::Pro);
        let t4_plus = tier_mean(&runs, track, DriverTier::ProPlus);
        let t5 = tier_mean(&runs, track, DriverTier::Legend);
        let t5_plus = tier_mean(&runs, track, DriverTier::LegendPlus);

        for plus_tier in [DriverTier::ProPlus, DriverTier::LegendPlus] {
            let run = runs.iter().find(|r| r.track == track && r.tier == plus_tier).expect("plus grid ran");
            let mistakes: u32 = run.results.iter().map(|c| c.stats.total_mistakes()).sum();
            if mistakes != 0 {
                failures.push(format!("{track} {}: expected 0 human mistakes, got {mistakes}", plus_tier.short_name()));
            }
        }

        if t4_plus >= t4 {
            failures.push(format!("{track}: T4+ ({t4_plus:.2} s) is not faster than T4 ({t4:.2} s)"));
        }
        if t5_plus >= t5 {
            failures.push(format!("{track}: T5+ ({t5_plus:.2} s) is not faster than T5 ({t5:.2} s)"));
        }
        if t5_plus >= t4_plus {
            failures.push(format!("{track}: T5+ ({t5_plus:.2} s) is not faster than T4+ ({t4_plus:.2} s)"));
        }
    }

    for &t_idx in &KEYBOARD_REFERENCE_TRACKS {
        let track = SAMPLE_TRACKS[t_idx].0;
        let (reference, _) = run_keyboard_reference_race(t_idx, LAPS);
        let ref_lap = reference.mean_flying_lap().expect("reference flying lap");
        let t5_plus = tier_mean(&runs, track, DriverTier::LegendPlus);
        if t5_plus >= ref_lap {
            failures.push(format!("{track}: T5+ ({t5_plus:.2} s) did not beat keyboard reference ({ref_lap:.2} s)"));
        }
    }

    assert!(failures.is_empty(), "Spec 131 pace/error-free checks failed:\n{}", failures.join("\n"));
}


