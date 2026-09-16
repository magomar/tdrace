use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;
use crate::track::geometry::{BarrierType, LineSegment, WallBarrier};
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

                raw_points.push(pt);
                raw_widths.push(w);
                raw_left_curbs.push(if t < 0.5 { wp1.left_curb } else { wp2.left_curb });
                raw_right_curbs.push(if t < 0.5 { wp1.right_curb } else { wp2.right_curb });
                raw_surfaces.push(wp1.surface.or(wp2.surface).unwrap_or(SurfaceType::Asphalt));
                raw_elevations.push(elev);
                raw_bank_angles.push(bank);
                raw_left_walls.push(if t < 0.5 { wp1.left_wall } else { wp2.left_wall });
                raw_right_walls.push(if t < 0.5 { wp1.right_wall } else { wp2.right_wall });
                raw_left_wall_dists.push(wp1.left_wall_distance.or(wp2.left_wall_distance));
                raw_right_wall_dists.push(wp1.right_wall_distance.or(wp2.right_wall_distance));
                raw_wall_types.push(wp1.wall_type.or(wp2.wall_type));
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
                left_wall: raw_left_walls[i],
                right_wall: raw_right_walls[i],
                left_wall_distance: raw_left_wall_dists[i],
                right_wall_distance: raw_right_wall_dists[i],
                wall_type: raw_wall_types[i],
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
                left_wall: true,
                right_wall: true,
                left_wall_distance: None,
                right_wall_distance: None,
                wall_type: None,
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
            left_wall: if t < 0.5 { s0.left_wall } else { s1.left_wall },
            right_wall: if t < 0.5 { s0.right_wall } else { s1.right_wall },
            left_wall_distance: s0.left_wall_distance,
            right_wall_distance: s0.right_wall_distance,
            wall_type: s0.wall_type,
        }
    }

    /// Projects any 2D world position onto this road segment centerline.
    pub fn project_point(&self, pos: Vec2) -> SplineProjection {
        if self.samples.len() < 2 {
            return SplineProjection {
                closest_point: pos,
                distance_to_spline: 0.0,
                lateral_offset: 0.0,
                progress_distance: 0.0,
                normalized_progress: 0.0,
                tangent: Vec2::X,
                normal: Vec2::Y,
                track_width: 10.0,
                left_curb: false,
                right_curb: false,
                is_on_track: true,
                is_on_curb: false,
                base_surface: SurfaceType::Asphalt,
                elevation: 0.0,
                bank_angle: 0.0,
            };
        }

        let mut best_dist_sq = f32::INFINITY;
        let mut best_point = Vec2::ZERO;
        let mut best_progress = 0.0f32;
        let mut best_sample_idx = 0;
        let mut best_t = 0.0f32;

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
                let seg_len = seg.length();
                best_t = if seg_len > 1e-4 {
                    (cp - p0).length() / seg_len
                } else {
                    0.0
                };
                best_progress = self.samples[i].distance + best_t * seg_len;
            }
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
    /// Ordered sequence of segment IDs traversed in this layout.
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
            segment_sequence,
            is_closed: true,
            total_lap_length: 0.0,
            start_finish_segment,
            checkpoint_ids: Vec::new(),
        }
    }

    pub const fn with_closed(mut self, is_closed: bool) -> Self {
        self.is_closed = is_closed;
        self
    }

    pub fn with_checkpoints(mut self, checkpoint_ids: Vec<usize>) -> Self {
        self.checkpoint_ids = checkpoint_ids;
        self
    }
}

/// Complete track network representing multi-branch circuits and layouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackNetwork {
    pub junctions: Vec<RoadJunction>,
    pub segments: Vec<RoadSegment>,
    pub layouts: Vec<TrackLayout>,
    pub default_layout_id: String,
}

impl TrackNetwork {
    /// Creates an empty track network.
    pub fn new() -> Self {
        Self {
            junctions: Vec::new(),
            segments: Vec::new(),
            layouts: Vec::new(),
            default_layout_id: "main".to_string(),
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
            segment_sequence: vec![seg_id],
            is_closed: spline.closed,
            total_lap_length: spline.total_length,
            start_finish_segment: seg_id,
            checkpoint_ids: Vec::new(),
        };

        Self {
            junctions: Vec::new(),
            segments: vec![segment],
            layouts: vec![layout],
            default_layout_id: "main".to_string(),
        }
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
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for TrackNetwork {
    fn default() -> Self {
        Self::new()
    }
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
}
