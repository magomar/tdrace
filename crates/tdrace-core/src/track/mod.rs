pub mod checkpoint;
pub mod geometry;
pub mod presets;
pub mod spline;

pub use checkpoint::{Checkpoint, CheckpointCrossResult, TrackProgressTracker};
pub use geometry::{
    BarrierType, LineSegment, Obstacle, ObstacleShape, SpawnPose, SurfaceShape, SurfaceZone,
    TrackGeometry, WallBarrier,
};
pub use presets::{
    classic_grand_prix, drift_park, generate_checkpoints, generate_grid_positions,
    generate_walls_from_spline, kart_arena, oval_speedway,
};
pub use spline::{SplineProjection, SplineSample, TrackSpline, TrackWaypoint};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::physics::car::Car;
use crate::physics::surface::SurfaceType;

/// Complete racing circuit specification including spline, boundaries, surfaces, obstacles, and checkpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub spline: TrackSpline,
    pub geometry: TrackGeometry,
    pub checkpoints: Vec<Checkpoint>,
    pub grid_positions: Vec<SpawnPose>,
    pub default_surface: SurfaceType,
    pub pit_box_area: Option<SurfaceShape>,
}

impl Track {
    /// Samples the exact surface type at any arbitrary 2D world coordinate.
    ///
    /// Surface resolution hierarchy:
    /// 1. Track spline projection:
    ///    - Main drivable track ribbon (`SurfaceType::Asphalt` or segment override).
    ///    - Apex / exit curbs (`SurfaceType::Curb`).
    /// 2. Specific geometric surface zones (hazard areas, sand traps, oil slicks, asphalt runoff).
    ///    Only reachable when off-track: zones are rendered beneath the track ribbon.
    /// 3. Default off-track terrain (`SurfaceType::Grass`).
    pub fn sample_surface(&self, point: Vec2) -> SurfaceType {
        // 1. Project onto spline: drivable ribbon and curbs always win
        let proj = self.spline.project_point(point);
        if proj.is_on_track {
            return proj.base_surface;
        }
        if proj.is_on_curb {
            return SurfaceType::Curb;
        }

        // 2. Check explicit off-track surface zones (sand traps, oil slicks, runoff)
        for zone in &self.geometry.surface_zones {
            if zone.contains(point) {
                return zone.surface;
            }
        }

        // 3. Default terrain
        self.default_surface
    }

    /// Samples surface types underneath all 4 wheels of a car [FL, FR, RL, RR] for split-mu physics.
    #[inline]
    pub fn sample_car_surfaces(&self, car: &Car) -> [SurfaceType; 4] {
        let wheel_positions = car.wheel_positions_world();
        [
            self.sample_surface(wheel_positions[0]),
            self.sample_surface(wheel_positions[1]),
            self.sample_surface(wheel_positions[2]),
            self.sample_surface(wheel_positions[3]),
        ]
    }

    /// Tests if a car's center is currently inside the pit box servicing zone.
    pub fn is_in_pit_box(&self, car: &Car) -> bool {
        if let Some(pit_shape) = &self.pit_box_area {
            pit_shape.contains(car.state.position)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::config::CarConfig;

    #[test]
    fn test_track_presets_creation() {
        let gp = classic_grand_prix();
        assert_eq!(gp.name, "Classic Grand Prix");
        assert!(gp.checkpoints.len() >= 10);
        assert!(!gp.grid_positions.is_empty());

        let oval = oval_speedway();
        assert_eq!(oval.name, "Oval Speedway");
        assert!(oval.spline.total_length() > 400.0);

        let drift = drift_park();
        assert_eq!(drift.name, "Drift Park");
        assert!(!drift.geometry.surface_zones.is_empty());

        let kart = kart_arena();
        assert_eq!(kart.name, "Kart Arena");
    }

    #[test]
    fn test_track_surface_sampling() {
        let track = classic_grand_prix();

        // Sample on start line center (should be Asphalt)
        let surf_start = track.sample_surface(Vec2::new(0.0, 0.0));
        assert_eq!(surf_start, SurfaceType::Asphalt);

        // Sample far outside the track
        let surf_far = track.sample_surface(Vec2::new(0.0, -100.0));
        assert_eq!(surf_far, SurfaceType::Grass);

        // Sample inside sand trap
        let surf_sand = track.sample_surface(Vec2::new(220.0, 330.0));
        assert_eq!(surf_sand, SurfaceType::Sand);

        // Regression: sand trap AABB overlaps the hairpin ribbon (centerline y=310,
        // half-width 6, zone starts at y=315). On-track points must stay Asphalt.
        let surf_overlap = track.sample_surface(Vec2::new(200.0, 313.0));
        assert_eq!(surf_overlap, SurfaceType::Asphalt);

        // Test sample_car_surfaces
        let car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
        let wheel_surfs = track.sample_car_surfaces(&car);
        assert_eq!(wheel_surfs, [SurfaceType::Asphalt; 4]);
    }
}
