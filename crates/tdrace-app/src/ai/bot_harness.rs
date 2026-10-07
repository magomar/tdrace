//! Headless bot race harness (spec 046:
//! `specs/046_humanlike_bot_driving_with_tiered_mistakes_and_varied_lines.md`).
//!
//! Runs a grid of cars on an official track through `race_kit::RaceWorld`, the same race step
//! as `RaceSession` (spec 056), without rendering, audio or FX. Tests and
//! `bot_behaviour_benchmark` use it to measure lines, braking points, lap times and mistakes.

use cabinet::input::filter::{DigitalInputConfig, DigitalInputFilter, SteeringProfile};
use race_kit::{RaceFormat, RaceRules, RaceWorld};
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::{CarConfig, PlayerHandling};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::Track;

use super::humanize::{BotDrivingStats, Corner, HumanTraits};
use super::{BotAiDriver, BotProfile, DriverQuality, DriverTier, DrivingStyle};
use crate::catalog::CLASSIC_ARCADE_CARS;

/// Physics step of the game (`RaceSession::FIXED_DT`).
pub const HARNESS_DT: f32 = 1.0 / 120.0;
/// Number of fixed track stations where the lateral position is sampled each lap.
pub const LINE_STATIONS: usize = 200;

/// One car in a harness race.
pub struct HarnessEntry {
    pub driver: BotAiDriver,
    pub config: CarConfig,
    /// Keyboard reference driver: the bot's controls are cut to key presses and sent through the
    /// player filter and `PlayerHandling`, as for a human car.
    pub keyboard: Option<DigitalInputConfig>,
}

impl HarnessEntry {
    pub fn bot(driver: BotAiDriver, config: CarConfig) -> Self {
        Self { driver, config, keyboard: None }
    }
}

/// Key press threshold of the keyboard reference driver.
const KEY_THRESHOLD: f32 = 0.3;

fn key(v: f32) -> f32 {
    if v > KEY_THRESHOLD { 1.0 } else if v < -KEY_THRESHOLD { -1.0 } else { 0.0 }
}

/// What one car did in a harness race.
#[derive(Debug, Clone, Default)]
pub struct HarnessCarResult {
    /// Completed lap times in seconds, the standing-start lap first.
    pub lap_times: Vec<f32>,
    /// Lateral offset (m, + = right) at each of the `LINE_STATIONS` stations, one row per lap.
    pub lateral_by_lap: Vec<Vec<f32>>,
    /// Track distance (m) of each brake onset, one row per lap.
    pub brake_onsets_by_lap: Vec<Vec<f32>>,
    /// Longest continuous time below 2 m/s after the first 15 m (s).
    pub longest_slow_s: f32,
    /// Longest time without gaining 5 m of track progress after the first 15 m (s).
    pub longest_no_progress_s: f32,
    /// Order-sensitive hash of every control output (determinism checks).
    pub controls_hash: u64,
    pub finished: bool,
    /// The bot's human-layer counters at the end of the race.
    pub stats: BotDrivingStats,
}

impl HarnessCarResult {
    /// Mean of the flying laps (every lap after the standing start).
    pub fn mean_flying_lap(&self) -> Option<f32> {
        let flying = self.lap_times.get(1..)?;
        (!flying.is_empty()).then(|| flying.iter().sum::<f32>() / flying.len() as f32)
    }
}

fn hash_controls(h: u64, c: &CarControls) -> u64 {
    let mut h = h;
    for v in [c.throttle.to_bits(), c.steer.to_bits(), c.brake.to_bits(), c.handbrake as u32, c.reverse as u32] {
        h = (h ^ v as u64).wrapping_mul(0x100000001B3);
    }
    h
}

/// Runs `entries` from the track grid until every car completes `laps` laps or `max_time_s`
/// passes.
pub fn run_harness_race(track: &Track, entries: Vec<HarnessEntry>, laps: u32, max_time_s: f32) -> Vec<HarnessCarResult> {
    assert!(entries.len() <= track.grid_positions.len(), "track has only {} grid positions", track.grid_positions.len());
    let n = entries.len();
    let len = track.spline.total_length();
    let mut drivers = Vec::with_capacity(n);
    // The harness counts laps itself, so the world never finishes a car.
    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules { format: RaceFormat::TimeAttack, ..RaceRules::default() });
    let mut filters = Vec::with_capacity(n);
    for (i, e) in entries.into_iter().enumerate() {
        let spawn = track.grid_positions[i];
        let mut car = Car::new(e.config).with_pose(spawn.position, spawn.angle);
        // Like RaceSession::apply_car_damage_setting: the game's default (damage off) reaches every car,
        // so curb and landing strikes do not wear the suspension either.
        car.config.damage_enabled = world.rules.damage_enabled;
        if let Some(kb) = e.keyboard {
            car.config.player = PlayerHandling::human(kb.steer_authority, kb.traction_help);
        }
        world.spawn(car, TrackProgressTracker::new(track.checkpoints.len(), 3));
        drivers.push(e.driver);
        filters.push(e.keyboard.map(DigitalInputFilter::new));
    }
    let mut results = vec![HarnessCarResult { controls_hash: 0xcbf29ce484222325, ..Default::default() }; n];
    let mut lap_lateral: Vec<Vec<f32>> = vec![vec![f32::NAN; LINE_STATIONS]; n];
    let mut lap_onsets: Vec<Vec<f32>> = vec![Vec::new(); n];
    let mut prev_brake = vec![0.0f32; n];
    let mut slow = vec![0.0f32; n];
    let mut travelled = vec![0.0f32; n];
    let mut progress_mark = vec![0.0f32; n];
    let mut no_progress = vec![0.0f32; n];
    let station_len = len / LINE_STATIONS as f32;

    let steps = (max_time_s / HARNESS_DT) as usize;
    for _ in 0..steps {
        if results.iter().all(|r| r.finished) {
            break;
        }
        let mut controls = Vec::with_capacity(n);
        for i in 0..n {
            let others: Vec<&Car> = world.vehicles.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, c)| c).collect();
            let mut c = drivers[i].compute_controls(&world.vehicles[i], track, &others, HARNESS_DT);
            if let Some(filter) = filters[i].as_mut() {
                let (steer, throttle, brake) = filter.update(key(c.steer), key(c.throttle), key(c.brake), HARNESS_DT);
                c = CarControls { steer, throttle, brake, ..c };
            }
            results[i].controls_hash = hash_controls(results[i].controls_hash, &c);
            controls.push(c);
        }

        let prev: Vec<(f32, u32)> = world.trackers.iter().map(|t| (t.progress_distance, t.current_lap)).collect();
        world.step(track, &controls, HARNESS_DT);
        let (cars, trackers) = (&world.vehicles, &world.trackers);

        for i in 0..n {
            let (prev_progress, prev_lap) = prev[i];
            if results[i].finished {
                continue;
            }
            let t = &trackers[i];
            let speed = cars[i].state.speed;

            travelled[i] += speed * HARNESS_DT;
            if travelled[i] > 15.0 && speed < 2.0 {
                slow[i] += HARNESS_DT;
                results[i].longest_slow_s = results[i].longest_slow_s.max(slow[i]);
            } else {
                slow[i] = 0.0;
            }
            let gained = (t.progress_distance - progress_mark[i]).rem_euclid(len);
            if travelled[i] <= 15.0 || (gained > 5.0 && gained < 0.5 * len) {
                progress_mark[i] = t.progress_distance;
                no_progress[i] = 0.0;
            } else {
                no_progress[i] += HARNESS_DT;
                results[i].longest_no_progress_s = results[i].longest_no_progress_s.max(no_progress[i]);
            }

            // Lateral samples at stations crossed this tick (forward motion only).
            let advance = (t.progress_distance - prev_progress).rem_euclid(len);
            if advance > 0.0 && advance < 20.0 {
                let first = (prev_progress / station_len).floor() as usize + 1;
                let last = (prev_progress + advance) / station_len;
                let mut k = first;
                while (k as f32) <= last {
                    let proj = track.spline.project_point_continuity(cars[i].state.position, t.progress_distance, 50.0);
                    lap_lateral[i][k % LINE_STATIONS] = proj.lateral_offset;
                    k += 1;
                }
            }

            let brake = controls[i].brake;
            if brake > 0.2 && prev_brake[i] <= 0.2 && speed > 10.0 {
                lap_onsets[i].push(t.progress_distance);
            }
            prev_brake[i] = brake;

            if t.current_lap > prev_lap && prev_lap >= 1 {
                if let Some(lt) = t.last_lap_time {
                    results[i].lap_times.push(lt);
                }
                results[i].lateral_by_lap.push(std::mem::replace(&mut lap_lateral[i], vec![f32::NAN; LINE_STATIONS]));
                results[i].brake_onsets_by_lap.push(std::mem::take(&mut lap_onsets[i]));
                if results[i].lap_times.len() as u32 >= laps {
                    results[i].finished = true;
                }
            } else if t.current_lap > prev_lap {
                // Crossing the line from the grid starts lap 1: discard the grid run-up.
                lap_lateral[i] = vec![f32::NAN; LINE_STATIONS];
                lap_onsets[i].clear();
            }
        }
    }
    for (r, d) in results.iter_mut().zip(&drivers) {
        r.stats = d.human.stats.clone();
    }
    results
}

/// Tracks and cars of the spec 046 sample: (track slug in the classic module, car id).
pub const SAMPLE_TRACKS: [(&str, &str); 4] = [
    ("gt_coastal_grand_prix", "classic_gt"),
    ("stock_tri_oval_speedway", "classic_nascar"),
    ("gt_ridge_ring", "classic_rally"),
    ("kart_pine_grove", "classic_kart"),
];

/// One race of a six-car grid, one car per `DrivingStyle::ALL` entry, all at one tier.
#[derive(Debug, Clone)]
pub struct StyleGridRun {
    pub track: &'static str,
    pub tier: DriverTier,
    /// Indexed like `DrivingStyle::ALL`.
    pub results: Vec<HarnessCarResult>,
}

/// Fixed seed of one car in the sample.
pub fn sample_seed(track_idx: usize, tier: DriverTier, style_idx: usize) -> u64 {
    0x5EED_0450_0000 ^ ((track_idx as u64) << 16 | (tier.to_u8() as u64) << 8 | style_idx as u64).wrapping_mul(0x9E3779B97F4A7C15)
}

fn car_config(car_id: &str) -> CarConfig {
    CLASSIC_ARCADE_CARS.iter().find(|c| c.id == car_id).expect("sample car must exist").to_car_config()
}

/// A bot of one style and tier, as the game builds it.
pub fn sample_bot(style: DrivingStyle, tier: DriverTier, seed: u64) -> BotAiDriver {
    BotAiDriver::with_seed(BotProfile::from_style_and_quality(style, &DriverQuality::for_tier(tier)), seed)
}

/// Runs one style grid per (sample track, tier), in parallel threads.
pub fn run_style_grids(tiers: &[DriverTier], laps: u32) -> Vec<StyleGridRun> {
    let jobs: Vec<(usize, DriverTier)> = (0..SAMPLE_TRACKS.len()).flat_map(|t| tiers.iter().map(move |&tier| (t, tier))).collect();
    std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|&(t, tier)| {
                scope.spawn(move || {
                    let (slug, car_id) = SAMPLE_TRACKS[t];
                    let track = tdrace_core::catalog::official_track("classic", slug);
                    let cfg = car_config(car_id);
                    let entries = DrivingStyle::ALL
                        .iter()
                        .enumerate()
                        .map(|(s, &style)| HarnessEntry::bot(sample_bot(style, tier, sample_seed(t, tier, s)), cfg))
                        .collect();
                    StyleGridRun { track: slug, tier, results: run_harness_race(&track, entries, laps, laps as f32 * 240.0) }
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("harness race thread panicked")).collect()
    })
}

/// `SAMPLE_TRACKS` indices of the keyboard reference races (Coastal GP, Ridge Ring).
pub const KEYBOARD_REFERENCE_TRACKS: [usize; 2] = [0, 2];

/// Races the keyboard reference driver (a T3 Balanced bot with `HumanTraits::none()`, on the
/// Balanced keyboard filter) from the back of a Tier 1 style grid. Returns the reference result and
/// the six bot results.
pub fn run_keyboard_reference_race(track_idx: usize, laps: u32) -> (HarnessCarResult, Vec<HarnessCarResult>) {
    let (slug, car_id) = SAMPLE_TRACKS[track_idx];
    let track = tdrace_core::catalog::official_track("classic", slug);
    let cfg = car_config(car_id);
    let mut entries: Vec<HarnessEntry> = DrivingStyle::ALL
        .iter()
        .enumerate()
        .map(|(s, &style)| HarnessEntry::bot(sample_bot(style, DriverTier::Rookie, sample_seed(track_idx, DriverTier::Rookie, s)), cfg))
        .collect();
    let mut reference = BotProfile::from_style_and_quality(DrivingStyle::Balanced, &DriverQuality::for_tier(DriverTier::Contender));
    reference.traits = HumanTraits::none();
    entries.push(HarnessEntry {
        driver: BotAiDriver::new(reference),
        config: cfg,
        keyboard: Some(DigitalInputConfig::from_profile(SteeringProfile::Balanced)),
    });
    let mut results = run_harness_race(&track, entries, laps, laps as f32 * 240.0);
    let reference = results.pop().expect("reference result");
    (reference, results)
}

fn std_dev(xs: &[f32]) -> f32 {
    let n = xs.len() as f32;
    let mean = xs.iter().sum::<f32>() / n;
    (xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / n).sqrt()
}

/// Mean over the track stations of the lap-to-lap standard deviation of the lateral position (m).
pub fn line_spread_m(r: &HarnessCarResult) -> f32 {
    let sds: Vec<f32> = (0..LINE_STATIONS)
        .filter_map(|k| {
            let xs: Vec<f32> = r.lateral_by_lap.iter().map(|lap| lap[k]).filter(|x| x.is_finite()).collect();
            (xs.len() >= 3).then(|| std_dev(&xs))
        })
        .collect();
    if sds.is_empty() { 0.0 } else { sds.iter().sum::<f32>() / sds.len() as f32 }
}

/// Mean over the corners of the lap-to-lap standard deviation of the brake onset (m). Each onset
/// belongs to the first corner that starts within 150 m after it.
pub fn brake_onset_spread_m(r: &HarnessCarResult, corners: &[Corner], len: f32) -> f32 {
    let mut by_corner: Vec<Vec<f32>> = vec![Vec::new(); corners.len()];
    for lap in &r.brake_onsets_by_lap {
        let mut seen = vec![false; corners.len()];
        for &d in lap {
            let next = corners
                .iter()
                .enumerate()
                .map(|(i, c)| (i, (c.start - d).rem_euclid(len)))
                .filter(|&(_, gap)| gap < 150.0)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((i, gap)) = next {
                if !seen[i] {
                    seen[i] = true;
                    by_corner[i].push(gap);
                }
            }
        }
    }
    let sds: Vec<f32> = by_corner.iter().filter(|xs| xs.len() >= 3).map(|xs| std_dev(xs)).collect();
    if sds.is_empty() { 0.0 } else { sds.iter().sum::<f32>() / sds.len() as f32 }
}

/// Lap-to-lap spread of the flying laps: standard deviation over mean.
pub fn lap_spread(laps: &[f32]) -> f32 {
    if laps.len() < 2 { 0.0 } else { std_dev(laps) / (laps.iter().sum::<f32>() / laps.len() as f32) }
}
