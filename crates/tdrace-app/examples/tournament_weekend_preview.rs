//! Native visual smoke test for the Spec 104 tournament weekend: the race HUD with its stage badge and the
//! bracket screen after the heat and after the semifinal.
//! Saves screenshots to TDRACE_UI_PREVIEW_DIR and exits.
use macroquad::prelude::*;
use race_kit::FinishState;
use tdrace_app::game::{GameState, RaceSession};
use tdrace_app::series::SeriesDefinition;

const SUPERBUGGY: &str = include_str!("../../../series/autocross/autocross_superbuggy_world_series.toml");

fn window_conf() -> Conf {
    Conf { window_title: "Tournament weekend preview".into(), window_width: 1600, window_height: 900, ..Default::default() }
}

/// Ends the race: the player takes `player_rank`, the other cars follow in grid order.
fn finish_race(session: &mut RaceSession, player_rank: usize) {
    let n = session.world.vehicles.len();
    let mut order: Vec<usize> = (1..n).collect();
    order.insert(player_rank - 1, 0);
    let laps = session.total_laps;
    for (rank, &car) in order.iter().enumerate() {
        let time = 10.0 * laps as f32 + rank as f32 * 1.5;
        session.world.finish[car] = FinishState::Finished { time, position: rank + 1 };
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
    session.launch_or_resume_championship(&SeriesDefinition::from_toml(SUPERBUGGY).unwrap());

    // Heat: the HUD of the race with the stage badge.
    session.state = GameState::Racing;
    session.world.time = 12.0;
    shoot(&mut session, &dir, "tournament-hud-heat").await;

    // After the heat: Enter on the results screen opens the bracket with the heat results and the semifinal draw.
    finish_race(&mut session, 2);
    session.input.gamepad.snapshot.btn_confirm_pressed = true;
    session.update_finished_screen();
    session.input.gamepad.snapshot.btn_confirm_pressed = false;
    shoot(&mut session, &dir, "tournament-bracket-after-heat").await;

    // [LAUNCH RACE]: the semifinal HUD with its badge and 5 laps.
    session.launch_tournament_race();
    session.state = GameState::Racing;
    shoot(&mut session, &dir, "tournament-hud-semifinal").await;
}
