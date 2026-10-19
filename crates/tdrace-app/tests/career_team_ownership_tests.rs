use std::collections::HashSet;
use race_kit::ai::{DriverTier, DrivingStyle};
use tdrace_app::{
    BotAiDriver, BotProfile, DriverHireQuote, DriverStats,
    ModuleCareerProgress, PlayerProfile, PointSystem, RoundDriverResult, SeriesSession,
    XpAwardReceipt,
};

#[test]
fn test_scenario_3_scouting_market_pool_diversity_and_tier_proportional_salaries() {
    // Given a Career Championship at Tier 1 (Rookie)
    // When the Driver Scouting Market generates a candidate pool via generate_scout_pool(1, seed)
    let pool = SeriesSession::generate_scout_pool(1, 1337);

    // Then the pool contains 6 candidate drivers spanning all 6 DrivingStyle archetypes
    assert_eq!(pool.len(), 6);
    let styles: HashSet<DrivingStyle> = pool.iter().map(|q| q.style).collect();
    assert_eq!(
        styles.len(),
        6,
        "Scout pool must cover all 6 DrivingStyle archetypes"
    );

    // And the candidates include a mix of Tier 1 (Rookie) and Tier 2 (Amateur) drivers sampled from the bell-curve distribution
    let has_t1 = pool.iter().any(|q| q.tier == DriverTier::Rookie);
    let has_t2 = pool.iter().any(|q| q.tier == DriverTier::Amateur);
    assert!(has_t1, "Tier 1 scout pool must include Tier 1 (Rookie) candidates");
    assert!(has_t2, "Tier 1 scout pool must include Tier 2 (Amateur) candidates");

    // And every Tier 2 (Amateur) candidate has a strictly higher per_race_salary than every Tier 1 (Rookie) candidate
    let max_t1_salary = pool
        .iter()
        .filter(|q| q.tier == DriverTier::Rookie)
        .map(|q| q.per_race_salary)
        .max()
        .unwrap();
    let min_t2_salary = pool
        .iter()
        .filter(|q| q.tier == DriverTier::Amateur)
        .map(|q| q.per_race_salary)
        .min()
        .unwrap();
    assert!(
        min_t2_salary > max_t1_salary,
        "Every Tier 2 candidate salary ({}) must be strictly higher than every Tier 1 candidate salary ({})",
        min_t2_salary,
        max_t1_salary
    );
}

#[test]
fn test_scenario_4_founding_a_player_team_by_purchasing_a_second_car_and_hiring_a_bot_teammate() {
    // Given a player with 20,000 Cr starting a Tier 1 GT Championship
    let mut profile = PlayerProfile::default();
    profile.credits = 20_000;
    let mut career = ModuleCareerProgress::default_for_gt(1);

    let mut champ = SeriesSession::new(
        "GT4 Clubsport Cup",
        PointSystem::FiaStandard {
            fastest_lap_bonus: false,
        },
        vec!["monza".to_string(), "spa".to_string()],
        3,
        &[
            ("player", "Player", "Solo Driver Entry"),
            ("magnus_wolff", "Magnus Wolff", "Bullseye GT"),
            ("cedric_laval", "Cédric Laval", "Valente GT"),
            ("lawrence_sterling", "Lawrence Sterling", "Valente GT"),
        ],
    );

    let contract = DriverHireQuote {
        character_id: "marco_rossi".to_string(),
        character_name: "Marco Rossi".to_string(),
        style: DrivingStyle::Aggressive,
        tier: DriverTier::Rookie,
        stats: DriverStats {
            speed: 0.78,
            aggression: 0.88,
            precision: 0.75,
            defense: 0.80,
        },
        per_race_salary: 350,
    };

    // When the player founds "Apex Racing Team", purchases a companion Tier 1 car ("gt_bmr_bavaria_t1" for 12,000 Cr),
    // and hires candidate "marco_rossi" at 350 Cr/race
    let res = champ.found_player_team(
        &mut profile,
        &mut career,
        "Apex Racing Team",
        "gt_bmr_bavaria_t1",
        12_000,
        contract,
    );
    assert!(res.is_ok(), "Founding team should succeed: {:?}", res);

    // Then 12,000 Cr is deducted from the player's wallet (leaving 8,000 Cr)
    assert_eq!(profile.credits, 8_000);

    // And "gt_bmr_bavaria_t1" is added to the player's unlocked cars
    assert!(career.unlocked_cars.contains(&"gt_bmr_bavaria_t1".to_string()));
    assert!(profile.owned_cars.contains(&"gt_bmr_bavaria_t1".to_string()));

    // And both "player" and "marco_rossi" are registered in the championship standings under team_name == "Apex Racing Team"
    let player_entry = champ
        .standings
        .iter()
        .find(|s| s.driver_id == "player")
        .expect("Player should be in standings");
    let marco_entry = champ
        .standings
        .iter()
        .find(|s| s.driver_id == "marco_rossi")
        .expect("Marco Rossi should be in standings");
    assert_eq!(player_entry.team_name, "Apex Racing Team");
    assert_eq!(marco_entry.team_name, "Apex Racing Team");
    assert_eq!(
        marco_entry.car_model_id.as_deref(),
        Some("gt_bmr_bavaria_t1")
    );

    let team_entry = champ
        .team_standings
        .iter()
        .find(|t| t.team_name == "Apex Racing Team")
        .expect("Apex Racing Team should be in team_standings");
    assert!(team_entry.is_player_owned);
    assert_eq!(team_entry.driver_ids.len(), 2);
}

#[test]
fn test_scenario_5_insufficient_credits_prevents_team_founding_while_allowing_solo_entry() {
    // Given a player with 5,000 Cr and a companion car price of 12,000 Cr plus a minimum driver salary of 350 Cr
    let mut profile = PlayerProfile::default();
    profile.credits = 5_000;
    let mut career = ModuleCareerProgress::default_for_gt(1);

    let mut champ = SeriesSession::new(
        "GT4 Clubsport Cup",
        PointSystem::FiaStandard {
            fastest_lap_bonus: false,
        },
        vec!["monza".to_string()],
        3,
        &[
            ("player", "Player", "Solo Privateer"),
            ("magnus_wolff", "Magnus Wolff", "Bullseye GT"),
        ],
    );

    let contract = DriverHireQuote {
        character_id: "marco_rossi".to_string(),
        character_name: "Marco Rossi".to_string(),
        style: DrivingStyle::Aggressive,
        tier: DriverTier::Rookie,
        stats: DriverStats {
            speed: 0.78,
            aggression: 0.88,
            precision: 0.75,
            defense: 0.80,
        },
        per_race_salary: 350,
    };

    // When the player attempts to select "FOUND / MANAGE PLAYER TEAM"
    let err = champ
        .found_player_team(
            &mut profile,
            &mut career,
            "Apex Racing Team",
            "gt_bmr_bavaria_t1",
            12_000,
            contract,
        )
        .expect_err("Insufficient credits must reject team founding");

    // Then the action is blocked with an "INSUFFICIENT FUNDS" notification
    assert!(
        err.contains("INSUFFICIENT FUNDS"),
        "Expected INSUFFICIENT FUNDS error, got: {}",
        err
    );
    assert_eq!(profile.credits, 5_000, "Credits must remain untouched");

    // And the player remains able to start the championship as an independent solo driver
    assert!(champ.player_team.is_none());
    assert_eq!(champ.standings[0].driver_id, "player");
}

#[test]
fn test_scenario_6_post_race_salary_deduction_and_50_percent_teammate_prize_dividend() {
    // Given an active championship where the player owns "Apex Racing Team" with hired teammate "marco_rossi" (salary 400 Cr/race)
    // and current wallet balance 5,000 Cr
    let mut profile = PlayerProfile::default();
    profile.credits = 17_000;
    let mut career = ModuleCareerProgress::default_for_gt(1);

    let mut champ = SeriesSession::new(
        "GT4 Clubsport Cup",
        PointSystem::FiaStandard {
            fastest_lap_bonus: false,
        },
        vec!["monza".to_string(), "spa".to_string()],
        3,
        &[
            ("player", "Player", "Solo Entry"),
            ("magnus_wolff", "Magnus Wolff", "Bullseye GT"),
        ],
    );

    let contract = DriverHireQuote {
        character_id: "marco_rossi".to_string(),
        character_name: "Marco Rossi".to_string(),
        style: DrivingStyle::Aggressive,
        tier: DriverTier::Rookie,
        stats: DriverStats {
            speed: 0.78,
            aggression: 0.88,
            precision: 0.75,
            defense: 0.80,
        },
        per_race_salary: 400,
    };

    champ
        .found_player_team(
            &mut profile,
            &mut career,
            "Apex Racing Team",
            "gt_bmr_bavaria_t1",
            12_000,
            contract,
        )
        .unwrap();
    assert_eq!(profile.credits, 5_000);

    // When a championship round finishes with the player earning 1,500 Cr personal prize money
    // and "marco_rossi" finishing in a position worth 1,000 Cr gross prize purse
    profile.add_credits(1_500);
    let (dividend, salary_paid, deferred_warning) =
        champ.settle_post_race_team_finances(&mut profile, 2, 1_000);

    // Then the player receives +500 Cr (50% teammate prize dividend) and pays -400 Cr (teammate salary)
    assert_eq!(dividend, 500);
    assert_eq!(salary_paid, 400);
    assert!(deferred_warning.is_none());

    // And the player's final wallet balance is updated to 6,600 Cr (5,000 + 1,500 + 500 - 400)
    assert_eq!(profile.credits, 6_600);

    // And the post-race XpAwardReceipt displays both the dividend and salary line items
    let receipt = XpAwardReceipt {
        credits_earned: 1_500,
        new_credits_balance: profile.credits,
        teammate_dividend_credits: dividend,
        teammate_salary_paid: salary_paid,
        teammate_name: Some("Marco Rossi".to_string()),
        teammate_finish_pos: Some(2),
        salary_deferred_warning: deferred_warning,
        ..Default::default()
    };
    let ledger = receipt
        .format_constructor_ledger()
        .expect("Ledger string should be formatted");
    assert!(ledger.contains("+500 Cr"), "Ledger should show +500 Cr dividend: {}", ledger);
    assert!(ledger.contains("-400 Cr"), "Ledger should show -400 Cr salary: {}", ledger);
}

#[test]
fn test_scenario_7_end_of_season_hired_bot_progression_and_salary_renegotiation() {
    // Given a completed championship season where the player's hired Tier 1 (Rookie) teammate finished P2 overall in the Drivers' Standings
    let mut profile = PlayerProfile::default();
    profile.credits = 25_000;
    let mut career = ModuleCareerProgress::default_for_gt(1);

    let mut champ = SeriesSession::new(
        "GT4 Clubsport Cup",
        PointSystem::FiaStandard {
            fastest_lap_bonus: false,
        },
        vec!["monza".to_string()],
        3,
        &[
            ("player", "Player", "Apex Racing Team"),
            ("magnus_wolff", "Magnus Wolff", "Bullseye GT"),
            ("cedric_laval", "Cédric Laval", "Valente GT"),
            ("lawrence_sterling", "Lawrence Sterling", "Valente GT"),
            ("felix_navarro", "Félix Navarro", "Albion GT"),
            ("gideon_palmer", "Gideon Palmer", "Silberstern GT"),
        ],
    );

    let contract = DriverHireQuote {
        character_id: "marco_rossi".to_string(),
        character_name: "Marco Rossi".to_string(),
        style: DrivingStyle::Aggressive,
        tier: DriverTier::Rookie,
        stats: DriverStats {
            speed: 0.80,
            aggression: 0.85,
            precision: 0.78,
            defense: 0.75,
        },
        per_race_salary: 350,
    };

    champ
        .found_player_team(
            &mut profile,
            &mut career,
            "Apex Racing Team",
            "gt_bmr_bavaria_t1",
            12_000,
            contract,
        )
        .unwrap();

    // Finish the season with player P1 and Marco Rossi P2
    champ.submit_round_results(
        "Monza",
        vec![
            RoundDriverResult {
                driver_id: "player".to_string(),
                driver_name: "Player".to_string(),
                team_name: "Apex Racing Team".to_string(),
                finish_position: 1,
                total_time: 100.0,
                best_lap: Some(33.0),
                points_awarded: 0,
                has_fastest_lap: false,
            },
            RoundDriverResult {
                driver_id: "marco_rossi".to_string(),
                driver_name: "Marco Rossi".to_string(),
                team_name: "Apex Racing Team".to_string(),
                finish_position: 2,
                total_time: 101.0,
                best_lap: Some(33.3),
                points_awarded: 0,
                has_fastest_lap: false,
            },
            RoundDriverResult {
                driver_id: "magnus_wolff".to_string(),
                driver_name: "Magnus Wolff".to_string(),
                team_name: "Bullseye GT".to_string(),
                finish_position: 3,
                total_time: 102.0,
                best_lap: Some(33.6),
                points_awarded: 0,
                has_fastest_lap: false,
            },
        ],
    );
    assert!(champ.is_completed);

    // Find a deterministic seed that triggers the ~80% StepUp advancement roll for P2 (rank_0 = 1) out of 6 drivers
    let winning_seed = (1u64..100)
        .find(|&s| {
            let outcome =
                SeriesSession::evaluate_hired_bot_progression(DriverTier::Rookie, 1, 6, s);
            outcome.new_tier() == DriverTier::Amateur
        })
        .expect("Should find a seed that triggers StepUp to Amateur");

    // When end-of-season progression is evaluated and the advancement roll succeeds
    let report = champ
        .evaluate_and_update_teammate_contract(winning_seed)
        .expect("Progression report should be generated");

    // Then the teammate's DriverTier advances from Tier 1 (Rookie) to Tier 2 (Amateur)
    assert!(report.outcome.is_promoted(DriverTier::Rookie));
    assert_eq!(report.previous_tier, DriverTier::Rookie);
    assert_eq!(report.new_tier, DriverTier::Amateur);

    // And the teammate's contract renewal salary increases proportionally to reflect their new Tier 2 (Amateur) status
    assert!(
        report.new_salary > report.previous_salary,
        "Promoted teammate salary ({}) must exceed previous Tier 1 salary ({})",
        report.new_salary,
        report.previous_salary
    );
    assert_eq!(
        champ.player_team.as_ref().unwrap().contract.tier,
        DriverTier::Amateur
    );
    assert_eq!(
        champ.player_team.as_ref().unwrap().contract.per_race_salary,
        report.new_salary
    );
}

#[test]
fn test_cooperative_teammate_avoidance_flag_on_bot_ai_driver() {
    let bot = BotAiDriver::new(BotProfile::pro()).with_player_teammate(true);
    assert!(bot.is_player_teammate);
}
