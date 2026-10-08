//! Parametric pit lane kit: predefined junction components on a free-form main spline.
//!
//! See `specs/101_parametric_pit_lane_kit_blocks_on_spline_circuits.md`. A pit lane is an entry junction, a free
//! pit road and an exit junction. The junction block below (`Side`, `JunctionShape`, `JunctionComponent`,
//! `JunctionError`, `build_junction`) is self-contained and uses the names of spec 102, which moves it to
//! `junction_kit.rs` for road branches.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;

use super::geometry::{LineSegment, PitBox, PitLane, PitLaneChevron, PitLaneExitQuad, PitLaneJunctionData};
use super::scenery::{Building, BuildingStyle};
use super::spline::{catmull_rom_centripetal_2d, TrackSpline, TrackWaypoint};
use super::Track;

// ---------------------------------------------------------------------------------------------------------------
// Junction component (shared with spec 102)
// ---------------------------------------------------------------------------------------------------------------

/// Side of the main track (relative to driving direction) on which the branch runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    /// +1 for the left-pointing spline normal, -1 for the right.
    #[inline]
    pub fn sign(self) -> f32 {
        match self {
            Side::Left => 1.0,
            Side::Right => -1.0,
        }
    }
}

/// Shape of a junction between the main track and the branch road.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JunctionShape {
    /// Smooth sideways taper; the branch ends parallel to the main track.
    Taper,
    /// Branch leaves (or rejoins) the main track at a fixed angle through a circular arc.
    TurnOff { angle_deg: f32 },
}

/// Predefined junction component anchored to the main spline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JunctionComponent {
    /// Arc length on the main spline where the junction touches the track (m).
    /// Entry: where the split starts. Exit: where the merge ends.
    pub s: f32,
    pub kind: JunctionShape,
    /// Length of the junction (m): main-spline span for `Taper`, arc length for `TurnOff`.
    pub length: f32,
    /// Gap between main track edge and branch road edge at the junction's free end (m).
    pub divider_gap: f32,
}

/// Which end of the branch a junction is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JunctionRole {
    /// Leaves the main track (pit entry, branch split).
    Entry,
    /// Rejoins the main track (pit exit, branch merge).
    Exit,
}

/// Junction rule violations (spec 101 guards 1-4 and 6).
#[derive(Debug, Clone, PartialEq)]
pub enum JunctionError {
    InvalidParameter { name: &'static str },
    JunctionTooSteep { junction: JunctionRole, angle_deg: f32 },
    OffsetExceedsCurvature { junction: JunctionRole, s: f32, radius: f32 },
    ArcTooTight { junction: JunctionRole, radius: f32 },
    JointKink { junction: JunctionRole, angle_deg: f32 },
}

/// Clearance kept between a curve's centre and the outer branch edge, and the minimum arc radius margin (m).
pub const JUNCTION_RADIUS_MARGIN: f32 = 3.0;
/// Edge gap at which a dividing wall fits between the main track and the branch (m).
pub const DIVIDER_WALL_MIN_GAP: f32 = 1.2;
/// Sampling step along a junction (m).
const JUNCTION_STEP: f32 = 0.5;

/// Geometry of one compiled junction. Every point list is in driving order.
#[derive(Debug, Clone, PartialEq)]
pub struct JunctionGeometry {
    pub role: JunctionRole,
    /// Branch centreline samples. Entry: from `s` to the free end. Exit: from the free end to `s`.
    pub centreline: Vec<Vec2>,
    /// Point where the branch road starts (entry) or ends (exit).
    pub free_end: Vec2,
    /// Unit driving direction at the free end.
    pub free_heading: Vec2,
    /// Line across the branch road at the free end.
    pub gate: LineSegment,
    /// Edge gap (branch inner edge minus main edge, m) at each centreline sample.
    pub edge_gaps: Vec<f32>,
    /// Main track edge point at each centreline sample, on the branch side.
    pub track_edges: Vec<Vec2>,
    /// Branch inner edge point at each centreline sample.
    pub branch_edges: Vec<Vec2>,
    /// Point where the branch inner edge leaves (entry) or rejoins (exit) the main track edge.
    pub apex: Vec2,
    /// Paved quads between the main edge and the branch inner edge: entry from `s` to the apex, exit from the
    /// last station with a wall-wide gap to `s`.
    pub quads: Vec<[Vec2; 4]>,
    /// Gore chevrons between `s` and the apex (entry only).
    pub chevrons: Vec<PitLaneChevron>,
    /// End of the dividing wall: the first (entry) or last (exit) station with a wall-wide gap.
    pub divider_end: Vec2,
}

/// Builds a junction component on `main`, checking guards 1-4. Guard 6 (joint kink) needs the road and is
/// checked by the caller with [`joint_kink_deg`].
pub fn build_junction(
    main: &TrackSpline,
    comp: &JunctionComponent,
    role: JunctionRole,
    side: Side,
    road_width: f32,
) -> Result<JunctionGeometry, JunctionError> {
    // Guard 1: parameter ranges. Written so NaN fails.
    if !(comp.length > 0.0 && comp.length.is_finite()) {
        return Err(JunctionError::InvalidParameter { name: "length" });
    }
    if !(comp.divider_gap >= 0.0 && comp.divider_gap.is_finite()) {
        return Err(JunctionError::InvalidParameter { name: "divider_gap" });
    }
    if !comp.s.is_finite() {
        return Err(JunctionError::InvalidParameter { name: "s" });
    }
    if let JunctionShape::TurnOff { angle_deg } = comp.kind {
        if !(10.0..60.0).contains(&angle_deg) {
            return Err(JunctionError::InvalidParameter { name: "angle_deg" });
        }
    }
    if main.samples.len() < 2 {
        return Err(JunctionError::InvalidParameter { name: "s" });
    }

    let sigma = side.sign();
    let anchor = main.sample_at_distance(comp.s);
    let offset_d = anchor.width * 0.5 + comp.divider_gap + road_width * 0.5;
    // Main-spline span the junction covers, in driving order.
    let span_main = match comp.kind {
        JunctionShape::Taper => comp.length,
        JunctionShape::TurnOff { angle_deg } => {
            let theta = angle_deg.to_radians();
            comp.length / theta * theta.sin()
        }
    };
    let (span_start, span_end) = match role {
        JunctionRole::Entry => (comp.s, comp.s + span_main),
        JunctionRole::Exit => (comp.s - span_main, comp.s),
    };

    // Guard 2: Taper divergence.
    if let JunctionShape::Taper = comp.kind {
        let angle_deg = (1.5 * offset_d / comp.length).atan().to_degrees();
        if angle_deg >= 60.0 {
            return Err(JunctionError::JunctionTooSteep { junction: role, angle_deg });
        }
    }

    // Guard 3: fold. The outer branch edge must stay inside the radius of a main curve that bends toward it.
    let outer = offset_d + road_width * 0.5;
    let mut s_probe = span_start;
    while s_probe <= span_end + 1e-3 {
        let kappa = signed_curvature(main, s_probe);
        if kappa * sigma > 0.0 {
            let radius = 1.0 / kappa.abs();
            if outer >= radius - JUNCTION_RADIUS_MARGIN {
                return Err(JunctionError::OffsetExceedsCurvature { junction: role, s: s_probe, radius });
            }
        }
        s_probe += 1.0;
    }

    let centreline = match comp.kind {
        JunctionShape::Taper => taper_centreline(main, comp, role, sigma, offset_d),
        JunctionShape::TurnOff { angle_deg } => {
            let theta = angle_deg.to_radians();
            let radius = comp.length / theta;
            // Guard 4: arc radius.
            if radius < road_width * 0.5 + JUNCTION_RADIUS_MARGIN {
                return Err(JunctionError::ArcTooTight { junction: role, radius });
            }
            turnoff_centreline(main, comp, role, side, road_width, theta, radius)?
        }
    };

    Ok(finish_junction(main, role, sigma, road_width, centreline))
}

/// Heading jump (degrees) between a junction's free-end heading and the road heading next to it.
pub fn joint_kink_deg(junction_heading: Vec2, road_heading: Vec2) -> f32 {
    let a = junction_heading.normalize_or_zero();
    let b = road_heading.normalize_or_zero();
    a.perp_dot(b).atan2(a.dot(b)).abs().to_degrees()
}

/// Signed curvature of `main` at `s` (1/m, positive when the track turns left), from tangents 1 m either side.
pub fn signed_curvature(main: &TrackSpline, s: f32) -> f32 {
    const H: f32 = 1.0;
    let t0 = main.sample_at_distance(s - H).tangent;
    let t1 = main.sample_at_distance(s + H).tangent;
    t0.perp_dot(t1).atan2(t0.dot(t1)) / (2.0 * H)
}

/// Branch edge gap at `p`: distance from the main track edge to the branch inner edge, positive when apart.
fn edge_gap_at(main: &TrackSpline, sigma: f32, road_width: f32, p: Vec2) -> (f32, Vec2, Vec2, Vec2) {
    let proj = main.project_point(p);
    let n = proj.normal;
    let track_edge = proj.closest_point + n * (sigma * proj.track_width * 0.5);
    let branch_edge = p - n * (sigma * road_width * 0.5);
    let gap = (branch_edge - track_edge).dot(n * sigma);
    (gap, track_edge, branch_edge, n)
}

fn taper_centreline(main: &TrackSpline, comp: &JunctionComponent, role: JunctionRole, sigma: f32, offset_d: f32) -> Vec<Vec2> {
    let n = ((comp.length / JUNCTION_STEP).ceil() as usize).max(2);
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let (s_main, u) = match role {
                JunctionRole::Entry => (comp.s + t * comp.length, t),
                JunctionRole::Exit => (comp.s - comp.length + t * comp.length, 1.0 - t),
            };
            let smooth = u * u * (3.0 - 2.0 * u);
            let sample = main.sample_at_distance(s_main);
            sample.point + sample.normal * (sigma * offset_d * smooth)
        })
        .collect()
}

/// Circular arc that starts parallel to the main track at `s` and turns `theta` toward `side`. The start offset is
/// solved so the free-end edge gap equals `divider_gap`.
fn turnoff_centreline(
    main: &TrackSpline,
    comp: &JunctionComponent,
    role: JunctionRole,
    side: Side,
    road_width: f32,
    theta: f32,
    radius: f32,
) -> Result<Vec<Vec2>, JunctionError> {
    let sigma = side.sign();
    let anchor = main.sample_at_distance(comp.s);
    let half_w = anchor.width * 0.5;
    let n = ((comp.length / JUNCTION_STEP).ceil() as usize).max(2);

    // Arc in driving order for a given start offset `d0`.
    let arc = |d0: f32| -> Vec<Vec2> {
        let t0 = anchor.tangent;
        let n0 = anchor.normal * sigma;
        let start = anchor.point + n0 * d0;
        let centre = start + n0 * radius;
        (0..=n)
            .map(|i| {
                let f = i as f32 / n as f32;
                match role {
                    JunctionRole::Entry => {
                        let phi = f * theta;
                        centre - n0 * (radius * phi.cos()) + t0 * (radius * phi.sin())
                    }
                    JunctionRole::Exit => {
                        let phi = (1.0 - f) * theta;
                        centre - n0 * (radius * phi.cos()) - t0 * (radius * phi.sin())
                    }
                }
            })
            .collect()
    };
    let free_end_of = |pts: &[Vec2]| match role {
        JunctionRole::Entry => pts[pts.len() - 1],
        JunctionRole::Exit => pts[0],
    };

    // On a straight: d0 + r(1 - cos θ) = D. Correct for main curvature with a few fixed-point steps.
    let target = comp.divider_gap;
    let mut d0 = half_w + comp.divider_gap + road_width * 0.5 - radius * (1.0 - theta.cos());
    for _ in 0..6 {
        let (gap, ..) = edge_gap_at(main, sigma, road_width, free_end_of(&arc(d0)));
        let err = target - gap;
        d0 += err;
        if err.abs() < 1e-3 {
            break;
        }
    }
    // The branch must start on the main road: centre at or beyond the main centreline, outer edge at most flush
    // with the main edge. Outside that range the arc is too short or too long for the gap.
    if !(d0 >= 0.0 && d0 <= half_w - road_width * 0.5) {
        return Err(JunctionError::InvalidParameter { name: "length" });
    }
    Ok(arc(d0))
}

fn finish_junction(main: &TrackSpline, role: JunctionRole, sigma: f32, road_width: f32, centreline: Vec<Vec2>) -> JunctionGeometry {
    let count = centreline.len();
    let mut edge_gaps = Vec::with_capacity(count);
    let mut track_edges = Vec::with_capacity(count);
    let mut branch_edges = Vec::with_capacity(count);
    for &p in &centreline {
        let (gap, te, be, _) = edge_gap_at(main, sigma, road_width, p);
        edge_gaps.push(gap);
        track_edges.push(te);
        branch_edges.push(be);
    }

    let (free_end, free_heading) = match role {
        JunctionRole::Entry => (centreline[count - 1], (centreline[count - 1] - centreline[count - 2]).normalize_or_zero()),
        JunctionRole::Exit => (centreline[0], (centreline[1] - centreline[0]).normalize_or_zero()),
    };
    let across = free_heading.perp() * (road_width * 0.5);
    let gate = LineSegment::new(free_end - across, free_end + across);

    // Apex: where the gap crosses zero, walking away from `s`.
    let lerp_at = |i: usize, j: usize| -> (Vec2, Vec2, f32) {
        let (g0, g1) = (edge_gaps[i], edge_gaps[j]);
        let f = if (g1 - g0).abs() > 1e-6 { (-g0 / (g1 - g0)).clamp(0.0, 1.0) } else { 0.0 };
        (track_edges[i].lerp(track_edges[j], f), branch_edges[i].lerp(branch_edges[j], f), f)
    };
    let order: Vec<usize> = match role {
        JunctionRole::Entry => (0..count).collect(),
        JunctionRole::Exit => (0..count).rev().collect(),
    };
    let mut apex_te = track_edges[order[count - 1]];
    let mut apex_be = branch_edges[order[count - 1]];
    let mut apex_k = count - 1;
    for k in 1..count {
        if edge_gaps[order[k]] >= 0.0 {
            let (te, be, _) = lerp_at(order[k - 1], order[k]);
            apex_te = te;
            apex_be = be;
            apex_k = k;
            break;
        }
    }
    let apex = (apex_te + apex_be) * 0.5;

    let divider_k = (0..count).find(|&k| edge_gaps[order[k]] >= DIVIDER_WALL_MIN_GAP).unwrap_or(count - 1);
    let divider_end = {
        let i = order[divider_k];
        let gap = edge_gaps[i].max(0.0);
        let n = (branch_edges[i] - track_edges[i]).normalize_or_zero();
        branch_edges[i] - n * (gap * 0.5).min(1.0)
    };

    let mut quads = Vec::new();
    let mut chevrons = Vec::new();
    match role {
        JunctionRole::Entry => {
            for k in 0..apex_k {
                let (i, j) = (order[k], order[k + 1]);
                let (te1, be1) = if k + 1 == apex_k { (apex_te, apex_be) } else { (track_edges[j], branch_edges[j]) };
                quads.push([track_edges[i], te1, be1, branch_edges[i]]);
            }
            let te_start = track_edges[0];
            let pe_start = branch_edges[0];
            let v_track = (apex_te - te_start).normalize_or_zero();
            let v_pit = (apex_be - pe_start).normalize_or_zero();
            let gore_len = (apex - te_start).length();
            if gore_len > 8.0 {
                let bisect = -(v_track + v_pit).normalize_or_zero();
                let num = ((gore_len - 4.0) / 3.0).floor() as usize;
                for step in 1..=num {
                    let d = step as f32 * 3.0;
                    if d >= gore_len - 2.0 {
                        break;
                    }
                    let f = d / gore_len;
                    let pt_track = te_start.lerp(apex_te, f);
                    let pt_pit = pe_start.lerp(apex_be, f);
                    chevrons.push(PitLaneChevron { apex: (pt_track + pt_pit) * 0.5 + bisect * 1.2, pt_track, pt_pit });
                }
            }
        }
        JunctionRole::Exit => {
            let first = order[divider_k];
            for i in first..count - 1 {
                quads.push([track_edges[i], track_edges[i + 1], branch_edges[i + 1], branch_edges[i]]);
            }
        }
    }

    JunctionGeometry {
        role,
        centreline,
        free_end,
        free_heading,
        gate,
        edge_gaps,
        track_edges,
        branch_edges,
        apex,
        quads,
        chevrons,
        divider_end,
    }
}

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
