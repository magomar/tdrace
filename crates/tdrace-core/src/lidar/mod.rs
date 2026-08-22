use std::f32::consts::PI;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::collision::sat::OrientedBox;
use crate::physics::car::Car;
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LidarScanner {
    pub config: LidarConfig,
}

impl LidarScanner {
    pub const fn new(config: LidarConfig) -> Self {
        Self { config }
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
    pub fn scan(&self, car: &Car, track: &Track, opponents: &[Car]) -> Vec<LidarHit> {
        let mut results = vec![LidarHit::default(); self.config.num_rays];
        self.scan_into(car, track, opponents, &mut results);
        results
    }

    /// Zero-allocation LIDAR sweep writing directly into a pre-allocated output buffer.
    pub fn scan_into(
        &self,
        car: &Car,
        track: &Track,
        opponents: &[Car],
        out_hits: &mut [LidarHit],
    ) {
        let n = self.config.num_rays.min(out_hits.len());
        if n == 0 {
            return;
        }

        let fwd = car.forward_vector();
        let sensor_pos = car.state.position + fwd * self.config.offset_forward;
        let car_heading = car.state.angle;
        let is_full_360 = (self.config.fov_radians - 2.0 * PI).abs() < 1e-3;

        let max_range = self.config.max_range;
        let max_r_sq = (max_range + 6.0) * (max_range + 6.0);

        // Pre-filter candidate walls within sensor range to avoid testing far away segments
        let candidate_walls: Vec<&crate::track::geometry::WallBarrier> = track
            .geometry
            .all_walls()
            .filter(|w| {
                let mid = (w.segment.start + w.segment.end) * 0.5;
                mid.distance_squared(sensor_pos) < max_r_sq
            })
            .collect();

        // Precompute opponent bounding boxes and velocities
        let opponent_obbs: Vec<(OrientedBox, Vec2)> = opponents
            .iter()
            .map(|opp| (OrientedBox::from_car(opp), opp.state.velocity))
            .collect();

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
            let ray_dir = Vec2::new(ray_angle.cos(), ray_angle.sin());

            out_hits[i] = self.cast_ray_candidates(
                sensor_pos,
                ray_dir,
                car.state.velocity,
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
        let walls: Vec<&crate::track::geometry::WallBarrier> = track.geometry.all_walls().collect();
        self.cast_ray_candidates(
            origin,
            dir,
            host_velocity,
            &walls,
            &track.geometry.obstacles,
            opponents,
        )
    }

    /// Casts a single ray against candidate walls, obstacles, and opponent bounding boxes.
    #[inline]
    pub fn cast_ray_candidates(
        &self,
        origin: Vec2,
        dir: Vec2,
        host_velocity: Vec2,
        candidate_walls: &[&crate::track::geometry::WallBarrier],
        obstacles: &[crate::track::geometry::Obstacle],
        opponents: &[(OrientedBox, Vec2)],
    ) -> LidarHit {
        let max_range = self.config.max_range;
        let mut closest_dist = max_range;
        let mut hit_normal = -dir;
        let mut hit_type = LidarHitType::None;
        let mut relative_vel = Vec2::ZERO;

        // 1. Ray vs Candidate Track Wall Barriers
        for wall in candidate_walls {
            if let Some((dist, normal)) = wall.segment.intersect_ray(origin, dir, closest_dist) {
                if dist < closest_dist {
                    closest_dist = dist;
                    hit_normal = normal;
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
                    hit_normal = normal;
                    hit_type = LidarHitType::Obstacle;
                    relative_vel = -host_velocity;
                }
            }
        }

        // 3. Ray vs Dynamic Opponent Cars
        for (opp_box, opp_vel) in opponents {
            if let Some((dist, normal)) = intersect_ray_obb(origin, dir, opp_box, closest_dist) {
                if dist < closest_dist {
                    closest_dist = dist;
                    hit_normal = normal;
                    hit_type = LidarHitType::OpponentCar;
                    relative_vel = *opp_vel - host_velocity;
                }
            }
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

/// Ray vs OBB intersection helper.
#[inline]
fn intersect_ray_obb(
    origin: Vec2,
    dir: Vec2,
    obb: &OrientedBox,
    max_range: f32,
) -> Option<(f32, Vec2)> {
    let d = origin - obb.center;
    let cos_a = obb.angle.cos();
    let sin_a = obb.angle.sin();

    // Transform ray into OBB local coordinate frame
    let local_origin = Vec2::new(d.x * cos_a + d.y * sin_a, -d.x * sin_a + d.y * cos_a);
    let local_dir = Vec2::new(dir.x * cos_a + dir.y * sin_a, -dir.x * sin_a + dir.y * cos_a);

    let mut t_min = 0.0f32;
    let mut t_max = max_range;
    let mut hit_norm_local = Vec2::ZERO;

    // X slab
    if local_dir.x.abs() > 1e-6 {
        let inv_d = 1.0 / local_dir.x;
        let mut t1 = (-obb.half_extents.x - local_origin.x) * inv_d;
        let mut t2 = (obb.half_extents.x - local_origin.x) * inv_d;
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
    } else if local_origin.x.abs() > obb.half_extents.x {
        return None;
    }

    // Y slab
    if local_dir.y.abs() > 1e-6 {
        let inv_d = 1.0 / local_dir.y;
        let mut t1 = (-obb.half_extents.y - local_origin.y) * inv_d;
        let mut t2 = (obb.half_extents.y - local_origin.y) * inv_d;
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
    } else if local_origin.y.abs() > obb.half_extents.y {
        return None;
    }

    if t_min <= max_range {
        // Transform local normal to world normal
        let world_normal = Vec2::new(
            hit_norm_local.x * cos_a - hit_norm_local.y * sin_a,
            hit_norm_local.x * sin_a + hit_norm_local.y * cos_a,
        );
        Some((t_min, world_normal))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::config::CarConfig;
    use crate::track::presets::classic_grand_prix;

    #[test]
    fn test_lidar_scanner_basic() {
        let track = classic_grand_prix();
        let scanner = LidarScanner::new(LidarConfig::surround_32());
        let car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);

        let hits = scanner.scan(&car, &track, &[]);
        assert_eq!(hits.len(), 32);

        // At (0,0) track width is 14m, walls are at lateral distance ~11m
        let hit_left = hits[8]; // 90 degrees left
        assert!(hit_left.distance > 5.0 && hit_left.distance < 20.0);
        assert_eq!(hit_left.hit_type, LidarHitType::TrackWall);
    }

    #[test]
    fn test_lidar_opponent_detection() {
        let track = classic_grand_prix();
        let scanner = LidarScanner::new(LidarConfig::forward_cone_16());
        let host = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
        let opponent = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(15.0, 0.0), 0.0);

        let hits = scanner.scan(&host, &track, &[opponent]);
        // Center rays (pointing forward) should hit the opponent car
        let center_hit = hits[hits.len() / 2];
        assert_eq!(center_hit.hit_type, LidarHitType::OpponentCar);
        assert!(center_hit.distance < 15.0);
    }
}
