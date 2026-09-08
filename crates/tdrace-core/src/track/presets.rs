use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::checkpoint::Checkpoint;
use super::geometry::{
    BarrierType, JumpRamp, LineSegment, SpawnPose, SurfaceShape, SurfaceZone,
    TrackGeometry, WallBarrier,
};
use super::spline::{TrackSpline, TrackWaypoint};
use super::{Track, TrackCategory};
use crate::physics::surface::SurfaceType;

/// Trims local self-intersecting loops (swallowtail singularities) from an offset boundary polyline.
pub fn untangle_polyline(pts: &mut Vec<Vec2>, closed: bool) {
    let mut changed = true;
    let mut passes = 0;
    // Local swallowtail singularities on sharp corners can span up to ~30-40 consecutive samples on 1000m+ tracks.
    let max_loop_span = 40;

    while changed && passes < 16 {
        changed = false;
        passes += 1;
        let n = pts.len();
        if n < 4 {
            break;
        }

        'outer: for i in 0..n {
            let p0 = pts[i];
            let next_i = (i + 1) % n;
            let p1 = pts[next_i];
            let seg_a = LineSegment::new(p0, p1);

            let max_span = max_loop_span.min(n / 2);
            for span in 2..=max_span {
                let j = (i + span) % n;
                if !closed && i + span >= n {
                    break;
                }
                let next_j = (j + 1) % n;
                if !closed && j + 1 >= n {
                    break;
                }
                if next_j == i || next_i == j {
                    continue;
                }

                let p2 = pts[j];
                let p3 = pts[next_j];
                let seg_b = LineSegment::new(p2, p3);

                if let Some(hit) = seg_a.intersect_segment(&seg_b) {
                    // Local loop between i and j (span samples)
                    if j > i {
                        let mut new_pts = Vec::with_capacity(n);
                        for k in 0..=i {
                            new_pts.push(pts[k]);
                        }
                        new_pts.push(hit);
                        for k in (j + 1)..n {
                            new_pts.push(pts[k]);
                        }
                        *pts = new_pts;
                        changed = true;
                        break 'outer;
                    } else if closed {
                        // Loop wraps around the array boundary (j < i)
                        let mut new_pts = Vec::with_capacity(n);
                        new_pts.push(hit);
                        for k in (j + 1)..=i {
                            new_pts.push(pts[k]);
                        }
                        *pts = new_pts;
                        changed = true;
                        break 'outer;
                    }
                }
            }
        }
    }
}

/// Trims or removes wall barrier segments that intersect or fall inside non-local drivable road corridors at the same elevation.
pub fn trim_walls_at_crossings(walls: &mut Vec<(WallBarrier, f32)>, spline: &TrackSpline) -> Vec<WallBarrier> {
    if walls.is_empty() || spline.samples.len() < 2 {
        return walls.drain(..).map(|(w, _)| w).collect();
    }

    let total_len = spline.total_length();
    let min_loop_dist = (total_len * 0.25).min(30.0).max(15.0);
    let n_samples = spline.samples.len();
    let n_segs = if spline.closed { n_samples } else { n_samples - 1 };

    let mut result_walls = Vec::with_capacity(walls.len());

    for (wall, wall_dist) in walls.drain(..) {
        let mut segments_to_process = vec![wall.segment];

        for j in 0..n_segs {
            let next_j = (j + 1) % n_samples;
            let s0 = &spline.samples[j];
            let s1 = &spline.samples[next_j];

            let seg_elev = (s0.elevation + s1.elevation) * 0.5;
            let elev_diff = (wall.elevation - seg_elev).abs();
            if elev_diff >= 2.5 {
                continue; // Overpass bridge or underpass
            }

            let seg_dist = s0.distance;
            let arc_dist = if spline.closed {
                let d = (wall_dist - seg_dist).abs();
                d.min(total_len - d)
            } else {
                (wall_dist - seg_dist).abs()
            };

            if arc_dist < min_loop_dist {
                continue; // Local segment - skip
            }

            let center_seg = LineSegment::new(s0.point, s1.point);
            let center_len_sq = center_seg.length_squared();
            if center_len_sq < 1e-4 {
                continue;
            }

            let curb0 = if s0.left_curb || s0.right_curb { 1.35 } else { 0.0 };
            let curb1 = if s1.left_curb || s1.right_curb { 1.35 } else { 0.0 };
            let hw0 = s0.width * 0.5 + curb0 + 0.15;
            let hw1 = s1.width * 0.5 + curb1 + 0.15;
            let max_hw = hw0.max(hw1);

            let seg_center = (s0.point + s1.point) * 0.5;
            let seg_radius = center_len_sq.sqrt() * 0.5 + max_hw;

            let left_edge = LineSegment::new(s0.point + s0.normal * hw0, s1.point + s1.normal * hw1);
            let right_edge = LineSegment::new(s0.point - s0.normal * hw0, s1.point - s1.normal * hw1);

            let mut next_processed = Vec::new();

            for seg in segments_to_process {
                let seg_mid = (seg.start + seg.end) * 0.5;
                let seg_radius_w = seg.length() * 0.5;
                let dist_sq = (seg_mid - seg_center).length_squared();
                let threshold = seg_radius + seg_radius_w;

                if dist_sq > threshold * threshold {
                    // Spatially far away - keep segment without detailed intersection tests
                    next_processed.push(seg);
                    continue;
                }

                // Helper to test if a point is inside road ribbon of segment j
                let point_in_ribbon = |p: Vec2| -> bool {
                    let ap = p - s0.point;
                    let ab = s1.point - s0.point;
                    let t = ap.dot(ab) / center_len_sq;
                    if t < -0.05 || t > 1.05 {
                        return false;
                    }
                    let t_clamped = t.clamp(0.0, 1.0);
                    let proj_pt = s0.point + ab * t_clamped;
                    let local_hw = hw0 + (hw1 - hw0) * t_clamped;
                    (p - proj_pt).length_squared() <= (local_hw * local_hw)
                };

                let p0_inside = point_in_ribbon(seg.start);
                let p1_inside = point_in_ribbon(seg.end);

                let hit_left = seg.intersect_segment(&left_edge);
                let hit_right = seg.intersect_segment(&right_edge);
                let hit_center = seg.intersect_segment(&center_seg);

                if p0_inside && p1_inside {
                    continue;
                }

                if !p0_inside && !p1_inside {
                    let mut hits = Vec::new();
                    if let Some(h) = hit_left { hits.push(h); }
                    if let Some(h) = hit_right { hits.push(h); }

                    if hits.len() >= 2 {
                        let dir = seg.direction();
                        hits.sort_by(|a, b| {
                            let da = (*a - seg.start).dot(dir);
                            let db = (*b - seg.start).dot(dir);
                            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        let h1 = hits[0];
                        let h2 = hits[hits.len() - 1];
                        if (h1 - seg.start).length() > 0.15 {
                            next_processed.push(LineSegment::new(seg.start, h1));
                        }
                        if (seg.end - h2).length() > 0.15 {
                            next_processed.push(LineSegment::new(h2, seg.end));
                        }
                    } else if hit_center.is_some() && !hits.is_empty() {
                        let h = hits[0];
                        let mid = (seg.start + seg.end) * 0.5;
                        if point_in_ribbon(mid) {
                            if (h - seg.start).length() > 0.15 {
                                next_processed.push(LineSegment::new(seg.start, h));
                            }
                        } else {
                            next_processed.push(seg);
                        }
                    } else if hit_center.is_some() && point_in_ribbon(seg_mid) {
                        continue;
                    } else {
                        next_processed.push(seg);
                    }
                } else if !p0_inside && p1_inside {
                    let hit = hit_left.or(hit_right).unwrap_or_else(|| {
                        let mut low = 0.0f32;
                        let mut high = 1.0f32;
                        for _ in 0..8 {
                            let mid_t = (low + high) * 0.5;
                            let pt = seg.start.lerp(seg.end, mid_t);
                            if point_in_ribbon(pt) { high = mid_t; } else { low = mid_t; }
                        }
                        seg.start.lerp(seg.end, low)
                    });
                    if (hit - seg.start).length() > 0.15 {
                        next_processed.push(LineSegment::new(seg.start, hit));
                    }
                } else {
                    let hit = hit_left.or(hit_right).unwrap_or_else(|| {
                        let mut low = 0.0f32;
                        let mut high = 1.0f32;
                        for _ in 0..8 {
                            let mid_t = (low + high) * 0.5;
                            let pt = seg.start.lerp(seg.end, mid_t);
                            if point_in_ribbon(pt) { low = mid_t; } else { high = mid_t; }
                        }
                        seg.start.lerp(seg.end, high)
                    });
                    if (seg.end - hit).length() > 0.15 {
                        next_processed.push(LineSegment::new(hit, seg.end));
                    }
                }
            }
            segments_to_process = next_processed;
        }

        for seg in segments_to_process {
            if seg.length() > 0.10 {
                result_walls.push(WallBarrier {
                    segment: seg,
                    restitution: wall.restitution,
                    friction: wall.friction,
                    barrier_type: wall.barrier_type,
                    elevation: wall.elevation,
                });
            }
        }
    }

    result_walls
}

/// Trims clashing non-local wall corners where approaching walls intersect each other at crossroads.
pub fn trim_corner_intersections(
    left_walls: &mut Vec<WallBarrier>,
    right_walls: &mut Vec<WallBarrier>,
    spline: &TrackSpline,
) {
    let total_len = spline.total_length();
    let min_loop_dist = (total_len * 0.25).min(30.0).max(15.0);

    for _ in 0..4 {
        let mut modified = false;
        let n_left = left_walls.len();
        let n_right = right_walls.len();
        let total_w = n_left + n_right;

        'outer_pair: for i in 0..total_w {
            for j in (i + 1)..total_w {
                let (seg_a, elev_a) = if i < n_left {
                    (left_walls[i].segment, left_walls[i].elevation)
                } else {
                    (right_walls[i - n_left].segment, right_walls[i - n_left].elevation)
                };

                let (seg_b, elev_b) = if j < n_left {
                    (left_walls[j].segment, left_walls[j].elevation)
                } else {
                    (right_walls[j - n_left].segment, right_walls[j - n_left].elevation)
                };

                if (elev_a - elev_b).abs() >= 2.5 {
                    continue;
                }
                // Skip connected segments sharing endpoints
                if (seg_a.start - seg_b.start).length_squared() < 0.01
                    || (seg_a.start - seg_b.end).length_squared() < 0.01
                    || (seg_a.end - seg_b.start).length_squared() < 0.01
                    || (seg_a.end - seg_b.end).length_squared() < 0.01
                {
                    continue;
                }

                if let Some(hit) = seg_a.intersect_segment(&seg_b) {
                    let mid_a = (seg_a.start + seg_a.end) * 0.5;
                    let mid_b = (seg_b.start + seg_b.end) * 0.5;
                    let proj_a = spline.project_point(mid_a);
                    let proj_b = spline.project_point(mid_b);
                    let d = (proj_a.progress_distance - proj_b.progress_distance).abs();
                    let arc_dist = if spline.closed { d.min(total_len - d) } else { d };
                    if arc_dist < min_loop_dist {
                        continue;
                    }

                    // Trim both segments at `hit`
                    let d0_a = (seg_a.start - proj_b.closest_point).length_squared();
                    let d1_a = (seg_a.end - proj_b.closest_point).length_squared();
                    let new_seg_a = if d0_a >= d1_a {
                        LineSegment::new(seg_a.start, hit)
                    } else {
                        LineSegment::new(hit, seg_a.end)
                    };

                    let d0_b = (seg_b.start - proj_a.closest_point).length_squared();
                    let d1_b = (seg_b.end - proj_a.closest_point).length_squared();
                    let new_seg_b = if d0_b >= d1_b {
                        LineSegment::new(seg_b.start, hit)
                    } else {
                        LineSegment::new(hit, seg_b.end)
                    };

                    if i < n_left {
                        left_walls[i].segment = new_seg_a;
                    } else {
                        right_walls[i - n_left].segment = new_seg_a;
                    }

                    if j < n_left {
                        left_walls[j].segment = new_seg_b;
                    } else {
                        right_walls[j - n_left].segment = new_seg_b;
                    }

                    modified = true;
                    break 'outer_pair;
                }
            }
        }
        if !modified {
            break;
        }
    }

    left_walls.retain(|w| w.segment.length() > 0.10);
    right_walls.retain(|w| w.segment.length() > 0.10);
}

/// Builds boundary wall barriers along the track edges given a spline and barrier offset.
pub fn generate_walls_from_spline(
    spline: &TrackSpline,
    barrier_offset: f32,
    barrier_type: BarrierType,
) -> (Vec<WallBarrier>, Vec<WallBarrier>, Vec<Vec2>, Vec<Vec2>) {
    let n = spline.samples.len();
    if n < 2 {
        return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    }

    let mut left_pts = Vec::with_capacity(n);
    let mut right_pts = Vec::with_capacity(n);

    for s in &spline.samples {
        let elev_factor = (s.elevation / 3.0).clamp(0.0, 1.0);
        let curb_extra = if s.left_curb || s.right_curb { 1.35 } else { 0.75 };
        let bridge_offset = curb_extra + 0.50;

        let left_base = s.left_wall_distance.unwrap_or(barrier_offset);
        let right_base = s.right_wall_distance.unwrap_or(barrier_offset);

        let left_offset = left_base * (1.0 - elev_factor) + bridge_offset * elev_factor;
        let right_offset = right_base * (1.0 - elev_factor) + bridge_offset * elev_factor;

        let left_half_w = s.width * 0.5 + left_offset;
        let right_half_w = s.width * 0.5 + right_offset;
        left_pts.push(s.point + s.normal * left_half_w);
        right_pts.push(s.point - s.normal * right_half_w);
    }

    untangle_polyline(&mut left_pts, spline.closed);
    untangle_polyline(&mut right_pts, spline.closed);

    let mut raw_left_walls = Vec::new();
    let mut raw_right_walls = Vec::new();

    let seg_count_left = if spline.closed { left_pts.len() } else { left_pts.len().saturating_sub(1) };
    for i in 0..seg_count_left {
        let next_i = (i + 1) % left_pts.len();
        let s_curr = &spline.samples[i % n];
        let s_next = &spline.samples[next_i % n];
        if s_curr.left_wall && s_next.left_wall {
            let elev = (s_curr.elevation + s_next.elevation) * 0.5;
            let b_type = s_curr.wall_type.or(s_next.wall_type).unwrap_or(barrier_type);
            raw_left_walls.push((
                WallBarrier::with_elevation(left_pts[i], left_pts[next_i], b_type, elev),
                s_curr.distance,
            ));
        }
    }

    let seg_count_right = if spline.closed { right_pts.len() } else { right_pts.len().saturating_sub(1) };
    for i in 0..seg_count_right {
        let next_i = (i + 1) % right_pts.len();
        let s_curr = &spline.samples[i % n];
        let s_next = &spline.samples[next_i % n];
        if s_curr.right_wall && s_next.right_wall {
            let elev = (s_curr.elevation + s_next.elevation) * 0.5;
            let b_type = s_curr.wall_type.or(s_next.wall_type).unwrap_or(barrier_type);
            raw_right_walls.push((
                WallBarrier::with_elevation(
                    right_pts[i],
                    right_pts[next_i],
                    b_type,
                    elev,
                ),
                s_curr.distance,
            ));
        }
    }

    let mut left_walls = trim_walls_at_crossings(&mut raw_left_walls, spline);
    let mut right_walls = trim_walls_at_crossings(&mut raw_right_walls, spline);
    trim_corner_intersections(&mut left_walls, &mut right_walls, spline);

    (left_walls, right_walls, left_pts, right_pts)
}

/// Generates a sequence of checkpoints distributed along the track spline.
pub fn generate_checkpoints(
    spline: &TrackSpline,
    count: usize,
    num_sectors: usize,
) -> Vec<Checkpoint> {
    let mut checkpoints = Vec::with_capacity(count);
    let total_len = spline.total_length();
    let sectors = num_sectors.max(1);

    for i in 0..count {
        let dist = (i as f32 / count as f32) * total_len;
        let sample = spline.sample_at_distance(dist);
        let half_w = sample.width * 0.5 + 4.0; // gate extends slightly beyond track edge

        let gate_left = sample.point + sample.normal * half_w;
        let gate_right = sample.point - sample.normal * half_w;

        let sector = (i * sectors) / count;
        let is_finish = i == 0;

        let mut cp = Checkpoint::new(
            i,
            LineSegment::new(gate_left, gate_right),
            sample.tangent,
            sector,
            is_finish,
        );
        cp.target_distance = dist;
        cp.elevation = sample.elevation;
        checkpoints.push(cp);
    }

    checkpoints
}

/// Generates starting grid spawn positions on the main straight before the start line.
pub fn generate_grid_positions(
    spline: &TrackSpline,
    num_slots: usize,
    spacing: f32,
    stagger_lateral: f32,
) -> Vec<SpawnPose> {
    generate_grid_positions_at_distance(spline, spline.total_length(), num_slots, spacing, stagger_lateral)
}

/// Generates starting grid spawn positions at a specific track distance before a finish/start line.
pub fn generate_grid_positions_at_distance(
    spline: &TrackSpline,
    finish_dist: f32,
    num_slots: usize,
    spacing: f32,
    stagger_lateral: f32,
) -> Vec<SpawnPose> {
    let mut slots = Vec::with_capacity(num_slots);

    for i in 0..num_slots {
        // Place slots behind start/finish line (at negative offset along spline)
        let slot_dist = finish_dist - 15.0 - (i as f32 * spacing);
        let sample = spline.sample_at_distance(slot_dist);

        let lateral_stagger = if i % 2 == 0 {
            -stagger_lateral
        } else {
            stagger_lateral
        };

        let right_vec = Vec2::new(sample.tangent.y, -sample.tangent.x);
        let pos = sample.point + right_vec * lateral_stagger;
        let angle = sample.tangent.y.atan2(sample.tangent.x);

        slots.push(SpawnPose::new(pos, angle, i));
    }

    slots
}

/// Preset 1: Classic Grand Prix Circuit
/// Flowing corners, high-speed chicane, apex curbs, asphalt runoff, hairpin sand trap, pit lane.
pub fn classic_grand_prix() -> Track {
    let waypoints = vec![
        // Main Straight & Start/Finish
        TrackWaypoint::new(Vec2::new(70.0, 0.0), 14.0),
        TrackWaypoint::new(Vec2::new(120.0, 0.0), 14.0),
        // Turn 1 & 2 High-Speed Chicane
        TrackWaypoint::new(Vec2::new(180.0, 30.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(230.0, -10.0), 13.0).with_curbs(false, true),
        // Sweeping Curve into Back Straight
        TrackWaypoint::new(Vec2::new(320.0, 40.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(380.0, 130.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(360.0, 240.0), 13.0).with_curbs(false, true),
        // Hairpin Turn
        TrackWaypoint::new(Vec2::new(280.0, 310.0), 12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(200.0, 310.0), 12.0).with_curbs(true, true),
        // Technical Esses & Infield
        TrackWaypoint::new(Vec2::new(140.0, 240.0), 12.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(80.0, 260.0), 12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(20.0, 200.0), 12.0).with_curbs(false, true),
        // Final Corner onto Main Straight
        TrackWaypoint::new(Vec2::new(-30.0, 100.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-20.0, 20.0), 14.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Armco);

    let mut surface_zones = Vec::new();
    // Sand trap outside the hairpin (around x: 200..290, y: 315..360)
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(180.0, 315.0),
            max: Vec2::new(300.0, 370.0),
        },
        SurfaceType::Sand,
        "Hairpin Sand Trap",
    ));

    // Asphalt runoff outside Turn 1
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(170.0, 35.0),
            max: Vec2::new(220.0, 75.0),
        },
        SurfaceType::Asphalt,
        "Turn 1 Runoff",
    ));

    let mut checkpoints = generate_checkpoints(&spline, 12, 3);
    // Add Pit Entry / Exit checkpoints
    let pit_entry_proj = spline.project_point(Vec2::new(-25.0, 0.0));
    let mut pit_entry = Checkpoint::new(
        100,
        LineSegment::new(Vec2::new(-25.0, -5.0), Vec2::new(-25.0, -18.0)),
        Vec2::new(1.0, 0.0),
        0,
        false,
    )
    .with_pit_flags(true, false);
    pit_entry.target_distance = pit_entry_proj.progress_distance;

    let pit_exit_proj = spline.project_point(Vec2::new(135.0, 0.0));
    let mut pit_exit = Checkpoint::new(
        101,
        LineSegment::new(Vec2::new(135.0, -5.0), Vec2::new(135.0, -18.0)),
        Vec2::new(1.0, 0.0),
        0,
        false,
    )
    .with_pit_flags(false, true);
    pit_exit.target_distance = pit_exit_proj.progress_distance;

    checkpoints.push(pit_entry);
    checkpoints.push(pit_exit);

    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Classic Grand Prix".to_string(),
        description: "High-speed sweeping chicanes, hairpin sand traps & tactical pit lane.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: Some(SurfaceShape::Aabb {
            min: Vec2::new(30.0, -16.0),
            max: Vec2::new(70.0, -8.0),
        }),
        default_laps: 3,
        predefined_car: Some("sports_car".to_string()),
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "f1".to_string()],
    }
}

/// Preset 2: Oval Speedway
/// High speed 2-turn oval with perimeter concrete walls, tight wall collisions, asphalt apron, and 22-degree banked turns.
pub fn oval_speedway() -> Track {
    let waypoints = vec![
        // Front Straight
        TrackWaypoint::new(Vec2::new(0.0, -60.0), 18.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(150.0, -60.0), 18.0).with_bank_angle(0.0),
        // Turn 1 & 2 (East Banked Curve)
        TrackWaypoint::new(Vec2::new(230.0, -25.0), 20.0).with_curbs(true, true).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(250.0, 30.0), 20.0).with_curbs(true, true).with_bank_angle(22.0),
        TrackWaypoint::new(Vec2::new(230.0, 85.0), 20.0).with_curbs(true, true).with_bank_angle(12.0),
        // Back Straight
        TrackWaypoint::new(Vec2::new(150.0, 120.0), 18.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(0.0, 120.0), 18.0).with_bank_angle(0.0),
        // Turn 3 & 4 (West Banked Curve)
        TrackWaypoint::new(Vec2::new(-80.0, 85.0), 20.0).with_curbs(true, true).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(-100.0, 30.0), 20.0).with_curbs(true, true).with_bank_angle(22.0),
        TrackWaypoint::new(Vec2::new(-80.0, -25.0), 20.0).with_curbs(true, true).with_bank_angle(12.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 12, 7.0, 3.0);

    Track {
        name: "Oval Speedway".to_string(),
        description: "Full-throttle banked superspeedway surrounded by concrete barriers.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        predefined_car: Some("sports_car".to_string()),
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string()],
    }
}

/// Preset: Dirty Oval Speedway
/// High-sliding dirt superspeedway oval with 18-degree banked curves and loose gravel cushion.
pub fn dirty_oval_speedway() -> Track {
    let waypoints = vec![
        // Front Straight
        TrackWaypoint::new(Vec2::new(22.5, -62.5), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(172.5, -62.5), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        // Turn 1 & 2 (East Banked Dirt Curve)
        TrackWaypoint::new(Vec2::new(252.5, -27.5), 20.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(272.5, 27.5), 20.0).with_surface(SurfaceType::Dirt).with_bank_angle(18.0),
        TrackWaypoint::new(Vec2::new(252.5, 82.5), 20.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
        // Back Straight
        TrackWaypoint::new(Vec2::new(172.5, 117.5), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(22.5, 117.5), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        // Turn 3 & 4 (West Banked Dirt Curve)
        TrackWaypoint::new(Vec2::new(-57.5, 82.5), 20.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(-77.5, 27.5), 20.0).with_surface(SurfaceType::Dirt).with_bank_angle(18.0),
        TrackWaypoint::new(Vec2::new(-57.5, -27.5), 20.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 12, 7.0, 3.0);

    Track {
        name: "Dirty Oval Speedway".to_string(),
        description: "High-sliding dirt superspeedway oval with 18-degree banked curves and loose gravel cushion.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 5,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Backwards compatibility alias for `dirty_oval_speedway`.
pub fn dirt_oval_speedway() -> Track {
    dirty_oval_speedway()
}

/// Preset 3: Drift Park
/// High drift circuit with sweeping corners, asphalt runoff, tire stacks and clipping cones.
pub fn drift_park() -> Track {
    let waypoints = vec![
        // Start straight
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
        TrackWaypoint::new(Vec2::new(70.0, 0.0), 12.0),
        // Turn 1 Sweeper Right
        TrackWaypoint::new(Vec2::new(120.0, 20.0), 14.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(140.0, 60.0), 14.0).with_curbs(true, true),
        // Turn 2 Left
        TrackWaypoint::new(Vec2::new(110.0, 100.0), 14.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(70.0, 110.0), 14.0).with_curbs(true, true),
        // Turn 3 Hairpin Left
        TrackWaypoint::new(Vec2::new(20.0, 90.0), 12.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(0.0, 60.0), 12.0).with_curbs(true, true),
        // Turn 4 S-Bend Right
        TrackWaypoint::new(Vec2::new(-30.0, 40.0), 13.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(-60.0, 20.0), 13.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(-40.0, -10.0), 12.0).with_curbs(true, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.0, BarrierType::TireWall);

    let surface_zones = vec![
        // Generous asphalt runoff on Turn 1 outer edge
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(130.0, 10.0),
                max: Vec2::new(165.0, 75.0),
            },
            SurfaceType::Asphalt,
            "Turn 1 Asphalt Runoff",
        ),
        // Asphalt runoff on Turn 3 Hairpin outer edge
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-5.0, 95.0),
                radius: 18.0,
            },
            SurfaceType::Asphalt,
            "Turn 3 Runoff Area",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 10, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Drift Park".to_string(),
        description: "Tight technical drift arena with sweeping corners and generous asphalt runoff.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        predefined_car: Some("drift_car".to_string()),
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "kart".to_string()],
    }
}

/// Preset 4: Kart Arena
/// Compact track with tight chicanes, 90-degree corners, and aggressive curbs.
pub fn kart_arena() -> Track {
    let waypoints = vec![
        // Start straight
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
        TrackWaypoint::new(Vec2::new(60.0, 0.0), 8.0),
        // Turn 1 90-degree right
        TrackWaypoint::new(Vec2::new(85.0, 15.0), 8.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(85.0, 45.0), 8.0).with_curbs(false, true),
        // Quick Chicane Left-Right
        TrackWaypoint::new(Vec2::new(60.0, 60.0), 7.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(60.0, 90.0), 7.5).with_curbs(false, true),
        // Hairpin Turn
        TrackWaypoint::new(Vec2::new(30.0, 110.0), 8.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(0.0, 100.0), 8.0).with_curbs(true, false),
        // Switchback
        TrackWaypoint::new(Vec2::new(10.0, 60.0), 7.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-20.0, 40.0), 8.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-20.0, 15.0), 8.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.0, BarrierType::TireWall);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 8, 5.5, 1.8);

    Track {
        name: "Kart Arena".to_string(),
        description: "Short, high-density karting circuit with fast transitions and chicanes.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        predefined_car: Some("kart".to_string()),
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "kart".to_string()],
    }
}

/// Preset 5: Ramp Raceway
/// Multi-elevation stadium circuit with high-speed launch ramps, gap jumps over hazard pits, and banked curves.
pub fn ramp_raceway() -> Track {
    let waypoints = vec![
        // Launch Straight & Start/Finish
        TrackWaypoint::new(Vec2::new(82.5, 50.0), 14.0),
        TrackWaypoint::new(Vec2::new(117.5, 15.0), 14.0),
        // Turn 1 High-Speed Sweeper
        TrackWaypoint::new(Vec2::new(160.0, 25.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(180.0, 60.0), 14.0).with_curbs(false, true),
        // Back Straight with Tabletop Jump Ramp
        TrackWaypoint::new(Vec2::new(180.0, 120.0), 14.0),
        TrackWaypoint::new(Vec2::new(170.0, 180.0), 14.0).with_curbs(true, false),
        // Stadium Hairpin Turn
        TrackWaypoint::new(Vec2::new(130.0, 230.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(70.0, 240.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(20.0, 210.0), 13.0).with_curbs(true, true),
        // Infield Straight
        TrackWaypoint::new(Vec2::new(0.0, 150.0), 13.5),
        TrackWaypoint::new(Vec2::new(-32.5, 92.5), 14.0),
        // Banked Outer Carousel
        TrackWaypoint::new(Vec2::new(-85.0, 57.5), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-95.0, 12.5), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-67.5, -22.5), 14.0).with_curbs(false, true),
        // Final Launch Ramp onto Front Straight
        TrackWaypoint::new(Vec2::new(-20.0, -20.0), 14.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Armco);

    let jump_ramps = vec![
        // Ramp 1: Back Straight Tabletop Jump
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(180.5, 95.0),
                half_extents: Vec2::new(5.0, 6.0),
                angle: 1.73,
            },
            Vec2::new(-0.16113189, 0.9869329),
            4.5,
            18.0,
            2.5,
            "Back Straight Tabletop Ramp",
        ),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(157.5, 105.0),
                max: Vec2::new(199.5, 135.0),
            },
            SurfaceType::Water,
            "Gap Jump Sand Hazard",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 8, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Ramp Raceway".to_string(),
        description: "High-speed stadium circuit with launch ramps, hazard water puddles, gap jumps & banked turns.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        predefined_car: Some("sports_car".to_string()),
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string()],
    }
}

/// Preset 6: Oasis Rally
/// Desert rally circuit featuring a pure compacted Dirt ribbon, perilous off-track Sand traps, and Oasis Water hazards.
pub fn oasis_rally() -> Track {
    let waypoints = vec![
        // Main Desert Dirt Straight & Start/Finish
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 15.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(70.0, 0.0), 15.5).with_surface(SurfaceType::Dirt),
        // Canyon Sweeping Right Entry
        TrackWaypoint::new(Vec2::new(140.0, 15.0), 16.0).with_surface(SurfaceType::Dirt),
        // Canyon Sweeping Right
        TrackWaypoint::new(Vec2::new(200.0, 50.0), 15.0).with_surface(SurfaceType::Dirt),
        // Canyon Ridge Climb
        TrackWaypoint::new(Vec2::new(245.0, 110.0), 15.5).with_surface(SurfaceType::Dirt),
        // Desert Basin High Hairpin
        TrackWaypoint::new(Vec2::new(255.0, 175.0), 16.0).with_surface(SurfaceType::Dirt),
        // Hairpin Exit to North Ridge
        TrackWaypoint::new(Vec2::new(215.0, 230.0), 15.0).with_surface(SurfaceType::Dirt),
        // North Ridge Straight
        TrackWaypoint::new(Vec2::new(145.0, 245.0), 15.0).with_surface(SurfaceType::Dirt),
        // Northern Oasis Approach Chicane
        TrackWaypoint::new(Vec2::new(75.0, 230.0), 14.5).with_surface(SurfaceType::Dirt),
        // Oasis Lake Sweeper
        TrackWaypoint::new(Vec2::new(15.0, 185.0), 14.0).with_surface(SurfaceType::Dirt),
        // Oasis Chicane Exit
        TrackWaypoint::new(Vec2::new(-45.0, 160.0), 15.0).with_surface(SurfaceType::Dirt),
        // Western Desert Flat Sweeper
        TrackWaypoint::new(Vec2::new(-100.0, 125.0), 16.0).with_surface(SurfaceType::Dirt),
        // Desert Ridge Switchback
        TrackWaypoint::new(Vec2::new(-120.0, 60.0), 15.5).with_surface(SurfaceType::Dirt),
        // Southern Desert Spring Chicane
        TrackWaypoint::new(Vec2::new(-85.0, 5.0), 15.0).with_surface(SurfaceType::Dirt),
        // Home Stretch Entry
        TrackWaypoint::new(Vec2::new(-40.0, -10.0), 15.0).with_surface(SurfaceType::Dirt),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.5, BarrierType::TireWall);

    let surface_zones = vec![
        // Deep off-track Sand traps along critical runoffs
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(230.0, 130.0),
                max: Vec2::new(290.0, 210.0),
            },
            SurfaceType::Sand,
            "Canyon Sand Trap 1",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(-160.0, 30.0),
                max: Vec2::new(-95.0, 110.0),
            },
            SurfaceType::Sand,
            "Western Sand Trap 2",
        ),
        // Oasis Water Pond: Northern Oasis Lagoon (circular hazard in chicane infield)
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(25.0, 190.0),
                radius: 12.0,
            },
            SurfaceType::Water,
            "Northern Oasis Lagoon",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Oasis Rally".to_string(),
        description: "Pure dirt desert rally circuit with oasis water hazards, perilous sand traps & high-sliding rally dynamics.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 3,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "rally".to_string()],
    }
}

/// Backwards compatibility alias for `oasis_rally`.
pub fn dune_raid() -> Track {
    oasis_rally()
}

/// Backwards compatibility alias for `oasis_rally`.
pub fn sahara_dunes() -> Track {
    oasis_rally()
}

/// Preset 7: Outlaw Pass
/// Perilous mountain circuit carving through a dramatic, narrow mountain pass ("The Pass")
/// with towering cliff rock faces, tight technical switchbacks, and high-speed mountain descents.
pub fn outlaw_pass() -> Track {
    let waypoints = vec![
        // Sector 1: Start Straight & High-Speed Sweeper
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0),
        TrackWaypoint::new(Vec2::new(80.0, 0.0), 13.0),
        TrackWaypoint::new(Vec2::new(130.0, -20.0), 13.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(170.0, 10.0), 13.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(180.0, 60.0), 13.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(150.0, 100.0), 13.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(100.0, 90.0), 12.0).with_curbs(true, false),
        // Sector 2: The Outlaw Pass (Tight, dramatic mountain gorge narrowing down to 7.0m)
        TrackWaypoint::new(Vec2::new(55.0, 120.0), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(20.0, 160.0), 7.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-20.0, 170.0), 7.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-60.0, 140.0), 7.0).with_curbs(true, false),
        // Sector 3: Canyon Descent & Return Straight
        TrackWaypoint::new(Vec2::new(-100.0, 100.0), 9.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-110.0, 40.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-80.0, -10.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-30.0, -15.0), 13.0).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.8, BarrierType::Armco);

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Outlaw Pass".to_string(),
        description: "Perilous mountain circuit carving through a dramatic narrow canyon pass with tight switchbacks and cliff rock walls.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        predefined_car: Some("sports_car".to_string()),
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "rally".to_string()],
    }
}

/// Preset 8: Dirt Figure-8 Arena
/// High-action rally stadium circuit featuring a flat at-grade figure-8 crossover,
/// sweeping dirt carousels, tabletop jump, and high-sliding rally dynamics.
pub fn dirt_figure_eight() -> Track {
    let waypoints = vec![
        // Sector 1: Start/Finish Straight (West Loop South Straight heading East)
        TrackWaypoint::new(Vec2::new(-90.0, -48.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-50.0, -45.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 2: Approach & Crossing to East Loop (SW to NE through (0.0, 0.0) at ground level)
        TrackWaypoint::new(Vec2::new(-24.0, -22.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(24.0, 22.0), 13.5).with_surface(SurfaceType::Dirt),
        // Sector 3: East Loop North Bank & Turn
        TrackWaypoint::new(Vec2::new(50.0, 45.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(90.0, 48.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(130.0, 40.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 4: East Carousel Sweeper
        TrackWaypoint::new(Vec2::new(155.0, 18.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(160.0, 0.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(155.0, -18.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 5: East Loop South Bank
        TrackWaypoint::new(Vec2::new(130.0, -40.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(90.0, -48.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(50.0, -45.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 6: Approach & Crossing to West Loop (SE to NW through (0.0, 0.0) at ground level)
        TrackWaypoint::new(Vec2::new(24.0, -22.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-24.0, 22.0), 13.5).with_surface(SurfaceType::Dirt),
        // Sector 7: West Loop North Bank & Turn
        TrackWaypoint::new(Vec2::new(-50.0, 45.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-90.0, 48.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-130.0, 40.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 8: West Carousel Sweeper Return to Finish
        TrackWaypoint::new(Vec2::new(-155.0, 18.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-160.0, 0.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-155.0, -18.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-130.0, -40.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(12.5, 11.25),
                half_extents: Vec2::new(6.25, 8.95),
                angle: -2.3339927,
            },
            Vec2::new(-0.69123477, -0.72263026),
            4.0,
            8.194263,
            1.8,
            "Crossover Tabletop East",
        ).with_surface(SurfaceType::Dirt),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-13.5, -12.25),
                half_extents: Vec2::new(6.25, 8.95),
                angle: 0.7290585,
            },
            Vec2::new(0.7458019, 0.66616774),
            4.0,
            8.194263,
            1.8,
            "Crossover Tabletop West",
        ).with_surface(SurfaceType::Dirt),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(155.0, 0.0),
                radius: 14.0,
            },
            SurfaceType::Sand,
            "East Carousel Sand Runoff",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-155.0, 0.0),
                radius: 14.0,
            },
            SurfaceType::Sand,
            "West Carousel Sand Runoff",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Dirt Figure-8 Arena".to_string(),
        description: "Stadium figure-8 dirt arena featuring an at-grade flat crossover, sweeping dirt carousels & tabletop jumps.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Preset: Figure 8 Arena (Classic Asphalt)
/// High-speed classic asphalt figure-8 arena with at-grade flat crossover, smooth concrete barriers, and sweeping carousels.
pub fn figure_eight() -> Track {
    let mut track = dirt_figure_eight();
    track.name = "Figure 8".to_string();
    track.description = "High-speed asphalt figure-8 arena featuring an at-grade flat crossover, sweeping carousels & concrete safety walls.".to_string();
    track.default_surface = SurfaceType::Grass;
    for wp in &mut track.spline.waypoints {
        wp.surface = Some(SurfaceType::Asphalt);
        wp.wall_type = Some(BarrierType::Concrete);
    }
    track.module_id = Some("classic".to_string());
    track.modules = vec!["classic".to_string()];
    track.predefined_car = Some("sports_car".to_string());
    track.rebuild_geometry(5.0, BarrierType::Concrete);
    track
}


/// Preset 9: Höljes Motorstadion (World RX Sweden)
/// The holy grail of Rallycross ("The Magic Weekend") in Värmland, Sweden.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,210m).
/// Features high-speed asphalt start, the iconic downhill Höljes jump crest, sweeping banked Velodrome, and mixed gravel infield.
pub fn holjes_rx() -> Track {
    let waypoints = vec![
        // Sector 1: Start/Finish Straight & Turn 1 Sweep (Asphalt)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(37.1, 15.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(75.4, 25.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(111.2, 7.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(148.9, -4.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        // Sector 2: Transition to Gravel & Downhill Höljes Jump Crest
        TrackWaypoint::new(Vec2::new(170.3, 25.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(170.2, 65.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(186.5, 101.8), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(205.8, 137.1), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(205.9, 175.7), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 3: The Velodrome (High-speed sweeping banked dirt curve)
        TrackWaypoint::new(Vec2::new(199.2, 211.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(236.0, 227.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(275.8, 232.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(301.0, 259.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(274.1, 284.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(234.2, 284.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(198.2, 268.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        // Sector 4: Infield Technical Switchback (Dirt)
        TrackWaypoint::new(Vec2::new(169.4, 239.8), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(143.1, 209.3), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(119.4, 176.6), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(101.1, 140.8), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(90.7, 101.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(70.3, 67.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Sector 5: Final Curves onto Start/Finish Straight
        TrackWaypoint::new(Vec2::new(34.0, 52.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-6.2, 50.4), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-46.5, 51.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-86.3, 47.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-107.8, 15.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-79.7, -7.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-39.6, -7.7), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(196.0, 119.5),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 1.07,
            },
            Vec2::new(0.48, 0.88),
            2.2,
            5.5,
            1.3,
            "Höljes Jump Crest",
        ).with_surface(SurfaceType::Dirt),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(120.0, -25.0),
                max: Vec2::new(180.0, 15.0),
            },
            SurfaceType::Sand,
            "Turn 1 Sand Trap",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(260.0, 210.0),
                max: Vec2::new(320.0, 280.0),
            },
            SurfaceType::Sand,
            "Velodrome Outer Runoff",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Höljes Motorstadion (World RX Sweden)".to_string(),
        description: "The Holy Grail of Rallycross in Sweden featuring the legendary Höljes Jump, banked Velodrome & mixed gravel sliding.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Preset 10: Lydden Hill Race Circuit (World RX Great Britain)
/// The historic birthplace of Rallycross in Kent, England (1967).
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,170m).
/// Features Chessons Drift (wide gravel sweeper), North Bend hairpin, Hairy Hill descent, and The Elbow.
pub fn lydden_hill() -> Track {
    let waypoints = vec![
        // Sector 1: Pit Straight & Canterbury Straight (Asphalt)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(40.6, 9.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(81.9, 6.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(113.7, -17.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(122.5, -57.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(128.6, -99.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(134.6, -140.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(141.3, -181.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(149.9, -222.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(162.5, -262.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(181.0, -299.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(195.5, -338.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(188.7, -378.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(158.1, -405.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Sector 2: Chessons Drift & North Bend Hairpin (Loose Gravel & Dirt Slide)
        TrackWaypoint::new(Vec2::new(116.8, -409.4), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(78.2, -395.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(50.9, -364.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(33.4, -326.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(23.7, -285.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(24.8, -244.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Sector 3: Hairy Hill Descent & Dover Slope (Gravel)
        TrackWaypoint::new(Vec2::new(34.6, -203.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(46.5, -163.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(61.4, -124.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(75.0, -85.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 4: Devils Elbow onto Pit Straight (Asphalt Transition)
        TrackWaypoint::new(Vec2::new(72.7, -43.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(37.6, -26.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-2.1, -38.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-32.4, -22.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(140.0, -425.0),
                max: Vec2::new(210.0, -320.0),
            },
            SurfaceType::Sand,
            "Chessons Drift Runoff",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(10.0, -420.0),
                max: Vec2::new(80.0, -350.0),
            },
            SurfaceType::Sand,
            "North Bend Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Lydden Hill Circuit (World RX Great Britain)".to_string(),
        description: "The historic birthplace of Rallycross featuring the iconic Chessons Drift gravel slide, North Bend & Devil's Elbow.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Preset 11: Lånkebanen / Hell RX (World RX Norway)
/// The spectacular Norwegian World RX circuit in Stjørdal / Hell ("Welcome to Hell").
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,019m).
/// Features dramatic downhill asphalt Turn 1, sweeping loose gravel carousel, undulating terrain, and high-speed jumps.
pub fn hell_rx() -> Track {
    let waypoints = vec![
        // Sector 1: Downhill Start Straight (Asphalt)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(36.2, 3.9), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(72.4, 7.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(106.5, 0.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(116.6, -32.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(92.5, -55.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Sector 2: Technical Mid-Field Complex (Asphalt)
        TrackWaypoint::new(Vec2::new(57.9, -67.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(23.8, -79.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(24.9, -108.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(59.0, -119.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(92.9, -131.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(116.2, -159.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(135.4, -189.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Sector 3: Transition to Loose Gravel Carousel & Jump Crest
        TrackWaypoint::new(Vec2::new(148.5, -223.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(127.6, -248.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(93.1, -239.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(65.4, -215.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(38.1, -191.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(13.9, -164.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Sector 4: Infield Dirt Esses
        TrackWaypoint::new(Vec2::new(-6.5, -134.4), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-25.3, -103.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-57.2, -89.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 5: Asphalt Climb onto Main Straight
        TrackWaypoint::new(Vec2::new(-92.9, -95.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-127.6, -89.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-132.8, -57.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-100.4, -42.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-66.1, -30.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-33.5, -14.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(79.0, -227.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 2.44,
            },
            Vec2::new(-0.76, 0.65),
            2.2,
            5.5,
            1.3,
            "Hell Gravel Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(100.0, -45.0),
                max: Vec2::new(140.0, 15.0),
            },
            SurfaceType::Sand,
            "Turn 1 Asphalt Runoff",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(-145.0, -75.0),
                max: Vec2::new(-95.0, -20.0),
            },
            SurfaceType::Sand,
            "West Hairpin Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Lånkebanen (World RX Norway)".to_string(),
        description: "Welcome to Hell! Fast downhill asphalt sweep, loose gravel carousel, technical esses & high-flying crests.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Preset 12: Circuit de Lohéac (World RX France)
/// The temple of French Rallycross in Brittany.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,088m).
/// Features a long asphalt launch straight, tight 90-degree Turn 1, technical gravel infield, tabletop jump, and fast sweeping finish.
pub fn loheac_rx() -> Track {
    let waypoints = vec![
        // Sector 1: Long Front Straight (Asphalt)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(38.8, 1.1), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(77.7, 2.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(112.0, -11.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Sector 2: Turn 1 (Heavy braking 90-degree right on asphalt)
        TrackWaypoint::new(Vec2::new(121.8, -48.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Sector 3: Transition to Infield Gravel & Tabletop Jump Crest
        TrackWaypoint::new(Vec2::new(125.6, -87.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(133.6, -125.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(156.9, -155.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(193.1, -169.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(231.3, -176.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(269.4, -183.7), 13.0).with_surface(SurfaceType::Dirt),
        // Sector 4: Western Turnaround & Return Transition
        TrackWaypoint::new(Vec2::new(307.5, -191.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(337.9, -210.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(321.9, -243.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(288.8, -263.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(251.6, -268.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(213.1, -263.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(174.6, -258.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(136.2, -252.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(98.7, -242.9), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(68.1, -219.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(50.2, -185.4), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Sector 5: Infield Chicane & Sweeping Final Corner onto Pit Straight
        TrackWaypoint::new(Vec2::new(52.5, -147.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(70.5, -112.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(68.4, -76.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(33.4, -63.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-4.4, -61.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-22.1, -30.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(145.0, -140.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: -0.92,
            },
            Vec2::new(0.61, -0.79),
            2.2,
            5.5,
            1.3,
            "Lohéac Infield Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(100.0, -60.0),
                max: Vec2::new(150.0, 5.0),
            },
            SurfaceType::Sand,
            "Turn 1 Sand Trap",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(280.0, -280.0),
                max: Vec2::new(350.0, -220.0),
            },
            SurfaceType::Sand,
            "Western Hairpin Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Circuit de Lohéac (World RX France)".to_string(),
        description: "The French Rallycross classic in Brittany with long asphalt drag straight, gravel tabletop jump & tight switchbacks.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Estering Buxtehude (World RX Germany)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 952.0m.
/// Features 69% asphalt / 31% dirt with the famous Turn 1 hairpin, downhill forest straight & gravel carousel.
pub fn estering_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(36.5, 3.4), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(72.9, 6.8), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(109.4, 10.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(145.9, 12.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(180.5, 3.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(186.0, -28.4), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(152.3, -40.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(119.5, -55.9), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(103.4, -88.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(80.0, -116.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(45.8, -128.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(10.2, -136.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-25.8, -137.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-38.2, -108.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-30.8, -73.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-58.9, -55.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-95.2, -50.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-131.1, -44.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-162.0, -24.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-191.8, -3.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-181.4, 23.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-145.1, 19.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-108.7, 14.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-72.4, 10.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-36.5, 3.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Estering Buxtehude (World RX Germany)".to_string(),
        description: "The cathedral of German Rallycross featuring the iconic Turn 1 hairpin dive, high-speed forest drag and technical gravel carousel.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Pista Automóvel de Montalegre (World RX Portugal)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,050.0m.
/// Features 64% asphalt / 36% dirt with mountain straight, technical dirt stadium hairpin and dirt jump.
pub fn montalegre_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(37.5, 1.6), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(74.9, 3.1), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(112.4, 4.7), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(149.9, 6.2), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(187.3, 8.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(224.8, 9.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(262.3, 10.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(281.8, -14.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(254.3, -37.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(220.9, -33.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(186.8, -24.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(149.9, -28.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(118.9, -46.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(83.0, -36.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(46.9, -26.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(20.2, -48.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(1.0, -79.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-35.8, -78.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-69.7, -85.9), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-87.0, -118.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-121.5, -118.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-149.0, -93.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-152.9, -59.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-117.8, -57.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-92.8, -32.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-56.5, -32.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-20.9, -28.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-135.0, -106.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 2.40,
            },
            Vec2::new(-0.74, 0.68),
            2.2,
            5.5,
            1.3,
            "Montalegre Dirt Table",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Pista Automóvel de Montalegre (World RX Portugal)".to_string(),
        description: "High-altitude mountain thriller in Portugal featuring an undulating drag straight, gravel stadium section and fast table crest.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Nyirád Racing Center (Euro RX Hungary)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,220.0m.
/// Features 30% asphalt / 70% dirt in the famous 'Red Cauldron' bauxite quarry with high-sliding elevation drops.
pub fn nyirad_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(39.8, 8.1), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(79.7, 16.3), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(119.5, 24.4), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(159.4, 32.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(193.7, 21.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(218.5, -7.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(228.5, 23.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(216.9, 60.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(183.5, 69.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(143.2, 74.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(104.7, 87.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(66.2, 91.6), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(35.0, 68.1), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(10.4, 36.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-28.2, 29.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-38.5, 62.3), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-10.8, 92.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(24.2, 112.3), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(62.4, 126.3), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(101.0, 138.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(105.9, 154.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(68.3, 164.6), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(30.4, 150.3), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-5.7, 131.4), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-37.5, 106.2), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-64.6, 76.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-86.0, 41.8), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-78.1, 4.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-40.0, -7.1), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Nyirád Racing Center (Euro RX Hungary)".to_string(),
        description: "The infamous 'Red Cauldron' carved out of red bauxite quarries, featuring heavy gravel elevation changes and sweeping technical slides.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Tykkimäen Moottorirata (World RX Finland)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,060.0m.
/// Features 53% asphalt / 47% dirt with severe elevation rollercoasters and the flying Tykkimäki dirt crest.
pub fn kouvola_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(36.3, -1.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(70.9, 7.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(69.2, 43.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(90.8, 67.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(95.0, 36.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(107.1, 3.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(128.8, 28.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(161.9, 33.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(183.4, 21.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(187.0, 18.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(160.7, -9.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(134.1, -35.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(107.5, -62.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(78.1, -86.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(44.1, -72.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(8.5, -60.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-29.3, -59.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-66.3, -56.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-93.4, -31.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-84.6, 3.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-62.9, 34.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-33.9, 58.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-0.1, 75.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(32.1, 95.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(59.3, 80.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(39.4, 50.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(7.3, 30.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-48.0, -58.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 3.07,
            },
            Vec2::new(-1.0, 0.07),
            2.2,
            5.5,
            1.3,
            "Tykkimäki Dirt Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Tykkimäen Moottorirata (World RX Finland)".to_string(),
        description: "Finnish rallycross heartland featuring severe elevation rollercoasters, blind gravel drops and the flying Tykkimäki dirt crest.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Circuit de Barcelona-Catalunya RX (World RX Spain)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,125.0m.
/// Features 50% asphalt / 50% dirt inside the iconic Spanish GP stadium with technical gravel hairpins and dirt jump.
pub fn catalunya_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(38.9, 6.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(73.7, -12.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(91.4, -47.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(85.1, -86.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(64.4, -120.4), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(43.7, -154.9), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(23.4, -189.6), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(3.2, -224.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-31.2, -239.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-71.0, -225.0), 11.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-34.7, -211.0), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-6.3, -187.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(10.4, -151.3), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(25.9, -114.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(55.2, -87.4), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(63.8, -49.7), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(37.5, -22.3), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-0.3, -31.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-32.2, -55.4), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-61.9, -67.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-101.6, -73.2), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-141.4, -78.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-156.4, -58.6), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-119.9, -41.8), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-83.0, -26.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-56.6, 3.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-28.7, 25.9), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(18.0, -133.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 1.17,
            },
            Vec2::new(0.39, 0.92),
            2.2,
            5.5,
            1.3,
            "Stadium Dirt Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Circuit de Barcelona-Catalunya RX (World RX Spain)".to_string(),
        description: "World RX stadium circuit inside the iconic Spanish Grand Prix stadium, featuring downhill gravel hairpin slides and stadium jump.".to_string(),
        category: TrackCategory::Main,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        predefined_car: Some("rally_car".to_string()),
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string(), "classic".to_string()],
    }
}

/// Layout shape options for prototypical circuit generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackShape {
    Oval,
    HorizontalEight,
}

impl TrackShape {
    pub const ALL: [TrackShape; 2] = [TrackShape::Oval, TrackShape::HorizontalEight];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Oval => "Oval",
            Self::HorizontalEight => "Horizontal Eight",
        }
    }
}

/// Race direction on circuit start/finish line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaceDirection {
    Right,
    Left,
}

impl RaceDirection {
    pub const ALL: [RaceDirection; 2] = [RaceDirection::Right, RaceDirection::Left];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Right => "Right",
            Self::Left => "Left",
        }
    }
}

/// Generates waypoints for a high-speed two-turn oval with the given race direction and road surface.
pub fn generate_oval_waypoints(direction: RaceDirection, surface: SurfaceType, width: f32) -> Vec<TrackWaypoint> {
    let mut wps = match direction {
        RaceDirection::Right => vec![
            // Front Straight heading Right (+X)
            TrackWaypoint::new(Vec2::new(-30.0, -60.0), width),
            TrackWaypoint::new(Vec2::new(75.0, -60.0), width),
            // East Curve
            TrackWaypoint::new(Vec2::new(135.0, -35.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(155.0, 0.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(135.0, 35.0), width).with_curbs(true, true),
            // Back Straight heading Left (-X)
            TrackWaypoint::new(Vec2::new(75.0, 60.0), width),
            TrackWaypoint::new(Vec2::new(-75.0, 60.0), width),
            // West Curve
            TrackWaypoint::new(Vec2::new(-135.0, 35.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(-155.0, 0.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(-135.0, -35.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(-75.0, -60.0), width),
        ],
        RaceDirection::Left => vec![
            // Front Straight heading Left (-X)
            TrackWaypoint::new(Vec2::new(30.0, -60.0), width),
            TrackWaypoint::new(Vec2::new(-75.0, -60.0), width),
            // West Curve
            TrackWaypoint::new(Vec2::new(-135.0, -35.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(-155.0, 0.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(-135.0, 35.0), width).with_curbs(true, true),
            // Back Straight heading Right (+X)
            TrackWaypoint::new(Vec2::new(-75.0, 60.0), width),
            TrackWaypoint::new(Vec2::new(75.0, 60.0), width),
            // East Curve
            TrackWaypoint::new(Vec2::new(135.0, 35.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(155.0, 0.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(135.0, -35.0), width).with_curbs(true, true),
            TrackWaypoint::new(Vec2::new(75.0, -60.0), width),
        ],
    };
    if surface != SurfaceType::Asphalt {
        for wp in &mut wps {
            wp.surface = Some(surface);
        }
    }
    wps
}

/// Generates waypoints for a horizontal Figure-8 circuit with central crossover, given race direction and road surface.
pub fn generate_horizontal_eight_waypoints(direction: RaceDirection, surface: SurfaceType, width: f32) -> Vec<TrackWaypoint> {
    let mut wps = match direction {
        RaceDirection::Right => vec![
            // Sector 1: Start/Finish Straight (West Loop South Straight heading East/Right)
            TrackWaypoint::new(Vec2::new(-90.0, -48.0), width),
            TrackWaypoint::new(Vec2::new(-50.0, -45.0), width),
            // Sector 2: Approach & Crossing to East Loop (SW to NE through (0.0, 0.0))
            TrackWaypoint::new(Vec2::new(-24.0, -22.0), width),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), width),
            TrackWaypoint::new(Vec2::new(24.0, 22.0), width),
            // Sector 3: East Loop North Bank & Turn
            TrackWaypoint::new(Vec2::new(50.0, 45.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(90.0, 48.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(130.0, 40.0), width).with_curbs(true, false),
            // Sector 4: East Carousel Sweeper
            TrackWaypoint::new(Vec2::new(155.0, 18.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(160.0, 0.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(155.0, -18.0), width).with_curbs(true, false),
            // Sector 5: East Loop South Bank
            TrackWaypoint::new(Vec2::new(130.0, -40.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(90.0, -48.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(50.0, -45.0), width),
            // Sector 6: Approach & Crossing to West Loop (SE to NW through (0.0, 0.0))
            TrackWaypoint::new(Vec2::new(24.0, -22.0), width),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), width),
            TrackWaypoint::new(Vec2::new(-24.0, 22.0), width),
            // Sector 7: West Loop North Bank & Turn
            TrackWaypoint::new(Vec2::new(-50.0, 45.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-90.0, 48.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-130.0, 40.0), width).with_curbs(true, false),
            // Sector 8: West Carousel Sweeper Return to Finish
            TrackWaypoint::new(Vec2::new(-155.0, 18.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-160.0, 0.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-155.0, -18.0), width).with_curbs(true, false),
            TrackWaypoint::new(Vec2::new(-130.0, -40.0), width),
        ],
        RaceDirection::Left => vec![
            // Sector 1: Start/Finish Straight (East Loop South Straight heading West/Left)
            TrackWaypoint::new(Vec2::new(90.0, -48.0), width),
            TrackWaypoint::new(Vec2::new(50.0, -45.0), width),
            // Sector 2: Approach & Crossing to West Loop (SE to NW through (0.0, 0.0))
            TrackWaypoint::new(Vec2::new(24.0, -22.0), width),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), width),
            TrackWaypoint::new(Vec2::new(-24.0, 22.0), width),
            // Sector 3: West Loop North Bank & Turn
            TrackWaypoint::new(Vec2::new(-50.0, 45.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-90.0, 48.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-130.0, 40.0), width).with_curbs(false, true),
            // Sector 4: West Carousel Sweeper
            TrackWaypoint::new(Vec2::new(-155.0, 18.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-160.0, 0.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-155.0, -18.0), width).with_curbs(false, true),
            // Sector 5: West Loop South Bank
            TrackWaypoint::new(Vec2::new(-130.0, -40.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-90.0, -48.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(-50.0, -45.0), width),
            // Sector 6: Approach & Crossing to East Loop (SW to NE through (0.0, 0.0))
            TrackWaypoint::new(Vec2::new(-24.0, -22.0), width),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), width),
            TrackWaypoint::new(Vec2::new(24.0, 22.0), width),
            // Sector 7: East Loop North Bank & Turn
            TrackWaypoint::new(Vec2::new(50.0, 45.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(90.0, 48.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(130.0, 40.0), width).with_curbs(false, true),
            // Sector 8: East Carousel Sweeper Return to Finish
            TrackWaypoint::new(Vec2::new(155.0, 18.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(160.0, 0.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(155.0, -18.0), width).with_curbs(false, true),
            TrackWaypoint::new(Vec2::new(130.0, -40.0), width),
        ],
    };
    if surface != SurfaceType::Asphalt {
        for wp in &mut wps {
            wp.surface = Some(surface);
        }
    }
    wps
}

/// Creates a prototypical template circuit for a specific module, layout shape, and race direction.
/// Follows standard module defaults:
/// - `classic`: Asphalt road, Grass off-track, sports coupe, 14m width
/// - `f1`: Asphalt road, Grass off-track, F1 car, 15m width
/// - `kart`: Asphalt road, Asphalt off-track, 125cc kart, 10m width
/// - `rally`: Dirt road, Dirt off-track, rally car, 12m width
pub fn create_prototypical_track(
    module_id: &str,
    shape: TrackShape,
    direction: RaceDirection,
) -> Track {
    let mod_id_clean = module_id.to_lowercase();
    let mod_str = mod_id_clean.as_str();

    let (road_surface, offtrack_surface, predefined_car, barrier_type, barrier_offset, width, default_laps) = match mod_str {
        "f1" => (
            SurfaceType::Asphalt,
            SurfaceType::Grass,
            "f1_car",
            BarrierType::Armco,
            4.0,
            15.0,
            5,
        ),
        "kart" => (
            SurfaceType::Asphalt,
            SurfaceType::Asphalt,
            "kart",
            BarrierType::TireWall,
            2.0,
            10.0,
            5,
        ),
        "rally" => (
            SurfaceType::Dirt,
            SurfaceType::Dirt,
            "rally_car",
            BarrierType::TireWall,
            3.5,
            12.0,
            3,
        ),
        _ => ( // "classic" and fallback
            SurfaceType::Asphalt,
            SurfaceType::Grass,
            "sports_car",
            BarrierType::TireWall,
            3.0,
            14.0,
            3,
        ),
    };

    let waypoints = match shape {
        TrackShape::Oval => generate_oval_waypoints(direction, road_surface, width),
        TrackShape::HorizontalEight => generate_horizontal_eight_waypoints(direction, road_surface, width),
    };

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, barrier_offset, barrier_type);

    let num_checkpoints = match shape {
        TrackShape::Oval => 8,
        TrackShape::HorizontalEight => 16,
    };
    let checkpoints = generate_checkpoints(&spline, num_checkpoints, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    let shape_name = match shape {
        TrackShape::Oval => "Oval",
        TrackShape::HorizontalEight => "Figure-8",
    };
    let dir_name = match direction {
        RaceDirection::Right => "Right",
        RaceDirection::Left => "Left",
    };
    let mod_name = match mod_str {
        "f1" => "Formula GP",
        "kart" => "Karting",
        "rally" => "Rallycross",
        _ => "Classic",
    };

    let name = format!("{} {} ({})", mod_name, shape_name, dir_name);
    let description = format!(
        "Prototypical {} circuit with {} layout in {} direction (Road: {}, Off-track: {}).",
        mod_name,
        shape_name,
        dir_name,
        road_surface.name(),
        offtrack_surface.name()
    );

    Track {
        name,
        description,
        category: TrackCategory::Draft,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: offtrack_surface,
        pit_box_area: None,
        default_laps,
        predefined_car: Some(predefined_car.to_string()),
        module_id: Some(mod_str.to_string()),
        modules: vec![mod_str.to_string()],
    }
}

/// Prototypical template for Classic Motorsport module.
pub fn classic_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("classic", shape, direction)
}

/// Prototypical template for Formula 1 module.
pub fn f1_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("f1", shape, direction)
}

/// Prototypical template for Karting module.
pub fn kart_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("kart", shape, direction)
}

/// Prototypical template for Rally Cross Championship module.
pub fn rally_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("rally", shape, direction)
}



