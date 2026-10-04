use std::f32::consts::PI;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::collision::sat::OrientedBox;
use crate::body::Body2D;
use crate::track::Track;

/// Target classification for LIDAR beam impacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LidarHitType {
    /// No collision detected within maximum range.
    None,
    /// Track boundary wall or barrier segment.
    TrackWall,
    /// Static track obstacle (tire bundle, bollard).
    Obstacle,
    /// Dynamic opponent racing vehicle.
    OpponentCar,
}

/// Point measurement from a single LIDAR raycast beam.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LidarHit {
    /// Absolute Euclidean distance in meters to the nearest collision surface.
    pub distance: f32,
    /// Distance normalized into [0.0, 1.0] relative to `max_range` (1.0 = no obstacle in range).
    pub normalized_distance: f32,
    /// 2D world coordinates of the laser impact point.
    pub hit_point: Vec2,
    /// Surface normal vector at the impact point.
    pub hit_normal: Vec2,
    /// Classification of the impacted object.
    pub hit_type: LidarHitType,
    /// Relative velocity vector of the hit object relative to sensor host (m/s).
    pub relative_velocity: Vec2,
}

impl Default for LidarHit {
    fn default() -> Self {
        Self {
            distance: 50.0,
            normalized_distance: 1.0,
            hit_point: Vec2::ZERO,
            hit_normal: Vec2::ZERO,
            hit_type: LidarHitType::None,
            relative_velocity: Vec2::ZERO,
        }
    }
}

/// Configuration settings for the vehicle LIDAR observation sensor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LidarConfig {
    /// Number of discrete laser rays cast per scan sweep.
    pub num_rays: usize,
    /// Total Field of View in radians (e.g. 2*PI for 360 surround, 2.094 for 120-degree cone).
    pub fov_radians: f32,
    /// Maximum detection distance in meters.
    pub max_range: f32,
    /// Longitudinal mounting offset from vehicle CG along forward vector (meters).
    pub offset_forward: f32,
    /// Base angular offset relative to car heading in radians (0.0 = forward centered).
    pub angle_offset: f32,
}

impl Default for LidarConfig {
    fn default() -> Self {
        Self::surround_32()
    }
}

impl LidarConfig {
    /// 360-degree all-around sensor with 32 evenly distributed beams (50m range).
    pub const fn surround_32() -> Self {
        Self {
            num_rays: 32,
            fov_radians: 2.0 * PI,
            max_range: 50.0,
            offset_forward: 1.2,
            angle_offset: 0.0,
        }
    }

    /// 120-degree forward viewing cone with 16 beams (60m range).
    pub const fn forward_cone_16() -> Self {
        Self {
            num_rays: 16,
            fov_radians: 2.0943951, // 120 deg
            max_range: 60.0,
            offset_forward: 1.5,
            angle_offset: 0.0,
        }
    }

    /// 180-degree forward semicircle with 19 beams (Gymnasium CarRacing-v3 style).
    pub const fn gym_carracing_19() -> Self {
        Self {
            num_rays: 19,
            fov_radians: PI,
            max_range: 45.0,
            offset_forward: 1.2,
            angle_offset: 0.0,
        }
    }

    /// High-resolution 64-beam 360 surround scanner (75m range).
    pub const fn surround_64() -> Self {
        Self {
            num_rays: 64,
            fov_radians: 2.0 * PI,
            max_range: 75.0,
            offset_forward: 1.2,
            angle_offset: 0.0,
        }
    }
}

/// High-speed deterministic 2D LIDAR raycaster for RL observations and sensor simulation.
use std::cell::RefCell;

/// High-speed deterministic 2D LIDAR raycaster for RL observations and sensor simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarScanner {
    pub config: LidarConfig,
    #[serde(skip)]
    scratch_walls: RefCell<Vec<crate::track::geometry::WallBarrier>>,
    #[serde(skip)]
    scratch_opponents: RefCell<Vec<LidarPreparedOpponent>>,
}

impl PartialEq for LidarScanner {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
    }
}

impl LidarScanner {
    pub const fn new(config: LidarConfig) -> Self {
        Self {
            config,
            scratch_walls: RefCell::new(Vec::new()),
            scratch_opponents: RefCell::new(Vec::new()),
        }
    }

    /// Computes the ray directions in vehicle local space.
    pub fn compute_ray_angles(&self) -> Vec<f32> {
        let n = self.config.num_rays;
        let mut angles = Vec::with_capacity(n);

        if n == 0 {
            return angles;
        }

        let is_full_360 = (self.config.fov_radians - 2.0 * PI).abs() < 1e-3;

        for i in 0..n {
            let angle = if is_full_360 {
                self.config.angle_offset + (i as f32 / n as f32) * 2.0 * PI
            } else if n == 1 {
                self.config.angle_offset
            } else {
                let fov = self.config.fov_radians;
                self.config.angle_offset - fov * 0.5 + (i as f32 / (n - 1) as f32) * fov
            };
            angles.push(angle);
        }

        angles
    }

    /// Performs a full LIDAR sweep from the car's perspective against track boundaries, obstacles, and opponent cars.
    pub fn scan<B: Body2D>(&self, car: &B, track: &Track, opponents: &[B]) -> Vec<LidarHit> {
        let mut results = vec![LidarHit::default(); self.config.num_rays];
        self.scan_into(car, track, opponents, &mut results);
        results
    }

    /// Zero-allocation LIDAR sweep writing directly into a pre-allocated output buffer (Spec 084).
    pub fn scan_into<B: Body2D>(
        &self,
        car: &B,
        track: &Track,
        opponents: &[B],
        out_hits: &mut [LidarHit],
    ) {
        let n = self.config.num_rays.min(out_hits.len());
        if n == 0 {
            return;
        }

        let fwd = car.forward_vector();
        let sensor_pos = car.position() + fwd * self.config.offset_forward;
        let car_heading = car.angle();
        let is_full_360 = (self.config.fov_radians - 2.0 * PI).abs() < 1e-3;

        let max_range = self.config.max_range;
        let max_r_sq = (max_range + 6.0) * (max_range + 6.0);
        let car_elev = car.total_elevation();
        let max_range_padded = max_range + 6.0;
        let min_x = sensor_pos.x - max_range_padded;
        let max_x = sensor_pos.x + max_range_padded;
        let min_y = sensor_pos.y - max_range_padded;
        let max_y = sensor_pos.y + max_range_padded;

        // Pre-filter candidate walls using reusable scratch buffer with zero heap allocations
        let mut candidate_walls = self.scratch_walls.borrow_mut();
        candidate_walls.clear();
        for w in track.geometry.all_walls() {
            if !w.is_physical() {
                continue;
            }
            if (car_elev - w.elevation).abs() > 2.0 {
                continue;
            }
            let w_min_x = w.segment.start.x.min(w.segment.end.x);
            let w_max_x = w.segment.start.x.max(w.segment.end.x);
            let w_min_y = w.segment.start.y.min(w.segment.end.y);
            let w_max_y = w.segment.start.y.max(w.segment.end.y);
            if w_max_x < min_x || w_min_x > max_x || w_max_y < min_y || w_min_y > max_y {
                continue;
            }
            if w.segment.distance_sq_to_point(sensor_pos) < max_r_sq {
                candidate_walls.push(*w);
            }
        }

        // Precompute opponent bounding boxes using reusable scratch buffer
        let mut opponent_obbs = self.scratch_opponents.borrow_mut();
        opponent_obbs.clear();
        for opp in opponents {
            if (car_elev - opp.total_elevation()).abs() <= 2.0 {
                let obb = OrientedBox::from_body(opp);
                let (sin_a, cos_a) = obb.angle.sin_cos();
                let d = sensor_pos - obb.center;
                let local_origin = Vec2::new(d.x * cos_a + d.y * sin_a, -d.x * sin_a + d.y * cos_a);
                opponent_obbs.push(LidarPreparedOpponent {
                    local_origin,
                    half_extents: obb.half_extents,
                    cos_a,
                    sin_a,
                    opp_vel: opp.velocity(),
                });
            }
        }

        for i in 0..n {
            let angle_rel = if is_full_360 {
                self.config.angle_offset + (i as f32 / n as f32) * 2.0 * PI
            } else if n == 1 {
                self.config.angle_offset
            } else {
                let fov = self.config.fov_radians;
                self.config.angle_offset - fov * 0.5 + (i as f32 / (n - 1) as f32) * fov
            };

            let ray_angle = car_heading + angle_rel;
            let (sin, cos) = ray_angle.sin_cos();
            let ray_dir = Vec2::new(cos, sin);

            out_hits[i] = self.cast_ray_candidates(
                sensor_pos,
                ray_dir,
                car.velocity(),
                &candidate_walls,
                &track.geometry.obstacles,
                &opponent_obbs,
            );
        }
    }

    /// Casts a single ray against track walls, obstacles, and opponent bounding boxes.
    #[inline]
    pub fn cast_single_ray(
        &self,
        origin: Vec2,
        dir: Vec2,
        host_velocity: Vec2,
        track: &Track,
        opponents: &[(OrientedBox, Vec2)],
    ) -> LidarHit {
        let mut walls = self.scratch_walls.borrow_mut();
        walls.clear();
        for w in track.geometry.all_walls() {
            if w.is_physical() {
                walls.push(*w);
            }
        }
        let mut opps = self.scratch_opponents.borrow_mut();
        opps.clear();
        for (obb, vel) in opponents {
            let (sin_a, cos_a) = obb.angle.sin_cos();
            let d = origin - obb.center;
            let local_origin = Vec2::new(d.x * cos_a + d.y * sin_a, -d.x * sin_a + d.y * cos_a);
            opps.push(LidarPreparedOpponent {
                local_origin,
                half_extents: obb.half_extents,
                cos_a,
                sin_a,
                opp_vel: *vel,
            });
        }
        self.cast_ray_candidates(
            origin,
            dir,
            host_velocity,
            &walls,
            &track.geometry.obstacles,
            &opps,
        )
    }

    /// Casts a single ray against candidate walls, obstacles, and opponent bounding boxes.
    #[inline]
    pub fn cast_ray_candidates(
        &self,
        origin: Vec2,
        dir: Vec2,
        host_velocity: Vec2,
        candidate_walls: &[crate::track::geometry::WallBarrier],
        obstacles: &[crate::track::geometry::Obstacle],
        opponents: &[LidarPreparedOpponent],
    ) -> LidarHit {
        let max_range = self.config.max_range;
        let mut closest_dist = max_range;
        let mut hit_normal = -dir;
        let mut hit_type = LidarHitType::None;
        let mut relative_vel = Vec2::ZERO;

        let mut winning_wall: Option<&crate::track::geometry::WallBarrier> = None;
        let mut winning_obs_normal: Option<Vec2> = None;
        let mut winning_opp_normal: Option<Vec2> = None;

        // 1. Ray vs Candidate Track Wall Barriers (fast distance test without normal computation)
        for wall in candidate_walls {
            if let Some(dist) = wall.segment.intersect_ray_dist(origin, dir, closest_dist) {
                if dist < closest_dist {
                    closest_dist = dist;
                    winning_wall = Some(wall);
                    hit_type = LidarHitType::TrackWall;
                    relative_vel = -host_velocity;
                }
            }
        }

        // 2. Ray vs Static Obstacles
        for obs in obstacles {
            if let Some((dist, normal)) = obs.intersect_ray(origin, dir, closest_dist) {
                if dist < closest_dist {
                    closest_dist = dist;
                    winning_wall = None;
                    winning_obs_normal = Some(normal);
                    winning_opp_normal = None;
                    hit_type = LidarHitType::Obstacle;
                    relative_vel = -host_velocity;
                }
            }
        }

        // 3. Ray vs Dynamic Opponent Cars
        for opp in opponents {
            if let Some((dist, normal)) = intersect_ray_prepared_obb(dir, opp, closest_dist) {
                if dist < closest_dist {
                    closest_dist = dist;
                    winning_wall = None;
                    winning_obs_normal = None;
                    winning_opp_normal = Some(normal);
                    hit_type = LidarHitType::OpponentCar;
                    relative_vel = opp.opp_vel - host_velocity;
                }
            }
        }

        // Compute surface normal only for the winning impact surface
        if let Some(wall) = winning_wall {
            let mut normal = wall.segment.normal();
            if normal.dot(dir) > 0.0 {
                normal = -normal;
            }
            hit_normal = normal;
        } else if let Some(normal) = winning_obs_normal {
            hit_normal = normal;
        } else if let Some(normal) = winning_opp_normal {
            hit_normal = normal;
        }

        let normalized_distance = (closest_dist / max_range).clamp(0.0, 1.0);
        let hit_point = origin + dir * closest_dist;

        LidarHit {
            distance: closest_dist,
            normalized_distance,
            hit_point,
            hit_normal,
            hit_type,
            relative_velocity: relative_vel,
        }
    }
}

/// Precomputed opponent bounding geometry in sensor-relative space (Spec 084).
/// Hoists trigonometric angle resolution and origin translation outside per-ray loops.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LidarPreparedOpponent {
    pub local_origin: Vec2,
    pub half_extents: Vec2,
    pub cos_a: f32,
    pub sin_a: f32,
    pub opp_vel: Vec2,
}

/// Ray vs precomputed OBB intersection helper.
#[inline(always)]
fn intersect_ray_prepared_obb(
    dir: Vec2,
    opp: &LidarPreparedOpponent,
    max_range: f32,
) -> Option<(f32, Vec2)> {
    let local_dir = Vec2::new(
        dir.x * opp.cos_a + dir.y * opp.sin_a,
        -dir.x * opp.sin_a + dir.y * opp.cos_a,
    );

    let mut t_min = 0.0f32;
    let mut t_max = max_range;
    let mut hit_norm_local = Vec2::ZERO;

    // X slab
    if local_dir.x.abs() > 1e-6 {
        let inv_d = 1.0 / local_dir.x;
        let mut t1 = (-opp.half_extents.x - opp.local_origin.x) * inv_d;
        let mut t2 = (opp.half_extents.x - opp.local_origin.x) * inv_d;
        let mut n1 = Vec2::new(-1.0, 0.0);
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
            n1 = Vec2::new(1.0, 0.0);
        }
        if t1 > t_min {
            t_min = t1;
            hit_norm_local = n1;
        }
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    } else if opp.local_origin.x.abs() > opp.half_extents.x {
        return None;
    }

    // Y slab
    if local_dir.y.abs() > 1e-6 {
        let inv_d = 1.0 / local_dir.y;
        let mut t1 = (-opp.half_extents.y - opp.local_origin.y) * inv_d;
        let mut t2 = (opp.half_extents.y - opp.local_origin.y) * inv_d;
        let mut n1 = Vec2::new(0.0, -1.0);
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
            n1 = Vec2::new(0.0, 1.0);
        }
        if t1 > t_min {
            t_min = t1;
            hit_norm_local = n1;
        }
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    } else if opp.local_origin.y.abs() > opp.half_extents.y {
        return None;
    }

    if t_min <= 0.0 {
        return None;
    }

    let world_norm = Vec2::new(
        hit_norm_local.x * opp.cos_a - hit_norm_local.y * opp.sin_a,
        hit_norm_local.x * opp.sin_a + hit_norm_local.y * opp.cos_a,
    );
    Some((t_min, world_norm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wheelbase::{Car, CarConfig};

    #[test]
    fn test_lidar_scanner_basic() {
        let track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        let scanner = LidarScanner::new(LidarConfig::surround_32());
        let pos = track.grid_positions[0].position;
        let angle = track.grid_positions[0].angle;
        let car = Car::new(CarConfig::sports_car()).with_pose(pos, angle);

        let hits = scanner.scan(&car, &track, &[]);
        assert_eq!(hits.len(), 32);

        let hit_left = hits[8]; // 90 degrees left
        assert!(hit_left.distance > 3.0 && hit_left.distance < 30.0);
        assert_eq!(hit_left.hit_type, LidarHitType::TrackWall);
    }

    #[test]
    fn test_lidar_opponent_detection() {
        let track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        let scanner = LidarScanner::new(LidarConfig::forward_cone_16());
        let pos = track.grid_positions[0].position;
        let angle = track.grid_positions[0].angle;
        let fwd = Vec2::from_angle(angle);
        let host = Car::new(CarConfig::sports_car()).with_pose(pos, angle);
        let opponent = Car::new(CarConfig::sports_car()).with_pose(pos + fwd * 15.0, angle);

        let hits = scanner.scan(&host, &track, &[opponent]);
        // Center rays (pointing forward) should hit the opponent car
        let center_hit = hits[hits.len() / 2];
        assert_eq!(center_hit.hit_type, LidarHitType::OpponentCar);
        assert!(center_hit.distance < 15.0);
    }

    #[test]
    fn test_lidar_ignores_virtual_barrier() {
        let mut track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        let pos = track.grid_positions[0].position;
        let angle = track.grid_positions[0].angle;
        let fwd = Vec2::from_angle(angle);
        let right = fwd.perp();
        let barrier_pos = pos + fwd * 5.0;

        // Insert a virtual barrier right in front of the car
        track.geometry.outer_walls.push(crate::track::geometry::WallBarrier::new(
            barrier_pos - right * 10.0,
            barrier_pos + right * 10.0,
            crate::track::geometry::BarrierType::Virtual,
        ));

        let scanner = LidarScanner::new(LidarConfig::forward_cone_16());
        let host = Car::new(CarConfig::sports_car()).with_pose(pos, angle);

        let hits = scanner.scan(&host, &track, &[]);
        let center_hit = hits[hits.len() / 2];
        // Center ray should pass through virtual barrier at 5m and hit the actual wall or nothing at 5m
        assert!(
            (center_hit.distance - 5.0).abs() > 0.5,
            "Lidar must pass straight through virtual barrier at 5m"
        );
    }
}
