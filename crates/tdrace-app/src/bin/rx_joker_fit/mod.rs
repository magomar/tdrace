//! Shared by `build_world_rx_joker` and `build_classic_rx_joker`: fits a spec 102 branch layout to the old joker
//! of a Rallycross circuit, converts the circuit, and keeps the migration report.
//!
//! See `specs/102_predefined_junction_components_for_road_splits_and_joker_loops.md` Pillar VII. For each junction
//! it tries a `Taper` and a `TurnOff` (angle measured from the old joker) of 10 to 40 m in 5 m steps, anchored on the
//! main waypoint the old joker used, and keeps the fit that deviates least from the old joker and passes every
//! guard. A circuit converts only when the compiled branch stays within [`MAX_DEVIATION_M`] of the old joker, the
//! joker still costs the lap time spec 088 requires, and validation reports no error. Otherwise the caller keeps
//! the legacy network and this module says why.

use std::path::Path;

use glam::Vec2;
use tdrace_core::track::bake::{bake, BakeOptions};
use tdrace_core::track::branch_kit::{self, BranchLayout};
use tdrace_core::track::geometry::{BarrierType, LineSegment};
use tdrace_core::track::junction_kit::{build_junction, JunctionComponent, JunctionRole, JunctionShape, Side, NOSE_GAP};
use tdrace_core::track::network::{RoadSegment, SegmentId, TrackNetwork};
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{validate_track, Track, ValidationSeverity};
use tdrace_core::SurfaceType;

/// Largest deviation of the compiled branch centreline from the old joker (spec 102 Pillar VII, m).
pub const MAX_DEVIATION_M: f32 = 2.0;

/// The deviation limit in force: [`MAX_DEVIATION_M`], or `RX_FIT_MAX_DEVIATION_M` to see what a looser limit would
/// convert. A looser limit is a decision for the spec, not for a run: do not commit circuits converted with one.
fn max_deviation() -> f32 {
    std::env::var("RX_FIT_MAX_DEVIATION_M").ok().and_then(|v| v.parse().ok()).unwrap_or(MAX_DEVIATION_M)
}
/// The joker lap must cost between these many seconds (spec 088, `test_rx_joker_costs_lap_time`).
const JOKER_COST_RANGE_S: (f32, f32) = (1.0, 7.5);
/// Junction lengths tried, in metres.
const LENGTHS_M: [f32; 7] = [10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0];
/// Offsets tried around the divider gap that puts the free end on the old joker (m).
const GAP_OFFSETS_M: [f32; 9] = [-2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0];
/// Fits kept per junction before the two junctions are combined.
const KEEP_PER_JUNCTION: usize = 8;
/// Old joker waypoints this close (m of joker arc) to a junction free end are dropped: the junction replaces them,
/// and the road needs room to turn from the junction heading to the old joker. Tried in this order.
const FREE_END_LEADS_M: [f32; 3] = [6.0, 12.0, 20.0];
/// Smallest divider gap a fit uses: the nose gap plus room for the free road to bulge toward the main road (m).
const MIN_FIT_GAP_M: f32 = NOSE_GAP + 0.4;
/// Nominal road widths tried: the old joker's, then 2 m narrower so a `TurnOff` can start on a main road that is no
/// wider than the joker.
const ROAD_WIDTH_TRIMS_M: [f32; 2] = [0.0, 2.0];
/// A fit whose junction alone deviates more than the deviation limit plus this is not tried (m): the free road
/// between the junctions can not make it good again.
const JUNCTION_DEVIATION_SLACK_M: f32 = 2.0;

/// One accepted fit.
#[derive(Debug, Clone)]
pub struct Fit {
    pub layout: BranchLayout,
    pub deviation: f32,
    pub cost_s: f32,
}

/// What became of one circuit.
#[derive(Debug, Clone)]
pub enum Outcome {
    Converted(Fit),
    Kept { reason: String },
}

/// Convenience helper to construct a Taper junction component.
#[allow(dead_code)]
pub fn taper(s: f32, length: f32, divider_gap: f32) -> JunctionComponent {
    JunctionComponent { s, kind: JunctionShape::Taper, length, divider_gap }
}

/// Convenience helper to construct a TurnOff junction component.
#[allow(dead_code)]
pub fn turn_off(s: f32, angle_deg: f32, length: f32, divider_gap: f32) -> JunctionComponent {
    JunctionComponent { s, kind: JunctionShape::TurnOff { angle_deg }, length, divider_gap }
}

/// Progress distance along `main` of waypoint `idx`.
pub fn waypoint_station(main: &TrackSpline, idx: usize) -> f32 {
    main.project_point(main.waypoints[idx].point).progress_distance
}

/// Helper to create a waypoint for a redrawn branch.
#[allow(dead_code)]
pub fn wp(x: f32, y: f32, width: f32, surface: SurfaceType, elevation: f32) -> TrackWaypoint {
    TrackWaypoint::new(Vec2::new(x, y), width).with_surface(surface).with_elevation(elevation)
}

/// An explicit redraw for a circuit that cannot be fitted automatically (Spec 105).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Redraw {
    /// A fully explicit BranchLayout ready to compile.
    Layout(BranchLayout),
    /// Explicit junction components with interior waypoints or automatic waypoint slicing from `old`.
    Config {
        split_idx: Option<usize>,
        merge_idx: Option<usize>,
        side: Option<Side>,
        road_width: Option<f32>,
        split: JunctionComponent,
        merge: JunctionComponent,
        road_waypoints: Option<Vec<TrackWaypoint>>,
        leads: Option<(f32, f32)>,
        nose_barrier: Option<BarrierType>,
    },
}

#[allow(dead_code)]
impl Redraw {
    pub fn new_config(split: JunctionComponent, merge: JunctionComponent) -> Self {
        Self::Config {
            split_idx: None,
            merge_idx: None,
            side: None,
            road_width: None,
            split,
            merge,
            road_waypoints: None,
            leads: None,
            nose_barrier: None,
        }
    }

    pub fn with_anchors(mut self, split_idx: usize, merge_idx: usize) -> Self {
        if let Self::Config { split_idx: ref mut s, merge_idx: ref mut m, .. } = self {
            *s = Some(split_idx);
            *m = Some(merge_idx);
        }
        self
    }

    pub fn with_split_idx(mut self, split_idx: usize) -> Self {
        if let Self::Config { split_idx: ref mut s, .. } = self {
            *s = Some(split_idx);
        }
        self
    }

    pub fn with_merge_idx(mut self, merge_idx: usize) -> Self {
        if let Self::Config { merge_idx: ref mut m, .. } = self {
            *m = Some(merge_idx);
        }
        self
    }

    pub fn with_side(mut self, s: Side) -> Self {
        if let Self::Config { side: ref mut opt_side, .. } = self {
            *opt_side = Some(s);
        }
        self
    }

    pub fn with_road_width(mut self, w: f32) -> Self {
        if let Self::Config { road_width: ref mut opt_width, .. } = self {
            *opt_width = Some(w);
        }
        self
    }

    pub fn with_waypoints(mut self, wps: Vec<TrackWaypoint>) -> Self {
        if let Self::Config { road_waypoints: ref mut opt_wps, .. } = self {
            *opt_wps = Some(wps);
        }
        self
    }

    pub fn with_leads(mut self, lead_in: f32, lead_out: f32) -> Self {
        if let Self::Config { leads: ref mut opt_leads, .. } = self {
            *opt_leads = Some((lead_in, lead_out));
        }
        self
    }

    pub fn with_nose_barrier(mut self, b: BarrierType) -> Self {
        if let Self::Config { nose_barrier: ref mut opt_barrier, .. } = self {
            *opt_barrier = Some(b);
        }
        self
    }

    pub fn to_layout(
        &self,
        main: &TrackSpline,
        name: &str,
        default_side: Side,
        default_width: f32,
        default_split_idx: usize,
        default_merge_idx: usize,
        old: &[TrackWaypoint],
    ) -> Result<BranchLayout, String> {
        match self {
            Redraw::Layout(layout) => Ok(layout.clone()),
            Redraw::Config {
                split_idx,
                merge_idx,
                side,
                road_width,
                split,
                merge,
                road_waypoints,
                leads,
                nose_barrier,
            } => {
                let side = side.unwrap_or(default_side);
                let road_width = road_width.unwrap_or(default_width);
                let mut split = split.clone();
                let effective_split_idx = split_idx.unwrap_or(default_split_idx);
                if split.s == 0.0 || split_idx.is_some() {
                    split.s = waypoint_station(main, effective_split_idx);
                }
                let effective_merge_idx = merge_idx.unwrap_or(default_merge_idx);
                let mut merge = merge.clone();
                if merge.s == 0.0 || merge_idx.is_some() {
                    merge.s = waypoint_station(main, effective_merge_idx);
                }
                let split_geom = build_junction(main, &split, JunctionRole::Entry, side, road_width)
                    .map_err(|e| format!("split junction: {:?}", e))?;
                let merge_geom = build_junction(main, &merge, JunctionRole::Exit, side, road_width)
                    .map_err(|e| format!("merge junction: {:?}", e))?;
                let (free_in, free_out) = (split_geom.free_end, merge_geom.free_end);
                let waypoints = if let Some(wps) = road_waypoints {
                    wps.clone()
                } else {
                    let reference = Reference::new(main, old, name);
                    let (arc_in, arc_out) = (reference.arc_of(free_in), reference.arc_of(free_out));
                    let (lead_in, lead_out) = leads.unwrap_or((10.0, 10.0));
                    old[1..old.len() - 1]
                        .iter()
                        .filter(|w| {
                            let a = reference.arc_of(w.point);
                            a >= arc_in + lead_in && a <= arc_out - lead_out
                        })
                        .cloned()
                        .collect()
                };
                Ok(BranchLayout {
                    layout_id: "joker".to_string(),
                    name: name.to_string(),
                    side,
                    split,
                    merge,
                    road_waypoints: waypoints,
                    road_width,
                    nose_barrier: nose_barrier.unwrap_or(BarrierType::TireWall),
                })
            }
        }
    }
}

pub fn shape_name(kind: JunctionShape) -> String {
    match kind {
        JunctionShape::Taper => "Taper".to_string(),
        JunctionShape::TurnOff { angle_deg } => format!("TurnOff {:.0}\u{b0}", angle_deg),
    }
}

/// The old joker as a dense polyline with, for every point, its main station and signed lateral offset.
pub struct Reference {
    pub points: Vec<Vec2>,
    pub arcs: Vec<f32>,
    /// Main-spline arc length of each point's projection.
    pub station: Vec<f32>,
    /// Offset from the main centreline along the left normal (m).
    pub lateral: Vec<f32>,
    /// Angle between the joker direction and the main direction (degrees).
    pub divergence: Vec<f32>,
}

impl Reference {
    pub fn new(main: &TrackSpline, waypoints: &[TrackWaypoint], name: &str) -> Self {
        let segment = RoadSegment::new(SegmentId(2), name, waypoints.to_vec());
        let points: Vec<Vec2> = segment.samples.iter().map(|s| s.point).collect();
        let mut arcs = Vec::with_capacity(points.len());
        let (mut station, mut lateral, mut divergence) = (Vec::new(), Vec::new(), Vec::new());
        for (i, p) in points.iter().enumerate() {
            arcs.push(segment.samples[i].distance);
            let proj = main.project_point(*p);
            station.push(proj.progress_distance);
            lateral.push((*p - proj.closest_point).dot(proj.normal));
            let t = segment.samples[i].tangent;
            divergence.push(t.perp_dot(proj.tangent).atan2(t.dot(proj.tangent)).abs().to_degrees());
        }
        Self { points, arcs, station, lateral, divergence }
    }

    /// Distance from `p` to the old joker.
    pub fn distance(&self, p: Vec2) -> f32 {
        self.points.windows(2).map(|w| LineSegment::new(w[0], w[1]).distance_to_point(p)).fold(f32::MAX, f32::min)
    }

    /// Joker arc length of the point nearest to `p`.
    pub fn arc_of(&self, p: Vec2) -> f32 {
        let mut best = (f32::MAX, 0.0);
        for (i, w) in self.points.windows(2).enumerate() {
            let seg = LineSegment::new(w[0], w[1]);
            let c = seg.closest_point(p);
            let d = c.distance(p);
            if d < best.0 {
                best = (d, self.arcs[i] + c.distance(w[0]));
            }
        }
        best.1
    }

    /// Lateral offset (outward positive for `sigma`) and divergence at main station `target`, searching from the
    /// split end (`forward`) or from the merge end.
    pub fn at_station(&self, target: f32, forward: bool, sigma: f32) -> Option<(f32, f32)> {
        let n = self.points.len();
        let order: Vec<usize> = if forward { (0..n).collect() } else { (0..n).rev().collect() };
        for pair in order.windows(2) {
            let (i, j) = (pair[0], pair[1]);
            let (a, b) = (self.station[i], self.station[j]);
            let crossed = if forward { a <= target && target <= b } else { a >= target && target >= b };
            if crossed && (b - a).abs() > 1e-4 {
                let f = (target - a) / (b - a);
                let lat = (self.lateral[i] + (self.lateral[j] - self.lateral[i]) * f) * sigma;
                let div = self.divergence[i] + (self.divergence[j] - self.divergence[i]) * f;
                return Some((lat, div));
            }
        }
        None
    }
}

/// A candidate junction with its deviation from the old joker.
#[derive(Clone)]
struct Candidate {
    comp: JunctionComponent,
    deviation: f32,
}

/// The best fits of one junction, and why the others were not kept.
struct Candidates {
    kept: Vec<Candidate>,
    /// Smallest deviation of any junction that passed the guards, kept or not.
    best_seen: Option<f32>,
    /// The first guard that rejected a junction.
    first_reject: Option<String>,
}

fn junction_candidates(
    main: &TrackSpline,
    reference: &Reference,
    s: f32,
    role: JunctionRole,
    side: Side,
    road_width: f32,
) -> Candidates {
    let sigma = side.sign();
    let forward = role == JunctionRole::Entry;
    let half_main = main.sample_at_distance(s).width * 0.5;
    let station_at = |span: f32| if forward { s + span } else { s - span };
    let mut out = Candidates { kept: Vec::new(), best_seen: None, first_reject: None };
    let mut consider = |comp: JunctionComponent| {
        let junction = match build_junction(main, &comp, role, side, road_width) {
            Ok(j) => j,
            Err(e) => {
                out.first_reject.get_or_insert(format!("{:?}", e));
                return;
            }
        };
        if junction.nose.is_none() {
            return;
        }
        let deviation = junction.centreline.iter().step_by(2).map(|p| reference.distance(*p)).fold(0.0f32, f32::max);
        out.best_seen = Some(out.best_seen.map_or(deviation, |b| b.min(deviation)));
        let same = |c: &Candidate| {
            c.comp.kind == comp.kind && c.comp.length == comp.length && (c.comp.divider_gap - comp.divider_gap).abs() < 0.01
        };
        if deviation <= max_deviation() + JUNCTION_DEVIATION_SLACK_M && !out.kept.iter().any(same) {
            out.kept.push(Candidate { comp, deviation });
        }
    };

    for length in LENGTHS_M {
        // Taper: its free end sits where the old joker is after `length` metres, as far out as the nose needs.
        if let Some((lateral, _)) = reference.at_station(station_at(length), forward, sigma) {
            let base_gap = lateral - half_main - road_width * 0.5;
            for offset in GAP_OFFSETS_M {
                consider(JunctionComponent {
                    s,
                    kind: JunctionShape::Taper,
                    length,
                    divider_gap: (base_gap + offset).max(MIN_FIT_GAP_M),
                });
            }
        }

        // TurnOff: the angle is the old joker's divergence where the arc ends (a few rounds settle it). The arc
        // starts on the main road, so its free-end gap follows from the start offset d0.
        let mut angle = 30.0f32;
        for _ in 0..4 {
            let theta = angle.to_radians();
            let span = length / theta * theta.sin();
            if let Some((_, divergence)) = reference.at_station(station_at(span), forward, sigma) {
                angle = divergence.clamp(10.0, 59.9);
            }
        }
        let theta = angle.to_radians();
        let radius = length / theta;
        let d0_max = half_main - road_width * 0.5;
        if d0_max >= 0.0 {
            for d0 in [0.0, d0_max * 0.5, d0_max] {
                let gap = d0 + radius * (1.0 - theta.cos()) - half_main - road_width * 0.5;
                if gap >= MIN_FIT_GAP_M {
                    consider(JunctionComponent { s, kind: JunctionShape::TurnOff { angle_deg: angle }, length, divider_gap: gap });
                }
            }
        }
    }
    out.kept.sort_by(|a, b| a.deviation.total_cmp(&b.deviation));
    out.kept.truncate(KEEP_PER_JUNCTION);
    out
}

/// Seconds a car limited only by top speed, corner grip, acceleration and braking needs for the lap. Same model as
/// `speed_limited_lap_time` in `rally_tracks_tests.rs`.
fn speed_limited_lap_time(spline: &TrackSpline) -> f32 {
    const TOP_SPEED: f32 = 40.0;
    const CORNER_GRIP: f32 = 10.0;
    const ACCELERATION: f32 = 8.0;
    const BRAKING: f32 = 10.0;
    const CURVATURE_SPAN_M: f32 = 4.0;
    let s = &spline.samples;
    let n = s.len();
    let ds = |i: usize| if i + 1 < n { s[i + 1].distance - s[i].distance } else { spline.total_length - s[i].distance };
    let span = ((CURVATURE_SPAN_M * n as f32 / spline.total_length).round() as usize).max(1);
    let mut v: Vec<f32> = (0..n)
        .map(|i| {
            let (a, b) = (&s[(i + n - span) % n], &s[(i + span) % n]);
            let turn = a.tangent.perp_dot(b.tangent).atan2(a.tangent.dot(b.tangent)).abs();
            let curvature = turn / (2.0 * span as f32 * spline.total_length / n as f32);
            (CORNER_GRIP / curvature.max(1e-6)).sqrt().min(TOP_SPEED)
        })
        .collect();
    for k in 0..2 * n {
        let (i, j) = (k % n, (k + 1) % n);
        v[j] = v[j].min((v[i] * v[i] + 2.0 * ACCELERATION * ds(i)).sqrt());
    }
    for k in (0..2 * n).rev() {
        let (i, j) = (k % n, (k + 1) % n);
        v[i] = v[i].min((v[j] * v[j] + 2.0 * BRAKING * ds(i)).sqrt());
    }
    (0..n).map(|i| ds(i) / ((v[i] + v[(i + 1) % n]) * 0.5)).sum()
}

/// Extra seconds the joker route costs over the main route.
fn joker_cost(network: &TrackNetwork) -> Result<f32, String> {
    let main = network.build_composite_spline_for_layout("main").ok_or("no main composite spline")?;
    let joker = network.build_composite_spline_for_layout("joker").ok_or("no joker composite spline")?;
    Ok(speed_limited_lap_time(&joker) - speed_limited_lap_time(&main))
}

/// Compiles `layout` into `track`, maps the circuit's checkpoints onto the new segments, adds the joker checkpoint and
/// bakes the walls, as `track_bake --rebuild` does.
pub fn install_layout(track: &mut Track, layout: &BranchLayout) -> Result<(), String> {
    track.branch_layout = Some(layout.clone());
    let compiled = layout.compile(track).map_err(|e| format!("{:?}", e))?;
    branch_kit::install_new(track, compiled)?;
    bake(track, &BakeOptions { rebuild: true, ..BakeOptions::default() })
        .map(|_| ())
        .map_err(|e| format!("bake: {}", e))
}

/// Validation errors and the joker lap cost of a converted circuit.
fn check_converted(track: &Track) -> Result<f32, String> {
    let errors: Vec<String> = validate_track(track)
        .into_iter()
        .filter(|d| d.severity == ValidationSeverity::Error)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();
    if !errors.is_empty() {
        let mut unique = errors;
        unique.sort();
        unique.dedup();
        return Err(format!("validation: {}", unique.join(", ")));
    }
    let cost = joker_cost(track.network.as_ref().ok_or("no network")?)?;
    if !(JOKER_COST_RANGE_S.0..=JOKER_COST_RANGE_S.1).contains(&cost) {
        return Err(format!("the joker lap costs {:.2} s, spec 088 needs {:.1}-{:.1} s", cost, JOKER_COST_RANGE_S.0, JOKER_COST_RANGE_S.1));
    }
    Ok(cost)
}

/// Fits and applies a layout for the old joker `old` (first and last waypoint are the main waypoints `split_idx` and
/// `merge_idx`). On success `track` is converted and baked; otherwise it is left as it was.
#[allow(dead_code)]
pub fn convert(track: &mut Track, id: &str, name: &str, split_idx: usize, merge_idx: usize, old: &[TrackWaypoint]) -> Outcome {
    convert_with_redraw(track, id, name, split_idx, merge_idx, old, None)
}

/// Applies an explicit redraw if provided, otherwise fits and applies a layout for the old joker `old`.
/// On success `track` is converted and baked; otherwise it is left as it was.
pub fn convert_with_redraw(
    track: &mut Track,
    id: &str,
    name: &str,
    split_idx: usize,
    merge_idx: usize,
    old: &[TrackWaypoint],
    redraw: Option<&Redraw>,
) -> Outcome {
    let kept = |reason: String| {
        println!("  {} keeps its legacy joker network: {}", id, reason);
        Outcome::Kept { reason }
    };
    let main = &track.spline;
    if old.len() < 3 || split_idx == 0 || split_idx >= merge_idx {
        return kept("the split or merge is not between two main waypoints of the lap".to_string());
    }
    let arc = |i: usize| main.project_point(main.waypoints[i].point).progress_distance;
    let (s_split, s_merge) = (arc(split_idx), arc(merge_idx));
    let reference = Reference::new(main, old, name);

    // Side: where the old joker is farthest from the main road. Road width: the most common interior width.
    let widest = reference.lateral.iter().copied().fold(0.0f32, |a, b| if b.abs() > a.abs() { b } else { a });
    if widest.abs() < 10.0 {
        return kept(format!("the old joker stays within {:.1} m of the main road", widest.abs()));
    }
    let side = if widest > 0.0 { Side::Left } else { Side::Right };
    let mut widths: Vec<f32> = old[1..old.len() - 1].iter().map(|w| w.width).collect();
    widths.sort_by(f32::total_cmp);
    let joker_width = widths[widths.len() / 2].round().max(8.0);

    if let Some(r) = redraw {
        let layout = match r.to_layout(main, name, side, joker_width, split_idx, merge_idx, old) {
            Ok(l) => l,
            Err(e) => return kept(format!("redraw layout error: {}", e)),
        };
        let geometry = match layout.compile_geometry(track) {
            Ok(g) => g,
            Err(e) => return kept(format!("redraw compile guards failed: {:?}", e)),
        };
        let deviation = if layout.road_waypoints.is_empty() {
            geometry.road.points.iter().step_by(2).map(|p| reference.distance(*p)).fold(0.0f32, f32::max)
        } else {
            let i_start = geometry.road.control_index[2];
            let i_end = geometry.road.control_index[2 + layout.road_waypoints.len() - 1];
            geometry.road.points[i_start..=i_end].iter().step_by(2).map(|p| reference.distance(*p)).fold(0.0f32, f32::max)
        };
        if deviation > max_deviation() {
            return kept(format!("redraw free road deviation {:.2} m > {:.1} m", deviation, max_deviation()));
        }
        let mut trial = track.clone();
        match install_layout(&mut trial, &layout).and_then(|_| check_converted(&trial)) {
            Ok(cost_s) => {
                *track = trial;
                return Outcome::Converted(Fit { layout, deviation, cost_s });
            }
            Err(e) => return kept(format!("redraw validation failed: {}", e)),
        }
    }

    let mut first_failure: Option<String> = None;
    let mut best_deviation: Option<f32> = None;
    let mut no_junction: Option<String> = None;
    for trim in ROAD_WIDTH_TRIMS_M {
        let road_width = joker_width - trim;
        if road_width < 8.0 {
            continue;
        }
        let splits = junction_candidates(main, &reference, s_split, JunctionRole::Entry, side, road_width);
        let merges = junction_candidates(main, &reference, s_merge, JunctionRole::Exit, side, road_width);
        let describe = |what: &str, c: &Candidates| match (c.best_seen, &c.first_reject) {
            (Some(dev), _) => format!(
                "no {} junction within {:.1} m of the old joker (best fit {:.1} m)",
                what,
                max_deviation() + JUNCTION_DEVIATION_SLACK_M,
                dev
            ),
            (None, Some(reject)) => format!("no {} junction passes the guards (first: {})", what, reject),
            (None, None) => format!("no {} junction fits", what),
        };
        if splits.kept.is_empty() {
            no_junction.get_or_insert(describe("split", &splits));
            continue;
        }
        if merges.kept.is_empty() {
            no_junction.get_or_insert(describe("merge", &merges));
            continue;
        }

        let mut pairs: Vec<(&Candidate, &Candidate)> =
            splits.kept.iter().flat_map(|a| merges.kept.iter().map(move |b| (a, b))).collect();
        pairs.sort_by(|a, b| (a.0.deviation + a.1.deviation).total_cmp(&(b.0.deviation + b.1.deviation)));
        for (split, merge) in pairs {
            let free_in = build_junction(main, &split.comp, JunctionRole::Entry, side, road_width).map(|j| j.free_end);
            let free_out = build_junction(main, &merge.comp, JunctionRole::Exit, side, road_width).map(|j| j.free_end);
            let (Ok(free_in), Ok(free_out)) = (free_in, free_out) else { continue };
            let (arc_in, arc_out) = (reference.arc_of(free_in), reference.arc_of(free_out));
            for lead in FREE_END_LEADS_M {
                // Old joker waypoints between the two free ends become the free road.
                let interior: Vec<TrackWaypoint> = old[1..old.len() - 1]
                    .iter()
                    .filter(|w| {
                        let a = reference.arc_of(w.point);
                        a >= arc_in + lead && a <= arc_out - lead
                    })
                    .cloned()
                    .collect();
                let layout = BranchLayout {
                    layout_id: "joker".to_string(),
                    name: name.to_string(),
                    side,
                    split: split.comp.clone(),
                    merge: merge.comp.clone(),
                    road_waypoints: interior,
                    road_width,
                    nose_barrier: BarrierType::TireWall,
                };
                let geometry = match layout.compile_geometry(track) {
                    Ok(g) => g,
                    Err(e) => {
                        first_failure.get_or_insert(format!("no fit passed the guards; first: {:?}", e));
                        continue;
                    }
                };
                let deviation = geometry.centreline.iter().step_by(2).map(|p| reference.distance(*p)).fold(0.0f32, f32::max);
                best_deviation = Some(best_deviation.map_or(deviation, |b| b.min(deviation)));
                if deviation > max_deviation() {
                    continue;
                }
                let mut trial = track.clone();
                match install_layout(&mut trial, &layout).and_then(|_| check_converted(&trial)) {
                    Ok(cost_s) => {
                        *track = trial;
                        return Outcome::Converted(Fit { layout, deviation, cost_s });
                    }
                    Err(e) => {
                        first_failure.get_or_insert(e);
                    }
                }
            }
        }
    }
    let reason = match (best_deviation, first_failure, no_junction) {
        (Some(dev), _, _) if dev > max_deviation() => format!("best deviation {:.2} m > {:.1} m", dev, max_deviation()),
        (_, Some(reason), _) => reason,
        (_, None, Some(reason)) => reason,
        _ => "no fit passed the guards".to_string(),
    };
    kept(reason)
}

// ---------------------------------------------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------------------------------------------

/// One row of `docs/circuits/branch_junction_migration.md`.
pub struct Row {
    pub circuit: String,
    pub outcome: Outcome,
}

const REPORT_HEADER: &str = "# Branch Junction Migration (Spec 102)\n\n\
Generated by `build_world_rx_joker` and `build_classic_rx_joker`. A circuit converts when a layout of two junction\n\
components (Taper or TurnOff, 10-40 m, anchored on the main waypoints the old joker used) passes every guard, its\n\
compiled branch stays within 2.0 m of the old joker, the joker lap still costs the time spec 088 requires, and\n\
validation reports no error. Otherwise the circuit keeps its legacy network and its junction wall warnings.\n\n\
| Circuit | Result | Split | Merge | Length in / out (m) | Max deviation (m) | Joker lap cost (s) | Reason |\n\
|---------|--------|-------|-------|--------------------:|------------------:|-------------------:|--------|\n";

fn format_row(row: &Row) -> String {
    match &row.outcome {
        Outcome::Converted(fit) => format!(
            "| {} | converted | {} | {} | {:.0} / {:.0} | {:.2} | {:.2} | - |",
            row.circuit,
            shape_name(fit.layout.split.kind),
            shape_name(fit.layout.merge.kind),
            fit.layout.split.length,
            fit.layout.merge.length,
            fit.deviation,
            fit.cost_s
        ),
        Outcome::Kept { reason } => {
            format!("| {} | kept legacy | - | - | - | - | - | {} |", row.circuit, reason.replace('|', "/"))
        }
    }
}

/// Merges `rows` into the report: rows of other circuits are kept, the table is sorted by circuit name, and the
/// summary line counts every row.
pub fn write_report(path: &Path, rows: &[Row]) {
    // `RX_FIT_REPORT` sends the report of an experimental run elsewhere, so it does not replace the committed one.
    let override_path = std::env::var("RX_FIT_REPORT").ok().map(std::path::PathBuf::from);
    let path = override_path.as_deref().unwrap_or(path);
    let mut table: Vec<(String, String)> = Vec::new();
    if let Ok(text) = std::fs::read_to_string(path) {
        for line in text.lines().filter(|l| l.starts_with("| ") && !l.starts_with("| Circuit")) {
            let name = line.trim_start_matches("| ").split(" | ").next().unwrap_or("").to_string();
            table.push((name, line.to_string()));
        }
    }
    for row in rows {
        table.retain(|(name, _)| *name != row.circuit);
        table.push((row.circuit.clone(), format_row(row)));
    }
    table.sort();
    let converted = table.iter().filter(|(_, line)| line.contains("| converted |")).count();
    let mut out = String::from(REPORT_HEADER);
    for (_, line) in &table {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!("\n{} of {} circuits converted.\n", converted, table.len()));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create the report folder");
    }
    std::fs::write(path, out).expect("write the migration report");
}
