//! Predefined junction components on a free-form main spline, shared by pit lanes and road branches.
//!
//! See `specs/102_predefined_junction_components_for_road_splits_and_joker_loops.md`. The block moved here from
//! `pit_kit.rs` (spec 101) with its geometry, parameters and guards unchanged. A junction is a branch road that
//! leaves (`Entry`) or rejoins (`Exit`) the main track on one `Side`, as a `Taper` or a `TurnOff`. Spec 102 adds the
//! nose point and the outer envelope that the road branch walls are built from.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::geometry::{LineSegment, PitLaneChevron};
use super::spline::{catmull_rom_centripetal_2d, TrackSpline};

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
/// Length of the nose barrier that closes the island between two roads (spec 102 Pillar IV, m).
pub const NOSE_LENGTH: f32 = 1.6;
/// Clearance kept between the nose barrier ends and each road edge (m).
pub const NOSE_CLEARANCE: f32 = 0.3;
/// Edge gap at which the nose barrier fits between the two road edges: the nose plus a clearance on each side (m).
pub const NOSE_GAP: f32 = NOSE_LENGTH + 2.0 * NOSE_CLEARANCE;
/// Spacing of the outer envelope stations along the main spline (m).
pub const ENVELOPE_STEP: f32 = 1.0;

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
    /// End of the dividing wall, between the main edge and the branch inner edge at the free end.
    pub divider_end: Vec2,
    /// Where the branch road's own edge leaves (entry) or rejoins (exit) the main track edge. `apex` measures the gap
    /// along the main normal (spec 101, the pit markings); on a steep taper the true edge leaves up to a couple of metres later.
    pub edge_apex: Vec2,
    /// First point after `edge_apex` where the wedge between the two edges is [`NOSE_GAP`] wide; `None` when the
    /// junction never gets that wide.
    pub nose: Option<NosePoint>,
    /// Main-spline arc lengths (start, end) of the junction in driving order. Not wrapped: `end` can pass the lap length.
    pub span: (f32, f32),
}

/// Where the wedge between the main track edge and the branch road edge first is [`NOSE_GAP`] wide (spec 102).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NosePoint {
    /// Midpoint between the two road edges, on the bisector of the wedge.
    pub point: Vec2,
    /// Main track edge point (beyond its curb, if any) and branch edge point, [`NOSE_GAP`] apart across the wedge.
    pub track_edge: Vec2,
    pub branch_edge: Vec2,
    /// Unit vector from `track_edge` to `branch_edge`: perpendicular to the bisector of the two road edges.
    pub across: Vec2,
    /// Angle between the two road edges at the nose (degrees).
    pub divergence_deg: f32,
    /// Index into `centreline` of the first sample at or past the nose, walking away from `s`.
    pub index: usize,
}

/// Cross-section of the wedge between the main track edge and the edge of the branch road that faces it (spec 102).
///
/// Measured across the bisector of the two edges, so a barrier that fits across it keeps its clearance from both
/// edges even where the branch leaves at a steep angle. Where the edges are not within 75 degrees of each other the
/// section runs along the shortest line between them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wedge {
    /// Where the cross-section meets the main track edge, and the branch edge point it starts from.
    pub main_edge: Vec2,
    pub branch_edge: Vec2,
    /// Unit vector from `main_edge` to `branch_edge`.
    pub across: Vec2,
    /// Gap between the edges along `across` (m). Negative while the branch edge is still inside the main ribbon.
    pub width: f32,
    /// Cosine of half the angle between the edges: a wall `o` from an edge stands `o / cos_half` along `across`.
    pub cos_half: f32,
    /// Angle between the two edge directions (degrees).
    pub divergence_deg: f32,
    /// Width of the curb on the main edge here, measured across the main edge (m); 0 without a curb. A wall stands
    /// beyond it.
    pub curb: f32,
}

impl Wedge {
    /// The main edge with its curb: where a wall next to the main road can start. Measured along `across`.
    #[inline]
    pub fn main_wall_edge(&self) -> Vec2 {
        self.main_edge + self.across * (self.curb / self.cos_half)
    }

    /// Gap between the main curb and the branch edge along `across` (m).
    #[inline]
    pub fn clear_width(&self) -> f32 {
        self.width - self.curb / self.cos_half
    }
}

/// The wedge at branch point `p` driving along `heading`, on `side` of the main track. `half_width` is half the width
/// of the branch road there.
pub fn wedge_at(main: &TrackSpline, side: Side, p: Vec2, heading: Vec2, half_width: f32) -> Wedge {
    let sigma = side.sign();
    let t_b = heading.normalize_or_zero();
    let n_b = Vec2::new(-t_b.y, t_b.x);
    // The branch edge that faces the main track.
    let edge = p - n_b * (sigma * half_width);
    let proj = main.project_point(edge);
    let t_m = proj.tangent;
    let n_m = proj.normal * sigma;
    let lateral = (edge - proj.closest_point).dot(n_m) - proj.track_width * 0.5;
    let cos_phi = t_m.dot(t_b).clamp(-1.0, 1.0);
    let (across, width, cos_half) = if cos_phi > 0.25 {
        let bisector = (t_m + t_b).normalize_or_zero();
        let mut across = bisector.perp();
        if across.dot(n_m) < 0.0 {
            across = -across;
        }
        let cos_half = across.dot(n_m).max(0.5);
        (across, lateral / cos_half, cos_half)
    } else {
        (n_m, lateral, 1.0)
    };
    let has_curb = if sigma > 0.0 { proj.left_curb } else { proj.right_curb };
    Wedge {
        main_edge: edge - across * width,
        branch_edge: edge,
        across,
        width,
        cos_half,
        divergence_deg: cos_phi.acos().to_degrees(),
        curb: if has_curb { TrackSpline::DEFAULT_CURB_WIDTH } else { 0.0 },
    }
}

/// One station of the outer envelope of a junction: the outer edge, on the branch side, of the union of the main
/// and branch ribbons (spec 102 Pillar I).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopePoint {
    /// Distance from the start of the junction span along the main spline (m).
    pub s: f32,
    /// Main centreline point at the station and the unit main normal pointing to the branch side.
    pub centre: Vec2,
    pub outward: Vec2,
    /// Half the main track width at the station (m).
    pub main_half: f32,
    /// Distance of the branch outer edge from the main centreline, along `outward` (m).
    pub branch_edge: f32,
    /// 1 / cos of the heading difference between the branch and the main track: a wall that stands `g` from the
    /// branch edge is `g * branch_slant` from it along `outward`.
    pub branch_slant: f32,
}

impl EnvelopePoint {
    /// Distance of the envelope from the main centreline along `outward` (m).
    #[inline]
    pub fn edge_offset(&self) -> f32 {
        self.main_half.max(self.branch_edge)
    }

    /// The envelope point itself.
    #[inline]
    pub fn point(&self) -> Vec2 {
        self.centre + self.outward * self.edge_offset()
    }

    /// Distance from the main centreline along `outward` of a wall that stands `gap` outside the envelope (m).
    #[inline]
    pub fn wall_offset(&self, gap: f32) -> f32 {
        (self.main_half + gap).max(self.branch_edge + gap * self.branch_slant)
    }

    /// Point of a wall that stands `gap` outside the envelope.
    #[inline]
    pub fn wall_point(&self, gap: f32) -> Vec2 {
        self.centre + self.outward * self.wall_offset(gap)
    }
}

impl JunctionGeometry {
    /// Outer envelope on `side`: the outer edge of the union of the main and branch ribbons, one station every
    /// [`ENVELOPE_STEP`] over the junction span, in driving order. Walls that stand outside both roads are built
    /// from it (spec 102 Pillar IV).
    pub fn outer_envelope(&self, main: &TrackSpline, side: Side, road_width: f32) -> Vec<EnvelopePoint> {
        let sigma = side.sign();
        let (s_start, s_end) = self.span;
        let total = main.total_length;
        let unwrap = |progress: f32| {
            let d = progress - s_start;
            if main.closed {
                (d + 0.5 * total).rem_euclid(total) - 0.5 * total
            } else {
                d
            }
        };

        // Station, branch outer edge and slant of every centreline sample.
        let n = self.centreline.len();
        let mut rows: Vec<(f32, f32, f32)> = Vec::with_capacity(n);
        for i in 0..n {
            let p = self.centreline[i];
            let heading = (self.centreline[(i + 1).min(n - 1)] - self.centreline[i.saturating_sub(1)]).normalize_or_zero();
            let proj = main.project_point(p);
            let lateral = (p - proj.closest_point).dot(proj.normal * sigma);
            let cos = heading.dot(proj.tangent).clamp(0.2, 1.0);
            let station = unwrap(proj.progress_distance).max(rows.last().map_or(f32::NEG_INFINITY, |r| r.0));
            rows.push((station, lateral + road_width * 0.5 / cos, 1.0 / cos));
        }

        let length = s_end - s_start;
        let count = (length / ENVELOPE_STEP).ceil().max(1.0) as usize;
        (0..=count)
            .map(|k| {
                let station = (k as f32 * ENVELOPE_STEP).min(length);
                let at = rows.partition_point(|r| r.0 <= station).clamp(1, n - 1);
                let (a, b) = (rows[at - 1], rows[at]);
                let f = if b.0 - a.0 > 1e-4 { ((station - a.0) / (b.0 - a.0)).clamp(0.0, 1.0) } else { 0.0 };
                let sample = main.sample_at_distance(s_start + station);
                EnvelopePoint {
                    s: station,
                    centre: sample.point,
                    outward: sample.normal * sigma,
                    main_half: sample.width * 0.5,
                    branch_edge: a.1 + (b.1 - a.1) * f,
                    branch_slant: a.2 + (b.2 - a.2) * f,
                }
            })
            .collect()
    }
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

    Ok(finish_junction(main, role, side, road_width, centreline, (span_start, span_end)))
}

/// Distance of the guide points that fix the road heading at both junction joints (m).
const ROAD_GUIDE_DISTANCE: f32 = 5.0;
/// Sampling step along the free road (m).
pub const ROAD_STEP: f32 = 1.0;
/// Largest heading jump allowed at a junction/road joint (degrees).
const MAX_JOINT_KINK_DEG: f32 = 2.0;

/// The free road between two junctions.
#[derive(Debug, Clone, PartialEq)]
pub struct FreeRoad {
    /// Centreline samples from the entry free end to the exit free end, [`ROAD_STEP`] apart at most.
    pub points: Vec<Vec2>,
    /// Index in `points` of each control point: the entry free end, its guide point, the interior points, the exit
    /// guide point and the exit free end.
    pub control_index: Vec<usize>,
}

/// Builds the free road from the entry free end to the exit free end through `interior`: a centripetal Catmull-Rom
/// curve with guide points that fix the road heading at both joints. Checks guard 6 (joint kink).
pub fn free_road(entry: &JunctionGeometry, exit: &JunctionGeometry, interior: &[Vec2]) -> Result<FreeRoad, JunctionError> {
    let h = ROAD_GUIDE_DISTANCE;
    let mut controls = vec![entry.free_end, entry.free_end + entry.free_heading * h];
    controls.extend(interior.iter().copied());
    controls.push(exit.free_end - exit.free_heading * h);
    controls.push(exit.free_end);
    let ghost_start = entry.free_end - entry.free_heading * h;
    let ghost_end = exit.free_end + exit.free_heading * h;
    let mut road = vec![controls[0]];
    let mut control_index = vec![0];
    for i in 0..controls.len() - 1 {
        let p0 = if i == 0 { ghost_start } else { controls[i - 1] };
        let p1 = controls[i];
        let p2 = controls[i + 1];
        let p3 = if i + 2 < controls.len() { controls[i + 2] } else { ghost_end };
        let n = (((p2 - p1).length() / ROAD_STEP).ceil() as usize).max(1);
        for k in 1..=n {
            road.push(catmull_rom_centripetal_2d(p0, p1, p2, p3, k as f32 / n as f32));
        }
        control_index.push(road.len() - 1);
    }

    // Guard 6: heading jump at both joints, from the road tangent at the joint itself.
    let nc = controls.len();
    const EPS_T: f32 = 1e-3;
    let road_out = catmull_rom_centripetal_2d(ghost_start, controls[0], controls[1], controls[2], EPS_T) - controls[0];
    let entry_kink = joint_kink_deg(entry.free_heading, road_out);
    if entry_kink > MAX_JOINT_KINK_DEG {
        return Err(JunctionError::JointKink { junction: JunctionRole::Entry, angle_deg: entry_kink });
    }
    let road_in = controls[nc - 1]
        - catmull_rom_centripetal_2d(controls[nc - 3], controls[nc - 2], controls[nc - 1], ghost_end, 1.0 - EPS_T);
    let exit_kink = joint_kink_deg(exit.free_heading, road_in);
    if exit_kink > MAX_JOINT_KINK_DEG {
        return Err(JunctionError::JointKink { junction: JunctionRole::Exit, angle_deg: exit_kink });
    }
    Ok(FreeRoad { points: road, control_index })
}

/// Cumulative arc length at each point of a polyline.
pub fn cumulative_lengths(points: &[Vec2]) -> Vec<f32> {
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
pub fn first_self_crossing(points: &[Vec2]) -> Option<usize> {
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

fn finish_junction(
    main: &TrackSpline,
    role: JunctionRole,
    side: Side,
    road_width: f32,
    centreline: Vec<Vec2>,
    span: (f32, f32),
) -> JunctionGeometry {
    let sigma = side.sign();
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

    // Spec 102: apex and nose from the branch road's true edge, across the wedge between the two edges.
    let wedges: Vec<Wedge> = (0..count)
        .map(|i| {
            let heading = (centreline[(i + 1).min(count - 1)] - centreline[i.saturating_sub(1)]).normalize_or_zero();
            wedge_at(main, side, centreline[i], heading, road_width * 0.5)
        })
        .collect();
    let first_open = (1..count).find(|&k| wedges[order[k]].width >= 0.0);
    let edge_apex = match first_open {
        Some(k) => {
            let (a, b) = (&wedges[order[k - 1]], &wedges[order[k]]);
            let f = (-a.width / (b.width - a.width).max(1e-6)).clamp(0.0, 1.0);
            let (m, e) = (a.main_edge.lerp(b.main_edge, f), a.branch_edge.lerp(b.branch_edge, f));
            (m + e) * 0.5
        }
        None => apex,
    };
    let nose = first_open.and_then(|open| {
        let k = (open..count).find(|&k| wedges[order[k]].clear_width() >= NOSE_GAP)?;
        let (a, b) = (&wedges[order[k - 1]], &wedges[order[k]]);
        let f = ((NOSE_GAP - a.clear_width()) / (b.clear_width() - a.clear_width()).max(1e-6)).clamp(0.0, 1.0);
        let (m, e) = (a.main_wall_edge().lerp(b.main_wall_edge(), f), a.branch_edge.lerp(b.branch_edge, f));
        Some(NosePoint {
            point: (m + e) * 0.5,
            track_edge: m,
            branch_edge: e,
            across: (e - m).normalize_or_zero(),
            divergence_deg: a.divergence_deg + (b.divergence_deg - a.divergence_deg) * f,
            index: order[k],
        })
    });

    let divider_k = (0..count).find(|&k| edge_gaps[order[k]] >= DIVIDER_WALL_MIN_GAP).unwrap_or(count - 1);
    // The dividing wall ends at the free end, beside the gate: a bot that steers straight at the gate centre from the
    // main road passed a wall end inside the junction and stuck on it (red_bull_ring).
    let divider_end = {
        let i = order[count - 1];
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
        edge_apex,
        nose,
        span,
    }
}
