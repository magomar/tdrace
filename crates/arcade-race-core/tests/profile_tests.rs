//! # Classic Academy & Rookie Profile Unit Tests (Spec 060)

use arcade_race_core::profile::{
    AcademyAttemptResult, AcademyLessonDef, AcademyLessonId, AcademyMedal, AcademyObjective,
    ClassicAcademyProgress, LicenseGrade, ACADEMY_DAILY_STIPEND_CAP,
};

#[test]
fn test_default_curriculum_structure() {
    let curriculum = AcademyLessonDef::default_curriculum();
    // Tier 1 (Spec 060) first and unchanged, then Tier 2 and Tier 3 (Spec 064).
    assert_eq!(curriculum.len(), 12);
    assert_eq!(AcademyLessonDef::tier_curriculum(1).len(), 4);

    let l1 = &curriculum[0];
    assert_eq!(l1.id, AcademyLessonId::Lesson1ApexLine);
    assert_eq!(l1.track_slug, "gt_velocity_park");
    assert_eq!(l1.gold_time_sec, 18.5);
    assert_eq!(l1.silver_time_sec, 20.0);
    assert_eq!(l1.bronze_time_sec, 22.5);
    assert_eq!(l1.bronze_credit_bounty, 1_000);
    assert_eq!(l1.silver_credit_bounty, 500);
    assert_eq!(l1.gold_credit_bounty, 500);

    let l4 = &curriculum[3];
    assert_eq!(l4.id, AcademyLessonId::Lesson4GraduationSprint);
    assert_eq!(l4.track_slug, "ax_meadow_sprint");
    assert_eq!(l4.gold_time_sec, 68.0);
    assert_eq!(l4.silver_time_sec, 73.0);
    assert_eq!(l4.bronze_time_sec, 80.0);
}

#[test]
fn test_fresh_academy_initialization_and_lesson_unlock_progression() {
    let mut academy = ClassicAcademyProgress::new();
    assert!(!academy.is_graduated());
    assert_eq!(academy.total_stars(), 0);

    // Lesson 1 is unlocked by default
    assert!(academy.is_lesson_unlocked(AcademyLessonId::Lesson1ApexLine));
    // Subsequent lessons are locked until predecessor is passed
    assert!(!academy.is_lesson_unlocked(AcademyLessonId::Lesson2BrakingChicane));
    assert!(!academy.is_lesson_unlocked(AcademyLessonId::Lesson3SurfaceTransition));
    assert!(!academy.is_lesson_unlocked(AcademyLessonId::Lesson4GraduationSprint));

    // Pass Lesson 1 with Bronze
    let (medal, credits, xp) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 21.0, true);
    assert_eq!(medal, AcademyMedal::Bronze);
    assert_eq!(credits, 1_000);
    assert_eq!(xp, 100);
    assert!(academy.is_lesson_unlocked(AcademyLessonId::Lesson2BrakingChicane));
    assert!(!academy.is_lesson_unlocked(AcademyLessonId::Lesson3SurfaceTransition));
}

#[test]
fn test_idempotent_bounty_payouts_on_replay() {
    let mut academy = ClassicAcademyProgress::new();

    // First attempt: Silver medal (gets Bronze 1000 + Silver 500 = 1500)
    let (medal, credits, xp) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 19.5, true);
    assert_eq!(medal, AcademyMedal::Silver);
    assert_eq!(credits, 1_500);
    assert_eq!(xp, 150);

    // Replay with worse time: 0 credits, best time unchanged
    let (medal_r1, credits_r1, xp_r1) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 20.5, true);
    assert_eq!(medal_r1, AcademyMedal::Bronze);
    assert_eq!(credits_r1, 0);
    assert_eq!(xp_r1, 0);

    // Replay with same Silver time: 0 credits
    let (medal_r2, credits_r2, xp_r2) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 19.5, true);
    assert_eq!(medal_r2, AcademyMedal::Silver);
    assert_eq!(credits_r2, 0);
    assert_eq!(xp_r2, 0);

    // Replay with Gold: gets remaining Gold bounty (500 Cr)
    let (medal_r3, credits_r3, xp_r3) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 17.8, true);
    assert_eq!(medal_r3, AcademyMedal::Gold);
    assert_eq!(credits_r3, 500);
    assert_eq!(xp_r3, 50);

    // Replay Gold again: 0 credits
    let (medal_r4, credits_r4, xp_r4) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 17.5, true);
    assert_eq!(medal_r4, AcademyMedal::Gold);
    assert_eq!(credits_r4, 0);
    assert_eq!(xp_r4, 0);
}

#[test]
fn test_all_bronze_curriculum_purse_totals_7000() {
    let mut academy = ClassicAcademyProgress::new();
    let mut total_credits = 0u64;

    // Lesson 1 bronze
    let (_, cr1, _) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 22.0, true);
    total_credits += cr1;

    // Lesson 2 bronze
    let (_, cr2, _) = academy.record_attempt(AcademyLessonId::Lesson2BrakingChicane, 28.5, true);
    total_credits += cr2;

    // Lesson 3 bronze
    let (_, cr3, _) = academy.record_attempt(AcademyLessonId::Lesson3SurfaceTransition, 38.5, true);
    total_credits += cr3;

    // Lesson 4 bronze
    let (_, cr4, _) = academy.record_attempt(AcademyLessonId::Lesson4GraduationSprint, 79.0, true);
    total_credits += cr4;

    assert_eq!(total_credits, 7_000);
    assert!(academy.is_graduated());
    assert!(academy.license_granted);
}

#[test]
fn test_all_silver_curriculum_purse_totals_10750() {
    let mut academy = ClassicAcademyProgress::new();
    let mut total_credits = 0u64;

    let (_, cr1, _) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 19.5, true);
    total_credits += cr1;
    let (_, cr2, _) = academy.record_attempt(AcademyLessonId::Lesson2BrakingChicane, 25.5, true);
    total_credits += cr2;
    let (_, cr3, _) = academy.record_attempt(AcademyLessonId::Lesson3SurfaceTransition, 34.5, true);
    total_credits += cr3;
    let (_, cr4, _) = academy.record_attempt(AcademyLessonId::Lesson4GraduationSprint, 72.0, true);
    total_credits += cr4;

    assert_eq!(total_credits, 10_750);
    assert!(academy.is_graduated());
}

#[test]
fn test_all_gold_curriculum_purse_totals_14500() {
    let mut academy = ClassicAcademyProgress::new();
    let mut total_credits = 0u64;

    let (_, cr1, _) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 18.0, true);
    total_credits += cr1;
    let (_, cr2, _) = academy.record_attempt(AcademyLessonId::Lesson2BrakingChicane, 23.5, true);
    total_credits += cr2;
    let (_, cr3, _) = academy.record_attempt(AcademyLessonId::Lesson3SurfaceTransition, 31.0, true);
    total_credits += cr3;
    let (_, cr4, _) = academy.record_attempt(AcademyLessonId::Lesson4GraduationSprint, 67.0, true);
    total_credits += cr4;

    assert_eq!(total_credits, 14_500);
    assert_eq!(academy.total_stars(), 12);
    assert!(academy.is_graduated());
}

/// A result that earns exactly `medal` on `def`, whatever its objective.
fn result_for(def: &AcademyLessonDef, medal: AcademyMedal) -> AcademyAttemptResult {
    let target = match medal {
        AcademyMedal::Gold => def.gold_time_sec,
        AcademyMedal::Silver => def.silver_time_sec,
        _ => def.bronze_time_sec,
    };
    match def.objective {
        AcademyObjective::DriftScore => AcademyAttemptResult { drift_score: target, ..Default::default() },
        AcademyObjective::Podium { .. } => AcademyAttemptResult {
            position: match medal {
                AcademyMedal::Gold => 1,
                AcademyMedal::Silver => 2,
                _ => 3,
            },
            worst_wear: 0.1,
            ..Default::default()
        },
        AcademyObjective::WearCap { .. } => AcademyAttemptResult { time_sec: target, worst_wear: 0.1, ..Default::default() },
        _ => AcademyAttemptResult { time_sec: target, ..Default::default() },
    }
}

fn earn(academy: &mut ClassicAcademyProgress, id: AcademyLessonId, medal: AcademyMedal) -> u64 {
    let def = AcademyLessonDef::find(id).unwrap();
    academy.record_result(id, &result_for(&def, medal), true).1
}

fn graduated() -> ClassicAcademyProgress {
    let mut academy = ClassicAcademyProgress::new();
    for id in AcademyLessonId::ALL {
        earn(&mut academy, id, AcademyMedal::Bronze);
    }
    academy
}

#[test]
fn test_lesson_ids_carry_tier_slug_and_index() {
    for tier in 1..=3u8 {
        let ids = AcademyLessonId::tier_lessons(tier);
        assert_eq!(ids.len(), 4);
        for (pos, id) in ids.iter().enumerate() {
            assert_eq!(id.tier(), tier);
            assert_eq!(id.index(), (tier as usize - 1) * 4 + pos);
            assert_eq!(AcademyLessonId::from_index(id.index()), Some(*id));
        }
    }
    assert_eq!(AcademyLessonId::from_index(12), None);
    assert!(AcademyLessonId::tier_lessons(4).is_empty());
    // Tier 1 keeps its ids, slugs and order
    assert_eq!(AcademyLessonId::ALL.len(), 4);
    assert_eq!(AcademyLessonId::Lesson1ApexLine.slug(), "lesson_1_apex_line");
    let slugs: Vec<&str> = AcademyLessonId::tier_lessons(2).iter().chain(AcademyLessonId::tier_lessons(3)).map(|i| i.slug()).collect();
    assert_eq!(slugs, ["acad_2_1", "acad_2_2", "acad_2_3", "acad_2_4", "acad_3_1", "acad_3_2", "acad_3_3", "acad_3_4"]);
}

#[test]
fn test_tier_curriculum_rules_objectives_and_purses() {
    let purse = |tier: u8| -> u64 {
        AcademyLessonDef::tier_curriculum(tier)
            .iter()
            .map(|l| l.bronze_credit_bounty + l.silver_credit_bounty + l.gold_credit_bounty)
            .sum()
    };
    assert_eq!(purse(1), 14_500);
    assert_eq!(purse(2), 25_000);
    assert_eq!(purse(3), 50_000);

    let drift = AcademyLessonDef::find(AcademyLessonId::Acad2DirtDrift).unwrap();
    assert_eq!((drift.bronze_credit_bounty, drift.silver_credit_bounty, drift.gold_credit_bounty), (2_500, 1_250, 1_250));
    assert_eq!((drift.gold_time_sec, drift.silver_time_sec, drift.bronze_time_sec), (1200.0, 900.0, 600.0));
    assert_eq!(drift.objective, AcademyObjective::DriftScore);

    for def in AcademyLessonDef::tier_curriculum(1).iter().chain(AcademyLessonDef::tier_curriculum(2).iter()) {
        assert!(!def.rules.damage_enabled, "{}", def.id.slug());
        assert_eq!(def.rules.wear_rate_scale, 0.0, "{}", def.id.slug());
    }
    for def in AcademyLessonDef::tier_curriculum(3) {
        assert!(!def.rules.damage_enabled, "{}", def.id.slug());
        assert_eq!(def.rules.wear_rate_scale, 3.0, "{}", def.id.slug());
    }
    let worn = AcademyLessonDef::find(AcademyLessonId::Acad3WornRubber).unwrap();
    assert_eq!(worn.rules.start_wear, 0.65);
    let joker = AcademyLessonDef::find(AcademyLessonId::Acad2MixedQualifier).unwrap();
    assert!(joker.rules.joker_required);
    assert_eq!(joker.rules.laps, 2);
    let conserve = AcademyLessonDef::find(AcademyLessonId::Acad3TyreConservation).unwrap();
    assert_eq!(conserve.rules.bots, 3);
    assert!(!conserve.requires_clean_attempt());
    assert_eq!(
        AcademyLessonDef::find(AcademyLessonId::Acad3ThermalThrottle).unwrap().objective,
        AcademyObjective::WearCap { max_wear: 0.40 }
    );
    // Stipends per tier
    assert_eq!(AcademyLessonDef::find(AcademyLessonId::Lesson1ApexLine).unwrap().stipend_credits(), 1_500);
    assert_eq!(drift.stipend_credits(), 2_000);
    assert_eq!(worn.stipend_credits(), 3_000);
}

#[test]
fn test_tier_unlock_rules() {
    let academy = ClassicAcademyProgress::new();
    assert!(academy.is_tier_unlocked(1));
    assert!(!academy.is_tier_unlocked(2));
    assert!(!academy.is_lesson_unlocked(AcademyLessonId::Acad2DirtDrift));

    // Tier 1 stays sequential
    assert!(!academy.is_lesson_unlocked(AcademyLessonId::Lesson2BrakingChicane));

    let mut academy = graduated();
    assert!(academy.is_tier_unlocked(2));
    // Tier 2 opens all four missions together
    for id in AcademyLessonId::tier_lessons(2) {
        assert!(academy.is_lesson_unlocked(*id));
    }
    assert!(!academy.is_tier_unlocked(3));

    // Bronze on 2 of 4 keeps Tier 3 shut, a third opens it
    earn(&mut academy, AcademyLessonId::Acad2DirtDrift, AcademyMedal::Bronze);
    earn(&mut academy, AcademyLessonId::Acad2ScandinavianFlick, AcademyMedal::Gold);
    assert!(!academy.is_tier_unlocked(3));
    assert!(!academy.is_lesson_unlocked(AcademyLessonId::Acad3ThermalThrottle));
    earn(&mut academy, AcademyLessonId::Acad2RampBalance, AcademyMedal::Bronze);
    assert!(academy.is_tier_unlocked(3));
    assert!(academy.is_lesson_unlocked(AcademyLessonId::Acad3TyreConservation));
    // There is no Tier 4 in the base game
    assert!(!academy.is_tier_unlocked(4));
}

#[test]
fn test_tier_passes_grant_licenses_and_never_lower_a_grade() {
    let mut academy = graduated();
    assert_eq!(academy.license_grade, LicenseGrade::ClassD);

    earn(&mut academy, AcademyLessonId::Acad2DirtDrift, AcademyMedal::Bronze);
    earn(&mut academy, AcademyLessonId::Acad2ScandinavianFlick, AcademyMedal::Bronze);
    assert_eq!(academy.license_grade, LicenseGrade::ClassD);
    earn(&mut academy, AcademyLessonId::Acad2RampBalance, AcademyMedal::Bronze);
    assert_eq!(academy.license_grade, LicenseGrade::ClassC);

    for id in &AcademyLessonId::tier_lessons(3)[..2] {
        earn(&mut academy, *id, AcademyMedal::Bronze);
    }
    assert_eq!(academy.license_grade, LicenseGrade::ClassC);
    earn(&mut academy, AcademyLessonId::Acad3WornRubber, AcademyMedal::Bronze);
    assert_eq!(academy.license_grade, LicenseGrade::ClassB);

    // A profile that holds Class A keeps it
    let mut ace = graduated();
    ace.license_grade = LicenseGrade::ClassA;
    for id in &AcademyLessonId::tier_lessons(2)[..3] {
        earn(&mut ace, *id, AcademyMedal::Bronze);
    }
    assert_eq!(ace.license_grade, LicenseGrade::ClassA);
}

#[test]
fn test_medal_bounties_pay_once_as_deltas() {
    let mut academy = graduated();
    let id = AcademyLessonId::Acad2DirtDrift;
    assert_eq!(earn(&mut academy, id, AcademyMedal::Silver), 3_750);
    assert_eq!(earn(&mut academy, id, AcademyMedal::Gold), 1_250);
    assert_eq!(earn(&mut academy, id, AcademyMedal::Gold), 0);
    assert_eq!(earn(&mut academy, id, AcademyMedal::Bronze), 0);
    assert_eq!(academy.lessons[&id].highest_medal, AcademyMedal::Gold);
    // The stored best of a drift lesson is the best score
    let def = AcademyLessonDef::find(id).unwrap();
    assert_eq!(academy.lessons[&id].best_time_sec, Some(def.gold_time_sec));
}

#[test]
fn test_objective_medal_rules() {
    let drift = AcademyLessonDef::find(AcademyLessonId::Acad2DirtDrift).unwrap();
    let score = |s: f32| drift.evaluate_result(&AcademyAttemptResult { drift_score: s, ..Default::default() });
    assert_eq!(score(1300.0), AcademyMedal::Gold);
    assert_eq!(score(900.0), AcademyMedal::Silver);
    assert_eq!(score(600.0), AcademyMedal::Bronze);
    assert_eq!(score(599.0), AcademyMedal::None);

    // A wear over the cap gives no medal, however fast the run
    let capped = AcademyLessonDef::find(AcademyLessonId::Acad3ThermalThrottle).unwrap();
    let fast = |w: f32| capped.evaluate_result(&AcademyAttemptResult { time_sec: capped.gold_time_sec - 1.0, worst_wear: w, ..Default::default() });
    assert_eq!(fast(0.40), AcademyMedal::Gold);
    assert_eq!(fast(0.41), AcademyMedal::None);

    // Podium: position decides, under the wear cap
    let podium = AcademyLessonDef::find(AcademyLessonId::Acad3TyreConservation).unwrap();
    let place = |p: u32, w: f32| podium.evaluate_result(&AcademyAttemptResult { position: p, worst_wear: w, ..Default::default() });
    assert_eq!(place(1, 0.2), AcademyMedal::Gold);
    assert_eq!(place(2, 0.2), AcademyMedal::Silver);
    assert_eq!(place(3, 0.2), AcademyMedal::Bronze);
    assert_eq!(place(4, 0.2), AcademyMedal::Bronze);
    assert_eq!(place(1, 0.5), AcademyMedal::None);
    assert_eq!(place(0, 0.2), AcademyMedal::None);
}

#[test]
fn test_a_worn_out_attempt_counts_but_pays_nothing() {
    let mut academy = graduated();
    for id in &AcademyLessonId::tier_lessons(2)[..3] {
        earn(&mut academy, *id, AcademyMedal::Bronze);
    }
    let id = AcademyLessonId::Acad3ThermalThrottle;
    let def = AcademyLessonDef::find(id).unwrap();
    let worn_out = AcademyAttemptResult { time_sec: def.gold_time_sec - 1.0, worst_wear: 0.55, ..Default::default() };
    assert_eq!(academy.record_result(id, &worn_out, true), (AcademyMedal::None, 0, 0));
    let progress = &academy.lessons[&id];
    assert_eq!(progress.attempts, 1);
    assert!(!progress.completed);
    assert_eq!(progress.highest_medal, AcademyMedal::None);
}

#[test]
fn test_stipend_cap_is_five_per_local_day() {
    let mut academy = ClassicAcademyProgress::new();
    for _ in 0..ACADEMY_DAILY_STIPEND_CAP {
        assert!(academy.try_take_stipend("2026-10-11"));
    }
    assert!(!academy.try_take_stipend("2026-10-11"));
    assert!(!academy.try_take_stipend("2026-10-11"));
    assert_eq!(academy.stipends_today, 5);
    // The next local day pays again
    assert!(academy.try_take_stipend("2026-10-12"));
    assert_eq!(academy.stipend_day.as_deref(), Some("2026-10-12"));
    assert_eq!(academy.stipends_today, 1);
}

#[test]
fn test_spec_060_save_loads_with_its_tier_1_progress() {
    // A profile saved by the Spec 060 build: no stipend fields, four lessons.
    let json = r#"{
        "lessons": {
            "Lesson1ApexLine": {"completed": true, "best_time_sec": 18.2, "highest_medal": "Gold",
                "bronze_claimed": true, "silver_claimed": true, "gold_claimed": true, "attempts": 3},
            "Lesson2BrakingChicane": {"completed": false, "best_time_sec": null, "highest_medal": "None",
                "bronze_claimed": false, "silver_claimed": false, "gold_claimed": false, "attempts": 0}
        },
        "license_granted": false,
        "license_granted_timestamp": null,
        "license_grade": "ClassD"
    }"#;
    let mut academy: ClassicAcademyProgress = serde_json::from_str(json).unwrap();
    assert_eq!(academy.lessons[&AcademyLessonId::Lesson1ApexLine].highest_medal, AcademyMedal::Gold);
    assert_eq!(academy.stipend_day, None);
    assert_eq!(academy.stipends_today, 0);
    // Gold on lesson 1 does not pay again
    let (_, credits, _) = academy.record_attempt(AcademyLessonId::Lesson1ApexLine, 18.0, true);
    assert_eq!(credits, 0);
    // The Class D licence opens Tier 2
    assert!(academy.is_tier_unlocked(2));

    // Without the licence, Tier 2 needs Lesson 4
    let no_licence: ClassicAcademyProgress = serde_json::from_str(
        r#"{"lessons": {}, "license_granted": false, "license_granted_timestamp": null}"#,
    )
    .unwrap();
    assert!(!no_licence.is_tier_unlocked(2));
}
