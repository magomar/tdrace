//! Integration and regression tests for Orthogonal AI Driving Styles (6 styles)
//! and Quality Tiers (5 tiers matching vehicle tiers 1..=5) according to Spec 023.

use tdrace_app::ai::{BotProfile, DriverCharacter, DriverQuality, DriverStats, DriverTier, DrivingStyle};
use tdrace_app::game::RaceSession;
use tdrace_app::series::format::ChampionshipDefinition;

const ALL_STYLES: [DrivingStyle; 6] = [
    DrivingStyle::Smooth,
    DrivingStyle::Aggressive,
    DrivingStyle::Tenacious,
    DrivingStyle::Calculating,
    DrivingStyle::Bold,
    DrivingStyle::Balanced,
];

const ALL_TIERS: [DriverTier; 5] = [
    DriverTier::Rookie,
    DriverTier::Amateur,
    DriverTier::Contender,
    DriverTier::Pro,
    DriverTier::Legend,
];

#[test]
fn test_all_30_combinations_generate_valid_bounded_profiles() {
    let mut count = 0;
    for style in ALL_STYLES {
        for tier in ALL_TIERS {
            count += 1;
            let quality = DriverQuality::for_tier(tier);
            let profile = BotProfile::from_style_and_quality(style, &quality);
            let stats = DriverStats::from_style_and_quality(style, &quality);

            // Bounded physical constraints
            assert!(
                profile.speed_factor >= 0.80 && profile.speed_factor <= 1.15,
                "speed_factor out of bounds: {} for {:?} {:?}",
                profile.speed_factor,
                style,
                tier
            );
            assert!(
                profile.brake_margin >= 0.80 && profile.brake_margin <= 1.45,
                "brake_margin out of bounds: {} for {:?} {:?}",
                profile.brake_margin,
                style,
                tier
            );
            assert!(
                profile.avoidance_distance >= 3.5 && profile.avoidance_distance <= 12.0,
                "avoidance_distance out of bounds: {} for {:?} {:?}",
                profile.avoidance_distance,
                style,
                tier
            );
            assert!(
                profile.lookahead_time >= 0.20 && profile.lookahead_time <= 0.55,
                "lookahead_time out of bounds: {} for {:?} {:?}",
                profile.lookahead_time,
                style,
                tier
            );
            assert!(
                profile.steering_kp >= 1.5 && profile.steering_kp <= 3.5,
                "steering_kp out of bounds: {} for {:?} {:?}",
                profile.steering_kp,
                style,
                tier
            );
            assert!(
                profile.steering_kd >= 0.03 && profile.steering_kd <= 0.12,
                "steering_kd out of bounds: {} for {:?} {:?}",
                profile.steering_kd,
                style,
                tier
            );
            assert!(
                profile.aggression >= 0.20 && profile.aggression <= 1.00,
                "aggression out of bounds: {} for {:?} {:?}",
                profile.aggression,
                style,
                tier
            );

            // Verify non-NaN
            assert!(!profile.speed_factor.is_nan());
            assert!(!profile.brake_margin.is_nan());
            assert!(!profile.avoidance_distance.is_nan());
            assert!(!profile.lookahead_time.is_nan());

            // Stats boundaries
            assert!(stats.speed >= 0.65 && stats.speed <= 1.0);
            assert!(stats.aggression >= 0.40 && stats.aggression <= 1.0);
            assert!(stats.precision >= 0.50 && stats.precision <= 1.0);
            assert!(stats.defense >= 0.50 && stats.defense <= 1.0);
        }
    }
    assert_eq!(count, 30, "Must generate exactly 30 unique combinations (6 styles * 5 tiers)");
}

#[test]
fn test_monotonic_pace_scaling_across_tiers() {
    for style in ALL_STYLES {
        let p_rookie = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Rookie));
        let p_amateur = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Amateur));
        let p_contender = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Contender));
        let p_pro = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Pro));
        let p_legend = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Legend));

        assert!(
            p_rookie.speed_factor < p_amateur.speed_factor,
            "Rookie pace must be lower than Amateur for {:?}",
            style
        );
        assert!(
            p_amateur.speed_factor < p_contender.speed_factor,
            "Amateur pace must be lower than Contender for {:?}",
            style
        );
        assert!(
            p_contender.speed_factor < p_pro.speed_factor,
            "Contender pace must be lower than Pro for {:?}",
            style
        );
        assert!(
            p_pro.speed_factor < p_legend.speed_factor,
            "Pro pace must be lower than Legend for {:?}",
            style
        );
    }
}

#[test]
fn test_monotonic_braking_safety_padding_across_tiers() {
    for style in ALL_STYLES {
        let p_rookie = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Rookie));
        let p_amateur = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Amateur));
        let p_contender = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Contender));
        let p_pro = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Pro));
        let p_legend = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Legend));

        // Lower tier = larger brake margin (brakes earlier / safer)
        assert!(
            p_rookie.brake_margin > p_amateur.brake_margin,
            "Rookie brake margin must be greater than Amateur for {:?}",
            style
        );
        assert!(
            p_amateur.brake_margin > p_contender.brake_margin,
            "Amateur brake margin must be greater than Contender for {:?}",
            style
        );
        assert!(
            p_contender.brake_margin > p_pro.brake_margin,
            "Contender brake margin must be greater than Pro for {:?}",
            style
        );
        assert!(
            p_pro.brake_margin > p_legend.brake_margin,
            "Pro brake margin must be greater than Legend for {:?}",
            style
        );
    }
}

#[test]
fn test_monotonic_avoidance_distance_padding_across_tiers() {
    for style in ALL_STYLES {
        let p_rookie = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Rookie));
        let p_amateur = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Amateur));
        let p_contender = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Contender));
        let p_pro = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Pro));
        let p_legend = BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(DriverTier::Legend));

        // Lower tier = larger avoidance buffer
        assert!(p_rookie.avoidance_distance > p_amateur.avoidance_distance);
        assert!(p_amateur.avoidance_distance > p_contender.avoidance_distance);
        assert!(p_contender.avoidance_distance > p_pro.avoidance_distance);
        assert!(p_pro.avoidance_distance > p_legend.avoidance_distance);
    }
}

#[test]
fn test_compound_and_legacy_archetype_resolution() {
    // Compound style_tier strings
    let agg_rookie = BotProfile::from_archetype("aggressive_rookie");
    let agg_legend = BotProfile::from_archetype("aggressive_legend");
    assert!(agg_legend.speed_factor > agg_rookie.speed_factor);
    assert!(agg_rookie.brake_margin > agg_legend.brake_margin);

    let smooth_rookie = BotProfile::from_archetype("smooth_rookie");
    let smooth_pro = BotProfile::from_archetype("smooth_pro");
    assert!(smooth_pro.speed_factor > smooth_rookie.speed_factor);

    // Legacy names
    let rookie = BotProfile::from_archetype("rookie");
    assert!(rookie.speed_factor < 0.95);
    assert!(rookie.brake_margin > 1.20);

    let fast = BotProfile::from_archetype("fast");
    assert!(fast.speed_factor > 1.04);
}

#[test]
fn test_series_toml_with_orthogonal_style_and_tier() {
    let toml_str = r#"
[championship]
id = "custom_cup"
name = "Custom Cup"
description = "Test championship"
module_id = "gt"
tier = 3
laps_per_round = 3

[scoring]
system = "standard"

[[rounds]]
order = 1
track_id = "classic_grand_prix"

[[drivers]]
id = "player"
name = "Player 1"
team = "Apex"
is_player = true
car_model_id = "gt_toyota_supra_gt4"

[[drivers]]
id = "rival_brawler"
name = "Wild Rookie"
team = "Rookie Team"
is_player = false
ai_style = "aggressive"
ai_tier = 1

[[drivers]]
id = "rival_veteran"
name = "Pro Tactician"
team = "Pro Team"
is_player = false
ai_style = "calculating"
ai_tier = 4
"#;

    let def: ChampionshipDefinition = toml::from_str(toml_str).expect("Valid TOML series definition");
    assert_eq!(def.drivers.len(), 3);
    assert_eq!(def.drivers[1].ai_style.as_deref(), Some("aggressive"));
    assert_eq!(def.drivers[1].ai_tier, Some(1));
    assert_eq!(def.drivers[2].ai_style.as_deref(), Some("calculating"));
    assert_eq!(def.drivers[2].ai_tier, Some(4));

    let mut session = RaceSession::new();
    session.active_module_id = "gt";
    session.launch_or_resume_championship(&def);

    // Verify opponents were initialized with their orthogonal profiles
    assert_eq!(session.opponent_drivers.len(), 2);
    let opp1 = &session.opponent_drivers[0];
    let opp2 = &session.opponent_drivers[1];

    // Rookie aggressive opp1 should have lower pace and higher brake margin than pro opp2
    assert!(opp1.profile.brake_margin > opp2.profile.brake_margin);
    assert!(opp1.profile.speed_factor < opp2.profile.speed_factor);
}

#[test]
fn test_driver_character_classification() {
    let silvia = DriverCharacter::find_global("silvia_tanaka").expect("Silvia Tanaka");
    assert_eq!(silvia.style, DrivingStyle::Smooth);

    let marco = DriverCharacter::find_global("marco_rossi").expect("Marco Rossi");
    assert_eq!(marco.style, DrivingStyle::Aggressive);
}
