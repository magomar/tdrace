//! Headless bot race harness (spec 046:
//! `specs/046_humanlike_bot_driving_with_tiered_mistakes_and_varied_lines.md`).
//!
//! Runs a grid of cars on an official track through `race_kit::RaceWorld`, the same race step
//! as `RaceSession` (spec 056), without rendering, audio or FX. Tests and
//! `bot_behaviour_benchmark` use it to measure lines, braking points, lap times and mistakes.

use cabinet::input::filter::{DigitalInputConfig, DigitalInputFilter, SteeringProfile};
use race_kit::{RaceEvent, RaceFormat, RaceRules, RaceWorld};
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::physics::config::{CarConfig, PlayerHandling};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::Track;

use super::humanize::{BotDrivingStats, Corner, HumanTraits};
use super::{BotAiDriver, BotProfile, DriverQuality, DriverTier, DrivingStyle, StyleTuning};
use crate::catalog::CLASSIC_ARCADE_CARS;
use crate::module::autocross::AutocrossGameModule;
use crate::module::classic::ClassicGameModule;
use crate::module::kart::KartGameModule;
use crate::module::rally::RallyGameModule;
use crate::module::GameModule;

/// Physics step of the game (`RaceSession::FIXED_DT`).
pub const HARNESS_DT: f32 = 1.0 / 120.0;
/// Time without wall impacts that separates two wall contacts (s).
pub const WALL_CONTACT_GAP_S: f32 = 0.25;
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
    /// Separate wall contacts: a `RaceEvent::WallImpact` after at least `WALL_CONTACT_GAP_S` without
    /// one. A scrape along a wall is one contact.
    pub wall_contacts: u32,
    /// Physics ticks the race ran (the same for every car).
    pub race_ticks: u32,
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
    // Ticks since each car's last wall impact.
    let mut off_wall = vec![u32::MAX; n];
    let gap_ticks = (WALL_CONTACT_GAP_S / HARNESS_DT).round() as u32;
    let mut ticks = 0u32;

    let steps = (max_time_s / HARNESS_DT) as usize;
    for _ in 0..steps {
        if results.iter().all(|r| r.finished) {
            break;
        }
        ticks += 1;
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
        let mut touching = vec![false; n];
        for ev in world.events() {
            if let RaceEvent::WallImpact { car, .. } = ev {
                touching[*car] = true;
            }
        }
        for i in 0..n {
            if touching[i] {
                if off_wall[i] >= gap_ticks && !results[i].finished {
                    results[i].wall_contacts += 1;
                }
                off_wall[i] = 0;
            } else {
                off_wall[i] = off_wall[i].saturating_add(1);
            }
        }
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
        r.race_ticks = ticks;
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

/// Builds the car of a circuit group.
pub type CarFn = fn() -> CarConfig;

/// (circuit id, Classic car of its group). Each Classic circuit group adds its circuits here.
pub const CLASSIC_CIRCUIT_CARS: [(&str, CarFn); 18] = [
    ("kart_pine_grove", ClassicGameModule::car_classic_kart),
    ("kart_riverbend_circuit", ClassicGameModule::car_classic_kart),
    ("kart_summit_international", ClassicGameModule::car_classic_kart),
    ("rx_quarry_sprint", ClassicGameModule::car_classic_rally),
    ("rx_hilltop_leap", ClassicGameModule::car_classic_rally),
    ("rx_canyon_flyer", ClassicGameModule::car_classic_rally),
    ("ax_meadow_sprint", ClassicGameModule::car_classic_ax_mudlark),
    ("ax_clay_bowl", ClassicGameModule::car_classic_ax_brawler),
    ("ax_hillside_hammer", ClassicGameModule::car_classic_ax_mudlark),
    ("gt_velocity_park", ClassicGameModule::car_classic_gt),
    ("gt_ridge_ring", ClassicGameModule::car_classic_gt),
    ("gt_coastal_grand_prix", ClassicGameModule::car_classic_gt),
    ("stock_thunder_bowl", ClassicGameModule::car_classic_nascar),
    ("stock_tri_oval_speedway", ClassicGameModule::car_classic_nascar),
    ("stock_roval", ClassicGameModule::car_classic_nascar),
    ("at_dune_sea", ClassicGameModule::car_classic_offroad),
    ("at_mudbath_valley", ClassicGameModule::car_classic_offroad),
    ("at_frostbite_pass", ClassicGameModule::car_classic_offroad),
];

/// One circuit of a Steam launch module with its own-category car (spec 083:
/// `specs/083_evorl_bot_driving_style_and_performance_optimization.md`).
#[derive(Debug, Clone, Copy)]
pub struct LaunchCircuit {
    pub module: &'static str,
    pub id: &'static str,
    pub car: CarConfig,
}

/// Every official circuit of the launch modules (Classic, Karting, Autocross, Rallycross).
/// Classic circuits use the car of their group (`CLASSIC_CIRCUIT_CARS`); the other modules use
/// their default vehicle.
pub fn launch_circuit_cars() -> Vec<LaunchCircuit> {
    fn default_car(module: &dyn GameModule) -> CarConfig {
        let id = module.default_vehicle_id();
        module.vehicles().into_iter().find(|v| v.id == id).expect("module default vehicle must exist").config
    }
    let mut out: Vec<LaunchCircuit> = tdrace_core::catalog::module_circuits("classic")
        .map(|c| {
            let car = CLASSIC_CIRCUIT_CARS.iter().find(|(id, _)| *id == c.id).unwrap_or_else(|| panic!("classic circuit '{}' has no car", c.id)).1;
            LaunchCircuit { module: "classic", id: c.id, car: car() }
        })
        .collect();
    let modules: [(&'static str, &dyn GameModule); 3] = [("kart", &KartGameModule), ("autocross", &AutocrossGameModule), ("rally", &RallyGameModule)];
    for (module, game_module) in modules {
        let car = default_car(game_module);
        out.extend(tdrace_core::catalog::module_circuits(module).map(|c| LaunchCircuit { module, id: c.id, car }));
    }
    out
}

/// Candidate bot settings from offline calibration (spec 083). `None` keeps the current constant.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotGenome {
    /// Stage A: the Balanced style tuning. Other styles keep their offsets from Balanced.
    pub lookahead_time: Option<f32>,
    pub steering_kp: Option<f32>,
    pub steering_kd: Option<f32>,
    pub brake_margin: Option<f32>,
    pub speed_mult: Option<f32>,
    /// Stage B: the quality of the bot's own tier.
    pub pace_limit: Option<f32>,
    pub brake_padding: Option<f32>,
    pub consistency: Option<f32>,
    pub composure: Option<f32>,
}

/// Search box of each `BotGenome` value (spec 083, "What gets tuned"). Values are clamped into it.
pub const GENOME_BOUNDS: [(&str, f32, f32); 9] = [
    ("lookahead_time", 0.20, 0.55),
    ("steering_kp", 1.50, 3.50),
    ("steering_kd", 0.03, 0.14),
    ("brake_margin", 0.80, 1.30),
    ("speed_mult", 0.94, 1.10),
    ("pace_limit", 0.70, 1.15),
    ("brake_padding", -0.15, 0.30),
    ("consistency", 0.40, 1.00),
    ("composure", 0.30, 1.00),
];

impl BotGenome {
    fn values(&self) -> [Option<f32>; 9] {
        [
            self.lookahead_time,
            self.steering_kp,
            self.steering_kd,
            self.brake_margin,
            self.speed_mult,
            self.pace_limit,
            self.brake_padding,
            self.consistency,
            self.composure,
        ]
    }

    /// The bot profile of `style` at `tier` with this genome applied. Fails on a non-finite value.
    pub fn profile(&self, style: DrivingStyle, tier: DriverTier) -> Result<BotProfile, String> {
        let mut v = self.values();
        for (x, (name, lo, hi)) in v.iter_mut().zip(GENOME_BOUNDS) {
            if let Some(x) = x {
                if !x.is_finite() {
                    return Err(format!("genome value '{name}' is not finite"));
                }
                *x = x.clamp(lo, hi);
            }
        }
        let [lookahead, kp, kd, brake, speed_mult, pace_limit, brake_padding, consistency, composure] = v;
        // A Stage A value moves every style by its change from the Balanced constant.
        let base = StyleTuning::for_style(DrivingStyle::Balanced);
        let mut tuning = StyleTuning::for_style(style);
        let shift = |t: &mut f32, genome: Option<f32>, balanced: f32| {
            if let Some(g) = genome {
                *t += g - balanced;
            }
        };
        shift(&mut tuning.lookahead_time, lookahead, base.lookahead_time);
        shift(&mut tuning.steering_kp, kp, base.steering_kp);
        shift(&mut tuning.steering_kd, kd, base.steering_kd);
        shift(&mut tuning.brake_margin, brake, base.brake_margin);
        shift(&mut tuning.speed_mult, speed_mult, base.speed_mult);
        let mut quality = DriverQuality::for_tier(tier);
        quality.pace_limit = pace_limit.unwrap_or(quality.pace_limit);
        quality.brake_padding = brake_padding.unwrap_or(quality.brake_padding);
        quality.consistency = consistency.unwrap_or(quality.consistency);
        quality.composure = composure.unwrap_or(quality.composure);
        Ok(BotProfile::from_tuning_and_quality(style, &tuning, &quality))
    }
}

/// One driver in an `EvalRequest`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalDriver {
    /// `DrivingStyle` name, lower case (e.g. "balanced").
    pub style: String,
    /// `DriverTier` number, 1 (Rookie) to 5 (Legend).
    pub tier: u8,
    #[serde(default)]
    pub genome: BotGenome,
}

fn default_max_time_s() -> f32 {
    600.0
}

/// One race for `bot_eval_worker` (spec 083).
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalRequest {
    /// "module/id", e.g. "kart/lonato".
    pub circuit: String,
    /// Car id (a module vehicle or a Classic arcade car). Default: the circuit's launch car.
    #[serde(default)]
    pub car: Option<String>,
    pub laps: u32,
    pub seed: u64,
    pub subject: EvalDriver,
    #[serde(default)]
    pub opponents: Vec<EvalDriver>,
    /// Grid slot of the subject; opponents fill the other slots in order.
    #[serde(default)]
    pub subject_slot: usize,
    #[serde(default = "default_max_time_s")]
    pub max_time_s: f32,
}

/// One opponent's outcome in an `EvalResult`.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct EvalOpponent {
    pub rank: usize,
    pub finished: bool,
    pub mean_flying_lap: Option<f32>,
    pub spins: u32,
    pub offs: u32,
    pub mistakes: u32,
    pub wall_contacts: u32,
    pub longest_no_progress_s: f32,
}

/// Outcome of an `EvalRequest`. Every field but `error` and `wall_time_ms` is deterministic.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct EvalResult {
    /// Set when the request could not run; the other fields are then empty.
    pub error: Option<String>,
    /// 1 = winner. Cars that tie share a rank.
    pub subject_rank: usize,
    pub finished: bool,
    pub dnf: bool,
    pub lap_times: Vec<f32>,
    pub mean_flying_lap: Option<f32>,
    pub best_lap: Option<f32>,
    pub spins: u32,
    pub offs: u32,
    pub mistakes: u32,
    pub wall_contacts: u32,
    pub longest_no_progress_s: f32,
    pub controls_hash: u64,
    /// Physics ticks the race ran.
    pub steps: u32,
    /// Ticks times cars.
    pub car_steps: u64,
    /// The circuit is a closed spline loop (a lap race), not an arena or an open layout.
    pub closed_loop: bool,
    pub wall_time_ms: f64,
    pub opponents: Vec<EvalOpponent>,
}

impl EvalResult {
    pub fn error(message: String) -> Self {
        Self { error: Some(message), ..Default::default() }
    }
}

/// A car by id: a launch module vehicle or a Classic arcade car.
pub fn launch_car_by_id(id: &str) -> Option<CarConfig> {
    let modules: [&dyn GameModule; 4] = [&ClassicGameModule, &KartGameModule, &AutocrossGameModule, &RallyGameModule];
    modules
        .iter()
        .flat_map(|m| m.vehicles())
        .find(|v| v.id == id)
        .map(|v| v.config)
        .or_else(|| CLASSIC_ARCADE_CARS.iter().find(|c| c.id == id).map(|c| c.to_car_config()))
}

fn parse_driver(d: &EvalDriver) -> Result<(DrivingStyle, DriverTier), String> {
    let style = DrivingStyle::ALL
        .into_iter()
        .find(|s| format!("{s:?}").eq_ignore_ascii_case(&d.style))
        .ok_or_else(|| format!("unknown style '{}'", d.style))?;
    let tier = match d.tier {
        1 => DriverTier::Rookie,
        2 => DriverTier::Amateur,
        3 => DriverTier::Contender,
        4 => DriverTier::Pro,
        5 => DriverTier::Legend,
        t => return Err(format!("unknown tier {t}")),
    };
    Ok((style, tier))
}

/// Seed of driver `index` (0 = subject) in a request. It does not depend on the grid slot, so a
/// seat swap keeps every driver's random stream (common random numbers).
pub fn eval_driver_seed(seed: u64, index: usize) -> u64 {
    seed ^ (index as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15)
}

/// Runs one `EvalRequest` through `run_harness_race`. Never panics on bad input.
pub fn eval_race(req: &EvalRequest) -> EvalResult {
    let started = std::time::Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| try_eval_race(req)));
    let mut result = match outcome {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => EvalResult::error(e),
        Err(_) => EvalResult::error("race panicked".into()),
    };
    result.wall_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    result
}

fn try_eval_race(req: &EvalRequest) -> Result<EvalResult, String> {
    let (module, id) = req.circuit.split_once('/').ok_or_else(|| format!("circuit '{}' is not 'module/id'", req.circuit))?;
    let circuit = tdrace_core::catalog::find(id, Some(module))
        .filter(|c| c.module == module)
        .ok_or_else(|| format!("unknown circuit '{}'", req.circuit))?;
    let car = match &req.car {
        Some(car_id) => launch_car_by_id(car_id).ok_or_else(|| format!("unknown car '{car_id}'"))?,
        None => launch_circuit_cars()
            .into_iter()
            .find(|c| c.module == module && c.id == circuit.id)
            .ok_or_else(|| format!("circuit '{}' has no launch car; pass 'car'", req.circuit))?
            .car,
    };
    if req.laps == 0 || !(req.max_time_s.is_finite() && req.max_time_s > 0.0) {
        return Err("laps and max_time_s must be positive".into());
    }
    if req.subject_slot > req.opponents.len() {
        return Err(format!("subject_slot {} is past the grid of {}", req.subject_slot, req.opponents.len() + 1));
    }
    let track = circuit.load().map_err(|e| format!("circuit '{}': {e}", req.circuit))?;
    let n = req.opponents.len() + 1;
    if n > track.grid_positions.len() {
        return Err(format!("{n} cars but the circuit has {} grid positions", track.grid_positions.len()));
    }

    // Driver 0 is the subject; grid order puts it at `subject_slot`.
    let mut drivers = Vec::with_capacity(n);
    for (index, d) in std::iter::once(&req.subject).chain(&req.opponents).enumerate() {
        let (style, tier) = parse_driver(d)?;
        let profile = d.genome.profile(style, tier)?;
        drivers.push(BotAiDriver::with_seed(profile, eval_driver_seed(req.seed, index)));
    }
    let subject = drivers.remove(0);
    drivers.insert(req.subject_slot, subject);
    let entries = drivers.into_iter().map(|d| HarnessEntry::bot(d, car)).collect();
    let results = run_harness_race(&track, entries, req.laps, req.max_time_s);

    let finish_key = |r: &HarnessCarResult| (!r.finished, std::cmp::Reverse(r.lap_times.len()), r.lap_times.iter().sum::<f32>());
    let rank = |i: usize| 1 + results.iter().filter(|r| finish_key(r) < finish_key(&results[i])).count();
    let s = &results[req.subject_slot];
    let out = EvalResult {
        error: None,
        subject_rank: rank(req.subject_slot),
        finished: s.finished,
        dnf: !s.finished,
        lap_times: s.lap_times.clone(),
        mean_flying_lap: s.mean_flying_lap(),
        best_lap: s.lap_times.get(1..).and_then(|f| f.iter().copied().reduce(f32::min)),
        spins: s.stats.spins,
        offs: s.stats.offs,
        mistakes: s.stats.total_mistakes(),
        wall_contacts: s.wall_contacts,
        longest_no_progress_s: s.longest_no_progress_s,
        controls_hash: s.controls_hash,
        steps: s.race_ticks,
        car_steps: s.race_ticks as u64 * n as u64,
        closed_loop: matches!(track.kind, tdrace_core::track::TrackKind::Circuit) && track.spline.closed,
        wall_time_ms: 0.0,
        opponents: (0..n)
            .filter(|&i| i != req.subject_slot)
            .map(|i| {
                let r = &results[i];
                EvalOpponent {
                    rank: rank(i),
                    finished: r.finished,
                    mean_flying_lap: r.mean_flying_lap(),
                    spins: r.stats.spins,
                    offs: r.stats.offs,
                    mistakes: r.stats.total_mistakes(),
                    wall_contacts: r.wall_contacts,
                    longest_no_progress_s: r.longest_no_progress_s,
                }
            })
            .collect(),
    };
    let floats = out.lap_times.iter().copied().chain([out.longest_no_progress_s]).chain(out.mean_flying_lap).chain(out.best_lap);
    if floats.into_iter().any(|x| !x.is_finite()) {
        return Err("race produced a non-finite value".into());
    }
    Ok(out)
}
