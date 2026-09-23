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

#[test]
fn test_bell_curve_weights_specification_and_sampling() {
    use tdrace_app::ai::DriverTier;

    assert_eq!(DriverTier::Rookie.bell_curve_weights(), [55, 35, 10, 0, 0]);
    assert_eq!(DriverTier::Amateur.bell_curve_weights(), [20, 50, 25, 5, 0]);
    assert_eq!(DriverTier::Contender.bell_curve_weights(), [5, 20, 50, 20, 5]);
    assert_eq!(DriverTier::Pro.bell_curve_weights(), [0, 5, 25, 50, 20]);
    assert_eq!(DriverTier::Legend.bell_curve_weights(), [0, 0, 10, 35, 55]);

    for tier in [
        DriverTier::Rookie,
        DriverTier::Amateur,
        DriverTier::Contender,
        DriverTier::Pro,
        DriverTier::Legend,
    ] {
        let sum: u8 = tier.bell_curve_weights().iter().sum();
        assert_eq!(sum, 100, "Weights must sum to exactly 100% for {:?}", tier);
    }

    // Statistical check: 2000 draws for Contender (Tier 3)
    let draws = 2000;
    let tiers = DriverTier::Contender.sample_grid_tiers(draws, 42);
    let mut counts = HashMap::new();
    for t in tiers {
        *counts.entry(t).or_insert(0) += 1;
    }

    // Contender expected: T1 5% (100), T2 20% (400), T3 50% (1000), T4 20% (400), T5 5% (100)
    let t1 = counts.get(&DriverTier::Rookie).copied().unwrap_or(0);
    let t2 = counts.get(&DriverTier::Amateur).copied().unwrap_or(0);
    let t3 = counts.get(&DriverTier::Contender).copied().unwrap_or(0);
    let t4 = counts.get(&DriverTier::Pro).copied().unwrap_or(0);
    let t5 = counts.get(&DriverTier::Legend).copied().unwrap_or(0);

    assert!(t1 >= 60 && t1 <= 140, "T1 count: {}", t1);
    assert!(t2 >= 320 && t2 <= 480, "T2 count: {}", t2);
    assert!(t3 >= 850 && t3 <= 1150, "T3 count: {}", t3);
    assert!(t4 >= 320 && t4 <= 480, "T4 count: {}", t4);
    assert!(t5 >= 60 && t5 <= 140, "T5 count: {}", t5);
}

#[test]
fn test_session_casual_race_dynamic_difficulty_and_bell_curve() {
    use tdrace_app::ai::DriverTier;
    use tdrace_app::game::RaceSession;

    let mut session = RaceSession::new();
    assert_eq!(session.casual_ai_difficulty, DriverTier::Contender);

    session.set_casual_ai_difficulty(DriverTier::Legend);
    assert_eq!(session.casual_ai_difficulty, DriverTier::Legend);

    session.num_bots = 7;
    session.init_race();

    assert_eq!(session.opponent_drivers.len(), 7);
    assert_eq!(session.opponent_tiers.len(), 7);
    assert_eq!(session.ai_drivers.len(), 7);
    assert_eq!(session.grid_participants.len(), 8); // 1 player + 7 bots

    // Player has driver_tier = None
    assert_eq!(session.grid_participants.iter().find(|p| p.is_player).unwrap().driver_tier, None);

    // AI Bots have their driver_tier set and their profile matches character.resolve_profile(assigned_tier)
    for (i, opp) in session.opponent_drivers.iter().enumerate() {
        let assigned_tier = session.opponent_tiers[i];
        // For target Legend, only Tiers 3, 4, 5 are possible [0, 0, 10, 35, 55]
        assert!(
            assigned_tier >= DriverTier::Contender,
            "For Legend target, bot tier must be Contender, Pro, or Legend, got {:?}",
            assigned_tier
        );

        let expected_profile = opp.resolve_profile(assigned_tier);
        let ai_driver = &session.ai_drivers[i];
        assert_eq!(ai_driver.profile.speed_factor, expected_profile.speed_factor);
        assert_eq!(ai_driver.profile.steering_kp, expected_profile.steering_kp);
        assert_eq!(ai_driver.profile.brake_margin, expected_profile.brake_margin);

        let part = session.grid_participants.iter().find(|p| p.bot_index == Some(i)).unwrap();
        assert_eq!(part.driver_tier, Some(assigned_tier));
    }

    // Cycling difficulty
    session.cycle_casual_ai_difficulty();
    assert_eq!(session.casual_ai_difficulty, DriverTier::Rookie);
    for tier in &session.opponent_tiers {
        // For Rookie target, only Tiers 1, 2, 3 are possible [55, 35, 10, 0, 0]
        assert!(*tier <= DriverTier::Contender, "For Rookie target, bot tier must be <= Contender, got {:?}", tier);
    }
}
