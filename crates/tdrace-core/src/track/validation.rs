use serde::{Deserialize, Serialize};

use super::Track;
use crate::track::geometry::{LineSegment, ObstacleShape, SurfaceShape};

/// Severity level of a track validation diagnostic finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

/// A specific diagnostic issue identified during circuit validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackValidationError {
    pub severity: ValidationSeverity,
    pub code: &'static str,
    pub message: String,
    pub details: Option<String>,
    pub entity_index: Option<usize>,
}

impl TrackValidationError {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            code,
            message: message.into(),
            details: None,
            entity_index: None,
        }
    }

    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            code,
            message: message.into(),
            details: None,
            entity_index: None,
        }
    }

    pub fn info(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Info,
            code,
            message: message.into(),
            details: None,
            entity_index: None,
        }
    }

    pub fn with_index(mut self, idx: usize) -> Self {
        self.entity_index = Some(idx);
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Performs comprehensive geometric, topological, and gameplay checks on a track.
pub fn validate_track(track: &Track) -> Vec<TrackValidationError> {
    let mut diagnostics = Vec::new();

    // 1. Waypoint & Spline Topology Checks
    let waypoints = &track.spline.waypoints;
    let n_wp = waypoints.len();

    if n_wp < 3 {
        diagnostics.push(
            TrackValidationError::error(
                "ERR_INSUFFICIENT_WAYPOINTS",
                format!("Track requires at least 3 waypoints (currently {}).", n_wp),
            )
            .with_details("Add more waypoints to define a drivable centerline curve."),
        );
        return diagnostics; // Cannot perform further spline tests if < 3 waypoints
    }

    // Check consecutive waypoint distances and widths
    for i in 0..n_wp {
        let next_i = if track.spline.closed {
            (i + 1) % n_wp
        } else if i + 1 < n_wp {
            i + 1
        } else {
            continue;
        };

        let p0 = waypoints[i].point;
        let p1 = waypoints[next_i].point;
        let dist = (p1 - p0).length();

        if dist < 3.0 {
            diagnostics.push(
                TrackValidationError::error(
                    "ERR_WAYPOINT_TOO_CLOSE",
                    format!(
                        "Waypoints #{} and #{} are only {:.1}m apart (minimum is 3.0m).",
                        i + 1,
                        next_i + 1,
                        dist
                    ),
                )
                .with_index(i)
                .with_details("Move waypoints further apart or delete redundant node."),
            );
        } else if dist > 350.0 {
            diagnostics.push(
                TrackValidationError::warning(
                    "WARN_WAYPOINT_TOO_FAR",
                    format!(
                        "Waypoints #{} and #{} are {:.1}m apart (recommended <= 300m for smooth spline interpolation).",
                        i + 1,
                        next_i + 1,
                        dist
                    ),
                )
                .with_index(i),
            );
        }

        let w = waypoints[i].width;
        if w < 4.0 {
            diagnostics.push(
                TrackValidationError::error(
                    "ERR_TRACK_TOO_NARROW",
                    format!("Waypoint #{} width is {:.1}m (minimum drivable width is 4.0m).", i + 1, w),
                )
                .with_index(i),
            );
        } else if w > 50.0 {
            diagnostics.push(
                TrackValidationError::warning(
                    "WARN_TRACK_VERY_WIDE",
                    format!("Waypoint #{} width is {:.1}m (unusually wide).", i + 1, w),
                )
                .with_index(i),
            );
        }
    }

    // Spline total length check
    let total_len = track.spline.total_length();
    if total_len < 60.0 {
        diagnostics.push(
            TrackValidationError::error(
                "ERR_TRACK_TOO_SHORT",
                format!("Total track length is only {:.1}m (minimum 60m).", total_len),
            )
            .with_details("Extend the circuit length for proper racing dynamics and timing."),
        );
    }

    // 2. Spline Centerline Crossover / Overpass Clearance Checks
    let samples = &track.spline.samples;
    let n_samples = samples.len();
    if n_samples >= 4 {
        let n_segs = if track.spline.closed { n_samples } else { n_samples - 1 };
        // Check non-adjacent spline segments for 2D intersection
        for i in 0..n_segs {
            let next_i = (i + 1) % n_samples;
            let seg_i = LineSegment::new(samples[i].point, samples[next_i].point);

            // Start j at i + 6 to avoid false positives on adjacent spline samples on sharp curves
            let j_start = i + 6;
            let j_end = if track.spline.closed {
                if i < 5 { n_segs.saturating_sub(6 - i) } else { n_segs }
            } else {
                n_segs
            };

            for j in j_start..j_end {
                let next_j = (j + 1) % n_samples;
                let seg_j = LineSegment::new(samples[j].point, samples[next_j].point);

                if let Some(hit) = seg_i.intersect_segment(&seg_j) {
                    let elev_i = (samples[i].elevation + samples[next_i].elevation) * 0.5;
                    let elev_j = (samples[j].elevation + samples[next_j].elevation) * 0.5;
                    let clearance = (elev_i - elev_j).abs();

                    if clearance < 3.5 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_CROSSOVER_INSUFFICIENT_CLEARANCE",
                                format!(
                                    "Track centerline self-intersects at ({:.1}, {:.1}) with only {:.1}m vertical clearance (minimum 3.5m required for overpass bridge).",
                                    hit.x, hit.y, clearance
                                ),
                            )
                            .with_details(format!(
                                "Segment near distance {:.0}m (elev {:.1}m) crosses segment near distance {:.0}m (elev {:.1}m). Increase bridge height or reroute track.",
                                samples[i].distance, elev_i, samples[j].distance, elev_j
                            )),
                        );
                    } else if clearance < 4.0 {
                        diagnostics.push(
                            TrackValidationError::warning(
                                "WARN_CROSSOVER_LOW_CLEARANCE",
                                format!(
                                    "Track crossover bridge at ({:.1}, {:.1}) has tight vertical clearance ({:.1}m, recommended >= 4.0m).",
                                    hit.x, hit.y, clearance
                                ),
                            ),
                        );
                    }
                }
            }
        }
    }

    // 3. Wall Barriers vs Track Spline & Drivable Ribbon Checks
    let all_walls: Vec<_> = track.geometry.all_walls().copied().collect();
    let n_walls = all_walls.len();

    if n_samples >= 2 && n_walls > 0 {
        let n_segs = if track.spline.closed { n_samples } else { n_samples - 1 };

        // For each wall, test intersection against centerline and drivable ribbon
        for (w_idx, wall) in all_walls.iter().enumerate() {
            let w_seg = &wall.segment;
            let w_elev = wall.elevation;

            // 3a. Centerline Crossing Check against all spline segments
            for i in 0..n_segs {
                let next_i = (i + 1) % n_samples;
                let s_curr = &samples[i];
                let s_next = &samples[next_i];

                let track_center_seg = LineSegment::new(s_curr.point, s_next.point);
                let track_elev = (s_curr.elevation + s_next.elevation) * 0.5;
                let elev_diff = (w_elev - track_elev).abs();

                if let Some(hit) = w_seg.intersect_segment(&track_center_seg) {
                    if elev_diff < 3.0 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_WALL_CROSSES_TRACK",
                                format!(
                                    "Wall barrier #{} ({:?} -> {:?}) crosses track centerline at ({:.1}, {:.1}) with only {:.1}m elevation clearance.",
                                    w_idx + 1,
                                    w_seg.start,
                                    w_seg.end,
                                    hit.x,
                                    hit.y,
                                    elev_diff
                                ),
                            )
                            .with_index(w_idx)
                            .with_details(format!(
                                "Wall elevation is {:.1}m while track elevation is {:.1}m near distance {:.0}m. Cars will crash into wall.",
                                w_elev, track_elev, s_curr.distance
                            )),
                        );
                    }
                }
            }

            // 3b. Drivable Track Surface Intrusion Check (Start, Midpoint, End)
            let test_points = [
                ("start", w_seg.start),
                ("midpoint", (w_seg.start + w_seg.end) * 0.5),
                ("end", w_seg.end),
            ];

            for (pt_name, pt) in test_points {
                let proj = track.spline.project_point(pt);
                let elev_diff = (w_elev - proj.elevation).abs();

                if elev_diff < 2.5 {
                    let curb_extra = if proj.left_curb || proj.right_curb { 1.35 } else { 0.0 };
                    let road_half_w = proj.track_width * 0.5 + curb_extra;

                    // If point is closer to centerline than the road edge (allowing 0.15m tolerance for wall on edge)
                    if proj.distance_to_spline < road_half_w - 0.20 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_WALL_INTRUDES_TRACK",
                                format!(
                                    "Wall barrier #{} {} vertex ({:.1}, {:.1}) intrudes into drivable track surface (dist to center: {:.1}m, road half-width: {:.1}m).",
                                    w_idx + 1,
                                    pt_name,
                                    pt.x,
                                    pt.y,
                                    proj.distance_to_spline,
                                    road_half_w
                                ),
                            )
                            .with_index(w_idx)
                            .with_details(format!(
                                "Occurs near track distance {:.0}m. Move wall outwards away from racing line.",
                                proj.progress_distance
                            )),
                        );
                        break; // Only report once per wall segment
                    }
                }
            }
        }

        // 4. Wall-Wall Self-Intersection Checks (Clashing / Pinching Barriers)
        for i in 0..n_walls {
            for j in (i + 1)..n_walls {
                let w_a = &all_walls[i];
                let w_b = &all_walls[j];

                // Skip adjacent connected segments (sharing endpoints)
                if (w_a.segment.start - w_b.segment.start).length_squared() < 0.01
                    || (w_a.segment.start - w_b.segment.end).length_squared() < 0.01
                    || (w_a.segment.end - w_b.segment.start).length_squared() < 0.01
                    || (w_a.segment.end - w_b.segment.end).length_squared() < 0.01
                {
                    continue;
                }

                if let Some(hit) = w_a.segment.intersect_segment(&w_b.segment) {
                    let elev_diff = (w_a.elevation - w_b.elevation).abs();
                    if elev_diff < 2.5 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_WALL_SELF_INTERSECTION",
                                format!(
                                    "Wall barriers #{} and #{} intersect at ({:.1}, {:.1}) at same elevation ({:.1}m vs {:.1}m).",
                                    i + 1,
                                    j + 1,
                                    hit.x,
                                    hit.y,
                                    w_a.elevation,
                                    w_b.elevation
                                ),
                            )
                            .with_index(i)
                            .with_details("Intersecting barrier walls create geometric pinch traps. Retract or untangle overlapping wall lines."),
                        );
                    }
                }
            }
        }
    }

    // 5. Checkpoint Checks
    let checkpoints = &track.checkpoints;
    if checkpoints.is_empty() {
        diagnostics.push(
            TrackValidationError::error(
                "ERR_NO_CHECKPOINTS",
                "Track has no checkpoints defined.",
            )
            .with_details("Use Auto-Generate Checkpoints in the editor to create timing gates."),
        );
    } else {
        let finish_count = checkpoints.iter().filter(|cp| cp.is_finish_line).count();
        if finish_count == 0 {
            diagnostics.push(
                TrackValidationError::error(
                    "ERR_NO_FINISH_LINE",
                    "Track has no start/finish line checkpoint.",
                )
                .with_details("Mark at least one checkpoint with is_finish = true."),
            );
        } else if finish_count > 1 {
            diagnostics.push(
                TrackValidationError::warning(
                    "WARN_MULTIPLE_FINISH_LINES",
                    format!("Track has {} finish line checkpoints (typically exactly 1).", finish_count),
                ),
            );
        }

        if track.spline.closed && checkpoints.len() < 4 {
            diagnostics.push(
                TrackValidationError::warning(
                    "WARN_FEW_CHECKPOINTS",
                    format!("Track only has {} checkpoints (recommended >= 6 to prevent cutting).", checkpoints.len()),
                ),
            );
        }

        // Check if any wall blocks a mainline checkpoint gate at same elevation within the drivable track
        for (cp_idx, cp) in checkpoints.iter().enumerate() {
            // Skip pit lane entry/exit gates
            if cp.is_pit_entry || cp.is_pit_exit {
                continue;
            }

            let cp_sample = track.spline.sample_at_distance(cp.target_distance);
            let drivable_half_w = cp_sample.width * 0.5;

            for (w_idx, wall) in all_walls.iter().enumerate() {
                let elev_diff = (cp.elevation - wall.elevation).abs();
                if elev_diff < 2.5 {
                    if let Some(hit) = cp.gate.intersect_segment(&wall.segment) {
                        // Check if intersection occurs inside the drivable road span
                        let gate_mid = (cp.gate.start + cp.gate.end) * 0.5;
                        let dist_from_mid = (hit - gate_mid).length();
                        if dist_from_mid < drivable_half_w - 0.5 {
                            diagnostics.push(
                                TrackValidationError::error(
                                    "ERR_CHECKPOINT_BLOCKED",
                                    format!(
                                        "Checkpoint #{} timing gate is obstructed by wall barrier #{} at ({:.1}, {:.1}) on the drivable road.",
                                        cp_idx + 1,
                                        w_idx + 1,
                                        hit.x,
                                        hit.y
                                    ),
                                )
                                .with_index(cp_idx)
                                .with_details("Timing gate path must remain unobstructed across the drivable track."),
                            );
                        }
                    }
                }
            }
        }
    }

    // 6. Starting Grid Checks
    let grid = &track.grid_positions;
    if grid.is_empty() {
        diagnostics.push(
            TrackValidationError::error(
                "ERR_NO_STARTING_GRID",
                "Track has no starting grid spawn positions.",
            )
            .with_details("Use Auto-Generate Grid in the editor to place starting slots."),
        );
    } else {
        if grid.len() < 2 {
            diagnostics.push(
                TrackValidationError::warning(
                    "WARN_SMALL_GRID",
                    "Starting grid has only 1 slot (multi-car racing disabled).",
                ),
            );
        }

        // Check for duplicate grid positions
        for i in 0..grid.len() {
            for j in (i + 1)..grid.len() {
                let dist = (grid[i].position - grid[j].position).length();
                if dist < 1.5 {
                    diagnostics.push(
                        TrackValidationError::error(
                            "ERR_OVERLAPPING_GRID_SLOTS",
                            format!("Grid slot #{} and #{} overlap ({:.1}m apart).", i + 1, j + 1, dist),
                        )
                        .with_index(i),
                    );
                }
            }
        }

        // Check grid slots against track boundaries and walls
        for (i, pose) in grid.iter().enumerate() {
            let proj = track.spline.project_point(pose.position);
            let half_track_w = proj.track_width * 0.5 + if proj.left_curb || proj.right_curb { 1.4 } else { 0.0 };

            if proj.distance_to_spline > half_track_w + 0.5 {
                diagnostics.push(
                    TrackValidationError::error(
                        "ERR_GRID_SLOT_OFF_TRACK",
                        format!(
                            "Starting grid slot #{} at ({:.1}, {:.1}) is off the drivable track ({:.1}m from centerline, width {:.1}m).",
                            i + 1,
                            pose.position.x,
                            pose.position.y,
                            proj.distance_to_spline,
                            proj.track_width
                        ),
                    )
                    .with_index(i),
                );
            }

            // Check clearance to walls (car half-width ~ 1.0m, require >= 0.8m)
            for (w_idx, wall) in all_walls.iter().enumerate() {
                let elev_diff = (proj.elevation - wall.elevation).abs();
                if elev_diff < 2.5 {
                    let d = wall.segment.distance_to_point(pose.position);
                    if d < 0.8 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_GRID_SLOT_COLLIDES_WALL",
                                format!(
                                    "Starting grid slot #{} at ({:.1}, {:.1}) collides with wall #{} (clearance only {:.2}m, minimum 0.8m).",
                                    i + 1,
                                    pose.position.x,
                                    pose.position.y,
                                    w_idx + 1,
                                    d
                                ),
                            )
                            .with_index(i),
                        );
                    }
                }
            }
        }
    }

    // 7. Static Obstacle Checks
    for (i, obs) in track.geometry.obstacles.iter().enumerate() {
        let center = obs.center();
        let proj = track.spline.project_point(center);
        let elev_diff = (obs.elevation - proj.elevation).abs();

        if elev_diff < 2.5 {
            let half_w = proj.track_width * 0.5;
            let curb_w = half_w + if proj.left_curb || proj.right_curb { 1.4 } else { 0.0 };

            match &obs.shape {
                ObstacleShape::Circle { radius, .. } => {
                    if proj.distance_to_spline < half_w + *radius - 0.2 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_OBSTACLE_ON_TRACK",
                                format!(
                                    "Obstacle #{} '{}' at ({:.1}, {:.1}) is placed directly on the drivable track surface.",
                                    i + 1,
                                    obs.name,
                                    center.x,
                                    center.y
                                ),
                            )
                            .with_index(i),
                        );
                    } else if proj.distance_to_spline < curb_w + *radius {
                        diagnostics.push(
                            TrackValidationError::warning(
                                "WARN_OBSTACLE_NEAR_TRACK",
                                format!(
                                    "Obstacle #{} '{}' at ({:.1}, {:.1}) is within safety margin of track edge ({:.1}m from center).",
                                    i + 1,
                                    obs.name,
                                    center.x,
                                    center.y,
                                    proj.distance_to_spline
                                ),
                            )
                            .with_index(i),
                        );
                    }
                }
                ObstacleShape::Box { half_extents, .. } => {
                    let r = half_extents.x.max(half_extents.y);
                    if proj.distance_to_spline < half_w + r - 0.2 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_OBSTACLE_ON_TRACK",
                                format!(
                                    "Box obstacle #{} '{}' at ({:.1}, {:.1}) intrudes into the drivable track surface.",
                                    i + 1,
                                    obs.name,
                                    center.x,
                                    center.y
                                ),
                            )
                            .with_index(i),
                        );
                    }
                }
                ObstacleShape::Polygon { vertices } => {
                    if vertices.len() < 3 {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_INVALID_OBSTACLE_POLYGON",
                                format!("Obstacle #{} '{}' has fewer than 3 vertices.", i + 1, obs.name),
                            )
                            .with_index(i),
                        );
                    } else if proj.distance_to_spline < half_w {
                        diagnostics.push(
                            TrackValidationError::error(
                                "ERR_OBSTACLE_ON_TRACK",
                                format!(
                                    "Polygon obstacle #{} '{}' at ({:.1}, {:.1}) intrudes into the drivable track surface.",
                                    i + 1,
                                    obs.name,
                                    center.x,
                                    center.y
                                ),
                            )
                            .with_index(i),
                        );
                    }
                }
            }
        }
    }

    // 8. Surface Zone & Hazard Checks
    for (i, zone) in track.geometry.surface_zones.iter().enumerate() {
        match &zone.shape {
            SurfaceShape::Circle { radius, .. } => {
                if *radius < 0.5 {
                    diagnostics.push(
                        TrackValidationError::warning(
                            "WARN_TINY_SURFACE_ZONE",
                            format!("Surface zone #{} '{}' has very small radius ({:.1}m).", i + 1, zone.name, radius),
                        )
                        .with_index(i),
                    );
                }
            }
            SurfaceShape::Aabb { min, max } => {
                if (max.x - min.x).abs() < 0.5 || (max.y - min.y).abs() < 0.5 {
                    diagnostics.push(
                        TrackValidationError::warning(
                            "WARN_TINY_SURFACE_ZONE",
                            format!("Surface zone #{} '{}' is nearly zero-sized.", i + 1, zone.name),
                        )
                        .with_index(i),
                    );
                }
            }
            SurfaceShape::OrientedBox { half_extents, .. } => {
                if half_extents.x < 0.5 || half_extents.y < 0.5 {
                    diagnostics.push(
                        TrackValidationError::warning(
                            "WARN_TINY_SURFACE_ZONE",
                            format!("Surface zone #{} '{}' is very small.", i + 1, zone.name),
                        )
                        .with_index(i),
                    );
                }
            }
            SurfaceShape::Polygon { vertices } => {
                if vertices.len() < 3 {
                    diagnostics.push(
                        TrackValidationError::error(
                            "ERR_INVALID_POLYGON_ZONE",
                            format!("Surface zone #{} '{}' has fewer than 3 vertices.", i + 1, zone.name),
                        )
                        .with_index(i),
                    );
                }
            }
        }
    }

    // 9. Jump Ramp Checks
    for (i, ramp) in track.geometry.jump_ramps.iter().enumerate() {
        if ramp.launch_speed < 3.0 {
            diagnostics.push(
                TrackValidationError::warning(
                    "WARN_LOW_RAMP_SPEED",
                    format!("Jump ramp #{} '{}' launch speed is low ({:.1} m/s).", i + 1, ramp.name, ramp.launch_speed),
                )
                .with_index(i),
            );
        }
        if ramp.height <= 0.0 {
            diagnostics.push(
                TrackValidationError::warning(
                    "WARN_ZERO_RAMP_HEIGHT",
                    format!("Jump ramp #{} '{}' height is {:.1}m.", i + 1, ramp.name, ramp.height),
                )
                .with_index(i),
            );
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::geometry::{BarrierType, Obstacle, WallBarrier};
    use crate::track::presets::classic_grand_prix;
    use crate::track::spline::TrackWaypoint;
    use glam::Vec2;

    #[test]
    fn test_preset_track_validation_passes_cleanly() {
        let track = classic_grand_prix();
        let diags = validate_track(&track);
        let errors: Vec<_> = diags.iter().filter(|d| d.severity == ValidationSeverity::Error).collect();
        assert!(errors.is_empty(), "Preset track should have 0 validation errors: {:?}", errors);
    }

    #[test]
    fn test_insufficient_waypoints_detected() {
        let mut track = classic_grand_prix();
        track.spline.waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(50.0, 0.0), 12.0),
        ];
        let diags = validate_track(&track);
        assert!(diags.iter().any(|d| d.code == "ERR_INSUFFICIENT_WAYPOINTS"));
    }

    #[test]
    fn test_missing_finish_line_detected() {
        let mut track = classic_grand_prix();
        for cp in &mut track.checkpoints {
            cp.is_finish_line = false;
        }
        let diags = validate_track(&track);
        assert!(diags.iter().any(|d| d.code == "ERR_NO_FINISH_LINE"));
    }

    #[test]
    fn test_overlapping_grid_slots_detected() {
        let mut track = classic_grand_prix();
        if track.grid_positions.len() >= 2 {
            track.grid_positions[1].position = track.grid_positions[0].position;
            let diags = validate_track(&track);
            assert!(diags.iter().any(|d| d.code == "ERR_OVERLAPPING_GRID_SLOTS"));
        }
    }

    #[test]
    fn test_wall_crossing_track_centerline_detected() {
        let mut track = classic_grand_prix();
        // Insert a wall across the main straight at same elevation
        let blocking_wall = WallBarrier::with_elevation(
            Vec2::new(50.0, -10.0),
            Vec2::new(50.0, 10.0),
            BarrierType::Concrete,
            0.0,
        );
        track.geometry.inner_walls.push(blocking_wall);

        let diags = validate_track(&track);
        assert!(
            diags.iter().any(|d| d.code == "ERR_WALL_CROSSES_TRACK"),
            "Validation must detect wall crossing track centerline: {:?}",
            diags
        );
    }

    #[test]
    fn test_wall_elevated_overpass_bridge_allowed() {
        let mut track = classic_grand_prix();
        // Insert an elevated wall crossing over the ground straight with 5.0m elevation
        let bridge_wall = WallBarrier::with_elevation(
            Vec2::new(50.0, -10.0),
            Vec2::new(50.0, 10.0),
            BarrierType::Concrete,
            5.0,
        );
        track.geometry.inner_walls.push(bridge_wall);

        let diags = validate_track(&track);
        assert!(
            !diags.iter().any(|d| d.code == "ERR_WALL_CROSSES_TRACK"),
            "Elevated wall (5m clearance) should not trigger ERR_WALL_CROSSES_TRACK: {:?}",
            diags
        );
    }

    #[test]
    fn test_wall_self_intersection_detected() {
        let mut track = classic_grand_prix();
        let wall1 = WallBarrier::with_elevation(
            Vec2::new(100.0, 50.0),
            Vec2::new(120.0, 50.0),
            BarrierType::Armco,
            0.0,
        );
        let wall2 = WallBarrier::with_elevation(
            Vec2::new(110.0, 40.0),
            Vec2::new(110.0, 60.0),
            BarrierType::Armco,
            0.0,
        );
        track.geometry.inner_walls.push(wall1);
        track.geometry.inner_walls.push(wall2);

        let diags = validate_track(&track);
        assert!(
            diags.iter().any(|d| d.code == "ERR_WALL_SELF_INTERSECTION"),
            "Validation must detect intersecting wall barriers: {:?}",
            diags
        );
    }

    #[test]
    fn test_obstacle_on_track_detected() {
        let mut track = classic_grand_prix();
        let obs = Obstacle::circle(99, Vec2::new(0.0, 0.0), 2.0, "Dangerous Barrel");
        track.geometry.obstacles.push(obs);

        let diags = validate_track(&track);
        assert!(
            diags.iter().any(|d| d.code == "ERR_OBSTACLE_ON_TRACK"),
            "Validation must detect obstacle on track: {:?}",
            diags
        );
    }
}

