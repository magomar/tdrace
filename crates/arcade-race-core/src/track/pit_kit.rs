//! Parametric pit lane kit: predefined junction components on a free-form main spline.
//!
//! See `specs/101_parametric_pit_lane_kit_blocks_on_spline_circuits.md`. A pit lane is an entry junction, a free
//! pit road and an exit junction. The junction component (`Side`, `JunctionShape`, `JunctionComponent`,
//! `JunctionError`, `build_junction`) lives in `junction_kit.rs` since spec 102, shared with road branches.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;

use super::geometry::{BarrierType, LineSegment, PitBox, PitLane, PitLaneExitQuad, PitLaneJunctionData};
use super::junction_kit::{
    build_junction, joint_kink_deg, signed_curvature, JunctionComponent, JunctionError, JunctionGeometry,
    JunctionRole, JunctionShape, Side, JUNCTION_RADIUS_MARGIN,
};
use super::scenery::{Building, BuildingStyle};
use super::spline::{catmull_rom_centripetal_2d, TrackSpline, TrackWaypoint};
use super::Track;

// ---------------------------------------------------------------------------------------------------------------
// Pit lane layout
// ---------------------------------------------------------------------------------------------------------------

/// Predefined box row component placed along the pit road.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitBoxRow {
    /// Arc length along the compiled pit lane where the first box centre sits (m).
    pub start_s: f32,
    /// Number of stalls, >= 1.
    pub count: u32,
    /// Distance between stall centres (m).
    pub spacing: f32,
    /// Place one PitGarage building behind each stall.
    pub garages: bool,
}

/// Source of truth for a pit lane. Compiled at bake time into `PitLane`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PitLaneLayout {
    pub side: Side,
    pub entry: JunctionComponent,
    pub exit: JunctionComponent,
    /// Interior control points of the free pit road, between the junction ends.
    /// May be empty: the road then joins the two junction ends directly.
    pub road_waypoints: Vec<Vec2>,
    pub road_width: f32,
    pub speed_limit: f32,
    pub box_row: PitBoxRow,
}

/// Pit lane rule violations (spec 101 Pillar V). Junction rules are wrapped in `Junction`.
#[derive(Debug, Clone, PartialEq)]
pub enum PitKitError {
    Junction(JunctionError),
    InvalidParameter { name: &'static str },
    RoadTooTight { s: f32, radius: f32 },
    RoadOverlapsTrack { s: f32 },
    BoxRowOffRoad { box_index: u32 },
    JunctionOrder,
}

impl From<JunctionError> for PitKitError {
    fn from(e: JunctionError) -> Self {
        PitKitError::Junction(e)
    }
}

/// Output of [`PitLaneLayout::compile`]: the runtime pit lane, its junction markings, garages and wall anchors.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledPitLane {
    pub lane: PitLane,
    pub junctions: PitLaneJunctionData,
    pub garages: Vec<Building>,
    /// Pit side of the main track.
    pub side: Side,
    /// Where the dividing pit wall starts (entry junction) and ends (exit junction).
    pub divider_start: Vec2,
    pub divider_end: Vec2,
    /// Arc length on the compiled lane where the free road starts and ends.
    pub road_start_s: f32,
    pub road_end_s: f32,
}

/// Distance of the guide points that fix the road heading at both junction joints (m).
const ROAD_GUIDE_DISTANCE: f32 = 5.0;
/// Sampling step along the free road (m).
const ROAD_STEP: f32 = 1.0;
/// Minimum radius at a stall (m).
const BOX_MIN_RADIUS: f32 = 50.0;
/// Upper bound on stalls, so untrusted JSON cannot ask for an unbounded allocation.
const MAX_PIT_BOXES: u32 = 100;
/// Default pit box stop radius (spec 062).
const BOX_STOP_RADIUS: f32 = 3.0;
/// Garage depth away from the road (m), and its clearance outside the outer pit wall (m).
const GARAGE_DEPTH: f32 = 10.0;
const GARAGE_WALL_CLEARANCE: f32 = 1.0;
/// Offset of the outer pit wall from the pit road edge, as `Track::generate_pit_lane_walls` places it (m).
const OUTER_WALL_OFFSET: f32 = 1.5;
/// Tolerance on the road-to-track gap rule (m). The 5 m guide points make the road bulge up to ~0.13 m toward the
/// track next to a joint when the road then bends away from it.
const ROAD_GAP_TOLERANCE: f32 = 0.25;
/// Largest heading jump allowed at a junction/road joint (degrees).
const MAX_JOINT_KINK_DEG: f32 = 2.0;

impl PitLaneLayout {
    /// Compiles the layout on `track`'s main spline. Pure: the same layout and main spline give the same lane.
    pub fn compile(&self, track: &Track) -> Result<CompiledPitLane, PitKitError> {
        let main = &track.spline;
        self.check_parameters()?;

        let entry = build_junction(main, &self.entry, JunctionRole::Entry, self.side, self.road_width)?;
        let exit = build_junction(main, &self.exit, JunctionRole::Exit, self.side, self.road_width)?;
        self.check_order(main)?;

        // Free road: centripetal Catmull-Rom with guide points that fix the heading at both joints.
        let h = ROAD_GUIDE_DISTANCE;
        let mut controls = vec![entry.free_end, entry.free_end + entry.free_heading * h];
        controls.extend(self.road_waypoints.iter().copied());
        controls.push(exit.free_end - exit.free_heading * h);
        controls.push(exit.free_end);
        let ghost_start = entry.free_end - entry.free_heading * h;
        let ghost_end = exit.free_end + exit.free_heading * h;
        let mut road = vec![controls[0]];
        for i in 0..controls.len() - 1 {
            let p0 = if i == 0 { ghost_start } else { controls[i - 1] };
            let p1 = controls[i];
            let p2 = controls[i + 1];
            let p3 = if i + 2 < controls.len() { controls[i + 2] } else { ghost_end };
            let n = (((p2 - p1).length() / ROAD_STEP).ceil() as usize).max(1);
            for k in 1..=n {
                road.push(catmull_rom_centripetal_2d(p0, p1, p2, p3, k as f32 / n as f32));
            }
        }

        // Guard 6: heading jump at both joints, from the road tangent at the joint itself.
        let nc = controls.len();
        const EPS_T: f32 = 1e-3;
        let road_out = catmull_rom_centripetal_2d(ghost_start, controls[0], controls[1], controls[2], EPS_T) - controls[0];
        let entry_kink = joint_kink_deg(entry.free_heading, road_out);
        if entry_kink > MAX_JOINT_KINK_DEG {
            return Err(JunctionError::JointKink { junction: JunctionRole::Entry, angle_deg: entry_kink }.into());
        }
        let road_in = controls[nc - 1]
            - catmull_rom_centripetal_2d(controls[nc - 3], controls[nc - 2], controls[nc - 1], ghost_end, 1.0 - EPS_T);
        let last = road.len() - 1;
        let exit_kink = joint_kink_deg(exit.free_heading, road_in);
        if exit_kink > MAX_JOINT_KINK_DEG {
            return Err(JunctionError::JointKink { junction: JunctionRole::Exit, angle_deg: exit_kink }.into());
        }

        // Guard 7: outside the junctions the road keeps its gap to the main track and does not cross itself.
        let min_gap = self.entry.divider_gap.min(self.exit.divider_gap) - ROAD_GAP_TOLERANCE;
        let road_len_at = cumulative_lengths(&road);
        for (i, &p) in road.iter().enumerate() {
            let proj = main.project_point(p);
            let gap = proj.lateral_offset.abs() - proj.track_width * 0.5 - self.road_width * 0.5;
            if gap < min_gap {
                return Err(PitKitError::RoadOverlapsTrack { s: road_len_at[i] });
            }
        }
        if let Some(i) = first_self_crossing(&road) {
            return Err(PitKitError::RoadOverlapsTrack { s: road_len_at[i] });
        }

        // Compiled lane: entry junction, road, exit junction.
        let mut points: Vec<Vec2> = entry.centreline[..entry.centreline.len() - 1].to_vec();
        let road_start_s = cumulative_lengths(&entry.centreline).last().copied().unwrap_or(0.0);
        points.extend_from_slice(&road);
        let road_end_s = road_start_s + road_len_at[last];
        points.extend_from_slice(&exit.centreline[1..]);
        let waypoints: Vec<TrackWaypoint> = points
            .iter()
            .map(|&p| {
                let mut wp = TrackWaypoint::new(p, self.road_width)
                    .with_runoff_surface(SurfaceType::Asphalt)
                    .with_walls(false, false);
                wp.surface = Some(SurfaceType::Asphalt);
                wp.elevation = main.project_point(p).elevation;
                wp
            })
            .collect();
        let spline = TrackSpline::new(waypoints, false);

        // Guard 5: free road minimum radius, measured on the compiled lane.
        let min_radius = self.road_width * 0.5 + JUNCTION_RADIUS_MARGIN;
        let mut s = road_start_s;
        while s <= road_end_s {
            let radius = 1.0 / signed_curvature(&spline, s).abs().max(1e-6);
            if radius < min_radius {
                return Err(PitKitError::RoadTooTight { s, radius });
            }
            s += ROAD_STEP;
        }

        // Guard 8 and Pillar IV: stalls on the road part, on stretches of radius >= 50 m, with a garage each.
        let sigma = self.side.sign();
        let mut pit_boxes = Vec::with_capacity(self.box_row.count as usize);
        let mut garages = Vec::new();
        for i in 0..self.box_row.count {
            let s_box = self.box_row.start_s + i as f32 * self.box_row.spacing;
            let off_road = s_box - BOX_STOP_RADIUS < road_start_s || s_box + BOX_STOP_RADIUS > road_end_s;
            let tight = [s_box - BOX_STOP_RADIUS, s_box, s_box + BOX_STOP_RADIUS]
                .iter()
                .any(|&sp| signed_curvature(&spline, sp).abs() > 1.0 / BOX_MIN_RADIUS);
            if off_road || tight {
                return Err(PitKitError::BoxRowOffRoad { box_index: i });
            }
            let sample = spline.sample_at_distance(s_box);
            pit_boxes.push(PitBox::new(sample.point, sample.tangent, BOX_STOP_RADIUS, sample.elevation));
            if self.box_row.garages {
                let out = sample.normal * sigma;
                let reach = self.road_width * 0.5 + OUTER_WALL_OFFSET + GARAGE_WALL_CLEARANCE + GARAGE_DEPTH * 0.5;
                let heading = sample.tangent.y.atan2(sample.tangent.x);
                // A building's front faces its right; turn it so the front faces the road.
                let angle = if sigma > 0.0 { heading } else { heading + std::f32::consts::PI };
                let size = Vec2::new(self.box_row.spacing * 0.9, GARAGE_DEPTH);
                let mut garage = Building::new(0, sample.point + out * reach, size, angle, BuildingStyle::PitGarage);
                garage.elevation = sample.elevation;
                garages.push(garage);
            }
        }

        let lane = PitLane::new(
            spline,
            self.road_width,
            self.speed_limit,
            pit_boxes,
            entry.gate,
            exit.gate,
        );
        let junctions = junction_data(&lane, &entry, &exit);
        Ok(CompiledPitLane {
            lane,
            junctions,
            garages,
            side: self.side,
            divider_start: entry.divider_end,
            divider_end: exit.divider_end,
            road_start_s,
            road_end_s,
        })
    }

    /// Guard 1 for the pit lane parameters. Written so NaN fails.
    fn check_parameters(&self) -> Result<(), PitKitError> {
        let bad = |name| Err(PitKitError::InvalidParameter { name });
        if !(self.road_width >= 4.0 && self.road_width.is_finite()) {
            return bad("road_width");
        }
        if !(self.speed_limit > 0.0 && self.speed_limit.is_finite()) {
            return bad("speed_limit");
        }
        if !(1..=MAX_PIT_BOXES).contains(&self.box_row.count) {
            return bad("count");
        }
        if !(self.box_row.spacing > 0.0 && self.box_row.spacing.is_finite()) {
            return bad("spacing");
        }
        if !self.box_row.start_s.is_finite() {
            return bad("start_s");
        }
        if self.road_waypoints.iter().any(|p| !p.is_finite()) {
            return bad("road_waypoints");
        }
        Ok(())
    }

    /// Guard 9: entry span, then exit span, along the driving direction. Wraps across S/F on a closed main spline.
    fn check_order(&self, main: &TrackSpline) -> Result<(), PitKitError> {
        let span = |c: &JunctionComponent| match c.kind {
            JunctionShape::Taper => c.length,
            JunctionShape::TurnOff { angle_deg } => {
                let theta = angle_deg.to_radians();
                c.length / theta * theta.sin()
            }
        };
        let total = main.total_length;
        let ahead = if main.closed {
            (self.exit.s - self.entry.s).rem_euclid(total)
        } else {
            self.exit.s - self.entry.s
        };
        if ahead > span(&self.entry) + span(&self.exit) && ahead < total {
            Ok(())
        } else {
            Err(PitKitError::JunctionOrder)
        }
    }
}

/// Writes a compiled layout into `track`: its pit lane, and its garages in place of every `PitGarage` building (the
/// layout is the only source of pit garages). Walls are left to `trim_walls_for_pit_lane` and
/// `generate_pit_lane_walls`.
pub fn install(track: &mut Track, compiled: CompiledPitLane) {
    track.pit_lane = Some(compiled.lane);
    let buildings = &mut track.geometry.buildings;
    buildings.retain(|b| b.style != BuildingStyle::PitGarage);
    let first_id = buildings.iter().map(|b| b.id + 1).max().unwrap_or(0);
    for (i, mut garage) in compiled.garages.into_iter().enumerate() {
        garage.id = first_id + i;
        buildings.push(garage);
    }
}

/// Builds the runtime junction markings from the two components (no search).
fn junction_data(lane: &PitLane, entry: &JunctionGeometry, exit: &JunctionGeometry) -> PitLaneJunctionData {
    let exit_quads: Vec<PitLaneExitQuad> = exit
        .quads
        .iter()
        .enumerate()
        .map(|(i, q)| PitLaneExitQuad { quad: *q, has_dashed_line: i % 2 == 0, line_start: q[3], line_end: q[2] })
        .collect();

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    let mut expand = |p: Vec2, r: f32| {
        min = min.min(p - Vec2::splat(r));
        max = max.max(p + Vec2::splat(r));
    };
    for s in &lane.spline.samples {
        expand(s.point, lane.road_width * 2.0);
    }
    for b in &lane.pit_boxes {
        expand(b.position, b.stop_radius + 6.0);
    }
    for q in entry.quads.iter().chain(exit.quads.iter()) {
        for p in q {
            expand(*p, 6.0);
        }
    }
    expand(entry.apex, 6.0);
    expand(entry.track_edges[0], 6.0);
    expand(entry.branch_edges[0], 6.0);

    PitLaneJunctionData {
        bounds_min: min,
        bounds_max: max,
        entrance_quads: entry.quads.clone(),
        has_gore: true,
        p_apex: entry.apex,
        track_edge_apex: entry.apex,
        pit_inner_apex: entry.apex,
        te_start: entry.track_edges[0],
        pe_start: entry.branch_edges[0],
        chevrons: entry.chevrons.clone(),
        exit_quads,
    }
}

fn cumulative_lengths(points: &[Vec2]) -> Vec<f32> {
    let mut out = Vec::with_capacity(points.len());
    let mut acc = 0.0;
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            acc += p.distance(points[i - 1]);
        }
        out.push(acc);
    }
    out
}

/// Index of the first polyline segment that crosses a non-adjacent segment.
fn first_self_crossing(points: &[Vec2]) -> Option<usize> {
    let segs: Vec<LineSegment> = points.windows(2).map(|w| LineSegment::new(w[0], w[1])).collect();
    for i in 0..segs.len() {
        for j in i + 2..segs.len() {
            if segs[i].intersect_segment(&segs[j]).is_some() {
                return Some(i);
            }
        }
    }
    None
}

/// The main wall is cut this far before the entry and after the exit (m); the perimeter runs from the cut ends.
const PERIMETER_OVERLAP: f32 = 10.0;
/// Spacing of perimeter wall points (m).
const PERIMETER_STEP: f32 = 1.5;
/// Longest ray used to find the main wall beside the track (m).
const MAIN_WALL_SEARCH: f32 = 30.0;

/// Outer pit perimeter: one unbroken polyline from the main wall before the entry, round the outside of the pit
/// lane, to the main wall after the exit. Where the pit road is near the track it follows the main wall line;
/// elsewhere it runs `OUTER_WALL_OFFSET` outside the pit road edge. Garages stay outside it.
/// Where the pit perimeter replaces the main wall: pit side and main-spline anchors of the pit lane ends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerimeterSpan {
    pub side: Side,
    pub entry_s: f32,
    pub exit_s: f32,
}

/// The perimeter span of `track`'s pit lane: from its layout, or for a free-form lane whose ends touch the main road,
/// from the main-spline projection of its ends and the side its middle lies on. `None`: no perimeter.
pub fn perimeter_span(track: &Track) -> Option<PerimeterSpan> {
    let lane = track.pit_lane.as_ref()?;
    if lane.spline.samples.len() < 2 || track.spline.samples.len() < 2 {
        return None;
    }
    if let Some(layout) = track.pit_lane_layout.as_ref().filter(|l| l.compile(track).is_ok()) {
        return Some(PerimeterSpan { side: layout.side, entry_s: layout.entry.s, exit_s: layout.exit.s });
    }
    let samples = &lane.spline.samples;
    // A free-form lane whose end does not touch the main road cannot be enclosed without walling off the path to
    // it; it keeps the legacy walls.
    let touches_main = |p: Vec2| {
        let proj = track.spline.project_point(p);
        proj.distance_to_spline - lane.road_width * 0.5 <= proj.track_width * 0.5 + 0.5
    };
    if !touches_main(samples[0].point) || !touches_main(samples[samples.len() - 1].point) {
        return None;
    }
    let mid = &samples[samples.len() / 2];
    let proj = track.spline.project_point(mid.point);
    let side = if (mid.point - proj.closest_point).dot(proj.normal) >= 0.0 { Side::Left } else { Side::Right };
    Some(PerimeterSpan {
        side,
        entry_s: track.spline.project_point(samples[0].point).progress_distance,
        exit_s: track.spline.project_point(samples[samples.len() - 1].point).progress_distance,
    })
}

pub fn perimeter_chain(track: &Track, lane: &PitLane, span: &PerimeterSpan) -> (Vec<Vec2>, BarrierType) {
    let main = &track.spline;
    if main.samples.len() < 2 || lane.spline.samples.len() < 2 {
        return (Vec::new(), BarrierType::Concrete);
    }
    let sigma = span.side.sign();
    // Distance from the main edge to the main wall on the pit side, and its type, measured just outside the cut.
    let wall_at = |s: f32| -> Option<(f32, BarrierType)> {
        let sample = main.sample_at_distance(s);
        let out = sample.normal * sigma;
        let edge = sample.point + out * (sample.width * 0.5);
        track
            .geometry
            .all_walls()
            .filter_map(|w| w.segment.intersect_ray(edge, out, MAIN_WALL_SEARCH).map(|(d, _)| (d, w.barrier_type)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
    };
    let cut_in = span.entry_s - PERIMETER_OVERLAP;
    let cut_out = span.exit_s + PERIMETER_OVERLAP;
    let (wall_in, wall_out) = (wall_at(cut_in - 2.0), wall_at(cut_out + 2.0));
    let fallback = track.effective_barrier_offset();
    let gap_in = wall_in.map_or(fallback, |w| w.0);
    let gap_out = wall_out.map_or(fallback, |w| w.0);
    let barrier = wall_in.or(wall_out).map_or(BarrierType::Concrete, |w| w.1);
    // Each end starts on the cut end of the main wall when there is one, so the two join.
    let wall_end_near = |s: f32, gap: f32| {
        let sample = main.sample_at_distance(s);
        let target = sample.point + sample.normal * (sigma * (sample.width * 0.5 + gap));
        track
            .geometry
            .all_walls()
            .flat_map(|w| [w.segment.start, w.segment.end])
            .filter(|p| p.distance(target) < 3.0)
            .min_by(|a, b| a.distance(target).total_cmp(&b.distance(target)))
            .unwrap_or(target)
    };

    let mut pts = vec![wall_end_near(cut_in, gap_in)];
    let total = lane.spline.total_length;
    let mut d = 0.0;
    while d <= total {
        let sample = lane.spline.sample_at_distance(d);
        let outer = sample.point + sample.normal * (sigma * (lane.road_width * 0.5 + OUTER_WALL_OFFSET));
        let proj = main.project_point(outer);
        let lat = (outer - proj.closest_point).dot(proj.normal) * sigma;
        let gap = if d < total * 0.5 { gap_in } else { gap_out };
        let floor = proj.track_width * 0.5 + gap;
        pts.push(if lat >= floor { outer } else { proj.closest_point + proj.normal * (sigma * floor) });
        d += PERIMETER_STEP;
    }
    pts.push(wall_end_near(cut_out, gap_out));
    pts.dedup_by(|a, b| a.distance(*b) < 0.1);
    (pts, barrier)
}

/// True for points on the pit side of the main track, within `MAIN_WALL_SEARCH` of its edge, from
/// `PERIMETER_OVERLAP` before the entry anchor to `PERIMETER_OVERLAP` after the exit anchor: the stretch where the
/// perimeter replaces the main wall.
pub fn in_main_wall_span(track: &Track, span: &PerimeterSpan, p: Vec2) -> bool {
    let main = &track.spline;
    let proj = main.project_point(p);
    let lat = (p - proj.closest_point).dot(proj.normal) * span.side.sign();
    if lat <= proj.track_width * 0.5 || lat > proj.track_width * 0.5 + MAIN_WALL_SEARCH {
        return false;
    }
    let s = proj.progress_distance;
    let (from, to) = (span.entry_s - PERIMETER_OVERLAP, span.exit_s + PERIMETER_OVERLAP);
    if main.closed {
        let total = main.total_length;
        (s - from).rem_euclid(total) < (to - from).rem_euclid(total)
    } else {
        s > from && s < to
    }
}

/// Pieces of `seg` that lie outside the perimeter span (0.5 m resolution).
pub fn wall_pieces_outside_span(track: &Track, span: &PerimeterSpan, seg: &LineSegment) -> Vec<LineSegment> {
    let len = seg.start.distance(seg.end);
    let n = ((len / 0.5).ceil() as usize).max(1);
    let at = |i: usize| seg.start.lerp(seg.end, i as f32 / n as f32);
    let mut pieces = Vec::new();
    let mut run_start: Option<usize> = None;
    for i in 0..=n {
        let keep = !in_main_wall_span(track, span, at(i));
        match (keep, run_start) {
            (true, None) => run_start = Some(i),
            (false, Some(start)) => {
                if i - 1 > start {
                    pieces.push(LineSegment::new(at(start), at(i - 1)));
                }
                run_start = None;
            }
            _ => {}
        }
    }
    if let Some(start) = run_start {
        if n > start {
            pieces.push(LineSegment::new(at(start), at(n)));
        }
    }
    pieces
}

/// Holes in a pit lane's enclosure (spec 101). Rays at 60-120 degrees to the lane or track direction, every 2 m:
/// - from the lane centre to either side: must hit a wall or reach the main road within 40 m (near the ends the
///   main road can bend round to the pit side);
/// - from the main edge on the pit side, 30 m before the entry to 30 m after the exit, outward: must hit a wall.
///
/// Returns one line per open ray.
pub fn enclosure_holes(track: &Track) -> Vec<String> {
    const STEP: f32 = 2.0;
    const RANGE: f32 = 40.0;
    const MARGIN: f32 = 30.0;
    let Some(lane) = &track.pit_lane else { return Vec::new() };
    let main = &track.spline;
    if lane.spline.samples.len() < 2 || main.samples.len() < 2 {
        return Vec::new();
    }
    let walls: Vec<&LineSegment> = track.geometry.all_walls().map(|w| &w.segment).collect();
    let wall_hit = |o: Vec2, dir: Vec2| walls.iter().filter_map(|w| w.intersect_ray(o, dir, RANGE).map(|(t, _)| t)).fold(f32::INFINITY, f32::min);
    let reaches_main = |o: Vec2, dir: Vec2, until: f32| {
        let mut t = 0.5;
        while t < until.min(RANGE) {
            let proj = main.project_point(o + dir * t);
            if proj.distance_to_spline <= proj.track_width * 0.5 {
                return true;
            }
            t += 0.5;
        }
        false
    };
    let mid = &lane.spline.samples[lane.spline.samples.len() / 2];
    let mid_proj = main.project_point(mid.point);
    let sigma = if (mid.point - mid_proj.closest_point).dot(mid_proj.normal) >= 0.0 { 1.0 } else { -1.0 };
    let angles = || (6..=12).map(|k| (10.0 * k as f32).to_radians());
    let mut holes = Vec::new();

    let mut d = 0.0;
    while d <= lane.spline.total_length {
        let sample = lane.spline.sample_at_distance(d);
        for a in angles() {
            let outer = sample.tangent * a.cos() + sample.normal * (sigma * a.sin());
            let hit = wall_hit(sample.point, outer);
            if hit > RANGE && !reaches_main(sample.point, outer, hit) {
                holes.push(format!("lane s {d:.0} m: outer ray at {:.0} deg is open", a.to_degrees()));
            }
            let inner = sample.tangent * a.cos() - sample.normal * (sigma * a.sin());
            let hit = wall_hit(sample.point, inner);
            if hit > RANGE && !reaches_main(sample.point, inner, hit) {
                holes.push(format!("lane s {d:.0} m: inner ray at {:.0} deg is open", a.to_degrees()));
            }
        }
        d += STEP;
    }

    let entry = main.project_point(lane.spline.samples[0].point).progress_distance;
    let exit = main.project_point(lane.spline.samples[lane.spline.samples.len() - 1].point).progress_distance;
    let (entry, exit) = perimeter_span(track).map_or((entry, exit), |sp| (sp.entry_s, sp.exit_s));
    let total = main.total_length;
    let length = if main.closed { (exit - entry).rem_euclid(total) } else { exit - entry };
    let mut k = -MARGIN;
    while k <= length + MARGIN {
        let sample = main.sample_at_distance(entry + k);
        let edge = sample.point + sample.normal * (sigma * sample.width * 0.5);
        for a in angles() {
            let dir = sample.tangent * a.cos() + sample.normal * (sigma * a.sin());
            if wall_hit(edge, dir) > RANGE {
                holes.push(format!("main s {:.0} m: outward ray at {:.0} deg is open", (entry + k).rem_euclid(total.max(1.0)), a.to_degrees()));
            }
        }
        k += STEP;
    }
    holes
}

/// True when `p` lies more than 0.5 m past either end of the lane, along the end tangent. Projection onto an open
/// spline clamps to its end there, so its lateral offset says nothing about the lane: a main wall 10 m before a lane
/// that starts beside it looked like a wall inside the lane.
pub fn beyond_lane_end(lane: &PitLane, p: Vec2) -> bool {
    let samples = &lane.spline.samples;
    let (Some(first), Some(last)) = (samples.first(), samples.last()) else { return false };
    (p - first.point).dot(first.tangent) < -0.5 || (p - last.point).dot(last.tangent) > 0.5
}
