//! Road branches built from predefined junction components.
//!
//! See `specs/102_predefined_junction_components_for_road_splits_and_joker_loops.md`. A branch (today: the
//! Rallycross joker) leaves the main route at a split junction, runs on a free road and rejoins at a merge
//! junction. Both junctions are components of `junction_kit.rs`. `BranchLayout::compile` turns the layout into the
//! `TrackNetwork` of spec 006, and `walls` builds the walls of the junction region from the components, so no wall
//! is searched and clipped at a split or merge.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;

use super::checkpoint::Checkpoint;
use super::geometry::{BarrierType, LineSegment, WallBarrier};
use super::presets::merge_collinear_walls;
use super::junction_kit::{
    build_junction, cumulative_lengths, free_road, EnvelopePoint, FreeRoad, JunctionComponent, JunctionError,
    JunctionGeometry, JunctionRole, JunctionShape, NosePoint, Side, NOSE_GAP, NOSE_LENGTH,
};
use super::network::{
    GoreConfig, JunctionId, MergeConfig, RoadJunction, RoadSegment, SegmentId, SocketId, SplineSocket, TrackLayout,
    TrackNetwork,
};
use super::spline::{TrackSpline, TrackWaypoint};
use super::{wall_clear_of_roads, Track};

/// Source of truth for a branch that leaves the main route and rejoins it. Compiled into `Track::network`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchLayout {
    /// Id of the `TrackLayout` that takes the branch (for example "joker").
    pub layout_id: String,
    pub name: String,
    /// Side of the main route (relative to driving direction) on which the branch leaves.
    pub side: Side,
    /// `s` must be the arc length of a main waypoint (guard 2).
    pub split: JunctionComponent,
    /// `s` must be the arc length of a main waypoint (guard 2).
    pub merge: JunctionComponent,
    /// Interior waypoints of the free branch road, between the junction free ends. Full waypoints, so a branch keeps
    /// its own width, surface and elevation per point.
    pub road_waypoints: Vec<TrackWaypoint>,
    pub road_width: f32,
    pub nose_barrier: BarrierType,
}

/// Branch rule violations (spec 102 Pillar V). Junction rules are wrapped in `Junction`.
#[derive(Debug, Clone, PartialEq)]
pub enum BranchKitError {
    Junction(JunctionError),
    InvalidParameter { name: &'static str },
    AnchorOffWaypoint { junction: JunctionRole },
    NoseOutsideJunction { junction: JunctionRole },
    DividerTooNarrow { s: f32 },
    RoadTooTight { s: f32, radius: f32 },
    JunctionOrder,
}

impl From<JunctionError> for BranchKitError {
    fn from(e: JunctionError) -> Self {
        BranchKitError::Junction(e)
    }
}

/// Geometry of a compiled branch: the two junction components, the free road and the whole branch centreline.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchGeometry {
    pub split: JunctionGeometry,
    pub merge: JunctionGeometry,
    pub road: FreeRoad,
    /// Split centreline, free road and merge centreline as one polyline, in driving order.
    pub centreline: Vec<Vec2>,
    /// Index in `centreline` where the free road starts and ends.
    pub road_start: usize,
    pub road_end: usize,
    /// Index in the main waypoints of the split and merge anchors.
    pub split_waypoint: usize,
    pub merge_waypoint: usize,
    /// The merge anchor lies before the split anchor on the main spline: the branch passes the start/finish line.
    pub wraps: bool,
}

/// Output of [`BranchLayout::compile`].
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledBranch {
    /// Segments, junctions and layouts. Layout checkpoint lists are empty: the circuit's checkpoints are not part of
    /// the branch (`install` keeps the ones the track already has).
    pub network: TrackNetwork,
    pub geometry: BranchGeometry,
    /// Id of the branch segment (always 2).
    pub branch_segment: SegmentId,
    /// Gate, direction and elevation of the joker checkpoint, at the arc-length midpoint of the branch segment.
    pub joker_gate: (LineSegment, Vec2, f32),
}

impl CompiledBranch {
    /// The joker checkpoint with the given id, on the branch segment.
    pub fn joker_checkpoint(&self, id: usize) -> Checkpoint {
        let (gate, direction, elevation) = self.joker_gate;
        Checkpoint::new(id, gate, direction, 1, false)
            .with_segment(self.branch_segment)
            .with_joker(true)
            .with_elevation(elevation)
    }
}

/// Tolerance of the anchor rule (guard 2, m).
const ANCHOR_TOLERANCE: f32 = 0.01;
/// Waypoint spacing of the compiled branch segment inside a junction and on the free road (m).
const JUNCTION_WAYPOINT_STEP: f32 = 2.0;
const ROAD_WAYPOINT_STEP: f32 = 4.0;
/// Extra radius the free road needs beyond half its width (guard 5, spec 097, m).
const ROAD_RADIUS_MARGIN: f32 = 3.0;
/// Smallest road width the layout accepts (m).
const MIN_ROAD_WIDTH: f32 = 4.0;
/// Tolerance on the divider gap rule (guard 4, m): the nose gap is measured on the sampled centreline.
const DIVIDER_GAP_TOLERANCE: f32 = 0.05;

impl BranchLayout {
    /// Compiles the layout on `track`'s main spline into a network. Pure: the same layout and main spline give the
    /// same network.
    pub fn compile(&self, track: &Track) -> Result<CompiledBranch, BranchKitError> {
        let geometry = self.compile_geometry(track)?;
        let (network, joker_gate) = self.build_network(&track.spline, &geometry)?;
        Ok(CompiledBranch { network, geometry, branch_segment: SegmentId(2), joker_gate })
    }

    /// Junction components, free road and guards 1-7, without the network.
    pub fn compile_geometry(&self, track: &Track) -> Result<BranchGeometry, BranchKitError> {
        let main = &track.spline;
        self.check_parameters(main)?;

        // Guard 2.
        let split_waypoint = anchor_waypoint(main, self.split.s, JunctionRole::Entry)?;
        let merge_waypoint = anchor_waypoint(main, self.merge.s, JunctionRole::Exit)?;

        // Guard 1.
        let split = build_junction(main, &self.split, JunctionRole::Entry, self.side, self.road_width)?;
        let merge = build_junction(main, &self.merge, JunctionRole::Exit, self.side, self.road_width)?;
        self.check_order(main)?;
        let wraps = main.closed && self.merge.s < self.split.s;

        // Guard 3: both noses lie inside their junctions.
        for (comp, junction, role) in
            [(&self.split, &split, JunctionRole::Entry), (&self.merge, &merge, JunctionRole::Exit)]
        {
            if comp.divider_gap < NOSE_GAP || junction.nose.is_none() {
                return Err(BranchKitError::NoseOutsideJunction { junction: role });
            }
        }

        // Guard 1 (joint kink), through the free road.
        let interior: Vec<Vec2> = self.road_waypoints.iter().map(|w| w.point).collect();
        let road = free_road(&split, &merge, &interior)?;

        let mut centreline = split.centreline.clone();
        let road_start = centreline.len() - 1;
        centreline.extend_from_slice(&road.points[1..]);
        let road_end = centreline.len() - 1;
        centreline.extend_from_slice(&merge.centreline[1..]);

        let geometry = BranchGeometry {
            split,
            merge,
            road,
            centreline,
            road_start,
            road_end,
            split_waypoint,
            merge_waypoint,
            wraps,
        };
        self.check_divider(main, &geometry)?;
        self.check_road_radius(&geometry)?;
        Ok(geometry)
    }

    /// Guard 7 and the parameter ranges. Written so NaN fails.
    fn check_parameters(&self, main: &TrackSpline) -> Result<(), BranchKitError> {
        let bad = |name| Err(BranchKitError::InvalidParameter { name });
        if self.layout_id.trim().is_empty() || self.layout_id.eq_ignore_ascii_case("main") {
            return bad("layout_id");
        }
        if !(self.road_width >= MIN_ROAD_WIDTH && self.road_width.is_finite()) {
            return bad("road_width");
        }
        if self.road_waypoints.iter().any(|w| !w.point.is_finite() || !(w.width > 0.0 && w.width.is_finite())) {
            return bad("road_waypoints");
        }
        if main.samples.len() < 2 || main.waypoints.len() < 3 {
            return bad("spline");
        }
        Ok(())
    }

    /// Guard 6: the split span, then the merge span, along the driving direction. Wraps across the start line on a
    /// closed main spline.
    fn check_order(&self, main: &TrackSpline) -> Result<(), BranchKitError> {
        let span = |c: &JunctionComponent| match c.kind {
            JunctionShape::Taper => c.length,
            JunctionShape::TurnOff { angle_deg } => {
                let theta = angle_deg.to_radians();
                c.length / theta * theta.sin()
            }
        };
        let total = main.total_length;
        let ahead = if main.closed { (self.merge.s - self.split.s).rem_euclid(total) } else { self.merge.s - self.split.s };
        if ahead > span(&self.split) + span(&self.merge) && ahead < total {
            Ok(())
        } else {
            Err(BranchKitError::JunctionOrder)
        }
    }

    /// Guard 4: between the two noses the edge gap between the main road and the branch stays at the nose gap.
    fn check_divider(&self, main: &TrackSpline, geom: &BranchGeometry) -> Result<(), BranchKitError> {
        let first = geom.split.nose.map_or(0, |n| n.index);
        let last = geom.road_end + geom.merge.nose.map_or(0, |n| n.index);
        let arcs = cumulative_lengths(&geom.centreline);
        for i in first..=last.max(first) {
            let proj = main.project_point(geom.centreline[i]);
            let gap = proj.distance_to_spline - proj.track_width * 0.5 - self.road_width * 0.5;
            if gap < NOSE_GAP - DIVIDER_GAP_TOLERANCE {
                return Err(BranchKitError::DividerTooNarrow { s: arcs[i] });
            }
        }
        Ok(())
    }

    /// Guard 5: the free road's tightest radius, from turn angles over a 2 m stencil.
    fn check_road_radius(&self, geom: &BranchGeometry) -> Result<(), BranchKitError> {
        let min_radius = self.road_width * 0.5 + ROAD_RADIUS_MARGIN;
        let pts = &geom.road.points;
        let arcs = cumulative_lengths(pts);
        const STENCIL: usize = 2;
        for i in STENCIL..pts.len().saturating_sub(STENCIL) {
            let (a, b) = (pts[i] - pts[i - STENCIL], pts[i + STENCIL] - pts[i]);
            let turn = a.perp_dot(b).atan2(a.dot(b)).abs();
            let ds = (a.length() + b.length()) * 0.5;
            let radius = if turn > 1e-6 { ds / turn } else { f32::INFINITY };
            if radius < min_radius {
                return Err(BranchKitError::RoadTooTight { s: arcs[i], radius });
            }
        }
        Ok(())
    }

    /// The branch segment's waypoints: the junction centrelines and the free road thinned to the waypoint steps, with
    /// the road waypoints kept as given and every other property interpolated between them.
    fn branch_waypoints(&self, main: &TrackSpline, geom: &BranchGeometry) -> Vec<TrackWaypoint> {
        let cl = &geom.centreline;
        let arcs = cumulative_lengths(cl);
        let first_surface = self.road_waypoints.first().and_then(|w| w.surface);
        let last_surface = self.road_waypoints.last().and_then(|w| w.surface);
        let elevation_at = |p: Vec2| main.project_point(p).elevation;
        let template = |i: usize, surface: Option<SurfaceType>| {
            let mut wp = TrackWaypoint::new(cl[i], self.road_width);
            wp.surface = surface;
            wp.elevation = elevation_at(cl[i]);
            wp
        };

        // Keys: indices into the centreline whose waypoints are fixed.
        let mut keys: Vec<(usize, TrackWaypoint)> = vec![(0, template(0, first_surface))];
        let push_key = |keys: &mut Vec<(usize, TrackWaypoint)>, i: usize, wp: TrackWaypoint| {
            if keys.last().map_or(true, |k| i > k.0) {
                keys.push((i, wp));
            }
        };
        push_key(&mut keys, geom.road_start, template(geom.road_start, first_surface));
        for (j, wp) in self.road_waypoints.iter().enumerate() {
            let i = geom.road_start + geom.road.control_index[2 + j];
            let mut key = wp.clone();
            key.point = cl[i];
            push_key(&mut keys, i, key);
        }
        push_key(&mut keys, geom.road_end, template(geom.road_end, last_surface));
        push_key(&mut keys, cl.len() - 1, template(cl.len() - 1, last_surface));

        let at = |i: usize| -> TrackWaypoint {
            let k = keys.partition_point(|key| key.0 <= i).max(1) - 1;
            let (i0, left) = (&keys[k].0, &keys[k].1);
            if *i0 == i || k + 1 >= keys.len() {
                let mut wp = left.clone();
                wp.point = cl[i];
                return wp;
            }
            let (i1, right) = (&keys[k + 1].0, &keys[k + 1].1);
            let t = ((arcs[i] - arcs[*i0]) / (arcs[*i1] - arcs[*i0]).max(1e-4)).clamp(0.0, 1.0);
            let mut wp = left.clone();
            wp.point = cl[i];
            wp.width = left.width + (right.width - left.width) * t;
            wp.elevation = left.elevation + (right.elevation - left.elevation) * t;
            wp.bank_angle = left.bank_angle + (right.bank_angle - left.bank_angle) * t;
            wp
        };

        let mut out = Vec::new();
        let mut last_arc = 0.0;
        let mut next_key = 0;
        for i in 0..cl.len() {
            while next_key < keys.len() && keys[next_key].0 < i {
                next_key += 1;
            }
            let is_key = next_key < keys.len() && keys[next_key].0 == i;
            let step = if i < geom.road_start || i > geom.road_end { JUNCTION_WAYPOINT_STEP } else { ROAD_WAYPOINT_STEP };
            let due = arcs[i] - last_arc >= step - 1e-4;
            let ahead_key = keys.get(if is_key { next_key + 1 } else { next_key }).map(|k| arcs[k.0] - arcs[i]);
            let crowds_key = !is_key && ahead_key.is_some_and(|d| d < step * 0.5);
            if i == 0 || is_key || i == cl.len() - 1 || (due && !crowds_key) {
                out.push(at(i));
                last_arc = arcs[i];
            }
        }
        out
    }

    fn build_network(&self, main: &TrackSpline, geom: &BranchGeometry) -> Result<(TrackNetwork, (LineSegment, Vec2, f32)), BranchKitError> {
        let wps = &main.waypoints;
        let (sw, mw) = (geom.split_waypoint, geom.merge_waypoint);
        // The segments reach whole main waypoints, so a segment can only be cut at one of them.
        if !geom.wraps && sw == 0 {
            return Err(BranchKitError::InvalidParameter { name: "split.s" });
        }

        let socket_on_main = |s: f32, surface: Option<SurfaceType>| {
            let a = main.sample_at_distance(s);
            SplineSocket::new(a.point, a.tangent, a.width)
                .with_surface(surface.or(wps_surface(main, s)).unwrap_or(SurfaceType::Asphalt))
                .with_elevation(a.elevation)
        };
        let branch_surface = |first: bool| {
            let w = if first { self.road_waypoints.first() } else { self.road_waypoints.last() };
            w.and_then(|w| w.surface)
        };
        let branch_socket = |junction: &JunctionGeometry, s: f32, first: bool| {
            let a = main.sample_at_distance(s);
            let p = if first { junction.centreline[0] } else { junction.centreline[junction.centreline.len() - 1] };
            // The component starts and ends parallel to the main track, so the socket tangent is the main tangent.
            SplineSocket::new(p, a.tangent, self.road_width)
                .with_surface(branch_surface(first).or(wps_surface(main, s)).unwrap_or(SurfaceType::Asphalt))
                .with_elevation(a.elevation)
        };

        let split_in = socket_on_main(self.split.s, None);
        let split_e0 = split_in;
        let split_e1 = branch_socket(&geom.split, self.split.s, true);
        let merge_eg = socket_on_main(self.merge.s, None);
        let merge_i0 = merge_eg;
        let merge_i1 = branch_socket(&geom.merge, self.merge.s, false);

        let gore = self.gore_config(main, geom);
        let merge_config = self.merge_config(main, geom);
        let split_junction = RoadJunction::split(
            JunctionId(0),
            format!("{} Split", self.name),
            split_in,
            vec![split_e0, split_e1],
            Some(gore),
        );
        let merge_junction = RoadJunction::merge(
            JunctionId(1),
            format!("{} Merge", self.name),
            vec![merge_i0, merge_i1],
            merge_eg,
            Some(merge_config),
        );

        let n = wps.len();
        let sock = |j: u32, i: usize| Some(SocketId::new(JunctionId(j), i));
        let mut segments = Vec::new();
        let branch_wps = self.branch_waypoints(main, geom);
        let mut branch = RoadSegment::new(SegmentId(2), self.name.clone(), branch_wps).with_junctions(sock(0, 1), sock(1, 1));
        branch.recompute_samples(Some(&split_e1), Some(&merge_i1));

        let layouts;
        if !geom.wraps {
            let mut seg0 = RoadSegment::new(SegmentId(0), "Start / Finish Straight", wps[..=sw].to_vec())
                .with_junctions(sock(1, 0), sock(0, 0));
            let mut seg1 = RoadSegment::new(SegmentId(1), "Main Racing Line", wps[sw..=mw].to_vec())
                .with_junctions(sock(0, 0), sock(1, 0));
            let mut tail = wps[mw..].to_vec();
            tail.push(wps[0].clone());
            let mut seg3 = RoadSegment::new(SegmentId(3), "Return Straight", tail).with_junctions(sock(1, 0), sock(0, 0));
            seg0.recompute_samples(None, Some(&split_in));
            seg1.recompute_samples(Some(&split_e0), Some(&merge_i0));
            seg3.recompute_samples(Some(&merge_eg), None);
            segments.extend([seg0, seg1, branch, seg3]);
            layouts = (
                TrackLayout::new("main", "Standard Circuit", vec![SegmentId(0), SegmentId(1), SegmentId(3)], SegmentId(0)),
                TrackLayout::new(
                    self.layout_id.clone(),
                    layout_display_name(&self.layout_id),
                    vec![SegmentId(0), SegmentId(2), SegmentId(3)],
                    SegmentId(0),
                ),
            );
        } else {
            // The merge anchor comes first on the lap: the trunk runs from the merge to the split, and both routes
            // through the start line run from the split to the merge.
            let mut trunk = RoadSegment::new(SegmentId(0), "Trunk", wps[mw..=sw].to_vec()).with_junctions(sock(1, 0), sock(0, 0));
            let mut through = wps[sw..n].to_vec();
            through.extend_from_slice(&wps[..=mw]);
            let mut seg1 = RoadSegment::new(SegmentId(1), "Main Racing Line", through).with_junctions(sock(0, 0), sock(1, 0));
            trunk.recompute_samples(Some(&merge_eg), Some(&split_in));
            seg1.recompute_samples(Some(&split_e0), Some(&merge_i0));
            segments.extend([trunk, seg1, branch]);
            layouts = (
                TrackLayout::new("main", "Standard Circuit", vec![SegmentId(0), SegmentId(1)], SegmentId(1)),
                TrackLayout::new(
                    self.layout_id.clone(),
                    layout_display_name(&self.layout_id),
                    vec![SegmentId(0), SegmentId(2)],
                    SegmentId(2),
                ),
            );
        }

        let branch_seg = segments.iter().find(|s| s.id == SegmentId(2)).expect("branch segment");
        let mid = branch_seg.sample_at_distance(branch_seg.length * 0.5);
        let half_w = mid.width * 0.5;
        let joker_gate = (
            LineSegment::new(mid.point - mid.normal * half_w, mid.point + mid.normal * half_w),
            mid.tangent,
            mid.elevation,
        );

        let mut network = TrackNetwork {
            junctions: vec![split_junction, merge_junction],
            segments,
            layouts: vec![layouts.0, layouts.1],
            default_layout_id: "main".to_string(),
            ..Default::default()
        };
        let lengths: Vec<f32> = network
            .layouts
            .iter()
            .map(|l| l.segment_sequence.iter().filter_map(|id| network.get_segment(*id)).map(|s| s.length).sum())
            .collect();
        for (layout, len) in network.layouts.iter_mut().zip(lengths) {
            layout.total_lap_length = len;
        }
        network.validate().map_err(|_| BranchKitError::InvalidParameter { name: "network" })?;
        Ok((network, joker_gate))
    }

    /// Gore of the split: the apex of the component, the nose barrier and the length to the nose.
    fn gore_config(&self, main: &TrackSpline, geom: &BranchGeometry) -> GoreConfig {
        let junction = &geom.split;
        let (nose, angle) = nose_barrier(main, junction, self.nose_barrier);
        let length = junction.nose.map_or(0.0, |n| junction.apex.distance(n.point));
        GoreConfig {
            apex_point: junction.apex,
            divergence_angle: angle,
            gore_length: length,
            nose_barrier: nose,
            has_chevrons: false,
        }
    }

    fn merge_config(&self, main: &TrackSpline, geom: &BranchGeometry) -> MergeConfig {
        let junction = &geom.merge;
        let (_, angle) = nose_barrier(main, junction, self.nose_barrier);
        MergeConfig {
            convergence_point: junction.apex,
            merge_angle: angle,
            merge_length: junction.nose.map_or(0.0, |n| junction.apex.distance(n.point)),
        }
    }
}

/// Display name of a branch layout: "Joker Lap Detour" for `joker`.
fn layout_display_name(layout_id: &str) -> String {
    let mut chars = layout_id.chars();
    let head: String = chars.next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
    format!("{}{} Lap Detour", head, chars.as_str())
}

/// Surface of the main spline at arc length `s`.
fn wps_surface(main: &TrackSpline, s: f32) -> Option<SurfaceType> {
    Some(main.sample_at_distance(s).surface)
}

/// Index of the main waypoint whose arc length is `s` (guard 2).
fn anchor_waypoint(main: &TrackSpline, s: f32, junction: JunctionRole) -> Result<usize, BranchKitError> {
    let total = main.total_length;
    main.waypoints
        .iter()
        .position(|wp| {
            let d = main.project_point(wp.point).progress_distance;
            let diff = (d - s).abs();
            let diff = if main.closed { diff.min((diff - total).abs()) } else { diff };
            diff <= ANCHOR_TOLERANCE
        })
        .ok_or(BranchKitError::AnchorOffWaypoint { junction })
}

/// The nose barrier of a junction: a [`NOSE_LENGTH`] segment at the nose point, perpendicular to the bisector of the
/// two road edges, and the divergence angle between them in degrees. A junction with no nose gives an empty barrier at
/// the apex.
pub fn nose_barrier(main: &TrackSpline, junction: &JunctionGeometry, barrier: BarrierType) -> (WallBarrier, f32) {
    let Some(nose) = junction.nose else {
        return (WallBarrier::new(junction.apex, junction.apex, barrier), 0.0);
    };
    let (dir_main, dir_branch) = edge_directions(main, junction, &nose);
    let bisector = (dir_main + dir_branch).normalize_or_zero();
    let across = bisector.perp();
    let (a, b) = (nose.point - across * (NOSE_LENGTH * 0.5), nose.point + across * (NOSE_LENGTH * 0.5));
    // Start on the main track's side.
    let (start, end) = if a.distance(nose.track_edge) <= b.distance(nose.track_edge) { (a, b) } else { (b, a) };
    let angle = dir_main.perp_dot(dir_branch).atan2(dir_main.dot(dir_branch)).abs().to_degrees();
    (WallBarrier::new(start, end, barrier), angle)
}

/// Unit driving directions of the main track edge and of the branch road at the nose: the main tangent beside the
/// nose's main edge point, and the direction of the branch centreline beside its branch edge point.
fn edge_directions(main: &TrackSpline, junction: &JunctionGeometry, nose: &NosePoint) -> (Vec2, Vec2) {
    let main_dir = main.project_point(nose.track_edge).tangent;
    let c = &junction.centreline;
    let branch = c
        .windows(2)
        .min_by(|a, b| {
            let da = LineSegment::new(a[0], a[1]).distance_to_point(nose.branch_edge);
            let db = LineSegment::new(b[0], b[1]).distance_to_point(nose.branch_edge);
            da.total_cmp(&db)
        })
        .map(|w| (w[1] - w[0]).normalize_or_zero())
        .unwrap_or(main_dir);
    // The centreline of an exit junction is stored in driving order too, so both point the way cars drive.
    (main_dir, branch)
}

// ---------------------------------------------------------------------------------------------------------------
// Installing a compiled branch
// ---------------------------------------------------------------------------------------------------------------

/// Writes a compiled branch into `track.network`. The checkpoint lists of layouts the track already has (matched by
/// id) are kept: they belong to the circuit, not to the branch.
pub fn install(track: &mut Track, compiled: CompiledBranch) {
    let mut network = compiled.network;
    if let Some(old) = &track.network {
        for layout in &mut network.layouts {
            if let Some(previous) = old.get_layout(&layout.id) {
                layout.checkpoint_ids = previous.checkpoint_ids.clone();
            }
        }
    }
    network.recompute_composite_splines();
    track.network = Some(network);
}

// ---------------------------------------------------------------------------------------------------------------
// Walls (Pillar IV)
// ---------------------------------------------------------------------------------------------------------------

/// Length of the blend from the main wall to the envelope wall at an anchor (m).
const CHAIN_BLEND: f32 = 5.0;
/// Spacing of the island and road wall vertices (m).
const WALL_STEP: f32 = 1.0;
/// A wall end this close to a point of the other piece's end counts as joined (m).
const END_TOUCH: f32 = 1e-3;
/// Reach of the ray that finds the main wall beside an anchor (m).
const ANCHOR_RAY_RANGE: f32 = 60.0;

/// True when main station `s` lies on the stretch the branch replaces: from the split anchor to the merge anchor.
fn in_replaced_span(main: &TrackSpline, from: f32, to: f32, s: f32) -> bool {
    if main.closed && to < from {
        s >= from || s <= to
    } else {
        s >= from && s <= to
    }
}

/// First point where the ray from `origin` along `dir` meets one of `walls`, farther than `min_range`. The wall ends
/// are extended by [`END_TOUCH`], so a ray through a shared end still hits.
pub fn ray_wall_hit<'a>(
    origin: Vec2,
    dir: Vec2,
    min_range: f32,
    max_range: f32,
    walls: impl IntoIterator<Item = &'a WallBarrier>,
) -> Option<Vec2> {
    let ray = LineSegment::new(origin, origin + dir * max_range);
    walls
        .into_iter()
        .filter_map(|w| {
            let d = w.segment.direction();
            let extended = LineSegment::new(w.segment.start - d * END_TOUCH, w.segment.end + d * END_TOUCH);
            extended.intersect_segment(&ray)
        })
        .map(|p| (p.distance(origin), p))
        .filter(|(range, _)| *range > min_range)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, p)| p)
}

/// Main wall on `side` between the anchors is replaced by arc length: pieces on the stretch are dropped, cut exactly
/// where the wall crosses the main normal at each anchor. Pure arc-length bookkeeping; nothing is searched against
/// the branch road. A layout that does not compile leaves the walls alone.
pub fn trim_main_walls(track: &mut Track) {
    let Some(layout) = track.branch_layout.clone() else { return };
    if layout.compile_geometry(track).is_err() {
        return;
    }
    let main = &track.spline;
    let sigma = layout.side.sign();
    let (from, to) = (layout.split.s, layout.merge.s);
    let rays: Vec<LineSegment> = [from, to]
        .iter()
        .map(|&s| {
            let c = main.sample_at_distance(s);
            LineSegment::new(c.point, c.point + c.normal * sigma * ANCHOR_RAY_RANGE)
        })
        .collect();

    let walls = if layout.side == Side::Left { &track.geometry.inner_walls } else { &track.geometry.outer_walls };
    let mut kept = Vec::with_capacity(walls.len());
    for w in walls {
        let len = w.segment.length();
        if len < 1e-4 {
            continue;
        }
        let dir = w.segment.direction();
        let mut cuts: Vec<f32> = vec![0.0, len];
        for ray in &rays {
            if let Some(p) = w.segment.intersect_segment(ray) {
                let t = (p - w.segment.start).dot(dir);
                if t > 1e-4 && t < len - 1e-4 {
                    cuts.push(t);
                }
            }
        }
        cuts.sort_by(f32::total_cmp);
        for pair in cuts.windows(2) {
            let (a, b) = (w.segment.start + dir * pair[0], w.segment.start + dir * pair[1]);
            let station = main.project_point((a + b) * 0.5).progress_distance;
            if !in_replaced_span(main, from, to, station) {
                kept.push(WallBarrier { segment: LineSegment::new(a, b), ..*w });
            }
        }
    }
    let kept = merge_collinear_walls(kept);
    if layout.side == Side::Left {
        track.geometry.inner_walls = kept;
    } else {
        track.geometry.outer_walls = kept;
    }
}

/// Walls of the branch region, all of them collidable network walls: the outer chains round both junctions, the
/// free road's outer wall, and the island between the two roads with its nose and cap.
///
/// Built from the compiled geometry, so nothing is searched and clipped. The main wall on the branch side is cut by
/// [`trim_main_walls`] where these walls take over.
pub fn build_walls(track: &Track, compiled: &CompiledBranch) -> Vec<WallBarrier> {
    let Some(layout) = &track.branch_layout else { return Vec::new() };
    let main = &track.spline;
    let geom = &compiled.geometry;
    let sigma = layout.side.sign();
    let barrier_type = track.dominant_barrier_type().unwrap_or(BarrierType::TireWall);
    let near = |points: &[Vec2]| {
        let (min, max) = points
            .iter()
            .fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        track
            .local_barrier_offset(min - Vec2::splat(20.0), max + Vec2::splat(20.0))
            .unwrap_or_else(|| track.effective_barrier_offset())
    };
    let (g_split, g_merge) = (near(&geom.split.centreline), near(&geom.merge.centreline));
    let g_island = g_split.min(g_merge);
    let main_walls = if layout.side == Side::Left { &track.geometry.inner_walls } else { &track.geometry.outer_walls };
    let elevation_at = |p: Vec2| main.project_point(p).elevation;

    let mut walls: Vec<WallBarrier> = Vec::new();
    let push_polyline = |walls: &mut Vec<WallBarrier>, points: &[Vec2], kind: BarrierType| {
        let pieces: Vec<WallBarrier> = points
            .windows(2)
            .filter(|w| w[0].distance(w[1]) > 1e-4)
            .map(|w| WallBarrier::with_elevation(w[0], w[1], kind, (elevation_at(w[0]) + elevation_at(w[1])) * 0.5))
            .collect();
        walls.extend(merge_collinear_walls(pieces));
    };

    // Outer chain of each junction, then the free road wall between them.
    let split_env = geom.split.outer_envelope(main, layout.side, layout.road_width);
    let merge_env = geom.merge.outer_envelope(main, layout.side, layout.road_width);
    let anchor_hit = |s: f32| {
        let c = main.sample_at_distance(s);
        ray_wall_hit(c.point, c.normal * sigma, c.width * 0.5, ANCHOR_RAY_RANGE, main_walls)
    };
    let mut split_chain = chain(&split_env, g_split, anchor_hit(layout.split.s), true);
    let mut merge_chain = chain(&merge_env, g_merge, anchor_hit(layout.merge.s), false);
    let free_out = |j: &JunctionGeometry, g: f32| j.free_end + j.free_heading.perp() * (sigma * (layout.road_width * 0.5 + g));
    let (q_split, q_merge) = (free_out(&geom.split, g_split), free_out(&geom.merge, g_merge));
    if let Some(last) = split_chain.last_mut() {
        *last = q_split;
    }
    if let Some(first) = merge_chain.first_mut() {
        *first = q_merge;
    }
    push_polyline(&mut walls, &split_chain, barrier_type);
    walls.extend(road_outer_wall(track, compiled, (g_split, g_merge), (q_split, q_merge), barrier_type));
    push_polyline(&mut walls, &merge_chain, barrier_type);

    // Island: divider walls along both road edges, the nose and the cap.
    let (nose_wall, _) = nose_barrier(main, &geom.split, layout.nose_barrier);
    let (cap_wall, _) = nose_barrier(main, &geom.merge, layout.nose_barrier);
    let (main_divider, branch_divider) = island_dividers(track, compiled, g_island, &nose_wall, &cap_wall);
    push_polyline(&mut walls, &main_divider, barrier_type);
    push_polyline(&mut walls, &branch_divider, barrier_type);
    for mut w in [nose_wall, cap_wall] {
        w.elevation = elevation_at(w.segment.midpoint());
        walls.push(w);
    }
    walls
}

/// The outer chain of one junction: wall points `gap` outside the envelope, blended over [`CHAIN_BLEND`] metres from
/// the main wall at the anchor (`hit`, where the main wall crosses the main normal there). Split: anchor first.
/// Merge: anchor last. The free-end point is replaced by the caller.
fn chain(env: &[EnvelopePoint], gap: f32, hit: Option<Vec2>, anchor_first: bool) -> Vec<Vec2> {
    let anchor = if anchor_first { 0 } else { env.len() - 1 };
    let delta = hit.map_or(0.0, |p| (p - env[anchor].centre).dot(env[anchor].outward) - env[anchor].wall_offset(gap));
    env.iter()
        .map(|e| {
            let d = (e.s - env[anchor].s).abs();
            let blend = if d < CHAIN_BLEND {
                let t = d / CHAIN_BLEND;
                1.0 - t * t * (3.0 - 2.0 * t)
            } else {
                0.0
            };
            e.centre + e.outward * (e.wall_offset(gap) + delta * blend)
        })
        .collect()
}

/// Outer wall of the free road, on the branch side, with the rules of spec 088 (a piece is dropped where it would lie
/// on a road, in the main wall gap, on a main wall or along one). Starts at `q.0` and ends at `q.1`.
fn road_outer_wall(
    track: &Track,
    compiled: &CompiledBranch,
    g: (f32, f32),
    q: (Vec2, Vec2),
    barrier_type: BarrierType,
) -> Vec<WallBarrier> {
    let Some(layout) = &track.branch_layout else { return Vec::new() };
    let main = &track.spline;
    let net = &compiled.network;
    let Some(seg) = net.get_segment(compiled.branch_segment) else { return Vec::new() };
    let sigma = layout.side.sign();
    let geom = &compiled.geometry;
    let (a0, a1) = (
        seg.project_point(geom.split.free_end).progress_distance,
        seg.project_point(geom.merge.free_end).progress_distance,
    );
    if a1 - a0 < 2.0 {
        return Vec::new();
    }

    // Vertices every WALL_STEP along the road part of the branch segment.
    let mut points = vec![q.0];
    let mut last_d = a0;
    for sample in seg.samples.iter().filter(|s| s.distance > a0 && s.distance < a1) {
        if sample.distance - last_d >= WALL_STEP {
            let t = (sample.distance - a0) / (a1 - a0);
            let gap = g.0 + (g.1 - g.0) * t;
            points.push(sample.point + sample.normal * (sigma * (sample.width * 0.5 + gap)));
            last_d = sample.distance;
        }
    }
    points.push(q.1);

    let main_walls = || track.geometry.inner_walls.iter().chain(&track.geometry.outer_walls);
    let main_gap = (0.5 * (g.0 + g.1) - 0.5).clamp(0.3, 3.5);
    let keep = |w: &WallBarrier| {
        let pts = [w.segment.start, w.segment.end, w.segment.midpoint()];
        let off_main_road = pts.iter().all(|&p| {
            let proj = main.project_point(p);
            let curb_extra = if proj.left_curb || proj.right_curb { 1.35 } else { 0.0 };
            proj.distance_to_spline >= proj.track_width * 0.5 + curb_extra + main_gap
        });
        let doubles_main_wall = pts.iter().all(|&p| main_walls().any(|m| m.segment.distance_to_point(p) < 0.5));
        off_main_road
            && !doubles_main_wall
            && !main_walls().any(|m| m.segment.intersect_segment(&w.segment).is_some())
            && wall_clear_of_roads(&net.segments, w, f32::INFINITY, 0.3)
    };

    let pieces: Vec<WallBarrier> = points
        .windows(2)
        .filter(|w| w[0].distance(w[1]) > 1e-4)
        .map(|w| {
            let elev = seg.project_point((w[0] + w[1]) * 0.5).elevation;
            WallBarrier::with_elevation(w[0], w[1], barrier_type, elev)
        })
        .filter(|w| keep(w))
        .collect();
    merge_collinear_walls(pieces)
}

/// The two divider walls of the island, in driving order, from the nose to the cap. Each stands `min(gap, (G - 1.6) / 2)`
/// from its road edge, where `G` is the edge gap, so they are 1.6 m apart at the nose. The first and last vertices are
/// the ends of the nose and cap barriers, which close the loop.
fn island_dividers(
    track: &Track,
    compiled: &CompiledBranch,
    gap: f32,
    nose: &WallBarrier,
    cap: &WallBarrier,
) -> (Vec<Vec2>, Vec<Vec2>) {
    let Some(layout) = &track.branch_layout else { return (Vec::new(), Vec::new()) };
    let main = &track.spline;
    let geom = &compiled.geometry;
    let (Some(first), Some(last)) = (geom.split.nose, geom.merge.nose) else { return (Vec::new(), Vec::new()) };
    let seg = compiled.network.get_segment(compiled.branch_segment);
    let (from, to) = (first.index, geom.road_end + last.index);
    if to <= from {
        return (Vec::new(), Vec::new());
    }

    let mut on_main = vec![nose.segment.start];
    let mut on_branch = vec![nose.segment.end];
    let mut last_p = geom.centreline[from];
    for i in from + 1..to {
        let p = geom.centreline[i];
        if p.distance(last_p) < WALL_STEP {
            continue;
        }
        last_p = p;
        let proj = main.project_point(p);
        let c = proj.closest_point;
        let out = (p - c).normalize_or_zero();
        let half_branch = seg.map_or(layout.road_width * 0.5, |s| s.project_point(p).track_width * 0.5);
        let g = proj.distance_to_spline - proj.track_width * 0.5 - half_branch;
        let o = gap.min((g - NOSE_LENGTH) * 0.5).max(0.0);
        on_main.push(c + out * (proj.track_width * 0.5 + o));
        on_branch.push(p - out * (half_branch + o));
    }
    on_main.push(cap.segment.start);
    on_branch.push(cap.segment.end);
    (on_main, on_branch)
}
