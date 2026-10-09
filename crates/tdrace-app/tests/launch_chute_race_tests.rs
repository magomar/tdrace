//! Spec 103 (`specs/103_autocross_and_rallycross_launch_chutes_and_templated_track_components.md`): bots race from a
//! launch chute. Lap 1 starts on the pad, merges onto the circuit without snagging, ends at the finish line on the
//! main ribbon, and no later lap enters the chute.

use race_kit::{RaceFormat, RaceRules, RaceWorld};
use tdrace_app::ai::bot_harness::{sample_bot, HARNESS_DT};
use tdrace_app::ai::{DriverTier, DrivingStyle};
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_core::physics::car::Car;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::{ChuteSide, LaunchChuteSpec, Track};
use tdrace_core::CarConfig;

const STYLES: [DrivingStyle; 4] =
    [DrivingStyle::Balanced, DrivingStyle::Aggressive, DrivingStyle::Smooth, DrivingStyle::Calculating];

/// Longest time a bot may go without progress (a bot that long is stuck on a wall).
const MAX_NO_PROGRESS_S: f32 = 10.0;

#[derive(Default, Debug)]
struct ChuteRace {
    /// Every bot started on the pad.
    started_on_pad: bool,
    /// Per bot: the lap it was on when it first left the chute segment, and whether it finished.
    finished: Vec<bool>,
    laps_completed: Vec<u32>,
    /// A bot was on the chute segment after lap 1 (it turned back into it).
    chute_after_lap_one: bool,
    /// A bot was flagged wrong way for more than a second in a row.
    longest_wrong_way_s: f32,
    longest_no_progress_s: Vec<f32>,
    /// A bot never left the chute segment.
    stayed_in_chute: Vec<bool>,
}

/// Races `n` bots from the chute grid of `track` for `laps` laps.
fn race(track: &Track, car: CarConfig, n: usize, laps: u32, max_time_s: f32) -> ChuteRace {
    let chute = track.launch_chute().expect("track has a launch chute");
    let chute_seg = chute.segment_id;
    let net = track.network.as_ref().unwrap();
    let pad = net.get_segment(chute_seg).unwrap();

    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(laps), ..RaceRules::default() });
    let mut drivers = Vec::new();
    for i in 0..n {
        let spawn = track.grid_positions[i];
        world.spawn(
            Car::new(car).with_pose(spawn.position, spawn.angle),
            TrackProgressTracker::new(track.checkpoints.len(), 3),
        );
        let tier = if i % 2 == 0 { DriverTier::Rookie } else { DriverTier::Pro };
        drivers.push(sample_bot(STYLES[i % STYLES.len()], tier, 103 + i as u64));
    }
    let mut out = ChuteRace {
        started_on_pad: world.vehicles.iter().all(|c| pad.project_point(c.state.position).is_on_track),
        finished: vec![false; n],
        laps_completed: vec![0; n],
        longest_no_progress_s: vec![0.0; n],
        stayed_in_chute: vec![true; n],
        ..Default::default()
    };
    let loop_len = track.spline.total_length();
    let mut progress_mark: Option<Vec<f32>> = None;
    let mut no_progress = vec![0.0f32; n];
    let mut wrong_way = vec![0.0f32; n];

    for _ in 0..(max_time_s / HARNESS_DT) as usize {
        if world.vehicles.iter().enumerate().all(|(i, _)| world.is_finished(i)) {
            break;
        }
        let mut controls = Vec::with_capacity(n);
        for i in 0..n {
            let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
            controls.push(drivers[i].compute_controls(&world.vehicles[i], track, &others, HARNESS_DT));
        }
        world.step(track, &controls, HARNESS_DT);
        let progress_mark = progress_mark.get_or_insert_with(|| world.trackers.iter().map(|t| t.progress_distance).collect());
        for i in 0..n {
            let t = &world.trackers[i];
            let on_chute = t.multi_route.as_ref().is_some_and(|m| m.current_segment_id == chute_seg);
            if !on_chute {
                out.stayed_in_chute[i] = false;
            }
            if on_chute && t.current_lap > 1 {
                out.chute_after_lap_one = true;
            }
            out.laps_completed[i] = t.current_lap.saturating_sub(1);
            out.finished[i] = world.is_finished(i);
            if t.is_wrong_way {
                wrong_way[i] += HARNESS_DT;
                out.longest_wrong_way_s = out.longest_wrong_way_s.max(wrong_way[i]);
            } else {
                wrong_way[i] = 0.0;
            }
            let gained = (t.progress_distance - progress_mark[i]).rem_euclid(loop_len);
            if (gained > 5.0 && gained < 0.5 * loop_len) || world.is_finished(i) {
                progress_mark[i] = t.progress_distance;
                no_progress[i] = 0.0;
            } else {
                no_progress[i] += HARNESS_DT;
                out.longest_no_progress_s[i] = out.longest_no_progress_s[i].max(no_progress[i]);
            }
        }
    }
    out
}

fn stamped(module: &str, slug: &str) -> Track {
    let mut track = tdrace_core::catalog::official_track(module, slug);
    track
        .place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left))
        .unwrap_or_else(|e| panic!("{slug}: {e}"));
    track
}

fn assert_clean(slug: &str, r: &ChuteRace, laps: u32) {
    assert!(r.started_on_pad, "{slug}: every car starts on the launch pad");
    assert!(r.stayed_in_chute.iter().all(|stayed| !stayed), "{slug}: a bot never left the chute: {:?}", r.stayed_in_chute);
    assert!(!r.chute_after_lap_one, "{slug}: a bot turned back into the chute after lap 1");
    assert!(r.longest_wrong_way_s < 1.0, "{slug}: a bot drove the wrong way for {:.1} s", r.longest_wrong_way_s);
    for (i, stuck) in r.longest_no_progress_s.iter().enumerate() {
        assert!(*stuck < MAX_NO_PROGRESS_S, "{slug}: bot {i} made no progress for {stuck:.1} s");
    }
    assert!(r.finished.iter().all(|f| *f), "{slug}: not every bot finished {laps} laps: {:?}", r.laps_completed);
}

/// Scenario: Lap 1 Execution from Launch Chute into Main Circuit
///
/// Given an Autocross circuit with a launch chute
/// When 8 bots start from the packed grid and race 3 laps
/// Then they start on the pad, merge onto the loop without getting stuck, and finish all 3 laps
#[test]
fn test_bots_race_three_laps_from_an_autocross_chute() {
    for slug in ["matschenberg_ax", "nova_paka_ax"] {
        let track = stamped("autocross", slug);
        assert_eq!(track.grid_positions.len(), 8);
        assert_clean(slug, &race(&track, ClassicGameModule::car_classic_ax_mudlark(), 8, 3, 400.0), 3);
    }
}

/// Scenario: Subsequent Laps Exclude the Launch Chute
///
/// Given a Rallycross circuit with a launch chute and a joker lap
/// When 8 bots race 3 laps
/// Then none of them turns back into the chute after lap 1, and they all finish
#[test]
fn test_bots_race_three_laps_from_a_rallycross_chute() {
    for slug in ["holjes_rx", "spa_rx"] {
        let track = stamped("rally", slug);
        assert_clean(slug, &race(&track, ClassicGameModule::car_classic_rally(), 8, 3, 400.0), 3);
    }
}
