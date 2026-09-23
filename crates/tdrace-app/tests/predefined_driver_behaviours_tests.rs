//! Integration tests for predefined driver behaviours (characters) across all modules and game modes.

use std::collections::HashSet;
use tdrace_app::ai::{BotProfile, DriverCharacter, DriverStats};
use tdrace_app::game::RaceSession;
use tdrace_app::series::format::{DriverConfig, SeriesDefinition};
use tdrace_app::ui::menu::GameMode;

#[test]
fn test_all_across_modules_integrity_and_distinct_ids() {
    let all = DriverCharacter::all_across_modules();
    assert_eq!(all.len(), 72, "Must contain exactly 72 drivers (12 per module * 6 modules)");

    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for d in &all {
        assert!(ids.insert(d.id), "Duplicate driver ID found: {}", d.id);
        assert!(!d.name.is_empty(), "Driver name must not be empty");
        assert!(!d.bio.is_empty(), "Driver bio must not be empty");
        assert!(d.default_profile().speed_factor > 0.0, "Speed factor must be positive for {}", d.name);
        assert!(d.default_profile().lookahead_time > 0.0, "Lookahead must be positive for {}", d.name);
        names.insert(d.name);
    }
    assert_eq!(names.len(), 72, "All 72 drivers should have unique names");
}

#[test]
fn test_find_global_resolves_all_modules() {
    // Check one known driver from each of the 6 modules
    let classic = DriverCharacter::find_global("silvia_tanaka").expect("silvia_tanaka in classic");
    assert_eq!(classic.name, "Silvia Tanaka");

    let gt = DriverCharacter::find_global("max_hunter").expect("max_hunter in gt");
    assert_eq!(gt.name, "Max Hunter");

    let nascar = DriverCharacter::find_global("dale_vance").expect("dale_vance in nascar");
    assert_eq!(nascar.name, "Dale 'The Intimidator' Vance");

    let rally = DriverCharacter::find_global("johan_vance").expect("johan_vance in rally");
    assert_eq!(rally.name, "Johan Vance");

    let kart = DriverCharacter::find_global("marco_armani").expect("marco_armani in kart");
    assert_eq!(kart.name, "Marco Armani");

    let offroad = DriverCharacter::find_global("astrid_lindholm").expect("astrid_lindholm in offroad");
    assert_eq!(offroad.name, "Astrid Lindholm");

    // Non-existent driver returns None
    assert!(DriverCharacter::find_global("non_existent_driver_xyz").is_none());
}

#[test]
fn test_behavior_archetypes_diversity() {
    let archetypes = [
        "smooth",
        "aggressive",
        "tenacious",
        "calculating",
        "fast",
        "balanced",
        "strategic",
        "bold",
        "rookie",
    ];

    let mut profiles = Vec::new();
    let mut stats_list = Vec::new();

    for arch in &archetypes {
        let p = BotProfile::from_archetype(arch);
        let s = DriverStats::from_archetype(arch);
        profiles.push(p);
        stats_list.push(s);
    }

    // Verify smooth vs aggressive
    let smooth = BotProfile::from_archetype("smooth");
    let aggressive = BotProfile::from_archetype("aggressive");
    assert!(smooth.brake_margin > aggressive.brake_margin, "Smooth drivers brake earlier (larger margin)");
    assert!(aggressive.aggression > smooth.aggression, "Aggressive drivers have higher overtaking aggression");
    assert!(aggressive.speed_factor > smooth.speed_factor, "Aggressive drivers push entry speed more");

    // Verify calculating vs bold
    let calculating = BotProfile::from_archetype("calculating");
    let bold = BotProfile::from_archetype("bold");
    assert!(calculating.avoidance_distance > bold.avoidance_distance, "Calculating drivers maintain safer clearance");
    assert!(bold.steering_kp > calculating.steering_kp, "Bold drivers have sharper steering gain");

    // Verify stats diversity
    let s_smooth = DriverStats::from_archetype("smooth");
    let s_aggressive = DriverStats::from_archetype("aggressive");
    assert!(s_smooth.precision > s_aggressive.precision, "Smooth drivers have higher precision");
    assert!(s_aggressive.aggression > s_smooth.aggression, "Aggressive drivers have higher aggression stat");
}

#[test]
fn test_index_based_archetypes() {
    for idx in 0..8 {
        let p = BotProfile::archetype_for_index(idx);
        let s = DriverStats::archetype_for_index(idx);
        assert!(p.speed_factor >= 0.85 && p.speed_factor <= 1.10);
        assert!(s.speed >= 0.70 && s.speed <= 1.0);
    }

    // Adjacent index archetypes should have different parameters
    let p0 = BotProfile::archetype_for_index(0);
    let p1 = BotProfile::archetype_for_index(1);
    assert_ne!(p0.speed_factor, p1.speed_factor);
    assert_ne!(p0.brake_margin, p1.brake_margin);
}

#[test]
fn test_seeded_sampling_variation_and_determinism() {
    let pool = DriverCharacter::ROSTER;
    let seed1 = 42;
    let seed2 = 99999;

    let sample1a = DriverCharacter::sample_from_slice(&pool, 6, seed1);
    let sample1b = DriverCharacter::sample_from_slice(&pool, 6, seed1);
    let sample2 = DriverCharacter::sample_from_slice(&pool, 6, seed2);

    assert_eq!(sample1a.len(), 6);
    assert_eq!(sample1b.len(), 6);
    assert_eq!(sample2.len(), 6);

    // Deterministic with same seed
    let ids1a: Vec<&str> = sample1a.iter().map(|d| d.id).collect();
    let ids1b: Vec<&str> = sample1b.iter().map(|d| d.id).collect();
    assert_eq!(ids1a, ids1b, "Same seed must produce identical opponent selection");

    // Different with different seed
    let ids2: Vec<&str> = sample2.iter().map(|d| d.id).collect();
    assert_ne!(ids1a, ids2, "Different seeds should produce different driver grids");
}

#[test]
fn test_series_ai_character_propagation_and_distinct_profiles() {
    let mut session = RaceSession::new();
    session.active_module_id = "gt";

    // Create a series definition with custom archetype tags
    let mut def = SeriesDefinition::default();
    def.series.id = "custom_gt_test".into();
    def.series.name = "Custom GT Test Cup".into();
    def.series.module_id = "gt".into();
    def.series.bot_count = Some(4);

    def.drivers = vec![
        DriverConfig {
            id: "player".into(),
            name: "Player".into(),
            team: "Player Team".into(),
            is_player: true,
            car_model_id: Some("gt_toyota_supra_gt4".into()),
            country: None,
            ai_character: None,
            ai_style: None,
            ai_tier: None,
            livery_idx: None,
        },
        DriverConfig {
            id: "custom_d1".into(),
            name: "Driver One".into(),
            team: "Team Alpha".into(),
            is_player: false,
            car_model_id: Some("gt_porsche_718_gt4".into()),
            country: None,
            ai_character: Some("aggressive".into()),
            ai_style: None,
            ai_tier: None,
            livery_idx: None,
        },
        DriverConfig {
            id: "custom_d2".into(),
            name: "Driver Two".into(),
            team: "Team Beta".into(),
            is_player: false,
            car_model_id: Some("gt_bmw_m4_gt4".into()),
            country: None,
            ai_character: Some("smooth".into()),
            ai_style: None,
            ai_tier: None,
            livery_idx: None,
        },
        DriverConfig {
            id: "custom_d3".into(),
            name: "Driver Three".into(),
            team: "Team Gamma".into(),
            is_player: false,
            car_model_id: Some("gt_aston_vantage_gt4".into()),
            country: None,
            ai_character: Some("calculating".into()),
            ai_style: None,
            ai_tier: None,
            livery_idx: None,
        },
        DriverConfig {
            id: "custom_d4".into(),
            name: "Driver Four".into(),
            team: "Team Delta".into(),
            is_player: false,
            car_model_id: Some("gt_amg_gt4".into()),
            country: None,
            ai_character: Some("bold".into()),
            ai_style: None,
            ai_tier: None,
            livery_idx: None,
        },
    ];

    // Launch championship
    session.launch_or_resume_championship(&def);

    // Verify championship session entries retained ai_character
    let champ = session.championship_session.as_ref().expect("Championship session must be set");
    let entry_d1 = champ.standings.iter().find(|s| s.driver_id == "custom_d1").expect("custom_d1 in standings");
    assert_eq!(entry_d1.ai_character.as_deref(), Some("aggressive"));

    let entry_d2 = champ.standings.iter().find(|s| s.driver_id == "custom_d2").expect("custom_d2 in standings");
    assert_eq!(entry_d2.ai_character.as_deref(), Some("smooth"));

    // Verify opponent drivers have distinct profiles based on archetypes
    assert_eq!(session.opponent_drivers.len(), 4);
    let d1_driver = session.opponent_drivers.iter().find(|d| d.id == "custom_d1").expect("custom_d1 driver");
    let d2_driver = session.opponent_drivers.iter().find(|d| d.id == "custom_d2").expect("custom_d2 driver");

    assert_eq!(d1_driver.default_profile().aggression, BotProfile::aggressive().aggression);
    assert_eq!(d2_driver.default_profile().aggression, BotProfile::smooth().aggression);
    assert_ne!(d1_driver.default_profile().brake_margin, d2_driver.default_profile().brake_margin);

    // Verify ai_drivers instantiated in the race session match the profile parameters
    assert_eq!(session.ai_drivers.len(), 4);
    assert_eq!(session.ai_drivers[0].profile.aggression, d1_driver.default_profile().aggression);
    assert_eq!(session.ai_drivers[1].profile.aggression, d2_driver.default_profile().aggression);
}

#[test]
fn test_all_modules_standard_race_instantiates_predefined_characters() {
    let modules = ["classic", "gt", "nascar", "rally", "kart", "extreme_offroad"];

    for module_id in &modules {
        let mut session = RaceSession::new();
        session.switch_to_module(module_id);
        session.game_mode = GameMode::StandardRace;
        session.total_laps = 3;
        session.init_race();

        let expected_bots = session.num_bots;
        assert_eq!(session.opponent_drivers.len(), expected_bots, "Module {} should spawn {} opponent drivers", module_id, expected_bots);
        assert_eq!(session.ai_drivers.len(), expected_bots, "Module {} should have {} AI drivers", module_id, expected_bots);

        // Verify all opponent drivers have distinct names and valid profiles
        let mut opponent_names = HashSet::new();
        for opp in &session.opponent_drivers {
            assert!(opponent_names.insert(opp.name), "Module {} should have unique opponent names on grid", module_id);
            assert!(opp.default_profile().speed_factor > 0.0);
            assert!(opp.default_profile().lookahead_time > 0.0);
            assert!(opp.default_stats().speed > 0.0);
        }
    }
}
