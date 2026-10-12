//! # Career Onboarding & Classic Academy Integration Tests (Spec 060)

use tdrace_app::db::HallOfFameDb;
use tdrace_app::profile::{
    AcademyLessonId, AcademyMedal, LicenseGrade, PlayerProfile,
};

#[test]
fn test_fresh_rookie_profile_zero_start_invariants() {
    let rookie = PlayerProfile::new_rookie("Ayrton");

    // Invariant 1: 10,000 Credits starting wallet (Spec 053, 060)
    assert_eq!(rookie.credits, 10_000);
    assert_eq!(rookie.lifetime_credits, 10_000);

    // Invariant 2: 0 Global Driver XP
    assert_eq!(rookie.driver_xp, 0);
    assert_eq!(rookie.lifetime_driver_xp, 0);

    // Invariant 3: Empty owned cars collection (no free starter vehicles)
    assert!(rookie.owned_cars.is_empty());

    // Invariant 4: Ungranted racing license
    assert_eq!(rookie.license_grade, LicenseGrade::None);
    assert!(!rookie.has_racing_license());
    assert!(!rookie.academy_progress.is_graduated());
    assert_eq!(rookie.academy_progress.total_stars(), 0);

    // Invariant 5: Career Mode access is strictly locked
    assert!(!rookie.can_access_career());
}

#[test]
fn test_rookie_profile_database_persistence() {
    let db = HallOfFameDb::open_in_memory().expect("In-memory database should initialize");

    let mut rookie = PlayerProfile::new_rookie("Max");
    rookie.credits = 12_000;
    rookie.lifetime_credits = 12_000;
    rookie.driver_xp = 350;
    rookie.lifetime_driver_xp = 350;
    rookie.license_grade = LicenseGrade::ClassD;
    rookie.owned_cars.push("kart_blackline_cadet_t1".to_string());

    // Record lesson 1 completion
    let (medal, cr, _) = rookie.academy_progress.record_attempt(AcademyLessonId::Lesson1ApexLine, 59.5, true);
    assert_eq!(medal, AcademyMedal::Gold);
    assert_eq!(cr, 2_000);

    let id = db.create_profile(&rookie).expect("Create rookie profile");
    let loaded = db.get_profile_by_id(id).expect("Fetch profile").expect("Profile exists");

    assert_eq!(loaded.name, "Max");
    assert_eq!(loaded.credits, 12_000);
    assert_eq!(loaded.driver_xp, 350);
    assert_eq!(loaded.license_grade, LicenseGrade::ClassD);
    assert_eq!(loaded.owned_cars, vec!["kart_blackline_cadet_t1".to_string()]);
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
    rookie.owned_cars.push("kart_blackline_cadet_t1".to_string());
    assert!(!rookie.has_racing_license());
    assert!(!rookie.can_access_career());

    // 2. Remove car and grant license -> still cannot access career (no car to race with)
    rookie.owned_cars.clear();
    rookie.academy_progress.license_granted = true;
    rookie.license_grade = LicenseGrade::ClassD;
    assert!(rookie.has_racing_license());
    assert!(!rookie.can_access_career());

    // 3. Both license granted AND at least one car owned -> career unlocked!
    rookie.owned_cars.push("kart_blackline_cadet_t1".to_string());
    assert!(rookie.has_racing_license());
    assert!(rookie.can_access_career());
}

#[test]
fn test_academy_curriculum_graduation_grants_license() {
    let mut rookie = PlayerProfile::new_rookie("Lando");
    assert!(!rookie.has_racing_license());

    // Complete lessons 1, 2, 3 with Bronze
    rookie.academy_progress.record_attempt(AcademyLessonId::Lesson1ApexLine, 72.7, true);
    rookie.academy_progress.record_attempt(AcademyLessonId::Lesson2BrakingChicane, 72.6, true);
    rookie.academy_progress.record_attempt(AcademyLessonId::Lesson3SurfaceTransition, 68.8, true);
    assert!(!rookie.has_racing_license());

    // Complete Lesson 4 (Graduation Sprint) with Bronze
    let (medal, cr, _) = rookie.academy_progress.record_attempt(AcademyLessonId::Lesson4GraduationSprint, 125.3, true);
    assert_eq!(medal, AcademyMedal::Bronze);
    assert_eq!(cr, 2_500);

    // Graduation ceremony grants the National Grassroots Racing License
    assert!(rookie.academy_progress.is_graduated());
    assert_eq!(rookie.academy_progress.license_grade, LicenseGrade::ClassD);
}

#[test]
fn test_academy_all_bronze_progression_purse_and_unlocks() {
    use tdrace_app::game::academy::{self, AcademyChallengeState};

    let mut rookie = PlayerProfile::new_rookie("Alain");
    assert_eq!(rookie.credits, 10_000);
    assert_eq!(rookie.driver_xp, 0);

    // Lesson 1: Bronze (target <= 74.2s)
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson1ApexLine));
    assert!(!rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson2BrakingChicane));
    let state1 = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    let eval1 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson1ApexLine, 69.6, &state1);
    assert_eq!(eval1.medal, AcademyMedal::Bronze);
    assert_eq!(eval1.credits_awarded, 1_000);
    assert_eq!(rookie.credits, 11_000);
    assert_eq!(eval1.xp_awarded, 200); // 100 base + 100 distance
    assert_eq!(rookie.driver_xp, 200);
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson2BrakingChicane));

    // Lesson 2: Bronze (target <= 75.2s)
    let state2 = AcademyChallengeState::new(AcademyLessonId::Lesson2BrakingChicane);
    let eval2 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson2BrakingChicane, 71.3, &state2);
    assert_eq!(eval2.medal, AcademyMedal::Bronze);
    assert_eq!(eval2.credits_awarded, 1_500);
    assert_eq!(rookie.credits, 12_500);
    assert_eq!(eval2.xp_awarded, 250); // 150 base + 100 distance
    assert_eq!(rookie.driver_xp, 450);
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson3SurfaceTransition));

    // Lesson 3: Bronze (target <= 73.2s)
    let state3 = AcademyChallengeState::new(AcademyLessonId::Lesson3SurfaceTransition);
    let eval3 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson3SurfaceTransition, 66.9, &state3);
    assert_eq!(eval3.medal, AcademyMedal::Bronze);
    assert_eq!(eval3.credits_awarded, 2_000);
    assert_eq!(rookie.credits, 14_500);
    assert_eq!(eval3.xp_awarded, 300); // 200 base + 100 distance
    assert_eq!(rookie.driver_xp, 750);
    assert!(rookie.academy_progress.is_lesson_unlocked(AcademyLessonId::Lesson4GraduationSprint));

    // Lesson 4: Bronze (target <= 127.2s)
    let state4 = AcademyChallengeState::new(AcademyLessonId::Lesson4GraduationSprint);
    let eval4 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson4GraduationSprint, 123.4, &state4);
    assert_eq!(eval4.medal, AcademyMedal::Bronze);
    assert_eq!(eval4.credits_awarded, 2_500);
    assert_eq!(rookie.credits, 17_000); // 10,000 seed + 7,000 purse!
    assert_eq!(eval4.xp_awarded, 450); // 350 base + 100 distance
    assert_eq!(rookie.driver_xp, 1_200);
    assert!(eval4.newly_graduated);
    assert!(rookie.has_racing_license());
    assert_eq!(rookie.license_grade, LicenseGrade::ClassD);
}

#[test]
fn test_academy_all_silver_and_gold_curriculum_purses() {
    use tdrace_app::game::academy::{self, AcademyChallengeState};

    // Silver Curriculum Run
    let mut rookie_silver = PlayerProfile::new_rookie("Niki");
    let times_silver = [
        (AcademyLessonId::Lesson1ApexLine, 64.7),
        (AcademyLessonId::Lesson2BrakingChicane, 66.1),
        (AcademyLessonId::Lesson3SurfaceTransition, 61.6),
        (AcademyLessonId::Lesson4GraduationSprint, 112.2),
    ];
    for (id, time) in times_silver {
        let state = AcademyChallengeState::new(id);
        let eval = academy::evaluate_academy_attempt(&mut rookie_silver, id, time, &state);
        assert_eq!(eval.medal, AcademyMedal::Silver);
    }
    assert_eq!(
        rookie_silver.credits,
        10_000 + academy::total_curriculum_purse(AcademyMedal::Silver)
    );
    assert_eq!(rookie_silver.credits, 20_750); // 10,000 seed + 10,750 purse!
    assert!(rookie_silver.has_racing_license());
    assert_eq!(rookie_silver.license_grade, LicenseGrade::ClassD);

    // Gold Curriculum Run
    let mut rookie_gold = PlayerProfile::new_rookie("Jackie");
    let times_gold = [
        (AcademyLessonId::Lesson1ApexLine, 59.5),
        (AcademyLessonId::Lesson2BrakingChicane, 60.7),
        (AcademyLessonId::Lesson3SurfaceTransition, 56.4),
        (AcademyLessonId::Lesson4GraduationSprint, 103.3),
    ];
    for (id, time) in times_gold {
        let state = AcademyChallengeState::new(id);
        let eval = academy::evaluate_academy_attempt(&mut rookie_gold, id, time, &state);
        assert_eq!(eval.medal, AcademyMedal::Gold);
    }
    assert_eq!(
        rookie_gold.credits,
        10_000 + academy::total_curriculum_purse(AcademyMedal::Gold)
    );
    assert_eq!(rookie_gold.credits, 24_500); // 10,000 seed + 14,500 purse!
    assert!(rookie_gold.has_racing_license());
    assert_eq!(rookie_gold.license_grade, LicenseGrade::ClassD);
}

#[test]
fn test_academy_attempt_idempotency_on_replay() {
    use tdrace_app::game::academy::{self, AcademyChallengeState};

    let mut rookie = PlayerProfile::new_rookie("Mika");

    // First attempt: Gold (2,000 Cr bounty + 100 distance XP)
    let state = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    let eval1 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson1ApexLine, 59.5, &state);
    assert_eq!(eval1.medal, AcademyMedal::Gold);
    assert_eq!(eval1.credits_awarded, 2_000);
    assert_eq!(rookie.credits, 12_000); // 10,000 seed + 2,000 bounty
    assert!(eval1.is_new_best);

    // Replay attempt: achieves Gold again with a faster time (57.9s)
    // Awards practice stipend (+1,500 Cr) and distance XP (+100 XP), but NO duplicate 2,000 Cr bounty
    let eval2 = academy::evaluate_academy_attempt(&mut rookie, AcademyLessonId::Lesson1ApexLine, 57.9, &state);
    assert_eq!(eval2.medal, AcademyMedal::Gold);
    assert_eq!(eval2.credits_awarded, 1_500); // Practice stipend
    assert_eq!(rookie.credits, 13_500); // 12,000 + 1,500 practice stipend
    assert!(eval2.is_new_best); // Best time updated
    assert_eq!(
        rookie.academy_progress.lessons.get(&AcademyLessonId::Lesson1ApexLine).unwrap().best_time_sec,
        Some(57.9)
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
        59.5,
        &state_collision,
    );
    assert_eq!(eval_collision.medal, AcademyMedal::None);
    assert_eq!(eval_collision.credits_awarded, 0);
    assert_eq!(rookie.credits, 10_000);
    assert!(!eval_collision.clean_attempt);
    assert!(eval_collision.failure_reason.is_some());

    // Case 2: Attempt invalidated by exceeding track limits (off-track > 2.0s)
    let mut state_offtrack = AcademyChallengeState::new(AcademyLessonId::Lesson1ApexLine);
    state_offtrack.update_off_track(2.5);
    assert!(!state_offtrack.is_clean());
    let eval_offtrack = academy::evaluate_academy_attempt(
        &mut rookie,
        AcademyLessonId::Lesson1ApexLine,
        59.5,
        &state_offtrack,
    );
    assert_eq!(eval_offtrack.medal, AcademyMedal::None);
    assert_eq!(eval_offtrack.credits_awarded, 0);
    assert_eq!(rookie.credits, 10_000);
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
    // Lesson 1: Gold 61.2s, Silver 66.5s, Bronze 74.2s
    // At 50% progress (fraction = 0.5):
    // Gold target = 30.6s, Silver target = 33.25s, Bronze target = 37.1s

    // At 28.0s elapsed -> Ahead of Gold
    match challenge.pace_status(28.0, 0.5) {
        AcademyPaceStatus::AheadOfGold(delta) => assert!(delta < 0.0),
        other => panic!("Expected AheadOfGold, got {:?}", other),
    }

    // At 32.0s elapsed -> Ahead of Silver
    match challenge.pace_status(32.0, 0.5) {
        AcademyPaceStatus::AheadOfSilver(delta) => assert!(delta < 0.0),
        other => panic!("Expected AheadOfSilver, got {:?}", other),
    }

    // At 36.0s elapsed -> Ahead of Bronze
    match challenge.pace_status(36.0, 0.5) {
        AcademyPaceStatus::AheadOfBronze(delta) => assert!(delta < 0.0),
        other => panic!("Expected AheadOfBronze, got {:?}", other),
    }

    // At 40.0s elapsed -> Behind Bronze
    match challenge.pace_status(40.0, 0.5) {
        AcademyPaceStatus::BehindBronze(delta) => assert!(delta > 0.0),
        other => panic!("Expected BehindBronze, got {:?}", other),
    }
}

#[test]
fn test_classic_academy_selectable_in_modality_menu() {
    use tdrace_app::ui::menu::{ModalityCategory, ModalityItem};

    let items = ModalityCategory::SinglePlayer.items();
    assert_eq!(items[2], ModalityItem::CareerMode);
    assert_eq!(ModalityItem::ClassicAcademy.title(), "Classic Academy");
    assert_eq!(ModalityItem::ClassicAcademy.tag(), "RACING LICENSE & SEED CASH");
}

#[test]
fn test_career_mode_locked_modal_when_unlicensed() {
    use tdrace_app::game::{GameState, RaceSession};
    use tdrace_app::ui::menu::{ModalityCategory, ModalityModal};

    let mut session = RaceSession::new();
    // Fresh profile: no license
    assert!(!session.active_profile.has_racing_license());

    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 2, // CareerMode index
        modal: None,
    };

    // Confirm selection -> should trigger LicenseRequired modal
    tdrace_app::game::inject_key_presses_for_tests(&[macroquad::input::KeyCode::Enter]);
    session.update();
    tdrace_app::game::inject_key_presses_for_tests(&[]);

    match &session.state {
        GameState::ModalitySelect { modal, .. } => {
            assert_eq!(*modal, Some(ModalityModal::LicenseRequired));
            assert_eq!(modal.as_ref().unwrap().title(), "RACING LICENSE REQUIRED");
        }
        other => panic!("Expected ModalitySelect with LicenseRequired modal, got {:?}", other),
    }

    // Confirming LicenseRequired modal enrolls player in Classic Academy
    tdrace_app::game::inject_key_presses_for_tests(&[macroquad::input::KeyCode::Enter]);
    session.update();
    tdrace_app::game::inject_key_presses_for_tests(&[]);

    match session.state {
        GameState::ClassicAcademy { selected_idx, showing_graduation } => {
            assert_eq!(selected_idx, 0);
            assert!(!showing_graduation);
        }
        other => panic!("Expected ClassicAcademy state, got {:?}", other),
    }
}

#[test]
fn test_graduation_ceremony_ui_and_showroom_navigation() {
    use tdrace_app::game::{GarageOrigin, GameState, RaceSession};
    use tdrace_app::ui::{academy_card_rect, graduation_showroom_button_rect};

    let (btn_x, btn_y, btn_w, btn_h) = graduation_showroom_button_rect(1920.0, 1080.0);
    assert!(btn_w > 0.0);
    assert!(btn_h > 0.0);
    assert!(btn_x > 0.0);
    assert!(btn_y > 0.0);

    // Verify academy cards layout
    let (c0_x, c0_y, c0_w, c0_h) = academy_card_rect(1920.0, 1080.0, 0);
    let (_c1_x, c1_y, _c1_w, _c1_h) = academy_card_rect(1920.0, 1080.0, 1);
    assert!(c0_w > 0.0 && c0_h > 0.0);
    assert!(c1_y > c0_y, "Card 1 must be positioned below Card 0");
    assert!(c0_x > 0.0);

    let mut session = RaceSession::new();
    session.state = GameState::ClassicAcademy {
        selected_idx: 3,
        showing_graduation: true,
    };

    // Press Enter on graduation modal -> transitions directly to Showroom (Garage)
    tdrace_app::game::inject_key_presses_for_tests(&[macroquad::input::KeyCode::Enter]);
    session.update();
    tdrace_app::game::inject_key_presses_for_tests(&[]);

    assert_eq!(session.state, GameState::Garage(GarageOrigin::ModalitySelect));
    assert_eq!(session.active_module_id, "kart");
    assert_eq!(session.garage_tier, 1);
}

#[test]
fn test_grassroots_pricing_calibration_and_affordability() {
    use tdrace_app::profile::ModuleCareerProgress;

    // 1. Verify pricing matches Grassroots Pricing Reference Table (Spec 060 Section 2.1 & 2.2)
    assert_eq!(ModuleCareerProgress::car_credit_cost_for_module("kart", 1), 5_000);
    assert_eq!(ModuleCareerProgress::car_credit_cost_for_module("autocross", 1), 12_000);
    assert_eq!(ModuleCareerProgress::car_credit_cost_for_module("rally", 1), 16_000);
    assert_eq!(ModuleCareerProgress::car_credit_cost_for_module("gt", 1), 70_000);

    // Specific catalog vehicle cost resolutions
    assert_eq!(
        ModuleCareerProgress::car_credit_cost_for_car("kart_blackline_cadet_t1", "kart", 1),
        5_000
    );
    assert_eq!(
        ModuleCareerProgress::car_credit_cost_for_car("autocross_lifelive_tn5", "autocross", 1),
        12_000
    );
    assert_eq!(
        ModuleCareerProgress::car_credit_cost_for_car("rally_gallia_200_t1", "rally", 1),
        16_000
    );
    assert_eq!(
        ModuleCareerProgress::car_credit_cost_for_car("gt_vandorn_stratus_t1", "gt", 1),
        70_000
    );

    // 2. Affordability with all-Bronze purse (7,000 Cr)
    let bronze_purse = 7_000u64;
    let kart_progress = ModuleCareerProgress::default_for_module(1, "kart");
    let ax_progress = ModuleCareerProgress::default_for_module(1, "autocross");
    let rally_progress = ModuleCareerProgress::default_for_module(1, "rally");
    let gt_progress = ModuleCareerProgress::default_for_module(1, "gt");

    // Cadet Kart is affordable immediately
    assert!(kart_progress.can_buy_car("kart_blackline_cadet_t1", 1, bronze_purse));
    // Cross Car, Rally4, and GT4 are unaffordable with Bronze purse
    assert!(!ax_progress.can_buy_car("autocross_lifelive_tn5", 1, bronze_purse));
    assert!(!rally_progress.can_buy_car("rally_gallia_200_t1", 1, bronze_purse));
    assert!(!gt_progress.can_buy_car("gt_vandorn_stratus_t1", 1, bronze_purse));

    // 3. Affordability with all-Gold purse (14,500 Cr)
    let gold_purse = 14_500u64;
    // Cadet Kart and Cross Car Junior are affordable
    assert!(kart_progress.can_buy_car("kart_blackline_cadet_t1", 1, gold_purse));
    assert!(ax_progress.can_buy_car("autocross_lifelive_tn5", 1, gold_purse));
    // Rally4 (16,000 Cr) and GT4 (70,000 Cr) are still unaffordable
    assert!(!rally_progress.can_buy_car("rally_gallia_200_t1", 1, gold_purse));
    assert!(!gt_progress.can_buy_car("gt_vandorn_stratus_t1", 1, gold_purse));
}

#[test]
fn test_starter_car_purchase_adds_to_owned_cars_and_unlocks_career() {
    use tdrace_app::game::{GameState, RaceSession};
    use tdrace_app::profile::ModuleCareerProgress;
    use tdrace_app::ui::menu::{ModalityCategory, ModalityModal};

    let mut session = RaceSession::new();
    let mut rookie = PlayerProfile::new_rookie("Sebastian");

    // 1. Graduate Academy (grants license and holds 10,000 Cr starting wallet)
    rookie.academy_progress.license_granted = true;
    rookie.license_grade = LicenseGrade::ClassD;
    assert_eq!(rookie.credits, 10_000);
    assert!(rookie.has_racing_license());
    assert!(rookie.owned_cars.is_empty());
    assert!(!rookie.can_access_career(), "Licensed rookie without a car cannot access career");

    session.active_profile = rookie;

    // 2. Modality Select: CareerMode should trigger VehicleRequired modal
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 2, // CareerMode
        modal: None,
    };
    tdrace_app::game::inject_key_presses_for_tests(&[macroquad::input::KeyCode::Enter]);
    session.update();
    tdrace_app::game::inject_key_presses_for_tests(&[]);

    match &session.state {
        GameState::ModalitySelect { modal, .. } => {
            assert_eq!(*modal, Some(ModalityModal::VehicleRequired));
            assert_eq!(modal.as_ref().unwrap().title(), "STARTER VEHICLE REQUIRED");
        }
        other => panic!("Expected ModalitySelect with VehicleRequired modal, got {:?}", other),
    }

    // 3. Purchase Cadet Kart in Kart module (5,000 Cr)
    let mut kart_progress = ModuleCareerProgress::default_for_module(1, "kart");
    assert!(kart_progress.can_buy_car("kart_blackline_cadet_t1", 1, session.active_profile.credits));
    kart_progress
        .buy_car(&mut session.active_profile, "kart_blackline_cadet_t1", 1)
        .expect("Purchase Cadet Kart");

    // Verify credits deducted: 10,000 - 5,000 = 5,000 Cr reserve preserved!
    assert_eq!(session.active_profile.credits, 5_000);
    // Verify car added to owned_cars
    assert_eq!(session.active_profile.owned_cars, vec!["kart_blackline_cadet_t1".to_string()]);
    // Career mode is now fully unlocked!
    assert!(session.active_profile.can_access_career());

    // 4. Returning to ModalitySelect and selecting CareerMode enters CareerSelect without modal
    session.state = GameState::ModalitySelect {
        category: ModalityCategory::SinglePlayer,
        selected_idx: 2,
        modal: None,
    };
    tdrace_app::game::inject_key_presses_for_tests(&[macroquad::input::KeyCode::Enter]);
    session.update();
    tdrace_app::game::inject_key_presses_for_tests(&[]);

    match session.state {
        GameState::CareerSelect { .. } => {
            // Career mode successfully accessed!
        }
        other => panic!("Expected CareerSelect state after unlocking, got {:?}", other),
    }
}

#[test]
fn test_grassroots_starter_models_in_catalog() {
    use tdrace_app::catalog::get_models_for_module;

    let starters = get_models_for_module("starter");
    assert!(!starters.is_empty());
    for model in &starters {
        assert_eq!(model.tier, 1);
        assert!(
            model.module_id == "kart" || model.module_id == "autocross" || model.module_id == "rally" || model.module_id == "rallycross",
            "Unexpected starter module: {}",
            model.module_id
        );
    }

    // Verify Cadet Kart, Cross Car Junior, and Rally4 are included
    assert!(starters.iter().any(|m| m.module_id == "kart"));
    assert!(starters.iter().any(|m| m.module_id == "autocross"));
    assert!(starters.iter().any(|m| m.module_id == "rally" || m.module_id == "rallycross"));
}

#[test]
fn test_feeder_ladder_and_discipline_license_gating() {
    let mut profile = PlayerProfile::new_rookie("Ayrton");
    assert_eq!(profile.license_grade, LicenseGrade::None);

    // Unlicensed driver cannot enter any discipline
    assert!(!profile.can_enter_category("kart"));
    assert!(!profile.can_enter_category("autocross"));
    assert!(!profile.can_enter_category("rally"));
    assert!(!profile.can_enter_category("nascar"));
    assert!(!profile.can_enter_category("gt"));

    // Promoting to Class D (Academy Graduation) unlocks Karting
    profile.license_grade = LicenseGrade::ClassD;
    assert!(profile.can_enter_category("kart"));
    assert!(!profile.can_enter_category("autocross"));
    assert!(!profile.can_enter_category("rally"));
    assert!(!profile.can_enter_category("nascar"));
    assert!(!profile.can_enter_category("gt"));

    // Promoting to Class C (Karting season podium) unlocks Autocross & Rallycross
    profile.license_grade = LicenseGrade::ClassC;
    assert!(profile.can_enter_category("kart"));
    assert!(profile.can_enter_category("autocross"));
    assert!(profile.can_enter_category("rally"));
    assert!(!profile.can_enter_category("nascar"));
    assert!(!profile.can_enter_category("gt"));

    // Promoting to Class B (Dirt season podium) unlocks Stock Car & Extreme Off-Road
    profile.license_grade = LicenseGrade::ClassB;
    assert!(profile.can_enter_category("kart"));
    assert!(profile.can_enter_category("autocross"));
    assert!(profile.can_enter_category("rally"));
    assert!(profile.can_enter_category("nascar"));
    assert!(profile.can_enter_category("extreme_offroad"));
    assert!(!profile.can_enter_category("gt"));

    // Promoting to Class A (Stock/Dirt season podium) unlocks GT Racing
    profile.license_grade = LicenseGrade::ClassA;
    assert!(profile.can_enter_category("gt"));
    assert!(profile.can_enter_category("gt_challenge"));
}



// ---------------------------------------------------------------------------------------------
// Spec 064: Academy Tiers 2 and 3
// ---------------------------------------------------------------------------------------------

mod multitier {
    use tdrace_app::game::academy::{
        self, evaluate_academy_attempt_on, setup_academy_session, AcademyChallengeState, AcademyLessonDef,
        AcademyObjective,
    };
    use tdrace_app::game::RaceSession;
    use tdrace_app::profile::{AcademyLessonId, AcademyMedal, LicenseGrade, ModuleCareerProgress, PlayerProfile};
    use tdrace_core::physics::car::CarControls;
    use tdrace_core::physics::surface::SurfaceType;

    /// A clean challenge state that earns `medal` on `id`, whatever its objective.
    fn challenge_for(id: AcademyLessonId, medal: AcademyMedal) -> (f32, AcademyChallengeState) {
        let def = AcademyLessonDef::find(id).unwrap();
        let target = match medal {
            AcademyMedal::Gold => def.gold_time_sec,
            AcademyMedal::Silver => def.silver_time_sec,
            _ => def.bronze_time_sec,
        };
        let mut state = AcademyChallengeState::new(id);
        match def.objective {
            AcademyObjective::DriftScore => state.observe(0.1, target),
            AcademyObjective::Podium { .. } => {
                state.observe(0.1, 0.0);
                state.finish_position = match medal {
                    AcademyMedal::Gold => 1,
                    AcademyMedal::Silver => 2,
                    _ => 3,
                };
            }
            _ => state.observe(0.1, 0.0),
        }
        (target, state)
    }

    fn evaluate(profile: &mut PlayerProfile, id: AcademyLessonId, medal: AcademyMedal, day: &str) -> academy::AcademyAttemptEvaluation {
        let (time, state) = challenge_for(id, medal);
        evaluate_academy_attempt_on(profile, id, time, &state, day)
    }

    fn graduate(profile: &mut PlayerProfile) {
        for id in AcademyLessonId::ALL {
            evaluate(profile, id, AcademyMedal::Bronze, "2026-10-11");
        }
        assert!(profile.academy_progress.is_graduated());
    }

    fn session_with_db() -> RaceSession {
        let mut session = RaceSession::new();
        session.hof_db = Some(tdrace_app::db::HallOfFameDb::open_in_memory().unwrap());
        session
    }

    #[test]
    fn test_setup_sessions_for_tier_2_and_3_missions() {
        let mut session = RaceSession::new();
        let expect = [
            (AcademyLessonId::Acad2DirtDrift, "ax_clay_bowl", "classic_ax_brawler", 1, 0),
            (AcademyLessonId::Acad2ScandinavianFlick, "rx_hilltop_leap", "classic_rally", 1, 0),
            (AcademyLessonId::Acad2RampBalance, "at_dune_sea", "classic_offroad", 1, 0),
            (AcademyLessonId::Acad2MixedQualifier, "rx_canyon_flyer", "classic_rally", 2, 0),
            (AcademyLessonId::Acad3ThermalThrottle, "gt_ridge_ring", "classic_gt", 3, 0),
            (AcademyLessonId::Acad3OvalStint, "stock_tri_oval_speedway", "classic_nascar", 5, 0),
            (AcademyLessonId::Acad3WornRubber, "gt_coastal_grand_prix", "classic_gt", 2, 0),
            (AcademyLessonId::Acad3TyreConservation, "stock_roval", "classic_nascar", 4, 3),
        ];
        for (id, track, car, laps, bots) in expect {
            setup_academy_session(&mut session, id).unwrap_or_else(|e| panic!("{}: {e}", id.slug()));
            assert_eq!(session.active_academy_lesson, Some(id));
            assert_eq!(session.track_choice.track_id(), track, "{}", id.slug());
            assert_eq!(session.selected_car_model_id, Some(car), "{}", id.slug());
            assert_eq!(session.total_laps, laps, "{}", id.slug());
            assert_eq!(session.num_bots, bots, "{}", id.slug());
        }
    }

    #[test]
    fn test_tier_2_mission_runs_with_damage_and_wear_off_whatever_the_player_setting() {
        let mut session = session_with_db();
        session.config.gameplay.car_damage = true;
        setup_academy_session(&mut session, AcademyLessonId::Acad2DirtDrift).unwrap();
        session.init_race();

        assert!(session.config.gameplay.car_damage, "the player's own setting is never rewritten");
        assert!(!session.world.rules.damage_enabled);
        for car in &session.world.vehicles {
            assert!(!car.config.damage_enabled);
            assert_eq!(car.config.wear_rate_scale, 0.0);
        }

        // A hard hit leaves health at 100% and no repair bill
        let car = &mut session.world.vehicles[0];
        car.apply_collision_damage_to_zone(tdrace_core::physics::car::ImpactZone::FrontNose, 20_000.0);
        assert_eq!(car.state.health, 1.0);

        // After a sustained drift all four wheels still show wear 0.0
        car.state.velocity = glam::Vec2::new(22.0, 0.0);
        let controls = CarControls { throttle: 1.0, steer: 0.8, handbrake: true, ..Default::default() };
        for _ in 0..600 {
            car.step(&controls, SurfaceType::Dirt, 1.0 / 60.0);
        }
        assert!(car.state.wheel_assemblies.iter().all(|w| w.wear == 0.0));
    }

    #[test]
    fn test_tier_3_mission_scales_wear_and_worn_rubber_starts_at_65_percent() {
        let mut session = session_with_db();
        setup_academy_session(&mut session, AcademyLessonId::Acad3WornRubber).unwrap();
        session.init_race();
        assert!(!session.world.rules.damage_enabled);
        let player = &session.world.vehicles[0];
        assert_eq!(player.config.wear_rate_scale, 3.0);
        assert!(player.state.wheel_assemblies.iter().all(|w| w.wear == 0.65));

        // The 3-lap pace run starts on fresh tyres
        setup_academy_session(&mut session, AcademyLessonId::Acad3ThermalThrottle).unwrap();
        session.init_race();
        assert_eq!(session.world.vehicles[0].config.wear_rate_scale, 3.0);
        assert!(session.world.vehicles[0].state.wheel_assemblies.iter().all(|w| w.wear == 0.0));
    }

    #[test]
    fn test_academy_rules_end_with_the_session() {
        let mut session = session_with_db();
        session.config.gameplay.car_damage = true;
        setup_academy_session(&mut session, AcademyLessonId::Acad3WornRubber).unwrap();
        session.init_race();
        assert!(!session.world.rules.damage_enabled);

        // Leaving the Academy, then the next race outside it
        let _ = session.race_exit_target();
        assert_eq!(session.active_academy_lesson, None);
        session.init_race();
        assert!(session.config.gameplay.car_damage);
        assert!(session.world.rules.damage_enabled, "the player's damage setting is back");
        for car in &session.world.vehicles {
            assert!(car.config.damage_enabled);
            assert_eq!(car.config.wear_rate_scale, 1.0);
            assert!(car.state.wheel_assemblies.iter().all(|w| w.wear == 0.0));
        }
    }

    #[test]
    fn test_mixed_surface_qualifier_has_the_mandatory_joker_rule() {
        use tdrace_app::game::joker_rule_for;
        let mut session = RaceSession::new();
        setup_academy_session(&mut session, AcademyLessonId::Acad2MixedQualifier).unwrap();
        assert!(AcademyLessonDef::find(AcademyLessonId::Acad2MixedQualifier).unwrap().rules.joker_required);
        assert_eq!(joker_rule_for(&session.track).mandatory, 1, "rx_canyon_flyer must carry the joker rule");
    }

    #[test]
    fn test_unlock_and_license_flow_through_evaluation() {
        let mut profile = PlayerProfile::new_rookie("Kimi");
        assert!(!profile.academy_progress.is_lesson_unlocked(AcademyLessonId::Acad2DirtDrift));
        graduate(&mut profile);
        assert_eq!(profile.license_grade, LicenseGrade::ClassD);
        assert!(profile.academy_progress.is_lesson_unlocked(AcademyLessonId::Acad2DirtDrift));

        // Bronze on 2 of 4 Tier 2 missions: Tier 3 stays locked, a third opens it and grants Class C
        evaluate(&mut profile, AcademyLessonId::Acad2DirtDrift, AcademyMedal::Bronze, "2026-10-11");
        evaluate(&mut profile, AcademyLessonId::Acad2ScandinavianFlick, AcademyMedal::Bronze, "2026-10-11");
        assert!(!profile.academy_progress.is_tier_unlocked(3));
        assert_eq!(profile.license_grade, LicenseGrade::ClassD);
        let third = evaluate(&mut profile, AcademyLessonId::Acad2RampBalance, AcademyMedal::Bronze, "2026-10-11");
        assert!(profile.academy_progress.is_tier_unlocked(3));
        assert_eq!(profile.license_grade, LicenseGrade::ClassC);
        assert_eq!(third.license_earned, Some(LicenseGrade::ClassC));

        // Tier 3 pass grants Class B
        for id in &AcademyLessonId::tier_lessons(3)[..2] {
            evaluate(&mut profile, *id, AcademyMedal::Bronze, "2026-10-11");
        }
        assert_eq!(profile.license_grade, LicenseGrade::ClassC);
        let pass = evaluate(&mut profile, AcademyLessonId::Acad3WornRubber, AcademyMedal::Bronze, "2026-10-11");
        assert_eq!(profile.license_grade, LicenseGrade::ClassB);
        assert_eq!(pass.license_earned, Some(LicenseGrade::ClassB));

        // A Class A holder keeps it
        let mut ace = PlayerProfile::new_rookie("Lewis");
        graduate(&mut ace);
        ace.license_grade = LicenseGrade::ClassA;
        for id in &AcademyLessonId::tier_lessons(2)[..3] {
            evaluate(&mut ace, *id, AcademyMedal::Bronze, "2026-10-11");
        }
        assert_eq!(ace.license_grade, LicenseGrade::ClassA);
    }

    #[test]
    fn test_medal_bounties_pay_as_deltas_and_the_rerun_pays_the_stipend_only() {
        let mut profile = PlayerProfile::new_rookie("Nico");
        graduate(&mut profile);
        profile.credits = 5_000;
        let id = AcademyLessonId::Acad2DirtDrift;

        evaluate(&mut profile, id, AcademyMedal::Silver, "2026-10-11");
        assert_eq!(profile.credits, 5_000 + 3_750);
        evaluate(&mut profile, id, AcademyMedal::Gold, "2026-10-11");
        assert_eq!(profile.credits, 5_000 + 5_000);
        let rerun = evaluate(&mut profile, id, AcademyMedal::Gold, "2026-10-11");
        assert_eq!(profile.credits, 5_000 + 5_000 + 2_000, "the second Gold pays the stipend only");
        assert_eq!(rerun.credits_awarded, 2_000);
    }

    #[test]
    fn test_stipends_are_capped_at_five_per_local_day() {
        let mut profile = PlayerProfile::new_rookie("Oscar");
        graduate(&mut profile);
        for id in AcademyLessonId::tier_lessons(2) {
            evaluate(&mut profile, *id, AcademyMedal::Bronze, "2026-10-11");
        }
        // The first clears paid bounties, no stipend. Bronze again on Tier 2 pays 2,000 each, 5 times a day.
        let id = AcademyLessonId::Acad2DirtDrift;
        for run in 1..=5 {
            let before = profile.credits;
            let eval = evaluate(&mut profile, id, AcademyMedal::Bronze, "2026-10-12");
            assert_eq!(profile.credits - before, 2_000, "run {run}");
            assert!(!eval.stipend_limit_reached);
        }
        let before = profile.credits;
        let sixth = evaluate(&mut profile, id, AcademyMedal::Bronze, "2026-10-12");
        assert_eq!(profile.credits, before, "the 6th run pays nothing");
        assert!(sixth.stipend_limit_reached);
        // Next local day pays again
        let before = profile.credits;
        evaluate(&mut profile, id, AcademyMedal::Bronze, "2026-10-13");
        assert_eq!(profile.credits - before, 2_000);
    }

    #[test]
    fn test_stipend_amount_follows_the_tier() {
        let mut profile = PlayerProfile::new_rookie("Pierre");
        graduate(&mut profile);
        let before = profile.credits;
        // Tier 1 lesson 1 re-run: the Spec 060 stipend of 1,500 Cr
        evaluate(&mut profile, AcademyLessonId::Lesson1ApexLine, AcademyMedal::Bronze, "2026-10-11");
        assert_eq!(profile.credits - before, 1_500);
        for id in &AcademyLessonId::tier_lessons(2)[..3] {
            evaluate(&mut profile, *id, AcademyMedal::Bronze, "2026-10-11");
        }
        let id = AcademyLessonId::Acad3OvalStint;
        evaluate(&mut profile, id, AcademyMedal::Bronze, "2026-10-11");
        let before = profile.credits;
        evaluate(&mut profile, id, AcademyMedal::Bronze, "2026-10-12");
        assert_eq!(profile.credits - before, 3_000);
    }

    #[test]
    fn test_worn_out_tyres_give_no_medal_but_count_the_attempt() {
        let mut profile = PlayerProfile::new_rookie("Charles");
        graduate(&mut profile);
        let id = AcademyLessonId::Acad3ThermalThrottle;
        let def = AcademyLessonDef::find(id).unwrap();
        let mut state = AcademyChallengeState::new(id);
        state.observe(0.55, 0.0);
        let before = profile.credits;
        let eval = evaluate_academy_attempt_on(&mut profile, id, def.gold_time_sec - 1.0, &state, "2026-10-11");
        assert_eq!(eval.medal, AcademyMedal::None);
        assert!(eval.tyres_worn_out);
        assert!(eval.clean_attempt);
        assert_eq!(profile.credits, before);
        assert_eq!(profile.academy_progress.lessons[&id].attempts, 1);

        // Under the cap the same time earns Gold
        let mut ok = AcademyChallengeState::new(id);
        ok.observe(0.39, 0.0);
        let eval = evaluate_academy_attempt_on(&mut profile, id, def.gold_time_sec - 1.0, &ok, "2026-10-11");
        assert_eq!(eval.medal, AcademyMedal::Gold);
        assert!(!eval.tyres_worn_out);
    }

    #[test]
    fn test_rough_landing_invalidates_only_the_clean_landing_mission() {
        let id = AcademyLessonId::Acad2RampBalance;
        let mut state = AcademyChallengeState::new(id);
        state.register_landing(AcademyChallengeState::MAX_PERMISSIBLE_LANDING_IMPULSE * 0.9);
        assert!(state.is_clean());
        state.register_landing(AcademyChallengeState::MAX_PERMISSIBLE_LANDING_IMPULSE * 1.2);
        assert!(!state.is_clean());
        assert!(state.disqualification_reason.as_deref().unwrap().contains("Rough Landing"));

        let mut profile = PlayerProfile::new_rookie("Sergio");
        graduate(&mut profile);
        let def = AcademyLessonDef::find(id).unwrap();
        let eval = evaluate_academy_attempt_on(&mut profile, id, def.gold_time_sec - 1.0, &state, "2026-10-11");
        assert_eq!(eval.medal, AcademyMedal::None);
        assert!(eval.failure_reason.unwrap().contains("Rough Landing"));

        // A rally lesson does not care about a hard landing
        let mut other = AcademyChallengeState::new(AcademyLessonId::Acad2ScandinavianFlick);
        other.register_landing(5_000.0);
        assert!(other.is_clean());
    }

    #[test]
    fn test_tyre_conservation_sprint_is_scored_on_position_not_on_a_clean_lap() {
        let mut profile = PlayerProfile::new_rookie("Alex");
        graduate(&mut profile);
        let id = AcademyLessonId::Acad3TyreConservation;
        let mut state = AcademyChallengeState::new(id);
        state.register_collision(1_500.0); // heavy contact in a 4-car race
        state.observe(0.30, 0.0);
        state.finish_position = 2;
        let eval = evaluate_academy_attempt_on(&mut profile, id, 120.0, &state, "2026-10-11");
        assert_eq!(eval.medal, AcademyMedal::Silver);
        assert!(eval.clean_attempt);
    }

    #[test]
    fn test_pace_status_of_old_lessons_is_unchanged_and_tier_ids_resolve() {
        for tier in 1..=3u8 {
            for id in AcademyLessonId::tier_lessons(tier) {
                assert!(AcademyLessonDef::find(*id).is_some(), "{}", id.slug());
            }
        }
    }

    /// Spec 064 open balance question: the Tier 2 (25,000 Cr) and Tier 3 (50,000 Cr) purses against the
    /// car prices of the career they feed. All-Bronze buys the entry car of the class the licence opens,
    /// all-Gold reaches the next step but never skips a tier of cars.
    #[test]
    fn test_tier_purses_fit_the_car_prices_of_the_class_they_open() {
        let price = ModuleCareerProgress::car_credit_cost_for_module;
        let purse = |tier: u8, share: u64| -> u64 {
            AcademyLessonDef::tier_curriculum(tier)
                .iter()
                .map(|l| match share {
                    1 => l.bronze_credit_bounty,
                    2 => l.bronze_credit_bounty + l.silver_credit_bounty,
                    _ => l.bronze_credit_bounty + l.silver_credit_bounty + l.gold_credit_bounty,
                })
                .sum()
        };
        // Tier 2 opens Class C: Autocross and Rallycross
        assert!(purse(2, 1) >= price("autocross", 1), "all-Bronze Tier 2 buys the Cross Car Junior");
        assert!(purse(2, 3) >= price("autocross", 2), "all-Gold Tier 2 reaches the Cross Car Senior");
        assert!(purse(2, 3) < price("autocross", 3), "but not the Buggy 1600");
        assert!(purse(2, 3) < price("rally", 2), "and not the RX Supercar Lites");
        // Tier 3 opens Class B: Stock Car and Extreme Off-Road
        assert!(purse(3, 1) >= price("nascar", 1), "all-Bronze Tier 3 buys the Street Stock");
        assert!(purse(3, 3) >= price("nascar", 2), "all-Gold Tier 3 reaches the Late Model Stock");
        assert!(purse(3, 3) < price("nascar", 3), "but not the ARCA car");
        assert!(purse(3, 3) < price("extreme_offroad", 2), "and not the Trophy Truck");
    }

    #[test]
    fn test_academy_progress_with_stipend_fields_survives_the_database() {
        let db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
        let mut profile = PlayerProfile::new_rookie("Valtteri");
        graduate(&mut profile);
        evaluate(&mut profile, AcademyLessonId::Acad2DirtDrift, AcademyMedal::Gold, "2026-10-11");
        evaluate(&mut profile, AcademyLessonId::Acad2DirtDrift, AcademyMedal::Gold, "2026-10-11");
        let id = db.create_profile(&profile).unwrap();
        let loaded = db.get_profile_by_id(id).unwrap().unwrap();
        assert_eq!(loaded.academy_progress.stipend_day.as_deref(), Some("2026-10-11"));
        assert_eq!(loaded.academy_progress.stipends_today, 1);
        assert_eq!(loaded.academy_progress, profile.academy_progress);
    }
}

mod multitier_ui {
    use tdrace_app::game::academy::{evaluate_academy_attempt_on, AcademyChallengeState, AcademyLessonDef};
    use tdrace_app::game::{GameState, RaceSession};
    use tdrace_app::profile::{AcademyLessonId, AcademyMedal, PlayerProfile};
    use tdrace_app::ui::academy_ui::{academy_result_summary, lesson_targets_text, tier_unlock_rule};
    use tdrace_app::ui::hud::academy_wear_label;

    /// Presses `key` on the Academy screen with the cursor on `selected_idx`. Returns the new cursor.
    fn press_on_academy(session: &mut RaceSession, selected_idx: usize, key: macroquad::input::KeyCode) -> usize {
        session.state = GameState::ClassicAcademy { selected_idx, showing_graduation: false };
        tdrace_app::game::inject_key_presses_for_tests(&[key]);
        session.update();
        tdrace_app::game::inject_key_presses_for_tests(&[]);
        match session.state {
            GameState::ClassicAcademy { selected_idx, .. } => selected_idx,
            ref other => panic!("left the Academy screen: {other:?}"),
        }
    }

    fn graduate(profile: &mut PlayerProfile) {
        for id in AcademyLessonId::ALL {
            let def = AcademyLessonDef::find(id).unwrap();
            profile.academy_progress.record_attempt(id, def.bronze_time_sec - 0.1, true);
        }
    }

    #[test]
    fn test_academy_cursor_moves_inside_a_tier_and_across_tier_tabs() {
        use macroquad::input::KeyCode;
        let mut session = RaceSession::new();
        // Up and Down stay inside the four missions of the tier
        assert_eq!(press_on_academy(&mut session, 0, KeyCode::Down), 1);
        assert_eq!(press_on_academy(&mut session, 3, KeyCode::Down), 0);
        assert_eq!(press_on_academy(&mut session, 4, KeyCode::Up), 7);
        assert_eq!(press_on_academy(&mut session, 11, KeyCode::Down), 8);
        // Right and Left change the tier tab and keep the row
        assert_eq!(press_on_academy(&mut session, 1, KeyCode::Right), 5);
        assert_eq!(press_on_academy(&mut session, 5, KeyCode::Right), 9);
        assert_eq!(press_on_academy(&mut session, 9, KeyCode::Right), 1);
        assert_eq!(press_on_academy(&mut session, 1, KeyCode::Left), 9);
        assert_eq!(press_on_academy(&mut session, 6, KeyCode::E), 10);
        assert_eq!(press_on_academy(&mut session, 6, KeyCode::Q), 2);
    }

    #[test]
    fn test_a_locked_tier_cannot_be_started_and_an_open_one_can() {
        use macroquad::input::KeyCode;
        let mut session = RaceSession::new();
        // Rookie: Tier 2 and Tier 3 missions are locked
        for idx in [4, 8] {
            let _ = press_on_academy(&mut session, idx, KeyCode::Enter);
            assert_eq!(session.active_academy_lesson, None, "mission {idx} must not start");
        }
        // Graduated: a Tier 2 mission starts
        graduate(&mut session.active_profile);
        let _ = press_on_academy(&mut session, 5, KeyCode::Enter);
        assert_eq!(session.active_academy_lesson, Some(AcademyLessonId::Acad2ScandinavianFlick));
        // Tier 3 is still locked
        session.active_academy_lesson = None;
        let _ = press_on_academy(&mut session, 9, KeyCode::Enter);
        assert_eq!(session.active_academy_lesson, None);
    }

    #[test]
    fn test_wear_banner_reads_worst_wheel_against_the_cap() {
        assert_eq!(academy_wear_label(0.31, 0.40), "WEAR 31% / 40%");
        assert_eq!(academy_wear_label(0.0, 0.40), "WEAR 0% / 40%");
    }

    #[test]
    fn test_locked_tiers_show_their_unlock_rule() {
        let mut profile = PlayerProfile::new_rookie("Mick");
        assert!(tier_unlock_rule(2, &profile).contains("Tier 1"));
        assert!(tier_unlock_rule(3, &profile).contains("0 / 4"));
        profile.academy_progress.license_granted = true;
        let def = AcademyLessonDef::find(AcademyLessonId::Acad2ScandinavianFlick).unwrap();
        profile.academy_progress.record_attempt(def.id, def.bronze_time_sec - 0.1, true);
        assert!(tier_unlock_rule(3, &profile).contains("1 / 4"));
        assert!(tier_unlock_rule(1, &profile).is_empty());
    }

    #[test]
    fn test_target_lines_use_the_unit_of_each_objective() {
        let text = |id| lesson_targets_text(&AcademyLessonDef::find(id).unwrap());
        assert!(text(AcademyLessonId::Acad2DirtDrift).contains("drift pts"));
        assert!(text(AcademyLessonId::Acad3ThermalThrottle).contains("WEAR 40% MAX"));
        assert!(text(AcademyLessonId::Acad3TyreConservation).contains("1st Gold"));
        assert!(text(AcademyLessonId::Acad2RampBalance).contains("CLEAN LANDING"));
        assert!(text(AcademyLessonId::Lesson1ApexLine).contains("Gold 61.2s"));
    }

    #[test]
    fn test_result_banner_says_tyres_worn_out_and_the_stipend_limit() {
        let mut profile = PlayerProfile::new_rookie("Jenson");
        let day = "2026-10-11";
        for id in AcademyLessonId::ALL {
            let def = AcademyLessonDef::find(id).unwrap();
            let mut s = AcademyChallengeState::new(id);
            s.observe(0.0, 0.0);
            evaluate_academy_attempt_on(&mut profile, id, def.bronze_time_sec - 0.1, &s, day);
        }
        // Worn out
        let id = AcademyLessonId::Acad3ThermalThrottle;
        let def = AcademyLessonDef::find(id).unwrap();
        let mut worn = AcademyChallengeState::new(id);
        worn.observe(0.55, 0.0);
        let eval = evaluate_academy_attempt_on(&mut profile, id, def.gold_time_sec - 0.1, &worn, day);
        let (headline, _) = academy_result_summary(&eval);
        assert!(headline.contains("TYRES WORN OUT"), "{headline}");
        assert!(headline.contains("NO MEDAL"), "{headline}");
        assert_eq!(eval.medal, AcademyMedal::None);

        // Stipend limit
        let flick = AcademyLessonId::Acad2ScandinavianFlick;
        let fdef = AcademyLessonDef::find(flick).unwrap();
        let ok = AcademyChallengeState::new(flick);
        let mut last = None;
        for _ in 0..7 {
            last = Some(evaluate_academy_attempt_on(&mut profile, flick, fdef.bronze_time_sec - 0.1, &ok, day));
        }
        let (_, detail) = academy_result_summary(&last.unwrap());
        assert!(detail.contains("DAILY STIPEND LIMIT"), "{detail}");
    }
}
