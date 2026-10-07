//! Bakes a "source" circuit (metadata + waypoints, as written by the OSM importer) into a full circuit.
//!
//! See `specs/042_jsononly_official_circuit_catalog_and_embedded_track_data.md` §2.7. Baking fills the derived
//! data (spline samples, walls, boundary polylines, checkpoints, starting grid, default runoff). By default only
//! empty parts are filled, so hand edits in an already baked file are kept; `rebuild` regenerates them all.

use super::geometry::BarrierType;
use super::presets::{
    adaptive_checkpoint_count, generate_checkpoints, generate_walls_from_spline,
    generate_walls_from_spline_raw,
};
use super::spline::TrackSpline;
use super::Track;

/// Settings for the derived data.
#[derive(Debug, Clone, Copy)]
pub struct BakeOptions {
    /// Regenerate every derived part, not only the empty ones.
    pub rebuild: bool,
    /// Merge collinear wall segments to reduce barrier count (default: true).
    pub merge_walls: bool,
    /// Wall distance from the road edge (m) where a waypoint does not set its own.
    /// `None`: keep the distance of the track's current walls, or 4.0 m when it has none (the editor default).
    pub barrier_offset: Option<f32>,
    /// `None`: keep the most common type of the track's current walls, or `Steel` when it has none.
    pub barrier_type: Option<BarrierType>,
    /// `None`: keep the track's current checkpoint count, or 20 when it has none.
    pub checkpoint_count: Option<usize>,
    /// `None`: keep the track's current sector count, or 3 when it has no checkpoints.
    pub sector_count: Option<usize>,
    /// Regenerate checkpoints with adaptive length/speed count instead of preserving current count.
    pub adaptive_checkpoints: bool,
}

impl Default for BakeOptions {
    fn default() -> Self {
        Self {
            rebuild: false,
            merge_walls: true,
            barrier_offset: None,
            barrier_type: None,
            checkpoint_count: None,
            sector_count: None,
            adaptive_checkpoints: false,
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
    let barrier_offset = opts
        .barrier_offset
        .or_else(|| has_walls.then(|| current_wall_offset(track)).flatten())
        .unwrap_or(4.0);
    let barrier_type = opts
        .barrier_type
        .or_else(|| track.dominant_barrier_type())
        .unwrap_or(BarrierType::Steel);
    track.apply_default_runoff_surfaces();
    if opts.rebuild || track.spline.samples.is_empty() {
        track.spline = TrackSpline::new(track.spline.waypoints.clone(), track.spline.closed);
        report.spline = true;
    }
    let checkpoint_count = opts
        .checkpoint_count
        .unwrap_or_else(|| {
            if opts.adaptive_checkpoints || track.checkpoints.is_empty() {
                adaptive_checkpoint_count(&track.spline)
            } else {
                track.checkpoints.len()
            }
        });
    let sector_count = opts
        .sector_count
        .or_else(|| track.checkpoints.iter().map(|c| c.sector + 1).max())
        .unwrap_or(3);
    let grid_layout = current_grid_layout(track);
    if let Some(lane) = &mut track.pit_lane {
        if opts.rebuild || lane.spline.samples.is_empty() {
            lane.spline = TrackSpline::new(lane.spline.waypoints.clone(), false);
        }
    }
    let geometry = &mut track.geometry;
    let no_walls = geometry.inner_walls.is_empty()
        && geometry.outer_walls.is_empty()
        && geometry.left_boundary_polyline.is_empty()
        && geometry.right_boundary_polyline.is_empty();
    if opts.rebuild || no_walls {
        let (left_walls, right_walls, left_poly, right_poly) = if opts.merge_walls {
            generate_walls_from_spline(&track.spline, barrier_offset, barrier_type)
        } else {
            generate_walls_from_spline_raw(&track.spline, barrier_offset, barrier_type)
        };
        geometry.inner_walls = left_walls;
        geometry.outer_walls = right_walls;
        geometry.left_boundary_polyline = left_poly;
        geometry.right_boundary_polyline = right_poly;
        if track.pit_lane.is_some() {
            track.trim_walls_for_pit_lane();
            track.generate_pit_lane_walls();
        }
        report.walls = true;
    } else {
        if track.pit_lane.is_some() {
            track.trim_walls_for_pit_lane();
            track.generate_pit_lane_walls();
        }
    }
    if track.network.is_some() {
        track.trim_walls_for_network();
    }
    // Network checkpoints carry segment ids and the joker checkpoint; the main spline cannot regenerate them.
    if (opts.rebuild && track.network.is_none()) || track.checkpoints.is_empty() {
        track.checkpoints = generate_checkpoints(&track.spline, checkpoint_count, sector_count);
        report.checkpoints = true;
    }
    if opts.rebuild || track.grid_positions.is_empty() {
        // Keeps the current grid size and slot layout, or the module defaults when there is no grid.
        let placed = match grid_layout {
            Some((spacing, stagger)) => track.auto_generate_grid(track.grid_positions.len(), spacing, stagger),
            None => track.auto_generate_grid_default(),
        };
        if !placed {
            return Err(format!("'{}': cannot place a starting grid (no finish line)", track.name));
        }
        report.grid = true;
    }
    track.apply_default_runoff_surfaces();
    if let Some(ref mut net) = track.network {
        net.recompute_composite_splines();
    }
    track.geometry.recompute_scenery_obstacles();
    Ok(report)
}

/// The default wall distance the current walls were built with.
///
/// The wall generator puts each wall vertex (a segment start) at `width / 2 + offset` from its sample, along the
/// normal, so where no waypoint sets its own wall distance that gap is exactly the default offset. The most common
/// gap wins; waypoint distances, curves, bridges and removed crossing points only add scattered values.
fn current_wall_offset(track: &Track) -> Option<f32> {
    let samples = &track.spline.samples;
    if samples.is_empty() {
        return None;
    }
    let mut counts: Vec<(i32, usize)> = Vec::new();
    let geometry = &track.geometry;
    for walls in [&geometry.inner_walls, &geometry.outer_walls] {
        for v in walls.iter().map(|w| &w.segment.start) {
            let s = samples
                .iter()
                .min_by(|a, b| a.point.distance_squared(*v).total_cmp(&b.point.distance_squared(*v)))?;
            if s.is_bridge {
                continue;
            }
            let gap = ((s.point.distance(*v) - s.width * 0.5) * 20.0).round() as i32; // 0.05 m bins
            match counts.iter_mut().find(|(g, _)| *g == gap) {
                Some((_, n)) => *n += 1,
                None => counts.push((gap, 1)),
            }
        }
    }
    counts.into_iter().max_by_key(|(_, n)| *n).map(|(g, _)| g as f32 / 20.0).filter(|g| *g > 0.0)
}

/// (spacing, lateral stagger) of the current starting grid, measured along the current spline.
/// The grid generator puts slot `i` at `finish - 15 - i * spacing`, alternating `-stagger` / `+stagger`.
fn current_grid_layout(track: &Track) -> Option<(f32, f32)> {
    let grid = &track.grid_positions;
    if grid.len() < 3 || track.spline.samples.len() < 2 {
        return None;
    }
    // Slots 0 and 2k sit on the same side, so the lateral stagger does not skew their arc distance.
    let last_same_side = (grid.len() - 1) / 2 * 2;
    let total = track.spline.total_length();
    let p0 = track.spline.project_point(grid[0].position);
    let pk = track.spline.project_point(grid[last_same_side].position);
    let spacing = (p0.progress_distance - pk.progress_distance).rem_euclid(total) / last_same_side as f32;
    let stagger = p0.lateral_offset.abs();
    (spacing > 0.1 && spacing * (last_same_side as f32) < total * 0.5)
        .then(|| ((spacing * 10.0).round() / 10.0, (stagger * 10.0).round() / 10.0))
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
    fn test_rebuild_keeps_checkpoint_and_grid_counts() {
        let baked = crate::track::test_circuit("nascar", "daytona_superspeedway");
        let mut rebuilt = baked.clone();
        bake(&mut rebuilt, &BakeOptions { rebuild: true, ..Default::default() }).unwrap();
        assert_eq!(rebuilt.checkpoints.len(), baked.checkpoints.len());
        assert_eq!(rebuilt.grid_positions.len(), baked.grid_positions.len());
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

    #[test]
    fn test_adaptive_checkpoint_bake_on_short_circuit() {
        let mut track = crate::track::test_circuit("classic", "kart_pine_grove");
        let opts = BakeOptions {
            rebuild: true,
            adaptive_checkpoints: true,
            ..Default::default()
        };
        bake(&mut track, &opts).unwrap();
        assert!(
            track.checkpoints.len() >= 8 && track.checkpoints.len() <= 10,
            "Pine grove checkpoints count must be in [8, 10], got {}",
            track.checkpoints.len()
        );
        for window in track.checkpoints.windows(2) {
            let spacing = window[1].target_distance - window[0].target_distance;
            assert!(
                spacing >= 20.0 && spacing <= 70.0,
                "Pine grove checkpoint spacing should be ~25-65m, got {:.2}m",
                spacing
            );
        }
    }
}

