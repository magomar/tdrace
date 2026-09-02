pub mod checkpoint;
pub mod geometry;
pub mod presets;
pub mod spline;
pub mod validation;

pub use checkpoint::{Checkpoint, CheckpointCrossResult, TrackProgressTracker};
pub use geometry::{
    BarrierType, JumpRamp, LineSegment, Obstacle, ObstacleShape, SpawnPose, SurfaceLayer,
    SurfaceShape, SurfaceZone, TrackGeometry, WallBarrier,
};
pub use presets::{
    classic_grand_prix, dirt_figure_eight, dirt_oval_speedway, dirty_oval_speedway, drift_park, dune_raid,
    generate_checkpoints, generate_grid_positions, generate_grid_positions_at_distance, generate_walls_from_spline, hell_rx, holjes_rx,
    kart_arena, loheac_rx, lydden_hill, oasis_rally, outlaw_pass, oval_speedway, ramp_raceway,
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

/// Category of a racing circuit: tested & approved Main track, or experimental Draft track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackCategory {
    Main,
    Draft,
}

impl Default for TrackCategory {
    fn default() -> Self {
        Self::Main
    }
}

/// Complete racing circuit specification including spline, boundaries, surfaces, obstacles, and checkpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: TrackCategory,
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
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub modules: Vec<String>,
}

impl Track {
    /// Returns true if this track belongs to the specified motorsport module ID.
    pub fn belongs_to_module(&self, mod_id: &str) -> bool {
        if self.modules.iter().any(|m| m.eq_ignore_ascii_case(mod_id)) {
            return true;
        }
        if let Some(ref m) = self.module_id {
            if m.eq_ignore_ascii_case(mod_id) {
                return true;
            }
        }
        false
    }

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
        // 1. Check jump ramps (elevated platforms)
        for ramp in &self.geometry.jump_ramps {
            if ramp.contains(point) {
                return ramp.surface;
            }
        }

        // 2. Check above-track surface zones first (e.g. puddles, oil slicks, sand/grass placed on top of road)
        for zone in &self.geometry.surface_zones {
            if zone.is_above_track() && zone.contains(point) {
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

        // 3. Check below-track ground zones (e.g. sand traps, asphalt runoff, dirt base beneath road)
        for zone in &self.geometry.surface_zones {
            if !zone.is_above_track() && zone.contains(point) {
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

    /// Returns true if this track has at least one checkpoint designated as the start/finish line.
    pub fn has_finish_line(&self) -> bool {
        self.checkpoints.iter().any(|cp| cp.is_finish_line)
    }

    /// Returns the first checkpoint designated as the start/finish line, if one exists.
    pub fn finish_line_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|cp| cp.is_finish_line)
    }

    /// Regenerates starting grid positions on the straight before the finish line.
    ///
    /// If no finish line checkpoint exists in `self.checkpoints` or the spline has fewer than 2 samples,
    /// no positions are generated and `false` is returned.
    /// Returns `true` if grid positions were successfully generated.
    pub fn auto_generate_grid(&mut self, num_slots: usize, spacing: f32, stagger: f32) -> bool {
        if self.spline.samples.len() < 2 {
            return false;
        }

        let Some(finish_cp) = self.checkpoints.iter().find(|cp| cp.is_finish_line) else {
            return false;
        };

        let finish_gate_center = (finish_cp.gate.start + finish_cp.gate.end) * 0.5;
        let finish_dist = self.spline.project_point(finish_gate_center).progress_distance;

        self.grid_positions = generate_grid_positions_at_distance(&self.spline, finish_dist, num_slots, spacing, stagger);
        true
    }

    /// Returns total centerline track length in meters.
    #[inline]
    pub fn total_length_m(&self) -> f32 {
        self.spline.total_length()
    }

    /// Computes the breakdown of drivable surface types along the circuit as percentages (0.0 to 100.0).
    pub fn surface_breakdown(&self) -> Vec<(SurfaceType, f32)> {
        let samples = &self.spline.samples;
        if samples.len() < 2 {
            return vec![(self.default_surface, 100.0)];
        }
        let mut surface_lengths: std::collections::HashMap<SurfaceType, f32> = std::collections::HashMap::new();
        let mut total_len = 0.0;
        let n = if self.spline.closed { samples.len() } else { samples.len().saturating_sub(1) };
        for i in 0..n {
            let next_i = (i + 1) % samples.len();
            let seg_len = (samples[next_i].point - samples[i].point).length();
            let surf = samples[i].surface;
            *surface_lengths.entry(surf).or_insert(0.0) += seg_len;
            total_len += seg_len;
        }
        if total_len <= 1e-4 {
            return vec![(self.default_surface, 100.0)];
        }
        let mut breakdown: Vec<(SurfaceType, f32)> = surface_lengths
            .into_iter()
            .map(|(surf, len)| (surf, (len / total_len) * 100.0))
            .collect();
        // Sort descending by percentage
        breakdown.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        breakdown
    }

    /// Formats the surface breakdown as a human-readable summary string (e.g. "80% Asphalt, 20% Dirt" or "100% Dirt").
    pub fn surface_summary_string(&self) -> String {
        let breakdown = self.surface_breakdown();
        if breakdown.is_empty() {
            return "Asphalt".to_string();
        }
        if breakdown.len() == 1 {
            return format!("100% {}", breakdown[0].0.name());
        }
        breakdown
            .iter()
            .map(|(surf, pct)| format!("{:.0}% {}", pct.round(), surf.name()))
            .collect::<Vec<_>>()
            .join(", ")
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
            dirt_figure_eight(),
            holjes_rx(),
            lydden_hill(),
            hell_rx(),
            loheac_rx(),
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

    #[test]
    fn test_track_surface_breakdown() {
        let gp = classic_grand_prix();
        let gp_breakdown = gp.surface_breakdown();
        assert_eq!(gp_breakdown.len(), 1);
        assert_eq!(gp_breakdown[0].0, SurfaceType::Asphalt);
        assert!((gp_breakdown[0].1 - 100.0).abs() < 1e-3);
        assert_eq!(gp.surface_summary_string(), "100% Asphalt");
        assert!(gp.total_length_m() > 400.0);

        let rally = oasis_rally();
        let rally_breakdown = rally.surface_breakdown();
        assert_eq!(rally_breakdown.len(), 1);
        assert_eq!(rally_breakdown[0].0, SurfaceType::Dirt);
        assert!((rally_breakdown[0].1 - 100.0).abs() < 1e-3);
        assert_eq!(rally.surface_summary_string(), "100% Dirt");

        // Custom mixed-surface spline
        let mut mixed = classic_grand_prix();
        let n = mixed.spline.waypoints.len();
        for i in 0..n / 2 {
            mixed.spline.waypoints[i].surface = Some(SurfaceType::Dirt);
        }
        mixed.rebuild_geometry(5.0, BarrierType::Concrete);
        let mixed_breakdown = mixed.surface_breakdown();
        assert_eq!(mixed_breakdown.len(), 2);
        let summary = mixed.surface_summary_string();
        assert!(summary.contains("Dirt") && summary.contains("Asphalt"));
    }

    #[test]
    fn test_surface_layer_precedence_and_shapes() {
        let mut track = classic_grand_prix();
        track.geometry.surface_zones.clear();

        // 1. BelowTrack Sand Zone overlapping the start line (0,0)
        let sand_shape = SurfaceShape::Aabb {
            min: Vec2::new(-20.0, -20.0),
            max: Vec2::new(20.0, 20.0),
        };
        track.geometry.surface_zones.push(
            SurfaceZone::new(sand_shape, SurfaceType::Sand, "Sand Under Road")
                .with_layer(SurfaceLayer::BelowTrack),
        );

        // Point on track (0, 0) should remain Asphalt because track ribbon sits above BelowTrack sand
        assert_eq!(track.sample_surface(Vec2::new(0.0, 0.0)), SurfaceType::Asphalt);
        // Off-track point (0, 15) is outside the 14m wide track ribbon (half-width 7m) but inside the [-20..20, -20..20] sand zone
        assert_eq!(track.sample_surface(Vec2::new(0.0, 15.0)), SurfaceType::Sand);

        // 2. AboveTrack Sand Zone overlapping the start line (0,0)
        track.geometry.surface_zones[0].layer = SurfaceLayer::AboveTrack;
        // Point on track (0, 0) should now be Sand because AboveTrack sits on top of asphalt!
        assert_eq!(track.sample_surface(Vec2::new(0.0, 0.0)), SurfaceType::Sand);

        // 3. Triangle Surface Shape test
        let tri_shape = SurfaceShape::triangle(
            Vec2::new(50.0, 50.0),
            Vec2::new(70.0, 50.0),
            Vec2::new(60.0, 70.0),
        );
        let tri_zone = SurfaceZone::new(tri_shape, SurfaceType::Dirt, "Triangle Dirt Zone");
        assert!(tri_zone.contains(Vec2::new(60.0, 55.0)));
        assert!(!tri_zone.contains(Vec2::new(50.0, 70.0)));
        assert_eq!(tri_zone.shape.center(), Vec2::new(60.0, 170.0 / 3.0));

        // 4. Polygon Surface Shape test
        let poly_shape = SurfaceShape::Polygon {
            vertices: vec![
                Vec2::new(100.0, 100.0),
                Vec2::new(120.0, 100.0),
                Vec2::new(120.0, 120.0),
                Vec2::new(100.0, 120.0),
            ],
        };
        let poly_zone = SurfaceZone::new(poly_shape, SurfaceType::Water, "Poly Water Zone");
        assert!(poly_zone.contains(Vec2::new(110.0, 110.0)));
        assert!(!poly_zone.contains(Vec2::new(130.0, 130.0)));
        assert_eq!(poly_zone.shape.center(), Vec2::new(110.0, 110.0));
    }
}
