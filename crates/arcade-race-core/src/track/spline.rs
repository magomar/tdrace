use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::SurfaceType;
use crate::track::curve::{
    evaluate_curve_approach, extract_curves_from_samples, CurveApproachStatus, TrackCurve,
};
use crate::track::geometry::{BarrierType, LineSegment};

const fn default_true() -> bool {
    true
}

/// Waypoint defining a node along the track centerline with cross-section attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackWaypoint {
    /// 2D coordinate of the centerline node.
    pub point: Vec2,
    /// Total drivable asphalt track width at this point (meters).
    pub width: f32,
    /// Whether a curb (rumble strip) is installed on the left side of the track.
    pub left_curb: bool,
    /// Whether a curb (rumble strip) is installed on the right side of the track.
    pub right_curb: bool,
    /// Optional surface type override (defaults to Asphalt if None).
    pub surface: Option<SurfaceType>,
    /// Elevation / vertical altitude above ground in meters (default: 0.0).
    #[serde(default)]
    pub elevation: f32,
    /// Cross-slope banking angle in degrees (default: 0.0; + = right side elevated / banked left, - = left side elevated / banked right).
    #[serde(default)]
    pub bank_angle: f32,
    /// Whether a perimeter barrier wall is installed on the left side of the track.
    #[serde(default = "default_true")]
    pub left_wall: bool,
    /// Whether a perimeter barrier wall is installed on the right side of the track.
    #[serde(default = "default_true")]
    pub right_wall: bool,
    /// Distance between left track edge and left barrier wall in meters (default: None, inheriting global barrier offset).
    #[serde(default)]
    pub left_wall_distance: Option<f32>,
    /// Distance between right track edge and right barrier wall in meters (default: None, inheriting global barrier offset).
    #[serde(default)]
    pub right_wall_distance: Option<f32>,
    /// Optional barrier wall type override (e.g. Concrete or TireWall; default: None, inheriting global barrier type).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_type: Option<BarrierType>,
    /// Optional surface type override for the left corridor between track/curb and left wall (e.g. Gravel).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_runoff_surface: Option<SurfaceType>,
    /// Optional surface type override for the right corridor between track/curb and right wall (e.g. Gravel).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_runoff_surface: Option<SurfaceType>,
}

impl TrackWaypoint {
    pub const fn new(point: Vec2, width: f32) -> Self {
        Self {
            point,
            width,
            left_curb: false,
            right_curb: false,
            surface: None,
            elevation: 0.0,
            bank_angle: 0.0,
            left_wall: true,
            right_wall: true,
            left_wall_distance: None,
            right_wall_distance: None,
            wall_type: None,
            left_runoff_surface: None,
            right_runoff_surface: None,
        }
    }

    pub const fn with_runoff_surface(mut self, surface: SurfaceType) -> Self {
        self.left_runoff_surface = Some(surface);
        self.right_runoff_surface = Some(surface);
        self
    }

    pub const fn with_runoff_surfaces(
        mut self,
        left: Option<SurfaceType>,
        right: Option<SurfaceType>,
    ) -> Self {
        self.left_runoff_surface = left;
        self.right_runoff_surface = right;
        self
    }

    pub const fn with_wall_type(mut self, wall_type: Option<BarrierType>) -> Self {
        self.wall_type = wall_type;
        self
    }

    pub const fn with_curbs(mut self, left: bool, right: bool) -> Self {
        self.left_curb = left;
        self.right_curb = right;
        self
    }

    pub const fn with_walls(mut self, left: bool, right: bool) -> Self {
        self.left_wall = left;
        self.right_wall = right;
        self
    }

    pub const fn with_wall_distances(mut self, left: Option<f32>, right: Option<f32>) -> Self {
        self.left_wall_distance = left;
        self.right_wall_distance = right;
        self
    }

    pub const fn with_wall_distance(mut self, distance: f32) -> Self {
        self.left_wall_distance = Some(distance);
        self.right_wall_distance = Some(distance);
        self
    }

    pub const fn with_surface(mut self, surface: SurfaceType) -> Self {
        self.surface = Some(surface);
        self
    }

    pub const fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    pub const fn with_bank_angle(mut self, bank_angle: f32) -> Self {
        self.bank_angle = bank_angle;
        self
    }
}

/// A finely sampled point along the discretized spline curve with geometric properties.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplineSample {
    pub point: Vec2,
    pub tangent: Vec2,
    pub normal: Vec2, // Left-pointing normal
    pub distance: f32, // Cumulative arc-length distance from start (meters)
    pub width: f32,
    pub left_curb: bool,
    pub right_curb: bool,
    pub surface: SurfaceType,
    #[serde(default)]
    pub elevation: f32,
    #[serde(default)]
    pub bank_angle: f32,
    #[serde(default)]
    pub is_bridge: bool,
    #[serde(default)]
    pub grade_slope: f32,
    #[serde(default)]
    pub vertical_curvature: f32,
    #[serde(default = "default_true")]
    pub left_wall: bool,
    #[serde(default = "default_true")]
    pub right_wall: bool,
    #[serde(default)]
    pub left_wall_distance: Option<f32>,
    #[serde(default)]
    pub right_wall_distance: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_type: Option<BarrierType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_runoff_surface: Option<SurfaceType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_runoff_surface: Option<SurfaceType>,
}

/// Result of projecting a 2D world coordinate onto the track spline.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplineProjection {
    /// Nearest point on the centerline spline.
    pub closest_point: Vec2,
    /// Perpendicular distance to the spline centerline in meters.
    pub distance_to_spline: f32,
    /// Signed lateral offset from centerline (+ = right of center, - = left of center).
    pub lateral_offset: f32,
    /// Cumulative arc-length distance along the spline in meters [0, total_length).
    pub progress_distance: f32,
    /// Normalized progress along the circuit [0.0, 1.0).
    pub normalized_progress: f32,
    /// Spline tangent vector (track forward direction).
    pub tangent: Vec2,
    /// Spline left normal vector.
    pub normal: Vec2,
    /// Track width at this section.
    pub track_width: f32,
    /// Whether left curb is present.
    pub left_curb: bool,
    /// Whether right curb is present.
    pub right_curb: bool,
    /// True if the point is within the main drivable track ribbon.
    pub is_on_track: bool,
    /// True if the point is on a curb / rumble strip.
    pub is_on_curb: bool,
    /// Surface type at the projected center.
    pub base_surface: SurfaceType,
    /// Road surface elevation at the projected point in meters.
    pub elevation: f32,
    /// Road cross-slope banking angle in degrees.
    pub bank_angle: f32,
    /// Whether this track segment is an elevated crossover bridge span.
    pub is_bridge: bool,
    /// Longitudinal road slope angle in radians (+ = uphill, - = downhill).
    pub grade_slope: f32,
    /// Vertical road curvature d(grade_slope)/ds in 1/m (< 0 convex crest, > 0 concave dip).
    pub vertical_curvature: f32,
    /// Left wall distance at projected segment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_wall_distance: Option<f32>,
    /// Right wall distance at projected segment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_wall_distance: Option<f32>,
    /// Left runoff surface override at projected segment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_runoff_surface: Option<SurfaceType>,
    /// Right runoff surface override at projected segment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_runoff_surface: Option<SurfaceType>,
}

/// Smooth Catmull-Rom spline representation of the racing circuit centerline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSpline {
    pub waypoints: Vec<TrackWaypoint>,
    pub closed: bool,
    pub samples: Vec<SplineSample>,
    pub total_length: f32,
    #[serde(default)]
    pub curves: Vec<TrackCurve>,
    #[serde(default, skip_serializing)]
    pub sample_segments: Vec<(usize, f32)>,
    #[serde(default, skip_serializing)]
    pub waypoint_sample_indices: Vec<usize>,
}

impl Default for TrackSpline {
    fn default() -> Self {
        Self::empty()
    }
}

impl TrackSpline {
    /// Standard curb strip width in meters.
    pub const DEFAULT_CURB_WIDTH: f32 = 1.4;

    /// Default wall distance from track edge in meters when unspecified.
    pub const DEFAULT_WALL_DISTANCE: f32 = 8.0;

    /// Legacy fixed number of samples generated per Catmull-Rom segment.
    pub const STEPS_PER_SEGMENT: usize = 24;

    /// Target arc-length distance between sampled points along the spline in meters.
    pub const DEFAULT_TARGET_STEP_DISTANCE: f32 = 1.0;

    /// Minimum number of sample steps generated for any spline segment.
    pub const MIN_STEPS_PER_SEGMENT: usize = 4;

    /// An empty track spline with no waypoints or samples.
    pub fn empty() -> Self {
        Self {
            waypoints: Vec::new(),
            closed: false,
            samples: Vec::new(),
            total_length: 0.0,
            curves: Vec::new(),
            sample_segments: Vec::new(),
            waypoint_sample_indices: Vec::new(),
        }
    }

    /// Builds a smooth track spline from a list of waypoints with uniform arc-length resampling.
    pub fn new(waypoints: Vec<TrackWaypoint>, closed: bool) -> Self {
        if waypoints.len() < 3 {
            return Self {
                waypoints,
                closed,
                samples: Vec::new(),
                total_length: 0.0,
                curves: Vec::new(),
                sample_segments: Vec::new(),
                waypoint_sample_indices: Vec::new(),
            };
        }

        let mut samples = Vec::new();
        let num_wp = waypoints.len();
        let segments = if closed { num_wp } else { num_wp - 1 };

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
        let mut raw_left_runoff = Vec::new();
        let mut raw_right_runoff = Vec::new();
        let mut sample_segments = Vec::new();
        let mut waypoint_sample_indices = Vec::with_capacity(num_wp);

        for i in 0..segments {
            let p0 = if closed {
                waypoints[(i + num_wp - 1) % num_wp].point
            } else if i == 0 {
                waypoints[0].point
            } else {
                waypoints[i - 1].point
            };
            let p1 = waypoints[i % num_wp].point;
            let p2 = waypoints[(i + 1) % num_wp].point;
            let p3 = if closed {
                waypoints[(i + 2) % num_wp].point
            } else if i + 2 < num_wp {
                waypoints[i + 2].point
            } else {
                waypoints[num_wp - 1].point
            };

            let e0 = if closed {
                waypoints[(i + num_wp - 1) % num_wp].elevation
            } else if i == 0 {
                waypoints[0].elevation
            } else {
                waypoints[i - 1].elevation
            };
            let e1 = waypoints[i % num_wp].elevation;
            let e2 = waypoints[(i + 1) % num_wp].elevation;
            let e3 = if closed {
                waypoints[(i + 2) % num_wp].elevation
            } else if i + 2 < num_wp {
                waypoints[i + 2].elevation
            } else {
                waypoints[num_wp - 1].elevation
            };

            let b0 = if closed {
                waypoints[(i + num_wp - 1) % num_wp].bank_angle
            } else if i == 0 {
                waypoints[0].bank_angle
            } else {
                waypoints[i - 1].bank_angle
            };
            let b1 = waypoints[i % num_wp].bank_angle;
            let b2 = waypoints[(i + 1) % num_wp].bank_angle;
            let b3 = if closed {
                waypoints[(i + 2) % num_wp].bank_angle
            } else if i + 2 < num_wp {
                waypoints[i + 2].bank_angle
            } else {
                waypoints[num_wp - 1].bank_angle
            };

            let wp1 = &waypoints[i % num_wp];
            let wp2 = &waypoints[(i + 1) % num_wp];

            waypoint_sample_indices.push(raw_points.len());
            let chord_len = (p2 - p1).length();
            let steps_per_segment = ((chord_len / Self::DEFAULT_TARGET_STEP_DISTANCE).ceil() as usize)
                .max(Self::MIN_STEPS_PER_SEGMENT);

            let d01 = (p1 - p0).length().sqrt().max(1e-4);
            let d12 = (p2 - p1).length().sqrt().max(1e-4);
            let d23 = (p3 - p2).length().sqrt().max(1e-4);

            for s in 0..steps_per_segment {
                let t = s as f32 / steps_per_segment as f32;
                sample_segments.push((i, t));
                let pt = catmull_rom_centripetal_2d(p0, p1, p2, p3, t);
                let elev = catmull_rom_centripetal_1d_with_chords(e0, e1, e2, e3, d01, d12, d23, t);
                let bank = catmull_rom_centripetal_1d_with_chords(b0, b1, b2, b3, d01, d12, d23, t);
                let w = wp1.width + (wp2.width - wp1.width) * t;
                let lc = if t < 0.5 { wp1.left_curb } else { wp2.left_curb };
                let rc = if t < 0.5 { wp1.right_curb } else { wp2.right_curb };
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
                let surf = wp1.surface.unwrap_or(SurfaceType::Asphalt);
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
                raw_left_runoff.push(l_ro);
                raw_right_runoff.push(r_ro);
            }
        }

        // Add final point for closed/open
        if closed {
            sample_segments.push((0, 0.0));
            raw_points.push(raw_points[0]);
            raw_widths.push(raw_widths[0]);
            raw_left_curbs.push(raw_left_curbs[0]);
            raw_right_curbs.push(raw_right_curbs[0]);
            raw_surfaces.push(raw_surfaces[0]);
            raw_elevations.push(raw_elevations[0]);
            raw_bank_angles.push(raw_bank_angles[0]);
            raw_left_walls.push(raw_left_walls[0]);
            raw_right_walls.push(raw_right_walls[0]);
            raw_left_wall_dists.push(raw_left_wall_dists[0]);
            raw_right_wall_dists.push(raw_right_wall_dists[0]);
            raw_wall_types.push(raw_wall_types[0]);
            raw_left_runoff.push(raw_left_runoff[0]);
            raw_right_runoff.push(raw_right_runoff[0]);
        } else {
            let last = waypoints.last().unwrap();
            waypoint_sample_indices.push(raw_points.len());
            sample_segments.push((segments.saturating_sub(1), 1.0));
            raw_points.push(last.point);
            raw_widths.push(last.width);
            raw_left_curbs.push(last.left_curb);
            raw_right_curbs.push(last.right_curb);
            raw_surfaces.push(last.surface.unwrap_or(SurfaceType::Asphalt));
            raw_elevations.push(last.elevation);
            raw_bank_angles.push(last.bank_angle);
            raw_left_walls.push(last.left_wall);
            raw_right_walls.push(last.right_wall);
            raw_left_wall_dists.push(last.left_wall_distance);
            raw_right_wall_dists.push(last.right_wall_distance);
            raw_wall_types.push(last.wall_type);
            raw_left_runoff.push(last.left_runoff_surface);
            raw_right_runoff.push(last.right_runoff_surface);
        }

        // 2. Compute cumulative arc-length distances and orientations
        let mut cumulative_dist = 0.0f32;
        let mut dists = Vec::with_capacity(raw_points.len());
        dists.push(0.0);

        for i in 1..raw_points.len() {
            let seg_len = (raw_points[i] - raw_points[i - 1]).length();
            cumulative_dist += seg_len;
            dists.push(cumulative_dist);
        }

        let total_length = cumulative_dist;

        // 3. Detect overpass crossover bridges: non-adjacent spline segments that cross in 2D with clearance >= 2.5m
        let n_pts = raw_points.len();
        let mut is_bridge_flags = vec![false; n_pts];
        let n_segs = if closed { n_pts - 1 } else { n_pts.saturating_sub(1) };

        for i in 0..n_segs {
            let p0 = raw_points[i];
            let p1 = raw_points[(i + 1) % n_pts];
            let seg_i = LineSegment::new(p0, p1);

            let j_start = i + 6;
            let j_end = if closed {
                if i < 5 { n_segs.saturating_sub(6 - i) } else { n_segs }
            } else {
                n_segs
            };

            for j in j_start..j_end {
                let q0 = raw_points[j];
                let q1 = raw_points[(j + 1) % n_pts];
                let seg_j = LineSegment::new(q0, q1);

                if seg_i.intersect_segment(&seg_j).is_some() {
                    let elev_i = (raw_elevations[i] + raw_elevations[(i + 1) % n_pts]) * 0.5;
                    let elev_j = (raw_elevations[j] + raw_elevations[(j + 1) % n_pts]) * 0.5;
                    let clearance = (elev_i - elev_j).abs();

                    if clearance >= 2.5 {
                        let idx_bridge = if elev_i > elev_j { i } else { j };
                        is_bridge_flags[idx_bridge] = true;

                        // Contiguously expand across the elevated overpass span (while elevation >= 1.2m)
                        let max_bridge_span = 150.0f32;

                        // Forward expansion
                        let mut cur = (idx_bridge + 1) % n_pts;
                        let mut dist_walked = 0.0f32;
                        while dist_walked <= max_bridge_span {
                            let prev = (cur + n_pts - 1) % n_pts;
                            dist_walked += (raw_points[cur] - raw_points[prev]).length();
                            if raw_elevations[cur] < 1.2 {
                                break;
                            }
                            is_bridge_flags[cur] = true;
                            cur = (cur + 1) % n_pts;
                            if cur == idx_bridge {
                                break;
                            }
                        }

                        // Backward expansion
                        let mut cur = (idx_bridge + n_pts - 1) % n_pts;
                        let mut dist_walked = 0.0f32;
                        while dist_walked <= max_bridge_span {
                            let next = (cur + 1) % n_pts;
                            dist_walked += (raw_points[next] - raw_points[cur]).length();
                            if raw_elevations[cur] < 1.2 {
                                break;
                            }
                            is_bridge_flags[cur] = true;
                            if cur == 0 {
                                if closed {
                                    cur = n_pts - 1;
                                } else {
                                    break;
                                }
                            } else {
                                cur -= 1;
                            }
                            if cur == idx_bridge {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 4. Compute continuous longitudinal grade slope and vertical curvature
        let mut grade_slopes = Vec::with_capacity(n_pts);
        for i in 0..n_pts {
            let (i_prev, i_next) = if closed && n_pts > 2 {
                ((i + n_pts - 2) % (n_pts - 1), (i + 1) % (n_pts - 1))
            } else {
                (i.saturating_sub(1), (i + 1).min(n_pts - 1))
            };
            let delta_dist = if closed && n_pts > 2 && (i == 0 || i == n_pts - 1) {
                let seg_prev = (raw_points[0] - raw_points[n_pts - 2]).length();
                let seg_next = (raw_points[1] - raw_points[0]).length();
                (seg_prev + seg_next).max(1e-4)
            } else {
                (dists[i_next] - dists[i_prev]).abs().max(1e-4)
            };
            let delta_elev = raw_elevations[i_next] - raw_elevations[i_prev];
            let slope = (delta_elev / delta_dist).atan();
            grade_slopes.push(slope);
        }

        let mut vertical_curvatures = Vec::with_capacity(n_pts);
        for i in 0..n_pts {
            let (i_prev, i_next) = if closed && n_pts > 2 {
                ((i + n_pts - 2) % (n_pts - 1), (i + 1) % (n_pts - 1))
            } else {
                (i.saturating_sub(1), (i + 1).min(n_pts - 1))
            };
            let delta_dist = if closed && n_pts > 2 && (i == 0 || i == n_pts - 1) {
                let seg_prev = (raw_points[0] - raw_points[n_pts - 2]).length();
                let seg_next = (raw_points[1] - raw_points[0]).length();
                (seg_prev + seg_next).max(1e-4)
            } else {
                (dists[i_next] - dists[i_prev]).abs().max(1e-4)
            };
            let delta_slope = grade_slopes[i_next] - grade_slopes[i_prev];
            let curvature = delta_slope / delta_dist;
            vertical_curvatures.push(curvature);
        }

        // 5. Build SplineSample list
        let n = raw_points.len();
        for i in 0..n {
            let p_prev = if i > 0 { raw_points[i - 1] } else if closed { raw_points[n - 2] } else { raw_points[0] };
            let p_next = if i + 1 < n { raw_points[i + 1] } else if closed { raw_points[1] } else { raw_points[n - 1] };

            let delta = p_next - p_prev;
            let len = delta.length();
            let tangent = if len > 1e-6 { delta / len } else { Vec2::X };
            let normal = Vec2::new(-tangent.y, tangent.x);

            samples.push(SplineSample {
                point: raw_points[i],
                tangent,
                normal,
                distance: dists[i],
                width: raw_widths[i],
                left_curb: raw_left_curbs[i],
                right_curb: raw_right_curbs[i],
                surface: raw_surfaces[i],
                elevation: raw_elevations[i],
                bank_angle: raw_bank_angles[i],
                is_bridge: is_bridge_flags[i],
                grade_slope: grade_slopes[i],
                vertical_curvature: vertical_curvatures[i],
                left_wall: raw_left_walls[i],
                right_wall: raw_right_walls[i],
                left_wall_distance: raw_left_wall_dists[i],
                right_wall_distance: raw_right_wall_dists[i],
                wall_type: raw_wall_types[i],
                left_runoff_surface: raw_left_runoff[i],
                right_runoff_surface: raw_right_runoff[i],
            });
        }

        let curves = extract_curves_from_samples(&samples, total_length, closed);

        Self {
            waypoints,
            closed,
            samples,
            total_length,
            curves,
            sample_segments,
            waypoint_sample_indices,
        }
    }

    /// Returns the sample index corresponding to the given waypoint index, if within bounds.
    pub fn waypoint_sample_index(&self, waypoint_index: usize) -> Option<usize> {
        self.waypoint_sample_indices.get(waypoint_index).copied()
    }

    /// Computes synchronized untangled left/right road and curb outer boundary vertices,
    /// each slice exactly matching `samples.len()`.
    ///
    /// Synchronizes loop collapse intervals between road and curb boundaries to eliminate
    /// bowtie quads and inverted mesh artifacts at tight corner apexes.
    pub fn untangled_boundaries(
        &self,
        curb_extra_width: f32,
    ) -> (Vec<Vec2>, Vec<Vec2>, Vec<Vec2>, Vec<Vec2>) {
        let n = self.samples.len();
        if n == 0 {
            return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        }

        let mut left_road: Vec<Vec2> = self.samples.iter().map(|s| s.point + s.normal * (s.width * 0.5)).collect();
        let mut right_road: Vec<Vec2> = self.samples.iter().map(|s| s.point - s.normal * (s.width * 0.5)).collect();

        let mut left_curb: Vec<Vec2> = self.samples.iter().map(|s| {
            let extra = if s.left_curb { curb_extra_width } else { 0.0 };
            s.point + s.normal * (s.width * 0.5 + extra)
        }).collect();
        let mut right_curb: Vec<Vec2> = self.samples.iter().map(|s| {
            let extra = if s.right_curb { curb_extra_width } else { 0.0 };
            s.point - s.normal * (s.width * 0.5 + extra)
        }).collect();

        untangle_road_and_curb(&mut left_road, &mut left_curb, self.closed);
        untangle_road_and_curb(&mut right_road, &mut right_curb, self.closed);

        (left_road, right_road, left_curb, right_curb)
    }

    /// Computes untangled left and right road edge vertices, exactly matching `samples.len()`.
    pub fn untangled_road_edges(&self) -> (Vec<Vec2>, Vec<Vec2>) {
        let (left_road, right_road, _, _) = self.untangled_boundaries(1.35);
        (left_road, right_road)
    }

    /// Computes untangled left and right curb outer edge vertices, exactly matching `samples.len()`.
    pub fn untangled_curb_edges(&self, curb_extra_width: f32) -> (Vec<Vec2>, Vec<Vec2>) {
        let (_, _, left_curb, right_curb) = self.untangled_boundaries(curb_extra_width);
        (left_curb, right_curb)
    }

    /// Evaluates the next upcoming or active curve ahead of the given track progress distance.
    pub fn upcoming_curve(
        &self,
        progress_dist: f32,
        car_speed_mps: f32,
        max_lookahead: f32,
    ) -> Option<CurveApproachStatus> {
        evaluate_curve_approach(
            &self.curves,
            progress_dist,
            self.total_length,
            self.closed,
            car_speed_mps,
            max_lookahead,
        )
    }

    /// Helper to build a closed track spline from raw points with a constant width.
    pub fn from_points(points: &[Vec2], default_width: f32, closed: bool) -> Self {
        let waypoints = points
            .iter()
            .map(|&p| TrackWaypoint::new(p, default_width))
            .collect();
        Self::new(waypoints, closed)
    }

    /// Total track centerline length in meters.
    #[inline]
    /// Wall distance for sample `index` on one side (`left` or right).
    ///
    /// Where only one of the two waypoints bounding the sample's segment overrides the
    /// distance, this blends linearly toward `default` across the segment instead of the
    /// stored mid-segment step. Otherwise it returns the stored sample value.
    pub fn blended_wall_distance(&self, index: usize, left: bool, default: f32) -> Option<f32> {
        let sample = self.samples.get(index)?;
        let stored = if left { sample.left_wall_distance } else { sample.right_wall_distance };

        let num_wp = self.waypoints.len();
        if num_wp < 3 {
            return stored;
        }
        let segments = if self.closed { num_wp } else { num_wp - 1 };
        let (seg, t) = if let Some(&(s, t_val)) = self.sample_segments.get(index) {
            (s, t_val)
        } else {
        let steps = Self::STEPS_PER_SEGMENT;
        if self.samples.len() != segments * steps + 1 {
            // Baked samples from a different sampling layout: keep them as they are.
            return stored;
        }
            if index == segments * steps {
            if self.closed { (0, 0.0) } else { (segments - 1, 1.0) }
        } else {
            (index / steps, (index % steps) as f32 / steps as f32)
            }
        };
        let wp1 = &self.waypoints[seg % num_wp];
        let wp2 = &self.waypoints[(seg + 1) % num_wp];
        let (d1, d2) = if left {
            (wp1.left_wall_distance, wp2.left_wall_distance)
        } else {
            (wp1.right_wall_distance, wp2.right_wall_distance)
        };

        match (d1, d2) {
            (Some(_), None) | (None, Some(_)) => {
                let a = d1.unwrap_or(default);
                let b = d2.unwrap_or(default);
                Some(a + (b - a) * t)
            }
            _ => stored,
        }
    }

    pub fn total_length(&self) -> f32 {
        self.total_length
    }

    /// Samples the spline at a specific arc-length distance.
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

        let clamped_dist = if self.closed {
            let mut d = distance % self.total_length;
            if d < 0.0 {
                d += self.total_length;
            }
            d
        } else {
            distance.clamp(0.0, self.total_length)
        };

        // Binary search in cumulative distances
        let idx = match self.samples.binary_search_by(|s| {
            s.distance
                .partial_cmp(&clamped_dist)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
        };

        let next_idx = (idx + 1).min(self.samples.len() - 1);
        if idx == next_idx {
            return self.samples[idx];
        }

        let s0 = &self.samples[idx];
        let s1 = &self.samples[next_idx];
        let seg_len = (s1.distance - s0.distance).max(1e-4);
        let t = ((clamped_dist - s0.distance) / seg_len).clamp(0.0, 1.0);

        let point = s0.point.lerp(s1.point, t);
        let tangent = s0.tangent.lerp(s1.tangent, t).normalize_or_zero();
        let normal = Vec2::new(-tangent.y, tangent.x);
        let width = s0.width + (s1.width - s0.width) * t;
        let elevation = s0.elevation + (s1.elevation - s0.elevation) * t;
        let bank_angle = s0.bank_angle + (s1.bank_angle - s0.bank_angle) * t;
        let grade_slope = s0.grade_slope + (s1.grade_slope - s0.grade_slope) * t;
        let vertical_curvature = s0.vertical_curvature + (s1.vertical_curvature - s0.vertical_curvature) * t;
        let is_bridge = if t < 0.5 { s0.is_bridge } else { s1.is_bridge };
        let left_wall_distance = match (s0.left_wall_distance, s1.left_wall_distance) {
            (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * t),
            (Some(d1), None) => if t < 0.5 { Some(d1) } else { None },
            (None, Some(d2)) => if t < 0.5 { None } else { Some(d2) },
            (None, None) => None,
        };
        let right_wall_distance = match (s0.right_wall_distance, s1.right_wall_distance) {
            (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * t),
            (Some(d1), None) => if t < 0.5 { Some(d1) } else { None },
            (None, Some(d2)) => if t < 0.5 { None } else { Some(d2) },
            (None, None) => None,
        };

        SplineSample {
            point,
            tangent,
            normal,
            distance: clamped_dist,
            width,
            left_curb: if t < 0.5 { s0.left_curb } else { s1.left_curb },
            right_curb: if t < 0.5 { s0.right_curb } else { s1.right_curb },
            surface: s0.surface,
            elevation,
            bank_angle,
            is_bridge,
            grade_slope,
            vertical_curvature,
            left_wall: if t < 0.5 { s0.left_wall } else { s1.left_wall },
            right_wall: if t < 0.5 { s0.right_wall } else { s1.right_wall },
            left_wall_distance,
            right_wall_distance,
            wall_type: if t < 0.5 { s0.wall_type } else { s1.wall_type },
            left_runoff_surface: if t < 0.5 { s0.left_runoff_surface } else { s1.left_runoff_surface },
            right_runoff_surface: if t < 0.5 { s0.right_runoff_surface } else { s1.right_runoff_surface },
        }
    }

    /// Projects any 2D world position onto the spline centerline, computing progress and offsets.
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
        let mut best_progress = 0.0f32;
        let mut best_sample_idx = 0;
        let mut best_t = 0.0f32;

        for i in 0..self.samples.len() - 1 {
            let p0 = self.samples[i].point;
            let p1 = self.samples[i + 1].point;
            let ab = p1 - p0;
            let len_sq = ab.length_squared();
            let t = if len_sq > 1e-6 {
                ((pos - p0).dot(ab) / len_sq).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let proj_pt = p0 + ab * t;
            let d_sq = (pos - proj_pt).length_squared();

            if d_sq < best_dist_sq {
                best_dist_sq = d_sq;
                best_point = proj_pt;
                best_sample_idx = i;
                best_t = t;
                let seg_dist = (self.samples[i + 1].distance - self.samples[i].distance) * t;
                best_progress = self.samples[i].distance + seg_dist;
            }
        }

        let s0 = &self.samples[best_sample_idx];
        let s1 = &self.samples[(best_sample_idx + 1).min(self.samples.len() - 1)];

        let tangent = s0.tangent.lerp(s1.tangent, best_t).normalize_or_zero();
        let normal = Vec2::new(-tangent.y, tangent.x); // points left
        let right_vector = Vec2::new(tangent.y, -tangent.x); // points right

        let to_pos = pos - best_point;
        let lateral_offset = to_pos.dot(right_vector); // + right, - left
        let distance_to_spline = best_dist_sq.sqrt();

        let track_width = s0.width + (s1.width - s0.width) * best_t;
        let left_curb = if best_t < 0.5 { s0.left_curb } else { s1.left_curb };
        let right_curb = if best_t < 0.5 { s0.right_curb } else { s1.right_curb };
        let elevation = s0.elevation + (s1.elevation - s0.elevation) * best_t;
        let bank_angle = s0.bank_angle + (s1.bank_angle - s0.bank_angle) * best_t;
        let grade_slope = s0.grade_slope + (s1.grade_slope - s0.grade_slope) * best_t;
        let vertical_curvature = s0.vertical_curvature + (s1.vertical_curvature - s0.vertical_curvature) * best_t;
        let is_bridge = if best_t < 0.5 { s0.is_bridge } else { s1.is_bridge };

        let half_w = track_width * 0.5;
        let is_on_track = lateral_offset.abs() <= half_w;

        // Curb check
        let is_on_left_curb = left_curb
            && lateral_offset < -half_w
            && lateral_offset >= -half_w - Self::DEFAULT_CURB_WIDTH;
        let is_on_right_curb = right_curb
            && lateral_offset > half_w
            && lateral_offset <= half_w + Self::DEFAULT_CURB_WIDTH;
        let is_on_curb = is_on_left_curb || is_on_right_curb;

        let normalized_progress = if self.total_length > 1e-4 {
            (best_progress / self.total_length).clamp(0.0, 0.999999)
        } else {
            0.0
        };

        let left_wall_distance = match (s0.left_wall_distance, s1.left_wall_distance) {
            (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * best_t),
            (Some(d1), None) => if best_t < 0.5 { Some(d1) } else { None },
            (None, Some(d2)) => if best_t < 0.5 { None } else { Some(d2) },
            (None, None) => None,
        };
        let right_wall_distance = match (s0.right_wall_distance, s1.right_wall_distance) {
            (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * best_t),
            (Some(d1), None) => if best_t < 0.5 { Some(d1) } else { None },
            (None, Some(d2)) => if best_t < 0.5 { None } else { Some(d2) },
            (None, None) => None,
        };
        let left_runoff_surface = if best_t < 0.5 { s0.left_runoff_surface } else { s1.left_runoff_surface };
        let right_runoff_surface = if best_t < 0.5 { s0.right_runoff_surface } else { s1.right_runoff_surface };

        SplineProjection {
            closest_point: best_point,
            distance_to_spline,
            lateral_offset,
            progress_distance: best_progress,
            normalized_progress,
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
            is_bridge,
            grade_slope,
            vertical_curvature,
            left_wall_distance,
            right_wall_distance,
            left_runoff_surface,
            right_runoff_surface,
        }
    }

    /// Projects a 2D world position onto the spline centerline with continuity constraint around `prev_progress`.
    /// Restricts candidate segments to within `max_dist_delta` of `prev_progress` to avoid snapping
    /// to adjacent opposing track ribbons in close turns/chicanes.
    pub fn project_point_continuity(
        &self,
        pos: Vec2,
        prev_progress: f32,
        max_dist_delta: f32,
    ) -> SplineProjection {
        if self.samples.len() < 2 {
            return self.project_point(pos);
        }

        let total_len = self.total_length.max(1.0);
        let mut best_dist_sq = f32::INFINITY;
        let mut best_point = Vec2::ZERO;
        let mut best_progress = 0.0f32;
        let mut best_sample_idx = 0;
        let mut best_t = 0.0f32;
        let mut found_candidate = false;

        for i in 0..self.samples.len() - 1 {
            let seg_dist = self.samples[i].distance;
            let delta = if self.closed {
                let d = (seg_dist - prev_progress).abs();
                d.min(total_len - d)
            } else {
                (seg_dist - prev_progress).abs()
            };

            if delta > max_dist_delta {
                continue;
            }

            let p0 = self.samples[i].point;
            let p1 = self.samples[i + 1].point;
            let ab = p1 - p0;
            let len_sq = ab.length_squared();
            let t = if len_sq > 1e-6 {
                ((pos - p0).dot(ab) / len_sq).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let proj_pt = p0 + ab * t;
            let d_sq = (pos - proj_pt).length_squared();

            if d_sq < best_dist_sq {
                best_dist_sq = d_sq;
                best_point = proj_pt;
                best_sample_idx = i;
                best_t = t;
                let sub_dist = (self.samples[i + 1].distance - self.samples[i].distance) * t;
                best_progress = self.samples[i].distance + sub_dist;
                found_candidate = true;
            }
        }

        if !found_candidate {
            return self.project_point(pos);
        }

        let s0 = &self.samples[best_sample_idx];
        let s1 = &self.samples[(best_sample_idx + 1).min(self.samples.len() - 1)];

        let track_width = s0.width + (s1.width - s0.width) * best_t;
        let half_w = track_width * 0.5;

        // Continuity Plausibility Guard:
        // If the best candidate found in the continuity window is outside the track corridor
        // (beyond track boundary + curb width), verify whether a global projection finds a
        // closer point on the circuit. This catches uninitialized trackers, grid placements,
        // teleports, or window search discontinuities without sacrificing continuity on valid off-track slides.
        let corridor_limit = half_w + Self::DEFAULT_CURB_WIDTH;
        if best_dist_sq > corridor_limit * corridor_limit {
            let full = self.project_point(pos);
            if full.distance_to_spline * full.distance_to_spline < best_dist_sq {
                return full;
            }
        }

        let tangent = s0.tangent.lerp(s1.tangent, best_t).normalize_or_zero();
        let normal = Vec2::new(-tangent.y, tangent.x);
        let right_vector = Vec2::new(tangent.y, -tangent.x);

        let to_pos = pos - best_point;
        let lateral_offset = to_pos.dot(right_vector);
        let distance_to_spline = best_dist_sq.sqrt();
        let left_curb = if best_t < 0.5 { s0.left_curb } else { s1.left_curb };
        let right_curb = if best_t < 0.5 { s0.right_curb } else { s1.right_curb };
        let elevation = s0.elevation + (s1.elevation - s0.elevation) * best_t;
        let bank_angle = s0.bank_angle + (s1.bank_angle - s0.bank_angle) * best_t;
        let grade_slope = s0.grade_slope + (s1.grade_slope - s0.grade_slope) * best_t;
        let vertical_curvature = s0.vertical_curvature + (s1.vertical_curvature - s0.vertical_curvature) * best_t;
        let is_bridge = if best_t < 0.5 { s0.is_bridge } else { s1.is_bridge };

        let is_on_track = lateral_offset.abs() <= half_w;

        let is_on_left_curb = left_curb
            && lateral_offset < -half_w
            && lateral_offset >= -half_w - Self::DEFAULT_CURB_WIDTH;
        let is_on_right_curb = right_curb
            && lateral_offset > half_w
            && lateral_offset <= half_w + Self::DEFAULT_CURB_WIDTH;
        let is_on_curb = is_on_left_curb || is_on_right_curb;

        let normalized_progress = if self.total_length > 1e-4 {
            (best_progress / self.total_length).clamp(0.0, 0.999999)
        } else {
            0.0
        };

        let left_wall_distance = match (s0.left_wall_distance, s1.left_wall_distance) {
            (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * best_t),
            (Some(d1), None) => if best_t < 0.5 { Some(d1) } else { None },
            (None, Some(d2)) => if best_t < 0.5 { None } else { Some(d2) },
            (None, None) => None,
        };
        let right_wall_distance = match (s0.right_wall_distance, s1.right_wall_distance) {
            (Some(d1), Some(d2)) => Some(d1 + (d2 - d1) * best_t),
            (Some(d1), None) => if best_t < 0.5 { Some(d1) } else { None },
            (None, Some(d2)) => if best_t < 0.5 { None } else { Some(d2) },
            (None, None) => None,
        };
        let left_runoff_surface = if best_t < 0.5 { s0.left_runoff_surface } else { s1.left_runoff_surface };
        let right_runoff_surface = if best_t < 0.5 { s0.right_runoff_surface } else { s1.right_runoff_surface };

        SplineProjection {
            closest_point: best_point,
            distance_to_spline,
            lateral_offset,
            progress_distance: best_progress,
            normalized_progress,
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
            is_bridge,
            grade_slope,
            vertical_curvature,
            left_wall_distance,
            right_wall_distance,
            left_runoff_surface,
            right_runoff_surface,
        }
    }

    /// Returns the exact road surface elevation in meters at a 2D world coordinate,
    /// accounting for centerline elevation and cross-slope banking (superelevation).
    pub fn sample_cross_slope_elevation(&self, pos: Vec2) -> f32 {
        let proj = self.project_point(pos);
        let theta = proj.bank_angle.to_radians();
        // lateral_offset: + = right of center, - = left of center
        // When bank_angle > 0: right side is higher (+), left side is lower (-)
        proj.elevation + proj.lateral_offset * theta.sin()
    }
}

/// Trims local self-intersecting swallowtail loops from an offset boundary vertex array
/// by collapsing all vertices within the self-intersecting loop to the loop intersection point.
///
/// Preserves the exact length of the slice so vertex indices remain 1-to-1 with centerline samples.
pub fn untangle_offset_vertices(pts: &mut [Vec2], closed: bool) {
    let n = pts.len();
    if n < 4 {
        return;
    }
    let max_loop_span = 40.min(n / 2);
    let mut changed = true;
    let mut passes = 0;

    while changed && passes < 16 {
        changed = false;
        passes += 1;

        'outer: for i in 0..n {
            let p0 = pts[i];
            let next_i = (i + 1) % n;
            let p1 = pts[next_i];
            if (p1 - p0).length_squared() < 1e-6 {
                continue;
            }
            let seg_a = LineSegment::new(p0, p1);

            for span in 2..=max_loop_span {
                let j = (i + span) % n;
                if !closed && i + span >= n {
                    break;
                }
                let next_j = (j + 1) % n;
                if !closed && j + 1 >= n {
                    break;
                }
                if next_j == i || next_i == j {
                    continue;
                }

                let p2 = pts[j];
                let p3 = pts[next_j];
                if (p3 - p2).length_squared() < 1e-6 {
                    continue;
                }
                let seg_b = LineSegment::new(p2, p3);

                if (seg_a.start - seg_b.start).length_squared() < 1e-4
                    || (seg_a.start - seg_b.end).length_squared() < 1e-4
                    || (seg_a.end - seg_b.start).length_squared() < 1e-4
                    || (seg_a.end - seg_b.end).length_squared() < 1e-4
                {
                    continue;
                }

                if let Some(hit) = seg_a.intersect_segment(&seg_b) {
                    if j > i {
                        for k in (i + 1)..=j {
                            pts[k] = hit;
                        }
                    } else if closed {
                        for k in (i + 1)..n {
                            pts[k] = hit;
                        }
                        for k in 0..=j {
                            pts[k] = hit;
                        }
                    }
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
}

/// Finds the intersection point of two 2D lines defined by (origin, direction).
#[inline]
pub fn line_intersection_2d(p1: Vec2, d1: Vec2, p2: Vec2, d2: Vec2) -> Option<Vec2> {
    let cross = d1.x * d2.y - d1.y * d2.x;
    if cross.abs() < 1e-5 {
        return None;
    }
    let dp = p2 - p1;
    let t = (dp.x * d2.y - dp.y * d2.x) / cross;
    Some(p1 + d1 * t)
}

/// Synchronously untangles drivable road edges and curb outer edges.
///
/// When a tight corner causes the inner road boundary to cross itself (swallowtail loop),
/// collapsing only the road edge or collapsing road and curb independently creates
/// misaligned collapsed intervals, inverted trapezoids, and bowtie quad artifacts.
///
/// This function synchronously detects loop intervals on the road boundary and collapses
/// the corresponding curb vertices within the exact same sample interval to the apex curb
/// intersection, guaranteeing synchronized apex fans and zero boundary self-intersections.
pub fn untangle_road_and_curb(
    road_pts: &mut [Vec2],
    curb_pts: &mut [Vec2],
    closed: bool,
) {
    let n = road_pts.len();
    if n < 4 || curb_pts.len() != n {
        return;
    }
    let max_loop_span = 40.min(n / 2);
    let mut changed = true;
    let mut passes = 0;

    while changed && passes < 16 {
        changed = false;
        passes += 1;

        'outer: for i in 0..n {
            let p0 = road_pts[i];
            let next_i = (i + 1) % n;
            let p1 = road_pts[next_i];
            if (p1 - p0).length_squared() < 1e-6 {
                continue;
            }
            let seg_a = LineSegment::new(p0, p1);

            for span in 2..=max_loop_span {
                let j = (i + span) % n;
                if !closed && i + span >= n {
                    break;
                }
                let next_j = (j + 1) % n;
                if !closed && j + 1 >= n {
                    break;
                }
                if next_j == i || next_i == j {
                    continue;
                }

                let p2 = road_pts[j];
                let p3 = road_pts[next_j];
                if (p3 - p2).length_squared() < 1e-6 {
                    continue;
                }
                let seg_b = LineSegment::new(p2, p3);

                if (seg_a.start - seg_b.start).length_squared() < 1e-4
                    || (seg_a.start - seg_b.end).length_squared() < 1e-4
                    || (seg_a.end - seg_b.start).length_squared() < 1e-4
                    || (seg_a.end - seg_b.end).length_squared() < 1e-4
                {
                    continue;
                }

                if let Some(road_hit) = seg_a.intersect_segment(&seg_b) {
                    let c0 = curb_pts[i];
                    let c1 = curb_pts[next_i];
                    let c2 = curb_pts[j];
                    let c3 = curb_pts[next_j];

                    let dir_a = c1 - c0;
                    let dir_b = c3 - c2;
                    let curb_hit = line_intersection_2d(c0, dir_a, c2, dir_b)
                        .filter(|hit| (*hit - road_hit).length_squared() <= 625.0)
                        .unwrap_or_else(|| {
                            let offset_a = c0 - p0;
                            let offset_b = c2 - p2;
                            road_hit + (offset_a + offset_b) * 0.5
                        });

                    if j > i {
                        for k in (i + 1)..=j {
                            road_pts[k] = road_hit;
                            curb_pts[k] = curb_hit;
                        }
                    } else if closed {
                        for k in (i + 1)..n {
                            road_pts[k] = road_hit;
                            curb_pts[k] = curb_hit;
                        }
                        for k in 0..=j {
                            road_pts[k] = road_hit;
                            curb_pts[k] = curb_hit;
                        }
                    }
                    changed = true;
                    break 'outer;
                }
            }
        }
    }

    // Resolve any remaining curb-only self-intersections
    untangle_offset_vertices(curb_pts, closed);
}

/// 2D Centripetal Catmull-Rom interpolation (alpha = 0.5) for points p0, p1, p2, p3 at parameter t in [0, 1].
///
/// Uses the Barry and Goldman pyramidal formulation with chord lengths parameterized by Euclidean distance:
/// t_i+1 = t_i + ||p_i+1 - p_i||^0.5.
///
/// Proven by Yuksel et al. (2011) to eliminate cusps, self-intersections, and overshoot when waypoint spacing varies widely.
#[inline]
pub fn catmull_rom_centripetal_2d(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t_norm: f32) -> Vec2 {
    if (p2 - p1).length_squared() < 1e-8 {
        return p1;
    }

    let d01 = (p1 - p0).length().sqrt().max(1e-4);
    let d12 = (p2 - p1).length().sqrt().max(1e-4);
    let d23 = (p3 - p2).length().sqrt().max(1e-4);

    let t0 = 0.0;
    let t1 = d01;
    let t2 = t1 + d12;
    let t3 = t2 + d23;

    let t = t1 + t_norm.clamp(0.0, 1.0) * d12;

    let a1 = p0 + (p1 - p0) * ((t - t0) / (t1 - t0));
    let a2 = p1 + (p2 - p1) * ((t - t1) / (t2 - t1));
    let a3 = p2 + (p3 - p2) * ((t - t2) / (t3 - t2));

    let b1 = a1 + (a2 - a1) * ((t - t0) / (t2 - t0));
    let b2 = a2 + (a3 - a2) * ((t - t1) / (t3 - t1));

    b1 + (b2 - b1) * ((t - t1) / (t2 - t1))
}

/// 1D Centripetal Catmull-Rom interpolation using provided physical track chord lengths.
#[inline]
pub fn catmull_rom_centripetal_1d_with_chords(
    p0: f32,
    p1: f32,
    p2: f32,
    p3: f32,
    d01: f32,
    d12: f32,
    d23: f32,
    t_norm: f32,
) -> f32 {
    let d01 = d01.max(1e-4);
    let d12 = d12.max(1e-4);
    let d23 = d23.max(1e-4);

    let t0 = 0.0;
    let t1 = d01;
    let t2 = t1 + d12;
    let t3 = t2 + d23;

    let t = t1 + t_norm.clamp(0.0, 1.0) * d12;

    let a1 = p0 + (p1 - p0) * ((t - t0) / (t1 - t0));
    let a2 = p1 + (p2 - p1) * ((t - t1) / (t2 - t1));
    let a3 = p2 + (p3 - p2) * ((t - t2) / (t3 - t2));

    let b1 = a1 + (a2 - a1) * ((t - t0) / (t2 - t0));
    let b2 = a2 + (a3 - a2) * ((t - t1) / (t3 - t1));

    b1 + (b2 - b1) * ((t - t1) / (t2 - t1))
}

/// 1D Centripetal Catmull-Rom interpolation for scalar values p0, p1, p2, p3 at parameter t in [0, 1].
#[inline]
pub fn catmull_rom_centripetal_1d(p0: f32, p1: f32, p2: f32, p3: f32, t_norm: f32) -> f32 {
    let d01 = (p1 - p0).abs().sqrt().max(1e-4);
    let d12 = (p2 - p1).abs().sqrt().max(1e-4);
    let d23 = (p3 - p2).abs().sqrt().max(1e-4);
    catmull_rom_centripetal_1d_with_chords(p0, p1, p2, p3, d01, d12, d23, t_norm)
}

/// 2D Uniform Catmull-Rom interpolation for points p0, p1, p2, p3 at parameter t in [0, 1].
#[inline]
pub fn catmull_rom_uniform_2d(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;

    let f0 = -0.5 * t3 + t2 - 0.5 * t;
    let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
    let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
    let f3 = 0.5 * t3 - 0.5 * t2;

    p0 * f0 + p1 * f1 + p2 * f2 + p3 * f3
}

/// 1D Uniform Catmull-Rom interpolation for scalar values p0, p1, p2, p3 at parameter t in [0, 1].
#[inline]
pub fn catmull_rom_uniform_1d(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;

    let f0 = -0.5 * t3 + t2 - 0.5 * t;
    let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
    let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
    let f3 = 0.5 * t3 - 0.5 * t2;

    p0 * f0 + p1 * f1 + p2 * f2 + p3 * f3
}

/// 2D Catmull-Rom interpolation for points p0, p1, p2, p3 at parameter t in [0, 1].
/// Delegates to Centripetal Catmull-Rom (alpha = 0.5).
#[inline]
pub fn catmull_rom_2d(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    catmull_rom_centripetal_2d(p0, p1, p2, p3, t)
}

/// 1D Catmull-Rom interpolation for scalar values p0, p1, p2, p3 at parameter t in [0, 1].
/// Delegates to Centripetal Catmull-Rom (alpha = 0.5).
#[inline]
pub fn catmull_rom_1d(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    catmull_rom_centripetal_1d(p0, p1, p2, p3, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_spline_creation_and_sampling() {
        let pts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ];
        let spline = TrackSpline::from_points(&pts, 12.0, true);
        assert!(spline.total_length() > 300.0);

        let sample = spline.sample_at_distance(0.0);
        assert_eq!(sample.distance, 0.0);
        assert!(sample.width > 0.0);
    }

    #[test]
    fn test_spline_projection_on_track_and_curb() {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(100.0, 100.0), 10.0).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(0.0, 100.0), 10.0).with_curbs(true, true),
        ];
        let spline = TrackSpline::new(waypoints, true);

        let center_sample = spline.sample_at_distance(50.0);
        let proj_center = spline.project_point(center_sample.point);
        assert!(proj_center.is_on_track);
        assert!(!proj_center.is_on_curb);
        assert!(proj_center.distance_to_spline < 0.1);

        // Point on right curb (offset by +5.5m in right normal direction)
        let right_vec = Vec2::new(center_sample.tangent.y, -center_sample.tangent.x);
        let curb_pt = center_sample.point + right_vec * 5.5;
        let proj_curb = spline.project_point(curb_pt);
        assert!(!proj_curb.is_on_track);
        assert!(proj_curb.is_on_curb, "Point offset by +5.5m should be on curb");

        // Point far off track
        let grass_pt = center_sample.point + right_vec * 20.0;
        let proj_grass = spline.project_point(grass_pt);
        assert!(!proj_grass.is_on_track);
        assert!(!proj_grass.is_on_curb);
    }

    #[test]
    fn test_figure_eight_bridge_detection() {
        // A figure-8 loop crossing at (0, 0). Lower crossing at z=0, upper overpass at z=5.0.
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(-50.0, 0.0), 10.0).with_elevation(0.0),
            TrackWaypoint::new(Vec2::new(-25.0, 25.0), 10.0).with_elevation(0.0),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0).with_elevation(0.0), // Lower crossing
            TrackWaypoint::new(Vec2::new(25.0, -25.0), 10.0).with_elevation(1.0),
            TrackWaypoint::new(Vec2::new(50.0, 0.0), 10.0).with_elevation(3.0),
            TrackWaypoint::new(Vec2::new(25.0, 25.0), 10.0).with_elevation(4.5),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0).with_elevation(5.0), // Upper crossing (bridge)
            TrackWaypoint::new(Vec2::new(-25.0, -25.0), 10.0).with_elevation(2.5),
        ];
        let spline = TrackSpline::new(waypoints, true);

        // Lower crossing around distance ~ 60 should NOT be bridge
        let lower_proj = spline.project_point(Vec2::new(-35.0, 10.0));
        assert!(!lower_proj.is_bridge, "Lower crossing should not be flagged as bridge");

        // Upper crossing should have bridge samples
        let bridge_samples: Vec<_> = spline.samples.iter().filter(|s| s.is_bridge).collect();
        assert!(!bridge_samples.is_empty(), "Upper crossing must be detected as bridge");
        for s in &bridge_samples {
            assert!(s.elevation >= 1.2, "Bridge segment must have elevated clearance");
        }

        // Non-crossing parts should have is_bridge = false
        let sample_start = spline.sample_at_distance(0.0);
        assert!(!sample_start.is_bridge);
    }

    #[test]
    fn test_natural_hill_elevation_no_bridge() {
        // Oval track climbing a massive natural hill (elevation up to 15m), but zero crossovers
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0).with_elevation(0.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0).with_elevation(5.0),
            TrackWaypoint::new(Vec2::new(100.0, 100.0), 10.0).with_elevation(15.0),
            TrackWaypoint::new(Vec2::new(0.0, 100.0), 10.0).with_elevation(7.5),
        ];
        let spline = TrackSpline::new(waypoints, true);

        // There are no track crossovers, so NO samples should be marked as bridge!
        for s in &spline.samples {
            assert!(!s.is_bridge, "Natural hill must never be flagged as bridge");
        }

        // Verify grade slope is non-zero along uphill sections
        let uphill_sample = spline.sample_at_distance(40.0);
        assert!(uphill_sample.grade_slope > 0.0, "Uphill section must have positive grade slope");

        // Verify vertical curvature exists over crest
        let crest_sample = spline.sample_at_distance(spline.total_length() * 0.5);
        assert!(crest_sample.vertical_curvature != 0.0, "Crest transition must have non-zero vertical curvature");
    }

    #[test]
    fn test_track_waypoint_and_spline_runoff_surfaces() {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0)
                .with_runoff_surface(SurfaceType::Gravel),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0)
                .with_runoff_surfaces(Some(SurfaceType::DeepSand), Some(SurfaceType::Asphalt)),
            TrackWaypoint::new(Vec2::new(100.0, 100.0), 10.0),
            TrackWaypoint::new(Vec2::new(0.0, 100.0), 10.0),
        ];
        let spline = TrackSpline::new(waypoints, true);

        // Near WP 0, both left and right should be Gravel
        let s0 = spline.sample_at_distance(0.0);
        assert_eq!(s0.left_runoff_surface, Some(SurfaceType::Gravel));
        assert_eq!(s0.right_runoff_surface, Some(SurfaceType::Gravel));

        // Near WP 1, left should be DeepSand, right should be Asphalt
        let s1 = spline.sample_at_distance(100.0);
        assert_eq!(s1.left_runoff_surface, Some(SurfaceType::DeepSand));
        assert_eq!(s1.right_runoff_surface, Some(SurfaceType::Asphalt));

        // Near WP 2, both should be None
        let s2 = spline.sample_at_distance(200.0);
        assert_eq!(s2.left_runoff_surface, None);
        assert_eq!(s2.right_runoff_surface, None);
    }

    #[test]
    fn test_centripetal_catmull_rom_equidistant_equivalence() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(10.0, 0.0);
        let p2 = Vec2::new(20.0, 10.0);
        let p3 = Vec2::new(30.0, 10.0);

        for step in 0..=10 {
            let t = step as f32 / 10.0;
            let p_centripetal = catmull_rom_centripetal_2d(p0, p1, p2, p3, t);
            let p_uniform = catmull_rom_uniform_2d(p0, p1, p2, p3, t);
            let diff = (p_centripetal - p_uniform).length();
            assert!(
                diff < 0.15,
                "Equidistant knots must produce near-identical results: diff={diff} at t={t}"
            );
        }
    }

    #[test]
    fn test_centripetal_catmull_rom_suppresses_extreme_overshoot() {
        // A 60m straight transitioning into an abrupt 3m turn
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(60.0, 0.0);
        let p2 = Vec2::new(60.0, 3.0);
        let p3 = Vec2::new(57.0, 3.0);

        let mut max_x_uniform = 0.0f32;
        let mut max_x_centripetal = 0.0f32;

        for step in 0..=20 {
            let t = step as f32 / 20.0;
            let u_pt = catmull_rom_uniform_2d(p0, p1, p2, p3, t);
            let c_pt = catmull_rom_centripetal_2d(p0, p1, p2, p3, t);

            max_x_uniform = max_x_uniform.max(u_pt.x);
            max_x_centripetal = max_x_centripetal.max(c_pt.x);

            if step == 0 {
                assert!((c_pt - p1).length() < 1e-4);
            }
            if step == 20 {
                assert!((c_pt - p2).length() < 1e-4);
            }
        }

        // Uniform Catmull-Rom shoots over by > 4.4 meters past x=60
        assert!(
            max_x_uniform > 64.0,
            "Uniform Catmull-Rom must demonstrate massive overshoot (x={max_x_uniform})"
        );

        // Centripetal Catmull-Rom must suppress this overshoot (< 0.6m past x=60)
        assert!(
            max_x_centripetal <= 60.55,
            "Centripetal Catmull-Rom must suppress corner overshoot: x={max_x_centripetal}"
        );
    }

    #[test]
    fn test_adaptive_spline_sampling_density() {
        // Track with a 100m straight and a 4m hairpin apex
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(104.0, 4.0), 10.0),
            TrackWaypoint::new(Vec2::new(100.0, 8.0), 10.0),
            TrackWaypoint::new(Vec2::new(0.0, 8.0), 10.0),
        ];
        let spline = TrackSpline::new(waypoints, true);

        // Waypoint 0 to 1 is 100m: must allocate ~100 steps
        let wp0_idx = spline.waypoint_sample_index(0).unwrap();
        let wp1_idx = spline.waypoint_sample_index(1).unwrap();
        let straight_steps = wp1_idx - wp0_idx;
        assert_eq!(straight_steps, 100);

        // Waypoint 1 to 2 is ~5.65m: must allocate ~6 steps
        let wp2_idx = spline.waypoint_sample_index(2).unwrap();
        let curve_steps = wp2_idx - wp1_idx;
        assert_eq!(curve_steps, 6);

        // Sample delta distances along the entire spline must be bounded around target 1.0m
        for (i, window) in spline.samples.windows(2).enumerate() {
            let step_dist = (window[1].point - window[0].point).length();
            assert!(
                step_dist >= 0.25 && step_dist <= 1.5,
                "Sample step distance must stay bounded near 1.0m (got {step_dist:.3}m at {i})"
            );
        }
    }

    #[test]
    fn test_swallowtail_loop_trimming_on_tight_hairpin() {
        // Construct a sharp 180° hairpin with apex radius R = 2.0m, but road width w = 12.0m (w/2 = 6.0m > R).
        // This causes the raw inner boundary to cross over itself (swallowtail loop).
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(30.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(32.0, 2.0), 12.0),
            TrackWaypoint::new(Vec2::new(30.0, 4.0), 12.0),
            TrackWaypoint::new(Vec2::new(0.0, 4.0), 12.0),
        ];
        let spline = TrackSpline::new(waypoints, true);

        // Raw inner road edge has self-intersections
        let raw_left: Vec<Vec2> = spline.samples.iter().map(|s| s.point + s.normal * (s.width * 0.5)).collect();
        let mut raw_has_self_intersection = false;
        'check_raw: for i in 0..raw_left.len() {
            let seg_a = LineSegment::new(raw_left[i], raw_left[(i + 1) % raw_left.len()]);
            for span in 2..=(15.min(raw_left.len() / 2)) {
                let j = (i + span) % raw_left.len();
                let seg_b = LineSegment::new(raw_left[j], raw_left[(j + 1) % raw_left.len()]);
                if seg_a.intersect_segment(&seg_b).is_some() {
                    raw_has_self_intersection = true;
                    break 'check_raw;
                }
            }
        }
        assert!(raw_has_self_intersection, "Raw inner boundary of R < w/2 hairpin must self-intersect");

        // Untangled road edges must eliminate all local self-intersections
        let (untangled_left, untangled_right) = spline.untangled_road_edges();
        assert_eq!(untangled_left.len(), spline.samples.len());
        assert_eq!(untangled_right.len(), spline.samples.len());

        for pts in [&untangled_left, &untangled_right] {
            for i in 0..pts.len() {
                let p0 = pts[i];
                let p1 = pts[(i + 1) % pts.len()];
                if (p1 - p0).length_squared() < 1e-4 {
                    continue;
                }
                let seg_a = LineSegment::new(p0, p1);
                for span in 2..=(15.min(pts.len() / 2)) {
                    let j = (i + span) % pts.len();
                    let p2 = pts[j];
                    let p3 = pts[(j + 1) % pts.len()];
                    if (p3 - p2).length_squared() < 1e-4 {
                        continue;
                    }
                    let seg_b = LineSegment::new(p2, p3);
                    if (seg_a.start - seg_b.start).length_squared() < 1e-4
                        || (seg_a.start - seg_b.end).length_squared() < 1e-4
                        || (seg_a.end - seg_b.start).length_squared() < 1e-4
                        || (seg_a.end - seg_b.end).length_squared() < 1e-4
                    {
                        continue;
                    }
                    assert!(
                        seg_a.intersect_segment(&seg_b).is_none(),
                        "Untangled boundary must not contain self-intersecting loops at ({i}, {j})"
                    );
                }
            }
        }
    }

    #[test]
    fn test_silverstone_loop_has_zero_self_intersections() {
        let track = crate::track::test_circuit("gt", "silverstone");
        let (left_road, right_road) = track.spline.untangled_road_edges();
        let (left_curb, right_curb) = track.spline.untangled_curb_edges(1.35);

        for edge in [&left_road, &right_road, &left_curb, &right_curb] {
            let n = edge.len();
            for i in 0..n {
                let p0 = edge[i];
                let p1 = edge[(i + 1) % n];
                if (p1 - p0).length_squared() < 1e-4 {
                    continue;
                }
                let seg_a = LineSegment::new(p0, p1);
                for span in 2..=(15.min(n / 2)) {
                    let j = (i + span) % n;
                    let p2 = edge[j];
                    let p3 = edge[(j + 1) % n];
                    if (p3 - p2).length_squared() < 1e-4 {
                        continue;
                    }
                    let seg_b = LineSegment::new(p2, p3);
                    if (seg_a.start - seg_b.start).length_squared() < 1e-4
                        || (seg_a.start - seg_b.end).length_squared() < 1e-4
                        || (seg_a.end - seg_b.start).length_squared() < 1e-4
                        || (seg_a.end - seg_b.end).length_squared() < 1e-4
                    {
                        continue;
                    }
                    assert!(
                        seg_a.intersect_segment(&seg_b).is_none(),
                        "Silverstone boundary must have zero self-intersections at segments {} and {}",
                        i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_synchronized_road_and_curb_untangling_on_hairpin() {
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(30.0, 0.0), 12.0).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(32.0, 2.0), 12.0).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(30.0, 4.0), 12.0).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(0.0, 4.0), 12.0).with_curbs(true, true),
        ];
        let spline = TrackSpline::new(waypoints, true);
        let n = spline.samples.len();

        let (road_l, road_r, curb_l, curb_r) = spline.untangled_boundaries(1.35);
        assert_eq!(road_l.len(), n);
        assert_eq!(road_r.len(), n);
        assert_eq!(curb_l.len(), n);
        assert_eq!(curb_r.len(), n);

        // Verify zero boundary self-intersections on road and curb
        for edge in [&road_l, &road_r, &curb_l, &curb_r] {
            for i in 0..n {
                let p0 = edge[i];
                let p1 = edge[(i + 1) % n];
                if (p1 - p0).length_squared() < 1e-4 {
                    continue;
                }
                let seg_a = LineSegment::new(p0, p1);
                for span in 2..=(15.min(n / 2)) {
                    let j = (i + span) % n;
                    let p2 = edge[j];
                    let p3 = edge[(j + 1) % n];
                    if (p3 - p2).length_squared() < 1e-4 {
                        continue;
                    }
                    let seg_b = LineSegment::new(p2, p3);
                    if (seg_a.start - seg_b.start).length_squared() < 1e-4
                        || (seg_a.start - seg_b.end).length_squared() < 1e-4
                        || (seg_a.end - seg_b.start).length_squared() < 1e-4
                        || (seg_a.end - seg_b.end).length_squared() < 1e-4
                    {
                        continue;
                    }
                    assert!(
                        seg_a.intersect_segment(&seg_b).is_none(),
                        "Synchronized boundary must not contain self-intersections at ({i}, {j})"
                    );
                }
            }
        }

        // Verify synchronized collapse on inner side (left)
        for i in 0..n {
            let next_i = (i + 1) % n;
            let road_collapsed = (road_l[next_i] - road_l[i]).length_squared() < 1e-4;
            let curb_collapsed = (curb_l[next_i] - curb_l[i]).length_squared() < 1e-4;
            if road_collapsed {
                assert!(curb_collapsed, "When road collapses, curb must also collapse at sample {i}");
            }
        }
    }
}

