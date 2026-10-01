//! # Classic Academy & Rookie Profile Unit Tests (Spec 060)

use arcade_race_core::profile::{
    AcademyLessonDef, AcademyLessonId, AcademyMedal, ClassicAcademyProgress,
};

#[test]
fn test_default_curriculum_structure() {
    let curriculum = AcademyLessonDef::default_curriculum();
    assert_eq!(curriculum.len(), 4);

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
