//! Human driver layer for bots (spec 046:
//! `specs/046_humanlike_bot_driving_with_tiered_mistakes_and_varied_lines.md`).
//!
//! Adds seeded variation and mistakes on top of the `BotAiDriver` controller. The tier sets how
//! much a bot varies and how often it makes a mistake; the driving style sets its line and which
//! kind of mistake it makes. `HumanTraits::none()` turns the layer off, and the controller then
//! drives exactly as it did before spec 046.

use tdrace_core::physics::car::{normalize_angle, Car};
use tdrace_core::track::spline::{SplineProjection, TrackSpline};

use super::driver::LcgRng;
use super::{DriverQuality, DrivingStyle};

/// Curvature (1/m) above which the track counts as a corner (radius below 250 m).
const CORNER_CURVATURE: f32 = 1.0 / 250.0;
/// Shortest corner kept (m).
const MIN_CORNER_LEN: f32 = 10.0;
/// Corner detection step and curvature span (m), as in the braking scan.
const CORNER_STEP: f32 = 5.0;
const CURVATURE_SPAN: f32 = 10.0;
/// A corner plan is drawn when the bot comes within this distance of the corner start (m).
const APPROACH: f32 = 120.0;
/// Longest wide entry and exit ramp of a corner line (m). A ramp is at most half the gap to the
/// neighbouring corner.
const LINE_RAMP: f32 = 40.0;
/// Margin kept to the track edge by the line offset (m).
const EDGE_MARGIN: f32 = 1.5;
/// Line wander time constant (s).
const WANDER_TAU: f32 = 3.0;
/// Time constant of the line-offset smoothing (s).
const LINE_SMOOTH_TAU: f32 = 0.5;
/// A brake-point shift acts fully at this distance before a corner point, and fades to 0 at it (m).
const BRAKE_SHIFT_RAMP: f32 = 20.0;
/// Pressure: cars within this distance behind or alongside, or within `PRESSURE_AHEAD` ahead.
const PRESSURE_BEHIND: f32 = 12.0;
const PRESSURE_AHEAD: f32 = 8.0;
/// Most nearby cars counted for pressure.
const MAX_PRESSURE_CARS: u32 = 2;
/// An "off" is more than this far past the track edge (m), so that kerb cuts do not count.
const OFF_MARGIN: f32 = 1.5;
/// A bot counts as launched once it first reaches this speed (m/s), the top of the 6 m/s
/// low-speed regime of the throttle cap. Corner-exit mistakes (PowerStab, the Cautious lift) wait
/// for it: on a grid just past an apex, a PowerStab's handbrake locked a cadet kart at ~1 m/s for
/// ~2 s. After the launch they fire as calibrated, low-speed hairpin exits included.
const LAUNCH_SPEED: f32 = 6.0;

/// The kinds of mistake a bot can make (spec 046 §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MistakeKind {
    /// Over-estimates the car's braking power by 30–60 % and arrives at the grip limit.
    LateBrake,
    /// Enters the corner 8–25 % above the car's grip limit.
    Overdrive,
    /// Loses the rear at the exit: handbrake, full throttle and full lock for 0.6–1.1 s. The
    /// handbrake bypasses TCS and ESC, as it does for a human driver; without it the Arcade
    /// assists catch every slide.
    PowerStab,
    /// Over-corrects in the corner (steer gain x 1.6 for 0.5 s) on the first slide or at the apex.
    OverCorrect,
    /// Brakes 5–15 m early and slow, or lifts at the apex.
    Cautious,
}

impl MistakeKind {
    pub const ALL: [Self; 5] = [Self::LateBrake, Self::Overdrive, Self::PowerStab, Self::OverCorrect, Self::Cautious];

    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Tier- and style-derived human behaviour settings (spec 046 §3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HumanTraits {
    /// Standard deviation of the wandering line offset (m).
    pub line_sigma_m: f32,
    /// Mean share of the free half-width used to go wide on corner entry and exit.
    pub entry_share: f32,
    /// Mean share of the free half-width used to clip the apex.
    pub apex_share: f32,
    /// Standard deviation of the per-corner line shares.
    pub line_choice_sigma: f32,
    /// Standard deviation of the per-corner brake-point shift (m).
    pub brake_shift_sigma_m: f32,
    /// Standard deviation of the per-corner corner-speed factor.
    pub corner_speed_sigma: f32,
    /// Steering reaction lag time constant (s).
    pub reaction_tau_s: f32,
    /// Mistake probability per corner in clean air.
    pub mistake_rate: f32,
    /// Relative weights of `MistakeKind::ALL`.
    pub mistake_weights: [f32; 5],
    /// Mistake-rate increase per nearby car: `p_gain * (1 - composure)`.
    pub pressure_gain: f32,
    /// Longest delay before a bot commits to a pass (s).
    pub max_pass_delay_s: f32,
}

impl HumanTraits {
    /// No variation and no mistakes: the pre-046 controller.
    pub const fn none() -> Self {
        Self {
            line_sigma_m: 0.0,
            entry_share: 0.0,
            apex_share: 0.0,
            line_choice_sigma: 0.0,
            brake_shift_sigma_m: 0.0,
            corner_speed_sigma: 0.0,
            reaction_tau_s: 0.0,
            mistake_rate: 0.0,
            mistake_weights: [0.0; 5],
            pressure_gain: 0.0,
            max_pass_delay_s: 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        *self != Self::none()
    }

    /// Traits of a style at a tier. `aggression` is the style's overtaking aggression.
    pub fn for_style_and_quality(style: DrivingStyle, quality: &DriverQuality, aggression: f32) -> Self {
        let loose = 1.0 - quality.consistency;
        let nerves = 1.0 - quality.composure;
        // (line factor, entry share, apex share, style rate, mistake weights, pressure gain)
        let (line_factor, entry, apex, style_rate, weights, p_gain) = match style {
            DrivingStyle::Smooth => (0.7, 0.6, 0.5, 0.7, [0.10, 0.15, 0.05, 0.10, 0.60], 2.0),
            DrivingStyle::Aggressive => (1.2, 0.3, 0.7, 1.3, [0.35, 0.25, 0.20, 0.10, 0.10], 2.0),
            DrivingStyle::Tenacious => (1.0, 0.4, 0.5, 0.8, [0.40, 0.15, 0.10, 0.15, 0.20], 4.0),
            DrivingStyle::Calculating => (0.8, 0.6, 0.6, 0.6, [0.10, 0.10, 0.05, 0.05, 0.70], 1.0),
            DrivingStyle::Bold => (1.3, 0.4, 0.7, 1.3, [0.20, 0.30, 0.30, 0.15, 0.05], 2.0),
            DrivingStyle::Balanced => (1.0, 0.5, 0.5, 1.0, [0.20, 0.20, 0.15, 0.15, 0.30], 2.0),
        };
        Self {
            line_sigma_m: (0.3 + 3.5 * loose) * line_factor,
            entry_share: entry,
            apex_share: apex,
            line_choice_sigma: 0.05 + 0.4 * loose,
            brake_shift_sigma_m: 1.0 + 20.0 * loose,
            corner_speed_sigma: 0.005 + 0.06 * loose,
            reaction_tau_s: 0.04 + 0.25 * loose,
            mistake_rate: 0.30 * nerves.powf(1.5) * style_rate,
            mistake_weights: weights,
            pressure_gain: p_gain * nerves,
            max_pass_delay_s: 0.6 * (1.0 - aggression).max(0.0),
        }
    }
}

/// Counters of what a bot did, for tests and the benchmark. The game does not read them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BotDrivingStats {
    /// Corner plans drawn.
    pub corners: u32,
    /// Corner plans drawn with at least one car close by.
    pub corners_under_pressure: u32,
    /// Mistakes by `MistakeKind::index`.
    pub mistakes: [u32; 5],
    /// Mistakes drawn with at least one car close by.
    pub mistakes_under_pressure: u32,
    /// Heading more than 90 deg away from the track direction.
    pub spins: u32,
    /// More than `OFF_MARGIN` past the track edge for more than 1 s.
    pub offs: u32,
    /// Total time more than `OFF_MARGIN` past the track edge (s).
    pub off_track_s: f32,
    /// Largest angle between the car heading and the track direction (deg).
    pub peak_heading_off_deg: f32,
}

impl BotDrivingStats {
    pub fn total_mistakes(&self) -> u32 {
        self.mistakes.iter().sum()
    }
}

/// What a corner plan changes in the braking scan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerModifiers {
    /// Multiplies the controller's corner speed.
    pub speed_mult: f32,
    /// Brake-point shift (m, + = later).
    pub brake_shift_m: f32,
    /// Corner speed as a multiple of the car's grip-limit speed (0 = not used).
    pub over_limit: f32,
    /// Assumed braking deceleration as a multiple of the car's grip (0 = not used).
    pub brake_decel_mult: f32,
}

impl CornerModifiers {
    pub const NEUTRAL: Self = Self { speed_mult: 1.0, brake_shift_m: 0.0, over_limit: 0.0, brake_decel_mult: 0.0 };
}

/// A corner found on the spline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    /// Track distance of the corner start (m).
    pub start: f32,
    /// Distance from the start to the apex (m).
    pub apex: f32,
    /// Corner length (m).
    pub len: f32,
    /// +1 = left turn (towards the spline normal), -1 = right turn.
    pub turn: f32,
    /// Length of the wide entry ramp before the start (m).
    pub ramp_in: f32,
    /// Length of the wide exit ramp after the end (m).
    pub ramp_out: f32,
}

/// Distance from `from` forward to `to` on a closed track of length `len`.
fn ahead(from: f32, to: f32, len: f32) -> f32 {
    (to - from).rem_euclid(len)
}

fn signed_curvature(spline: &TrackSpline, s: f32) -> f32 {
    let len = spline.total_length();
    let t0 = spline.sample_at_distance(s.rem_euclid(len)).tangent;
    let t1 = spline.sample_at_distance((s + CURVATURE_SPAN).rem_euclid(len)).tangent;
    normalize_angle(t1.y.atan2(t1.x) - t0.y.atan2(t0.x)) / CURVATURE_SPAN
}

/// Finds the corners of a closed track: runs of one turn direction whose curvature is above
/// `CORNER_CURVATURE`.
pub fn find_corners(spline: &TrackSpline) -> Vec<Corner> {
    let len = spline.total_length();
    if len <= 0.0 || spline.samples.is_empty() {
        return Vec::new();
    }
    let n = (len / CORNER_STEP).floor().max(1.0) as usize;
    let curv: Vec<f32> = (0..n).map(|i| signed_curvature(spline, i as f32 * CORNER_STEP)).collect();
    let class = |k: f32| if k > CORNER_CURVATURE { 1 } else if k < -CORNER_CURVATURE { -1 } else { 0 };
    // Start the scan on a straight so that no corner is split by the wrap.
    let Some(origin) = (0..n).find(|&i| class(curv[i]) == 0) else {
        return Vec::new();
    };
    let mut corners = Vec::new();
    let mut i = 0;
    while i < n {
        let idx = (origin + i) % n;
        let c = class(curv[idx]);
        if c == 0 {
            i += 1;
            continue;
        }
        let first = i;
        let (mut apex_i, mut apex_k) = (i, curv[idx].abs());
        while i < n && class(curv[(origin + i) % n]) == c {
            let k = curv[(origin + i) % n].abs();
            if k > apex_k {
                apex_i = i;
                apex_k = k;
            }
            i += 1;
        }
        let corner_len = (i - first) as f32 * CORNER_STEP;
        if corner_len >= MIN_CORNER_LEN {
            let start = ((origin + first) % n) as f32 * CORNER_STEP;
            corners.push(Corner {
                start,
                apex: (apex_i - first) as f32 * CORNER_STEP,
                len: corner_len,
                turn: c as f32,
                ramp_in: LINE_RAMP,
                ramp_out: LINE_RAMP,
            });
        }
    }
    let count = corners.len();
    for i in 0..count {
        let next = corners[(i + 1) % count];
        let gap = ahead(corners[i].start + corners[i].len, next.start, len);
        let ramp = (0.5 * gap).min(LINE_RAMP);
        corners[i].ramp_out = ramp;
        corners[(i + 1) % count].ramp_in = ramp;
    }
    corners
}

/// What a bot plans for one pass through one corner.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CornerPlan {
    entry_share: f32,
    apex_share: f32,
    brake_shift_m: f32,
    speed_mult: f32,
    /// Corner speed as a multiple of the car's grip-limit speed (0 = not used).
    over_limit: f32,
    /// Assumed braking deceleration as a multiple of the car's grip (0 = not used).
    brake_decel_mult: f32,
    mistake: Option<MistakeKind>,
    /// Cautious mistake variant: lift at the apex for this long (s).
    lift_s: f32,
    /// PowerStab duration (s).
    stab_s: f32,
    /// The event of this plan (stab, lift or over-correction) has run.
    event_done: bool,
    in_window: bool,
}

impl CornerPlan {
    const NEUTRAL: Self = Self {
        entry_share: 0.0,
        apex_share: 0.0,
        brake_shift_m: 0.0,
        speed_mult: 1.0,
        over_limit: 0.0,
        brake_decel_mult: 0.0,
        mistake: None,
        lift_s: 0.0,
        stab_s: 0.0,
        event_done: false,
        in_window: false,
    };
}

/// A timed pedal or steering override.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Event {
    None,
    /// (time left, held steer)
    PowerStab(f32, f32),
    Lift(f32),
    OverCorrect(f32),
}

/// Per-bot human layer state.
#[derive(Debug, Clone)]
pub struct HumanDriver {
    pub traits: HumanTraits,
    active: bool,
    rng: LcgRng,
    corners: Vec<Corner>,
    corners_for_len: f32,
    plans: Vec<CornerPlan>,
    /// Index of the corner whose window holds the bot, if any.
    current: Option<usize>,
    wander_m: f32,
    line_m: f32,
    steer_out: f32,
    has_steer: bool,
    event: Event,
    pass_pref: f32,
    pass_delay_s: f32,
    pass_timer: f32,
    spin_latched: bool,
    off_timer: f32,
    /// Set once the bot first reaches `LAUNCH_SPEED` in this race.
    launched: bool,
    pub stats: BotDrivingStats,
}

impl HumanDriver {
    pub fn new(traits: HumanTraits, seed: u64) -> Self {
        Self {
            traits,
            active: traits.is_active(),
            rng: LcgRng::new(seed),
            corners: Vec::new(),
            corners_for_len: -1.0,
            plans: Vec::new(),
            current: None,
            wander_m: 0.0,
            line_m: 0.0,
            steer_out: 0.0,
            has_steer: false,
            event: Event::None,
            pass_pref: 0.0,
            pass_delay_s: 0.0,
            pass_timer: 0.0,
            spin_latched: false,
            off_timer: 0.0,
            launched: false,
            stats: BotDrivingStats::default(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn uniform(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.rng.next_f32()
    }

    /// Standard normal value (Box-Muller).
    fn normal(&mut self) -> f32 {
        let u1 = self.rng.next_f32().max(1e-7);
        let u2 = self.rng.next_f32();
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }

    fn pick_mistake(&mut self) -> MistakeKind {
        let w = self.traits.mistake_weights;
        let total: f32 = w.iter().sum();
        let mut r = self.rng.next_f32() * total;
        for kind in MistakeKind::ALL {
            r -= w[kind.index()];
            if r <= 0.0 {
                return kind;
            }
        }
        MistakeKind::Cautious
    }

    fn draw_plan(&mut self, pressure_cars: u32) -> CornerPlan {
        let t = self.traits;
        let mut plan = CornerPlan {
            entry_share: (t.entry_share + t.line_choice_sigma * self.normal()).clamp(0.0, 1.0),
            apex_share: (t.apex_share + t.line_choice_sigma * self.normal()).clamp(0.0, 1.0),
            brake_shift_m: t.brake_shift_sigma_m * self.normal(),
            speed_mult: (1.0 + t.corner_speed_sigma * self.normal()).clamp(0.85, 1.15),
            in_window: true,
            ..CornerPlan::NEUTRAL
        };
        self.pass_pref = if self.rng.next_f32() < 0.5 { 1.0 } else { -1.0 };
        self.pass_delay_s = self.uniform(0.0, t.max_pass_delay_s);

        self.stats.corners += 1;
        if pressure_cars > 0 {
            self.stats.corners_under_pressure += 1;
        }
        let p = t.mistake_rate * (1.0 + t.pressure_gain * pressure_cars as f32);
        if self.rng.next_f32() < p {
            let kind = self.pick_mistake();
            match kind {
                MistakeKind::LateBrake => {
                    plan.over_limit = self.uniform(1.0, 1.08);
                    plan.brake_decel_mult = self.uniform(1.3, 1.6);
                }
                MistakeKind::Overdrive => plan.over_limit = self.uniform(1.08, 1.25),
                MistakeKind::PowerStab => plan.stab_s = self.uniform(0.6, 1.1),
                MistakeKind::OverCorrect => {}
                MistakeKind::Cautious => {
                    if self.rng.next_f32() < 0.5 {
                        plan.brake_shift_m -= self.uniform(5.0, 15.0);
                        plan.speed_mult *= self.uniform(0.93, 0.97);
                    } else {
                        plan.lift_s = self.uniform(0.3, 0.6);
                    }
                }
            }
            plan.mistake = Some(kind);
            self.stats.mistakes[kind.index()] += 1;
            if pressure_cars > 0 {
                self.stats.mistakes_under_pressure += 1;
            }
        }
        plan
    }

    /// Updates statistics, line wander and corner plans. Call once per tick before the
    /// controller reads any other hook.
    pub fn begin_tick(&mut self, car: &Car, spline: &TrackSpline, proj: &SplineProjection, others: &[&Car], dt: f32) {
        let len = spline.total_length();
        let curr_dist = proj.progress_distance;
        let track_dir = spline.sample_at_distance(curr_dist).tangent;
        let heading_off = car.forward_vector().dot(track_dir);
        if car.state.speed > 1.0 {
            self.stats.peak_heading_off_deg = self.stats.peak_heading_off_deg.max(heading_off.clamp(-1.0, 1.0).acos().to_degrees());
        }
        if !self.spin_latched && heading_off < 0.0 && car.state.speed > 1.0 {
            self.spin_latched = true;
            self.stats.spins += 1;
        } else if self.spin_latched && heading_off > 0.866 {
            self.spin_latched = false;
        }
        if proj.lateral_offset.abs() <= proj.track_width * 0.5 + OFF_MARGIN {
            self.off_timer = 0.0;
        } else {
            if self.off_timer <= 1.0 && self.off_timer + dt > 1.0 {
                self.stats.offs += 1;
            }
            self.off_timer += dt;
            self.stats.off_track_s += dt;
        }

        if !self.active || len <= 0.0 {
            return;
        }
        if self.corners_for_len != len {
            self.corners = find_corners(spline);
            self.plans = vec![CornerPlan::NEUTRAL; self.corners.len()];
            self.corners_for_len = len;
        }

        let decay = (-dt / WANDER_TAU).exp();
        self.wander_m = self.wander_m * decay + self.traits.line_sigma_m * (1.0 - decay * decay).sqrt() * self.normal();

        let pressure_cars = self.pressure_cars(car, others);
        self.current = None;
        for i in 0..self.corners.len() {
            let c = self.corners[i];
            let window = APPROACH + c.len + LINE_RAMP;
            let in_window = ahead(c.start - APPROACH, curr_dist, len) < window;
            if in_window && !self.plans[i].in_window {
                self.plans[i] = self.draw_plan(pressure_cars);
            }
            self.plans[i].in_window = in_window;
            if in_window && ahead(c.start - APPROACH, curr_dist, len) >= APPROACH - BRAKE_SHIFT_RAMP {
                self.current = Some(i);
            }
        }

        self.launched |= car.state.speed >= LAUNCH_SPEED;
        self.event = match self.event {
            Event::PowerStab(t, steer) if t > dt => Event::PowerStab(t - dt, steer),
            Event::Lift(t) if t > dt => Event::Lift(t - dt),
            Event::OverCorrect(t) if t > dt => Event::OverCorrect(t - dt),
            _ => Event::None,
        };
        if let (Event::None, Some(i)) = (self.event, self.current) {
            let c = self.corners[i];
            let into = ahead(c.start, curr_dist, len);
            let plan = &mut self.plans[i];
            if !plan.event_done && into < c.len + LINE_RAMP {
                let past_apex = into >= c.apex;
                let v = car.state.velocity;
                let slip = if car.state.speed > 8.0 { v.dot(car.right_vector()).atan2(v.dot(car.forward_vector())).abs() } else { 0.0 };
                let event = match plan.mistake {
                    Some(MistakeKind::PowerStab) if past_apex && self.launched => Some(Event::PowerStab(plan.stab_s, -c.turn)),
                    Some(MistakeKind::Cautious) if past_apex && self.launched && plan.lift_s > 0.0 => Some(Event::Lift(plan.lift_s)),
                    Some(MistakeKind::OverCorrect) if past_apex || slip > 5f32.to_radians() => Some(Event::OverCorrect(0.5)),
                    _ => None,
                };
                if let Some(e) = event {
                    self.event = e;
                    plan.event_done = true;
                }
            }
        }
    }

    fn pressure_cars(&self, car: &Car, others: &[&Car]) -> u32 {
        let fwd = car.forward_vector();
        others
            .iter()
            .filter(|o| {
                let to = o.state.position - car.state.position;
                let along = to.dot(fwd);
                let dist = to.length();
                (dist < PRESSURE_BEHIND && along <= 2.5) || (along > 0.0 && along < PRESSURE_AHEAD && to.dot(car.right_vector()).abs() < 3.0)
            })
            .count()
            .min(MAX_PRESSURE_CARS as usize) as u32
    }

    /// Lateral offset (m, along the spline normal) of the bot's line at track distance `s`.
    pub fn line_offset(&mut self, spline: &TrackSpline, s: f32, width: f32, dt: f32) -> f32 {
        if !self.active {
            return 0.0;
        }
        let len = spline.total_length();
        let free = (width * 0.5 - EDGE_MARGIN).max(0.0);
        let mut shape = 0.0;
        for (c, p) in self.corners.iter().zip(&self.plans) {
            if !p.in_window {
                continue;
            }
            let x = ahead(c.start - c.ramp_in, s, len) - c.ramp_in;
            let outside = -c.turn * p.entry_share * free;
            let inside = c.turn * p.apex_share * free;
            shape += if x < -c.ramp_in || x > c.len + c.ramp_out {
                0.0
            } else if x < 0.0 {
                outside * (x + c.ramp_in) / c.ramp_in.max(1.0)
            } else if x < c.apex {
                outside + (inside - outside) * x / c.apex.max(1.0)
            } else if x < c.len {
                inside + (outside - inside) * (x - c.apex) / (c.len - c.apex).max(1.0)
            } else {
                outside * (1.0 - (x - c.len) / c.ramp_out.max(1.0))
            };
        }
        let target = (self.wander_m + shape).clamp(-free, free);
        self.line_m += (target - self.line_m) * (1.0 - (-dt / LINE_SMOOTH_TAU).exp());
        self.line_m
    }

    /// Braking-scan modifiers of the corner plan at `scan_dist`.
    pub fn corner_modifiers(&self, scan_dist: f32, len: f32) -> CornerModifiers {
        if self.active {
            for (c, p) in self.corners.iter().zip(&self.plans) {
                if p.in_window && ahead(c.start, scan_dist, len) < c.len {
                    return CornerModifiers {
                        speed_mult: p.speed_mult,
                        brake_shift_m: p.brake_shift_m,
                        over_limit: p.over_limit,
                        brake_decel_mult: p.brake_decel_mult,
                    };
                }
            }
        }
        CornerModifiers::NEUTRAL
    }

    /// Braking distance with the brake-point shift applied: full shift from
    /// `BRAKE_SHIFT_RAMP` out, fading to none at the corner point itself.
    pub fn shifted_distance(dist_ahead: f32, shift_m: f32) -> f32 {
        (dist_ahead + shift_m * (dist_ahead / BRAKE_SHIFT_RAMP).min(1.0)).max(0.0)
    }

    /// Pass side for a car nearly straight ahead: the bot's own choice for this corner.
    pub fn pass_side(&self, natural: f32, opp_lateral: f32) -> f32 {
        if self.active && opp_lateral.abs() < 1.0 {
            self.pass_pref
        } else {
            natural
        }
    }

    /// Share of the pass steering the bot commits to now.
    pub fn pass_commit(&self) -> f32 {
        if self.active && self.pass_delay_s > 0.0 {
            (self.pass_timer / self.pass_delay_s).min(1.0)
        } else {
            1.0
        }
    }

    pub fn update_pass_timer(&mut self, had_pass_target: bool, dt: f32) {
        self.pass_timer = if had_pass_target { self.pass_timer + dt } else { 0.0 };
    }

    /// Steering with over-correction and reaction lag applied.
    pub fn shape_steer(&mut self, steer: f32, dt: f32) -> f32 {
        if !self.active {
            return steer;
        }
        let steer = match self.event {
            Event::OverCorrect(_) => (steer * 1.6).clamp(-1.0, 1.0),
            // The driver keeps the wheel turned into the corner while the rear steps out.
            Event::PowerStab(_, held) => held,
            _ => steer,
        };
        if !self.has_steer || self.traits.reaction_tau_s <= 0.0 {
            self.has_steer = true;
            self.steer_out = steer;
        } else {
            self.steer_out += (steer - self.steer_out) * (1.0 - (-dt / self.traits.reaction_tau_s).exp());
        }
        self.steer_out
    }

    /// Throttle, brake and handbrake with a PowerStab or lift event applied.
    pub fn shape_pedals(&self, throttle: f32, brake: f32) -> (f32, f32, bool) {
        match self.event {
            Event::PowerStab(..) => (1.0, 0.0, true),
            Event::Lift(_) => (0.0, brake, false),
            _ => (throttle, brake, false),
        }
    }

    /// Corners found on the current track (for the harness).
    pub fn corners(&self) -> &[Corner] {
        &self.corners
    }
}

