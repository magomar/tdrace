use std::collections::{HashMap, HashSet};
use tdrace_app::ai::{CareerRivalEntry, DriverCharacter, DriverTier, DrivingStyle, RosterEvolutionEngine, SkillProgressionOutcome};
use tdrace_app::db::HallOfFameDb;
use tdrace_app::profile::{ModuleCareerProgress, PlayerProfile};
use tdrace_app::series::{ChampionshipSession, PointSystem};

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

#[test]
fn test_career_roster_initialization_and_tier1_centering() {
    let roster = RosterEvolutionEngine::initialize_career_roster(7, DriverTier::Rookie, 12345);
    assert_eq!(roster.len(), 7);

    let mut ids = HashSet::new();
    let mut style_counts = HashMap::new();
    for rival in &roster {
        assert!(ids.insert(rival.driver_id.clone()), "Duplicate rival ID '{}'", rival.driver_id);
        *style_counts.entry(rival.style).or_insert(0) += 1;
        // Rookie centering: only Tiers 1..=3 are possible
        assert!(
            rival.tier <= DriverTier::Contender,
            "Rookie initialized rival must have tier <= Contender, got {:?}",
            rival.tier
        );
    }

    // With N=7, 5 styles have 1 driver, 1 style has 2 drivers
    let min_style = *style_counts.values().min().unwrap();
    let max_style = *style_counts.values().max().unwrap();
    assert_eq!(min_style, 1);
    assert_eq!(max_style, 2);
    assert_eq!(style_counts.len(), 6, "All 6 styles must be represented in career roster of 7");
}

#[test]
fn test_career_roster_evolution_90_10_churn_and_style_matching() {
    let initial = RosterEvolutionEngine::initialize_career_roster(7, DriverTier::Rookie, 42);
    assert_eq!(initial.len(), 7);

    let (evolved, report) = RosterEvolutionEngine::evolve_roster(&initial, DriverTier::Amateur, 100);
    assert_eq!(evolved.len(), 7);

    // 1. Retention & Churn count checks:
    // With N=7, retained_target = floor(7 * 0.90) = 6, churn = 1
    assert_eq!(report.retained_rivals.len(), 6);
    assert_eq!(report.churned_out.len(), 1);
    assert_eq!(report.churned_in.len(), 1);

    // 2. Podium protection check:
    // The top 3 drivers (indices 0, 1, 2) in initial must NOT be churned out
    let top_3_ids: Vec<String> = initial.iter().take(3).map(|r| r.driver_id.clone()).collect();
    let churned_id = &report.churned_out[0].driver_id;
    assert!(!top_3_ids.contains(churned_id), "Podium finisher '{}' must be protected from churn", churned_id);

    // 3. Style preservation check:
    // The replacement driver must have the exact same style as the churned out driver
    let departed_style = report.churned_out[0].style;
    let incoming_style = report.churned_in[0].style;
    assert_eq!(incoming_style, departed_style, "Incoming rival style must match departed rival style");

    // 4. Overall roster style distribution must be preserved
    let mut initial_styles: Vec<DrivingStyle> = initial.iter().map(|r| r.style).collect();
    let mut evolved_styles: Vec<DrivingStyle> = evolved.iter().map(|r| r.style).collect();
    initial_styles.sort();
    evolved_styles.sort();
    assert_eq!(initial_styles, evolved_styles, "Overall style distribution must be perfectly preserved");

    // 5. Uniqueness in evolved roster
    let mut ids = HashSet::new();
    for rival in &evolved {
        assert!(ids.insert(&rival.driver_id), "Duplicate driver ID in evolved roster: {}", rival.driver_id);
    }
}

#[test]
fn test_career_roster_probabilistic_skill_progression_distribution() {
    let mut step_up_count = 0;
    let mut hold_steady_count = 0;
    let mut standout_leap_count = 0;

    let base_roster = RosterEvolutionEngine::initialize_career_roster(7, DriverTier::Rookie, 999);

    let trials = 1000;
    for seed in 0..trials {
        let (_evolved, report) = RosterEvolutionEngine::evolve_roster(&base_roster, DriverTier::Amateur, seed as u64);
        for ret in &report.retained_rivals {
            match ret.outcome {
                SkillProgressionOutcome::StepUp => {
                    step_up_count += 1;
                    assert_eq!(ret.new_tier, DriverTier::Amateur, "StepUp must match target new_tier");
                }
                SkillProgressionOutcome::HoldSteady => {
                    hold_steady_count += 1;
                    assert_eq!(ret.new_tier, ret.previous_tier, "HoldSteady must retain previous_tier");
                }
                SkillProgressionOutcome::StandoutLeap => {
                    standout_leap_count += 1;
                    assert_eq!(ret.new_tier, DriverTier::Contender, "StandoutLeap from Tier 2 target must leap to Tier 3");
                }
            }
        }
    }

    let total = (step_up_count + hold_steady_count + standout_leap_count) as f64;
    let step_up_ratio = step_up_count as f64 / total;
    let hold_steady_ratio = hold_steady_count as f64 / total;
    let standout_leap_ratio = standout_leap_count as f64 / total;

    // Expected: 65% StepUp, 25% HoldSteady, 10% StandoutLeap
    assert!(
        (step_up_ratio - 0.65).abs() < 0.05,
        "Step up ratio should be ~0.65, got {:.3}",
        step_up_ratio
    );
    assert!(
        (hold_steady_ratio - 0.25).abs() < 0.05,
        "Hold steady ratio should be ~0.25, got {:.3}",
        hold_steady_ratio
    );
    assert!(
        (standout_leap_ratio - 0.10).abs() < 0.04,
        "Standout leap ratio should be ~0.10, got {:.3}",
        standout_leap_ratio
    );
}

#[test]
fn test_module_career_progress_advance_tier_evolution() {
    let mut progress = ModuleCareerProgress::default_for_module(1, "gt");
    assert_eq!(progress.level, 1);
    assert!(progress.career_rivals.is_empty());

    // Ensure rivals initialized
    progress.ensure_career_rivals(7, 42);
    assert_eq!(progress.career_rivals.len(), 7);
    let initial_rival_ids: Vec<String> = progress.career_rivals.iter().map(|r| r.driver_id.clone()).collect();

    // Setup requirements for advancing tier: 1 podium and sufficient XP
    progress.trophies_gold = 1;
    progress.add_xp(5000);
    assert!(progress.can_advance_tier());

    let (new_lvl, report) = progress.advance_tier_with_seed(12345).expect("advance_tier should succeed");
    assert_eq!(new_lvl, 2);
    assert_eq!(progress.level, 2);
    assert_eq!(progress.career_rivals.len(), 7);

    // Verify 90/10 churn
    assert_eq!(report.retained_rivals.len(), 6);
    assert_eq!(report.churned_out.len(), 1);
    assert_eq!(report.churned_in.len(), 1);

    // Verify departing rival was in initial, incoming rival was not
    assert!(initial_rival_ids.contains(&report.churned_out[0].driver_id));
    assert!(!initial_rival_ids.contains(&report.churned_in[0].driver_id));

    // Verify unlocked tracks and cars updated for Tier 2
    assert!(progress.is_track_unlocked("monza", false));
    assert!(progress.is_track_unlocked("silverstone", false));
    assert!(progress.is_track_unlocked("catalunya", false));
}

#[test]
fn test_sqlite_persistence_of_career_rivals() {
    let db = HallOfFameDb::open_in_memory().expect("Open in-memory db");

    let mut profile = PlayerProfile::default();
    profile.name = "Career Driver".to_string();
    let profile_id = db.create_profile(&profile).expect("Create profile");

    let mut progress = ModuleCareerProgress::default_for_module(profile_id, "gt");
    progress.ensure_career_rivals(7, 777);
    assert_eq!(progress.career_rivals.len(), 7);

    db.save_module_progress(&progress).expect("Save module progress");

    let loaded = db.get_module_progress(profile_id, "gt").expect("Get module progress").expect("Must exist");
    assert_eq!(loaded.career_rivals.len(), 7);
    assert_eq!(loaded.career_rivals, progress.career_rivals);
}

#[test]
fn test_championship_session_trigger_roster_evolution() {
    let rivals: Vec<CareerRivalEntry> = RosterEvolutionEngine::initialize_career_roster(7, DriverTier::Rookie, 555);
    let mut champ = ChampionshipSession::from_career_rivals(
        "GT Championship",
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["monza".to_string(), "spa".to_string()],
        3,
        "Player Team",
        &rivals,
    );

    assert_eq!(champ.standings.len(), 8); // 1 player + 7 rivals
    assert_eq!(champ.tier, 1);

    let report = champ.trigger_roster_evolution(2, 9999);
    assert_eq!(champ.tier, 2);
    assert_eq!(report.retained_rivals.len(), 6);
    assert_eq!(report.churned_out.len(), 1);
    assert_eq!(report.churned_in.len(), 1);

    // Standings should retain player and update the 7 rivals with new tiers
    assert_eq!(champ.standings.len(), 8);
    let player = champ.standings.iter().find(|s| s.driver_id == "player").unwrap();
    assert_eq!(player.driver_name, "Player");

    for s in champ.standings.iter().filter(|s| s.driver_id != "player") {
        assert!(s.ai_tier.is_some(), "Rival in standings must have ai_tier set");
    }
}
