//! Bakes a "source" circuit (metadata + waypoints, as written by the OSM importer) into a full circuit.
//!
//! See `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md` §2.7. Baking fills the derived
//! data (spline samples, walls, boundary polylines, checkpoints, starting grid, default runoff). By default only
//! empty parts are filled, so hand edits in an already baked file are kept; `rebuild` regenerates them all.

use super::geometry::BarrierType;
use super::presets::{generate_checkpoints, generate_walls_from_spline};
use super::spline::TrackSpline;
use super::Track;

/// Settings for the derived data.
#[derive(Debug, Clone, Copy)]
pub struct BakeOptions {
    /// Regenerate every derived part, not only the empty ones.
    pub rebuild: bool,
    /// Wall distance from the road edge (m) where a waypoint does not set its own.
    /// `None`: keep the distance of the track's current walls, or 4.0 m when it has none (the editor default).
    pub barrier_offset: Option<f32>,
    /// `None`: keep the most common type of the track's current walls, or `Steel` when it has none.
    pub barrier_type: Option<BarrierType>,
    pub checkpoint_count: usize,
    pub sector_count: usize,
}

impl Default for BakeOptions {
    fn default() -> Self {
        Self {
            rebuild: false,
            barrier_offset: None,
            barrier_type: None,
            checkpoint_count: 20,
            sector_count: 3,
        }
    }
}

/// Which derived parts a bake generated.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BakeReport {
    pub spline: bool,
    pub walls: bool,
    pub checkpoints: bool,
    pub grid: bool,
}

/// Bakes `track` in place.
pub fn bake(track: &mut Track, opts: &BakeOptions) -> Result<BakeReport, String> {
    if track.spline.waypoints.len() < 3 {
        return Err(format!(
            "'{}' has {} waypoints; at least 3 are needed",
            track.name,
            track.spline.waypoints.len()
        ));
    }
    let mut report = BakeReport::default();

    // Read the current wall setup before the spline is regenerated from new waypoints.
    let has_walls = !track.geometry.inner_walls.is_empty() || !track.geometry.outer_walls.is_empty();
    let barrier_offset = opts.barrier_offset.unwrap_or_else(|| {
        if has_walls && !track.spline.samples.is_empty() {
            (track.effective_barrier_offset() * 10.0).round() / 10.0
        } else {
            4.0
        }
    });
    let barrier_type = opts
        .barrier_type
        .or_else(|| track.dominant_barrier_type())
        .unwrap_or(BarrierType::Steel);

    track.apply_default_runoff_surfaces();
    if opts.rebuild || track.spline.samples.is_empty() {
        track.spline = TrackSpline::new(track.spline.waypoints.clone(), track.spline.closed);
        report.spline = true;
    }
    let geometry = &mut track.geometry;
    let no_walls = geometry.inner_walls.is_empty()
        && geometry.outer_walls.is_empty()
        && geometry.left_boundary_polyline.is_empty()
        && geometry.right_boundary_polyline.is_empty();
    if opts.rebuild || no_walls {
        let (left_walls, right_walls, left_poly, right_poly) =
            generate_walls_from_spline(&track.spline, barrier_offset, barrier_type);
        geometry.inner_walls = left_walls;
        geometry.outer_walls = right_walls;
        geometry.left_boundary_polyline = left_poly;
        geometry.right_boundary_polyline = right_poly;
        report.walls = true;
    }
    if opts.rebuild || track.checkpoints.is_empty() {
        track.checkpoints = generate_checkpoints(&track.spline, opts.checkpoint_count, opts.sector_count);
        report.checkpoints = true;
    }
    if opts.rebuild || track.grid_positions.is_empty() {
        if opts.rebuild {
            track.grid_positions.clear();
        }
        if !track.auto_generate_grid_default() {
            return Err(format!("'{}': cannot place a starting grid (no finish line)", track.name));
        }
        report.grid = true;
    }
    track.apply_default_runoff_surfaces();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::validation::{validate_track, ValidationSeverity};

    /// Monza reduced to what the importer writes: metadata and waypoints.
    fn source_monza() -> Track {
        let mut track = crate::track::test_circuit("gt", "monza");
        let waypoints = track.spline.waypoints.clone();
        track.spline = TrackSpline { waypoints, closed: true, ..Default::default() };
        track.geometry = Default::default();
        track.checkpoints.clear();
        track.grid_positions.clear();
        track
    }

    #[test]
    fn test_bake_fills_a_source_circuit() {
        let mut track = source_monza();
        let report = bake(&mut track, &BakeOptions::default()).unwrap();
        assert_eq!(report, BakeReport { spline: true, walls: true, checkpoints: true, grid: true });
        assert!(track.spline.samples.len() > track.spline.waypoints.len());
        assert!(!track.geometry.inner_walls.is_empty() && !track.geometry.outer_walls.is_empty());
        assert!(track.has_finish_line());
        assert_eq!(track.grid_positions.len(), track.default_grid_count());
        let errors: Vec<_> = validate_track(&track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .collect();
        assert!(errors.is_empty(), "{:?}", errors);
    }

    #[test]
    fn test_bake_keeps_existing_parts_unless_rebuild() {
        let baked = crate::track::test_circuit("gt", "monza");
        let mut track = baked.clone();
        let report = bake(&mut track, &BakeOptions::default()).unwrap();
        assert_eq!(report, BakeReport::default());
        assert_eq!(track.geometry.inner_walls, baked.geometry.inner_walls);

        let mut rebuilt = baked.clone();
        let opts = BakeOptions { rebuild: true, ..Default::default() };
        let report = bake(&mut rebuilt, &opts).unwrap();
        assert_eq!(report, BakeReport { spline: true, walls: true, checkpoints: true, grid: true });
    }

    #[test]
    fn test_rebuild_keeps_the_wall_setup_of_the_track() {
        let baked = crate::track::test_circuit("nascar", "daytona_superspeedway");
        let (offset, wall_type) = (baked.effective_barrier_offset(), baked.dominant_barrier_type());
        let mut rebuilt = baked.clone();
        bake(&mut rebuilt, &BakeOptions { rebuild: true, ..Default::default() }).unwrap();
        assert_eq!(rebuilt.dominant_barrier_type(), wall_type);
        assert!((rebuilt.effective_barrier_offset() - offset).abs() < 0.2, "{} vs {}", rebuilt.effective_barrier_offset(), offset);
    }

    #[test]
    fn test_bake_rejects_too_few_waypoints() {
        let mut track = source_monza();
        track.spline.waypoints.truncate(2);
        assert!(bake(&mut track, &BakeOptions::default()).is_err());
    }
}
