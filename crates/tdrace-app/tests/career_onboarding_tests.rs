//! # Career Onboarding & Classic Academy Integration Tests (Spec 060)

use tdrace_app::db::HallOfFameDb;
use tdrace_app::profile::{
    AcademyLessonId, AcademyMedal, PlayerProfile,
};

#[test]
fn test_fresh_rookie_profile_zero_start_invariants() {
    let rookie = PlayerProfile::new_rookie("Ayrton");

    // Invariant 1: 0 Credits and 0 Lifetime Credits
    assert_eq!(rookie.credits, 0);
    assert_eq!(rookie.lifetime_credits, 0);

    // Invariant 2: Empty owned cars collection (no free starter vehicles)
    assert!(rookie.owned_cars.is_empty());

    // Invariant 3: Ungranted racing license
    assert!(!rookie.has_racing_license());
    assert!(!rookie.academy_progress.is_graduated());
    assert_eq!(rookie.academy_progress.total_stars(), 0);

    // Invariant 4: Career Mode access is strictly locked
    assert!(!rookie.can_access_career());
}

#[test]
fn test_rookie_profile_database_persistence() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");

    let mut rookie = PlayerProfile::new_rookie("Max");
    rookie.credits = 1_000;
    rookie.lifetime_credits = 1_000;
    rookie.owned_cars.push("kart_crg_hero_60".to_string());

    // Record lesson 1 completion
    let (medal, cr, _) = rookie.academy_progress.record_attempt(AcademyLessonId::Lesson1ApexLine, 18.0, true);
    assert_eq!(medal, AcademyMedal::Gold);
    assert_eq!(cr, 2_000);

    let id = db.create_profile(&rookie).expect("Create rookie profile");
    let loaded = db.get_profile_by_id(id).expect("Fetch profile").expect("Profile exists");

    assert_eq!(loaded.name, "Max");
    assert_eq!(loaded.credits, 1_000);
    assert_eq!(loaded.owned_cars, vec!["kart_crg_hero_60".to_string()]);
    assert_eq!(loaded.academy_progress.total_stars(), 3);
    assert_eq!(
        loaded.academy_progress.lessons.get(&AcademyLessonId::Lesson1ApexLine).unwrap().highest_medal,
        AcademyMedal::Gold
    );
}

#[test]
fn test_career_mode_access_requires_both_license_and_owned_vehicle() {
    let mut rookie = PlayerProfile::new_rookie("Fernando");
    assert!(!rookie.can_access_career());

    // 1. Give player a car, but NO license -> still cannot access career
    rookie.owned_cars.push("kart_crg_hero_60".to_string());
    assert!(!rookie.has_racing_license());
    assert!(!rookie.can_access_career());

    // 2. Remove car and grant license -> still cannot access career (no car to race with)
    rookie.owned_cars.clear();
    rookie.academy_progress.license_granted = true;
    assert!(rookie.has_racing_license());
    assert!(!rookie.can_access_career());

    // 3. Both license granted AND at least one car owned -> career unlocked!
    rookie.owned_cars.push("kart_crg_hero_60".to_string());
    assert!(rookie.has_racing_license());
    assert!(rookie.can_access_career());
}

#[test]
fn test_academy_curriculum_graduation_grants_license() {
    let mut rookie = PlayerProfile::new_rookie("Lando");
    assert!(!rookie.has_racing_license());

    // Complete lessons 1, 2, 3 with Bronze
    rookie.academy_progress.record_attempt(AcademyLessonId::Lesson1ApexLine, 22.0, true);
    rookie.academy_progress.record_attempt(AcademyLessonId::Lesson2BrakingChicane, 28.0, true);
    rookie.academy_progress.record_attempt(AcademyLessonId::Lesson3SurfaceTransition, 38.0, true);
    assert!(!rookie.has_racing_license());

    // Complete Lesson 4 (Graduation Sprint) with Bronze
    let (medal, cr, _) = rookie.academy_progress.record_attempt(AcademyLessonId::Lesson4GraduationSprint, 79.0, true);
    assert_eq!(medal, AcademyMedal::Bronze);
    assert_eq!(cr, 2_500);

    // Graduation ceremony grants the National Grassroots Racing License
    assert!(rookie.has_racing_license());
    assert!(rookie.academy_progress.is_graduated());
    assert!(rookie.academy_progress.license_granted);
}
