//! Native visual smoke test for the Spec 064 Academy: the tier tabs, a locked tier, the Tier 3 wear banner
//! on the race HUD, and the result banner after a worn-out finish.
//! Saves screenshots to TDRACE_UI_PREVIEW_DIR and exits.
use macroquad::prelude::*;
use race_kit::FinishState;
use tdrace_app::game::academy::{setup_academy_session, AcademyLessonDef, AcademyLessonId, AcademyMedal};
use tdrace_app::game::{GameState, RaceSession};

fn window_conf() -> Conf {
    Conf { window_title: "Academy preview".into(), window_width: 1600, window_height: 900, ..Default::default() }
}

/// Ends the race with the player first.
fn finish_race(session: &mut RaceSession) {
    let laps = session.total_laps;
    for car in 0..session.world.vehicles.len() {
        let time = 10.0 * laps as f32 + car as f32 * 1.5;
        session.world.finish[car] = FinishState::Finished { time, position: car + 1 };
        session.world.trackers[car].current_lap = laps + 1;
        session.world.trackers[car].best_lap_time = Some(time / laps as f32);
    }
    session.check_race_finish();
}

async fn shoot(session: &mut RaceSession, dir: &str, name: &str) {
    for frame in 0..3 {
        clear_background(BLACK);
        session.render();
        if frame == 2 {
            get_screen_data().export_png(&format!("{}/{}.png", dir, name));
        }
        next_frame().await;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let dir = std::env::var("TDRACE_UI_PREVIEW_DIR").expect("Set TDRACE_UI_PREVIEW_DIR");
    let mut session = RaceSession::new();
    let db = tdrace_app::db::HallOfFameDb::open_in_memory().unwrap();
    let _ = db.seed_default_profile_if_empty().unwrap();
    session.hof_db = Some(db);
    session.refresh_profiles_and_stats();

    // A fresh rookie: Tier 1 open, Tiers 2 and 3 locked.
    session.state = GameState::ClassicAcademy { selected_idx: 0, showing_graduation: false };
    shoot(&mut session, &dir, "academy-tier1-rookie").await;
    session.state = GameState::ClassicAcademy { selected_idx: 4, showing_graduation: false };
    shoot(&mut session, &dir, "academy-tier2-locked").await;

    // Graduated, two Tier 2 Bronzes: Tier 2 open, Tier 3 still locked with its rule.
    for id in AcademyLessonId::ALL {
        session.active_profile.academy_progress.record_attempt(id, 1.0, true);
    }
    // Bronze on the time-based Tier 2 missions: the Flick and the Qualifier first, the Ramp mission third.
    let bronze = |session: &mut RaceSession, id: AcademyLessonId| {
        let def = AcademyLessonDef::find(id).unwrap();
        session.active_profile.academy_progress.record_attempt(id, def.bronze_time_sec - 0.1, true);
    };
    bronze(&mut session, AcademyLessonId::Acad2ScandinavianFlick);
    bronze(&mut session, AcademyLessonId::Acad2MixedQualifier);
    session.state = GameState::ClassicAcademy { selected_idx: 5, showing_graduation: false };
    shoot(&mut session, &dir, "academy-tier2-open").await;
    session.state = GameState::ClassicAcademy { selected_idx: 8, showing_graduation: false };
    shoot(&mut session, &dir, "academy-tier3-locked").await;

    // A third Bronze opens Tier 3.
    bronze(&mut session, AcademyLessonId::Acad2RampBalance);
    session.state = GameState::ClassicAcademy { selected_idx: 11, showing_graduation: false };
    shoot(&mut session, &dir, "academy-tier3-open").await;

    // Mission 3-1: the race HUD with the wear banner.
    setup_academy_session(&mut session, AcademyLessonId::Acad3ThermalThrottle).unwrap();
    session.init_race();
    session.state = GameState::Racing;
    session.world.time = 12.0;
    for wheel in session.world.vehicles[0].state.wheel_assemblies.iter_mut() {
        wheel.wear = 0.31;
        wheel.temperature = 92.0;
    }
    if let Some(challenge) = &mut session.academy_challenge {
        challenge.observe(0.31, 0.0);
    }
    shoot(&mut session, &dir, "academy-hud-wear-31").await;

    // Overheated tyres: the wheel borders of the cockpit HUD turn red.
    for wheel in session.world.vehicles[0].state.wheel_assemblies.iter_mut() {
        wheel.temperature = 130.0;
    }
    shoot(&mut session, &dir, "academy-hud-overheat").await;

    // Worn out at the flag: no medal.
    for wheel in session.world.vehicles[0].state.wheel_assemblies.iter_mut() {
        wheel.wear = 0.55;
    }
    if let Some(challenge) = &mut session.academy_challenge {
        challenge.observe(0.55, 0.0);
    }
    shoot(&mut session, &dir, "academy-hud-wear-over-cap").await;
    finish_race(&mut session);
    let eval = session.academy_last_evaluation.as_ref().expect("an Academy attempt is evaluated");
    assert_eq!(eval.medal, AcademyMedal::None);
    println!("worn out: {:?}", eval.tyres_worn_out);
    shoot(&mut session, &dir, "academy-result-worn-out").await;

    // A cleared mission re-run past the daily stipend limit.
    for id in AcademyLessonId::tier_lessons(2) {
        let def = AcademyLessonDef::find(*id).unwrap();
        session.active_profile.academy_progress.record_attempt(*id, def.gold_time_sec - 1.0, true);
    }
    session.active_profile.academy_progress.stipend_day = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
    session.active_profile.academy_progress.stipends_today = 5;
    // The race reloads the profile from the database.
    session.hof_db.as_ref().unwrap().update_profile(&session.active_profile).unwrap();
    setup_academy_session(&mut session, AcademyLessonId::Acad2ScandinavianFlick).unwrap();
    session.init_race();
    session.state = GameState::Racing;
    finish_race(&mut session);
    println!("stipend limit reached: {:?}", session.academy_last_evaluation.as_ref().map(|e| e.stipend_limit_reached));
    shoot(&mut session, &dir, "academy-result-stipend-limit").await;
}
