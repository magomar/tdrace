use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::physics::surface::SurfaceType;

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
    /// Whether a perimeter barrier wall is installed on the left side of the track.
    #[serde(default = "default_true")]
    pub left_wall: bool,
    /// Whether a perimeter barrier wall is installed on the right side of the track.
    #[serde(default = "default_true")]
    pub right_wall: bool,
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
            left_wall: true,
            right_wall: true,
        }
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

    pub const fn with_surface(mut self, surface: SurfaceType) -> Self {
        self.surface = Some(surface);
        self
    }

    pub const fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
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
    #[serde(default = "default_true")]
    pub left_wall: bool,
    #[serde(default = "default_true")]
    pub right_wall: bool,
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
}

/// Smooth Catmull-Rom spline representation of the racing circuit centerline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackSpline {
    pub waypoints: Vec<TrackWaypoint>,
    pub closed: bool,
    pub samples: Vec<SplineSample>,
    pub total_length: f32,
}

impl TrackSpline {
    /// Standard curb strip width in meters.
    pub const DEFAULT_CURB_WIDTH: f32 = 1.4;

    /// Builds a smooth track spline from a list of waypoints with uniform arc-length resampling.
    pub fn new(waypoints: Vec<TrackWaypoint>, closed: bool) -> Self {
        assert!(waypoints.len() >= 3, "TrackSpline requires at least 3 waypoints");

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
        let mut raw_left_walls = Vec::new();
        let mut raw_right_walls = Vec::new();

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

            let wp1 = &waypoints[i % num_wp];
            let wp2 = &waypoints[(i + 1) % num_wp];

            for s in 0..steps_per_segment {
                let t = s as f32 / steps_per_segment as f32;
                let pt = catmull_rom_2d(p0, p1, p2, p3, t);
                let elev = catmull_rom_1d(e0, e1, e2, e3, t).max(0.0);
                let w = wp1.width + (wp2.width - wp1.width) * t;
                let lc = if t < 0.5 { wp1.left_curb } else { wp2.left_curb };
                let rc = if t < 0.5 { wp1.right_curb } else { wp2.right_curb };
                let lw = if t < 0.5 { wp1.left_wall } else { wp2.left_wall };
                let rw = if t < 0.5 { wp1.right_wall } else { wp2.right_wall };
                let surf = wp1.surface.unwrap_or(SurfaceType::Asphalt);

                raw_points.push(pt);
                raw_widths.push(w);
                raw_left_curbs.push(lc);
                raw_right_curbs.push(rc);
                raw_surfaces.push(surf);
                raw_elevations.push(elev);
                raw_left_walls.push(lw);
                raw_right_walls.push(rw);
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
            raw_left_walls.push(raw_left_walls[0]);
            raw_right_walls.push(raw_right_walls[0]);
        } else {
            let last = waypoints.last().unwrap();
            raw_points.push(last.point);
            raw_widths.push(last.width);
            raw_left_curbs.push(last.left_curb);
            raw_right_curbs.push(last.right_curb);
            raw_surfaces.push(last.surface.unwrap_or(SurfaceType::Asphalt));
            raw_elevations.push(last.elevation);
            raw_left_walls.push(last.left_wall);
            raw_right_walls.push(last.right_wall);
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

        // 3. Build SplineSample list
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
                left_wall: raw_left_walls[i],
                right_wall: raw_right_walls[i],
            });
        }

        Self {
            waypoints,
            closed,
            samples,
            total_length,
        }
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
                left_wall: true,
                right_wall: true,
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
            left_wall: if t < 0.5 { s0.left_wall } else { s1.left_wall },
            right_wall: if t < 0.5 { s0.right_wall } else { s1.right_wall },
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
        }
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
}
