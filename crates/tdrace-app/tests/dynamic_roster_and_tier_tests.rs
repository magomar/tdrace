use std::collections::{HashMap, HashSet};
use tdrace_app::ai::{DriverCharacter, DrivingStyle};

#[test]
fn test_uniform_style_roster_sampling_quotas_and_uniqueness() {
    let all_drivers = DriverCharacter::all_across_modules();
    assert_eq!(all_drivers.len(), 72);

    for n in 1..=24 {
        for seed in [42, 1337, 99999, 1234567, 7654321] {
            let roster = DriverCharacter::sample_casual_race_roster(&all_drivers, n, seed);
            assert_eq!(roster.len(), n, "Requested {} drivers, got {}", n, roster.len());

            // 1. Uniqueness check
            let mut ids = HashSet::new();
            for driver in &roster {
                assert!(ids.insert(driver.id), "Duplicate driver ID '{}' found for n={} seed={}", driver.id, n, seed);
            }

            // 2. Balanced quotas check
            let mut style_counts = HashMap::new();
            for driver in &roster {
                *style_counts.entry(driver.style).or_insert(0) += 1;
            }

            let base_quota = n / 6;
            let remainder = n % 6;

            let mut count_plus_one = 0;
            let mut count_base = 0;

            for style in DrivingStyle::ALL {
                let count = style_counts.get(&style).copied().unwrap_or(0);
                if count == base_quota + 1 {
                    count_plus_one += 1;
                } else if count == base_quota {
                    count_base += 1;
                } else {
                    panic!(
                        "Style {:?} count {} is not base_quota ({}) or base_quota+1 ({}) for n={} seed={}",
                        style, count, base_quota, base_quota + 1, n, seed
                    );
                }
            }

            assert_eq!(count_plus_one, remainder, "Expected {} styles with extra quota slot for n={}", remainder, n);
            assert_eq!(count_base, 6 - remainder, "Expected {} styles with base quota for n={}", 6 - remainder, n);
        }
    }
}

#[test]
fn test_uniform_style_roster_sampling_determinism() {
    let all_drivers = DriverCharacter::all_across_modules();
    let seed = 424242;

    let roster_a = DriverCharacter::sample_casual_race_roster(&all_drivers, 8, seed);
    let roster_b = DriverCharacter::sample_casual_race_roster(&all_drivers, 8, seed);

    let ids_a: Vec<&str> = roster_a.iter().map(|d| d.id).collect();
    let ids_b: Vec<&str> = roster_b.iter().map(|d| d.id).collect();
    assert_eq!(ids_a, ids_b, "Sampling must be strictly deterministic given same seed");

    let roster_c = DriverCharacter::sample_casual_race_roster(&all_drivers, 8, seed + 1);
    let ids_c: Vec<&str> = roster_c.iter().map(|d| d.id).collect();
    assert_ne!(ids_a, ids_c, "Different seeds should produce different grids");
}

#[test]
fn test_uniform_distribution_across_styles_over_many_trials() {
    let mut style_first_counts = HashMap::new();
    let trials = 1200;

    for seed in 0..trials {
        let single = DriverCharacter::sample_casual_race_opponents(1, seed);
        assert_eq!(single.len(), 1);
        *style_first_counts.entry(single[0].style).or_insert(0) += 1;
    }

    // Expected per style is 1200 / 6 = 200.
    // Allow statistical tolerance within [140..260].
    for style in DrivingStyle::ALL {
        let count = style_first_counts.get(&style).copied().unwrap_or(0);
        assert!(
            count >= 140 && count <= 260,
            "Style {:?} appeared {} times out of {} trials (expected ~200)",
            style, count, trials
        );
    }
}
