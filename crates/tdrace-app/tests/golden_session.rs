//! Golden state hash for the full game physics step (`RaceSession::physics_step`).
//!
//! Spec 049 (`specs/049_reusable_racing_platform_layers.md`), Phase 0. Phase 2 moves the
//! simulation rows of `physics_step` into a headless `race-kit` world. This test pins the
//! exact result of today's step, including the parts the bot harness skips (canopy drag,
//! ramps), so that extraction can prove it changed nothing.
//!
//! The race uses a fixed roster seed, Classic Grand Prix, the sports car and 5 bots. The
//! player car gets no input in tests (macroquad is not running), so it stays on the grid
//! and the bots race around it.
//!
//! The race must not read state that other tests write. It uses `GameConfig::default()`,
//! its own empty user folders, and an in-memory Hall of Fame database. The default
//! database is `tdrace_records.db` in the working directory, which every test binary
//! shares; its best laps set the grid order, and in a full `cargo test` run they changed
//! this race (measured 2026-09-28).
//!
//! If a change is meant to alter physics results, re-record the constants below in a
//! commit of its own and say why in the message.

use tdrace_app::config::GameConfig;
use tdrace_app::db::HallOfFameDb;
use tdrace_app::game::RaceSession;
use tdrace_app::ui::menu::{CarChoice, TrackChoice};

const STEPS: usize = 3600;
const SEED: u64 = 0x5EED_0490;

/// Recorded hash per platform and build mode (see `arcade-race-core/tests/golden_sim.rs`
/// for why debug and release differ). `None` means no hash is recorded for this platform:
/// the test then checks only that two runs agree, and prints the hash to record.
fn recorded() -> Option<u64> {
    let mac_arm = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    match (mac_arm, cfg!(debug_assertions)) {
        (true, true) => Some(0xf749b7e579aa775a),
        (true, false) => Some(0x6724b29cb789c297),
        _ => None,
    }
}

fn fnv(h: u64, v: u64) -> u64 {
    (h ^ v).wrapping_mul(0x100000001B3)
}

/// Points every user folder at a fresh, empty directory owned by this test process.
fn isolate_user_storage() {
    let root = std::env::temp_dir().join(format!("tdrace_golden_session_{}", std::process::id()));
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

/// Returns the hash and the largest progress distance any bot reached.
fn run() -> (u64, f32) {
    isolate_user_storage();
    let mut session = RaceSession::new_with_config(GameConfig::default());
    session.hof_db = Some(HallOfFameDb::open_in_memory().expect("in-memory Hall of Fame"));
    // The constructor already read profile stats (best laps) from the shared DB; reload them.
    session.refresh_profiles_and_stats();
    session.fixed_roster_seed = Some(SEED);
    session.track_choice = TrackChoice::ClassicGrandPrix;
    session.car_choice = CarChoice::SportsCar;
    session.num_bots = 5;
    session.init_race();
    assert_eq!(session.world.vehicles.len(), 6);

    let mut hash: u64 = 0xcbf29ce484222325;
    for step in 0..STEPS {
        session.physics_step(RaceSession::FIXED_DT);
        for (car, tracker) in session.world.vehicles.iter().zip(&session.world.trackers) {
            let s = &car.state;
            for v in [
                s.position.x,
                s.position.y,
                s.angle,
                s.velocity.x,
                s.velocity.y,
                s.angular_velocity,
                s.speed,
                tracker.progress_distance,
            ] {
                assert!(v.is_finite(), "non-finite state at step {}", step);
                hash = fnv(hash, v.to_bits() as u64);
            }
            hash = fnv(hash, tracker.current_lap as u64);
            hash = fnv(hash, tracker.next_checkpoint_idx as u64);
        }
    }
    let furthest = session.world.trackers[1..].iter().map(|t| t.progress_distance).fold(0.0f32, f32::max);
    (hash, furthest)
}

#[test]
fn golden_session() {
    let (hash, furthest) = run();
    assert!(furthest > 100.0, "bots must race, furthest progress {:.1} m", furthest);
    match recorded() {
        Some(golden) => assert_eq!(hash, golden, "session state hash changed (got {:#018x}, recorded {:#018x})", hash, golden),
        None => {
            let (again, _) = run();
            assert_eq!(hash, again, "two runs in one process differ");
            eprintln!("no recorded session hash for this platform; got {:#018x}", hash);
        }
    }
}
