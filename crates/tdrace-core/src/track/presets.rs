use glam::Vec2;

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
    // Local swallowtail singularities on sharp corners only span a small number of consecutive samples (~3-10).
    let max_loop_span = 10;

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

/// Preset 9: Höljes Motorstadion (World RX Sweden)
/// The holy grail of Rallycross ("The Magic Weekend") in Värmland, Sweden.
/// Features high-speed asphalt start, the iconic Höljes jump crest, sweeping banked Velodrome, and mixed gravel infield.
pub fn holjes_rx() -> Track {
    let waypoints = vec![
        // Sector 1: Start/Finish Straight (Asphalt)
        TrackWaypoint::new(Vec2::new(-30.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(40.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt),
        // Sector 2: Turn 1 (Wide sweeping right hairpin on asphalt)
        TrackWaypoint::new(Vec2::new(95.0, 20.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(125.0, 60.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(105.0, 100.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Sector 3: The Iconic Höljes Jump Crest (Downhill gravel jump)
        TrackWaypoint::new(Vec2::new(65.0, 130.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(15.0, 150.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 4: The Velodrome (High-speed sweeping banked dirt curve)
        TrackWaypoint::new(Vec2::new(-45.0, 160.0), 14.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-95.0, 140.0), 14.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-120.0, 95.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 5: Infield Technical Switchback
        TrackWaypoint::new(Vec2::new(-105.0, 55.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-70.0, 35.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Sector 6: Final Curve onto Start/Finish Straight
        TrackWaypoint::new(Vec2::new(-60.0, 10.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(65.0, 130.0),
                half_extents: Vec2::new(5.0, 6.5),
                angle: 2.75,
            },
            Vec2::new(-0.93, 0.37),
            4.8,
            20.0,
            2.6,
            "Höljes Jump Crest",
        ),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(110.0, 30.0),
                max: Vec2::new(150.0, 90.0),
            },
            SurfaceType::Sand,
            "Turn 1 Sand Trap",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(-120.0, 145.0),
                max: Vec2::new(-20.0, 185.0),
            },
            SurfaceType::Sand,
            "Velodrome Outer Runoff",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
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
/// Features Chessons Drift (wide gravel sweeper), North Bend hairpin, Hairy Hill descent, and The Elbow.
pub fn lydden_hill() -> Track {
    let waypoints = vec![
        // Pit Straight & Start/Finish (Asphalt)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(80.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt),
        // Chessons Drift (Wide sweeping loose gravel drift corner)
        TrackWaypoint::new(Vec2::new(135.0, 15.0), 14.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(175.0, 50.0), 15.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(185.0, 100.0), 14.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(160.0, 150.0), 14.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // North Bend (Technical gravel hairpin)
        TrackWaypoint::new(Vec2::new(115.0, 185.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(65.0, 190.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(20.0, 165.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Hairy Hill (Downhill gravel descent)
        TrackWaypoint::new(Vec2::new(-15.0, 125.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-45.0, 95.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // The Elbow (Technical transition onto asphalt)
        TrackWaypoint::new(Vec2::new(-85.0, 75.0), 12.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-115.0, 50.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Devil's Elbow onto Pit Straight
        TrackWaypoint::new(Vec2::new(-95.0, 15.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-50.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(170.0, 30.0),
                max: Vec2::new(210.0, 130.0),
            },
            SurfaceType::Sand,
            "Chessons Drift Runoff",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(40.0, 180.0),
                max: Vec2::new(130.0, 215.0),
            },
            SurfaceType::Sand,
            "North Bend Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
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
/// Features dramatic downhill asphalt Turn 1, sweeping loose gravel carousel, undulating terrain, and high-speed jumps.
pub fn hell_rx() -> Track {
    let waypoints = vec![
        // Downhill Start Straight (Asphalt)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(75.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt),
        // Turn 1 (High-speed sweeping right on asphalt)
        TrackWaypoint::new(Vec2::new(130.0, 10.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(165.0, 35.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Transition to Gravel Carousel
        TrackWaypoint::new(Vec2::new(170.0, 80.0), 14.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(140.0, 125.0), 14.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(90.0, 145.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Technical Infield Dirt Esses
        TrackWaypoint::new(Vec2::new(40.0, 130.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(0.0, 150.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-45.0, 135.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // West Gravel Loop & Hairpin
        TrackWaypoint::new(Vec2::new(-90.0, 110.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-115.0, 70.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Uphill Asphalt Climb onto Main Straight
        TrackWaypoint::new(Vec2::new(-95.0, 30.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-55.0, 10.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(65.0, 140.0),
                half_extents: Vec2::new(5.0, 6.0),
                angle: 3.45,
            },
            Vec2::new(-0.95, -0.30),
            4.5,
            18.0,
            2.4,
            "Hell Gravel Jump",
        ),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(155.0, 10.0),
                max: Vec2::new(195.0, 70.0),
            },
            SurfaceType::Sand,
            "Turn 1 Asphalt Runoff",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(-135.0, 50.0),
                max: Vec2::new(-95.0, 125.0),
            },
            SurfaceType::Sand,
            "West Hairpin Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
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
/// Features a long asphalt launch straight, tight 90-degree Turn 1, technical gravel infield, tabletop jump, and fast sweeping finish.
pub fn loheac_rx() -> Track {
    let waypoints = vec![
        // Long Front Straight (Asphalt)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(90.0, 0.0), 14.5).with_surface(SurfaceType::Asphalt),
        // Turn 1 (Heavy braking 90-degree right on asphalt)
        TrackWaypoint::new(Vec2::new(145.0, 15.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(160.0, 50.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        // Transition to Gravel & Jump Crest
        TrackWaypoint::new(Vec2::new(140.0, 95.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(95.0, 120.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(45.0, 125.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Infield Gravel Chicane
        TrackWaypoint::new(Vec2::new(0.0, 100.0), 12.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-35.0, 120.0), 12.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        // Western Hairpin
        TrackWaypoint::new(Vec2::new(-80.0, 110.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-105.0, 75.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        // Return onto Asphalt & Sweeping Final Corner
        TrackWaypoint::new(Vec2::new(-90.0, 35.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-50.0, 10.0), 14.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(70.0, 123.0),
                half_extents: Vec2::new(5.0, 6.0),
                angle: std::f32::consts::PI,
            },
            Vec2::new(-1.0, 0.0),
            4.6,
            19.0,
            2.5,
            "Lohéac Infield Jump",
        ),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(145.0, 30.0),
                max: Vec2::new(185.0, 80.0),
            },
            SurfaceType::Sand,
            "Turn 1 Sand Trap",
        ),
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(-125.0, 60.0),
                max: Vec2::new(-85.0, 125.0),
            },
            SurfaceType::Sand,
            "Western Hairpin Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
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


