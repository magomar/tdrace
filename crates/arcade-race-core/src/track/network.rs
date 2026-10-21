use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;
use crate::track::geometry::{
    point_in_quad_2d, point_in_triangle_2d, BarrierType, LineSegment, SpawnPose, WallBarrier,
};
use crate::track::spline::{
    catmull_rom_1d, catmull_rom_2d, SplineProjection, SplineSample, TrackSpline, TrackWaypoint,
};

/// A unique handle identifying a road segment within a track network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SegmentId(pub u32);

/// A unique handle identifying a junction (split/merge/terminal) node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct JunctionId(pub u32);

/// Identifies a specific connection socket on a junction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SocketId {
    pub junction_id: JunctionId,
    pub socket_index: usize,
}

impl SocketId {
    pub const fn new(junction_id: JunctionId, socket_index: usize) -> Self {
        Self {
            junction_id,
            socket_index,
        }
    }
}

/// Geometric and physical boundary conditions at a segment connection port.
/// Guarantees C¹ positional and tangent continuity across connected splines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplineSocket {
    /// 2D world coordinates of the centerline connection point.
    pub point: Vec2,
    /// Outgoing tangent unit vector (direction of travel away from socket).
    pub tangent: Vec2,
    /// Inward-pointing normal vector (left normal: (-tangent.y, tangent.x)).
    pub normal: Vec2,
    /// Asphalt track width at the connection boundary (meters).
    pub width: f32,
    /// Road surface elevation above ground (meters).
    #[serde(default)]
    pub elevation: f32,
    /// Cross-slope banking angle (degrees).
    #[serde(default)]
    pub bank_angle: f32,
    /// Default surface type at junction boundary.
    #[serde(default = "default_asphalt")]
    pub surface: SurfaceType,
}

const fn default_asphalt() -> SurfaceType {
    SurfaceType::Asphalt
}

impl SplineSocket {
    /// Creates a new spline socket with standard 2D orientation and zero elevation/banking.
    pub fn new(point: Vec2, tangent: Vec2, width: f32) -> Self {
        let t = tangent.normalize_or_zero();
        let normal = Vec2::new(-t.y, t.x);
        Self {
            point,
            tangent: t,
            normal,
            width: width.max(1.0),
            elevation: 0.0,
            bank_angle: 0.0,
            surface: SurfaceType::Asphalt,
        }
    }

    pub const fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    pub const fn with_bank_angle(mut self, bank_angle: f32) -> Self {
        self.bank_angle = bank_angle;
        self
    }

    pub const fn with_surface(mut self, surface: SurfaceType) -> Self {
        self.surface = surface;
        self
    }

    /// Synthesizes an upstream ghost point P_{-1} behind the socket along -tangent.
    /// Used by Catmull-Rom interpolation to guarantee C¹ continuous derivatives at branch seams.
    #[inline]
    pub fn synthesize_ghost_point(&self, step_dist: f32) -> Vec2 {
        self.point - self.tangent * step_dist
    }

    /// Computes the left track edge coordinate at this socket.
    #[inline]
    pub fn left_edge(&self) -> Vec2 {
        self.point + self.normal * (self.width * 0.5)
    }

    /// Computes the right track edge coordinate at this socket.
    #[inline]
    pub fn right_edge(&self) -> Vec2 {
        self.point - self.normal * (self.width * 0.5)
    }
}

/// Configuration for the triangular paved runoff and nose wedge between diverging branches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoreConfig {
    /// Apex coordinate where the inner edges of diverging branches begin.
    pub apex_point: Vec2,
    /// Divergence angle between left and right branch centerlines in degrees.
    pub divergence_angle: f32,
    /// Length of the painted gore triangle in meters.
    pub gore_length: f32,
    /// Barrier installed at the gore apex (e.g. TireWall or concrete attenuator).
    pub nose_barrier: WallBarrier,
    /// Whether painted chevron hash markings are rendered inside the gore triangle.
    #[serde(default)]
    pub has_chevrons: bool,
}

impl GoreConfig {
    pub fn new(
        apex_point: Vec2,
        divergence_angle: f32,
        gore_length: f32,
        barrier_type: BarrierType,
    ) -> Self {
        let nose_radius = 1.2;
        let left_apex = apex_point + Vec2::new(-nose_radius, -nose_radius * 0.5);
        let right_apex = apex_point + Vec2::new(nose_radius, -nose_radius * 0.5);
        Self {
            apex_point,
            divergence_angle,
            gore_length,
            nose_barrier: WallBarrier::new(left_apex, right_apex, barrier_type),
            has_chevrons: true,
        }
    }
}

/// Configuration for the zipper convergence zone where incoming branches merge back into one trunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MergeConfig {
    /// Coordinate where incoming inner edges converge.
    pub convergence_point: Vec2,
    /// Angle between converging incoming branch centerlines in degrees.
    pub merge_angle: f32,
    /// Length of the merge acceleration zone in meters.
    pub merge_length: f32,
}

/// Type of topological junction connecting road segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JunctionKind {
    /// 1 incoming trunk splitting into 2 or more outgoing branches.
    Split {
        /// Entry socket receiving traffic from the ingress segment.
        ingress_socket: SplineSocket,
        /// Outgoing branch sockets (index 0 = Branch A / default, 1 = Branch B / alternate, etc.).
        egress_sockets: Vec<SplineSocket>,
        /// Optional geometric configuration of the bifurcation wedge / gore area.
        gore_config: Option<GoreConfig>,
    },
    /// 2 or more incoming branches merging into 1 outgoing trunk.
    Merge {
        /// Incoming branch sockets delivering traffic to the merge.
        ingress_sockets: Vec<SplineSocket>,
        /// Outgoing socket continuing traffic down the unified trunk.
        egress_socket: SplineSocket,
        /// Optional geometric merge taper configuration.
        merge_config: Option<MergeConfig>,
    },
    /// Open-ended terminal (for point-to-point rally or hillclimb stages).
    Terminal {
        socket: SplineSocket,
    },
}

/// Node representing a connection between road segments in a track network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadJunction {
    pub id: JunctionId,
    pub name: String,
    pub kind: JunctionKind,
}

impl RoadJunction {
    /// Creates a new split junction with 1 ingress socket and N egress branch sockets.
    pub fn split(
        id: JunctionId,
        name: impl Into<String>,
        ingress_socket: SplineSocket,
        egress_sockets: Vec<SplineSocket>,
        gore_config: Option<GoreConfig>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind: JunctionKind::Split {
                ingress_socket,
                egress_sockets,
                gore_config,
            },
        }
    }

    /// Creates a new merge junction with N ingress branch sockets and 1 egress socket.
    pub fn merge(
        id: JunctionId,
        name: impl Into<String>,
        ingress_sockets: Vec<SplineSocket>,
        egress_socket: SplineSocket,
        merge_config: Option<MergeConfig>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind: JunctionKind::Merge {
                ingress_sockets,
                egress_socket,
                merge_config,
            },
        }
    }

    /// Creates a terminal junction.
    pub fn terminal(id: JunctionId, name: impl Into<String>, socket: SplineSocket) -> Self {
        Self {
            id,
            name: name.into(),
            kind: JunctionKind::Terminal { socket },
        }
    }

    /// Retrieves an egress socket by index, if available on this junction.
    pub fn egress_socket(&self, index: usize) -> Option<&SplineSocket> {
        match &self.kind {
            JunctionKind::Split { egress_sockets, .. } => egress_sockets.get(index),
            JunctionKind::Merge { egress_socket, .. } if index == 0 => Some(egress_socket),
            _ => None,
        }
    }

    /// Retrieves an ingress socket by index, if available on this junction.
    pub fn ingress_socket(&self, index: usize) -> Option<&SplineSocket> {
        match &self.kind {
            JunctionKind::Split { ingress_socket, .. } if index == 0 => Some(ingress_socket),
            JunctionKind::Merge { ingress_sockets, .. } => ingress_sockets.get(index),
            JunctionKind::Terminal { socket } if index == 0 => Some(socket),
            _ => None,
        }
    }
}

/// Evaluates cubic Hermite transition width envelope W(u) across a road split transition zone.
/// Widens from w_trunk to (w1 + w2 + w_gore) over normalized parameter u in [0, 1].
#[inline]
pub fn compute_split_width_envelope(
    w_trunk: f32,
    w1: f32,
    w2: f32,
    w_gore: f32,
    u: f32,
) -> f32 {
    let t = u.clamp(0.0, 1.0);
    let s = 3.0 * t * t - 2.0 * t * t * t;
    let target_width = w1 + w2 + w_gore;
    w_trunk + (target_width - w_trunk) * s
}

/// A continuous spline ribbon forming a directed edge in the Track Network graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadSegment {
    pub id: SegmentId,
    pub name: String,
    /// Waypoints defining this segment's centerline and cross-section.
    pub waypoints: Vec<TrackWaypoint>,
    /// Dense, uniformly resampled arc-length spline samples.
    #[serde(default)]
    pub samples: Vec<SplineSample>,
    /// Total arc-length distance of this segment in meters.
    #[serde(default)]
    pub length: f32,
    /// Connection to upstream junction (entry). None if start of open circuit.
    #[serde(default)]
    pub entry_junction: Option<SocketId>,
    /// Connection to downstream junction (exit). None if open terminal.
    #[serde(default)]
    pub exit_junction: Option<SocketId>,
    /// Segment-specific wall barriers (left and right).
    #[serde(default)]
    pub walls: Vec<WallBarrier>,
}

impl RoadSegment {
    /// Creates a new road segment and automatically resamples its samples.
    pub fn new(id: SegmentId, name: impl Into<String>, waypoints: Vec<TrackWaypoint>) -> Self {
        let mut seg = Self {
            id,
            name: name.into(),
            waypoints,
            samples: Vec::new(),
            length: 0.0,
            entry_junction: None,
            exit_junction: None,
            walls: Vec::new(),
        };
        seg.recompute_samples(None, None);
        seg
    }

    pub const fn with_junctions(
        mut self,
        entry: Option<SocketId>,
        exit: Option<SocketId>,
    ) -> Self {
        self.entry_junction = entry;
        self.exit_junction = exit;
        self
    }

    /// Converts this road segment into a flat open `TrackSpline` using its waypoints and samples.
    pub fn to_spline(&self) -> TrackSpline {
        TrackSpline {
            waypoints: self.waypoints.clone(),
            closed: false,
            samples: self.samples.clone(),
            total_length: self.length,
            curves: Vec::new(),
            sample_segments: Vec::new(),
            waypoint_sample_indices: Vec::new(),
            projection_index: Default::default(),
        }
    }

    /// Computes lateral clearance gap G to another road segment at distance `s` along this segment (Spec 112).
    pub fn clearance_gap_to(&self, other: &RoadSegment, s: f32) -> Option<f32> {
        clearance_gap(self, other, s)
    }

    /// Recomputes dense spline samples with optional socket boundary constraints.
    ///
    /// When `entry_socket` is provided:
    /// - Waypoint 0 position is locked to socket.point
    /// - An analytical ghost point P_{-1} is synthesized along -socket.tangent for C¹ first-derivative continuity
    ///
    /// When `exit_socket` is provided:
    /// - The final waypoint position is locked to socket.point
    /// - An analytical ghost point P_{N} is synthesized along +socket.tangent
    pub fn recompute_samples(
        &mut self,
        entry_socket: Option<&SplineSocket>,
        exit_socket: Option<&SplineSocket>,
    ) {
        if self.waypoints.len() < 2 {
            self.samples.clear();
            self.length = 0.0;
            return;
        }

        // Apply boundary socket positions if specified
        if let Some(entry) = entry_socket {
            if let Some(first) = self.waypoints.first_mut() {
                first.point = entry.point;
                first.width = entry.width;
                first.elevation = entry.elevation;
                first.bank_angle = entry.bank_angle;
            }
        }
        if let Some(exit) = exit_socket {
            if let Some(last) = self.waypoints.last_mut() {
                last.point = exit.point;
                last.width = exit.width;
                last.elevation = exit.elevation;
                last.bank_angle = exit.bank_angle;
            }
        }

        let num_wp = self.waypoints.len();
        let segments = num_wp - 1;
        let steps_per_segment = 24;

        let mut raw_points = Vec::new();
        let mut raw_widths = Vec::new();
        let mut raw_left_curbs = Vec::new();
        let mut raw_right_curbs = Vec::new();
        let mut raw_surfaces = Vec::new();
        let mut raw_elevations = Vec::new();
        let mut raw_bank_angles = Vec::new();
        let mut raw_left_walls = Vec::new();
        let mut raw_right_walls = Vec::new();
        let mut raw_left_wall_dists = Vec::new();
        let mut raw_right_wall_dists = Vec::new();
        let mut raw_wall_types = Vec::new();
        let mut raw_left_runoffs = Vec::new();
        let mut raw_right_runoffs = Vec::new();

        let step_estimate = if num_wp >= 2 {
            (self.waypoints[1].point - self.waypoints[0].point).length().max(1.0)
        } else {
            10.0
        };

        for i in 0..segments {
            // P0 calculation
            let p0 = if i == 0 {
                if let Some(socket) = entry_socket {
                    socket.synthesize_ghost_point(step_estimate)
                } else {
                    self.waypoints[0].point
                }
            } else {
                self.waypoints[i - 1].point
            };

            let p1 = self.waypoints[i].point;
            let p2 = self.waypoints[i + 1].point;

            // P3 calculation
            let p3 = if i + 1 == segments {
                if let Some(socket) = exit_socket {
                    socket.point + socket.tangent * step_estimate
                } else {
                    self.waypoints[num_wp - 1].point
                }
            } else {
                self.waypoints[i + 2].point
            };

            let e0 = if i == 0 { self.waypoints[0].elevation } else { self.waypoints[i - 1].elevation };
            let e1 = self.waypoints[i].elevation;
            let e2 = self.waypoints[i + 1].elevation;
            let e3 = if i + 1 == segments { self.waypoints[num_wp - 1].elevation } else { self.waypoints[i + 2].elevation };

            let b0 = if i == 0 { self.waypoints[0].bank_angle } else { self.waypoints[i - 1].bank_angle };
            let b1 = self.waypoints[i].bank_angle;
            let b2 = self.waypoints[i + 1].bank_angle;
            let b3 = if i + 1 == segments { self.waypoints[num_wp - 1].bank_angle } else { self.waypoints[i + 2].bank_angle };

            let wp1 = &self.waypoints[i];
            let wp2 = &self.waypoints[i + 1];

            for s in 0..steps_per_segment {
                let t = s as f32 / steps_per_segment as f32;
                let pt = catmull_rom_2d(p0, p1, p2, p3, t);
                let elev = catmull_rom_1d(e0, e1, e2, e3, t).max(0.0);
                let bank = catmull_rom_1d(b0, b1, b2, b3, t);
                let w = wp1.width + (wp2.width - wp1.width) * t;

                let lc = if t < 0.5 { wp1.left_curb } else { wp2.left_curb };
                let rc = if t < 0.5 { wp1.right_curb } else { wp2.right_curb };
                let surf = (if t < 0.5 { wp1.surface } else { wp2.surface })
                    .or(wp1.surface)
                    .or(wp2.surface)
                    .unwrap_or(SurfaceType::Asphalt);
                let lw = if t < 0.5 { wp1.left_wall } else { wp2.left_wall };
                let rw = if t < 0.5 { wp1.right_wall } else { wp2.right_wall };
                let lwd = match (wp1.left_wall_distance, wp2.left_wall_distance) {
                    (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * t),
                    (Some(d1), None) => if t < 0.5 { Some(d1) } else { None },
                    (None, Some(d2)) => if t < 0.5 { None } else { Some(d2) },
                    (None, None) => None,
                };
                let rwd = match (wp1.right_wall_distance, wp2.right_wall_distance) {
                    (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * t),
                    (Some(d1), None) => if t < 0.5 { Some(d1) } else { None },
                    (None, Some(d2)) => if t < 0.5 { None } else { Some(d2) },
                    (None, None) => None,
                };
                let wt = if t < 0.5 { wp1.wall_type } else { wp2.wall_type };
                let l_ro = if t < 0.5 { wp1.left_runoff_surface } else { wp2.left_runoff_surface };
                let r_ro = if t < 0.5 { wp1.right_runoff_surface } else { wp2.right_runoff_surface };

                raw_points.push(pt);
                raw_widths.push(w);
                raw_left_curbs.push(lc);
                raw_right_curbs.push(rc);
                raw_surfaces.push(surf);
                raw_elevations.push(elev);
                raw_bank_angles.push(bank);
                raw_left_walls.push(lw);
                raw_right_walls.push(rw);
                raw_left_wall_dists.push(lwd);
                raw_right_wall_dists.push(rwd);
                raw_wall_types.push(wt);
                raw_left_runoffs.push(l_ro);
                raw_right_runoffs.push(r_ro);
            }
        }

        // Push final endpoint
        let last_wp = &self.waypoints[num_wp - 1];
        raw_points.push(last_wp.point);
        raw_widths.push(last_wp.width);
        raw_left_curbs.push(last_wp.left_curb);
        raw_right_curbs.push(last_wp.right_curb);
        raw_surfaces.push(last_wp.surface.unwrap_or(SurfaceType::Asphalt));
        raw_elevations.push(last_wp.elevation);
        raw_bank_angles.push(last_wp.bank_angle);
        raw_left_walls.push(last_wp.left_wall);
        raw_right_walls.push(last_wp.right_wall);
        raw_left_wall_dists.push(last_wp.left_wall_distance);
        raw_right_wall_dists.push(last_wp.right_wall_distance);
        raw_wall_types.push(last_wp.wall_type);
        raw_left_runoffs.push(last_wp.left_runoff_surface);
        raw_right_runoffs.push(last_wp.right_runoff_surface);

        // Compute cumulative arc-length distances
        let n = raw_points.len();
        let mut cum_dist = Vec::with_capacity(n);
        cum_dist.push(0.0);
        let mut total = 0.0;

        for i in 0..n - 1 {
            let d = (raw_points[i + 1] - raw_points[i]).length();
            total += d;
            cum_dist.push(total);
        }
        self.length = total;

        // Construct samples with smooth normalized tangents
        let mut samples = Vec::with_capacity(n);
        for i in 0..n {
            let tangent = if i == 0 && entry_socket.is_some() {
                entry_socket.unwrap().tangent
            } else if i + 1 == n && exit_socket.is_some() {
                exit_socket.unwrap().tangent
            } else if i == 0 {
                (raw_points[1] - raw_points[0]).normalize_or_zero()
            } else if i + 1 == n {
                (raw_points[n - 1] - raw_points[n - 2]).normalize_or_zero()
            } else {
                (raw_points[i + 1] - raw_points[i - 1]).normalize_or_zero()
            };

            let normal = Vec2::new(-tangent.y, tangent.x);

            samples.push(SplineSample {
                point: raw_points[i],
                tangent,
                normal,
                distance: cum_dist[i],
                width: raw_widths[i],
                left_curb: raw_left_curbs[i],
                right_curb: raw_right_curbs[i],
                surface: raw_surfaces[i],
                elevation: raw_elevations[i],
                bank_angle: raw_bank_angles[i],
                is_bridge: false,
                grade_slope: 0.0,
                vertical_curvature: 0.0,
                left_wall: raw_left_walls[i],
                right_wall: raw_right_walls[i],
                left_wall_distance: raw_left_wall_dists[i],
                right_wall_distance: raw_right_wall_dists[i],
                wall_type: raw_wall_types[i],
                left_runoff_surface: raw_left_runoffs[i],
                right_runoff_surface: raw_right_runoffs[i],
            });
        }

        self.samples = samples;
    }

    /// Evaluates spline sample at a given arc-length distance along this segment.
    pub fn sample_at_distance(&self, distance: f32) -> SplineSample {
        if self.samples.is_empty() {
            return SplineSample {
                point: Vec2::ZERO,
                tangent: Vec2::X,
                normal: Vec2::Y,
                distance: 0.0,
                width: 10.0,
                left_curb: false,
                right_curb: false,
                surface: SurfaceType::Asphalt,
                elevation: 0.0,
                bank_angle: 0.0,
                is_bridge: false,
                grade_slope: 0.0,
                vertical_curvature: 0.0,
                left_wall: true,
                right_wall: true,
                left_wall_distance: None,
                right_wall_distance: None,
                wall_type: None,
                left_runoff_surface: None,
                right_runoff_surface: None,
            };
        }

        let clamped_dist = distance.clamp(0.0, self.length);
        let n = self.samples.len();

        let idx = match self
            .samples
            .binary_search_by(|s| s.distance.partial_cmp(&clamped_dist).unwrap())
        {
            Ok(i) => return self.samples[i],
            Err(i) => i,
        };

        if idx == 0 {
            return self.samples[0];
        }
        if idx >= n {
            return self.samples[n - 1];
        }

        let s0 = &self.samples[idx - 1];
        let s1 = &self.samples[idx];
        let seg_len = (s1.distance - s0.distance).max(1e-6);
        let t = (clamped_dist - s0.distance) / seg_len;

        SplineSample {
            point: s0.point.lerp(s1.point, t),
            tangent: s0.tangent.lerp(s1.tangent, t).normalize_or_zero(),
            normal: s0.normal.lerp(s1.normal, t).normalize_or_zero(),
            distance: clamped_dist,
            width: s0.width + (s1.width - s0.width) * t,
            left_curb: if t < 0.5 { s0.left_curb } else { s1.left_curb },
            right_curb: if t < 0.5 { s0.right_curb } else { s1.right_curb },
            surface: s0.surface,
            elevation: s0.elevation + (s1.elevation - s0.elevation) * t,
            bank_angle: s0.bank_angle + (s1.bank_angle - s0.bank_angle) * t,
            is_bridge: if t < 0.5 { s0.is_bridge } else { s1.is_bridge },
            grade_slope: 0.0,
            vertical_curvature: 0.0,
            left_wall: if t < 0.5 { s0.left_wall } else { s1.left_wall },
            right_wall: if t < 0.5 { s0.right_wall } else { s1.right_wall },
            left_wall_distance: s0.left_wall_distance,
            right_wall_distance: s0.right_wall_distance,
            wall_type: s0.wall_type,
            left_runoff_surface: s0.left_runoff_surface,
            right_runoff_surface: s0.right_runoff_surface,
        }
    }

    /// Projects any 2D world position onto this road segment centerline.
    pub fn project_point(&self, pos: Vec2) -> SplineProjection {
        if self.samples.len() < 2 {
            return SplineProjection {
                closest_point: pos,
                distance_to_spline: f32::INFINITY,
                lateral_offset: 0.0,
                progress_distance: 0.0,
                normalized_progress: 0.0,
                tangent: Vec2::X,
                normal: Vec2::Y,
                track_width: 10.0,
                left_curb: false,
                right_curb: false,
                is_on_track: false,
                is_on_curb: false,
                base_surface: SurfaceType::Asphalt,
                elevation: 0.0,
                bank_angle: 0.0,
                is_bridge: false,
                grade_slope: 0.0,
                vertical_curvature: 0.0,
                left_wall_distance: None,
                right_wall_distance: None,
                left_runoff_surface: None,
                right_runoff_surface: None,
            };
        }

        let mut best_dist_sq = f32::INFINITY;
        let mut best_point = Vec2::ZERO;
        let mut best_sample_idx = 0;

        for i in 0..self.samples.len() - 1 {
            let p0 = self.samples[i].point;
            let p1 = self.samples[i + 1].point;
            let seg = LineSegment::new(p0, p1);
            let cp = seg.closest_point(pos);
            let d_sq = (pos - cp).length_squared();

            if d_sq < best_dist_sq {
                best_dist_sq = d_sq;
                best_point = cp;
                best_sample_idx = i;
            }
        }

        let s0 = &self.samples[best_sample_idx];
        let s1 = &self.samples[(best_sample_idx + 1).min(self.samples.len() - 1)];
        let seg_len = (s1.point - s0.point).length();
        let best_t = if seg_len > 1e-4 {
            (best_point - s0.point).length() / seg_len
        } else {
            0.0
        };
        let best_progress = s0.distance + best_t * seg_len;
        let tangent = s0.tangent.lerp(s1.tangent, best_t).normalize_or_zero();
        let normal = Vec2::new(-tangent.y, tangent.x);
        let track_width = s0.width + (s1.width - s0.width) * best_t;
        let elevation = s0.elevation + (s1.elevation - s0.elevation) * best_t;
        let bank_angle = s0.bank_angle + (s1.bank_angle - s0.bank_angle) * best_t;

        let delta = pos - best_point;
        let dist = delta.length();
        let cross = tangent.x * delta.y - tangent.y * delta.x;
        let lateral_offset = if cross >= 0.0 { -dist } else { dist };

        let half_w = track_width * 0.5;
        let curb_w = TrackSpline::DEFAULT_CURB_WIDTH;
        let left_curb = if best_t < 0.5 { s0.left_curb } else { s1.left_curb };
        let right_curb = if best_t < 0.5 { s0.right_curb } else { s1.right_curb };

        let is_on_track = dist <= half_w;
        let is_on_curb = if is_on_track {
            false
        } else if lateral_offset < 0.0 {
            left_curb && dist <= half_w + curb_w
        } else {
            right_curb && dist <= half_w + curb_w
        };

        let norm_prog = if self.length > 1e-4 {
            (best_progress / self.length).clamp(0.0, 1.0)
        } else {
            0.0
        };

        SplineProjection {
            closest_point: best_point,
            distance_to_spline: dist,
            lateral_offset,
            progress_distance: best_progress,
            normalized_progress: norm_prog,
            tangent,
            normal,
            track_width,
            left_curb,
            right_curb,
            is_on_track,
            is_on_curb,
            base_surface: s0.surface,
            elevation,
            bank_angle,
            is_bridge: if best_t < 0.5 { s0.is_bridge } else { s1.is_bridge },
            grade_slope: 0.0,
            vertical_curvature: 0.0,
            left_wall_distance: s0.left_wall_distance,
            right_wall_distance: s0.right_wall_distance,
            left_runoff_surface: s0.left_runoff_surface,
            right_runoff_surface: s0.right_runoff_surface,
        }
    }

    /// Projects a 2D world position onto this road segment centerline with continuity constraint around `prev_progress`.
    /// Restricts candidate segments to within `max_dist_delta` of `prev_progress` to avoid skipping ahead
    /// or snapping to opposing track loops.
    pub fn project_point_continuity(
        &self,
        pos: Vec2,
        prev_progress: f32,
        max_dist_delta: f32,
    ) -> SplineProjection {
        if self.samples.len() < 2 {
            return self.project_point(pos);
        }

        let mut best_dist_sq = f32::INFINITY;
        let mut best_point = Vec2::ZERO;
        let mut best_progress = 0.0f32;
        let mut best_sample_idx = 0;
        let mut best_t = 0.0f32;
        let mut found_candidate = false;

        for i in 0..self.samples.len() - 1 {
            let seg_dist = self.samples[i].distance;
            let delta = (seg_dist - prev_progress).abs();

            if delta > max_dist_delta {
                continue;
            }

            let p0 = self.samples[i].point;
            let p1 = self.samples[i + 1].point;
            let seg = LineSegment::new(p0, p1);
            let cp = seg.closest_point(pos);
            let d_sq = (pos - cp).length_squared();

            if d_sq < best_dist_sq {
                best_dist_sq = d_sq;
                best_point = cp;
                best_sample_idx = i;
                let seg_len = seg.length();
                best_t = if seg_len > 1e-4 {
                    (cp - p0).length() / seg_len
                } else {
                    0.0
                };
                best_progress = self.samples[i].distance + best_t * seg_len;
                found_candidate = true;
            }
        }

        if !found_candidate {
            return self.project_point(pos);
        }

        let s0 = &self.samples[best_sample_idx];
        let s1 = &self.samples[(best_sample_idx + 1).min(self.samples.len() - 1)];
        let tangent = s0.tangent.lerp(s1.tangent, best_t).normalize_or_zero();
        let normal = Vec2::new(-tangent.y, tangent.x);
        let track_width = s0.width + (s1.width - s0.width) * best_t;
        let elevation = s0.elevation + (s1.elevation - s0.elevation) * best_t;
        let bank_angle = s0.bank_angle + (s1.bank_angle - s0.bank_angle) * best_t;

        let delta = pos - best_point;
        let dist = delta.length();
        let cross = tangent.x * delta.y - tangent.y * delta.x;
        let lateral_offset = if cross >= 0.0 { -dist } else { dist };

        let half_w = track_width * 0.5;
        let curb_w = TrackSpline::DEFAULT_CURB_WIDTH;
        let left_curb = if best_t < 0.5 { s0.left_curb } else { s1.left_curb };
        let right_curb = if best_t < 0.5 { s0.right_curb } else { s1.right_curb };

        let is_on_track = dist <= half_w;
        let is_on_curb = if is_on_track {
            false
        } else if lateral_offset < 0.0 {
            left_curb && dist <= half_w + curb_w
        } else {
            right_curb && dist <= half_w + curb_w
        };

        let norm_prog = if self.length > 1e-4 {
            (best_progress / self.length).clamp(0.0, 1.0)
        } else {
            0.0
        };

        SplineProjection {
            closest_point: best_point,
            distance_to_spline: dist,
            lateral_offset,
            progress_distance: best_progress,
            normalized_progress: norm_prog,
            tangent,
            normal,
            track_width,
            left_curb,
            right_curb,
            is_on_track,
            is_on_curb,
            base_surface: s0.surface,
            elevation,
            bank_angle,
            is_bridge: if best_t < 0.5 { s0.is_bridge } else { s1.is_bridge },
            grade_slope: 0.0,
            vertical_curvature: 0.0,
            left_wall_distance: s0.left_wall_distance,
            right_wall_distance: s0.right_wall_distance,
            left_runoff_surface: s0.left_runoff_surface,
            right_runoff_surface: s0.right_runoff_surface,
        }
    }
}

/// A named circuit layout formed by an ordered sequence of connected road segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackLayout {
    /// Identifier (e.g. "main", "grand_prix", "joker_lap", "club").
    pub id: String,
    /// Human-readable title displayed in menus.
    pub display_name: String,
    /// Optional run-once lead-in segment traversed exclusively at the start of the race (a launch chute).
    /// It is not part of `segment_sequence`: it leads into the loop through a merge junction, and laps 2+ never use it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_segment: Option<SegmentId>,
    /// Cyclic loop segments traversed during full racing laps.
    pub segment_sequence: Vec<SegmentId>,
    /// Whether this route forms a closed loop back to the starting segment.
    #[serde(default = "default_true")]
    pub is_closed: bool,
    /// Total nominal lap distance in meters.
    #[serde(default)]
    pub total_lap_length: f32,
    /// Segment ID containing the primary Start/Finish line.
    pub start_finish_segment: SegmentId,
    /// Checkpoints active along this specific route.
    #[serde(default)]
    pub checkpoint_ids: Vec<usize>,
}

const fn default_true() -> bool {
    true
}

impl TrackLayout {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        segment_sequence: Vec<SegmentId>,
        start_finish_segment: SegmentId,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            entry_segment: None,
            segment_sequence,
            is_closed: true,
            total_lap_length: 0.0,
            start_finish_segment,
            checkpoint_ids: Vec::new(),
        }
    }

    pub const fn with_entry_segment(mut self, entry_segment: Option<SegmentId>) -> Self {
        self.entry_segment = entry_segment;
        self
    }

    pub const fn with_closed(mut self, is_closed: bool) -> Self {
        self.is_closed = is_closed;
        self
    }

    pub const fn with_total_length(mut self, length: f32) -> Self {
        self.total_lap_length = length;
        self
    }

    pub fn with_checkpoints(mut self, checkpoint_ids: Vec<usize>) -> Self {
        self.checkpoint_ids = checkpoint_ids;
        self
    }

    /// Computes and caches total lap length by summing lengths of all constituent segments.
    pub fn recompute_lap_length(&mut self, network: &TrackNetwork) -> f32 {
        let len: f32 = self
            .segment_sequence
            .iter()
            .filter_map(|&sid| network.get_segment(sid))
            .map(|s| s.length)
            .sum();
        self.total_lap_length = len;
        len
    }
}

/// A walled launch chute (spec 103): an open-ended spur that ends in a rigid rear barrier, holds the packed
/// starting grid, and merges into the circuit through a `Merge` junction. It is the lead-in of the layouts'
/// `entry_segment`: cars drive it once at the race start and never on laps 2+.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaunchChuteConfig {
    /// Identifier of the spur road segment forming the launch pad.
    pub segment_id: SegmentId,
    /// Upstream terminal boundary with protective end wall.
    pub terminal_barrier: WallBarrier,
    /// Side barrier walls flanking the launch chute.
    pub side_barriers: Vec<WallBarrier>,
    /// Junction ID where the launch chute converges into the primary circuit.
    pub merge_junction_id: JunctionId,
    /// Packed starting grid spawn configurations.
    pub grid_slots: Vec<SpawnPose>,
    /// Launch pad surface (typically Concrete or Asphalt for launch traction).
    pub surface: SurfaceType,
    /// Width of the launch pad (meters), accommodating multi-car rows (typically 14.0-18.0 m).
    pub pad_width: f32,
}

/// Complete track network representing multi-branch circuits and layouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackNetwork {
    pub junctions: Vec<RoadJunction>,
    pub segments: Vec<RoadSegment>,
    pub layouts: Vec<TrackLayout>,
    pub default_layout_id: String,
    /// Launch chute that leads the layouts' `entry_segment` into the loop, if the circuit has one (spec 103).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_chute: Option<LaunchChuteConfig>,
    #[serde(default, skip_serializing)]
    pub cached_composite_splines: Vec<(String, TrackSpline)>,
}

impl TrackNetwork {
    /// Creates an empty track network.
    pub fn new() -> Self {
        Self {
            junctions: Vec::new(),
            segments: Vec::new(),
            layouts: Vec::new(),
            default_layout_id: "main".to_string(),
            launch_chute: None,
            cached_composite_splines: Vec::new(),
        }
    }

    /// Automatically promotes a legacy single `TrackSpline` into a 1-segment cyclic `TrackNetwork`.
    pub fn from_single_spline(spline: &TrackSpline) -> Self {
        let seg_id = SegmentId(0);
        let segment = RoadSegment {
            id: seg_id,
            name: "Main Track".to_string(),
            waypoints: spline.waypoints.clone(),
            samples: spline.samples.clone(),
            length: spline.total_length,
            entry_junction: None,
            exit_junction: None,
            walls: Vec::new(),
        };

        let layout = TrackLayout {
            id: "main".to_string(),
            display_name: "Default Course".to_string(),
            entry_segment: None,
            segment_sequence: vec![seg_id],
            is_closed: spline.closed,
            total_lap_length: spline.total_length,
            start_finish_segment: seg_id,
            checkpoint_ids: Vec::new(),
        };

        let mut net = Self {
            junctions: Vec::new(),
            segments: vec![segment],
            layouts: vec![layout],
            default_layout_id: "main".to_string(),
            launch_chute: None,
            cached_composite_splines: Vec::new(),
        };
        net.recompute_composite_splines();
        net
    }

    /// Recomputes and caches composite splines for each layout (Spec 084).
    pub fn recompute_composite_splines(&mut self) {
        let mut splines = Vec::with_capacity(self.layouts.len());
        for layout in &self.layouts {
            if let Some(spline) = self.build_composite_spline_for_layout(&layout.id) {
                splines.push((layout.id.clone(), spline));
            }
        }
        self.cached_composite_splines = splines;
    }

    /// Recomputes samples, connected junction socket positions/widths, layout lap lengths,
    /// and composite splines after editing a segment's waypoints.
    pub fn rebuild_segment_samples(&mut self, seg_id: SegmentId) {
        let Some((entry_sock_id, exit_sock_id, wp0, wp_last)) = self.get_segment(seg_id).and_then(|s| {
            let first = s.waypoints.first()?.clone();
            let last = s.waypoints.last()?.clone();
            Some((s.entry_junction, s.exit_junction, first, last))
        }) else {
            return;
        };

        let entry_shared_idxs: Vec<usize> = entry_sock_id
            .and_then(|sid| {
                let j = self.get_junction(sid.junction_id)?;
                Some(match &j.kind {
                    JunctionKind::Split { egress_sockets, .. } => (0..egress_sockets.len())
                        .filter(|&i| {
                            i != sid.socket_index
                                && !self.segments.iter().any(|s| {
                                    s.id != seg_id
                                        && self.is_branch_segment(s.id)
                                        && s.entry_junction == Some(SocketId::new(sid.junction_id, i))
                                })
                        })
                        .collect(),
                    JunctionKind::Merge { ingress_sockets, .. } => (0..ingress_sockets.len())
                        .filter(|&i| {
                            !self.segments.iter().any(|s| {
                                s.id != seg_id
                                    && self.is_branch_segment(s.id)
                                    && s.exit_junction == Some(SocketId::new(sid.junction_id, i))
                            })
                        })
                        .collect(),
                    _ => Vec::new(),
                })
            })
            .unwrap_or_default();

        let exit_shared_idxs: Vec<usize> = exit_sock_id
            .and_then(|sid| {
                let j = self.get_junction(sid.junction_id)?;
                Some(match &j.kind {
                    JunctionKind::Merge { ingress_sockets, .. } => (0..ingress_sockets.len())
                        .filter(|&i| {
                            i != sid.socket_index
                                && !self.segments.iter().any(|s| {
                                    s.id != seg_id
                                        && self.is_branch_segment(s.id)
                                        && s.exit_junction == Some(SocketId::new(sid.junction_id, i))
                                })
                        })
                        .collect(),
                    JunctionKind::Split { egress_sockets, .. } => (0..egress_sockets.len())
                        .filter(|&i| {
                            !self.segments.iter().any(|s| {
                                s.id != seg_id
                                    && self.is_branch_segment(s.id)
                                    && s.entry_junction == Some(SocketId::new(sid.junction_id, i))
                            })
                        })
                        .collect(),
                    _ => Vec::new(),
                })
            })
            .unwrap_or_default();

        if let Some(sid) = entry_sock_id {
            if let Some(j) = self.get_junction_mut(sid.junction_id) {
                match &mut j.kind {
                    JunctionKind::Split {
                        ingress_socket,
                        egress_sockets,
                        gore_config,
                    } => {
                        if let Some(sock) = egress_sockets.get_mut(sid.socket_index) {
                            sock.point = wp0.point;
                            sock.width = wp0.width.max(1.0);
                            sock.elevation = wp0.elevation;
                            sock.bank_angle = wp0.bank_angle;
                            if let Some(surf) = wp0.surface {
                                sock.surface = surf;
                            }
                        }
                        for idx in entry_shared_idxs {
                            if let Some(sock) = egress_sockets.get_mut(idx) {
                                sock.point = wp0.point;
                                sock.elevation = wp0.elevation;
                            }
                        }
                        ingress_socket.point = wp0.point;
                        ingress_socket.elevation = wp0.elevation;
                        if let Some(gore) = gore_config {
                            let delta = wp0.point - gore.apex_point;
                            gore.apex_point = wp0.point;
                            gore.nose_barrier.segment.start += delta;
                            gore.nose_barrier.segment.end += delta;
                            gore.nose_barrier.elevation = wp0.elevation;
                        }
                    }
                    JunctionKind::Merge {
                        ingress_sockets,
                        egress_socket,
                        merge_config,
                    } if sid.socket_index == 0 => {
                        egress_socket.point = wp0.point;
                        egress_socket.width = wp0.width.max(1.0);
                        egress_socket.elevation = wp0.elevation;
                        egress_socket.bank_angle = wp0.bank_angle;
                        if let Some(surf) = wp0.surface {
                            egress_socket.surface = surf;
                        }
                        for idx in entry_shared_idxs {
                            if let Some(sock) = ingress_sockets.get_mut(idx) {
                                sock.point = wp0.point;
                                sock.elevation = wp0.elevation;
                            }
                        }
                        if let Some(cfg) = merge_config {
                            cfg.convergence_point = wp0.point;
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(sid) = exit_sock_id {
            if let Some(j) = self.get_junction_mut(sid.junction_id) {
                match &mut j.kind {
                    JunctionKind::Split {
                        ingress_socket,
                        egress_sockets,
                        gore_config,
                    } if sid.socket_index == 0 => {
                        ingress_socket.point = wp_last.point;
                        ingress_socket.width = wp_last.width.max(1.0);
                        ingress_socket.elevation = wp_last.elevation;
                        ingress_socket.bank_angle = wp_last.bank_angle;
                        if let Some(surf) = wp_last.surface {
                            ingress_socket.surface = surf;
                        }
                        for idx in exit_shared_idxs {
                            if let Some(sock) = egress_sockets.get_mut(idx) {
                                sock.point = wp_last.point;
                                sock.elevation = wp_last.elevation;
                            }
                        }
                        if let Some(gore) = gore_config {
                            let delta = wp_last.point - gore.apex_point;
                            gore.apex_point = wp_last.point;
                            gore.nose_barrier.segment.start += delta;
                            gore.nose_barrier.segment.end += delta;
                            gore.nose_barrier.elevation = wp_last.elevation;
                        }
                    }
                    JunctionKind::Merge {
                        ingress_sockets,
                        egress_socket,
                        merge_config,
                    } => {
                        if let Some(sock) = ingress_sockets.get_mut(sid.socket_index) {
                            sock.point = wp_last.point;
                            sock.width = wp_last.width.max(1.0);
                            sock.elevation = wp_last.elevation;
                            sock.bank_angle = wp_last.bank_angle;
                            if let Some(surf) = wp_last.surface {
                                sock.surface = surf;
                            }
                        }
                        for idx in exit_shared_idxs {
                            if let Some(sock) = ingress_sockets.get_mut(idx) {
                                sock.point = wp_last.point;
                                sock.elevation = wp_last.elevation;
                            }
                        }
                        egress_socket.point = wp_last.point;
                        egress_socket.elevation = wp_last.elevation;
                        if let Some(cfg) = merge_config {
                            cfg.convergence_point = wp_last.point;
                        }
                    }
                    JunctionKind::Terminal { socket } if sid.socket_index == 0 => {
                        socket.point = wp_last.point;
                        socket.width = wp_last.width.max(1.0);
                        socket.elevation = wp_last.elevation;
                        socket.bank_angle = wp_last.bank_angle;
                        if let Some(surf) = wp_last.surface {
                            socket.surface = surf;
                        }
                    }
                    _ => {}
                }
            }
        }

        let entry_sock = entry_sock_id
            .and_then(|sid| self.get_junction(sid.junction_id).and_then(|j| j.egress_socket(sid.socket_index)))
            .copied();
        let exit_sock = exit_sock_id
            .and_then(|sid| self.get_junction(sid.junction_id).and_then(|j| j.ingress_socket(sid.socket_index)))
            .copied();

        if let Some(seg) = self.get_segment_mut(seg_id) {
            seg.recompute_samples(entry_sock.as_ref(), exit_sock.as_ref());
        }

        let self_clone_for_len: Vec<(usize, Vec<SegmentId>)> = self
            .layouts
            .iter()
            .enumerate()
            .map(|(i, l)| (i, l.segment_sequence.clone()))
            .collect();
        for (idx, seq) in self_clone_for_len {
            let len: f32 = seq
                .iter()
                .filter_map(|&sid| self.get_segment(sid))
                .map(|s| s.length)
                .sum();
            self.layouts[idx].total_lap_length = len;
        }
        self.recompute_composite_splines();
    }

    /// Aligns the connected split/merge junction socket tangents and normals of `seg_id`
    /// with `main_spline` at their current anchor positions, then refreshes `seg_id`'s samples.
    pub fn align_segment_junctions_to_spline(&mut self, seg_id: SegmentId, main_spline: &TrackSpline) {
        if main_spline.samples.len() < 2 {
            return;
        }
        let Some((entry_sock_id, exit_sock_id)) = self
            .get_segment(seg_id)
            .map(|s| (s.entry_junction, s.exit_junction))
        else {
            return;
        };

        let rotate_sockets_to_spline = |j: &mut RoadJunction| {
            match &mut j.kind {
                JunctionKind::Split {
                    ingress_socket,
                    egress_sockets,
                    ..
                } => {
                    let proj = main_spline.project_point(ingress_socket.point);
                    let new_t = proj.tangent.normalize_or_zero();
                    if new_t.length_squared() > 0.5 {
                        let new_n = Vec2::new(-new_t.y, new_t.x);
                        let old_t = ingress_socket.tangent.normalize_or_zero();
                        let (cos_a, sin_a) = if old_t.length_squared() > 0.5 {
                            (old_t.dot(new_t), old_t.perp_dot(new_t))
                        } else {
                            (1.0, 0.0)
                        };
                        for sock in egress_sockets.iter_mut() {
                            let v = sock.tangent;
                            let rot = Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a)
                                .normalize_or_zero();
                            sock.tangent = if rot.length_squared() > 0.5 { rot } else { new_t };
                            sock.normal = Vec2::new(-sock.tangent.y, sock.tangent.x);
                        }
                        ingress_socket.tangent = new_t;
                        ingress_socket.normal = new_n;
                    }
                }
                JunctionKind::Merge {
                    ingress_sockets,
                    egress_socket,
                    ..
                } => {
                    let proj = main_spline.project_point(egress_socket.point);
                    let new_t = proj.tangent.normalize_or_zero();
                    if new_t.length_squared() > 0.5 {
                        let new_n = Vec2::new(-new_t.y, new_t.x);
                        let old_t = egress_socket.tangent.normalize_or_zero();
                        let (cos_a, sin_a) = if old_t.length_squared() > 0.5 {
                            (old_t.dot(new_t), old_t.perp_dot(new_t))
                        } else {
                            (1.0, 0.0)
                        };
                        for sock in ingress_sockets.iter_mut() {
                            let v = sock.tangent;
                            let rot = Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a)
                                .normalize_or_zero();
                            sock.tangent = if rot.length_squared() > 0.5 { rot } else { new_t };
                            sock.normal = Vec2::new(-sock.tangent.y, sock.tangent.x);
                        }
                        egress_socket.tangent = new_t;
                        egress_socket.normal = new_n;
                    }
                }
                JunctionKind::Terminal { .. } => {}
            }
        };

        if let Some(sid) = entry_sock_id {
            if let Some(j) = self.get_junction_mut(sid.junction_id) {
                rotate_sockets_to_spline(j);
            }
        }
        if let Some(sid) = exit_sock_id {
            if let Some(j) = self.get_junction_mut(sid.junction_id) {
                rotate_sockets_to_spline(j);
            }
        }

        let entry_sock = entry_sock_id
            .and_then(|sid| self.get_junction(sid.junction_id).and_then(|j| j.egress_socket(sid.socket_index)))
            .copied();
        let exit_sock = exit_sock_id
            .and_then(|sid| self.get_junction(sid.junction_id).and_then(|j| j.ingress_socket(sid.socket_index)))
            .copied();
        if let Some(seg) = self.get_segment_mut(seg_id) {
            seg.recompute_samples(entry_sock.as_ref(), exit_sock.as_ref());
        }
    }

    /// Returns a pre-cached composite spline for the given layout ID, if available (Spec 084).
    pub fn composite_spline_for_layout(&self, layout_id: &str) -> Option<&TrackSpline> {
        self.cached_composite_splines
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case(layout_id))
            .map(|(_, s)| s)
    }

    pub fn get_segment(&self, id: SegmentId) -> Option<&RoadSegment> {
        self.segments.iter().find(|s| s.id == id)
    }

    pub fn get_segment_mut(&mut self, id: SegmentId) -> Option<&mut RoadSegment> {
        self.segments.iter_mut().find(|s| s.id == id)
    }

    pub fn get_junction(&self, id: JunctionId) -> Option<&RoadJunction> {
        self.junctions.iter().find(|j| j.id == id)
    }

    pub fn get_junction_mut(&mut self, id: JunctionId) -> Option<&mut RoadJunction> {
        self.junctions.iter_mut().find(|j| j.id == id)
    }

    pub fn get_layout(&self, id: &str) -> Option<&TrackLayout> {
        self.layouts.iter().find(|l| l.id.eq_ignore_ascii_case(id))
    }

    pub fn active_or_default_layout(&self, layout_id: Option<&str>) -> Option<&TrackLayout> {
        if let Some(id) = layout_id {
            if let Some(layout) = self.get_layout(id) {
                return Some(layout);
            }
        }
        self.get_layout(&self.default_layout_id)
            .or_else(|| self.layouts.first())
    }

    /// Returns the network segments that are not on the default layout (e.g. Joker Lap detours, launch chutes, alternate routes).
    pub fn branch_segments(&self) -> Vec<&RoadSegment> {
        let default_seq = self
            .active_or_default_layout(None)
            .map(|l| &l.segment_sequence[..])
            .unwrap_or(&[]);
        if default_seq.is_empty() {
            self.segments.iter().filter(|s| s.id.0 != 0).collect()
        } else {
            self.segments.iter().filter(|s| !default_seq.contains(&s.id)).collect()
        }
    }

    /// Returns true if the segment with the given id is a branch segment (not part of the default layout sequence).
    pub fn is_branch_segment(&self, id: SegmentId) -> bool {
        let default_seq = self
            .active_or_default_layout(None)
            .map(|l| &l.segment_sequence[..])
            .unwrap_or(&[]);
        if default_seq.is_empty() {
            id.0 != 0
        } else {
            !default_seq.contains(&id)
        }
    }

    /// Returns the segment for the launch chute if configured (Spec 103).
    pub fn launch_chute_segment(&self) -> Option<&RoadSegment> {
        let chute_id = self.launch_chute.as_ref()?.segment_id;
        self.get_segment(chute_id)
    }

    /// Returns the network segments that belong to a joker / alternate branch layout (Spec 081 / 102).
    pub fn joker_segments(&self) -> Vec<&RoadSegment> {
        let default_seq = self
            .active_or_default_layout(None)
            .map(|l| &l.segment_sequence[..])
            .unwrap_or(&[]);
        let chute = self.launch_chute.as_ref().map(|c| c.segment_id);
        self.segments
            .iter()
            .filter(|seg| {
                Some(seg.id) != chute
                    && !default_seq.contains(&seg.id)
                    && self.layouts.iter().any(|l| l.segment_sequence.contains(&seg.id))
            })
            .collect()
    }

    /// Synthesizes a composite, continuous `TrackSpline` from an ordered sequence of segments in a layout.
    /// Enables transparent backwards compatibility with systems requiring a flat `TrackSpline`.
    pub fn build_composite_spline_for_layout(&self, layout_id: &str) -> Option<TrackSpline> {
        let layout = self.get_layout(layout_id)?;
        let mut combined_waypoints: Vec<TrackWaypoint> = Vec::new();

        for (seg_idx, seg_id) in layout.segment_sequence.iter().enumerate() {
            let seg = self.get_segment(*seg_id)?;
            if seg.waypoints.is_empty() {
                continue;
            }

            // Skip the first waypoint of subsequent segments if it duplicates the previous segment's end
            let start_idx = if seg_idx > 0 && !combined_waypoints.is_empty() {
                let prev_end = combined_waypoints.last().unwrap().point;
                if (seg.waypoints[0].point - prev_end).length() < 0.1 {
                    1
                } else {
                    0
                }
            } else {
                0
            };

            for wp in &seg.waypoints[start_idx..] {
                combined_waypoints.push(wp.clone());
            }
        }

        if combined_waypoints.len() < 3 {
            return None;
        }

        Some(TrackSpline::new(combined_waypoints, layout.is_closed))
    }

    /// The loop segment that follows a layout's `entry_segment`: the one in `segment_sequence` whose entry
    /// sits on the junction the entry segment ends in. `None` when the layout has no entry segment.
    pub fn entry_continuation_segment(&self, layout: &TrackLayout) -> Option<SegmentId> {
        let junction = self.get_segment(layout.entry_segment?)?.exit_junction?.junction_id;
        layout.segment_sequence.iter().copied().find(|&sid| {
            self.get_segment(sid).and_then(|s| s.entry_junction).is_some_and(|e| e.junction_id == junction)
        })
    }

    /// Synthesizes the open route of the first lap for a layout with an `entry_segment`: the entry segment, then
    /// the loop from the segment it merges into round to the merge point. Cars that start in the launch chute
    /// follow this route until they have merged, then the closed composite spline of the layout.
    pub fn build_entry_spline_for_layout(&self, layout_id: &str) -> Option<TrackSpline> {
        let layout = self.get_layout(layout_id)?;
        let entry_id = layout.entry_segment?;
        let start = layout.segment_sequence.iter().position(|&sid| Some(sid) == self.entry_continuation_segment(layout))?;
        let route = std::iter::once(entry_id)
            .chain(layout.segment_sequence[start..].iter().copied())
            .chain(layout.segment_sequence[..start].iter().copied());

        let mut waypoints: Vec<TrackWaypoint> = Vec::new();
        for seg_id in route {
            let seg = self.get_segment(seg_id)?;
            let skip = usize::from(waypoints.last().zip(seg.waypoints.first()).is_some_and(|(a, b)| (a.point - b.point).length() < 0.1));
            waypoints.extend(seg.waypoints[skip.min(seg.waypoints.len())..].iter().cloned());
        }
        (waypoints.len() >= 3).then(|| TrackSpline::new(waypoints, false))
    }

    /// Validates graph consistency, socket alignments, and layout sequences.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.segments.is_empty() {
            errors.push("TrackNetwork contains zero segments".to_string());
        }

        if self.layouts.is_empty() {
            errors.push("TrackNetwork defines zero layouts".to_string());
        }

        // Validate segments
        for seg in &self.segments {
            if seg.waypoints.len() < 2 {
                errors.push(format!(
                    "Segment {:?} ({}) has fewer than 2 waypoints",
                    seg.id, seg.name
                ));
            }
        }

        // Validate layouts
        for layout in &self.layouts {
            if layout.segment_sequence.is_empty() {
                errors.push(format!("Layout '{}' has an empty segment sequence", layout.id));
                continue;
            }

            for seg_id in &layout.segment_sequence {
                if self.get_segment(*seg_id).is_none() {
                    errors.push(format!(
                        "Layout '{}' references non-existent SegmentId {:?}",
                        layout.id, seg_id
                    ));
                }
            }

            if let Some(entry) = layout.entry_segment {
                if self.get_segment(entry).is_none() {
                    errors.push(format!("Layout '{}' has non-existent entry SegmentId {:?}", layout.id, entry));
                } else if layout.segment_sequence.contains(&entry) {
                    errors.push(format!("Layout '{}' lists its entry segment {:?} in the cyclic sequence", layout.id, entry));
                } else if self.entry_continuation_segment(layout).is_none() {
                    errors.push(format!("Layout '{}': entry segment {:?} does not lead into the loop", layout.id, entry));
                }
            }
        }

        if let Some(chute) = &self.launch_chute {
            if self.get_segment(chute.segment_id).is_none() {
                errors.push(format!("Launch chute references non-existent SegmentId {:?}", chute.segment_id));
            }
            if self.get_junction(chute.merge_junction_id).is_none() {
                errors.push(format!("Launch chute references non-existent JunctionId {:?}", chute.merge_junction_id));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Samples surface type at a 2D world point across all road segments and junction throat/gore areas.
    pub fn sample_surface(&self, point: Vec2) -> Option<SurfaceType> {
        // 1. Check road segments (drivable ribbon and curbs). Road of any segment wins over another
        //    segment's curb, since a branch's road can overlap a neighbour's curb near a junction.
        let mut on_curb = false;
        for seg in &self.segments {
            if seg.samples.len() >= 2 {
                let seg_proj = seg.project_point(point);
                if seg_proj.is_on_track {
                    return Some(seg_proj.base_surface);
                }
                on_curb |= seg_proj.is_on_curb;
            }
        }
        if on_curb {
            return Some(SurfaceType::Curb);
        }

        // 2. Check junction throat polygons & gore triangles
        for junction in &self.junctions {
            match &junction.kind {
                JunctionKind::Split {
                    ingress_socket,
                    egress_sockets,
                    gore_config,
                } => {
                    if egress_sockets.len() >= 2 {
                        let in_l = ingress_socket.left_edge();
                        let in_r = ingress_socket.right_edge();
                        let e0_l = egress_sockets[0].left_edge();
                        let e0_r = egress_sockets[0].right_edge();
                        let e1_l = egress_sockets[1].left_edge();
                        let e1_r = egress_sockets[1].right_edge();

                        if point_in_triangle_2d(point, in_l, e0_l, e1_l)
                            || point_in_triangle_2d(point, in_r, e0_r, e1_r)
                            || point_in_quad_2d(point, in_l, in_r, e0_r, e0_l)
                            || point_in_quad_2d(point, in_l, in_r, e1_r, e1_l)
                        {
                            return Some(ingress_socket.surface);
                        }
                    }
                    if let Some(gore) = gore_config {
                        let p_apex = gore.apex_point;
                        let v0 = egress_sockets.get(0).map_or(ingress_socket.tangent, |s| s.tangent);
                        let v1 = egress_sockets.get(1).map_or(ingress_socket.tangent, |s| s.tangent);
                        let gore_len = gore.gore_length.max(6.0);
                        let p0 = p_apex + v0 * gore_len;
                        let p1 = p_apex + v1 * gore_len;
                        // The gore keeps the ingress road's own surface; an artificial asphalt island
                        // would give a dirt car tarmac grip.
                        if point_in_triangle_2d(point, p_apex, p0, p1)
                            || point_in_triangle_2d(point, p_apex, gore.nose_barrier.segment.start, gore.nose_barrier.segment.end)
                        {
                            return Some(ingress_socket.surface);
                        }
                    }
                }
                JunctionKind::Merge {
                    ingress_sockets,
                    egress_socket,
                    ..
                } => {
                    let eg_l = egress_socket.left_edge();
                    let eg_r = egress_socket.right_edge();
                    for in_sock in ingress_sockets {
                        let in_l = in_sock.left_edge();
                        let in_r = in_sock.right_edge();
                        if point_in_quad_2d(point, in_l, in_r, eg_r, eg_l) {
                            return Some(egress_socket.surface);
                        }
                    }
                }
                JunctionKind::Terminal { .. } => {}
            }
        }

        // 3. Check explicit runoff corridors on branch segments
        for seg in &self.segments {
            if self.is_branch_segment(seg.id) && seg.samples.len() >= 2 {
                let seg_proj = seg.project_point(point);
                let half_w = seg_proj.track_width * 0.5;
                if seg_proj.lateral_offset < -half_w {
                    if let Some(runoff) = seg_proj.left_runoff_surface {
                        let limit = half_w
                            + seg_proj
                                .left_wall_distance
                                .unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);
                        if -seg_proj.lateral_offset <= limit {
                            return Some(runoff);
                        }
                    }
                } else if seg_proj.lateral_offset > half_w {
                    if let Some(runoff) = seg_proj.right_runoff_surface {
                        let limit = half_w
                            + seg_proj
                                .right_wall_distance
                                .unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);
                        if seg_proj.lateral_offset <= limit {
                            return Some(runoff);
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for TrackNetwork {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the lateral clearance gap G between two diverging road segments at distance `s1` along `seg1` (Spec 112).
///
/// Returns the distance between the facing road edges:
/// G = distance(centerlines) - (w1/2 + w2/2).
/// When G >= 0, it represents the physical ground clearance between the two drivable road surfaces.
/// Returns None if either segment has no samples.
pub fn clearance_gap(seg1: &RoadSegment, seg2: &RoadSegment, s1: f32) -> Option<f32> {
    if seg1.samples.len() < 2 || seg2.samples.len() < 2 {
        return None;
    }
    let s1_clamped = s1.clamp(0.0, seg1.length);
    let spline1 = seg1.to_spline();
    let c1 = spline1.sample_at_distance(s1_clamped);
    let spline2 = seg2.to_spline();
    let proj2 = spline2.project_point(c1.point);

    let to_seg2 = proj2.closest_point - c1.point;
    let sign = if to_seg2.dot(c1.normal) >= 0.0 { 1.0 } else { -1.0 };
    let edge1 = c1.point + c1.normal * (sign * c1.width * 0.5);

    let proj_edge = spline2.project_point(edge1);
    let gap = proj_edge.distance_to_spline - proj_edge.track_width * 0.5;
    Some(gap)
}

/// Dynamically generates adaptive median barriers centered in the gore wedge between two diverging road segments (Spec 112).
///
/// Starting where lateral clearance G(t) >= 2.2 m, barriers are placed along the centerline of the gore wedge,
/// maintaining at least 0.3 m clearance from both road edges.
/// Bypassed if divergence angle exceeds 60 degrees.
pub fn generate_adaptive_median_barriers(
    seg1: &RoadSegment,
    seg2: &RoadSegment,
    barrier_type: BarrierType,
) -> Vec<WallBarrier> {
    if seg1.samples.len() < 2 || seg2.samples.len() < 2 {
        return Vec::new();
    }

    // Divergence angle check at entrance (first sample tangents)
    let t1 = seg1.samples[0].tangent;
    let t2 = seg2.samples[0].tangent;
    let cos_angle = t1.dot(t2).clamp(-1.0, 1.0);
    // cos(60 deg) = 0.5. If angle > 60 deg, cos_angle < 0.5
    if cos_angle < 0.5 {
        return Vec::new();
    }

    let spline1 = seg1.to_spline();
    let spline2 = seg2.to_spline();

    let mut points: Vec<Vec2> = Vec::new();
    let step_m = 1.0;
    let max_s = seg1.length.min(seg2.length).min(100.0);
    let mut s = 0.0;

    while s <= max_s {
        let c1 = spline1.sample_at_distance(s);
        let proj2 = spline2.project_point(c1.point);

        let to_seg2 = proj2.closest_point - c1.point;
        let sign = if to_seg2.dot(c1.normal) >= 0.0 { 1.0 } else { -1.0 };
        let edge1 = c1.point + c1.normal * (sign * c1.width * 0.5);

        let proj_edge = spline2.project_point(edge1);
        let gap = proj_edge.distance_to_spline - proj_edge.track_width * 0.5;

        // Adaptive median barrier triggers when gap G(t) >= 2.2 m
        // Upper bound around 10.0 m when independent parallel walls can exist
        if gap >= 2.2 && gap <= 10.0 {
            let to_edge2 = (proj_edge.closest_point - edge1).normalize_or_zero();
            let p_median = edge1 + to_edge2 * (gap * 0.5);

            // Verify physical clearance >= 0.3 m from both road edges
            let clr1 = spline1.project_point(p_median).distance_to_spline - c1.width * 0.5;
            let clr2 = spline2.project_point(p_median).distance_to_spline - proj_edge.track_width * 0.5;

            if clr1 >= 0.3 && clr2 >= 0.3 {
                // Ensure monotonically spaced points (avoid clustering)
                if points.last().map_or(true, |last| last.distance(p_median) >= 0.8) {
                    points.push(p_median);
                }
            }
        } else if !points.is_empty() && gap > 10.0 {
            // Once the diverging gore exceeds 10 m, the gore wedge has ended and
            // independent parallel walls can exist. Stop tracing the gore.
            break;
        }

        s += step_m;
    }

    if points.len() < 2 {
        return Vec::new();
    }

    points
        .windows(2)
        .filter(|w| w[0].distance(w[1]) <= step_m * 2.0)
        .map(|w| WallBarrier::new(w[0], w[1], barrier_type))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spline_socket_ghost_point_and_c1_continuity() {
        let pt = Vec2::new(100.0, 50.0);
        let tangent = Vec2::new(1.0, 0.0);
        let socket = SplineSocket::new(pt, tangent, 12.0);

        // Ghost point should be strictly along -tangent
        let ghost = socket.synthesize_ghost_point(10.0);
        assert_eq!(ghost, Vec2::new(90.0, 50.0));

        // Tangent should be normalized
        assert!((socket.tangent.length() - 1.0).abs() < 1e-6);
        assert!((socket.normal.length() - 1.0).abs() < 1e-6);
        assert_eq!(socket.normal, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn test_split_width_envelope_hermite_widening() {
        let w_trunk = 12.0;
        let w1 = 8.0;
        let w2 = 8.0;
        let w_gore = 4.0; // target = 20.0

        let w_start = compute_split_width_envelope(w_trunk, w1, w2, w_gore, 0.0);
        let w_mid = compute_split_width_envelope(w_trunk, w1, w2, w_gore, 0.5);
        let w_end = compute_split_width_envelope(w_trunk, w1, w2, w_gore, 1.0);

        assert!((w_start - 12.0).abs() < 1e-5);
        assert!((w_end - 20.0).abs() < 1e-5);
        assert!(w_mid > w_start && w_mid < w_end);
    }

    #[test]
    fn test_road_segment_resampling_and_projection() {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(50.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0),
        ];

        let seg = RoadSegment::new(SegmentId(1), "Straight", waypoints);
        assert!(seg.length > 90.0);
        assert!(!seg.samples.is_empty());

        let proj = seg.project_point(Vec2::new(50.0, 2.0));
        assert!(proj.is_on_track);
        assert!((proj.lateral_offset.abs() - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_track_network_auto_promotion_from_single_spline() {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(100.0, 100.0), 10.0),
            TrackWaypoint::new(Vec2::new(0.0, 100.0), 10.0),
        ];
        let spline = TrackSpline::new(waypoints, true);
        let network = TrackNetwork::from_single_spline(&spline);

        assert_eq!(network.segments.len(), 1);
        assert_eq!(network.layouts.len(), 1);
        assert!(network.validate().is_ok());

        let layout = network.active_or_default_layout(None).unwrap();
        assert_eq!(layout.id, "main");
        assert!(layout.is_closed);

        let composite = network.build_composite_spline_for_layout("main").unwrap();
        assert!(composite.closed);
        assert!(composite.total_length > 300.0);
    }

    #[test]
    fn test_c1_tangent_alignment_at_branch_split_socket() {
        // Trunk segment ending at (100, 0) pointing East (+X)
        let trunk_end = Vec2::new(100.0, 0.0);
        let split_tangent = Vec2::new(1.0, 0.0);
        let split_socket_a = SplineSocket::new(trunk_end, split_tangent, 10.0);

        // Branch segment starting at the socket
        let branch_waypoints = vec![
            TrackWaypoint::new(trunk_end, 10.0),
            TrackWaypoint::new(Vec2::new(150.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(200.0, 20.0), 10.0),
        ];

        let mut branch_seg = RoadSegment::new(SegmentId(2), "Branch A", branch_waypoints);
        branch_seg.recompute_samples(Some(&split_socket_a), None);

        // First sample of the branch must match socket.tangent with negligible error (< 1e-4 rad)
        let initial_tangent = branch_seg.samples[0].tangent;
        let dot = initial_tangent.dot(split_socket_a.tangent).clamp(-1.0, 1.0);
        let angle_error = dot.acos();
        assert!(
            angle_error < 1e-4,
            "C1 tangent error at branch socket is too high: {} rad",
            angle_error
        );
    }

    #[test]
    fn test_gore_triangle_keeps_the_ingress_surface() {
        // Dirt trunk splitting into two dirt branches at x = 100, with a gore triangle at the apex.
        let ingress = SplineSocket::new(Vec2::new(100.0, 0.0), Vec2::X, 10.0).with_surface(SurfaceType::Dirt);
        let egress = vec![
            SplineSocket::new(Vec2::new(100.0, 0.0), Vec2::new(1.0, 0.5), 8.0).with_surface(SurfaceType::Dirt),
            SplineSocket::new(Vec2::new(100.0, 0.0), Vec2::new(1.0, -0.5), 8.0).with_surface(SurfaceType::Dirt),
        ];
        let gore = GoreConfig::new(Vec2::new(110.0, 0.0), 53.0, 20.0, BarrierType::TireWall);
        let junction = RoadJunction::split(JunctionId(0), "Dirt Split", ingress, egress, Some(gore));
        let mut net = TrackNetwork::new();
        net.junctions.push(junction);

        // Inside the gore triangle, clear of both branch roads (no segments exist in this network).
        let inside = Vec2::new(116.0, 0.0);
        assert_eq!(net.sample_surface(inside), Some(SurfaceType::Dirt));
        // Outside every junction area there is nothing to report.
        assert_eq!(net.sample_surface(Vec2::new(116.0, 40.0)), None);
    }

    #[test]
    fn test_degenerate_junction_triangle_contains_no_point() {
        let a = Vec2::new(10.0, 5.0);
        let far = Vec2::new(5000.0, 5000.0);
        // Coincident corners (a Split junction whose ingress and egress sockets share a point).
        assert!(!point_in_triangle_2d(far, a, a, a));
        assert!(!point_in_triangle_2d(a, a, a, a));
        // Collinear corners.
        assert!(!point_in_triangle_2d(Vec2::new(15.0, 5.0), a, Vec2::new(20.0, 5.0), Vec2::new(30.0, 5.0)));
        assert!(!point_in_quad_2d(far, a, a, a, a));
        // A real triangle still works.
        let (b, c) = (Vec2::new(20.0, 5.0), Vec2::new(10.0, 15.0));
        assert!(point_in_triangle_2d(Vec2::new(12.0, 7.0), a, b, c));
        assert!(!point_in_triangle_2d(far, a, b, c));
    }

    #[test]
    fn test_track_network_branch_segments_filtering() {
        let seg0 = RoadSegment::new(SegmentId(0), "Main Straight", vec![]);
        let seg1 = RoadSegment::new(SegmentId(1), "Main Turn", vec![]);
        let seg2 = RoadSegment::new(SegmentId(2), "Joker Loop", vec![]);
        let seg3 = RoadSegment::new(SegmentId(3), "Launch Chute", vec![]);

        let mut net = TrackNetwork::new();
        net.segments = vec![seg0, seg1, seg2, seg3];
        net.layouts = vec![
            TrackLayout {
                id: "main".into(),
                display_name: "Standard Circuit".into(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(1)],
                is_closed: true,
                total_lap_length: 500.0,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: vec![],
            },
            TrackLayout {
                id: "joker".into(),
                display_name: "Joker Detour".into(),
                entry_segment: None,
                segment_sequence: vec![SegmentId(0), SegmentId(2)],
                is_closed: true,
                total_lap_length: 530.0,
                start_finish_segment: SegmentId(0),
                checkpoint_ids: vec![],
            },
        ];
        net.default_layout_id = "main".into();

        // Segments 0 and 1 are on the default layout sequence -> not branches
        assert!(!net.is_branch_segment(SegmentId(0)));
        assert!(!net.is_branch_segment(SegmentId(1)));

        // Segments 2 and 3 are NOT on the default layout sequence -> branches
        assert!(net.is_branch_segment(SegmentId(2)));
        assert!(net.is_branch_segment(SegmentId(3)));

        let branches = net.branch_segments();
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].id, SegmentId(2));
        assert_eq!(branches[1].id, SegmentId(3));

        // When segment 3 is designated as launch chute
        net.launch_chute = Some(LaunchChuteConfig {
            segment_id: SegmentId(3),
            merge_junction_id: JunctionId(0),
            pad_width: 16.0,
            grid_slots: vec![],
            side_barriers: vec![],
            terminal_barrier: crate::track::geometry::WallBarrier::new(
                Vec2::ZERO,
                Vec2::ZERO,
                crate::track::geometry::BarrierType::Concrete,
            ),
            surface: SurfaceType::Asphalt,
        });

        assert_eq!(net.launch_chute_segment().map(|s| s.id), Some(SegmentId(3)));

        // Joker segments should only include segment 2 (not chute, not default layout)
        let jokers = net.joker_segments();
        assert_eq!(jokers.len(), 1);
        assert_eq!(jokers[0].id, SegmentId(2));
    }

    #[test]
    fn test_clearance_gap_parallel_and_diverging() {
        // Road 1 along y = 0, width 8.0 (edges at y = -4.0 and +4.0)
        let wps1 = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
            TrackWaypoint::new(Vec2::new(50.0, 0.0), 8.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 8.0),
        ];
        let seg1 = RoadSegment::new(SegmentId(1), "Road 1", wps1);

        // Road 2 along y = 10.2, width 8.0 (edges at y = +6.2 and +14.2)
        // Distance between facing edges (y = 4.0 and y = 6.2) is exactly 2.2 m
        let wps2 = vec![
            TrackWaypoint::new(Vec2::new(0.0, 10.2), 8.0),
            TrackWaypoint::new(Vec2::new(50.0, 10.2), 8.0),
            TrackWaypoint::new(Vec2::new(100.0, 10.2), 8.0),
        ];
        let seg2 = RoadSegment::new(SegmentId(2), "Road 2", wps2);

        let gap = clearance_gap(&seg1, &seg2, 50.0).expect("clearance gap computed");
        assert!((gap - 2.2).abs() < 1e-3, "Expected 2.2 m gap, got {}", gap);
        assert_eq!(seg1.clearance_gap_to(&seg2, 50.0), Some(gap));
    }
}
