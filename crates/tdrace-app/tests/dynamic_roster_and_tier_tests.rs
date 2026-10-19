use std::collections::{HashMap, HashSet};
use tdrace_app::ai::{CareerRivalEntry, DriverCharacter, DriverTier, DrivingStyle, RosterEvolutionEngine, SkillProgressionOutcome};
use tdrace_app::db::HallOfFameDb;
use tdrace_app::module::GameModule;
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
    assert_eq!(session.casual_ai_difficulty, DriverTier::Rookie);

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
fn test_career_roster_evolution_90_10_churn_and_replacements() {
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

    // 3. Replacement driver validation:
    // The replacement driver must be a valid character from the global registry,
    // not present in the initial top 3, and have a unique ID in the evolved roster
    let incoming = &report.churned_in[0];
    assert!(incoming.resolve_character().is_some(), "Replacement driver must resolve to global character");
    assert!(!initial.iter().take(3).any(|r| r.driver_id == incoming.driver_id));

    // 4. Uniqueness in evolved roster
    let mut ids = HashSet::new();
    for rival in &evolved {
        assert!(ids.insert(&rival.driver_id), "Duplicate driver ID in evolved roster: {}", rival.driver_id);
    }
}

#[test]
fn test_career_roster_uniform_replacement_style_distribution() {
    let initial = RosterEvolutionEngine::initialize_career_roster(7, DriverTier::Rookie, 42);
    let mut style_counts = HashMap::new();
    let trials = 1200;

    for seed in 0..trials {
        let (_evolved, report) = RosterEvolutionEngine::evolve_roster(&initial, DriverTier::Amateur, seed as u64);
        for incoming in &report.churned_in {
            *style_counts.entry(incoming.style).or_insert(0) += 1;
        }
    }

    // All 6 styles must be represented
    assert_eq!(style_counts.len(), 6, "All 6 driving styles must be sampled for replacements");
    for &style in &DrivingStyle::ALL {
        let count = *style_counts.get(&style).unwrap_or(&0);
        let ratio = count as f64 / trials as f64;
        // Expected ~1/6 = 0.1667, with 1200 trials tolerance +/- 0.05
        assert!(
            (ratio - 1.0 / 6.0).abs() < 0.05,
            "Style {:?} replacement frequency should be ~0.167, got {:.3} (count {})",
            style,
            ratio,
            count
        );
    }
}

#[test]
fn test_career_roster_rank_dependent_skill_advancement() {
    let initial = RosterEvolutionEngine::initialize_career_roster(7, DriverTier::Rookie, 77);
    let top_driver_id = initial[0].driver_id.clone();
    let bottom_driver_id = initial[6].driver_id.clone();

    let mut top_advance_count = 0;
    let mut top_hold_count = 0;
    let mut bottom_advance_count = 0;
    let mut bottom_hold_count = 0;

    let trials = 3000;
    for seed in 0..trials {
        let (_evolved, report) = RosterEvolutionEngine::evolve_roster(&initial, DriverTier::Amateur, seed as u64);

        if let Some(top_report) = report.retained_rivals.iter().find(|r| r.rival.driver_id == top_driver_id) {
            match top_report.outcome {
                SkillProgressionOutcome::StepUp | SkillProgressionOutcome::StandoutLeap => {
                    top_advance_count += 1;
                }
                SkillProgressionOutcome::HoldSteady => {
                    top_hold_count += 1;
                }
            }
        }

        if let Some(bottom_report) = report.retained_rivals.iter().find(|r| r.rival.driver_id == bottom_driver_id) {
            match bottom_report.outcome {
                SkillProgressionOutcome::StepUp | SkillProgressionOutcome::StandoutLeap => {
                    bottom_advance_count += 1;
                }
                SkillProgressionOutcome::HoldSteady => {
                    bottom_hold_count += 1;
                }
            }
        }
    }

    let top_total = (top_advance_count + top_hold_count) as f64;
    let top_adv_rate = top_advance_count as f64 / top_total;
    let top_hold_rate = top_hold_count as f64 / top_total;

    let bottom_total = (bottom_advance_count + bottom_hold_count) as f64;
    let bottom_adv_rate = bottom_advance_count as f64 / bottom_total;
    let bottom_hold_rate = bottom_hold_count as f64 / bottom_total;

    // 1st place (top) is configured for max 80% advance (67.5% StepUp + 12.5% StandoutLeap) and 20% HoldSteady
    assert!(
        (top_adv_rate - 0.80).abs() < 0.04,
        "Top finisher advance rate should be ~0.80 (max 80%), got {:.3}",
        top_adv_rate
    );
    assert!(
        (top_hold_rate - 0.20).abs() < 0.04,
        "Top finisher hold rate should be ~0.20, got {:.3}",
        top_hold_rate
    );

    // Last place (bottom) is configured for 70% advance (62.5% StepUp + 7.5% StandoutLeap) and 30% HoldSteady
    assert!(
        (bottom_adv_rate - 0.70).abs() < 0.04,
        "Bottom finisher advance rate should be ~0.70, got {:.3}",
        bottom_adv_rate
    );
    assert!(
        (bottom_hold_rate - 0.30).abs() < 0.04,
        "Bottom finisher hold rate should be ~0.30, got {:.3}",
        bottom_hold_rate
    );

    // Higher ranking must strictly yield higher advancement probability
    assert!(
        top_adv_rate > bottom_adv_rate,
        "Top finisher advance rate ({:.3}) must exceed bottom finisher advance rate ({:.3})",
        top_adv_rate,
        bottom_adv_rate
    );
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

#[test]
fn test_spec024_scenario_72_characters_alignment_and_no_baked_in_tiers() {
    let all = DriverCharacter::all_across_modules();
    assert_eq!(all.len(), 72, "Total global drivers must be exactly 72");

    // Check unique IDs
    let mut ids = HashSet::new();
    for d in &all {
        assert!(ids.insert(d.id), "Duplicate driver ID: {}", d.id);
    }

    // Check global style count: exactly 12 per style
    let mut global_counts = HashMap::new();
    for d in &all {
        *global_counts.entry(d.style).or_insert(0) += 1;
    }
    for style in DrivingStyle::ALL {
        let count = global_counts.get(&style).copied().unwrap_or(0);
        assert_eq!(count, 12, "Globally, style {:?} must have exactly 12 drivers, got {}", style, count);
    }

    // Check module breakdown: exactly 2 per style in each of the 6 modules
    let modules: Vec<(&str, Vec<DriverCharacter>)> = vec![
        ("Classic", DriverCharacter::ROSTER.to_vec()),
        ("GT", tdrace_app::module::gt::GtWorldChallengeModule::new().drivers()),
        ("NASCAR", tdrace_app::module::nascar::NascarGameModule::new().drivers()),
        ("Rally", tdrace_app::module::rally::RallyGameModule::new().drivers()),
        ("Kart", tdrace_app::module::kart::KartGameModule::new().drivers()),
        ("ExtremeOffRoad", tdrace_app::module::extreme_offroad::ExtremeOffRoadModule::new().drivers()),
    ];

    for (mod_name, drivers) in modules {
        assert_eq!(drivers.len(), 12, "Module {} must have exactly 12 drivers", mod_name);
        let mut mod_counts = HashMap::new();
        for d in &drivers {
            *mod_counts.entry(d.style).or_insert(0) += 1;
            // Verify offsets safety bounds
            assert!(d.offsets.delta_lookahead.abs() <= 0.10, "Driver {} lookahead offset out of range", d.id);
            assert!(d.offsets.delta_steering_kp.abs() <= 1.00, "Driver {} kp offset out of range", d.id);
            assert!(d.offsets.delta_steering_kd.abs() <= 0.10, "Driver {} kd offset out of range", d.id);
            assert!(d.offsets.delta_brake_margin.abs() <= 0.20, "Driver {} brake offset out of range", d.id);
            assert!(d.offsets.delta_aggression.abs() <= 0.20, "Driver {} aggression offset out of range", d.id);
            assert!(d.offsets.delta_avoidance.abs() <= 2.00, "Driver {} avoidance offset out of range", d.id);
            assert!(d.offsets.delta_speed_factor.abs() <= 0.10, "Driver {} speed offset out of range", d.id);

            // Verify dynamic resolution at all 5 tiers
            for tier in [DriverTier::Rookie, DriverTier::Amateur, DriverTier::Contender, DriverTier::Pro, DriverTier::Legend] {
                let profile = d.resolve_profile(tier);
                assert!(profile.speed_factor >= 0.80 && profile.speed_factor <= 1.15);
                assert!(profile.brake_margin >= 0.80 && profile.brake_margin <= 1.45);
                assert!(profile.avoidance_distance >= 3.5 && profile.avoidance_distance <= 12.0);
                assert!(profile.lookahead_time >= 0.20 && profile.lookahead_time <= 0.55);
                assert!(profile.steering_kp >= 1.5 && profile.steering_kp <= 3.5);
                assert!(profile.steering_kd >= 0.03 && profile.steering_kd <= 0.12);

                let stats = d.resolve_stats(tier);
                assert!(stats.speed >= 0.60 && stats.speed <= 0.99);
                assert!(stats.aggression >= 0.40 && stats.aggression <= 0.99);
                assert!(stats.precision >= 0.50 && stats.precision <= 0.99);
                assert!(stats.defense >= 0.50 && stats.defense <= 0.99);
            }
        }
        for style in DrivingStyle::ALL {
            let count = mod_counts.get(&style).copied().unwrap_or(0);
            assert_eq!(count, 2, "Module {} style {:?} must have exactly 2 drivers, got {}", mod_name, style, count);
        }
    }
}

#[test]
fn test_spec024_scenario_uniform_sampling_12_driver_rosters_across_1000_seeds() {
    let all = DriverCharacter::all_across_modules();
    let trials = 1000;
    let mut style_totals = HashMap::new();

    for seed in 0..trials {
        let roster = DriverCharacter::sample_casual_race_roster(&all, 12, seed as u64);
        assert_eq!(roster.len(), 12);

        let mut seed_style_counts = HashMap::new();
        for driver in &roster {
            *seed_style_counts.entry(driver.style).or_insert(0) += 1;
            *style_totals.entry(driver.style).or_insert(0) += 1;
        }

        // For N=12, base quota = 2, remainder = 0.
        // Therefore every single 12-driver grid has EXACTLY 2 of every style!
        for style in DrivingStyle::ALL {
            let count = seed_style_counts.get(&style).copied().unwrap_or(0);
            assert_eq!(count, 2, "In 12-driver grid, style {:?} must have exactly 2 drivers for seed {}", style, seed);
        }
    }

    // Mean across all trials is exactly 2.0
    for style in DrivingStyle::ALL {
        let total = style_totals.get(&style).copied().unwrap_or(0);
        let mean = total as f64 / trials as f64;
        assert_eq!(mean, 2.0, "Mean style count for {:?} must be 2.0", style);
    }

    // Non-multiple of 6 (e.g. N = 8): base = 1, remainder = 2. Expected mean = 8 / 6 = 1.333
    let mut n8_totals = HashMap::new();
    for seed in 0..trials {
        let roster = DriverCharacter::sample_casual_race_roster(&all, 8, seed as u64);
        for d in roster {
            *n8_totals.entry(d.style).or_insert(0) += 1;
        }
    }
    for style in DrivingStyle::ALL {
        let total = n8_totals.get(&style).copied().unwrap_or(0);
        let mean = total as f64 / trials as f64;
        assert!(
            (mean - 8.0 / 6.0).abs() < 0.08,
            "Mean count for style {:?} in N=8 grids should be ~1.333, got {:.3}",
            style, mean
        );
    }
}

#[test]
fn test_spec024_scenario_bell_curve_tier_distribution_contender() {
    let grids = 100;
    let n = 12;
    let total_bots = grids * n; // 1200
    let mut tier_counts = HashMap::new();

    for seed in 0..grids {
        let tiers = DriverTier::Contender.sample_grid_tiers(n, seed as u64);
        for t in tiers {
            *tier_counts.entry(t).or_insert(0) += 1;
        }
    }

    let t1 = tier_counts.get(&DriverTier::Rookie).copied().unwrap_or(0) as f64 / total_bots as f64;
    let t2 = tier_counts.get(&DriverTier::Amateur).copied().unwrap_or(0) as f64 / total_bots as f64;
    let t3 = tier_counts.get(&DriverTier::Contender).copied().unwrap_or(0) as f64 / total_bots as f64;
    let t4 = tier_counts.get(&DriverTier::Pro).copied().unwrap_or(0) as f64 / total_bots as f64;
    let t5 = tier_counts.get(&DriverTier::Legend).copied().unwrap_or(0) as f64 / total_bots as f64;

    // Spec 024 Pseudo-Gherkin:
    // Approximately 50-60% of opponents are Tier 3 (discrete weight: 50%)
    assert!(t3 >= 0.45 && t3 <= 0.55, "Tier 3 proportion should be ~50%, got {:.3}", t3);
    // Approximately 18-22% are Tier 2 and Tier 4 (discrete weights: 20% each)
    assert!(t2 >= 0.16 && t2 <= 0.24, "Tier 2 proportion should be ~20%, got {:.3}", t2);
    assert!(t4 >= 0.16 && t4 <= 0.24, "Tier 4 proportion should be ~20%, got {:.3}", t4);
    // Boundary outliers (discrete weights: 5% each)
    assert!(t1 >= 0.02 && t1 <= 0.08, "Tier 1 proportion should be ~5%, got {:.3}", t1);
    assert!(t5 >= 0.02 && t5 <= 0.08, "Tier 5 proportion should be ~5%, got {:.3}", t5);
}

#[test]
fn test_spec024_scenario_boundary_tier_distributions_rookie_and_legend() {
    let grids = 100;
    let n = 12;
    let total_bots = grids * n;

    // 1. Rookie Difficulty
    let mut rookie_counts = HashMap::new();
    for seed in 0..grids {
        let tiers = DriverTier::Rookie.sample_grid_tiers(n, seed as u64);
        for t in tiers {
            assert!(t <= DriverTier::Contender, "Rookie grid must not exceed Contender tier, got {:?}", t);
            *rookie_counts.entry(t).or_insert(0) += 1;
        }
    }
    let r1 = rookie_counts.get(&DriverTier::Rookie).copied().unwrap_or(0) as f64 / total_bots as f64;
    let r2 = rookie_counts.get(&DriverTier::Amateur).copied().unwrap_or(0) as f64 / total_bots as f64;
    let r3 = rookie_counts.get(&DriverTier::Contender).copied().unwrap_or(0) as f64 / total_bots as f64;
    // Expected weights: [55, 35, 10, 0, 0]
    assert!(r1 >= 0.48 && r1 <= 0.62, "Rookie Tier 1 proportion should be ~55%, got {:.3}", r1);
    assert!(r2 >= 0.28 && r2 <= 0.42, "Rookie Tier 2 proportion should be ~35%, got {:.3}", r2);
    assert!(r3 >= 0.06 && r3 <= 0.15, "Rookie Tier 3 proportion should be ~10%, got {:.3}", r3);
    assert_eq!(rookie_counts.get(&DriverTier::Pro).copied().unwrap_or(0), 0);
    assert_eq!(rookie_counts.get(&DriverTier::Legend).copied().unwrap_or(0), 0);

    // 2. Legend Difficulty
    let mut legend_counts = HashMap::new();
    for seed in 0..grids {
        let tiers = DriverTier::Legend.sample_grid_tiers(n, seed as u64);
        for t in tiers {
            assert!(t >= DriverTier::Contender, "Legend grid must not drop below Contender tier, got {:?}", t);
            *legend_counts.entry(t).or_insert(0) += 1;
        }
    }
    let l5 = legend_counts.get(&DriverTier::Legend).copied().unwrap_or(0) as f64 / total_bots as f64;
    let l4 = legend_counts.get(&DriverTier::Pro).copied().unwrap_or(0) as f64 / total_bots as f64;
    let l3 = legend_counts.get(&DriverTier::Contender).copied().unwrap_or(0) as f64 / total_bots as f64;
    // Expected weights: [0, 0, 10, 35, 55]
    assert!(l5 >= 0.48 && l5 <= 0.62, "Legend Tier 5 proportion should be ~55%, got {:.3}", l5);
    assert!(l4 >= 0.28 && l4 <= 0.42, "Legend Tier 4 proportion should be ~35%, got {:.3}", l4);
    assert!(l3 >= 0.06 && l3 <= 0.15, "Legend Tier 3 proportion should be ~10%, got {:.3}", l3);
    assert_eq!(legend_counts.get(&DriverTier::Rookie).copied().unwrap_or(0), 0);
    assert_eq!(legend_counts.get(&DriverTier::Amateur).copied().unwrap_or(0), 0);
}

#[test]
fn test_spec024_scenario_career_10_car_grid_90_10_churn() {
    let initial = RosterEvolutionEngine::initialize_career_roster(10, DriverTier::Rookie, 777);
    assert_eq!(initial.len(), 10);

    let (evolved, report) = RosterEvolutionEngine::evolve_roster(&initial, DriverTier::Amateur, 888);
    assert_eq!(evolved.len(), 10);

    // With N=10, retained = floor(10 * 0.90) = 9, churn = 1
    assert_eq!(report.retained_rivals.len(), 9);
    assert_eq!(report.churned_out.len(), 1);
    assert_eq!(report.churned_in.len(), 1);

    // Top 3 podium drivers protected
    let top_3_ids: Vec<String> = initial.iter().take(3).map(|r| r.driver_id.clone()).collect();
    let churned_out_id = &report.churned_out[0].driver_id;
    assert!(!top_3_ids.contains(churned_out_id));

    // Replacement driver drafted from global pool with valid style
    let incoming = &report.churned_in[0];
    assert!(incoming.resolve_character().is_some(), "Replacement must be valid global character");
    assert!(!initial.iter().take(3).any(|r| r.driver_id == incoming.driver_id));

    // Probabilistic skill check: majority advance to Tier 2 (Amateur)
    let advanced_to_tier2 = report.retained_rivals.iter().filter(|r| r.new_tier >= DriverTier::Amateur).count();
    assert!(
        advanced_to_tier2 >= 6,
        "Majority of retained rivals (>= 6 of 9) should advance to Tier 2+, got {}",
        advanced_to_tier2
    );
}

#[test]
fn test_career_roster_evolution_end_to_end_multitier_progression() {
    let mut progress = ModuleCareerProgress::default_for_module(42, "gt");
    progress.ensure_career_rivals(10, 101);
    assert_eq!(progress.career_rivals.len(), 10);

    let initial_rival_ids: HashSet<String> = progress.career_rivals.iter().map(|r| r.driver_id.clone()).collect();
    assert_eq!(initial_rival_ids.len(), 10);

    // Simulate advancing through all tiers: 1 -> 2 -> 3 -> 4 -> 5
    for target_tier in 2..=5 {
        progress.trophies_gold += 1;
        progress.xp = ModuleCareerProgress::tier_license_xp(target_tier);
        let (new_lvl, report) = progress.advance_tier_with_seed(1000 + target_tier as u64).expect("advance tier");
        assert_eq!(new_lvl, target_tier);
        assert_eq!(progress.career_rivals.len(), 10);

        // Every tier unlock has 9 retained and 1 churned
        assert_eq!(report.retained_rivals.len(), 9);
        assert_eq!(report.churned_out.len(), 1);
        assert_eq!(report.churned_in.len(), 1);

        // Replacement driver is valid and unique
        assert!(report.churned_in[0].resolve_character().is_some());
        assert!(!report.retained_rivals.iter().any(|r| r.rival.driver_id == report.churned_in[0].driver_id));
    }

    // At tier 5, several original rivals should still be in the roster (continuity)
    let final_rival_ids: HashSet<String> = progress.career_rivals.iter().map(|r| r.driver_id.clone()).collect();
    let retained_originals = initial_rival_ids.intersection(&final_rival_ids).count();
    // Over 4 transitions with 1 churn each, at least 10 - 4 = 6 original rivals MUST survive
    assert!(
        retained_originals >= 6,
        "Expected at least 6 original rivals to persist to Tier 5, found {}",
        retained_originals
    );

    // Verify final tiers: at Tier 5, majority of rivals should be Tier 4 or 5
    let high_tier_count = progress.career_rivals.iter().filter(|r| r.tier >= DriverTier::Pro).count();
    assert!(
        high_tier_count >= 7,
        "By Tier 5, at least 7 of 10 rivals should be Pro or Legend, got {}",
        high_tier_count
    );
}

#[test]
fn test_spec126_dynamic_championship_roster_synthesis_distributions() {
    use tdrace_app::series::SeriesSession;

    let session = SeriesSession::generate_dynamic_roster(
        "World Rallycross Supercars",
        "rally",
        3,
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["holjes".to_string(), "loheac".to_string()],
        5,
        32,
        "Apex RX",
        Some("rally_nordic_rx_t3"),
        2026_126,
    );

    assert_eq!(session.standings.len(), 32);
    assert_eq!(session.standings[0].driver_id, "player");
    assert_eq!(session.standings[0].car_model_id.as_deref(), Some("rally_nordic_rx_t3"));

    // All 31 AI drivers must be unique characters from the global pool with valid cars, teams, styles, and tiers
    let mut seen_ids = HashSet::new();
    let mut style_counts: HashMap<String, usize> = HashMap::new();
    for entry in session.standings.iter().skip(1) {
        assert!(seen_ids.insert(entry.driver_id.clone()), "Duplicate AI driver ID: {}", entry.driver_id);
        assert!(DriverCharacter::find_global(&entry.driver_id).is_some(), "AI driver must exist in global pool");
        let style = entry.ai_style.as_deref().expect("AI style must be populated");
        *style_counts.entry(style.to_string()).or_insert(0) += 1;

        let ai_tier = entry.ai_tier.expect("AI tier must be populated");
        assert!((1..=5).contains(&ai_tier), "AI tier must be 1..=5, got {}", ai_tier);

        let car_id = entry.car_model_id.as_deref().expect("AI car_model_id must be populated");
        let car_model = tdrace_app::catalog::find_model_by_id(car_id).expect("car_model_id must resolve in catalog");
        assert!(matches!(car_model.module_id, "rally" | "rallycross"));
        assert_eq!(car_model.tier, 3);
        assert!(!tdrace_app::series::is_generic_team_name(&entry.team_name, &entry.driver_name));
    }

    // Uniform 1/6 style quota across 31 AI drivers: base = 5, remainder = 1 (5 styles have 5, 1 style has 6)
    assert_eq!(style_counts.len(), 6, "All 6 driving styles must be represented");
    for count in style_counts.values() {
        assert!(*count == 5 || *count == 6, "Each style must have 5 or 6 drivers in a 31-bot grid, got {}", count);
    }
}

#[test]
fn test_spec126_per_race_traversal_probability_distributions_and_damping() {
    use tdrace_app::series::{SeriesSession, SubstitutionChannel};

    let base_session = SeriesSession::generate_dynamic_roster(
        "GT3 European Challenge",
        "gt",
        3,
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["monza".to_string(), "spa".to_string(), "brands_hatch".to_string()],
        3,
        16, // 1 player + 15 AI drivers (ranks 0..14)
        "Apex GT",
        None,
        42,
    );

    let ai_ids_by_rank: Vec<String> = base_session
        .standings
        .iter()
        .filter(|s| s.driver_id != "player")
        .map(|s| s.driver_id.clone())
        .collect();
    assert_eq!(ai_ids_by_rank.len(), 15);

    let trials = 4000u64;
    let mut zero_replacements = 0usize;
    let mut single_replacements = 0usize;
    let mut multi_replacements = 0usize;

    let mut med_top_half = 0usize;
    let mut med_bottom_half = 0usize;
    let mut admin_top_3 = 0usize;
    let mut admin_bottom_3 = 0usize;
    let mut rank_first_total = 0usize;
    let mut rank_last_total = 0usize;

    for t in 0..trials {
        let mut session = base_session.clone();
        session.current_round = 1;
        let events = session.evaluate_pre_race_substitutions(100_000 + t);

        match events.len() {
            0 => zero_replacements += 1,
            1 => single_replacements += 1,
            _ => multi_replacements += 1,
        }

        for ev in &events {
            let rank = ai_ids_by_rank
                .iter()
                .position(|id| id == &ev.outgoing_driver_id)
                .expect("outgoing driver was in initial AI roster");

            if rank == 0 {
                rank_first_total += 1;
            }
            if rank == 14 {
                rank_last_total += 1;
            }

            match ev.channel {
                SubstitutionChannel::MedicalPersonal => {
                    if rank < 7 {
                        med_top_half += 1;
                    } else if rank > 7 {
                        med_bottom_half += 1;
                    }
                }
                SubstitutionChannel::AdministrativeContract => {
                    if rank < 3 {
                        admin_top_3 += 1;
                    } else if rank >= 12 {
                        admin_bottom_3 += 1;
                    }
                }
            }
        }
    }

    let single_rate = single_replacements as f64 / trials as f64;
    let multi_rate = multi_replacements as f64 / trials as f64;

    // 1. Moderate 1-driver replacement probability (~14% - 19%)
    assert!(
        single_rate >= 0.13 && single_rate <= 0.20,
        "Single driver replacement rate should be moderate (~16%), got {:.4} ({}/{})",
        single_rate,
        single_replacements,
        trials
    );

    // 2. Very low multi-driver replacement probability (< 0.8%) due to gamma = 0.12 damping
    assert!(
        multi_rate < 0.008,
        "Multi-driver replacement rate must be very low (< 0.8%), got {:.4} ({}/{})",
        multi_rate,
        multi_replacements,
        trials
    );
    assert_eq!(zero_replacements + single_replacements + multi_replacements, trials as usize);

    // 3. Medical/Personal reasons are rank-independent (top 7 vs bottom 7 have roughly equal counts)
    let med_total = (med_top_half + med_bottom_half).max(1) as f64;
    let med_top_share = med_top_half as f64 / med_total;
    assert!(
        (med_top_share - 0.50).abs() < 0.12,
        "Medical channel must be rank-independent (~50% top half vs bottom half), got top={} bottom={} (share={:.3})",
        med_top_half,
        med_bottom_half,
        med_top_share
    );

    // 4. Administrative/Contract reasons are strongly rank-dependent (bottom 3 >> top 3)
    assert!(
        admin_bottom_3 > admin_top_3 * 4,
        "Admin/contract replacements must be much rarer for top 3 ({}) than bottom 3 ({})",
        admin_top_3,
        admin_bottom_3
    );

    // 5. Overall replacement probability for the top-ranked AI driver is significantly smaller than for the last-ranked AI driver
    assert!(
        rank_last_total > rank_first_total * 2,
        "Overall replacements for #1 ranked AI ({}) must be significantly smaller than for last-ranked AI ({})",
        rank_first_total,
        rank_last_total
    );
}

#[test]
fn test_spec126_temporal_one_race_stand_in_and_next_race_restoration() {
    use tdrace_app::series::{ReplacementDuration, SeriesSession};

    let mut session = SeriesSession::generate_dynamic_roster(
        "GT3 Cup",
        "gt",
        2,
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["monza".to_string(), "spa".to_string(), "silverstone".to_string()],
        3,
        8,
        "Apex GT",
        None,
        777,
    );

    // Award points to all initial drivers in Round 1
    for (idx, entry) in session.standings.iter_mut().enumerate() {
        entry.points = (100 - idx * 10) as u32;
        entry.wins = u32::from(idx == 1);
        entry.podiums = u32::from(idx <= 3);
    }
    session.current_round = 1;

    // Find a seed that triggers a OneRace temporal substitution
    let mut chosen_event = None;
    for seed in 1..5000u64 {
        let mut probe = session.clone();
        let events = probe.evaluate_pre_race_substitutions(seed);
        if let Some(ev) = events.iter().find(|e| e.duration == ReplacementDuration::OneRace) {
            chosen_event = Some(ev.clone());
            session = probe;
            break;
        }
    }

    let ev = chosen_event.expect("Must find a seed producing a OneRace temporal substitution");
    assert_eq!(ev.duration, ReplacementDuration::OneRace);
    assert!(ev.outgoing_points > 0, "Outgoing regular driver had points before temporary absence");

    // Verify incoming stand-in is in standings with 0 points, same team and car model
    let stand_in_entry = session
        .standings
        .iter()
        .find(|s| s.driver_id == ev.incoming_driver_id)
        .expect("Stand-in driver must be in active standings for this race");
    assert_eq!(stand_in_entry.points, 0, "Stand-in driver must start with 0 points");
    assert_eq!(stand_in_entry.wins, 0);
    assert_eq!(stand_in_entry.podiums, 0);
    assert_eq!(stand_in_entry.team_name, ev.team_name);
    assert_eq!(stand_in_entry.car_model_id, ev.car_model_id);
    assert!(
        !session.standings.iter().any(|s| s.driver_id == ev.outgoing_driver_id),
        "Temporarily sidelined regular driver is not on the active grid for this single race"
    );
    assert_eq!(session.active_stand_ins.len(), 1);

    // Advance to the next race and restore expired temporal replacements
    session.current_round = 2;
    let restored = session.restore_expired_temporal_replacements();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].driver_id, ev.outgoing_driver_id);
    assert_eq!(restored[0].points, ev.outgoing_points);

    let returned_regular = session
        .standings
        .iter()
        .find(|s| s.driver_id == ev.outgoing_driver_id)
        .expect("Regular driver must return to their seat for the next race");
    assert_eq!(
        returned_regular.points, ev.outgoing_points,
        "Returning regular driver must retain their prior championship points"
    );
    assert_eq!(returned_regular.team_name, ev.team_name);
    assert_eq!(returned_regular.car_model_id, ev.car_model_id);
    assert!(session.active_stand_ins.is_empty());
}

#[test]
fn test_spec126_permanent_replacement_zero_points_and_pool_reentry() {
    use tdrace_app::series::{ReplacementDuration, SeriesSession};

    let mut session = SeriesSession::generate_dynamic_roster(
        "Hypercar World GP",
        "gt",
        5,
        PointSystem::FiaStandard { fastest_lap_bonus: true },
        vec!["monza".to_string(), "spa".to_string(), "le_mans".to_string(), "suzuka".to_string()],
        3,
        16,
        "Apex Hypercar",
        None,
        999,
    );

    for (idx, entry) in session.standings.iter_mut().enumerate() {
        entry.points = (200 - idx * 10) as u32;
    }
    session.current_round = 1;

    // Find a seed that triggers a Permanent substitution
    let mut perm_event = None;
    for seed in 1..5000u64 {
        let mut probe = session.clone();
        let events = probe.evaluate_pre_race_substitutions(seed);
        if let Some(ev) = events.iter().find(|e| e.duration == ReplacementDuration::Permanent && events.len() == 1) {
            perm_event = Some(ev.clone());
            session = probe;
            break;
        }
    }

    let ev = perm_event.expect("Must find a seed producing a Permanent substitution");
    let removed_driver_id = ev.outgoing_driver_id.clone();

    // Incoming permanent replacement starts at 0 points
    let incoming = session
        .standings
        .iter()
        .find(|s| s.driver_id == ev.incoming_driver_id)
        .expect("Incoming permanent replacement must be in standings");
    assert_eq!(incoming.points, 0);
    assert_eq!(incoming.team_name, ev.team_name);
    assert_eq!(incoming.car_model_id, ev.car_model_id);
    assert!(session.active_stand_ins.is_empty(), "Permanent replacement must not queue a temporal stand-in");
    assert!(!session.standings.iter().any(|s| s.driver_id == removed_driver_id));

    // Verify the removed driver is back in the available pool and can re-enter in a later round
    session.current_round = 2;
    let mut reentered_team = None;
    for seed in 1..20_000u64 {
        let mut probe = session.clone();
        let events = probe.evaluate_pre_race_substitutions(seed);
        if let Some(re_ev) = events.iter().find(|e| e.incoming_driver_id == removed_driver_id) {
            let re_entry = probe
                .standings
                .iter()
                .find(|s| s.driver_id == removed_driver_id)
                .unwrap();
            assert_eq!(re_entry.points, 0, "Re-entering driver must start fresh with 0 points");
            reentered_team = Some(re_ev.team_name.clone());
            break;
        }
    }
    assert!(
        reentered_team.is_some(),
        "Permanently removed driver '{}' must return to the pool and be eligible to re-enter later",
        removed_driver_id
    );
}

#[test]
fn test_spec126_no_mid_tournament_substitutions_and_tier_advance_race1_eval() {
    use tdrace_app::game::RaceSession;
    use tdrace_app::series::{ChampionshipManager, TournamentConfig};

    // 1. Mid-tournament guard: while a tournament sprint weekend is in progress (heats -> semis -> finals),
    // evaluate_pre_race_substitutions must NEVER replace any driver.
    let mgr = ChampionshipManager::new();
    let def = mgr.get("autocross_superbuggy_world_series").unwrap();
    let mut champ = def.to_session();
    champ.ensure_tournament_weekend().unwrap();
    assert!(champ.tournament_in_progress());

    let roster_before: Vec<String> = champ.standings.iter().map(|s| s.driver_id.clone()).collect();
    for seed in 0..200u64 {
        let events = champ.evaluate_pre_race_substitutions(seed);
        assert!(events.is_empty(), "Must never trigger driver replacements mid-tournament");
    }
    let roster_after: Vec<String> = champ.standings.iter().map(|s| s.driver_id.clone()).collect();
    assert_eq!(roster_before, roster_after);

    // 2. Starting a new championship after advancing tier (level > 1 or tier > 1) applies pre-race substitution evaluation before Race 1
    let mut game = RaceSession::new();
    game.active_career_progress.level = 2;
    game.fixed_roster_seed = Some(1);
    let tier2_def = mgr.get("gt3_european_challenge").unwrap();
    game.launch_or_resume_championship(tier2_def);
    let active_champ = game.championship_session.as_ref().unwrap();
    assert_eq!(active_champ.current_round, 0);
    assert_eq!(
        active_champ.last_substitution_round,
        Some(0),
        "Pre-race substitution check must be evaluated before Race 1 of a new championship after advancing tier"
    );
    let _ = TournamentConfig::default();
}

#[test]
fn test_spec126_substitution_reason_templates_and_modal_dismissal() {
    use tdrace_app::game::RaceSession;
    use tdrace_app::series::{
        DriverSubstitutionEvent, ReplacementDuration, SubstitutionChannel, SUBSTITUTION_REASON_TEMPLATES,
    };

    // Minimum 10 templates required by user request; we provide 13 across both channels and durations
    assert!(
        SUBSTITUTION_REASON_TEMPLATES.len() >= 10,
        "Pool of explanation templates must contain at least 10 templates, found {}",
        SUBSTITUTION_REASON_TEMPLATES.len()
    );

    let med_count = SUBSTITUTION_REASON_TEMPLATES
        .iter()
        .filter(|t| t.channel == SubstitutionChannel::MedicalPersonal)
        .count();
    let admin_count = SUBSTITUTION_REASON_TEMPLATES
        .iter()
        .filter(|t| t.channel == SubstitutionChannel::AdministrativeContract)
        .count();
    let one_race_count = SUBSTITUTION_REASON_TEMPLATES
        .iter()
        .filter(|t| t.duration == ReplacementDuration::OneRace)
        .count();
    let perm_count = SUBSTITUTION_REASON_TEMPLATES
        .iter()
        .filter(|t| t.duration == ReplacementDuration::Permanent)
        .count();

    assert!(med_count >= 5);
    assert!(admin_count >= 5);
    assert!(one_race_count >= 4);
    assert!(perm_count >= 4);

    let mut seen_ids = HashSet::new();
    for tpl in SUBSTITUTION_REASON_TEMPLATES {
        assert!(seen_ids.insert(tpl.id), "Duplicate template ID: {}", tpl.id);
        let text = tpl.format_reason("Carlos Sainz", "Liam Lawson", "Apex Factory GT", "Porsche 911 GT3 R");
        assert!(text.contains("Carlos Sainz"));
        assert!(text.contains("Liam Lawson"));
        assert!(text.contains("Apex Factory GT"));
        assert!(text.contains("Porsche 911 GT3 R"));
        assert!(!text.contains("{outgoing}") && !text.contains("{incoming}") && !text.contains("{team}") && !text.contains("{car}"));
    }

    // Verify modal state and gamepad/keyboard dismissal on RaceSession
    let mut game = RaceSession::new();
    game.start_rally_career_tier(1);
    assert!(!game.has_pending_substitution_modal());

    game.championship_session.as_mut().unwrap().pending_substitutions.push(DriverSubstitutionEvent {
        round_index: 1,
        outgoing_driver_id: "jonas_lindqvist".to_string(),
        outgoing_driver_name: "Jonas Lindqvist".to_string(),
        outgoing_points: 18,
        incoming_driver_id: "klara_andersson".to_string(),
        incoming_driver_name: "Klara Andersson".to_string(),
        team_name: "Nordic World RX Team".to_string(),
        car_model_id: Some("rally_gallia_200_t1".to_string()),
        car_name: "Gallia 200 Rally4".to_string(),
        duration: ReplacementDuration::OneRace,
        channel: SubstitutionChannel::MedicalPersonal,
        reason_headline: "ILLNESS STAND-IN — HIGH FEVER & FLU".to_string(),
        reason_text: "Jonas Lindqvist woke up with a high fever.".to_string(),
    });
    assert!(game.has_pending_substitution_modal());

    game.input.gamepad.snapshot.btn_confirm_pressed = true;
    assert!(game.handle_substitution_modal_input());
    game.input.gamepad.snapshot.btn_confirm_pressed = false;
    assert!(!game.has_pending_substitution_modal());
}

#[test]
fn test_spec131_casual_mode_spawns_t4_plus_and_t5_plus_as_minority_proportion() {
    let draws = 3000;

    // 1. Rookie & Amateur casual grids spawn 0% T4+ / T5+
    for low_tier in [DriverTier::Rookie, DriverTier::Amateur] {
        let sampled = low_tier.sample_casual_grid_tiers(500, 126);
        assert!(
            sampled.iter().all(|t| !t.is_error_free()),
            "Low casual tier {:?} should not spawn error-free variants",
            low_tier
        );
    }

    // 2. Pro (T4) casual grid: ~15% T4+ and ~5% T5+ (20% minority), majority standard T4 (~45%) & T5 (~10%)
    let pro_grid = DriverTier::Pro.sample_casual_grid_tiers(draws, 2026);
    let pro_t4 = pro_grid.iter().filter(|&&t| t == DriverTier::Pro).count() as f64 / draws as f64;
    let pro_t4_plus = pro_grid.iter().filter(|&&t| t == DriverTier::ProPlus).count() as f64 / draws as f64;
    let pro_t5_plus = pro_grid.iter().filter(|&&t| t == DriverTier::LegendPlus).count() as f64 / draws as f64;
    assert!(pro_t4_plus >= 0.11 && pro_t4_plus <= 0.19, "Pro casual T4+ should be ~15%, got {:.3}", pro_t4_plus);
    assert!(pro_t5_plus >= 0.02 && pro_t5_plus <= 0.08, "Pro casual T5+ should be ~5%, got {:.3}", pro_t5_plus);
    assert!(pro_t4 > pro_t4_plus + pro_t5_plus, "Standard T4 ({:.3}) must remain majority over T4+/T5+ ({:.3})", pro_t4, pro_t4_plus + pro_t5_plus);

    // 3. Legend (T5) casual grid: ~20% T4+ and ~20% T5+ (~40% total error-free minority vs ~55% standard T4+T5)
    let legend_grid = DriverTier::Legend.sample_casual_grid_tiers(draws, 2027);
    let leg_t4 = legend_grid.iter().filter(|&&t| t == DriverTier::Pro).count() as f64 / draws as f64;
    let leg_t4_plus = legend_grid.iter().filter(|&&t| t == DriverTier::ProPlus).count() as f64 / draws as f64;
    let leg_t5 = legend_grid.iter().filter(|&&t| t == DriverTier::Legend).count() as f64 / draws as f64;
    let leg_t5_plus = legend_grid.iter().filter(|&&t| t == DriverTier::LegendPlus).count() as f64 / draws as f64;
    assert!(leg_t4_plus >= 0.15 && leg_t4_plus <= 0.25, "Legend casual T4+ should be ~20%, got {:.3}", leg_t4_plus);
    assert!(leg_t5_plus >= 0.15 && leg_t5_plus <= 0.25, "Legend casual T5+ should be ~20%, got {:.3}", leg_t5_plus);
    assert!(leg_t4 + leg_t5 > leg_t4_plus + leg_t5_plus, "Standard T4+T5 must outnumber T4+/T5+ in casual mode");
}

#[test]
fn test_spec131_championship_and_career_scale_t4_plus_and_t5_plus_with_player_progress() {
    let draws = 2000;

    // 1. Championship grid sampling scales monotonically with progression_score
    let leg_fresh = DriverTier::sample_championship_grid_tiers(5, 0.0, draws, 9001);
    let leg_mid = DriverTier::sample_championship_grid_tiers(5, 0.5, draws, 9001);
    let leg_vet = DriverTier::sample_championship_grid_tiers(5, 1.0, draws, 9001);

    let count_plus = |grid: &[DriverTier]| grid.iter().filter(|t| t.is_error_free()).count();
    let c_fresh = count_plus(&leg_fresh);
    let c_mid = count_plus(&leg_mid);
    let c_vet = count_plus(&leg_vet);
    assert!(
        c_fresh < c_mid && c_mid < c_vet,
        "Championship T4+/T5+ count must grow with experience: fresh={} < mid={} < vet={}",
        c_fresh, c_mid, c_vet
    );

    // 2. Career progress score & veteran rival promotion across seasons
    let mut progress = ModuleCareerProgress::default_for_module(77, "gt");
    assert_eq!(progress.elite_variant_progression_score(), 0.0);

    progress.level = 5;
    progress.ensure_career_rivals(12, 500);
    let initial_plus = progress.career_rivals.iter().filter(|r| r.tier.is_error_free()).count();

    // Simulate completing several Tier 5 championships
    progress.xp = ModuleCareerProgress::tier_license_xp(5);
    for season in 0..4 {
        progress.record_championship_finish(
            &format!("gt_t5_s{season}"),
            5,
            1,
            75,
            "2026-10-19T00:00:00Z",
        );
    }

    assert!(progress.elite_variant_progression_score() > 0.70);
    let veteran_plus = progress.career_rivals.iter().filter(|r| r.tier.is_error_free()).count();
    assert!(
        veteran_plus > initial_plus,
        "Veteran career rivals should promote more T4+/T5+ drivers after repeated championship finishes: initial={} -> veteran={}",
        initial_plus, veteran_plus
    );
}


