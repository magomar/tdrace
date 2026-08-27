use serde::{Deserialize, Serialize};

use super::Track;
use crate::track::geometry::SurfaceShape;

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

    // 2. Checkpoint Checks
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
    }

    // 3. Starting Grid Checks
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
    }

    // 4. Surface Zone & Hazard Checks
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

    // 5. Jump Ramp Checks
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
}
