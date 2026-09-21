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

    /// An empty track spline with no waypoints or samples.
    pub fn empty() -> Self {
        Self {
            waypoints: Vec::new(),
            closed: false,
            samples: Vec::new(),
            total_length: 0.0,
            curves: Vec::new(),
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
            };
        }

        let mut samples = Vec::new();
        let num_wp = waypoints.len();
        let segments = if closed { num_wp } else { num_wp - 1 };

        // 1. Resample each Catmull-Rom segment into fine sub-steps (~16-32 steps per segment)
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
        let mut raw_left_runoff = Vec::new();
        let mut raw_right_runoff = Vec::new();

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

            for s in 0..steps_per_segment {
                let t = s as f32 / steps_per_segment as f32;
                let pt = catmull_rom_2d(p0, p1, p2, p3, t);
                let elev = catmull_rom_1d(e0, e1, e2, e3, t).max(0.0);
                let bank = catmull_rom_1d(b0, b1, b2, b3, t);
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
        }
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
        let elevation = (s0.elevation + (s1.elevation - s0.elevation) * t).max(0.0);
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
                is_on_track: true,
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
        let elevation = (s0.elevation + (s1.elevation - s0.elevation) * best_t).max(0.0);
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

        let tangent = s0.tangent.lerp(s1.tangent, best_t).normalize_or_zero();
        let normal = Vec2::new(-tangent.y, tangent.x);
        let right_vector = Vec2::new(tangent.y, -tangent.x);

        let to_pos = pos - best_point;
        let lateral_offset = to_pos.dot(right_vector);
        let distance_to_spline = best_dist_sq.sqrt();

        let track_width = s0.width + (s1.width - s0.width) * best_t;
        let left_curb = if best_t < 0.5 { s0.left_curb } else { s1.left_curb };
        let right_curb = if best_t < 0.5 { s0.right_curb } else { s1.right_curb };
        let elevation = (s0.elevation + (s1.elevation - s0.elevation) * best_t).max(0.0);
        let bank_angle = s0.bank_angle + (s1.bank_angle - s0.bank_angle) * best_t;
        let grade_slope = s0.grade_slope + (s1.grade_slope - s0.grade_slope) * best_t;
        let vertical_curvature = s0.vertical_curvature + (s1.vertical_curvature - s0.vertical_curvature) * best_t;
        let is_bridge = if best_t < 0.5 { s0.is_bridge } else { s1.is_bridge };

        let half_w = track_width * 0.5;
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

/// 2D Catmull-Rom interpolation for points p0, p1, p2, p3 at parameter t in [0, 1].
#[inline]
fn catmull_rom_2d(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;

    let f0 = -0.5 * t3 + t2 - 0.5 * t;
    let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
    let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
    let f3 = 0.5 * t3 - 0.5 * t2;

    p0 * f0 + p1 * f1 + p2 * f2 + p3 * f3
}

/// 1D Catmull-Rom interpolation for scalar values p0, p1, p2, p3 at parameter t in [0, 1].
#[inline]
fn catmull_rom_1d(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;

    let f0 = -0.5 * t3 + t2 - 0.5 * t;
    let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
    let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
    let f3 = 0.5 * t3 - 0.5 * t2;

    p0 * f0 + p1 * f1 + p2 * f2 + p3 * f3
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
                .with_runoff_surfaces(Some(SurfaceType::Sand), Some(SurfaceType::Asphalt)),
            TrackWaypoint::new(Vec2::new(100.0, 100.0), 10.0),
            TrackWaypoint::new(Vec2::new(0.0, 100.0), 10.0),
        ];
        let spline = TrackSpline::new(waypoints, true);

        // Near WP 0, both left and right should be Gravel
        let s0 = spline.sample_at_distance(0.0);
        assert_eq!(s0.left_runoff_surface, Some(SurfaceType::Gravel));
        assert_eq!(s0.right_runoff_surface, Some(SurfaceType::Gravel));

        // Near WP 1, left should be Sand, right should be Asphalt
        let s1 = spline.sample_at_distance(100.0);
        assert_eq!(s1.left_runoff_surface, Some(SurfaceType::Sand));
        assert_eq!(s1.right_runoff_surface, Some(SurfaceType::Asphalt));

        // Near WP 2, both should be None
        let s2 = spline.sample_at_distance(200.0);
        assert_eq!(s2.left_runoff_surface, None);
        assert_eq!(s2.right_runoff_surface, None);
    }
}
