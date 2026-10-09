pub mod bake;
pub mod checkpoint;
pub mod curve;
pub mod geometry;
pub mod junction_kit;
pub mod network;
pub mod pit_kit;
pub mod presets;
pub mod scenery;
pub mod spline;
pub mod validation;

pub use checkpoint::{Checkpoint, CheckpointCrossResult, MultiRouteProgressTracker, TrackProgressTracker};
pub use curve::{
    classify_curve_degree, compute_safe_apex_speed, evaluate_curve_approach,
    CurveApproachStatus, CurveDirection, TrackCurve,
};
pub use network::{
    compute_split_width_envelope, GoreConfig, JunctionId, JunctionKind, MergeConfig,
    RoadJunction, RoadSegment, SegmentId, SocketId, SplineSocket, TrackLayout, TrackNetwork,
};
pub use geometry::{
    point_in_polygon, point_in_quad_2d, point_in_triangle_2d, BarrierType, JumpRamp,
    JumpRampCarExt, LineSegment, Obstacle, ObstacleShape, PitBox, PitLane, PitLaneChevron,
    PitLaneExitQuad, PitLaneJunctionData, SpawnPose, SurfaceLayer, SurfaceShape, SurfaceZone,
    TrackGeometry, WallBarrier,
};
pub use scenery::{
    Building, BuildingStyle, Grandstand, GrandstandStyle, Rock, RockType, Tree, TreeType,
};
pub use presets::{
    classic_template, create_prototypical_track, generate_arena_grid, generate_checkpoints, generate_grid_positions, generate_grid_positions_at_distance, generate_horizontal_eight_waypoints, generate_oval_waypoints, generate_walls_from_spline, generate_walls_from_spline_raw, gt_template, kart_template, merge_collinear_walls, rally_template, RaceDirection, TrackShape,
};
pub use spline::{SplineProjection, SplineSample, TrackSpline, TrackWaypoint};
pub use validation::{validate_track, TrackValidationError, ValidationSeverity};
pub use crate::car_category::CarCategory;

use std::fmt;
use std::fs;
use std::path::Path;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use wheelbase::Car;
use wheelbase::SurfaceType;

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

/// Topological classification of a racing circuit: Circuit, Arena, or Hybrid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TrackKind {
    /// Traditional 1D continuous spline ribbon with extruded drivable width.
    Circuit,
    /// Bounded 2D open enclosure with fully playable interior floor.
    Arena {
        /// Polygon vertices defining the perimeter boundary enclosure.
        boundary_hull: Vec<Vec2>,
        /// Primary surface type across the entire arena floor (e.g. Dirt, Mud, Ice).
        floor_surface: SurfaceType,
        /// Perimeter barrier specification enclosing the arena.
        #[serde(default)]
        perimeter_barrier: Option<BarrierType>,
    },
    /// Hybrid venue: A stadium bowl enclosing both a defined rhythm track and open infield.
    Hybrid {
        boundary_hull: Vec<Vec2>,
        floor_surface: SurfaceType,
        #[serde(default)]
        perimeter_barrier: Option<BarrierType>,
    },
}

impl Default for TrackKind {
    fn default() -> Self {
        Self::Circuit
    }
}

fn default_scale_fallback() -> String {
    "1:1".to_string()
}

/// Complete racing circuit specification including spline, boundaries, surfaces, obstacles, and checkpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: TrackCategory,
    #[serde(default)]
    pub kind: TrackKind,
    pub spline: TrackSpline,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<TrackNetwork>,
    pub geometry: TrackGeometry,
    pub checkpoints: Vec<Checkpoint>,
    pub grid_positions: Vec<SpawnPose>,
    pub default_surface: SurfaceType,
    pub pit_box_area: Option<SurfaceShape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pit_lane: Option<PitLane>,
    /// Precomputed runtime junction geometry and spatial AABB bounding box for pit lane.
    #[serde(default, skip_serializing)]
    pub pit_lane_junctions: Option<PitLaneJunctionData>,
    /// Source of truth for `pit_lane` when present (spec 101); the bake compiles it into `pit_lane`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pit_lane_layout: Option<pit_kit::PitLaneLayout>,
    /// Precomputed runtime barrier offset in meters to avoid expensive wall geometry sweeps.
    #[serde(default, skip_serializing)]
    pub cached_barrier_offset: Option<f32>,
    #[serde(default = "default_laps_fallback")]
    pub default_laps: u32,
    #[serde(default)]
    pub car_category: CarCategory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub car_model_id: Option<String>,
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default = "default_scale_fallback")]
    pub scale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wikipedia_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub osm_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
    #[serde(default)]
    pub is_inspired: bool,
    /// Short uppercase catalog badge, e.g. "WORLD RALLYCROSS SWEDEN".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tag: String,
    /// Catalog class shown in circuit lists, e.g. "World Rallycross" or "Superspeedway".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub category_label: String,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            category: TrackCategory::Main,
            kind: TrackKind::Circuit,
            spline: TrackSpline::default(),
            network: None,
            geometry: TrackGeometry::default(),
            checkpoints: Vec::new(),
            grid_positions: Vec::new(),
            default_surface: SurfaceType::Grass,
            pit_box_area: None,
            pit_lane: None,
            pit_lane_junctions: None,
            pit_lane_layout: None,
            cached_barrier_offset: None,
            default_laps: 3,
            car_category: CarCategory::Gt,
            car_model_id: None,
            module_id: None,
            modules: Vec::new(),
            scale: "1:1".to_string(),
            wikipedia_url: None,
            osm_url: None,
            country_code: None,
            country_name: None,
            min_width: None,
            max_width: None,
            is_inspired: false,
            tag: String::new(),
            category_label: String::new(),
        }
    }
}

impl Track {
    /// Ensures that the track network exists, promoting the legacy single spline if needed.
    pub fn ensure_network(&mut self) -> &mut TrackNetwork {
        if self.network.is_none() {
            self.network = Some(TrackNetwork::from_single_spline(&self.spline));
        }
        self.network.as_mut().unwrap()
    }

    /// Returns the active track network, falling back to a synthesized single-spline network.
    pub fn active_network(&self) -> TrackNetwork {
        self.network
            .clone()
            .unwrap_or_else(|| TrackNetwork::from_single_spline(&self.spline))
    }

    /// Returns the scale of the track model (defaults to "1:1").
    pub fn scale(&self) -> &str {
        if self.scale.is_empty() {
            "1:1"
        } else {
            &self.scale
        }
    }

    /// Sets the scale of the track model.
    pub fn with_scale(mut self, scale: impl Into<String>) -> Self {
        self.scale = scale.into();
        self
    }

    /// Sets the external reference URLs (Wikipedia and OSM) for the track.
    pub fn with_urls(mut self, wikipedia: Option<String>, osm: Option<String>) -> Self {
        self.wikipedia_url = wikipedia;
        self.osm_url = osm;
        self
    }

    /// Sets country metadata for this track.
    pub fn with_country(mut self, code: impl Into<String>, name: impl Into<String>) -> Self {
        self.country_code = Some(code.into());
        self.country_name = Some(name.into());
        self
    }

    /// Sets complete real-world provenance metadata (country, OSM relation/way URL, Wikipedia URL).
    pub fn with_provenance(
        mut self,
        country_code: &str,
        country_name: &str,
        osm_url: &str,
        wikipedia_url: &str,
    ) -> Self {
        self.country_code = Some(country_code.to_string());
        self.country_name = Some(country_name.to_string());
        self.osm_url = Some(osm_url.to_string());
        self.wikipedia_url = Some(wikipedia_url.to_string());
        self
    }

    /// Computes or retrieves the minimum and maximum drivable track width in meters.
    pub fn width_range(&self) -> (f32, f32) {
        if let (Some(min), Some(max)) = (self.min_width, self.max_width) {
            return (min, max);
        }
        let mut min_w = f32::INFINITY;
        let mut max_w = f32::NEG_INFINITY;
        for wp in &self.spline.waypoints {
            if wp.width > 0.0 {
                min_w = min_w.min(wp.width);
                max_w = max_w.max(wp.width);
            }
        }
        if min_w.is_infinite() {
            for s in &self.spline.samples {
                if s.width > 0.0 {
                    min_w = min_w.min(s.width);
                    max_w = max_w.max(s.width);
                }
            }
        }
        if min_w.is_infinite() {
            (12.0, 12.0)
        } else {
            (min_w, max_w)
        }
    }

    /// Formats the track width range as a human-readable string (e.g. "13.5m" or "11.0–15.0m").
    pub fn width_summary_string(&self) -> String {
        let (min, max) = self.width_range();
        if (min - max).abs() < 0.1 {
            format!("{:.1}m", min)
        } else {
            format!("{:.1}–{:.1}m", min, max)
        }
    }
}

impl Track {
    /// Returns true if this track is an open arena or hybrid stadium venue.
    pub fn is_arena(&self) -> bool {
        matches!(self.kind, TrackKind::Arena { .. } | TrackKind::Hybrid { .. })
    }

    /// Returns the perimeter boundary hull vertices if this track has an arena enclosure.
    pub fn arena_hull(&self) -> Option<&[Vec2]> {
        match &self.kind {
            TrackKind::Arena { boundary_hull, .. } => Some(boundary_hull),
            TrackKind::Hybrid { boundary_hull, .. } => Some(boundary_hull),
            TrackKind::Circuit => None,
        }
    }

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
    /// 1. On-track hazard overlays (`SurfaceType::Water`, `SurfaceType::Oil`, `SurfaceType::SheetIce`):
    ///    Sitting on top of the road, these affect the vehicle both on and off track.
    /// 2. Track spline projection:
    ///    - Main drivable track ribbon (`SurfaceType::Dirt`, `SurfaceType::Asphalt`).
    ///    - Apex / exit curbs (`SurfaceType::Curb`).
    ///    - Then the road, curbs and junction areas of `network` segments (e.g. a joker branch).
    /// 3. Segment runoff corridor:
    ///    - Off-track corridor between track/curb edge and segment wall boundary (`left_runoff_surface` / `right_runoff_surface`).
    /// 4. Arena / Hybrid open floor surface:
    ///    - Full-floor drivable ground inside perimeter boundary hull.
    /// 5. Off-track surface zones (e.g. `SurfaceType::DeepSand` traps, runoff areas):
    ///    Located underneath the track ribbon, only affecting the vehicle when running off track.
    /// 6. Grandstand concrete aprons.
    /// 7. Default off-track terrain (`SurfaceType::Grass`, `SurfaceType::DeepSand`).
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

        // 3. Project onto spline (if waypoints exist):
        let proj = (self.spline.waypoints.len() >= 2).then(|| self.spline.project_point(point));
        if let Some(proj) = &proj {
            if proj.is_on_track {
                return proj.base_surface;
            }
            // Check curbs
            if proj.is_on_curb {
                return SurfaceType::Curb;
            }
        }

        // 3b. Network branch road and junction areas come before the main line's run-off, because
        // a branch (e.g. a joker lap) can run inside that run-off corridor.
        if let Some(ref net) = self.network {
            if let Some(surf) = net.sample_surface(point) {
                return surf;
            }
        }

        // 3c. Pit lane road ribbon, pit boxes, and paved junction areas
        if let Some(surf) = self.sample_pit_lane_surface(point) {
            return surf;
        }

        if let Some(proj) = proj {
            // 4. Segment runoff corridor (off-track terrain between track/curb edge and boundary wall):
            let half_w = proj.track_width * 0.5;
            if proj.lateral_offset < -half_w {
                let left_ro = proj.left_runoff_surface.or_else(|| self.default_runoff_surface());
                if let Some(runoff) = left_ro {
                    let limit = half_w + proj.left_wall_distance.unwrap_or_else(|| self.effective_barrier_offset());
                    if -proj.lateral_offset <= limit {
                        return runoff;
                    }
                }
            } else if proj.lateral_offset > half_w {
                let right_ro = proj.right_runoff_surface.or_else(|| self.default_runoff_surface());
                if let Some(runoff) = right_ro {
                    let limit = half_w + proj.right_wall_distance.unwrap_or_else(|| self.effective_barrier_offset());
                    if proj.lateral_offset <= limit {
                        return runoff;
                    }
                }
            }
        }

        // 5. Check Arena / Hybrid floor surface inside boundary hull:
        match &self.kind {
            TrackKind::Arena { boundary_hull, floor_surface, .. } => {
                if point_in_polygon(point, boundary_hull) {
                    return *floor_surface;
                }
            }
            TrackKind::Hybrid { boundary_hull, floor_surface, .. } => {
                if point_in_polygon(point, boundary_hull) {
                    return *floor_surface;
                }
            }
            TrackKind::Circuit => {}
        }

        // 6. Check explicit off-track surface zones (e.g. hand-placed sand traps, asphalt runoffs):
        for zone in &self.geometry.surface_zones {
            if !zone.is_above_track() && zone.contains(point) {
                return zone.surface;
            }
        }

        // 7. Check grandstand concrete aprons
        for grandstand in &self.geometry.grandstands {
            if grandstand.contains(point) {
                return SurfaceType::Concrete;
            }
        }

        // 8. Default terrain
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

    /// Samples surface types underneath all 4 wheels of a car with a track progress distance hint.
    #[inline]
    pub fn sample_car_surfaces_with_hint(&self, car: &Car, hint_dist: f32) -> [SurfaceType; 4] {
        let wheel_positions = car.wheel_positions_world();
        [
            self.sample_surface_near(wheel_positions[0], hint_dist),
            self.sample_surface_near(wheel_positions[1], hint_dist),
            self.sample_surface_near(wheel_positions[2], hint_dist),
            self.sample_surface_near(wheel_positions[3], hint_dist),
        ]
    }

    /// Samples the surface at `point` with localized spline projection near `hint_dist`.
    pub fn sample_surface_near(&self, point: Vec2, hint_dist: f32) -> SurfaceType {
        // 1. Check jump ramps (elevated platforms)
        for ramp in &self.geometry.jump_ramps {
            if ramp.contains(point) {
                return ramp.surface;
            }
        }

        // 2. Check above-track surface zones
        for zone in &self.geometry.surface_zones {
            if zone.is_above_track() && zone.contains(point) {
                return zone.surface;
            }
        }

        // 3. Localized spline projection near hint distance
        let proj = self.spline.project_point_continuity(point, hint_dist, 45.0);
        if proj.is_on_track {
            return proj.base_surface;
        }
        if proj.is_on_curb {
            return SurfaceType::Curb;
        }

        // 3b. Network branch road and junction areas, before the main line's run-off (see sample_surface)
        if let Some(ref net) = self.network {
            if let Some(surf) = net.sample_surface(point) {
                return surf;
            }
        }

        // 3c. Pit lane road ribbon, pit boxes, and paved junction areas
        if let Some(surf) = self.sample_pit_lane_surface(point) {
            return surf;
        }

        // 4. Segment runoff corridor check
        let half_w = proj.track_width * 0.5;
        if proj.lateral_offset < -half_w {
            let left_ro = proj.left_runoff_surface.or_else(|| self.default_runoff_surface());
            if let Some(runoff) = left_ro {
                let limit = half_w + proj.left_wall_distance.unwrap_or_else(|| self.effective_barrier_offset());
                if -proj.lateral_offset <= limit {
                    return runoff;
                }
            }
        } else if proj.lateral_offset > half_w {
            let right_ro = proj.right_runoff_surface.or_else(|| self.default_runoff_surface());
            if let Some(runoff) = right_ro {
                let limit = half_w + proj.right_wall_distance.unwrap_or_else(|| self.effective_barrier_offset());
                if proj.lateral_offset <= limit {
                    return runoff;
                }
            }
        }

        // 5. Check Arena / Hybrid floor surface inside boundary hull:
        match &self.kind {
            TrackKind::Arena { boundary_hull, floor_surface, .. } => {
                if point_in_polygon(point, boundary_hull) {
                    return *floor_surface;
                }
            }
            TrackKind::Hybrid { boundary_hull, floor_surface, .. } => {
                if point_in_polygon(point, boundary_hull) {
                    return *floor_surface;
                }
            }
            TrackKind::Circuit => {}
        }

        // 6. Check explicit off-track surface zones
        for zone in &self.geometry.surface_zones {
            if !zone.is_above_track() && zone.contains(point) {
                return zone.surface;
            }
        }

        // 7. Check grandstand concrete aprons
        for grandstand in &self.geometry.grandstands {
            if grandstand.contains(point) {
                return SurfaceType::Concrete;
            }
        }

        // 8. Default terrain
        self.default_surface
    }

    /// Samples surface type for pit lane road ribbon, pit boxes, and paved junction areas.
    pub fn sample_pit_lane_surface(&self, point: Vec2) -> Option<SurfaceType> {
        let Some(ref lane) = self.pit_lane else {
            if let Some(ref shape) = self.pit_box_area {
                if shape.contains(point) {
                    return Some(SurfaceType::Asphalt);
                }
            }
            return None;
        };

        let fallback_storage;
        let junctions_ref = match &self.pit_lane_junctions {
            Some(j) => j,
            None => {
                fallback_storage = self.compute_pit_lane_junctions();
                fallback_storage.as_ref()?
            }
        };

        // Quick AABB rejection: if outside the pit lane bounding box, return immediately.
        if point.x < junctions_ref.bounds_min.x || point.x > junctions_ref.bounds_max.x
            || point.y < junctions_ref.bounds_min.y || point.y > junctions_ref.bounds_max.y
        {
            if let Some(ref shape) = self.pit_box_area {
                if shape.contains(point) {
                    return Some(SurfaceType::Asphalt);
                }
            }
            return None;
        }

        // 1. Pit lane drivable road ribbon
        if lane.spline.samples.len() >= 2 {
            let proj = lane.spline.project_point(point);
            if proj.is_on_track {
                return Some(proj.base_surface);
            }
            if proj.is_on_curb {
                return Some(SurfaceType::Curb);
            }
        }

        // 2. Pit service box stalls
        for pit_box in &lane.pit_boxes {
            if pit_box.contains_point(point) {
                return Some(SurfaceType::Asphalt);
            }
        }

        // 3. Paved entrance throat and exit merge junction areas
        for q in &junctions_ref.entrance_quads {
            if point_in_quad_2d(point, q[0], q[1], q[2], q[3]) {
                return Some(SurfaceType::Asphalt);
            }
        }
        if junctions_ref.has_gore
            && (point_in_triangle_2d(point, junctions_ref.p_apex, junctions_ref.track_edge_apex, junctions_ref.pit_inner_apex)
                || point_in_quad_2d(point, junctions_ref.te_start, junctions_ref.track_edge_apex, junctions_ref.pit_inner_apex, junctions_ref.pe_start))
        {
            return Some(SurfaceType::Asphalt);
        }
        for eq in &junctions_ref.exit_quads {
            if point_in_quad_2d(point, eq.quad[0], eq.quad[1], eq.quad[2], eq.quad[3]) {
                return Some(SurfaceType::Asphalt);
            }
        }

        if let Some(ref shape) = self.pit_box_area {
            if shape.contains(point) {
                return Some(SurfaceType::Asphalt);
            }
        }

        None
    }

    /// Recomputes cached pit lane junction geometry (entrance throat, gore markings, exit merge taper, AABB).
    pub fn recompute_pit_lane_junctions(&mut self) {
        self.pit_lane_junctions = self.compute_pit_lane_junctions();
    }

    /// Computes pit lane entrance throat, gore triangle, exit merge quads, and spatial AABB.
    pub fn compute_pit_lane_junctions(&self) -> Option<PitLaneJunctionData> {
        let lane = self.pit_lane.as_ref()?;
        // Spec 101: a compiled layout gives its junction markings directly; no search.
        if let Some(compiled) = self.pit_lane_layout.as_ref().and_then(|l| l.compile(self).ok()) {
            return Some(compiled.junctions);
        }
        let n_pit = lane.spline.samples.len();
        if n_pit < 4 || self.spline.samples.len() < 4 {
            let mut min = Vec2::splat(f32::INFINITY);
            let mut max = Vec2::splat(f32::NEG_INFINITY);
            for s in &lane.spline.samples {
                let r = lane.road_width * 1.5;
                min = min.min(s.point - Vec2::splat(r));
                max = max.max(s.point + Vec2::splat(r));
            }
            for b in &lane.pit_boxes {
                let r = b.stop_radius + 5.0;
                min = min.min(b.position - Vec2::splat(r));
                max = max.max(b.position + Vec2::splat(r));
            }
            if min.x.is_finite() {
                return Some(PitLaneJunctionData {
                    bounds_min: min,
                    bounds_max: max,
                    ..Default::default()
                });
            }
            return None;
        }

        let pit_hw = lane.road_width * 0.5;

        // Determine stable pit_side from midpoint
        let mid_idx = n_pit / 2;
        let mid_sample = &lane.spline.samples[mid_idx];
        let mid_proj = self.spline.project_point(mid_sample.point);
        let mid_to_pit = mid_sample.point - mid_proj.closest_point;
        let pit_side = if mid_to_pit.dot(mid_proj.normal) >= 0.0 { 1.0f32 } else { -1.0f32 };

        // 1. Entrance throat: between sample 0 and gore apex
        let mut apex_idx = None;
        for (i, s) in lane.spline.samples.iter().enumerate().take(n_pit / 2) {
            let proj = self.spline.project_point(s.point);
            let track_edge = proj.closest_point + proj.normal * (pit_side * proj.track_width * 0.5);
            let pit_inner = s.point - proj.normal * (pit_side * pit_hw);
            let gap = (pit_inner - track_edge).dot(proj.normal * pit_side);
            if gap >= 1.2 {
                apex_idx = Some(i);
                break;
            }
        }

        let mut entrance_quads = Vec::new();
        let mut has_gore = false;
        let mut p_apex = Vec2::ZERO;
        let mut track_edge_apex = Vec2::ZERO;
        let mut pit_inner_apex = Vec2::ZERO;
        let mut te_start = Vec2::ZERO;
        let mut pe_start = Vec2::ZERO;
        let mut chevrons = Vec::new();

        if let Some(apex_idx) = apex_idx {
            has_gore = true;
            for i in 0..apex_idx {
                let s0 = &lane.spline.samples[i];
                let s1 = &lane.spline.samples[i + 1];
                let p0 = self.spline.project_point(s0.point);
                let p1 = self.spline.project_point(s1.point);
                let te0 = p0.closest_point + p0.normal * (pit_side * p0.track_width * 0.5);
                let te1 = p1.closest_point + p1.normal * (pit_side * p1.track_width * 0.5);
                let pe0 = s0.point - p0.normal * (pit_side * pit_hw);
                let pe1 = s1.point - p1.normal * (pit_side * pit_hw);
                entrance_quads.push([te0, te1, pe1, pe0]);
            }

            let s_apex = &lane.spline.samples[apex_idx];
            let proj_apex = self.spline.project_point(s_apex.point);
            track_edge_apex = proj_apex.closest_point + proj_apex.normal * (pit_side * proj_apex.track_width * 0.5);
            pit_inner_apex = s_apex.point - proj_apex.normal * (pit_side * pit_hw);
            p_apex = (track_edge_apex + pit_inner_apex) * 0.5;

            let s_start = &lane.spline.samples[0];
            let p_start = self.spline.project_point(s_start.point);
            te_start = p_start.closest_point + p_start.normal * (pit_side * p_start.track_width * 0.5);
            pe_start = s_start.point - p_start.normal * (pit_side * pit_hw);

            let v_track = (track_edge_apex - te_start).normalize_or_zero();
            let v_pit = (pit_inner_apex - pe_start).normalize_or_zero();
            let gore_len = (p_apex - te_start).length();
            if gore_len > 8.0 {
                let num_chevrons = ((gore_len - 4.0) / 3.0).floor() as usize;
                let bisect = -(v_track + v_pit).normalize_or_zero();
                for step in 1..=num_chevrons {
                    let d = step as f32 * 3.0;
                    if d >= gore_len - 2.0 { break; }
                    let t_frac = d / gore_len;
                    let pt_t = te_start.lerp(track_edge_apex, t_frac);
                    let pt_p = pe_start.lerp(pit_inner_apex, t_frac);
                    let chevron_apex = (pt_t + pt_p) * 0.5 + bisect * 1.2;
                    chevrons.push(PitLaneChevron {
                        apex: chevron_apex,
                        pt_track: pt_t,
                        pt_pit: pt_p,
                    });
                }
            }
        }

        // 2. Exit merge taper
        let mut merge_start_idx = None;
        for i in (n_pit / 2..n_pit).rev() {
            let s = &lane.spline.samples[i];
            let proj = self.spline.project_point(s.point);
            let track_edge = proj.closest_point + proj.normal * (pit_side * proj.track_width * 0.5);
            let pit_inner = s.point - proj.normal * (pit_side * pit_hw);
            let gap = (pit_inner - track_edge).dot(proj.normal * pit_side);
            if gap >= 1.2 {
                merge_start_idx = Some(i);
                break;
            }
        }

        let mut exit_quads = Vec::new();
        if let Some(merge_idx) = merge_start_idx {
            for i in merge_idx..n_pit - 1 {
                let s0 = &lane.spline.samples[i];
                let s1 = &lane.spline.samples[i + 1];
                let p0 = self.spline.project_point(s0.point);
                let p1 = self.spline.project_point(s1.point);
                let te0 = p0.closest_point + p0.normal * (pit_side * p0.track_width * 0.5);
                let te1 = p1.closest_point + p1.normal * (pit_side * p1.track_width * 0.5);
                let pe0 = s0.point - p0.normal * (pit_side * pit_hw);
                let pe1 = s1.point - p1.normal * (pit_side * pit_hw);
                exit_quads.push(PitLaneExitQuad {
                    quad: [te0, te1, pe1, pe0],
                    has_dashed_line: i % 2 == 0,
                    line_start: pe0,
                    line_end: pe1,
                });
            }
        }

        // 3. Compute AABB bounding box
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        let mut expand_point = |p: Vec2, r: f32| {
            min = min.min(p - Vec2::splat(r));
            max = max.max(p + Vec2::splat(r));
        };

        for s in &lane.spline.samples {
            expand_point(s.point, lane.road_width * 2.0);
        }
        for b in &lane.pit_boxes {
            expand_point(b.position, b.stop_radius + 6.0);
        }
        for q in &entrance_quads {
            for pt in q {
                expand_point(*pt, 6.0);
            }
        }
        if has_gore {
            expand_point(p_apex, 6.0);
            expand_point(track_edge_apex, 6.0);
            expand_point(pit_inner_apex, 6.0);
            expand_point(te_start, 6.0);
            expand_point(pe_start, 6.0);
        }
        for eq in &exit_quads {
            for pt in &eq.quad {
                expand_point(*pt, 6.0);
            }
        }

        Some(PitLaneJunctionData {
            bounds_min: min,
            bounds_max: max,
            entrance_quads,
            has_gore,
            p_apex,
            track_edge_apex,
            pit_inner_apex,
            te_start,
            pe_start,
            chevrons,
            exit_quads,
        })
    }

    /// Projects a 2D world position onto the track centerline or any of its network segments.
    pub fn project_point(&self, point: Vec2) -> SplineProjection {
        let proj = self.spline.project_point(point);
        if proj.is_on_track || proj.is_on_curb {
            return proj;
        }

        if let Some(ref net) = self.network {
            let mut best_proj = proj;
            let mut best_dist = proj.distance_to_spline;
            for seg in &net.segments {
                if seg.samples.len() >= 2 {
                    let seg_proj = seg.project_point(point);
                    if seg_proj.is_on_track || seg_proj.is_on_curb {
                        return seg_proj;
                    }
                    if seg_proj.distance_to_spline < best_dist {
                        best_dist = seg_proj.distance_to_spline;
                        best_proj = seg_proj;
                    }
                }
            }
            return best_proj;
        }

        proj
    }

    /// Projects a 2D world position onto the track centerline with continuity constraint around `hint_dist`,
    /// automatically falling back to check any active network segments if off the primary spline.
    pub fn project_point_near(&self, point: Vec2, hint_dist: f32) -> SplineProjection {
        let proj = self.spline.project_point_continuity(point, hint_dist, 45.0);
        if proj.is_on_track || proj.is_on_curb {
            return proj;
        }

        if let Some(ref net) = self.network {
            let mut best_proj = proj;
            let mut best_dist = proj.distance_to_spline;
            for seg in &net.segments {
                if seg.samples.len() >= 2 {
                    let seg_proj = seg.project_point(point);
                    if seg_proj.is_on_track || seg_proj.is_on_curb {
                        return seg_proj;
                    }
                    if seg_proj.distance_to_spline < best_dist {
                        best_dist = seg_proj.distance_to_spline;
                        best_proj = seg_proj;
                    }
                }
            }
            return best_proj;
        }

        proj
    }

    /// Prunes or removes any wall barriers in `inner_walls` and `outer_walls` that penetrate
    /// or cross within the drivable road ribbon of any segment in `self.network`. Against the
    /// branch segments (not on the default layout) the check ignores elevation: on Spa RX the
    /// joker data sits ~4 m below the main road it overlaps, and a main wall blocked the joker.
    pub fn trim_walls_for_network(&mut self) {
        let Some(net) = &self.network else { return; };
        if net.segments.is_empty() { return; }
        let branches = branch_segments(net);

        let keep = |w: &WallBarrier| {
            wall_clear_of_roads(&net.segments, w, 2.0, -0.2) && wall_clear_of_roads(branches.iter().copied(), w, f32::INFINITY, -0.2)
        };
        // A wall that runs onto a road loses only the part on it. A merged straight wall used to go whole: on
        // rx_canyon_flyer that left an 11 m gap beside the joker split, and a bot slid out through it behind the
        // joker wall and stayed there (tdrace-0joa, tdrace-74s5).
        let trim = |walls: &[WallBarrier]| {
            let mut out = Vec::with_capacity(walls.len());
            for w in walls {
                if keep(w) {
                    out.push(*w);
                    continue;
                }
                let n = (w.segment.length() / WALL_TRIM_STEP_M).ceil().max(1.0) as usize;
                let points: Vec<Vec2> = (0..=n).map(|k| w.segment.start.lerp(w.segment.end, k as f32 / n as f32)).collect();
                let pieces = points
                    .windows(2)
                    .map(|p| WallBarrier { segment: LineSegment::new(p[0], p[1]), ..*w })
                    .filter(|piece| keep(piece))
                    .collect();
                out.extend(merge_collinear_walls(pieces));
            }
            out
        };
        self.geometry.inner_walls = trim(&self.geometry.inner_walls);
        self.geometry.outer_walls = trim(&self.geometry.outer_walls);
    }

    /// Builds `geometry.network_walls` along the network segments that are not on the default
    /// layout (the Rallycross joker branch). The walls use the gap that the main walls keep
    /// from the main road around the branch, and the track's most common barrier type.
    /// Every piece that comes within 0.3 m of a network road, or nearer the main road (with its
    /// curbs) than the main walls stand, or crosses a network road or a main wall, or runs along
    /// a main wall, is dropped, so the split and merge throats stay open. The check ignores elevation: some joker data overlaps the main road
    /// at a different height, and a wall there would block the main road.
    pub fn generate_network_walls(&mut self) {
        self.geometry.network_walls.clear();
        let Some(net) = &self.network else { return; };
        let barrier_type = self.dominant_barrier_type().unwrap_or(BarrierType::TireWall);

        // The main spline is smoothed across the segment seams, so check its ribbon (with curbs)
        // as well, out to `gap`: where the joker runs on or beside the main road, a joker wall
        // inside the main road's own wall gap stood ~1 m off its edge, and essay_rx bots on the
        // main route hit its end. A piece that crosses a main wall is dropped too, so the walls
        // never pinch. So is a piece within 0.5 m of a main wall along its length: it only doubles
        // that wall, and once the pieces are joined the line zigzags across it (holjes_rx).
        let main_walls = || self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls);
        let clear_of_main = |w: &WallBarrier, gap: f32| {
            let off_road = [w.segment.start, w.segment.end, (w.segment.start + w.segment.end) * 0.5].into_iter().all(|p| {
                let proj = self.spline.project_point(p);
                let curb_extra = if proj.left_curb || proj.right_curb { 1.35 } else { 0.0 };
                proj.distance_to_spline >= proj.track_width * 0.5 + curb_extra + gap
            });
            let doubles_main_wall = [w.segment.start, w.segment.end, (w.segment.start + w.segment.end) * 0.5]
                .into_iter()
                .all(|p| main_walls().any(|m| m.segment.distance_to_point(p) < 0.5));
            off_road && !doubles_main_wall && !main_walls().any(|m| m.segment.intersect_segment(&w.segment).is_some())
        };

        let mut walls = Vec::new();
        for seg in branch_segments(net) {
            if seg.samples.len() < 2 { continue; }
            // The track-wide offset comes from the first main samples, which can sit on a wide
            // run-off (Riga RX: 14.9 m). Measure the main walls around this branch instead.
            let (min, max) = seg.samples.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), s| {
                (lo.min(s.point), hi.max(s.point))
            });
            let barrier_offset = self
                .local_barrier_offset(min - Vec2::splat(20.0), max + Vec2::splat(20.0))
                .unwrap_or_else(|| self.effective_barrier_offset());
            let mut spline = seg.to_spline();
            // Segment samples flag every point above 0.6 m as a bridge, which would pull the
            // walls in to the deck edge. A branch is a ground road, so use the normal offset.
            for s in &mut spline.samples {
                s.is_bridge = false;
            }
            let (left, right, _, _) = generate_walls_from_spline_raw(&spline, barrier_offset, barrier_type);
            // 0.5 m inside the main wall line, so a joker wall that runs just outside it is kept, and at most
            // 3.5 m: riga_rx's main walls stand ~10 m off on its run-off, and its side-by-side joker lost its walls.
            let main_gap = (barrier_offset - 0.5).clamp(0.3, 3.5);
            for side in [left, right] {
                let kept: Vec<WallBarrier> = side
                    .into_iter()
                    .filter(|w| wall_clear_of_roads(&net.segments, w, f32::INFINITY, 0.3) && clear_of_main(w, main_gap))
                    .collect();
                for w in merge_collinear_walls(kept) {
                    let shares_endpoint = |a: &WallBarrier, b: &WallBarrier| {
                        (a.segment.start - b.segment.start).length_squared() < 0.01
                            || (a.segment.start - b.segment.end).length_squared() < 0.01
                            || (a.segment.end - b.segment.start).length_squared() < 0.01
                            || (a.segment.end - b.segment.end).length_squared() < 0.01
                    };
                    if !walls.iter().any(|other: &WallBarrier| {
                        !shares_endpoint(&w, other) && other.segment.intersect_segment(&w.segment).is_some()
                    }) {
                        walls.push(w);
                    }
                }
            }
        }
        self.geometry.network_walls = walls;
    }

    /// Median gap between the main road edge and the nearest main wall, over the main spline
    /// samples inside the box `min`..`max`. Gaps of 10 m or more are not walls of that road and
    /// are skipped. `None` when no sample qualifies.
    fn local_barrier_offset(&self, min: Vec2, max: Vec2) -> Option<f32> {
        let mut dists = Vec::new();
        for s in self.spline.samples.iter().filter(|s| s.point.cmpge(min).all() && s.point.cmple(max).all()) {
            let hw = s.width * 0.5;
            for edge in [s.point + s.normal * hw, s.point - s.normal * hw] {
                let d = self
                    .geometry
                    .inner_walls
                    .iter()
                    .chain(&self.geometry.outer_walls)
                    .map(|w| w.segment.distance_to_point(edge))
                    .fold(f32::MAX, f32::min);
                if d < 10.0 {
                    dists.push(d);
                }
            }
        }
        if dists.is_empty() {
            return None;
        }
        dists.sort_by(f32::total_cmp);
        Some(dists[dists.len() / 2])
    }
}

/// Length of the pieces a wall is cut into where it runs onto a network road (m).
const WALL_TRIM_STEP_M: f32 = 1.0;

/// Network segments that are not on the default layout (the Rallycross joker branch).
fn branch_segments(net: &TrackNetwork) -> Vec<&RoadSegment> {
    let Some(main) = net.active_or_default_layout(None) else { return Vec::new(); };
    net.segments.iter().filter(|s| !main.segment_sequence.contains(&s.id)).collect()
}

/// True when `wall` stays more than `edge_margin` outside the drivable ribbon of every segment
/// in `segments` (a negative margin lets it sit that far inside) and does not cross any segment
/// centerline. Roads more than `max_elevation_gap` above or below the wall are ignored.
fn wall_clear_of_roads<'a>(
    segments: impl IntoIterator<Item = &'a RoadSegment>,
    wall: &WallBarrier,
    max_elevation_gap: f32,
    edge_margin: f32,
) -> bool {
    let p0 = wall.segment.start;
    let p1 = wall.segment.end;
    let p_mid = (p0 + p1) * 0.5;

    for seg in segments {
        if seg.samples.len() < 2 { continue; }

        // Check endpoints and midpoint against this segment
        for pt in [p0, p1, p_mid] {
            let proj = seg.project_point(pt);
            if (wall.elevation - proj.elevation).abs() < max_elevation_gap {
                let half_w = proj.track_width * 0.5;
                if proj.lateral_offset.abs() < (half_w + edge_margin) {
                    return false; // Point inside drivable road surface
                }
            }
        }

        // Check centerline intersections
        for i in 0..seg.samples.len() - 1 {
            let s0 = &seg.samples[i];
            let s1 = &seg.samples[i + 1];
            let seg_elev = (s0.elevation + s1.elevation) * 0.5;
            if (wall.elevation - seg_elev).abs() < max_elevation_gap {
                let center_seg = LineSegment::new(s0.point, s1.point);
                if wall.segment.intersect_segment(&center_seg).is_some() {
                    return false; // Intersects road centerline
                }
            }
        }
    }
    true
}

impl wheelbase::SurfaceSampler for Track {
    #[inline]
    fn sample_surface(&self, world_pos: Vec2) -> wheelbase::SurfaceProperties {
        let surface_type = self.sample_surface(world_pos);
        let proj = self.project_point(world_pos);
        let mut props = wheelbase::SurfaceProperties::from_type(surface_type);
        props.elevation = proj.elevation;
        props.bank_angle = proj.bank_angle;
        props.grade_slope = proj.grade_slope;
        props.vertical_curvature = proj.vertical_curvature;
        props.track_right = Vec2::new(proj.tangent.y, -proj.tangent.x);
        props.track_forward = proj.tangent;
        props
    }
}

impl Track {
    /// Tests if a car's center is currently inside the pit box servicing zone.
    pub fn is_in_pit_box<B: crate::body::Body2D>(&self, car: &B) -> bool {
        if let Some(lane) = &self.pit_lane {
            if lane.pit_boxes.iter().any(|b| b.contains_point(car.position())) {
                return true;
            }
        }
        if let Some(pit_shape) = &self.pit_box_area {
            pit_shape.contains(car.position())
        } else {
            false
        }
    }

    /// Resolves the default runoff corridor surface type for the track based on discipline and environment:
    /// - Snow or icy circuits -> `SurfaceType::DeepSnow`
    /// - Sandy circuits -> `SurfaceType::DeepSand`
    /// - Pure dirt or mud circuits -> `SurfaceType::Dirt`
    /// - Kart circuits -> `SurfaceType::Concrete`
    /// - GT and Rallycross circuits -> `SurfaceType::DeepGravel` (gravel traps, Spec 099)
    pub fn default_runoff_surface(&self) -> Option<SurfaceType> {
        let name_lower = self.name.to_lowercase();

        // 1. Snow or icy circuits
        if self.default_surface.is_snow()
            || self.default_surface.is_ice()
            || (!self.spline.waypoints.is_empty()
                && self.spline.waypoints.iter().all(|wp| {
                    wp.surface.map_or(false, |s| s.is_snow() || s.is_ice())
                }))
            || name_lower.contains("snow")
            || name_lower.contains("ice")
            || name_lower.contains("arctic")
            || name_lower.contains("glacier")
            || name_lower.contains("frozen")
        {
            return Some(SurfaceType::DeepSnow);
        }

        // 2. Sandy circuits
        if self.default_surface.is_sand()
            || (!self.spline.waypoints.is_empty()
                && self.spline.waypoints.iter().all(|wp| wp.surface.map_or(false, |s| s.is_sand())))
            || name_lower.contains("sand")
            || name_lower.contains("dune")
            || name_lower.contains("sahara")
            || name_lower.contains("atacama")
        {
            return Some(SurfaceType::DeepSand);
        }

        // 3. Pure dirt or mud circuits
        if self.default_surface == SurfaceType::Dirt
            || self.default_surface.is_mud()
            || (!self.spline.waypoints.is_empty()
                && self.spline.waypoints.iter().all(|wp| {
                    wp.surface.map_or(false, |s| s == SurfaceType::Dirt || s.is_mud())
                }))
            || name_lower.contains("mud")
            || name_lower.contains("dirt")
            || name_lower.contains("clay")
        {
            return Some(SurfaceType::Dirt);
        }

        // 4. Kart circuits
        if self.car_category == CarCategory::Kart
            || self.belongs_to_module("kart")
            || name_lower.contains("kart")
        {
            return Some(SurfaceType::Concrete);
        }

        // 5. GT and Rallycross circuits
        if self.car_category == CarCategory::Gt
            || self.car_category == CarCategory::Rally
            || self.belongs_to_module("gt")
            || self.belongs_to_module("rally")
            || self.belongs_to_module("classic")
            || name_lower.contains(" rx")
            || name_lower.contains("rallycross")
            || name_lower.contains("grand prix")
        {
            return Some(SurfaceType::DeepGravel);
        }

        None
    }

    /// Returns the track-level barrier offset in meters, inferring from wall geometry or car category / module.
    /// Most common wall type on the track, or `None` when it has no walls.
    pub fn dominant_barrier_type(&self) -> Option<BarrierType> {
        let mut counts: Vec<(BarrierType, usize)> = Vec::new();
        for wall in self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls) {
            match counts.iter_mut().find(|(t, _)| *t == wall.barrier_type) {
                Some((_, n)) => *n += 1,
                None => counts.push((wall.barrier_type, 1)),
            }
        }
        counts.into_iter().max_by_key(|(_, n)| *n).map(|(t, _)| t)
    }

    /// Returns the track-level barrier offset in meters, using the cached value if available.
    pub fn effective_barrier_offset(&self) -> f32 {
        if let Some(cached) = self.cached_barrier_offset {
            return cached;
        }
        self.compute_effective_barrier_offset()
    }

    /// Computes the track-level barrier offset from wall geometry or category fallback.
    pub fn compute_effective_barrier_offset(&self) -> f32 {
        // Infer from inner/outer wall geometry if available
        let walls = if !self.geometry.inner_walls.is_empty() {
            &self.geometry.inner_walls
        } else {
            &self.geometry.outer_walls
        };

        if !walls.is_empty() && !self.spline.samples.is_empty() {
            let sample_count = self.spline.samples.len().min(10);
            let mut dists = Vec::with_capacity(sample_count);
            for s in &self.spline.samples[..sample_count] {
                let hw = s.width * 0.5;
                let edge = s.point + s.normal * hw;
                let mut min_d = f32::MAX;
                for w in walls {
                    let d = w.segment.distance_to_point(edge);
                    if d < min_d {
                        min_d = d;
                    }
                }
                if min_d < 30.0 {
                    dists.push(min_d);
                }
            }
            if !dists.is_empty() {
                dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                return dists[dists.len() / 2];
            }
        }

        let name_lower = self.name.to_lowercase();
        if self.car_category == CarCategory::Kart
            || self.belongs_to_module("kart")
            || name_lower.contains("kart")
        {
            2.0
        } else if self.belongs_to_module("nascar") {
            1.8
        } else if self.belongs_to_module("extreme_offroad") {
            6.0
        } else {
            3.5
        }
    }

    /// Recomputes and caches the effective barrier offset.
    pub fn recompute_barrier_offset(&mut self) {
        self.cached_barrier_offset = Some(self.compute_effective_barrier_offset());
    }

    /// Prunes or removes any wall barriers in `inner_walls` and `outer_walls` that penetrate
    /// or cross within the drivable road ribbon of `self.pit_lane`.
    pub fn trim_walls_for_pit_lane(&mut self) {
        let Some(ref lane) = self.pit_lane else { return; };
        if lane.spline.samples.len() < 2 { return; }

        // Spec 101: the pit perimeter replaces the pit-side main wall between the pit lane anchors, so cut it there
        // by arc length; a search-based trim drops whole merged wall pieces and leaves holes.
        if let Some(span) = pit_kit::perimeter_span(self) {
            let side_walls = if span.side == junction_kit::Side::Left { &self.geometry.inner_walls } else { &self.geometry.outer_walls };
            let cut: Vec<WallBarrier> = side_walls
                .iter()
                .flat_map(|w| {
                    pit_kit::wall_pieces_outside_span(self, &span, &w.segment)
                        .into_iter()
                        .map(move |segment| WallBarrier { segment, ..w.clone() })
                })
                .collect();
            if span.side == junction_kit::Side::Left {
                self.geometry.inner_walls = cut;
            } else {
                self.geometry.outer_walls = cut;
            }
        }
        let Some(ref lane) = self.pit_lane else { return; };

        let should_keep_wall = |wall: &WallBarrier| -> bool {
            let p0 = wall.segment.start;
            let p1 = wall.segment.end;
            let p_mid = (p0 + p1) * 0.5;
            let p_q1 = (p0 * 3.0 + p1) * 0.25;
            let p_q3 = (p0 + p1 * 3.0) * 0.25;

            for pt in [p0, p_q1, p_mid, p_q3, p1] {
                if pit_kit::beyond_lane_end(lane, pt) {
                    continue;
                }
                let proj = lane.spline.project_point(pt);
                if (wall.elevation - proj.elevation).abs() < 2.0 {
                    let half_w = proj.track_width * 0.5;
                    if proj.lateral_offset.abs() < (half_w - 0.2) {
                        return false;
                    }
                }
            }

            for i in 0..lane.spline.samples.len() - 1 {
                let s0 = &lane.spline.samples[i];
                let s1 = &lane.spline.samples[i + 1];
                let seg_elev = (s0.elevation + s1.elevation) * 0.5;
                if (wall.elevation - seg_elev).abs() < 2.0 {
                    let center_seg = LineSegment::new(s0.point, s1.point);
                    if wall.segment.intersect_segment(&center_seg).is_some() {
                        return false;
                    }
                }
            }
            true
        };

        self.geometry.inner_walls.retain(should_keep_wall);
        self.geometry.outer_walls.retain(should_keep_wall);
    }

    /// Generates physical wall barriers for `self.pit_lane`:
    /// - A solid dividing concrete barrier separating the main straight from the pit road along parallel sections.
    /// - An impact attenuator nose at the entrance gore apex.
    /// - Outer perimeter retaining wall along the team garage frontage.
    pub fn generate_pit_lane_walls(&mut self) {
        let Some(ref lane) = self.pit_lane else { return; };
        if lane.spline.samples.len() < 4 || self.spline.samples.len() < 4 { return; }
        // Spec 101: a compiled layout fixes the dividing wall ends; every pit lane gets an unbroken outer perimeter
        // in place of the outer wall pieces.
        let compiled = self.pit_lane_layout.as_ref().and_then(|l| l.compile(self).ok());
        let (perimeter, perimeter_type) = match pit_kit::perimeter_span(self) {
            Some(span) => pit_kit::perimeter_chain(self, lane, &span),
            None => (Vec::new(), BarrierType::Concrete),
        };

        let pit_w = lane.road_width;
        let pit_half_w = pit_w * 0.5;
        let mut dividing_wall_pts: Vec<Vec2> = Vec::new();
        let mut outer_wall_pts: Vec<Vec2> = Vec::new();

        // Determine which side of the track the pit lane is on using the midpoint of the pit spline
        let mid_idx = lane.spline.samples.len() / 2;
        let mid_sample = &lane.spline.samples[mid_idx];
        let mid_proj = self.spline.project_point(mid_sample.point);
        let mid_to_pit = mid_sample.point - mid_proj.closest_point;
        let pit_side = if mid_to_pit.dot(mid_proj.normal) >= 0.0 { 1.0f32 } else { -1.0f32 };

        for s in &lane.spline.samples {
            let proj = self.spline.project_point(s.point);
            if (s.elevation - proj.elevation).abs() > 2.0 {
                continue;
            }
            let track_half_w = proj.track_width * 0.5;

            // Coordinates of the two adjacent edges
            let track_edge = proj.closest_point + proj.normal * (pit_side * track_half_w);
            let pit_inner_edge = s.point - proj.normal * (pit_side * pit_half_w);

            let gap_dist = (pit_inner_edge - track_edge).dot(proj.normal * pit_side);

            // The dividing pit wall sits where there is clearance between track and pit road
            if gap_dist >= 1.2 {
                let buf = (gap_dist * 0.5).min(1.0);
                let wall_pt = pit_inner_edge - proj.normal * (pit_side * buf);
                let check_main = self.spline.project_point(wall_pt);
                let check_pit = lane.spline.project_point(wall_pt);
                let curb_extra = if check_main.left_curb || check_main.right_curb { 1.4 } else { 0.0 };
                if check_main.distance_to_spline >= (check_main.track_width * 0.5 + curb_extra + 0.3)
                    && check_pit.lateral_offset.abs() >= (pit_half_w + 0.2)
                {
                    if dividing_wall_pts.is_empty() || dividing_wall_pts.last().unwrap().distance(wall_pt) >= 1.0 {
                        dividing_wall_pts.push(wall_pt);
                    }
                }
            }

            // The outer retaining wall sits on the outer flank along the parallel working lane
            if gap_dist >= 2.5 {
                let pit_normal = Vec2::new(-s.tangent.y, s.tangent.x);
                let outer_pt = s.point + pit_normal * (pit_side * (pit_half_w + 1.5));
                let check_outer_main = self.spline.project_point(outer_pt);
                let check_outer_pit = lane.spline.project_point(outer_pt);
                let outer_curb = if check_outer_main.left_curb || check_outer_main.right_curb { 1.4 } else { 0.0 };
                if check_outer_main.distance_to_spline >= (check_outer_main.track_width * 0.5 + outer_curb + 0.5)
                    && check_outer_pit.lateral_offset.abs() >= (pit_half_w + 0.5)
                {
                    if outer_wall_pts.is_empty() || outer_wall_pts.last().unwrap().distance(outer_pt) >= 1.0 {
                        outer_wall_pts.push(outer_pt);
                    }
                }
            }
        }

        if !perimeter.is_empty() {
            outer_wall_pts.clear();
        }
        if let Some(compiled) = &compiled {
            let s_of = |p: Vec2| lane.spline.project_point(p).progress_distance;
            let (s_start, s_end) = (s_of(compiled.divider_start), s_of(compiled.divider_end));
            dividing_wall_pts.retain(|&p| {
                let s = s_of(p);
                s > s_start + 0.5 && s < s_end - 0.5
            });
            dividing_wall_pts.insert(0, compiled.divider_start);
            dividing_wall_pts.push(compiled.divider_end);
        }

        if dividing_wall_pts.len() >= 2 {
            // Build continuous segments for the dividing pit wall
            for i in 0..dividing_wall_pts.len() - 1 {
                let p0 = dividing_wall_pts[i];
                let p1 = dividing_wall_pts[i + 1];
                let len = (p1 - p0).length();
                if len < 0.1 || len > 4.0 { continue; }
                let penetrates = [p0, p1, (p0 + p1) * 0.5].iter().any(|pt| {
                    let check_m = self.spline.project_point(*pt);
                    let curb_m = if check_m.left_curb || check_m.right_curb { 1.4 } else { 0.0 };
                    if check_m.distance_to_spline < (check_m.track_width * 0.5 + curb_m + 0.25) {
                        return true;
                    }
                    let check_p = lane.spline.project_point(*pt);
                    if check_p.lateral_offset.abs() < (pit_half_w - 0.25) {
                        return true;
                    }
                    false
                });
                if penetrates {
                    continue;
                }
                let seg_exists = self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls)
                    .any(|w| (w.segment.start.distance(p0) < 0.3 && w.segment.end.distance(p1) < 0.3)
                          || (w.segment.start.distance(p1) < 0.3 && w.segment.end.distance(p0) < 0.3));
                if seg_exists {
                    continue;
                }
                let wall = WallBarrier::new(p0, p1, BarrierType::Concrete);
                let intersects_existing = self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls)
                    .any(|w| w.segment.intersect_segment(&wall.segment).is_some());
                if !intersects_existing {
                    if pit_side > 0.0 {
                        self.geometry.inner_walls.push(wall);
                    } else {
                        self.geometry.outer_walls.push(wall);
                    }
                }
            }

            // Place an impact attenuator barrier at the entrance gore apex (first station)
            let p_apex = dividing_wall_pts[0];
            let dir_fwd = (dividing_wall_pts[1] - dividing_wall_pts[0]).normalize_or_zero();
            let dir_trans = Vec2::new(-dir_fwd.y, dir_fwd.x);
            let p_att_0 = p_apex - dir_trans * 0.8;
            let p_att_1 = p_apex + dir_trans * 0.8;
            let c_a0 = self.spline.project_point(p_att_0);
            let c_a1 = self.spline.project_point(p_att_1);
            let curb_a0 = if c_a0.left_curb || c_a0.right_curb { 1.4 } else { 0.0 };
            let curb_a1 = if c_a1.left_curb || c_a1.right_curb { 1.4 } else { 0.0 };
            let p_a0 = lane.spline.project_point(p_att_0);
            let p_a1 = lane.spline.project_point(p_att_1);
            let p_amid = lane.spline.project_point((p_att_0 + p_att_1) * 0.5);
            if c_a0.distance_to_spline >= (c_a0.track_width * 0.5 + curb_a0 + 0.3)
                && c_a1.distance_to_spline >= (c_a1.track_width * 0.5 + curb_a1 + 0.3)
                && p_a0.lateral_offset.abs() >= (pit_half_w - 0.25)
                && p_a1.lateral_offset.abs() >= (pit_half_w - 0.25)
                && p_amid.lateral_offset.abs() >= (pit_half_w - 0.25)
            {
                let att_exists = self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls)
                    .any(|w| w.barrier_type == BarrierType::TireWall && (w.segment.start.distance(p_att_0) < 1.0 || w.segment.midpoint().distance(p_apex) < 1.5));
                if !att_exists {
                    let attenuator = WallBarrier::new(p_att_0, p_att_1, BarrierType::TireWall);
                    let intersects_existing = self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls)
                        .any(|w| w.segment.intersect_segment(&attenuator.segment).is_some());
                    if !intersects_existing {
                        if pit_side > 0.0 {
                            self.geometry.inner_walls.push(attenuator);
                        } else {
                            self.geometry.outer_walls.push(attenuator);
                        }
                    }
                }
            }
        }

        if outer_wall_pts.len() >= 2 {
            for i in 0..outer_wall_pts.len() - 1 {
                let p0 = outer_wall_pts[i];
                let p1 = outer_wall_pts[i + 1];
                let len = (p1 - p0).length();
                if len < 0.1 || len > 4.0 { continue; }
                let penetrates = [p0, p1, (p0 + p1) * 0.5].iter().any(|pt| {
                    let check_m = self.spline.project_point(*pt);
                    let curb_m = if check_m.left_curb || check_m.right_curb { 1.4 } else { 0.0 };
                    if check_m.distance_to_spline < (check_m.track_width * 0.5 + curb_m + 0.25) {
                        return true;
                    }
                    let check_p = lane.spline.project_point(*pt);
                    if check_p.lateral_offset.abs() < (pit_half_w - 0.25) {
                        return true;
                    }
                    false
                });
                if penetrates {
                    continue;
                }
                let seg_exists = self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls)
                    .any(|w| (w.segment.start.distance(p0) < 0.3 && w.segment.end.distance(p1) < 0.3)
                          || (w.segment.start.distance(p1) < 0.3 && w.segment.end.distance(p0) < 0.3));
                if seg_exists {
                    continue;
                }
                let wall = WallBarrier::new(p0, p1, BarrierType::Concrete);
                let intersects_existing = self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls)
                    .any(|w| w.segment.intersect_segment(&wall.segment).is_some());
                if !intersects_existing {
                    if pit_side > 0.0 {
                        self.geometry.inner_walls.push(wall);
                    } else {
                        self.geometry.outer_walls.push(wall);
                    }
                }
            }
        }
        for pair in perimeter.windows(2) {
            let mut wall = WallBarrier::new(pair[0], pair[1], perimeter_type);
            wall.elevation = self.spline.project_point((pair[0] + pair[1]) * 0.5).elevation;
            let exists = self.geometry.inner_walls.iter().chain(&self.geometry.outer_walls).any(|w| {
                (w.segment.start.distance(pair[0]) < 0.05 && w.segment.end.distance(pair[1]) < 0.05)
                    || (w.segment.start.distance(pair[1]) < 0.05 && w.segment.end.distance(pair[0]) < 0.05)
            });
            if !exists {
                if pit_side > 0.0 {
                    self.geometry.inner_walls.push(wall);
                } else {
                    self.geometry.outer_walls.push(wall);
                }
            }
        }
        self.recompute_pit_lane_junctions();
    }

    /// Populates segment runoff surfaces (`left_runoff_surface` / `right_runoff_surface`)
    /// and wall distances (`left_wall_distance` / `right_wall_distance`) across all waypoints
    /// and spline samples where they have not been explicitly customized.
    pub fn apply_default_runoff_surfaces(&mut self) {
        let default_runoff = self.default_runoff_surface();
        let bo = self.effective_barrier_offset();
        self.cached_barrier_offset = Some(bo);

        for wp in &mut self.spline.waypoints {
            if wp.left_runoff_surface.is_none() {
                wp.left_runoff_surface = default_runoff;
            }
            if wp.right_runoff_surface.is_none() {
                wp.right_runoff_surface = default_runoff;
            }
        }

        let sample_offset = |s: &SplineSample| {
            let elev_factor = if s.is_bridge { (s.elevation / 3.0).clamp(0.0, 1.0) } else { 0.0 };
            let curb_extra = if s.left_curb || s.right_curb { 1.35 } else { 0.75 };
            let bridge_offset = curb_extra + 0.50;
            bo * (1.0 - elev_factor) + bridge_offset * elev_factor
        };
        let blended: Vec<(Option<f32>, Option<f32>)> = (0..self.spline.samples.len())
            .map(|i| {
                let sample_bo = sample_offset(&self.spline.samples[i]);
                (
                    self.spline.blended_wall_distance(i, true, sample_bo),
                    self.spline.blended_wall_distance(i, false, sample_bo),
                )
            })
            .collect();

        for (s, (left_d, right_d)) in self.spline.samples.iter_mut().zip(blended) {
            if s.left_runoff_surface.is_none() {
                s.left_runoff_surface = default_runoff;
            }
            if s.right_runoff_surface.is_none() {
                s.right_runoff_surface = default_runoff;
            }
            let sample_bo = sample_offset(s);

            // Runoff corridor width applies with or without a wall, matching `sample_surface`.
            s.left_wall_distance = left_d.or(Some(sample_bo));
            s.right_wall_distance = right_d.or(Some(sample_bo));
        }
    }

    /// Chainable helper applying default runoff surfaces.
    pub fn with_default_runoff_surfaces(mut self) -> Self {
        self.apply_default_runoff_surfaces();
        self
    }

    /// Deserializes a `Track` from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, TrackError> {
        let mut track: Self = serde_json::from_str(json_str).map_err(|e| TrackError::Json(e.to_string()))?;
        if track.spline.curves.is_empty() && !track.spline.samples.is_empty() {
            track.spline.curves = curve::extract_curves_from_samples(
                &track.spline.samples,
                track.spline.total_length,
                track.spline.closed,
            );
        }
        if let Some(lane) = &mut track.pit_lane {
            if lane.spline.samples.is_empty() && !lane.spline.waypoints.is_empty() {
                lane.spline = TrackSpline::new(lane.spline.waypoints.clone(), false);
            }
        }
        if track.pit_lane.is_some() {
            track.trim_walls_for_pit_lane();
            track.generate_pit_lane_walls();
            track.recompute_pit_lane_junctions();
        }
        track.apply_default_runoff_surfaces();
        if track.network.is_some() {
            track.trim_walls_for_network();
            track.generate_network_walls();
            if let Some(ref mut net) = track.network {
                net.recompute_composite_splines();
            }
        }
        track.geometry.recompute_scenery_obstacles();
        Ok(track)
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
            self.apply_default_runoff_surfaces();
            self.spline = TrackSpline::new(self.spline.waypoints.clone(), self.spline.closed);
            let (left_walls, right_walls, left_poly, right_poly) =
                generate_walls_from_spline(&self.spline, barrier_offset, barrier_type);
            self.geometry.inner_walls = left_walls;
            self.geometry.outer_walls = right_walls;
            self.geometry.left_boundary_polyline = left_poly;
            self.geometry.right_boundary_polyline = right_poly;
            if let Some(lane) = &mut self.pit_lane {
                if lane.spline.samples.is_empty() && !lane.spline.waypoints.is_empty() {
                    lane.spline = TrackSpline::new(lane.spline.waypoints.clone(), false);
                }
            }
            if self.pit_lane.is_some() {
                self.trim_walls_for_pit_lane();
                self.generate_pit_lane_walls();
            }
            if self.network.is_some() {
                self.trim_walls_for_network();
                self.generate_network_walls();
                if let Some(ref mut net) = self.network {
                    net.recompute_composite_splines();
                }
            }
            self.apply_default_runoff_surfaces();
            self.geometry.recompute_scenery_obstacles();
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

    /// Regenerates starting grid positions on the straight before the finish line (or spline origin/arena centroid).
    ///
    /// If spline samples exist (>= 2):
    /// - Aligns slots behind the finish line checkpoint if one exists.
    /// - Falls back to the track spline end/origin (`total_length()`) if no finish line checkpoint exists.
    /// If an arena venue without spline:
    /// - Aligns slots behind finish line checkpoint or at arena centroid.
    /// Returns `true` if grid positions were successfully generated.
    pub fn auto_generate_grid(&mut self, num_slots: usize, spacing: f32, stagger: f32) -> bool {
        let Some(finish_cp) = self.checkpoints.iter().find(|cp| cp.is_finish_line) else {
            return false;
        };

        if self.spline.samples.len() >= 2 {
            let finish_gate_center = (finish_cp.gate.start + finish_cp.gate.end) * 0.5;
            let finish_dist = self.spline.project_point(finish_gate_center).progress_distance;
            self.grid_positions = generate_grid_positions_at_distance(&self.spline, finish_dist, num_slots, spacing, stagger);
            return true;
        }

        let center = (finish_cp.gate.start + finish_cp.gate.end) * 0.5;
        let forward = finish_cp.direction;
        let heading = forward.y.atan2(forward.x);
        self.grid_positions = presets::generate_arena_grid(center - forward * 15.0, heading, num_slots, spacing, stagger);
        true
    }

    /// Computes sensible default grid spacing and lateral stagger tailored for the track's module.
    pub fn default_grid_spacing_and_stagger(&self) -> (f32, f32) {
        if self.belongs_to_module("kart") {
            (5.5, 1.8)
        } else if self.belongs_to_module("gt") {
            (10.0, 2.5)
        } else if self.belongs_to_module("nascar") {
            (7.0, 3.0)
        } else {
            (8.0, 2.5)
        }
    }

    /// Computes sensible default starting grid slot count tailored for the track's module.
    pub fn default_grid_count(&self) -> usize {
        if self.belongs_to_module("gt") {
            18
        } else if self.belongs_to_module("nascar") {
            16
        } else if self.belongs_to_module("kart") {
            14
        } else if self.belongs_to_module("rally")
            || self.belongs_to_module("extreme_offroad")
            || self.belongs_to_module("autocross")
        {
            12
        } else {
            10
        }
    }

    /// Automatically regenerates starting grid positions using current grid length (or module default) and module defaults.
    pub fn auto_generate_grid_default(&mut self) -> bool {
        let count = if self.grid_positions.is_empty() { self.default_grid_count() } else { self.grid_positions.len() };
        let (spacing, stagger) = self.default_grid_spacing_and_stagger();
        self.auto_generate_grid(count, spacing, stagger)
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

/// Test fixture: loads an official circuit from the repository's `tracks/<module>/<id>.json`
/// (spec 042; the Rust circuit generators are gone, and this crate cannot see `tdrace_core::catalog`).
#[cfg(test)]
pub(crate) fn test_circuit(module: &str, id: &str) -> Track {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tracks").join(module).join(format!("{}.json", id));
    Track::load_from_file(&path)
        .unwrap_or_else(|e| panic!("{}: {} (run `git submodule update --init tracks`)", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wheelbase::CarConfig;

    #[test]
    fn test_track_presets_creation() {
        let gp = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        assert_eq!(gp.name, "Coastal Grand Prix");
        assert!(gp.checkpoints.len() >= 10);
        assert!(!gp.grid_positions.is_empty());

        let oval = crate::track::test_circuit("classic", "stock_tri_oval_speedway");
        assert_eq!(oval.name, "Tri-Oval Speedway");
        assert!(oval.spline.total_length() > 400.0);

        let drift = crate::track::test_circuit("classic", "gt_ridge_ring");
        assert_eq!(drift.name, "Ridge Ring");
        assert!(!drift.geometry.surface_zones.is_empty());

        let kart = crate::track::test_circuit("classic", "kart_pine_grove");
        assert_eq!(kart.name, "Pine Grove");

        let rx = crate::track::test_circuit("classic", "rx_quarry_sprint");
        assert_eq!(rx.name, "Quarry Sprint");
        assert!(rx.spline.total_length() >= 750.0 && rx.spline.total_length() <= 1400.0, "Quarry Sprint must be ~1km (got {})", rx.spline.total_length());
        assert_eq!(rx.car_category, CarCategory::Rally);
    }

    /// Closed 8-waypoint loop, 100 m apart, used by the runoff corridor tests.
    fn octagon_waypoints() -> Vec<TrackWaypoint> {
        (0..8)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                TrackWaypoint::new(Vec2::new(a.cos(), a.sin()) * 130.0, 12.0)
            })
            .collect()
    }

    fn octagon_track(waypoints: Vec<TrackWaypoint>) -> Track {
        let spline = TrackSpline::new(waypoints, true);
        let (inner_walls, outer_walls, left_poly, right_poly) =
            generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);
        Track {
            name: "Runoff Test Octagon".to_string(),
            spline,
            geometry: TrackGeometry {
                inner_walls,
                outer_walls,
                left_boundary_polyline: left_poly,
                right_boundary_polyline: right_poly,
                ..Default::default()
            },
            ..Default::default()
        }
        .with_default_runoff_surfaces()
    }

    /// Kart-style stadium: 40 m straights, 6 m radius U-turns, a 7 m road, waypoints about 3 m apart and
    /// walls 0.3 m out. Anticlockwise, so the left walls are the inner ones.
    fn tight_stadium_waypoints() -> Vec<TrackWaypoint> {
        let mut points = Vec::new();
        for (cx, start) in [(20.0f32, -std::f32::consts::FRAC_PI_2), (-20.0, std::f32::consts::FRAC_PI_2)] {
            for k in 0..7 {
                let x = if cx > 0.0 { -20.0 + k as f32 * 40.0 / 7.0 } else { 20.0 - k as f32 * 40.0 / 7.0 };
                points.push(Vec2::new(x, if cx > 0.0 { -6.0 } else { 6.0 }));
            }
            for k in 0..24 {
                let a = start + k as f32 * std::f32::consts::PI / 24.0;
                points.push(Vec2::new(cx, 0.0) + Vec2::new(a.cos(), a.sin()) * 6.0);
            }
        }
        points.into_iter().map(|p| TrackWaypoint::new(p, 7.0).with_wall_distances(Some(0.3), Some(0.3))).collect()
    }

    #[test]
    fn test_short_inner_walls_of_tight_turns_are_kept() {
        let spline = TrackSpline::new(tight_stadium_waypoints(), true);
        let (inner, _outer, left_poly, _right_poly) = generate_walls_from_spline(&spline, 0.6, BarrierType::TireWall);
        // The inner wall pieces of the U-turns are shorter than 0.10 m; trimming used to drop them all.
        let poly_len: f32 = (0..left_poly.len()).map(|i| left_poly[i].distance(left_poly[(i + 1) % left_poly.len()])).sum();
        let wall_len: f32 = inner.iter().map(|w| w.segment.length()).sum();
        assert!(inner.iter().any(|w| w.segment.length() < 0.10), "the test needs short inner wall pieces");
        assert!(wall_len > 0.99 * poly_len, "inner walls cover {:.1} m of the {:.1} m wall line", wall_len, poly_len);
    }

    #[test]
    fn test_single_waypoint_wall_distance_override_blends_without_step() {
        let mut waypoints = octagon_waypoints();
        waypoints[2] = waypoints[2].clone().with_wall_distances(Some(10.0), None);
        let track = octagon_track(waypoints);

        // Corridor width ramps 4 m -> 10 m -> 4 m across the two segments next to waypoint 2.
        let max_step = (10.0 - 4.0) / TrackSpline::MIN_STEPS_PER_SEGMENT as f32 + 1e-3;
        let samples = &track.spline.samples;
        for (i, pair) in samples.windows(2).enumerate() {
            let (a, b) = (pair[0].left_wall_distance.unwrap(), pair[1].left_wall_distance.unwrap());
            assert!((a - b).abs() <= max_step, "left corridor jumps from {a} to {b} at index {i}");
        }
        let wp2_idx = track.spline.waypoint_sample_index(2).unwrap();
        let peak = samples[wp2_idx].left_wall_distance.unwrap();
        assert!((peak - 10.0).abs() < 1e-3, "override must hold at its waypoint (got {peak})");
        let wp3_idx = track.spline.waypoint_sample_index(3).unwrap();
        let mid = samples[(wp2_idx + wp3_idx) / 2]
            .left_wall_distance
            .unwrap();
        assert!((mid - 7.0).abs() < 0.05, "corridor must be halfway at the segment midpoint (got {mid})");

        // Walls follow the same blend: no wall segment endpoint may jump sideways.
        for pair in track.geometry.inner_walls.windows(2) {
            let gap = (pair[0].segment.end - pair[1].segment.start).length();
            assert!(gap < 0.5, "left wall has a {gap:.2} m break");
        }
    }

    #[test]
    fn test_runoff_corridor_width_set_where_wall_is_absent() {
        let mut waypoints = octagon_waypoints();
        waypoints[4] = waypoints[4].clone().with_walls(false, true);
        let track = octagon_track(waypoints);

        // Physics treats the corridor as present without a wall; the sample data must say so too.
        for s in &track.spline.samples {
            assert!(s.left_wall_distance.is_some() && s.right_wall_distance.is_some());
        }
        let open = track.spline.samples.iter().find(|s| !s.left_wall).expect("walls-off samples");
        let probe = open.point + open.normal * (open.width * 0.5 + 2.0);
        assert_eq!(track.sample_surface(probe), track.default_runoff_surface().unwrap());
    }

    #[test]
    fn test_track_json_serialization_roundtrip() {
        let presets = [
            crate::track::test_circuit("classic", "gt_coastal_grand_prix"),
            crate::track::test_circuit("classic", "stock_tri_oval_speedway"),
            crate::track::test_circuit("classic", "gt_ridge_ring"),
            crate::track::test_circuit("classic", "kart_pine_grove"),
            crate::track::test_circuit("classic", "rx_hilltop_leap"),
            crate::track::test_circuit("classic", "at_dune_sea"),
            crate::track::test_circuit("classic", "rx_quarry_sprint"),
            crate::track::test_circuit("extreme_offroad", "dirt_figure_eight"),
            crate::track::test_circuit("rally", "holjes_rx"),
            crate::track::test_circuit("rally", "lydden_hill"),
            crate::track::test_circuit("rally", "hell_rx"),
            crate::track::test_circuit("rally", "loheac_rx"),
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
        let mut track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        for wp in &mut track.spline.waypoints {
            wp.wall_type = None;
        }
        track.rebuild_geometry(5.0, BarrierType::Concrete);
        assert!(!track.geometry.inner_walls.is_empty());
        assert_eq!(track.geometry.inner_walls.first().unwrap().barrier_type, BarrierType::Concrete);
        assert!(!track.geometry.outer_walls.is_empty());
    }

    #[test]
    fn test_track_surface_sampling() {
        let mut track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        let p0 = track.spline.samples[0].point;
        let n0 = track.spline.samples[0].normal;

        // Sample on start line center (should be Asphalt)
        let surf_start = track.sample_surface(p0);
        assert_eq!(surf_start, SurfaceType::Asphalt);

        // Sample far outside the track
        let surf_far = track.sample_surface(p0 + n0 * 300.0);
        assert_eq!(surf_far, SurfaceType::Grass);

        // Insert BelowTrack sand trap overlapping ribbon
        track.geometry.surface_zones.push(
            SurfaceZone::new(
                SurfaceShape::Aabb {
                    min: p0 - Vec2::splat(20.0),
                    max: p0 + Vec2::splat(20.0),
                },
                SurfaceType::DeepSand,
                "Sand Trap",
            ).with_layer(SurfaceLayer::BelowTrack),
        );
        // On-track points must stay Asphalt
        assert_eq!(track.sample_surface(p0), SurfaceType::Asphalt);
        // Off-track point inside trap beyond runoff corridor must sample DeepSand
        assert_eq!(track.sample_surface(p0 + n0 * 15.0), SurfaceType::DeepSand);

        // Test sample_car_surfaces
        let car = Car::new(CarConfig::sports_car()).with_pose(p0, 0.0);
        let wheel_surfs = track.sample_car_surfaces(&car);
        assert_eq!(wheel_surfs, [SurfaceType::Asphalt; 4]);
    }

    #[test]
    fn test_track_surface_breakdown() {
        let gp = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        let gp_breakdown = gp.surface_breakdown();
        assert_eq!(gp_breakdown.len(), 1);
        assert_eq!(gp_breakdown[0].0, SurfaceType::Asphalt);
        assert!((gp_breakdown[0].1 - 100.0).abs() < 1e-3);
        assert_eq!(gp.surface_summary_string(), "100% Asphalt");
        assert!(gp.total_length_m() > 400.0);

        let dune = crate::track::test_circuit("classic", "at_dune_sea");
        let dune_breakdown = dune.surface_breakdown();
        assert_eq!(dune_breakdown.len(), 1);
        assert_eq!(dune_breakdown[0].0, SurfaceType::PackedSand);
        assert!((dune_breakdown[0].1 - 100.0).abs() < 1e-3);
        assert_eq!(dune.surface_summary_string(), "100% Packed Sand");

        // Custom mixed-surface spline
        let mut mixed = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
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
        let mut track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        track.geometry.surface_zones.clear();
        let p0 = track.spline.samples[0].point;
        let n0 = track.spline.samples[0].normal;

        // 1. BelowTrack Sand Zone overlapping the start line
        let sand_shape = SurfaceShape::Aabb {
            min: p0 - Vec2::splat(20.0),
            max: p0 + Vec2::splat(20.0),
        };
        track.geometry.surface_zones.push(
            SurfaceZone::new(sand_shape, SurfaceType::DeepSand, "Sand Under Road")
                .with_layer(SurfaceLayer::BelowTrack),
        );

        // Point on track should remain Asphalt because track ribbon sits above BelowTrack sand
        assert_eq!(track.sample_surface(p0), SurfaceType::Asphalt);
        // Off-track point outside ribbon and runoff corridor but inside the sand zone
        assert_eq!(track.sample_surface(p0 + n0 * 15.0), SurfaceType::DeepSand);

        // 2. AboveTrack Sand Zone overlapping the start line
        track.geometry.surface_zones[0].layer = SurfaceLayer::AboveTrack;
        // Point on track should now be DeepSand because AboveTrack sits on top of asphalt!
        assert_eq!(track.sample_surface(p0), SurfaceType::DeepSand);

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

    #[test]
    fn test_grandstand_and_tree_scenery_sampling_and_serialization() {
        let mut track = crate::track::test_circuit("classic", "gt_coastal_grand_prix");
        let stand = Grandstand::new(1, Vec2::new(0.0, 50.0), 30.0, 10.0, 0.0);
        let tree = Tree::new(2, Vec2::new(100.0, 100.0), TreeType::Palm).with_scale(1.5);

        track.geometry.grandstands.clear();
        track.geometry.trees.clear();
        track.geometry.grandstands.push(stand.clone());
        track.geometry.trees.push(tree.clone());
        track.geometry.recompute_scenery_obstacles();

        // Grandstand footprint must sample as SurfaceType::Concrete
        assert_eq!(track.sample_surface(Vec2::new(0.0, 50.0)), SurfaceType::Concrete);
        assert_eq!(track.sample_surface_near(Vec2::new(0.0, 50.0), 0.0), SurfaceType::Concrete);

        // All obstacles with scenery must include both tree trunk obstacle and grandstand obstacle
        let all_obs = track.geometry.all_obstacles_with_scenery();
        assert!(all_obs.iter().any(|o| o.name.contains("Palm Trunk")));
        assert!(all_obs.iter().any(|o| o.name.contains("Grandstand #1")));

        // Roundtrip JSON serialization must preserve grandstands and trees
        let json = track.to_json_pretty().expect("Must serialize");
        let deserialized = Track::from_json(&json).expect("Must deserialize");
        assert_eq!(deserialized.geometry.grandstands.len(), 1);
        assert_eq!(deserialized.geometry.grandstands[0].length, 30.0);
        assert_eq!(deserialized.geometry.trees.len(), 1);
        assert_eq!(deserialized.geometry.trees[0].tree_type, TreeType::Palm);
    }

    #[test]
    fn test_segment_runoff_corridor_surface_resolution() {
        // Build a straight 3-waypoint track going along X axis from 0 to 100
        let wps = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0)
                .with_curbs(false, false)
                .with_runoff_surfaces(Some(SurfaceType::PackedGravel), Some(SurfaceType::Asphalt))
                .with_wall_distances(Some(6.0), Some(12.0)),
            TrackWaypoint::new(Vec2::new(50.0, 0.0), 10.0)
                .with_curbs(false, false)
                .with_runoff_surfaces(Some(SurfaceType::PackedGravel), Some(SurfaceType::Asphalt))
                .with_wall_distances(Some(6.0), Some(12.0)),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0)
                .with_curbs(false, false)
                .with_runoff_surfaces(Some(SurfaceType::PackedGravel), Some(SurfaceType::Asphalt))
                .with_wall_distances(Some(6.0), Some(12.0)),
        ];

        let track = Track {
            spline: TrackSpline::new(wps, false),
            default_surface: SurfaceType::Grass,
            ..Default::default()
        };

        // Center point at (50, 0) -> on track -> Asphalt (track base)
        assert_eq!(track.sample_surface(Vec2::new(50.0, 0.0)), SurfaceType::Asphalt);

        // Track is 10m wide (half-width = 5.0m).
        // Tangent is +X, normal points left (+Y), right points right (-Y).
        // Left side (+Y): lateral offset is -y in spline local frame.
        // At y = 8.0 (3.0m off track to the left, inside the 6m left runoff corridor):
        assert_eq!(
            track.sample_surface(Vec2::new(50.0, 8.0)),
            SurfaceType::PackedGravel,
            "Left runoff corridor must resolve to Gravel"
        );
        assert_eq!(
            track.sample_surface_near(Vec2::new(50.0, 8.0), 50.0),
            SurfaceType::PackedGravel,
            "Left runoff corridor near-sample must resolve to Gravel"
        );

        // At y = 14.0 (9.0m off track to the left, beyond the 6m corridor limit = 5 + 6 = 11.0m):
        assert_eq!(
            track.sample_surface(Vec2::new(50.0, 14.0)),
            SurfaceType::Grass,
            "Beyond left corridor limit must fall back to default backdrop (Grass)"
        );

        // Right side (-Y): lateral offset is +y (abs) in right direction.
        // At y = -10.0 (5.0m off track to the right, inside the 12m right runoff corridor):
        assert_eq!(
            track.sample_surface(Vec2::new(50.0, -10.0)),
            SurfaceType::Asphalt,
            "Right runoff corridor must resolve to Asphalt runoff"
        );

        // At y = -20.0 (15.0m off track to the right, beyond the 12m limit = 5 + 12 = 17.0m):
        assert_eq!(
            track.sample_surface(Vec2::new(50.0, -20.0)),
            SurfaceType::Grass,
            "Beyond right corridor limit must fall back to Grass"
        );
    }

    #[test]
    fn test_default_runoff_surfaces_across_disciplines() {
        let make_track = |name: &str, category: CarCategory, default_surf: SurfaceType, module: &str| {
            let wps = vec![
                TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
                TrackWaypoint::new(Vec2::new(50.0, 0.0), 10.0),
                TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0),
            ];
            Track {
                name: name.to_string(),
                car_category: category,
                default_surface: default_surf,
                module_id: Some(module.to_string()),
                modules: vec![module.to_string()],
                spline: TrackSpline::new(wps, false),
                ..Default::default()
            }
        };

        // 1. GT Circuit -> Gravel
        let mut gt_track = make_track("Spa GP", CarCategory::Gt, SurfaceType::Grass, "gt");
        assert_eq!(gt_track.default_runoff_surface(), Some(SurfaceType::DeepGravel));
        gt_track.apply_default_runoff_surfaces();
        assert_eq!(gt_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::DeepGravel));
        assert_eq!(gt_track.spline.samples[0].right_runoff_surface, Some(SurfaceType::DeepGravel));

        // 2. Rallycross Circuit -> Gravel
        let mut rx_track = make_track("Holjes RX", CarCategory::Rally, SurfaceType::Grass, "rally");
        assert_eq!(rx_track.default_runoff_surface(), Some(SurfaceType::DeepGravel));
        rx_track.apply_default_runoff_surfaces();
        assert_eq!(rx_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::DeepGravel));

        // 3. Kart Circuit -> Concrete
        let mut kart_track = make_track("Lonato Karting", CarCategory::Kart, SurfaceType::Grass, "kart");
        assert_eq!(kart_track.default_runoff_surface(), Some(SurfaceType::Concrete));
        kart_track.apply_default_runoff_surfaces();
        assert_eq!(kart_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::Concrete));

        // 4. Pure Dirt or Mud Circuit -> Dirt
        let mut dirt_track = make_track("Crandon Offroad", CarCategory::OffRoad, SurfaceType::Dirt, "extreme_offroad");
        assert_eq!(dirt_track.default_runoff_surface(), Some(SurfaceType::Dirt));
        dirt_track.apply_default_runoff_surfaces();
        assert_eq!(dirt_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::Dirt));

        let mut mud_track = make_track("Louisiana Mud Slough", CarCategory::OffRoad, SurfaceType::MudTrack, "extreme_offroad");
        assert_eq!(mud_track.default_runoff_surface(), Some(SurfaceType::Dirt));
        mud_track.apply_default_runoff_surfaces();
        assert_eq!(mud_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::Dirt));

        // 5. Sandy Circuit -> DeepSand
        let mut sand_track = make_track("Sahara Dunes", CarCategory::OffRoad, SurfaceType::PackedSand, "extreme_offroad");
        assert_eq!(sand_track.default_runoff_surface(), Some(SurfaceType::DeepSand));
        sand_track.apply_default_runoff_surfaces();
        assert_eq!(sand_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::DeepSand));

        // 6. Snow or Icy Circuit -> DeepSnow
        let mut snow_track = make_track("Alpine Snow Ridge", CarCategory::OffRoad, SurfaceType::PackedSnow, "extreme_offroad");
        assert_eq!(snow_track.default_runoff_surface(), Some(SurfaceType::DeepSnow));
        snow_track.apply_default_runoff_surfaces();
        assert_eq!(snow_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::DeepSnow));

        let mut ice_track = make_track("Arctic Ice Lake", CarCategory::OffRoad, SurfaceType::SheetIce, "extreme_offroad");
        assert_eq!(ice_track.default_runoff_surface(), Some(SurfaceType::DeepSnow));
        ice_track.apply_default_runoff_surfaces();
        assert_eq!(ice_track.spline.samples[0].left_runoff_surface, Some(SurfaceType::DeepSnow));

        // Custom override preservation:
        let mut custom_track = make_track("Nurburgring GP", CarCategory::Gt, SurfaceType::Grass, "gt");
        custom_track.spline.waypoints[1].left_runoff_surface = Some(SurfaceType::DeepSand);
        custom_track.apply_default_runoff_surfaces();
        assert_eq!(custom_track.spline.waypoints[0].left_runoff_surface, Some(SurfaceType::DeepGravel));
        assert_eq!(custom_track.spline.waypoints[1].left_runoff_surface, Some(SurfaceType::DeepSand));
        assert_eq!(custom_track.spline.waypoints[2].left_runoff_surface, Some(SurfaceType::DeepGravel));
    }

    #[test]
    fn test_pit_lane_surface_sampling_and_walls() {
        use super::geometry::{PitBox, PitLane};
        use super::spline::TrackWaypoint;

        // Main track: straight from x = -50 to x = 100 at y = 0
        let main_waypoints = vec![
            TrackWaypoint::new(Vec2::new(-50.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(50.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(100.0, 50.0), 12.0),
            TrackWaypoint::new(Vec2::new(-50.0, 50.0), 12.0),
        ];

        // Pit lane on driver's right (-y): parallel straight at y = -14.0
        let pit_waypoints = vec![
            TrackWaypoint::new(Vec2::new(-30.0, -6.0), 7.0),
            TrackWaypoint::new(Vec2::new(-10.0, -14.0), 7.0),
            TrackWaypoint::new(Vec2::new(20.0, -14.0), 7.0),
            TrackWaypoint::new(Vec2::new(50.0, -14.0), 7.0),
            TrackWaypoint::new(Vec2::new(70.0, -6.0), 7.0),
        ];

        let pit_lane = PitLane {
            spline: TrackSpline::new(pit_waypoints, false),
            road_width: 7.0,
            speed_limit: 16.67,
            pit_boxes: vec![
                PitBox {
                    position: Vec2::new(20.0, -15.8),
                    direction: Vec2::X,
                    stop_radius: 3.0,
                    elevation: 0.0,
                },
            ],
            entry_gate: LineSegment::new(Vec2::new(-10.0, -17.5), Vec2::new(-10.0, -10.5)),
            exit_gate: LineSegment::new(Vec2::new(50.0, -17.5), Vec2::new(50.0, -10.5)),
        };

        let mut track = Track {
            name: "Pit Test GP".to_string(),
            default_surface: SurfaceType::Grass,
            car_category: CarCategory::Gt,
            spline: TrackSpline::new(main_waypoints, true),
            pit_lane: Some(pit_lane),
            modules: vec!["gt".to_string()],
            ..Default::default()
        };

        // 1. Surface sampling tests:
        // A. On pit road centerline (x = 20, y = -14.0) -> Asphalt
        assert_eq!(track.sample_surface(Vec2::new(20.0, -14.0)), SurfaceType::Asphalt);
        // B. Inside pit box (x = 20, y = -15.8) -> Asphalt
        assert_eq!(track.sample_surface(Vec2::new(20.0, -15.8)), SurfaceType::Asphalt);
        // C. Off track away from pit and track (x = 20, y = -30.0) -> Grass
        assert_eq!(track.sample_surface(Vec2::new(20.0, -30.0)), SurfaceType::Grass);

        // 2. Pit wall generation:
        track.generate_pit_lane_walls();
        assert!(!track.geometry.outer_walls.is_empty() || !track.geometry.inner_walls.is_empty(),
            "Pit lane wall generation should produce dividing and outer walls");

        // 3. Validation:
        let errors = track.validate();
        let blocking = errors.iter().filter(|e| e.code == "ERR_WALL_BLOCKS_PIT_LANE").count();
        assert_eq!(blocking, 0, "No walls should block pit lane: {:?}", errors);
    }
}
