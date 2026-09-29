//! Spec 056 (`specs/056_racekit_headless_race_world.md`): tdrace results use the world's real times.

use race_kit::FinishState;
use tdrace_app::config::GameConfig;
use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

/// Points every user folder at a fresh, empty directory owned by this test process.
fn isolate_user_storage() {
    let root = std::env::temp_dir().join(format!("tdrace_race_results_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (var, sub) in [
        ("TDRACE_USER_DATA_DIR", "data"),
        ("TDRACE_USER_CONFIG_DIR", "config"),
        ("TDRACE_USER_TRACKS_DIR", "tracks"),
        ("TDRACE_USER_SERIES_DIR", "series"),
    ] {
        let dir = root.join(sub);
        std::fs::create_dir_all(&dir).expect("create isolated user folder");
        std::env::set_var(var, &dir);
    }
}

/// Scenario: tdrace results show real times
///
/// Given a tdrace race where the player finishes first and the bots are still racing
/// When the results screen is built
/// Then the player's time is the world finish time, each bot's time is marked projected, and no
/// time comes from rank * 0.65
#[test]
fn results_use_world_finish_and_projected_times() {
    isolate_user_storage();
    let mut session = RaceSession::new_with_config(GameConfig::default());
    session.hof_db = Some(HallOfFameDb::open_in_memory().expect("in-memory Hall of Fame"));
    session.refresh_profiles_and_stats();
    session.fixed_roster_seed = Some(0x5EED_0560);
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.car_choice = CarChoice::SportsCar;
    session.num_bots = 5;
    session.init_race();
    assert!(!session.is_time_attack);

    for _ in 0..1200 {
        session.physics_step(RaceSession::FIXED_DT);
    }
    // The player gets no input in tests, so put it past the last lap and let the world see it.
    session.world.trackers[0].current_lap = session.total_laps + 1;
    session.physics_step(RaceSession::FIXED_DT);
    let finish_time = session.world.time;
    assert_eq!(session.world.finish[0], FinishState::Finished { time: finish_time, position: 1 });

    session.check_race_finish();
    let results = &session.results;
    assert_eq!(results.len(), 6);
    assert!(results[0].is_player, "the player finished first");
    assert!(!results[0].projected);
    assert_eq!(results[0].total_time, finish_time, "the player's time is the world finish time");
    for (rank, row) in results.iter().enumerate().skip(1) {
        assert!(row.projected, "P{} is still racing, so its time is projected", rank + 1);
        assert!(row.total_time > finish_time, "a bot still racing finishes after the player");
        assert!(row.total_time >= results[rank - 1].total_time, "times do not go down");
        let old = session.session_time + rank as f32 * 0.65;
        assert!((row.total_time - old).abs() > 1.0, "P{} time {} looks like the old rank * 0.65 rule", rank + 1, row.total_time);
        assert!((row.delta_to_leader - (row.total_time - finish_time)).abs() < 1e-3);
    }
}
