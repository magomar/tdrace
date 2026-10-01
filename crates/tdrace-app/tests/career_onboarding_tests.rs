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

#[test]
fn test_academy_all_bronze_progression_purse_and_unlocks() {
    use tdrace_app::game::academy::{self, AcademyChallengeState};

    let mut rookie = PlayerProfile::new_rookie("Alain");
    assert_eq!(rookie.credits, 0);

    // Lesson 1: Bronze (target <= 22.5s)
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson1ApexLine));
    assert!(!rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson2BrakingChicane));
    let state1 = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    let eval1 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson1ApexLine, 21.0, &state1);
    assert_eq!(eval1.medal, AcademyMedal::Bronze);
    assert_eq!(eval1.credits_awarded, 1_000);
    assert_eq!(rookie.credits, 1_000);
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson2BrakingChicane));

    // Lesson 2: Bronze (target <= 29.0s)
    let state2 = AcademyChallengeState::new(AcademyLessonId::Lesson2BrakingChicane);
    let eval2 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson2BrakingChicane, 27.5, &state2);
    assert_eq!(eval2.medal, AcademyMedal::Bronze);
    assert_eq!(eval2.credits_awarded, 1_500);
    assert_eq!(rookie.credits, 2_500);
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson3SurfaceTransition));

    // Lesson 3: Bronze (target <= 39.0s)
    let state3 = AcademyChallengeState::new(AcademyLessonId::Lesson3SurfaceTransition);
    let eval3 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson3SurfaceTransition, 37.0, &state3);
    assert_eq!(eval3.medal, AcademyMedal::Bronze);
    assert_eq!(eval3.credits_awarded, 2_000);
    assert_eq!(rookie.credits, 4_500);
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson4GraduationSprint));

    // Lesson 4: Bronze (target <= 80.0s)
    let state4 = AcademyChallengeState::new(AcademyLessonId::Lesson4GraduationSprint);
    let eval4 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson4GraduationSprint, 78.0, &state4);
    assert_eq!(eval4.medal, AcademyMedal::Bronze);
    assert_eq!(eval4.credits_awarded, 2_500);
    assert_eq!(rookie.credits, 7_000); // 7,000 Cr exact purse!
    assert!(eval4.newly_graduated);
    assert!(rookie.has_racing_license());
}

#[test]
fn test_academy_all_silver_and_gold_curriculum_purses() {
    use tdrace_app::game::academy::{self, AcademyChallengeState};

    // Silver Curriculum Run
    let mut rookie_silver = PlayerProfile::new_rookie("Niki");
    let times_silver = [
        (AcademyLessonId::Lesson1ApexLine, 19.5),
        (AcademyLessonId::Lesson2BrakingChicane, 25.5),
        (AcademyLessonId::Lesson3SurfaceTransition, 34.0),
        (AcademyLessonId::Lesson4GraduationSprint, 72.0),
    ];
    for (id, time) in times_silver {
        let state = AcademyChallengeState::new(id);
        let eval = academy::evaluate_academy_attempt(&mut rookie_silver, id, time, &state);
        assert_eq!(eval.medal, AcademyMedal::Silver);
    }
    assert_eq!(rookie_silver.credits, academy::total_curriculum_purse(AcademyMedal::Silver));
    assert_eq!(rookie_silver.credits, 10_750); // 10,750 Cr exact purse!
    assert!(rookie_silver.has_racing_license());

    // Gold Curriculum Run
    let mut rookie_gold = PlayerProfile::new_rookie("Jackie");
    let times_gold = [
        (AcademyLessonId::Lesson1ApexLine, 18.0),
        (AcademyLessonId::Lesson2BrakingChicane, 23.5),
        (AcademyLessonId::Lesson3SurfaceTransition, 31.0),
        (AcademyLessonId::Lesson4GraduationSprint, 67.0),
    ];
    for (id, time) in times_gold {
        let state = AcademyChallengeState::new(id);
        let eval = academy::evaluate_academy_attempt(&mut rookie_gold, id, time, &state);
        assert_eq!(eval.medal, AcademyMedal::Gold);
    }
    assert_eq!(rookie_gold.credits, academy::total_curriculum_purse(AcademyMedal::Gold));
    assert_eq!(rookie_gold.credits, 14_500); // 14,500 Cr exact purse!
    assert!(rookie_gold.has_racing_license());
}

#[test]
fn test_academy_attempt_idempotency_on_replay() {
    use tdrace_app::game::academy::{self, AcademyChallengeState};

    let mut rookie = PlayerProfile::new_rookie("Mika");

    // First attempt: Gold (2,000 Cr bounty)
    let state = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    let eval1 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson1ApexLine, 18.0, &state);
    assert_eq!(eval1.medal, AcademyMedal::Gold);
    assert_eq!(eval1.credits_awarded, 2_000);
    assert_eq!(rookie.credits, 2_000);
    assert!(eval1.is_new_best);

    // Replay attempt: achieves Gold again with a faster time (17.5s)
    let eval2 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson1ApexLine, 17.5, &state);
    assert_eq!(eval2.medal, AcademyMedal::Gold);
    assert_eq!(eval2.credits_awarded, 0); // Idempotent! Zero duplicate bounties
    assert_eq!(rookie.credits, 2_000); // Wallet unchanged
    assert!(eval2.is_new_best); // Best time updated
    assert_eq!(
        rookie.academy_progress.lessons.get(&AcademyLessonId::Lesson1ApexLine).unwrap().best_time_sec,
        Some(17.5)
    );
}

#[test]
fn test_academy_clean_attempt_and_disqualification_rules() {
    use tdrace_app::game::academy::{self, AcademyChallengeState};

    let mut rookie = PlayerProfile::new_rookie("Carlos");

    // Case 1: Attempt invalidated by heavy collision
    let mut state_collision = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    state_collision.register_collision(950.0);
    assert!(!state_collision.is_clean());
    let eval_collision = academy::evaluate_academy_attempt(
        &mut rookie,
        AcademyLessonId::Lesson1ApexLine,
        18.0,
        &state_collision,
    );
    assert_eq!(eval_collision.medal, AcademyMedal::None);
    assert_eq!(eval_collision.credits_awarded, 0);
    assert_eq!(rookie.credits, 0);
    assert!(!eval_collision.clean_attempt);
    assert!(eval_collision.failure_reason.is_some());

    // Case 2: Attempt invalidated by exceeding track limits (off-track > 2.0s)
    let mut state_offtrack = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    state_offtrack.update_off_track(2.5);
    assert!(!state_offtrack.is_clean());
    let eval_offtrack = academy::evaluate_academy_attempt(
        &mut rookie,
        AcademyLessonId::Lesson1ApexLine,
        18.0,
        &state_offtrack,
    );
    assert_eq!(eval_offtrack.medal, AcademyMedal::None);
    assert_eq!(eval_offtrack.credits_awarded, 0);
    assert_eq!(rookie.credits, 0);
    assert!(!eval_offtrack.clean_attempt);
}

#[test]
fn test_setup_academy_session_and_execution() {
    use tdrace_app::game::academy::setup_academy_session;
    use tdrace_app::game::RaceSession;

    let mut session = RaceSession::new();

    // Setup Lesson 1
    setup_academy_session(&mut session, AcademyLessonId::Lesson1ApexLine)
        .expect("Setup Lesson 1");
    assert_eq!(session.active_academy_lesson, Some(AcademyLessonId::Lesson1ApexLine));
    assert_eq!(session.track_choice.track_id(), "gt_velocity_park");
    assert_eq!(session.selected_car_model_id, Some("classic_gt"));
    assert_eq!(session.total_laps, 1);
    assert_eq!(session.num_bots, 0);

    // Setup Lesson 4 (Graduation Sprint)
    setup_academy_session(&mut session, AcademyLessonId::Lesson4GraduationSprint)
        .expect("Setup Lesson 4");
    assert_eq!(session.active_academy_lesson, Some(AcademyLessonId::Lesson4GraduationSprint));
    assert_eq!(session.track_choice.track_id(), "ax_meadow_sprint");
    assert_eq!(session.selected_car_model_id, Some("classic_ax_mudlark"));
    assert_eq!(session.total_laps, 2);
    assert_eq!(session.num_bots, 1);
}

#[test]
fn test_academy_pace_status_real_time_splits() {
    use tdrace_app::game::academy::{AcademyChallengeState, AcademyPaceStatus};

    let challenge = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    // Lesson 1: Gold 18.5s, Silver 20.0s, Bronze 22.5s
    // At 50% progress (fraction = 0.5):
    // Gold target = 9.25s, Silver target = 10.0s, Bronze target = 11.25s

    // At 8.5s elapsed -> Ahead of Gold
    match challenge.pace_status(8.5, 0.5) {
        AcademyPaceStatus::AheadOfGold(delta) => assert!(delta < 0.0),
        other => panic!("Expected AheadOfGold, got {:?}", other),
    }

    // At 9.8s elapsed -> Ahead of Silver
    match challenge.pace_status(9.8, 0.5) {
        AcademyPaceStatus::AheadOfSilver(delta) => assert!(delta < 0.0),
        other => panic!("Expected AheadOfSilver, got {:?}", other),
    }

    // At 10.8s elapsed -> Ahead of Bronze
    match challenge.pace_status(10.8, 0.5) {
        AcademyPaceStatus::AheadOfBronze(delta) => assert!(delta < 0.0),
        other => panic!("Expected AheadOfBronze, got {:?}", other),
    }

    // At 12.0s elapsed -> Behind Bronze
    match challenge.pace_status(12.0, 0.5) {
        AcademyPaceStatus::BehindBronze(delta) => assert!(delta > 0.0),
        other => panic!("Expected BehindBronze, got {:?}", other),
    }
}

