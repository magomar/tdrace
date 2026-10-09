//! Spec 103 (`specs/103_autocross_and_rallycross_launch_chutes_and_templated_track_components.md`): bots race from a
//! launch chute. Lap 1 starts on the pad, merges onto the circuit without snagging, ends at the finish line on the
//! main ribbon, and no later lap enters the chute.

use race_kit::{RaceFormat, RaceRules, RaceWorld};
use tdrace_app::ai::bot_harness::{sample_bot, HARNESS_DT};
use tdrace_app::ai::{DriverTier, DrivingStyle};
use tdrace_app::module::classic::ClassicGameModule;
use tdrace_app::module::rally::RallyGameModule;
use tdrace_core::physics::car::Car;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::{ChuteSide, LaunchChuteSpec, Track};
use tdrace_core::CarConfig;

const STYLES: [DrivingStyle; 4] =
    [DrivingStyle::Balanced, DrivingStyle::Aggressive, DrivingStyle::Smooth, DrivingStyle::Calculating];

/// Longest time a bot may go without progress in the launch (a bot that long is stuck on a wall).
const MAX_NO_PROGRESS_S: f32 = 10.0;
/// Bots of a grid of 8 that must complete lap 1.
const MIN_THROUGH_LAP_ONE: usize = 6;
/// Longest time a bot may be flagged wrong way on the chute: a spin after a start collision is not a diversion.
const MAX_WRONG_WAY_S: f32 = 3.0;

#[derive(Default, Debug)]
struct ChuteRace {
    /// Every bot started on the pad.
    started_on_pad: bool,
    /// Per bot: laps completed.
    laps_completed: Vec<u32>,
    /// A bot was on the chute segment after lap 1 (it turned back into it).
    chute_after_lap_one: bool,
    /// Longest time in a row a bot was flagged wrong way while on the chute segment.
    longest_wrong_way_on_chute_s: f32,
    /// Per bot: the longest time without progress during the launch, from the start to the first crossing of the
    /// finish line. A bot that sticks later on the circuit is not this spec's concern.
    longest_launch_stall_s: Vec<f32>,
    /// Per bot: it never left the chute segment.
    stayed_in_chute: Vec<bool>,
}

/// Races `n` bots from the chute grid of `track` for `laps` laps.
fn race(track: &Track, car: CarConfig, n: usize, laps: u32, max_time_s: f32) -> ChuteRace {
    let net = track.network.as_ref().expect("a circuit with a launch chute has a network");
    let chute_seg = track.launch_chute().expect("track has a launch chute").segment_id;
    let pad = net.get_segment(chute_seg).unwrap();

    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::Laps(laps), ..RaceRules::default() });
    let mut drivers = Vec::new();
    for i in 0..n {
        let spawn = track.grid_positions[i];
        let mut car = Car::new(car).with_pose(spawn.position, spawn.angle);
        // As in the game: damage is off by default.
        car.config.damage_enabled = world.rules.damage_enabled;
        world.spawn(car, TrackProgressTracker::new(track.checkpoints.len(), 3));
        let tier = if i % 2 == 0 { DriverTier::Rookie } else { DriverTier::Pro };
        drivers.push(sample_bot(STYLES[i % STYLES.len()], tier, 55 + i as u64));
    }
    let mut out = ChuteRace {
        started_on_pad: world.vehicles.iter().all(|c| pad.project_point(c.state.position).is_on_track),
        laps_completed: vec![0; n],
        longest_launch_stall_s: vec![0.0; n],
        stayed_in_chute: vec![true; n],
        ..Default::default()
    };
    let loop_len = track.spline.total_length();
    let mut progress_mark: Option<Vec<f32>> = None;
    let mut no_progress = vec![0.0f32; n];
    let mut wrong_way = vec![0.0f32; n];

    for _ in 0..(max_time_s / HARNESS_DT) as usize {
        if (0..n).all(|i| world.is_finished(i)) {
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
            if t.is_wrong_way && on_chute {
                wrong_way[i] += HARNESS_DT;
                out.longest_wrong_way_on_chute_s = out.longest_wrong_way_on_chute_s.max(wrong_way[i]);
            } else {
                wrong_way[i] = 0.0;
            }
            let gained = (t.progress_distance - progress_mark[i]).rem_euclid(loop_len);
            if (gained > 5.0 && gained < 0.5 * loop_len) || world.is_finished(i) {
                progress_mark[i] = t.progress_distance;
                no_progress[i] = 0.0;
            } else {
                no_progress[i] += HARNESS_DT;
                // The launch: from the start line to the first crossing of the finish line (nothing passed yet).
                if t.current_lap == 1 && t.checkpoints_passed_this_lap == 0 {
                    out.longest_launch_stall_s[i] = out.longest_launch_stall_s[i].max(no_progress[i]);
                }
            }
        }
    }
    out
}

/// An official circuit with its launch chute (stamped here when the catalog copy has none yet).
fn with_chute(module: &str, slug: &str) -> Track {
    let mut track = tdrace_core::catalog::official_track(module, slug);
    if track.launch_chute().is_none() {
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap_or_else(|e| panic!("{slug}: {e}"));
    }
    track
}

/// The car of the circuit's own category: the module's default car (the Classic Rallycross car on a Classic circuit).
fn own_car(module: &str) -> CarConfig {
    match module {
        "autocross" => tdrace_app::catalog::find_model_by_id("autocross_ardennes_junior_t1").expect("AX default car").to_car_config(),
        "rally" => RallyGameModule::car_wrc_rally(),
        _ => ClassicGameModule::car_classic_rally(),
    }
}

fn check(slug: &str, r: &ChuteRace) -> Result<(), String> {
    let fail = |why: String| Err(format!("{slug}: {why}"));
    if !r.started_on_pad {
        return fail("a car does not start on the launch pad".to_string());
    }
    if r.stayed_in_chute.iter().any(|stayed| *stayed) {
        return fail(format!("a bot never left the chute: {:?}", r.stayed_in_chute));
    }
    if r.chute_after_lap_one {
        return fail("a bot turned back into the chute after lap 1".to_string());
    }
    if r.longest_wrong_way_on_chute_s >= MAX_WRONG_WAY_S {
        return fail(format!("a bot was flagged wrong way for {:.1} s on the chute", r.longest_wrong_way_on_chute_s));
    }
    if let Some((i, stuck)) = r.longest_launch_stall_s.iter().enumerate().find(|(_, s)| **s >= MAX_NO_PROGRESS_S) {
        return fail(format!("bot {i} made no progress for {stuck:.1} s in the launch"));
    }
    // Lap 1 ends at the finish line on the main ribbon. A bot can still stick later on a circuit's joker or its
    // hairpins (they do without a chute, on 11 of the 40 circuits), so most of the grid must get through lap 1.
    let through = r.laps_completed.iter().filter(|&&l| l >= 1).count();
    if through < MIN_THROUGH_LAP_ONE {
        return fail(format!("only {through} of {} bots completed lap 1: {:?}", r.laps_completed.len(), r.laps_completed));
    }
    Ok(())
}

/// Races 8 bots of each circuit's own car for 3 laps; returns what went wrong.
fn sweep(circuits: &[(&str, &str)]) -> Vec<String> {
    circuits
        .iter()
        .filter_map(|(module, slug)| {
            let track = with_chute(module, slug);
            assert_eq!(track.grid_positions.len(), 8, "{slug}: the chute holds 8 cars");
            check(slug, &race(&track, own_car(module), 8, 3, 400.0)).err()
        })
        .collect()
}

/// Scenario: Lap 1 Execution from Launch Chute into Main Circuit
///
/// Given an Autocross circuit with a launch chute
/// When 8 bots start from the packed grid and race 3 laps
/// Then they start on the pad, merge onto the loop without getting stuck, and complete lap 1 at the finish line
#[test]
fn test_bots_race_three_laps_from_an_autocross_chute() {
    let failures = sweep(&[("autocross", "matschenberg_ax"), ("autocross", "nova_paka_ax")]);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Scenario: Subsequent Laps Exclude the Launch Chute
///
/// Given a Rallycross circuit with a launch chute and a joker lap
/// When 8 bots race 3 laps
/// Then none of them turns back into the chute after lap 1
#[test]
fn test_bots_race_three_laps_from_a_rallycross_chute() {
    let failures = sweep(&[("rally", "holjes_rx"), ("rally", "spa_rx")]);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// The same race on all 40 circuits. About two minutes in release, far longer in a debug build:
/// `cargo test --release -p tdrace-app --test launch_chute_race_tests -- --ignored`.
#[test]
#[ignore = "races 40 circuits; run in release"]
fn test_bots_race_from_every_launch_chute() {
    let ids = |module: &str| -> Vec<(String, String)> {
        tdrace_core::catalog::module_circuits(module).map(|c| (c.module.to_string(), c.id.to_string())).collect()
    };
    let mut circuits = ids("autocross");
    circuits.extend(ids("rally"));
    circuits.extend(["rx_canyon_flyer", "rx_hilltop_leap", "rx_quarry_sprint"].map(|id| ("classic".to_string(), id.to_string())));
    let circuits: Vec<(&str, &str)> = circuits.iter().map(|(m, i)| (m.as_str(), i.as_str())).collect();
    assert_eq!(circuits.len(), 40);
    let failures = sweep(&circuits);
    assert!(failures.is_empty(), "{} of 40 circuits: {failures:#?}", failures.len());
}
