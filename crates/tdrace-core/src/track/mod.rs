pub mod checkpoint;
pub mod geometry;
pub mod presets;
pub mod spline;
pub mod validation;

pub use checkpoint::{Checkpoint, CheckpointCrossResult, TrackProgressTracker};
pub use geometry::{
    BarrierType, JumpRamp, LineSegment, Obstacle, ObstacleShape, SpawnPose, SurfaceShape,
    SurfaceZone, TrackGeometry, WallBarrier,
};
pub use presets::{
    classic_grand_prix, drift_park, dune_raid, generate_checkpoints, generate_grid_positions,
    generate_walls_from_spline, kart_arena, oasis_rally, outlaw_pass, oval_speedway, ramp_raceway,
    sahara_dunes,
};
pub use spline::{SplineProjection, SplineSample, TrackSpline, TrackWaypoint};
pub use validation::{validate_track, TrackValidationError, ValidationSeverity};

use std::fmt;
use std::fs;
use std::path::Path;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::physics::car::Car;
use crate::physics::surface::SurfaceType;

/// Error type for track parsing, serialization, and file I/O operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackError {
    Io(String),
    Json(String),
    Validation(String),
}

impl fmt::Display for TrackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "Track IO error: {}", msg),
            Self::Json(msg) => write!(f, "Track JSON error: {}", msg),
            Self::Validation(msg) => write!(f, "Track validation error: {}", msg),
        }
    }
}

impl std::error::Error for TrackError {}

fn default_laps_fallback() -> u32 {
    3
}

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
    #[serde(default = "default_laps_fallback")]
    pub default_laps: u32,
    #[serde(default)]
    pub predefined_car: Option<String>,
}

impl Track {
    /// Samples the exact surface type at any arbitrary 2D world coordinate.
    ///
    /// Surface resolution hierarchy:
    /// 1. On-track hazard overlays (`SurfaceType::Water`, `SurfaceType::Oil`, `SurfaceType::Ice`):
    ///    Sitting on top of the road, these affect the vehicle both on and off track.
    /// 2. Track spline projection:
    ///    - Main drivable track ribbon (`SurfaceType::Dirt`, `SurfaceType::Asphalt`).
    ///    - Apex / exit curbs (`SurfaceType::Curb`).
    /// 3. Off-track surface zones (e.g. `SurfaceType::Sand` traps, runoff areas):
    ///    Located underneath the track ribbon, only affecting the vehicle when running off track.
    /// 4. Default off-track terrain (`SurfaceType::Grass`, `SurfaceType::Sand`).
    pub fn sample_surface(&self, point: Vec2) -> SurfaceType {
        // 1. Check on-top hazard overlays first (water puddles, oil slicks, ice)
        for zone in &self.geometry.surface_zones {
            if zone.surface.is_on_track_hazard() && zone.contains(point) {
                return zone.surface;
            }
        }

        // 2. Project onto spline: drivable ribbon (Dirt / Asphalt) and curbs take precedence over underlying ground
        let proj = self.spline.project_point(point);
        if proj.is_on_track {
            return proj.base_surface;
        }
        if proj.is_on_curb {
            return SurfaceType::Curb;
        }

        // 3. Check off-track ground zones (e.g. Sand traps, asphalt runoff)
        for zone in &self.geometry.surface_zones {
            if zone.contains(point) {
                return zone.surface;
            }
        }

        // 4. Default terrain (e.g. Grass or Sand)
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

    /// Deserializes a `Track` from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, TrackError> {
        serde_json::from_str(json_str).map_err(|e| TrackError::Json(e.to_string()))
    }

    /// Serializes this `Track` to a compact JSON string.
    pub fn to_json(&self) -> Result<String, TrackError> {
        serde_json::to_string(self).map_err(|e| TrackError::Json(e.to_string()))
    }

    /// Serializes this `Track` to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, TrackError> {
        serde_json::to_string_pretty(self).map_err(|e| TrackError::Json(e.to_string()))
    }

    /// Loads and deserializes a track file from the local filesystem.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, TrackError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| TrackError::Io(format!("{}: {}", path.as_ref().display(), e)))?;
        Self::from_json(&content)
    }

    /// Saves and serializes this track to a file on the local filesystem.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), TrackError> {
        let json_str = self.to_json_pretty()?;
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| TrackError::Io(format!("{}: {}", parent.display(), e)))?;
            }
        }
        fs::write(path.as_ref(), json_str)
            .map_err(|e| TrackError::Io(format!("{}: {}", path.as_ref().display(), e)))?;
        Ok(())
    }

    /// Performs validation on the track geometry and gameplay rules.
    pub fn validate(&self) -> Vec<TrackValidationError> {
        validate_track(self)
    }

    /// Rebuilds spline samples, boundary polylines, and wall barriers from the spline waypoints.
    pub fn rebuild_geometry(&mut self, barrier_offset: f32, barrier_type: BarrierType) {
        if self.spline.waypoints.len() >= 3 {
            self.spline = TrackSpline::new(self.spline.waypoints.clone(), self.spline.closed);
            let (left_walls, right_walls, left_poly, right_poly) =
                generate_walls_from_spline(&self.spline, barrier_offset, barrier_type);
            self.geometry.inner_walls = left_walls;
            self.geometry.outer_walls = right_walls;
            self.geometry.left_boundary_polyline = left_poly;
            self.geometry.right_boundary_polyline = right_poly;
        }
    }

    /// Regenerates checkpoints evenly spaced along the spline.
    pub fn auto_generate_checkpoints(&mut self, count: usize, num_sectors: usize) {
        if self.spline.samples.len() >= 2 {
            self.checkpoints = generate_checkpoints(&self.spline, count, num_sectors);
        }
    }

    /// Regenerates starting grid positions on the straight before start line.
    pub fn auto_generate_grid(&mut self, num_slots: usize, spacing: f32, stagger: f32) {
        if self.spline.samples.len() >= 2 {
            self.grid_positions = generate_grid_positions(&self.spline, num_slots, spacing, stagger);
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
    fn test_track_json_serialization_roundtrip() {
        let presets = [
            classic_grand_prix(),
            oval_speedway(),
            drift_park(),
            kart_arena(),
            ramp_raceway(),
            oasis_rally(),
            outlaw_pass(),
        ];

        for track in &presets {
            let json = track.to_json_pretty().expect("Must serialize to JSON");
            assert!(!json.is_empty());
            let deserialized = Track::from_json(&json).expect("Must deserialize from JSON");
            assert_eq!(track.name, deserialized.name);
            assert_eq!(track.spline.waypoints.len(), deserialized.spline.waypoints.len());
            assert_eq!(track.checkpoints.len(), deserialized.checkpoints.len());
            assert_eq!(track.grid_positions.len(), deserialized.grid_positions.len());
            assert_eq!(track.geometry.surface_zones.len(), deserialized.geometry.surface_zones.len());
            assert_eq!(track.geometry.jump_ramps.len(), deserialized.geometry.jump_ramps.len());
            assert_eq!(track.geometry.obstacles.len(), deserialized.geometry.obstacles.len());
        }
    }

    #[test]
    fn test_track_rebuild_geometry() {
        let mut track = classic_grand_prix();
        track.rebuild_geometry(5.0, BarrierType::Concrete);
        assert!(!track.geometry.inner_walls.is_empty());
        assert_eq!(track.geometry.inner_walls.first().unwrap().barrier_type, BarrierType::Concrete);
        assert!(!track.geometry.outer_walls.is_empty());
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
