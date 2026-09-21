use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::checkpoint::Checkpoint;
use super::geometry::{
    BarrierType, JumpRamp, LineSegment, Obstacle, SpawnPose, SurfaceLayer, SurfaceShape,
    SurfaceZone, TrackGeometry, WallBarrier,
};
use super::scenery::{Grandstand, GrandstandStyle, Tree, TreeType};
use super::spline::{TrackSpline, TrackWaypoint};
use super::{CarCategory, Track, TrackCategory, TrackKind};
use wheelbase::SurfaceType;

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
                    is_bridge: wall.is_bridge,
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
                    let is_local_pair = if i < n_left && j < n_left {
                        let d_idx = (i as isize - j as isize).abs();
                        d_idx.min(n_left as isize - d_idx) <= 2
                    } else if i >= n_left && j >= n_left {
                        let ir = (i - n_left) as isize;
                        let jr = (j - n_left) as isize;
                        let d_idx = (ir - jr).abs();
                        d_idx.min(n_right as isize - d_idx) <= 2
                    } else {
                        false
                    };
                    if is_local_pair {
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
        let elev_factor = if s.is_bridge { (s.elevation / 3.0).clamp(0.0, 1.0) } else { 0.0 };
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
            let is_br = s_curr.is_bridge || s_next.is_bridge;
            raw_left_walls.push((
                WallBarrier::with_elevation(left_pts[i], left_pts[next_i], b_type, elev).with_bridge(is_br),
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
            let is_br = s_curr.is_bridge || s_next.is_bridge;
            raw_right_walls.push((
                WallBarrier::with_elevation(
                    right_pts[i],
                    right_pts[next_i],
                    b_type,
                    elev,
                ).with_bridge(is_br),
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

/// Generates an oval polygon hull centered at `center` with radii `rx` and `ry`.
pub fn generate_oval_hull(center: Vec2, rx: f32, ry: f32, num_pts: usize) -> Vec<Vec2> {
    let mut pts = Vec::with_capacity(num_pts);
    for i in 0..num_pts {
        let theta = (i as f32 / num_pts as f32) * std::f32::consts::TAU;
        pts.push(center + Vec2::new(rx * theta.cos(), ry * theta.sin()));
    }
    pts
}

/// Generates perimeter wall barriers enclosing a polygon hull.
pub fn generate_walls_from_hull(hull: &[Vec2], barrier_type: BarrierType) -> Vec<WallBarrier> {
    let mut walls = Vec::with_capacity(hull.len());
    for i in 0..hull.len() {
        let start = hull[i];
        let end = hull[(i + 1) % hull.len()];
        walls.push(WallBarrier::new(start, end, barrier_type));
    }
    walls
}

/// Generates starting grid spawn positions inside an arena field.
pub fn generate_arena_grid(
    center: Vec2,
    heading_rad: f32,
    num_slots: usize,
    spacing: f32,
    lateral: f32,
) -> Vec<SpawnPose> {
    let mut slots = Vec::with_capacity(num_slots);
    let forward = Vec2::new(heading_rad.cos(), heading_rad.sin());
    let lateral_vec = Vec2::new(-forward.y, forward.x);
    for i in 0..num_slots {
        let row = (i / 2) as f32;
        let side = if i % 2 == 0 { -lateral } else { lateral };
        let pos = center - forward * (row * spacing) + lateral_vec * side;
        slots.push(SpawnPose::new(pos, heading_rad, i));
    }
    slots
}

/// Generates a rhythmic array of whoop micro-ramps along a directional axis.
pub fn generate_whoops_array(
    start_id: usize,
    start: Vec2,
    dir: Vec2,
    count: usize,
    spacing: f32,
    width: f32,
    height: f32,
    surface: SurfaceType,
) -> Vec<JumpRamp> {
    let norm_dir = dir.normalize_or_zero();
    let angle = norm_dir.y.atan2(norm_dir.x);
    let mut ramps = Vec::with_capacity(count);
    for i in 0..count {
        let center = start + norm_dir * (i as f32 * spacing);
        ramps.push(
            JumpRamp::new(
                start_id + i,
                SurfaceShape::OrientedBox {
                    center,
                    half_extents: Vec2::new(spacing * 0.45, width * 0.5),
                    angle,
                },
                norm_dir,
                3.5,
                14.0,
                height,
                format!("Whoop Rhythm Mogul #{}", i + 1),
            )
            .with_surface(surface),
        );
    }
    ramps
}

/// Preset 1: Classic Grand Prix Circuit
/// Flowing corners, high-speed chicane, apex curbs, asphalt runoff, hairpin sand trap, pit lane.
pub fn classic_grand_prix() -> Track {
    let waypoints = vec![
        // Main Straight & Start/Finish
        TrackWaypoint::new(Vec2::new(70.0, 0.0), 14.0),
        TrackWaypoint::new(Vec2::new(120.0, 0.0), 14.0),
        // Turn 1 & 2 High-Speed Chicane
        TrackWaypoint::new(Vec2::new(165.0, 24.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(200.0, -10.0), 13.0).with_curbs(false, true),
        // Sweeping Curve into Back Straight
        TrackWaypoint::new(Vec2::new(255.0, 25.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(290.0, 85.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(270.0, 160.0), 13.0).with_curbs(false, true),
        // Hairpin Turn
        TrackWaypoint::new(Vec2::new(215.0, 210.0), 12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(150.0, 210.0), 12.0).with_curbs(true, true),
        // Technical Esses & Infield
        TrackWaypoint::new(Vec2::new(110.0, 165.0), 12.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(65.0, 180.0), 12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(15.0, 140.0), 12.0).with_curbs(false, true),
        // Final Corner onto Main Straight
        TrackWaypoint::new(Vec2::new(-25.0, 75.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-20.0, 20.0), 14.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

    let mut surface_zones = Vec::new();
    // Sand trap outside the hairpin
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(140.0, 215.0),
            max: Vec2::new(230.0, 260.0),
        },
        SurfaceType::Sand,
        "Hairpin Sand Trap",
    ));

    // Asphalt runoff outside Turn 1
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(155.0, 30.0),
            max: Vec2::new(200.0, 65.0),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: Some(SurfaceShape::Aabb {
            min: Vec2::new(30.0, -16.0),
            max: Vec2::new(70.0, -8.0),
        }),
        default_laps: 3,
        car_category: CarCategory::Gt,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset 2: Oval Speedway
/// High speed 2-turn oval with perimeter concrete walls, tight wall collisions, asphalt apron, and 22-degree banked turns.
pub fn oval_speedway() -> Track {
    let waypoints = vec![
        // Front Straight
        TrackWaypoint::new(Vec2::new(0.0, -60.0), 13.5).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(150.0, -60.0), 13.5).with_bank_angle(0.0),
        // Turn 1 & 2 (East Banked Curve)
        TrackWaypoint::new(Vec2::new(230.0, -25.0), 13.0).with_curbs(true, true).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(250.0, 30.0), 13.0).with_curbs(true, true).with_bank_angle(22.0),
        TrackWaypoint::new(Vec2::new(230.0, 85.0), 13.0).with_curbs(true, true).with_bank_angle(12.0),
        // Back Straight
        TrackWaypoint::new(Vec2::new(150.0, 120.0), 13.5).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(0.0, 120.0), 13.5).with_bank_angle(0.0),
        // Turn 3 & 4 (West Banked Curve)
        TrackWaypoint::new(Vec2::new(-80.0, 85.0), 13.0).with_curbs(true, true).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(-100.0, 30.0), 13.0).with_curbs(true, true).with_bank_angle(22.0),
        TrackWaypoint::new(Vec2::new(-80.0, -25.0), 13.0).with_curbs(true, true).with_bank_angle(12.0),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Dirty Oval Speedway
/// High-sliding dirt superspeedway oval with 18-degree banked curves and loose gravel cushion.
pub fn dirty_oval_speedway() -> Track {
    let waypoints = vec![
        // Front Straight
        TrackWaypoint::new(Vec2::new(22.5, -62.5), 13.5).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(172.5, -62.5), 13.5).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        // Turn 1 & 2 (East Banked Dirt Curve)
        TrackWaypoint::new(Vec2::new(252.5, -27.5), 13.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(272.5, 27.5), 13.0).with_surface(SurfaceType::Dirt).with_bank_angle(18.0),
        TrackWaypoint::new(Vec2::new(252.5, 82.5), 13.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
        // Back Straight
        TrackWaypoint::new(Vec2::new(172.5, 117.5), 13.5).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(22.5, 117.5), 13.5).with_surface(SurfaceType::Dirt).with_bank_angle(0.0),
        // Turn 3 & 4 (West Banked Dirt Curve)
        TrackWaypoint::new(Vec2::new(-57.5, 82.5), 13.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(-77.5, 27.5), 13.0).with_surface(SurfaceType::Dirt).with_bank_angle(18.0),
        TrackWaypoint::new(Vec2::new(-57.5, -27.5), 13.0).with_surface(SurfaceType::Dirt).with_bank_angle(10.0),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::OffRoad,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::Gt,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "kart".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Kart,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "kart".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset 5: Ramp Raceway
/// Multi-elevation stadium circuit with high-speed launch ramps, gap jumps over hazard pits, and banked curves.
pub fn ramp_raceway() -> Track {
    let waypoints = vec![
        // Launch Straight & Start/Finish
        TrackWaypoint::new(Vec2::new(82.5, 50.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(117.5, 15.0), 14.0).with_surface(SurfaceType::Dirt),
        // Turn 1 High-Speed Sweeper
        TrackWaypoint::new(Vec2::new(160.0, 25.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(180.0, 60.0), 14.0).with_surface(SurfaceType::Dirt),
        // Back Straight with Tabletop Jump Ramp
        TrackWaypoint::new(Vec2::new(180.0, 120.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(170.0, 180.0), 14.0).with_surface(SurfaceType::Dirt),
        // Stadium Hairpin Turn
        TrackWaypoint::new(Vec2::new(130.0, 230.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(70.0, 240.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(20.0, 210.0), 13.0).with_surface(SurfaceType::Dirt),
        // Infield Straight
        TrackWaypoint::new(Vec2::new(0.0, 150.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-32.5, 92.5), 14.0).with_surface(SurfaceType::Dirt),
        // Banked Outer Carousel
        TrackWaypoint::new(Vec2::new(-85.0, 57.5), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-95.0, 12.5), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-67.5, -22.5), 14.0).with_surface(SurfaceType::Dirt),
        // Final Launch Ramp onto Front Straight
        TrackWaypoint::new(Vec2::new(-20.0, -20.0), 14.0).with_surface(SurfaceType::Dirt),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

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
        ).with_surface(SurfaceType::Dirt),
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
        description: "High-speed dirt stadium circuit with launch ramps, hazard water puddles, gap jumps & banked dirt turns.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::Rally,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::OffRoad,
        module_id: Some("classic".to_string()),
        modules: vec!["extreme_offroad".to_string(), "classic".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
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
    track.car_category = CarCategory::Gt;
    track.rebuild_geometry(5.0, BarrierType::Concrete);
    track
}

/// Preset 8: Classic Rallycross
/// Dynamic 1.0 km mixed-surface rallycross circuit featuring asphalt launch straights, high-grip chicanes, sweeping dirt hairpins & tabletop jump ramps.
pub fn classic_rallycross() -> Track {
    let waypoints = vec![
        // Sector 1: Asphalt Main Straight & Start/Finish (heading East)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(80.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(160.0, 0.0), 14.0).with_surface(SurfaceType::Asphalt),

        // Sector 2: Asphalt Turn 1 Sweeper & Turn 2 Chicane
        TrackWaypoint::new(Vec2::new(220.0, 20.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(260.0, 60.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),

        // Sector 3: Transition to Infield Dirt Section
        TrackWaypoint::new(Vec2::new(280.0, 110.0), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(260.0, 160.0), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(210.0, 200.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(150.0, 220.0), 14.0).with_surface(SurfaceType::Dirt),

        // Sector 4: Dirt Back Straight with Tabletop Jump
        TrackWaypoint::new(Vec2::new(80.0, 215.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(10.0, 200.0), 14.0).with_surface(SurfaceType::Dirt),

        // Sector 5: Dirt Technical Hairpin & Basin Switchback
        TrackWaypoint::new(Vec2::new(-60.0, 170.0), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-90.0, 120.0), 14.5).with_surface(SurfaceType::Dirt),

        // Sector 6: Transition back to Asphalt High-Speed S-Chicane
        TrackWaypoint::new(Vec2::new(-100.0, 60.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-70.0, 10.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-30.0, -10.0), 14.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        // Tabletop jump on the dirt back straight around (45.0, 207.5)
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(45.0, 207.5),
                half_extents: Vec2::new(5.0, 6.0),
                angle: -0.21,
            },
            Vec2::new(-0.978, -0.208),
            4.0,
            16.0,
            2.0,
            "Dirt Back Straight Tabletop Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Aabb {
                min: Vec2::new(20.0, 195.0),
                max: Vec2::new(60.0, 220.0),
            },
            SurfaceType::Sand,
            "Jump Runoff Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Classic Rallycross".to_string(),
        description: "Dynamic 1.0 km mixed-surface rallycross circuit featuring asphalt launch straights, high-grip chicanes, sweeping dirt hairpins & tabletop jump ramps.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::Rally,
        module_id: Some("classic".to_string()),
        modules: vec!["classic".to_string(), "rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}


/// Preset 9: Höljes Motorstadion (World RX Sweden)
/// The holy grail of Rallycross ("The Magic Weekend") in Värmland, Sweden.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,210m).
/// Features high-speed asphalt start, the iconic downhill Höljes jump crest, sweeping banked Velodrome, and mixed gravel infield.
pub fn holjes_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(37.1, 15.7), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(75.4, 25.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(111.2, 7.6), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(148.9, -4.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(170.3, 25.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(170.2, 65.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(186.5, 101.8), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(205.8, 137.1), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(205.9, 175.7), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(199.2, 211.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(236.0, 227.9), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(275.8, 232.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(301.0, 259.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(274.1, 284.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(234.2, 284.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(198.2, 268.0), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(169.4, 239.8), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(143.1, 209.3), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(119.4, 176.6), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(101.1, 140.8), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(90.7, 101.9), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(70.3, 67.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(34.0, 52.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-6.2, 50.4), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-46.5, 51.6), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-86.3, 47.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-107.8, 15.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-79.7, -7.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-39.6, -7.7), 13.5).with_surface(SurfaceType::Dirt),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset 10: Lydden Hill Race Circuit (World RX Great Britain)
/// The historic birthplace of Rallycross in Kent, England (1967).
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,170m).
/// Features Chessons Drift (wide gravel sweeper), North Bend hairpin, Hairy Hill descent, and The Elbow.
pub fn lydden_hill() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(40.6, 9.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(81.9, 6.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(113.7, -17.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(122.5, -57.7), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(128.6, -99.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(134.6, -140.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(141.3, -181.7), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(149.9, -222.5), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(162.5, -262.3), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(181.0, -299.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(195.5, -338.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(188.7, -378.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(158.1, -405.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(116.8, -409.4), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(78.2, -395.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(50.9, -364.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(33.4, -326.1), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(23.7, -285.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(24.8, -244.1), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(34.6, -203.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(46.5, -163.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(61.4, -124.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(75.0, -85.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(72.7, -43.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(37.6, -26.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-2.1, -38.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-32.4, -22.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset 11: Lånkebanen / Hell RX (World RX Norway)
/// The spectacular Norwegian World RX circuit in Stjørdal / Hell ("Welcome to Hell").
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,019m).
/// Features dramatic downhill asphalt Turn 1, sweeping loose gravel carousel, undulating terrain, and high-speed jumps.
pub fn hell_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(36.2, 3.9), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(72.4, 7.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(106.5, 0.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(116.6, -32.3), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(92.5, -55.9), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(57.9, -67.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(23.8, -79.7), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(24.9, -108.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(59.0, -119.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(92.9, -131.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(116.2, -159.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(135.4, -189.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(148.5, -223.2), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(127.6, -248.5), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(93.1, -239.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(65.4, -215.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(38.1, -191.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(13.9, -164.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-6.5, -134.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-25.3, -103.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-57.2, -89.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-92.9, -95.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-127.6, -89.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-132.8, -57.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-100.4, -42.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-66.1, -30.2), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-33.5, -14.1), 14.0).with_surface(SurfaceType::Asphalt),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset 12: Circuit de Lohéac (World RX France)
/// The temple of French Rallycross in Brittany.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1,088m).
/// Features a long asphalt launch straight, tight 90-degree Turn 1, technical gravel infield, tabletop jump, and fast sweeping finish.
pub fn loheac_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(38.8, 1.1), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(77.7, 2.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(112.0, -11.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(121.8, -48.8), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(125.6, -87.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(133.6, -125.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(156.9, -155.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(193.1, -169.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(231.3, -176.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(269.4, -183.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(307.5, -191.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(337.9, -210.4), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(321.9, -243.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(288.8, -263.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(251.6, -268.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(213.1, -263.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(174.6, -258.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(136.2, -252.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(98.7, -242.9), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(68.1, -219.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(50.2, -185.4), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(52.5, -147.1), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(70.5, -112.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(68.4, -76.1), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(33.4, -63.2), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-4.4, -61.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-22.1, -30.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Estering Buxtehude (World RX Germany)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 952.0m.
/// Features 69% asphalt / 31% dirt with the famous Turn 1 hairpin, downhill forest straight & gravel carousel.
pub fn estering_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(36.5, 3.4), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(72.9, 6.8), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(109.4, 10.0), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(145.9, 12.3), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(180.5, 3.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(186.0, -28.4), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(152.3, -40.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(119.5, -55.9), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(103.4, -88.8), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(80.0, -116.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(45.8, -128.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(10.2, -136.8), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-25.8, -137.8), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-38.2, -108.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-30.8, -73.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-58.9, -55.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-95.2, -50.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-131.1, -44.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-162.0, -24.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-191.8, -3.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-181.4, 23.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-145.1, 19.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-108.7, 14.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-72.4, 10.3), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-36.5, 3.2), 14.5).with_surface(SurfaceType::Asphalt),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(198.0, 18.0), 40.0, 10.0, -0.25)
                    .with_style(GrandstandStyle::HillsideBleachers)
                    .with_tiers(5),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(140.0, -75.0), TreeType::Pine).with_scale(1.3),
                Tree::new(2, Vec2::new(125.0, -95.0), TreeType::Pine).with_scale(1.1),
                Tree::new(3, Vec2::new(65.0, -150.0), TreeType::Pine).with_scale(1.4),
                Tree::new(4, Vec2::new(25.0, -160.0), TreeType::Pine).with_scale(1.2),
                Tree::new(5, Vec2::new(-45.0, -160.0), TreeType::Pine).with_scale(1.3),
                Tree::new(6, Vec2::new(-90.0, -85.0), TreeType::Oak).with_scale(1.2),
                Tree::new(7, Vec2::new(-135.0, -70.0), TreeType::Oak).with_scale(1.3),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Pista Automóvel de Montalegre (World RX Portugal)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,050.0m.
/// Features 64% asphalt / 36% dirt with mountain straight, technical dirt stadium hairpin and dirt jump.
pub fn montalegre_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 11.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(37.5, 1.6), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(74.9, 3.1), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(112.4, 4.7), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(149.9, 6.2), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(187.3, 8.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(224.8, 9.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(262.3, 10.6), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(281.8, -14.7), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(254.3, -37.6), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(220.9, -33.7), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(186.8, -24.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(149.9, -28.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(118.9, -46.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(83.0, -36.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(46.9, -26.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(20.2, -48.5), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(1.0, -79.2), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-35.8, -78.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-69.7, -85.9), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-87.0, -118.8), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-121.5, -118.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-149.0, -93.6), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-152.9, -59.3), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-117.8, -57.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-92.8, -32.6), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-56.5, -32.7), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-20.9, -28.6), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Nyirád Racing Center (Euro RX Hungary)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,220.0m.
/// Features 30% asphalt / 70% dirt in the famous 'Red Cauldron' bauxite quarry with high-sliding elevation drops.
pub fn nyirad_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(39.8, 8.1), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(79.7, 16.3), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(119.5, 24.4), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(159.4, 32.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(193.7, 21.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(218.5, -7.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(228.5, 23.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(216.9, 60.9), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(183.5, 69.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(143.2, 74.5), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(104.7, 87.5), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(66.2, 91.6), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(35.0, 68.1), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(10.4, 36.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-28.2, 29.0), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-38.5, 62.3), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-10.8, 92.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(24.2, 112.3), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(62.4, 126.3), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(101.0, 138.5), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(105.9, 154.5), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(68.3, 164.6), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(30.4, 150.3), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-5.7, 131.4), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-37.5, 106.2), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-64.6, 76.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-86.0, 41.8), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-78.1, 4.5), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Tykkimäen Moottorirata (World RX Finland)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,060.0m.
/// Features 53% asphalt / 47% dirt with severe elevation rollercoasters and the flying Tykkimäki dirt crest.
pub fn kouvola_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 11.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(36.3, -1.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(70.9, 7.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(69.2, 43.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(90.8, 67.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(95.0, 36.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(107.1, 3.3), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(128.8, 28.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(161.9, 33.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(183.4, 21.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(187.0, 18.3), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(160.7, -9.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(134.1, -35.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(107.5, -62.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(78.1, -86.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(44.1, -72.2), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(8.5, -60.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-29.3, -59.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-66.3, -56.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-93.4, -31.5), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-84.6, 3.8), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-62.9, 34.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-33.9, 58.1), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-0.1, 75.3), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(32.1, 95.0), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(59.3, 80.2), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(39.4, 50.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(7.3, 30.6), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Circuit de Barcelona-Catalunya RX (World RX Spain)
/// Real-world 1:1 survey from OpenStreetMap (OSM) scaled to official FIA length of 1,125.0m.
/// Features 50% asphalt / 50% dirt inside the iconic Spanish GP stadium with technical gravel hairpins and dirt jump.
pub fn catalunya_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 11.5).with_surface(SurfaceType::Asphalt),
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
        TrackWaypoint::new(Vec2::new(63.8, -49.7), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(37.5, -22.3), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-0.3, -31.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-32.2, -55.4), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-61.9, -67.2), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-101.6, -73.2), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-141.4, -78.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-156.4, -58.6), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-119.9, -41.8), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-83.0, -26.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-56.6, 3.5), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-28.7, 25.9), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(10.0, 24.0), 55.0, 14.0, 0.16)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(8),
                Grandstand::new(2, Vec2::new(-100.0, -98.0), 45.0, 10.0, -0.12)
                    .with_style(GrandstandStyle::OpenBleachers)
                    .with_tiers(6),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(-20.0, 42.0), TreeType::Palm).with_scale(1.2),
                Tree::new(2, Vec2::new(15.0, 42.0), TreeType::Palm).with_scale(1.1),
                Tree::new(3, Vec2::new(50.0, 36.0), TreeType::Palm).with_scale(1.3),
                Tree::new(4, Vec2::new(-55.0, -250.0), TreeType::Cypress).with_scale(1.3),
                Tree::new(5, Vec2::new(-65.0, -248.0), TreeType::Cypress).with_scale(1.2),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Circuit Jules Tacheny Mettet (World RX Belgium)
/// Belgian Rallycross classic in Wallonia featuring the high-speed downhill plunge, technical gravel carousel and the flying Mettet dirt jump.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1149m).
pub fn mettet_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(35.2, -16.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(70.4, 4.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(93.7, 36.6), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(76.4, 72.3), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(50.0, 103.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(19.1, 129.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-20.6, 125.5), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-59.4, 113.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-95.5, 130.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-127.4, 156.2), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-159.3, 181.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-191.3, 207.7), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-223.2, 233.5), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-255.1, 259.3), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-287.0, 285.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-309.6, 298.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-337.4, 278.4), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-318.9, 258.3), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-287.0, 232.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-255.1, 206.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-223.3, 180.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-191.4, 154.9), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-159.5, 129.1), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-127.6, 103.3), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-95.7, 77.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-63.8, 51.6), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-31.9, 25.8), 14.5).with_surface(SurfaceType::Dirt),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-310.5, 251.1),
                half_extents: Vec2::new(3.8, 5.5),
                angle: -0.69,
            },
            Vec2::new(0.77, -0.63),
            2.2,
            5.5,
            1.3,
            "Mettet Dirt Jump Crest",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Circuit Jules Tacheny Mettet (World RX Belgium)".to_string(),
        description: "Belgian Rallycross classic in Wallonia featuring the high-speed downhill plunge, technical gravel carousel and the flying Mettet dirt jump.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Silverstone Circuit RX (World RX Great Britain)
/// The Speedmachine World RX arena at Silverstone featuring sweeping asphalt entries, technical loose dirt hairpin switches and the arena jump crest.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 972m).
pub fn silverstone_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(34.3, 0.9), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(68.8, -2.4), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(103.3, -5.7), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(137.9, -8.2), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(172.6, -10.5), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(205.7, -3.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(236.4, -15.5), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(271.0, -17.8), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(300.0, -1.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(301.3, 31.7), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(283.5, 61.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(266.1, 91.5), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(281.6, 119.1), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(293.4, 146.7), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(272.0, 174.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(250.4, 201.2), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(225.3, 224.1), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(194.2, 212.8), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(167.0, 191.3), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(139.8, 169.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(112.6, 148.2), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(85.3, 126.7), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(58.1, 105.1), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(30.9, 83.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(3.7, 62.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-23.4, 40.4), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-31.7, 12.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(180.6, 202.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: -2.47,
            },
            Vec2::new(-0.78, -0.62),
            2.2,
            5.5,
            1.3,
            "Silverstone Arena Dirt Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Silverstone Circuit RX (World RX Great Britain)".to_string(),
        description: "The Speedmachine World RX arena at Silverstone featuring sweeping asphalt entries, technical loose dirt hairpin switches and the arena jump crest.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Biķernieku Trase (World RX Latvia)
/// The historic Riga cathedral of speed featuring a punishing forest drag, sweeping double parallel dirt jump crests and high-grip technical gravel curves.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1294m).
pub fn riga_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-90.6, -120.8), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-45.5, -125.8), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-3.4, -144.0), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(36.0, -155.0), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(83.0, -147.0), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(136.2, -136.2), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(179.7, -123.2), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(193.7, -85.3), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(189.8, -38.5), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(191.3, 10.5), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(200.3, 52.1), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(163.3, 82.9), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(139.3, 57.0), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(101.6, 33.1), 12.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(58.2, 41.6), 12.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(26.9, 76.9), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-8.0, 97.2), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-52.0, 102.9), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-101.8, 108.4), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-154.2, 113.7), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-196.8, 117.7), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-223.7, 90.4), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-189.0, 66.2), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-144.2, 56.6), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-94.4, 51.9), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-52.6, 42.8), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-59.8, 0.1), 12.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-80.9, -39.0), 14.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-102.5, -80.9), 14.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-127.8, -124.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-76.9, 105.7),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 3.03,
            },
            Vec2::new(-0.99, 0.11),
            2.2,
            5.5,
            1.3,
            "Biķernieki Double Jump Crest",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Biķernieku Trase (World RX Latvia)".to_string(),
        description: "The historic Riga cathedral of speed featuring a punishing forest drag, sweeping double parallel dirt jump crests and high-grip technical gravel curves.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Killarney International Raceway (World RX South Africa)
/// Scenic Cape Town thriller in the shadow of Table Mountain, featuring a rapid asphalt drag, loose dirt jumps and high-drift hairpin transitions.
/// 1:1 metric reconstruction from OpenStreetMap survey data (FIA length: 1067m).
pub fn killarney_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 11.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(38.0, 2.2), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(76.1, 4.3), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(114.1, 6.5), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(151.6, 2.7), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(172.6, -27.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(163.0, -62.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(132.0, -80.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(108.2, -51.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(80.2, -28.7), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(43.7, -37.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(7.9, -50.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-28.6, -61.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-65.3, -71.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-102.2, -81.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-138.2, -86.7), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-161.3, -61.2), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-192.7, -82.4), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-226.6, -97.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-263.5, -88.0), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-279.2, -57.1), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-251.4, -39.3), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-213.3, -37.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-175.2, -36.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-137.2, -35.2), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-99.1, -33.6), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-61.0, -31.9), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-23.1, -29.4), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-118.0, -34.5),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 0.04,
            },
            Vec2::new(0.999, 0.04),
            2.2,
            5.5,
            1.3,
            "Killarney Dirt Kicker Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Killarney International Raceway (World RX South Africa)".to_string(),
        description: "Scenic Cape Town thriller in the shadow of Table Mountain, featuring a rapid asphalt drag, loose dirt jumps and high-drift hairpin transitions.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

pub fn yas_marina_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 11.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(32.6, -9.3), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(46.9, -42.5), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(81.2, -31.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(114.6, -13.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(149.0, -0.1), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(159.9, -28.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(127.3, -46.7), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(93.3, -62.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(59.4, -78.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(25.5, -94.6), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-8.4, -110.6), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-40.5, -107.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-67.0, -91.8), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-71.2, -123.6), 11.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-104.9, -139.7), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-140.1, -152.2), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-176.7, -160.4), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-211.3, -153.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-241.0, -130.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-251.4, -98.0), 11.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-220.0, -101.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-188.1, -117.1), 11.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-154.3, -101.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-120.4, -84.9), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-87.2, -67.5), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-56.0, -46.9), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-30.9, -19.2), 13.0).with_surface(SurfaceType::Dirt),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-122.5, -146.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: -2.80,
            },
            Vec2::new(-0.94, -0.34),
            2.2,
            5.5,
            1.3,
            "Yas Marina Arena Dirt Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 24, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Yas Marina RX Arena (World RX Abu Dhabi)".to_string(),
        description: "Spectacular twilight rallycross inside the Yas Marina amphitheater, featuring stadium dirt jumps, tight desert hairpins and high-speed grandstand sweeps.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(12.0, 24.0), 65.0, 16.0, 0.12)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(10)
                    .with_seat_color([0.15, 0.55, 0.95]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(-20.0, 42.0), TreeType::Palm).with_scale(1.3),
                Tree::new(2, Vec2::new(10.0, 48.0), TreeType::Palm).with_scale(1.2),
                Tree::new(3, Vec2::new(45.0, 45.0), TreeType::Palm).with_scale(1.4),
                Tree::new(4, Vec2::new(75.0, 35.0), TreeType::Palm).with_scale(1.2),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Circuit des Ducs (World RX / Euro RX France - Essay, Normandy)
/// Historic French rallycross proving ground in Normandy featuring a high-speed asphalt start,
/// technical sweeping switchbacks, the iconic "La Butte" dirt jump crest, and scenic Norman woods.
pub fn essay_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(33.4, -2.2), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(66.7, -4.4), 14.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(100.0, -5.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(125.3, 16.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(156.0, 27.1), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(186.7, 15.5), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(207.5, -10.2), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(195.4, -36.3), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(162.3, -40.4), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(129.0, -43.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(96.2, -37.7), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(64.6, -46.0), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(35.5, -62.3), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(3.3, -70.8), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-30.0, -73.7), 13.0).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-63.4, -75.9), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-96.8, -76.8), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-129.8, -73.8), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-161.1, -62.1), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-190.1, -45.5), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-194.2, -21.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-162.5, -29.2), 13.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-135.5, -42.0), 13.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-103.1, -48.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-69.7, -49.0), 13.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-37.6, -46.0), 13.0).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-20.3, -24.2), 14.0).with_surface(SurfaceType::Asphalt),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-145.0, -68.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 2.78,
            },
            Vec2::new(-0.94, 0.35),
            2.2,
            5.5,
            1.3,
            "La Butte Dirt Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 20, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Circuit des Ducs (Essay RX)".to_string(),
        description: "Historic French rallycross proving ground in Normandy featuring a high-speed asphalt start, the iconic 'La Butte' dirt jump crest, and scenic Norman woods.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(50.0, 22.0), 60.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(8)
                    .with_seat_color([0.85, 0.25, 0.20]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(125.0, 38.0), TreeType::Oak).with_scale(1.3),
                Tree::new(2, Vec2::new(165.0, 48.0), TreeType::Oak).with_scale(1.4),
                Tree::new(3, Vec2::new(205.0, 32.0), TreeType::Pine).with_scale(1.2),
                Tree::new(4, Vec2::new(228.0, -10.0), TreeType::Oak).with_scale(1.4),
                Tree::new(5, Vec2::new(215.0, -55.0), TreeType::AutumnMaple).with_scale(1.3),
                Tree::new(6, Vec2::new(-215.0, -50.0), TreeType::Oak).with_scale(1.3),
                Tree::new(7, Vec2::new(-218.0, -20.0), TreeType::Pine).with_scale(1.2),
                Tree::new(8, Vec2::new(-175.0, -8.0), TreeType::Oak).with_scale(1.4),
                Tree::new(9, Vec2::new(-140.0, -12.0), TreeType::AutumnMaple).with_scale(1.3),
                Tree::new(10, Vec2::new(-105.0, -18.0), TreeType::Oak).with_scale(1.2),
                Tree::new(11, Vec2::new(-70.0, -22.0), TreeType::Pine).with_scale(1.1),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
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
/// - `gt`: Asphalt road, Grass off-track, GT car, 15m width
/// - `kart`: Asphalt road, Asphalt off-track, 125cc kart, 10m width
/// - `rally`: Dirt road, Dirt off-track, rally car, 12m width
pub fn create_prototypical_track(
    module_id: &str,
    shape: TrackShape,
    direction: RaceDirection,
) -> Track {
    let mod_id_clean = module_id.to_lowercase();
    let mod_str = mod_id_clean.as_str();

    let (road_surface, offtrack_surface, car_category, barrier_type, barrier_offset, width, default_laps) = match mod_str {
        "gt" => (
            SurfaceType::Asphalt,
            SurfaceType::Grass,
            CarCategory::Gt,
            BarrierType::Steel,
            4.0,
            15.0,
            5,
        ),
        "kart" => (
            SurfaceType::Asphalt,
            SurfaceType::Asphalt,
            CarCategory::Kart,
            BarrierType::TireWall,
            2.0,
            10.0,
            5,
        ),
        "rally" => (
            SurfaceType::Dirt,
            SurfaceType::Dirt,
            CarCategory::Rally,
            BarrierType::TireWall,
            3.5,
            12.0,
            3,
        ),
        "nascar" => (
            SurfaceType::Asphalt,
            SurfaceType::Grass,
            CarCategory::Nascar,
            BarrierType::Concrete,
            1.5,
            22.0,
            10,
        ),
        _ => ( // "classic" and fallback
            SurfaceType::Asphalt,
            SurfaceType::Grass,
            CarCategory::Gt,
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
        "gt" => "GT World Challenge",
        "kart" => "Karting",
        "rally" => "Rallycross",
        "nascar" => "NASCAR Cup",
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
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: offtrack_surface,
        pit_box_area: None,
        default_laps,
        car_category,
        module_id: Some(mod_str.to_string()),
        modules: vec![mod_str.to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Prototypical template for Classic Motorsport module.
pub fn classic_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("classic", shape, direction)
}

/// Prototypical template for GT World Challenge module.
pub fn gt_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("gt", shape, direction)
}

/// Prototypical template for Karting module.
pub fn kart_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("kart", shape, direction)
}

/// Prototypical template for Rally Cross Championship module.
pub fn rally_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("rally", shape, direction)
}

/// Prototypical template for NASCAR Cup Series module.
pub fn nascar_template(shape: TrackShape, direction: RaceDirection) -> Track {
    create_prototypical_track("nascar", shape, direction)
}

/// Preset: Daytona Superspeedway (NASCAR Tri-Oval)
/// Premier 2.5-mile high-banked tri-oval featuring 31-degree banking in turns 1-4,
/// 18-degree banking in the tri-oval, wide 22m track surface for 3-wide pack drafting,
/// and perimeter SAFER/concrete barrier walls.
pub fn daytona_superspeedway() -> Track {
    let waypoints = vec![
        // Tri-Oval front straight and dogleg (Finish line at WP 0)
        TrackWaypoint::new(Vec2::new(0.0, -110.0), 22.0).with_bank_angle(18.0),
        TrackWaypoint::new(Vec2::new(140.0, -100.0), 22.0).with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(260.0, -85.0), 22.0).with_bank_angle(4.0),
        // Turn 1 & 2 (East 31-degree high-banked curve)
        TrackWaypoint::new(Vec2::new(370.0, -40.0), 22.0).with_bank_angle(31.0),
        TrackWaypoint::new(Vec2::new(430.0, 35.0), 22.0).with_bank_angle(31.0),
        TrackWaypoint::new(Vec2::new(400.0, 110.0), 22.0).with_bank_angle(31.0),
        TrackWaypoint::new(Vec2::new(320.0, 160.0), 22.0).with_bank_angle(20.0),
        // Superstretch (Back straight)
        TrackWaypoint::new(Vec2::new(180.0, 180.0), 22.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(0.0, 180.0), 22.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(-180.0, 180.0), 22.0).with_bank_angle(3.0),
        // Turn 3 & 4 (West 31-degree high-banked curve)
        TrackWaypoint::new(Vec2::new(-320.0, 160.0), 22.0).with_bank_angle(20.0),
        TrackWaypoint::new(Vec2::new(-400.0, 110.0), 22.0).with_bank_angle(31.0),
        TrackWaypoint::new(Vec2::new(-430.0, 35.0), 22.0).with_bank_angle(31.0),
        TrackWaypoint::new(Vec2::new(-370.0, -40.0), 22.0).with_bank_angle(31.0),
        // Turn 4 exit back to Tri-oval
        TrackWaypoint::new(Vec2::new(-260.0, -85.0), 22.0).with_bank_angle(4.0),
        TrackWaypoint::new(Vec2::new(-140.0, -100.0), 22.0).with_bank_angle(8.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 9.0, 3.5);

    Track {
        name: "Daytona Superspeedway".to_string(),
        description: "Premier 2.5-mile high-banked tri-oval with 31° banking and intense 3-wide pack drafting.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Talladega Superspeedway (NASCAR Monster Tri-Oval)
/// Massive 2.66-mile superspeedway with extreme 33-degree banking in turns,
/// 16.5-degree tri-oval, and the start/finish line located past the tri-oval toward turn 1.
pub fn talladega_superspeedway() -> Track {
    let waypoints = vec![
        // Front Straight past tri-oval (Finish line near turn 1 entry)
        TrackWaypoint::new(Vec2::new(100.0, -115.0), 24.0).with_bank_angle(6.0),
        TrackWaypoint::new(Vec2::new(260.0, -95.0), 24.0).with_bank_angle(4.0),
        // Turn 1 & 2 (Extreme 33-degree steep East curve)
        TrackWaypoint::new(Vec2::new(410.0, -45.0), 24.0).with_bank_angle(33.0),
        TrackWaypoint::new(Vec2::new(480.0, 45.0), 24.0).with_bank_angle(33.0),
        TrackWaypoint::new(Vec2::new(440.0, 135.0), 24.0).with_bank_angle(33.0),
        TrackWaypoint::new(Vec2::new(350.0, 195.0), 24.0).with_bank_angle(22.0),
        // Alabama Gang Backstretch
        TrackWaypoint::new(Vec2::new(200.0, 220.0), 24.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(0.0, 220.0), 24.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(-200.0, 220.0), 24.0).with_bank_angle(3.0),
        // Turn 3 & 4 (Extreme 33-degree steep West curve)
        TrackWaypoint::new(Vec2::new(-350.0, 195.0), 24.0).with_bank_angle(22.0),
        TrackWaypoint::new(Vec2::new(-440.0, 135.0), 24.0).with_bank_angle(33.0),
        TrackWaypoint::new(Vec2::new(-480.0, 45.0), 24.0).with_bank_angle(33.0),
        TrackWaypoint::new(Vec2::new(-410.0, -45.0), 24.0).with_bank_angle(33.0),
        // Turn 4 exit into Tri-oval
        TrackWaypoint::new(Vec2::new(-280.0, -95.0), 24.0).with_bank_angle(4.0),
        TrackWaypoint::new(Vec2::new(-130.0, -120.0), 24.0).with_bank_angle(16.5),
        // Tri-Oval apex
        TrackWaypoint::new(Vec2::new(-20.0, -135.0), 24.0).with_bank_angle(16.5),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 9.0, 3.5);

    Track {
        name: "Talladega Superspeedway".to_string(),
        description: "The biggest, fastest superspeedway with 33° banking and flat-out unrestricted slipstream battles.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Watkins Glen International (NASCAR Short Course)
/// Classic high-speed American road course featuring The Ninety, The Esses,
/// the high-speed Inner Loop "Bus Stop" chicane, and the banked Carousel.
pub fn watkins_glen_nascar() -> Track {
    let waypoints = vec![
        // Front Straight (Start/Finish at WP 0)
        TrackWaypoint::new(Vec2::new(-60.0, -160.0), 16.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(80.0, -160.0), 16.0).with_bank_angle(0.0),
        // Turn 1 (The Ninety)
        TrackWaypoint::new(Vec2::new(170.0, -120.0), 16.0).with_curbs(false, true),
        // The Esses (Rapid uphill S-curves)
        TrackWaypoint::new(Vec2::new(150.0, -40.0), 16.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(180.0, 40.0), 16.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(160.0, 120.0), 16.0).with_curbs(true, false),
        // Backstretch
        TrackWaypoint::new(Vec2::new(150.0, 220.0), 16.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(140.0, 320.0), 16.0).with_bank_angle(0.0),
        // The Bus Stop (Inner Loop Chicane)
        TrackWaypoint::new(Vec2::new(125.0, 380.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(145.0, 410.0), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(115.0, 440.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(130.0, 470.0), 14.0).with_curbs(true, false),
        // The Carousel (Banked 10-degree right sweeper)
        TrackWaypoint::new(Vec2::new(60.0, 520.0), 16.0).with_bank_angle(10.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-20.0, 530.0), 16.0).with_bank_angle(10.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-90.0, 490.0), 16.0).with_bank_angle(10.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-110.0, 420.0), 16.0).with_bank_angle(5.0),
        // The Chute / Infield
        TrackWaypoint::new(Vec2::new(-110.0, 320.0), 16.0),
        TrackWaypoint::new(Vec2::new(-130.0, 200.0), 16.0),
        // Turn 10 & 11 onto front straight
        TrackWaypoint::new(Vec2::new(-140.0, 80.0), 16.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-155.0, -30.0), 16.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-140.0, -110.0), 16.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.0, BarrierType::Steel);

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 8.5, 3.0);

    Track {
        name: "Watkins Glen International".to_string(),
        description: "Legendary NASCAR road course featuring The Esses, the Bus Stop chicane, and the high-speed Carousel.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Bristol Motor Speedway (The Last Great Colosseum)
/// 0.533-mile high-banked concrete short track with 28-30 degree steep banking,
/// tight walls, and non-stop paint-trading bumper action.
pub fn bristol_motor_speedway() -> Track {
    let waypoints = vec![
        // Front Straight (Finish Line at WP 0)
        TrackWaypoint::new(Vec2::new(0.0, -45.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(75.0, -45.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(10.0),
        // Turn 1 & 2 (High-banked concrete East curve)
        TrackWaypoint::new(Vec2::new(135.0, -20.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(28.0),
        TrackWaypoint::new(Vec2::new(150.0, 25.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(30.0),
        TrackWaypoint::new(Vec2::new(135.0, 70.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(28.0),
        // Back Straight
        TrackWaypoint::new(Vec2::new(75.0, 95.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(0.0, 95.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(-75.0, 95.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(10.0),
        // Turn 3 & 4 (High-banked concrete West curve)
        TrackWaypoint::new(Vec2::new(-135.0, 70.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(28.0),
        TrackWaypoint::new(Vec2::new(-150.0, 25.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(30.0),
        TrackWaypoint::new(Vec2::new(-135.0, -20.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(28.0),
        // Turn 4 exit to Front Straight
        TrackWaypoint::new(Vec2::new(-75.0, -45.0), 16.0).with_surface(SurfaceType::Concrete).with_bank_angle(10.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.2, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 16, 7.5, 3.0);

    Track {
        name: "Bristol Motor Speedway".to_string(),
        description: "The Last Great Colosseum: 0.533-mile steep concrete short track with 30° banking and bumper-to-bumper racing.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Martinsville Speedway (The Paperclip)
/// Historic 0.526-mile flat short track: long straights, tight flat concrete turns with 12° banking,
/// and intense heavy-braking bumper contact.
pub fn martinsville_speedway() -> Track {
    let waypoints = vec![
        // Frontstretch (Start/Finish Line at WP 0)
        TrackWaypoint::new(Vec2::new(0.0, -35.0), 16.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(100.0, -35.0), 16.0).with_bank_angle(0.0),
        // Turns 1 & 2 (Tight East concrete hairpin, 12-degree banking)
        TrackWaypoint::new(Vec2::new(140.0, -25.0), 16.0).with_bank_angle(8.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(160.0, 0.0), 16.0).with_bank_angle(12.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(160.0, 30.0), 16.0).with_bank_angle(12.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(140.0, 55.0), 16.0).with_bank_angle(8.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        // Backstretch
        TrackWaypoint::new(Vec2::new(100.0, 65.0), 16.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(0.0, 65.0), 16.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(-100.0, 65.0), 16.0).with_bank_angle(0.0),
        // Turns 3 & 4 (Tight West concrete hairpin, 12-degree banking)
        TrackWaypoint::new(Vec2::new(-140.0, 55.0), 16.0).with_bank_angle(8.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-160.0, 30.0), 16.0).with_bank_angle(12.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-160.0, 0.0), 16.0).with_bank_angle(12.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-140.0, -25.0), 16.0).with_bank_angle(8.0).with_surface(SurfaceType::Concrete).with_curbs(true, false),
        // Approach to Start/Finish
        TrackWaypoint::new(Vec2::new(-100.0, -35.0), 16.0).with_bank_angle(0.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.2, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 16, 7.5, 3.0);

    Track {
        name: "Martinsville Speedway".to_string(),
        description: "The Paperclip: 0.526-mile flat short track with tight 12° concrete corners, heavy curb-hopping, and brutal paint-trading.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Darlington Raceway (The Lady in Black / Too Tough to Tame)
/// Legendary 1.366-mile egg-shaped asymmetrical speedway: wide 25° sweeping Turns 1 & 2,
/// ultra-narrow 23° Turns 3 & 4 where stock cars brush the outside wall for the 'Darlington Stripe'.
pub fn darlington_raceway() -> Track {
    let waypoints = vec![
        // Frontstretch (Start/Finish Line at WP 0)
        TrackWaypoint::new(Vec2::new(0.0, -85.0), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(120.0, -85.0), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(240.0, -78.0), 18.0).with_bank_angle(6.0),
        // Turns 1 & 2 (Wide East sweeper, 25-degree banking)
        TrackWaypoint::new(Vec2::new(340.0, -40.0), 18.0).with_bank_angle(25.0),
        TrackWaypoint::new(Vec2::new(390.0, 30.0), 18.0).with_bank_angle(25.0),
        TrackWaypoint::new(Vec2::new(360.0, 100.0), 18.0).with_bank_angle(25.0),
        TrackWaypoint::new(Vec2::new(280.0, 145.0), 18.0).with_bank_angle(15.0),
        // Backstretch
        TrackWaypoint::new(Vec2::new(150.0, 160.0), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(0.0, 160.0), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(-120.0, 155.0), 18.0).with_bank_angle(4.0),
        // Turns 3 & 4 (Narrow & tight West curve, 23-degree banking)
        TrackWaypoint::new(Vec2::new(-210.0, 130.0), 18.0).with_bank_angle(23.0),
        TrackWaypoint::new(Vec2::new(-260.0, 75.0), 18.0).with_bank_angle(23.0),
        TrackWaypoint::new(Vec2::new(-270.0, 0.0), 18.0).with_bank_angle(23.0),
        TrackWaypoint::new(Vec2::new(-230.0, -55.0), 18.0).with_bank_angle(18.0),
        // Turn 4 exit to Frontstretch
        TrackWaypoint::new(Vec2::new(-140.0, -80.0), 18.0).with_bank_angle(5.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 10, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 8.0, 3.2);

    Track {
        name: "Darlington Raceway".to_string(),
        description: "The Lady in Black: 1.366-mile egg-shaped asymmetrical oval with 25° high banks and the famous wall-scraping 'Darlington Stripe'.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Charlotte Motor Speedway (The Beast of the Southeast)
/// Iconic 1.5-mile quad-oval speedway featuring 24-degree banking in turns,
/// frontstretch double dogleg, and multi-groove slipstream pack racing.
pub fn charlotte_motor_speedway() -> Track {
    let waypoints = vec![
        // Frontstretch quad-oval dogleg (Start/Finish at WP 0)
        TrackWaypoint::new(Vec2::new(0.0, -95.0), 20.0).with_bank_angle(5.0),
        TrackWaypoint::new(Vec2::new(110.0, -90.0), 20.0).with_bank_angle(5.0),
        TrackWaypoint::new(Vec2::new(220.0, -75.0), 20.0).with_bank_angle(8.0),
        // Turns 1 & 2 (24° High Banked East Curve)
        TrackWaypoint::new(Vec2::new(320.0, -35.0), 20.0).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(370.0, 35.0), 20.0).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(340.0, 105.0), 20.0).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(260.0, 150.0), 20.0).with_bank_angle(14.0),
        // Backstretch
        TrackWaypoint::new(Vec2::new(140.0, 165.0), 20.0).with_bank_angle(5.0),
        TrackWaypoint::new(Vec2::new(0.0, 165.0), 20.0).with_bank_angle(5.0),
        TrackWaypoint::new(Vec2::new(-140.0, 165.0), 20.0).with_bank_angle(5.0),
        // Turns 3 & 4 (24° High Banked West Curve)
        TrackWaypoint::new(Vec2::new(-260.0, 150.0), 20.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(-340.0, 105.0), 20.0).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(-370.0, 35.0), 20.0).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(-320.0, -35.0), 20.0).with_bank_angle(24.0),
        // Quad-oval entry kink
        TrackWaypoint::new(Vec2::new(-220.0, -75.0), 20.0).with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(-110.0, -90.0), 20.0).with_bank_angle(5.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 8.5, 3.5);

    Track {
        name: "Charlotte Motor Speedway".to_string(),
        description: "The Beast of the Southeast: 1.5-mile quad-oval with 24° banking, frontstretch dogleg, and high-speed pack drafting battles.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Indianapolis Motor Speedway (The Brickyard)
/// Legendary 2.5-mile rectangular speedway surveyed from OpenStreetMap (OSM):
/// four distinct 90-degree banked corners at 9.2° banking, long 5/8-mile straights,
/// 1/8-mile short chutes, and the iconic Yard of Bricks start/finish line.
pub fn indianapolis_motor_speedway() -> Track {
    let waypoints = vec![
        // Frontstretch & Yard of Bricks (Start/Finish line at WP 0)
        TrackWaypoint::new(Vec2::new(0.0, -180.0), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(83.8, -180.0), 20.0).with_bank_angle(0.0),
        // Turn 1 (Southeast 90° curve, 9.2° banking)
        TrackWaypoint::new(Vec2::new(167.6, -178.4), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(241.1, -141.7), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(277.6, -68.2), 20.0).with_bank_angle(9.2),
        // South Short Chute
        TrackWaypoint::new(Vec2::new(278.5, 15.6), 20.0).with_bank_angle(0.0),
        // Turn 2 (Northeast 90° curve, 9.2° banking)
        TrackWaypoint::new(Vec2::new(269.4, 98.2), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(215.6, 160.7), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(135.0, 178.4), 20.0).with_bank_angle(9.2),
        // Backstretch
        TrackWaypoint::new(Vec2::new(51.2, 178.6), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(-32.6, 178.4), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(-116.4, 178.1), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(-200.2, 177.9), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(-284.0, 177.7), 20.0).with_bank_angle(0.0),
        // Turn 3 (Northwest 90° curve, 9.2° banking)
        TrackWaypoint::new(Vec2::new(-367.8, 176.5), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(-441.4, 139.8), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(-477.9, 66.3), 20.0).with_bank_angle(9.2),
        // North Short Chute
        TrackWaypoint::new(Vec2::new(-478.8, -17.5), 20.0).with_bank_angle(0.0),
        // Turn 4 (Southwest 90° curve, 9.2° banking)
        TrackWaypoint::new(Vec2::new(-469.7, -100.1), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(-415.9, -162.3), 20.0).with_bank_angle(9.2),
        TrackWaypoint::new(Vec2::new(-335.3, -180.4), 20.0).with_bank_angle(9.2),
        // Approach to Start/Finish line
        TrackWaypoint::new(Vec2::new(-251.5, -180.0), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(-167.6, -180.0), 20.0).with_bank_angle(0.0),
        TrackWaypoint::new(Vec2::new(-83.8, -180.0), 20.0).with_bank_angle(0.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 9.0, 3.5);

    Track {
        name: "Indianapolis Motor Speedway".to_string(),
        description: "The Brickyard: 2.5-mile historic rectangular speedway with 9.2° banked turns, long drafting straights, and the famous Yard of Bricks start/finish line.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Eldora Speedway (Dirt Track Oval)
/// Historic 0.5-mile high-banked clay oval surveyed from OpenStreetMap (OSM):
/// 24° banking in turns, 8° on straights, full dirt surface, and relentless slide control.
pub fn eldora_speedway() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(57.4, -3.3), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(113.2, 7.3), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(154.2, 46.4), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(165.9, 101.2), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(140.1, 151.9), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(91.0, 180.7), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(35.1, 193.0), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(-22.3, 193.7), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(8.0),
        TrackWaypoint::new(Vec2::new(-78.2, 182.4), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(-118.7, 143.0), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(-128.4, 87.3), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(-106.0, 35.3), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(24.0),
        TrackWaypoint::new(Vec2::new(-57.0, 6.7), 18.0).with_surface(SurfaceType::Dirt).with_bank_angle(8.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.2, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 16, 7.5, 3.0);

    Track {
        name: "Eldora Speedway".to_string(),
        description: "Historic 0.5-mile high-banked clay oval featuring 24° banking, relentless dirt sliding, and close-quarters pack racing.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Iowa Speedway (The Fastest Short Track on the Planet)
/// 7/8-mile D-shaped asphalt oval surveyed from OpenStreetMap (OSM):
/// 12°-14° progressive banking in turns, 10° frontstretch tri-oval dogleg, and high-speed drafting.
pub fn iowa_speedway() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 20.0).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(86.7, -14.9), 20.0).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(174.4, -12.5), 20.0).with_bank_angle(10.0),
        TrackWaypoint::new(Vec2::new(256.3, 15.0), 20.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(305.6, 86.0), 20.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(303.6, 172.2), 20.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(247.6, 237.3), 20.0).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(165.7, 268.2), 20.0).with_bank_angle(4.0),
        TrackWaypoint::new(Vec2::new(80.5, 290.3), 20.0).with_bank_angle(4.0),
        TrackWaypoint::new(Vec2::new(-4.8, 312.2), 20.0).with_bank_angle(4.0),
        TrackWaypoint::new(Vec2::new(-91.3, 324.7), 20.0).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(-173.1, 297.2), 20.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(-222.3, 226.3), 20.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(-214.5, 139.8), 20.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(-159.2, 72.7), 20.0).with_bank_angle(12.0),
        TrackWaypoint::new(Vec2::new(-82.7, 29.8), 20.0).with_bank_angle(10.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 10, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 8.5, 3.2);

    Track {
        name: "Iowa Speedway".to_string(),
        description: "The Fastest Short Track on the Planet: 7/8-mile D-shaped oval with progressive 12°-14° banking and intense multi-groove racing.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Road America (Elkhart Lake)
/// Legendary 4.048-mile natural-terrain road course surveyed from OpenStreetMap (OSM) scaled to 0.5x (3257.5m):
/// Turn 1, Turn 3, Moraine Sweep, Turn 5, Hurry Downs, the Carousel, The Kink, Canada Corner, and Bill Mitchell Bend.
pub fn road_america() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 15.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(101.8, -0.2), 15.0),
        TrackWaypoint::new(Vec2::new(203.6, -0.2), 15.0),
        TrackWaypoint::new(Vec2::new(305.4, -0.0), 15.0),
        TrackWaypoint::new(Vec2::new(407.2, 0.2), 15.0),
        TrackWaypoint::new(Vec2::new(509.0, 0.5), 15.0),
        TrackWaypoint::new(Vec2::new(610.8, 0.4), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(664.7, -70.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(679.0, -170.7), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(641.4, -246.1), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(540.7, -232.2), 14.0),
        TrackWaypoint::new(Vec2::new(441.1, -211.5), 14.0),
        TrackWaypoint::new(Vec2::new(344.3, -179.8), 14.0),
        TrackWaypoint::new(Vec2::new(249.8, -142.0), 14.0),
        TrackWaypoint::new(Vec2::new(151.3, -116.8), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(91.8, -156.6), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(98.6, -257.3), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(198.5, -260.6), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(271.3, -325.1), 14.0),
        TrackWaypoint::new(Vec2::new(333.6, -405.6), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(416.5, -382.7), 14.0),
        TrackWaypoint::new(Vec2::new(507.4, -395.9), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(496.0, -487.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(398.9, -514.5), 14.0),
        TrackWaypoint::new(Vec2::new(298.6, -532.3), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(210.4, -488.7), 14.0),
        TrackWaypoint::new(Vec2::new(136.0, -419.5), 14.0),
        TrackWaypoint::new(Vec2::new(49.8, -366.0), 14.0),
        TrackWaypoint::new(Vec2::new(-45.0, -329.8), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-76.2, -257.5), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-17.4, -175.9), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-33.1, -77.1), 14.0).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.0, BarrierType::Steel);

    let checkpoints = generate_checkpoints(&spline, 16, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 9.0, 3.2);

    Track {
        name: "Road America".to_string(),
        description: "Historic 4.0-mile natural-terrain road course featuring the Carousel, The Kink, and Canada Corner.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Chicago Street Course (Grant Park 220)
/// NASCAR's premier 2.2-mile 12-turn downtown street circuit surveyed from OpenStreetMap (OSM) scaled to 0.5x (1770m):
/// tight 90° corners between concrete barrier walls along Columbus Drive, Balbo Drive, DuSable Lake Shore Drive, and Michigan Avenue.
pub fn chicago_street_course() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.0),
        TrackWaypoint::new(Vec2::new(4.8, -101.3), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(97.5, -108.9), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(171.1, -55.5), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(228.9, 27.8), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(267.0, 120.8), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(269.9, 221.9), 13.0),
        TrackWaypoint::new(Vec2::new(268.1, 323.3), 13.0),
        TrackWaypoint::new(Vec2::new(266.5, 424.7), 13.0),
        TrackWaypoint::new(Vec2::new(256.1, 518.1), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(154.8, 516.9), 13.0),
        TrackWaypoint::new(Vec2::new(53.4, 515.8), 12.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-8.3, 563.7), 12.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-8.3, 665.0), 13.0),
        TrackWaypoint::new(Vec2::new(-9.7, 766.4), 13.0),
        TrackWaypoint::new(Vec2::new(-11.7, 867.7), 13.0),
        TrackWaypoint::new(Vec2::new(-14.3, 969.1), 13.0),
        TrackWaypoint::new(Vec2::new(-16.9, 1070.4), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-103.9, 1082.9), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-205.2, 1081.3), 13.0),
        TrackWaypoint::new(Vec2::new(-306.6, 1079.8), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-304.5, 979.1), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-268.3, 891.6), 13.0),
        TrackWaypoint::new(Vec2::new(-231.8, 799.6), 12.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-260.4, 704.4), 12.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-303.6, 621.5), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-301.4, 520.2), 12.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-212.0, 509.9), 13.0),
        TrackWaypoint::new(Vec2::new(-110.7, 512.3), 13.0),
        TrackWaypoint::new(Vec2::new(-13.6, 504.2), 12.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-7.4, 405.4), 13.0),
        TrackWaypoint::new(Vec2::new(-5.9, 304.0), 13.0),
        TrackWaypoint::new(Vec2::new(-4.1, 202.7), 13.0),
        TrackWaypoint::new(Vec2::new(-1.9, 101.3), 13.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.2, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 8.5, 3.2);

    Track {
        name: "Chicago Street Course".to_string(),
        description: "NASCAR's premier 2.2-mile downtown street circuit through Grant Park with 12 tight 90-degree corners between concrete barrier walls.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Circuit of the Americas (COTA)
/// Austin Texas Grand Prix venue surveyed from OpenStreetMap (OSM) scaled to 0.5x FIA length (2756.5m):
/// steep uphill Turn 1 blind crest, Maggotts-inspired Esses, and multi-apex carousel.
pub fn cota() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(91.9, -0.0), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(183.8, -0.3), 15.0).with_surface(SurfaceType::Asphalt).with_elevation(2.0),
        TrackWaypoint::new(Vec2::new(275.6, -0.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(4.5),
        TrackWaypoint::new(Vec2::new(367.5, 1.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false).with_elevation(2.5),
        TrackWaypoint::new(Vec2::new(330.9, 51.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(298.3, 127.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(328.7, 213.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(336.7, 300.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(325.2, 387.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(404.7, 424.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(431.8, 508.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(505.4, 544.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(538.9, 625.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(541.5, 717.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(518.0, 768.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(465.6, 692.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(410.3, 619.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(351.7, 548.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(290.1, 480.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(225.2, 415.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(159.5, 351.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(209.6, 312.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(214.3, 264.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(129.9, 270.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(205.3, 229.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(231.4, 153.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(158.5, 109.5), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(69.9, 133.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(13.8, 71.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

    let checkpoints = generate_checkpoints(&spline, 20, 3);
    let starting_grid = generate_grid_positions(&spline, 20, 10.0, 2.5);

    Track {
        name: "Circuit of the Americas (COTA)".to_string(),
        description: "Austin Texas spectacle with steep uphill Turn 1 blind crest, Maggotts-inspired Esses, and multi-apex carousel.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions: starting_grid,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Gt,
        module_id: Some("gt".to_string()),
        modules: vec!["gt".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Sahara Dune Crossing
/// High-speed desert sprint circuit across rolling sand dunes with three progressive crest tabletop jump ramps.
pub fn sahara_dune_crossing() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-350.0, -180.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(-150.0, -180.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(100.0, -180.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(300.0, -170.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(450.0, -90.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(480.0, 50.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(410.0, 180.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(260.0, 240.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(110.0, 210.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(-20.0, 260.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-160.0, 270.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-330.0, 220.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-450.0, 110.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-440.0, -40.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-380.0, -140.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 6.0, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(200.0, -175.0),
                half_extents: Vec2::new(6.0, 9.0),
                angle: 0.05,
            },
            Vec2::new(0.998, 0.05),
            5.5,
            16.0,
            2.2,
            "Dune Ridge Leap",
        ).with_surface(SurfaceType::Sand),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(185.0, 225.0),
                half_extents: Vec2::new(6.0, 9.0),
                angle: 2.94,
            },
            Vec2::new(-0.98, 0.2),
            6.0,
            18.0,
            2.6,
            "Camelback Double",
        ).with_surface(SurfaceType::Sand),
        JumpRamp::new(
            3,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-390.0, 165.0),
                half_extents: Vec2::new(6.0, 9.0),
                angle: -2.44,
            },
            Vec2::new(-0.76, -0.65),
            5.0,
            15.0,
            2.0,
            "Erg Chebbi Big Air",
        ).with_surface(SurfaceType::Sand),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(450.0, -90.0),
                radius: 20.0,
            },
            SurfaceType::Sand,
            "Turn 1 Deep Sand Runoff",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-450.0, 110.0),
                radius: 22.0,
            },
            SurfaceType::Sand,
            "West Hairpin Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
    let grid_positions = generate_grid_positions(&spline, 12, 10.0, 3.5);

    Track {
        name: "Sahara Dune Crossing".to_string(),
        description: "High-speed Saharan desert sprint circuit across rolling sand dunes with three progressive crest tabletop jump ramps.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Atacama Sand Basin
/// Hyper-speed Chilean desert basin with parabolic high-speed sweepers across dried salt flats and powdery fesh-fesh dunes.
pub fn atacama_sand_basin() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-420.0, -220.0), 18.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(-150.0, -220.0), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(150.0, -220.0), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(420.0, -210.0), 18.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(600.0, -110.0), 18.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(660.0, 60.0), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(560.0, 200.0), 18.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(380.0, 250.0), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(120.0, 210.0), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-120.0, 260.0), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-350.0, 230.0), 18.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-540.0, 160.0), 18.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-630.0, 20.0), 18.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-560.0, -130.0), 18.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 8.0, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(280.0, -215.0),
                half_extents: Vec2::new(7.0, 10.0),
                angle: 0.04,
            },
            Vec2::new(0.999, 0.04),
            6.0,
            15.0,
            2.0,
            "Salt Basin High-Speed Crest",
        ).with_surface(SurfaceType::Sand),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-240.0, 245.0),
                half_extents: Vec2::new(7.0, 10.0),
                angle: -3.01,
            },
            Vec2::new(-0.99, -0.13),
            6.0,
            16.0,
            2.2,
            "Fesh-Fesh Dune Leap",
        ).with_surface(SurfaceType::Sand),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(600.0, -110.0),
                radius: 26.0,
            },
            SurfaceType::Sand,
            "Turn 1 Fesh-Fesh Runoff",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-630.0, 20.0),
                radius: 28.0,
            },
            SurfaceType::Sand,
            "West Carousel Sand Trap",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 18, 3);
    let grid_positions = generate_grid_positions(&spline, 12, 10.0, 3.5);

    Track {
        name: "Atacama Sand Basin".to_string(),
        description: "Hyper-speed Chilean desert basin with parabolic high-speed sweepers across dried salt flats and powdery fesh-fesh dunes.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Red Rock Canyon
/// Technical gorge circuit carved through towering red sandstone cliffs with tight hairpins, washboard gravel, and boulder obstacles.
pub fn red_rock_canyon() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-240.0, -120.0), 8.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-100.0, -120.0), 8.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(50.0, -115.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(180.0, -90.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(1.5),
        TrackWaypoint::new(Vec2::new(260.0, -20.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(3.0),
        TrackWaypoint::new(Vec2::new(250.0, 70.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(false, true).with_elevation(4.5),
        TrackWaypoint::new(Vec2::new(170.0, 130.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(6.0),
        TrackWaypoint::new(Vec2::new(60.0, 90.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(false, true).with_elevation(4.5),
        TrackWaypoint::new(Vec2::new(-30.0, 140.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(3.0),
        TrackWaypoint::new(Vec2::new(-140.0, 150.0), 8.5).with_surface(SurfaceType::Dirt).with_elevation(2.0),
        TrackWaypoint::new(Vec2::new(-240.0, 110.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(1.0),
        TrackWaypoint::new(Vec2::new(-300.0, 20.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-280.0, -60.0), 8.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.8, BarrierType::Concrete);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(115.0, 110.0),
                half_extents: Vec2::new(5.0, 5.0),
                angle: -2.62,
            },
            Vec2::new(-0.87, -0.5),
            5.0,
            16.0,
            2.2,
            "Canyon Crevasse Leap",
        ).with_surface(SurfaceType::Dirt),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(260.0, -20.0),
                radius: 14.0,
            },
            SurfaceType::Dirt,
            "Red Rock Hairpin Scree",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-300.0, 20.0),
                radius: 14.0,
            },
            SurfaceType::Dirt,
            "West Wall Scree Runoff",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 10, 8.5, 1.8);

    Track {
        name: "Red Rock Canyon".to_string(),
        description: "Technical gorge circuit carved through towering red sandstone cliffs with tight hairpins, washboard gravel, and boulder obstacles.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Baja 500 Desert Scrub
/// Grueling 3,100m open desert endurance layout with rhythm whoops sections, dry sandy wash riverbeds, and cactus hazard zones.
pub fn baja_500_desert_scrub() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-520.0, -280.0), 14.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(-320.0, -280.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-120.0, -280.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(80.0, -275.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(280.0, -260.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(480.0, -210.0), 14.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(660.0, -90.0), 14.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(720.0, 70.0), 14.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(630.0, 230.0), 14.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(440.0, 300.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(220.0, 260.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(0.0, 300.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-220.0, 290.0), 14.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-440.0, 250.0), 14.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-620.0, 150.0), 14.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-700.0, 0.0), 14.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-640.0, -160.0), 14.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 6.0, BarrierType::TireWall);

    let mut jump_ramps = generate_whoops_array(
        10,
        Vec2::new(-220.0, -280.0),
        Vec2::X,
        8,
        18.0,
        13.0,
        0.85,
        SurfaceType::Dirt,
    );

    jump_ramps.push(
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(380.0, -235.0),
                half_extents: Vec2::new(6.0, 8.0),
                angle: 0.24,
            },
            Vec2::new(0.97, 0.24),
            6.0,
            18.0,
            2.5,
            "Baja Dry Wash Launch",
        ).with_surface(SurfaceType::Sand),
    );
    jump_ramps.push(
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-330.0, 270.0),
                half_extents: Vec2::new(6.0, 8.0),
                angle: -2.96,
            },
            Vec2::new(-0.98, -0.18),
            5.5,
            16.0,
            2.2,
            "North Scrub Tabletop",
        ).with_surface(SurfaceType::Sand),
    );

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(720.0, 70.0),
                radius: 28.0,
            },
            SurfaceType::Sand,
            "Turn 1 Scrub Sand Trap",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-700.0, 0.0),
                radius: 28.0,
            },
            SurfaceType::Sand,
            "West Wash Sand Hazard",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 20, 3);
    let grid_positions = generate_grid_positions(&spline, 12, 10.0, 3.2);

    Track {
        name: "Baja 500 Desert Scrub".to_string(),
        description: "Grueling 3,100m open desert endurance layout with rhythm whoops sections, dry sandy wash riverbeds, and cactus hazard zones.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 2,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Mud Slough Arena
/// Enclosed 140m x 105m mud bog stadium bowl with deep viscous mud ruts, heavy tire walls, and central mud jump.
pub fn mud_slough_arena() -> Track {
    let hull = generate_oval_hull(Vec2::ZERO, 70.0, 52.5, 20);
    let outer_walls = generate_walls_from_hull(&hull, BarrierType::TireWall);

    let floor_zone = SurfaceZone::new(
        SurfaceShape::Polygon { vertices: hull.clone() },
        SurfaceType::Mud,
        "Mud Slough Arena Floor",
    ).with_layer(SurfaceLayer::BelowTrack);

    let puddle_zone = SurfaceZone::new(
        SurfaceShape::Circle {
            center: Vec2::new(30.0, 15.0),
            radius: 16.0,
        },
        SurfaceType::Water,
        "Deep Mud Pond Hazard",
    );

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(0.0, 16.0),
                half_extents: Vec2::new(6.0, 10.0),
                angle: 0.0,
            },
            Vec2::X,
            5.0,
            16.0,
            2.2,
            "Central Clay Mound East",
        ).with_surface(SurfaceType::Mud),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(0.0, -16.0),
                half_extents: Vec2::new(6.0, 10.0),
                angle: std::f32::consts::PI,
            },
            -Vec2::X,
            5.0,
            16.0,
            2.2,
            "Central Clay Mound West",
        ).with_surface(SurfaceType::Mud),
    ];

    let grid_positions = generate_arena_grid(Vec2::new(-20.0, 0.0), 0.0, 8, 8.0, 3.0);
    let checkpoints = vec![Checkpoint::new(
        0,
        LineSegment::new(Vec2::new(5.0, -16.0), Vec2::new(5.0, 16.0)),
        Vec2::X,
        0,
        true,
    )];

    Track {
        name: "Mud Slough Arena".to_string(),
        description: "Enclosed 140m x 105m mud bog stadium bowl with deep viscous mud ruts, heavy tire walls, and central mud jump.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Arena {
            boundary_hull: hull.clone(),
            floor_surface: SurfaceType::Mud,
            perimeter_barrier: Some(BarrierType::TireWall),
        },
        spline: TrackSpline::empty(),
        geometry: TrackGeometry {
            inner_walls: Vec::new(),
            outer_walls,
            obstacles: Vec::new(),
            surface_zones: vec![floor_zone, puddle_zone],
            jump_ramps,
            left_boundary_polyline: hull.clone(),
            right_boundary_polyline: hull,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Mud,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Gravel Quarry Chasm
/// Multi-tier industrial excavation circuit descending 4 terrace levels with sheer cliff drops, steel conveyor ramps, and haul truck obstacles.
pub fn gravel_quarry_chasm() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-260.0, -140.0), 12.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-120.0, -140.0), 12.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(40.0, -130.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(2.5),
        TrackWaypoint::new(Vec2::new(180.0, -90.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(5.5),
        TrackWaypoint::new(Vec2::new(270.0, 0.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(8.5),
        TrackWaypoint::new(Vec2::new(240.0, 100.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(false, true).with_elevation(11.5),
        TrackWaypoint::new(Vec2::new(140.0, 160.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(12.0),
        TrackWaypoint::new(Vec2::new(0.0, 130.0), 12.0).with_surface(SurfaceType::Dirt).with_elevation(9.0),
        TrackWaypoint::new(Vec2::new(-120.0, 160.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(6.0),
        TrackWaypoint::new(Vec2::new(-240.0, 130.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(3.5),
        TrackWaypoint::new(Vec2::new(-310.0, 40.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(true, false).with_elevation(1.5),
        TrackWaypoint::new(Vec2::new(-290.0, -70.0), 12.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.0, BarrierType::Steel);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(110.0, -110.0),
                half_extents: Vec2::new(6.0, 7.0),
                angle: 0.28,
            },
            Vec2::new(0.96, 0.28),
            5.0,
            16.0,
            2.4,
            "Terrace Drop Conveyor Ramp",
        ).with_surface(SurfaceType::Dirt),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(270.0, 0.0),
                radius: 18.0,
            },
            SurfaceType::Dirt,
            "Quarry East Terrace Scree",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 10, 8.5, 2.2);

    Track {
        name: "Gravel Quarry Chasm".to_string(),
        description: "Multi-tier industrial excavation circuit descending 4 terrace levels with sheer cliff drops, steel conveyor ramps, and haul truck obstacles.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Louisiana Mud Swampland
/// Deep bayou swamp course weaving between cypress trees, treacherous mud bogs, murky water hazards, and slippery wooden boardwalks.
pub fn louisiana_mud_swampland() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-220.0, -110.0), 11.0).with_surface(SurfaceType::Mud),
        TrackWaypoint::new(Vec2::new(-100.0, -110.0), 11.0).with_surface(SurfaceType::Mud),
        TrackWaypoint::new(Vec2::new(30.0, -100.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(150.0, -60.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(220.0, 20.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(190.0, 110.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(90.0, 150.0), 11.0).with_surface(SurfaceType::Mud),
        TrackWaypoint::new(Vec2::new(-20.0, 120.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-120.0, 160.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-210.0, 120.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-270.0, 30.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-250.0, -60.0), 11.0).with_surface(SurfaceType::Mud).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(35.0, 135.0),
                half_extents: Vec2::new(5.0, 6.0),
                angle: -2.85,
            },
            Vec2::new(-0.96, -0.28),
            4.5,
            16.0,
            2.0,
            "Bayou Creek Leap",
        ).with_surface(SurfaceType::Mud),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(220.0, 20.0),
                radius: 16.0,
            },
            SurfaceType::Water,
            "Bayou Water Bog Hazard",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-270.0, 30.0),
                radius: 16.0,
            },
            SurfaceType::Water,
            "West Bayou Swamp Water Hazard",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.0);

    Track {
        name: "Louisiana Mud Swampland".to_string(),
        description: "Deep bayou swamp course weaving between cypress trees, treacherous mud bogs, murky water hazards, and slippery wooden boardwalks.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Mud,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Arctic Frozen Lake
/// Expansive 240m x 160m sub-zero frozen lake drift arena flanked by deep snowbanks, low-friction ice sheet, and high-speed pendulum chicanes.
pub fn arctic_frozen_lake() -> Track {
    let lake_hull = generate_oval_hull(Vec2::ZERO, 120.0, 80.0, 24);
    let perimeter_walls = generate_walls_from_hull(&lake_hull, BarrierType::TireWall);

    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-70.0, -40.0), 14.0).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(0.0, -40.0), 14.0).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(60.0, -35.0), 14.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(85.0, 0.0), 14.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(60.0, 35.0), 14.0).with_surface(SurfaceType::Ice).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(10.0, 25.0), 14.0).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(-30.0, 40.0), 14.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-80.0, 20.0), 14.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-85.0, -15.0), 14.0).with_surface(SurfaceType::Ice).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 10, 8.0, 2.5);

    let floor_zone = SurfaceZone::new(
        SurfaceShape::Polygon { vertices: lake_hull.clone() },
        SurfaceType::Ice,
        "Frozen Lake Sheet",
    ).with_layer(SurfaceLayer::BelowTrack);

    let snowbank_1 = SurfaceZone::new(
        SurfaceShape::Circle {
            center: Vec2::new(85.0, 0.0),
            radius: 14.0,
        },
        SurfaceType::Snow,
        "Turn 1 Snowbank Berm",
    );
    let snowbank_2 = SurfaceZone::new(
        SurfaceShape::Circle {
            center: Vec2::new(-80.0, 20.0),
            radius: 14.0,
        },
        SurfaceType::Snow,
        "West Chicane Snowbank",
    );

    Track {
        name: "Arctic Frozen Lake".to_string(),
        description: "Expansive 240m x 160m sub-zero frozen lake drift arena flanked by deep snowbanks, low-friction ice sheet, and high-speed pendulum chicanes.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Hybrid {
            boundary_hull: lake_hull.clone(),
            floor_surface: SurfaceType::Ice,
            perimeter_barrier: Some(BarrierType::TireWall),
        },
        spline,
        geometry: TrackGeometry {
            inner_walls: Vec::new(),
            outer_walls: perimeter_walls,
            obstacles: Vec::new(),
            surface_zones: vec![floor_zone, snowbank_1, snowbank_2],
            jump_ramps: Vec::new(),
            left_boundary_polyline: lake_hull.clone(),
            right_boundary_polyline: lake_hull,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Snow,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Alpine Snow Ridge
/// Point-to-point alpine snow hillclimb climbing 18 vertical meters with knife-edge cliff edges, packed snow berms, and black ice patches.
pub fn alpine_snow_ridge() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-280.0, -150.0), 10.0).with_surface(SurfaceType::Snow),
        TrackWaypoint::new(Vec2::new(-140.0, -150.0), 10.0).with_surface(SurfaceType::Snow).with_elevation(2.0),
        TrackWaypoint::new(Vec2::new(10.0, -140.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(true, false).with_elevation(5.0),
        TrackWaypoint::new(Vec2::new(160.0, -100.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(true, false).with_elevation(8.5),
        TrackWaypoint::new(Vec2::new(260.0, -20.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(true, false).with_elevation(12.0),
        TrackWaypoint::new(Vec2::new(250.0, 80.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(false, true).with_elevation(15.5),
        TrackWaypoint::new(Vec2::new(160.0, 160.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(true, false).with_elevation(16.0),
        TrackWaypoint::new(Vec2::new(20.0, 130.0), 10.0).with_surface(SurfaceType::Snow).with_elevation(12.5),
        TrackWaypoint::new(Vec2::new(-100.0, 170.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(true, false).with_elevation(9.0),
        TrackWaypoint::new(Vec2::new(-220.0, 140.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(true, false).with_elevation(5.5),
        TrackWaypoint::new(Vec2::new(-300.0, 50.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(true, false).with_elevation(2.5),
        TrackWaypoint::new(Vec2::new(-310.0, -60.0), 10.0).with_surface(SurfaceType::Snow).with_curbs(false, true).with_elevation(0.5),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.0, BarrierType::Steel);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(90.0, 145.0),
                half_extents: Vec2::new(5.0, 6.0),
                angle: -2.93,
            },
            Vec2::new(-0.98, -0.21),
            5.0,
            16.0,
            2.2,
            "Alpine Peak Crest Launch",
        ).with_surface(SurfaceType::Snow),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(250.0, 80.0),
                radius: 14.0,
            },
            SurfaceType::Ice,
            "Summit Switchback Black Ice",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-220.0, 140.0),
                radius: 14.0,
            },
            SurfaceType::Ice,
            "Descent Hairpin Black Ice",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 10, 8.0, 2.0);

    Track {
        name: "Alpine Snow Ridge".to_string(),
        description: "Point-to-point alpine snow hillclimb climbing 18 vertical meters with knife-edge cliff edges, packed snow berms, and black ice patches.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Snow,
        pit_box_area: None,
        default_laps: 2,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Rovaniemi Ice Ring
/// Finnish frozen lake circuit featuring high-speed ice sweepers, packed snow berms, and rhythmic pendulum chicane complexes.
pub fn rovaniemi_ice_ring() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-190.0, -90.0), 13.0).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(-70.0, -90.0), 13.0).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(60.0, -85.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(170.0, -40.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(210.0, 30.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(150.0, 110.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(50.0, 100.0), 13.0).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(-50.0, 120.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-150.0, 105.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-210.0, 35.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-220.0, -40.0), 13.0).with_surface(SurfaceType::Ice).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(210.0, 30.0),
                radius: 16.0,
            },
            SurfaceType::Snow,
            "East Sweeper Snowbank",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-210.0, 35.0),
                radius: 16.0,
            },
            SurfaceType::Snow,
            "West Chicane Snowbank",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Rovaniemi Ice Ring".to_string(),
        description: "Finnish frozen lake circuit featuring high-speed ice sweepers, packed snow berms, and rhythmic pendulum chicane complexes.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Snow,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Glacier Crest Pass
/// Treacherous glacial ridge carved between bottomless ice chasms with crevasse gap jumps and zero barrier protection.
pub fn glacier_crest_pass() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-300.0, -160.0), 11.5).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(-140.0, -160.0), 11.5).with_surface(SurfaceType::Ice),
        TrackWaypoint::new(Vec2::new(30.0, -150.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(190.0, -110.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(true, false).with_elevation(2.0),
        TrackWaypoint::new(Vec2::new(290.0, -20.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(true, false).with_elevation(5.0),
        TrackWaypoint::new(Vec2::new(270.0, 90.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(false, true).with_elevation(8.0),
        TrackWaypoint::new(Vec2::new(160.0, 170.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(true, false).with_elevation(10.0),
        TrackWaypoint::new(Vec2::new(10.0, 140.0), 11.5).with_surface(SurfaceType::Ice).with_elevation(7.0),
        TrackWaypoint::new(Vec2::new(-120.0, 180.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(true, false).with_elevation(4.5),
        TrackWaypoint::new(Vec2::new(-240.0, 140.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(true, false).with_elevation(2.0),
        TrackWaypoint::new(Vec2::new(-330.0, 50.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-340.0, -70.0), 11.5).with_surface(SurfaceType::Ice).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Steel);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(110.0, -130.0),
                half_extents: Vec2::new(5.0, 6.5),
                angle: 0.24,
            },
            Vec2::new(0.97, 0.24),
            6.0,
            18.0,
            2.8,
            "Glacial Crevasse Leap East",
        ).with_surface(SurfaceType::Ice),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-50.0, 160.0),
                half_extents: Vec2::new(5.0, 6.5),
                angle: -2.85,
            },
            Vec2::new(-0.96, -0.28),
            5.5,
            16.0,
            2.4,
            "North Ridge Abyss Jump",
        ).with_surface(SurfaceType::Ice),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(290.0, -20.0),
                radius: 16.0,
            },
            SurfaceType::Snow,
            "Glacial Firn Drift East",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-330.0, 50.0),
                radius: 16.0,
            },
            SurfaceType::Snow,
            "West Firn Snow Hazard",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.0);

    Track {
        name: "Glacier Crest Pass".to_string(),
        description: "Treacherous glacial ridge carved between bottomless ice chasms with crevasse gap jumps and zero barrier protection.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Ice,
        pit_box_area: None,
        default_laps: 2,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Supercross Stadium Arena
/// Domed indoor football stadium featuring 6-lane clay rhythm sections, 22-degree banked bowl turns, supercross triples, and washboard whoops.
pub fn supercross_stadium_arena() -> Track {
    let stadium_hull = generate_oval_hull(Vec2::ZERO, 80.0, 57.5, 20);
    let perimeter_walls = generate_walls_from_hull(&stadium_hull, BarrierType::Concrete);

    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-50.0, -32.0), 10.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(0.0, -32.0), 10.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(45.0, -30.0), 10.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(60.0, -5.0), 10.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(40.0, 15.0), 10.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(0.0, 5.0), 10.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-35.0, 15.0), 10.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-60.0, 0.0), 10.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-45.0, -20.0), 10.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let checkpoints = generate_checkpoints(&spline, 10, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    let floor_zone = SurfaceZone::new(
        SurfaceShape::Polygon { vertices: stadium_hull.clone() },
        SurfaceType::Dirt,
        "Supercross Stadium Clay Floor",
    ).with_layer(SurfaceLayer::BelowTrack);

    let mut jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(20.0, -32.0),
                half_extents: Vec2::new(5.0, 6.0),
                angle: 0.04,
            },
            Vec2::new(0.999, 0.04),
            6.5,
            20.0,
            3.0,
            "Main Straight Supercross Triple",
        ).with_surface(SurfaceType::Dirt),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(20.0, 10.0),
                half_extents: Vec2::new(5.0, 6.0),
                angle: -3.0,
            },
            Vec2::new(-0.99, -0.14),
            6.0,
            18.0,
            2.5,
            "Infield Rhythm Tabletop",
        ).with_surface(SurfaceType::Dirt),
    ];

    let whoops = generate_whoops_array(
        10,
        Vec2::new(-45.0, -32.0),
        Vec2::X,
        5,
        10.0,
        9.5,
        0.8,
        SurfaceType::Dirt,
    );
    jump_ramps.extend(whoops);

    Track {
        name: "Supercross Stadium Arena".to_string(),
        description: "Domed indoor football stadium featuring 6-lane clay rhythm sections, 22-degree banked bowl turns, supercross triples, and washboard whoops.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Hybrid {
            boundary_hull: stadium_hull.clone(),
            floor_surface: SurfaceType::Dirt,
            perimeter_barrier: Some(BarrierType::Concrete),
        },
        spline,
        geometry: TrackGeometry {
            inner_walls: Vec::new(),
            outer_walls: perimeter_walls,
            obstacles: Vec::new(),
            surface_zones: vec![floor_zone],
            jump_ramps,
            left_boundary_polyline: stadium_hull.clone(),
            right_boundary_polyline: stadium_hull,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Monster Colosseum
/// Monumental 190m x 140m demolition stunt arena featuring twin 45-degree monster kickers, central car-crush pyramid, and pyrotechnic towers.
pub fn monster_colosseum() -> Track {
    let colosseum_hull = generate_oval_hull(Vec2::ZERO, 95.0, 70.0, 24);
    let outer_walls = generate_walls_from_hull(&colosseum_hull, BarrierType::Concrete);

    let floor_zone = SurfaceZone::new(
        SurfaceShape::Polygon { vertices: colosseum_hull.clone() },
        SurfaceType::Dirt,
        "Monster Colosseum Floor",
    ).with_layer(SurfaceLayer::BelowTrack);

    let crush_zone = SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(-20.0, -12.0),
            max: Vec2::new(20.0, 12.0),
        },
        SurfaceType::Dirt,
        "Car Crush Tabletop",
    );

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-30.0, 25.0),
                half_extents: Vec2::new(6.0, 10.0),
                angle: 0.0,
            },
            Vec2::X,
            7.0,
            28.0,
            4.2,
            "Monster Kicker East",
        ).with_surface(SurfaceType::Dirt),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(30.0, -25.0),
                half_extents: Vec2::new(6.0, 10.0),
                angle: std::f32::consts::PI,
            },
            -Vec2::X,
            7.0,
            28.0,
            4.2,
            "Monster Kicker West",
        ).with_surface(SurfaceType::Dirt),
    ];

    let obstacles = vec![
        Obstacle::oriented_box(1, Vec2::new(0.0, 0.0), Vec2::new(12.0, 4.0), 0.0, "Crushed Cars Row"),
        Obstacle::circle(2, Vec2::new(-60.0, 45.0), 2.5, "Pyrotechnic Tower NW"),
        Obstacle::circle(3, Vec2::new(60.0, 45.0), 2.5, "Pyrotechnic Tower NE"),
        Obstacle::circle(4, Vec2::new(-60.0, -45.0), 2.5, "Pyrotechnic Tower SW"),
        Obstacle::circle(5, Vec2::new(60.0, -45.0), 2.5, "Pyrotechnic Tower SE"),
    ];

    let grid_positions = generate_arena_grid(Vec2::new(-40.0, 0.0), 0.0, 10, 8.0, 3.5);
    let checkpoints = vec![Checkpoint::new(
        0,
        LineSegment::new(Vec2::new(0.0, -20.0), Vec2::new(0.0, 20.0)),
        Vec2::X,
        0,
        true,
    )];

    Track {
        name: "Monster Colosseum".to_string(),
        description: "Monumental 190m x 140m demolition stunt arena featuring twin 45-degree monster kickers, central car-crush pyramid, and pyrotechnic towers.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Arena {
            boundary_hull: colosseum_hull.clone(),
            floor_surface: SurfaceType::Dirt,
            perimeter_barrier: Some(BarrierType::Concrete),
        },
        spline: TrackSpline::empty(),
        geometry: TrackGeometry {
            inner_walls: Vec::new(),
            outer_walls,
            obstacles,
            surface_zones: vec![floor_zone, crush_zone],
            jump_ramps,
            left_boundary_polyline: colosseum_hull.clone(),
            right_boundary_polyline: colosseum_hull,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Stunt City Megastructure
/// Multi-level 220m x 170m urban stunt plaza with mega kickers, skyscraper wallrides, elevated gap ramps, and aerial stunt targets.
pub fn stunt_city_megastructure() -> Track {
    let plaza_hull = generate_oval_hull(Vec2::ZERO, 110.0, 85.0, 24);
    let outer_walls = generate_walls_from_hull(&plaza_hull, BarrierType::Steel);

    let floor_zone = SurfaceZone::new(
        SurfaceShape::Polygon { vertices: plaza_hull.clone() },
        SurfaceType::Asphalt,
        "Stunt Plaza Asphalt Floor",
    ).with_layer(SurfaceLayer::BelowTrack);

    let elevated_deck = SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(20.0, -25.0),
            max: Vec2::new(70.0, 25.0),
        },
        SurfaceType::Asphalt,
        "Elevated Stunt Deck",
    );

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-45.0, 0.0),
                half_extents: Vec2::new(7.0, 12.0),
                angle: 0.0,
            },
            Vec2::X,
            8.5,
            32.0,
            4.8,
            "Downtown Mega Kicker",
        ).with_surface(SurfaceType::Asphalt),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(45.0, 30.0),
                half_extents: Vec2::new(6.0, 10.0),
                angle: std::f32::consts::PI,
            },
            -Vec2::X,
            7.5,
            24.0,
            3.8,
            "Plaza Rooftop Gap Launch",
        ).with_surface(SurfaceType::Asphalt),
    ];

    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(-10.0, 35.0), 2.0, "Structural Column North"),
        Obstacle::circle(2, Vec2::new(-10.0, -35.0), 2.0, "Structural Column South"),
    ];

    let grid_positions = generate_arena_grid(Vec2::new(-50.0, 0.0), 0.0, 10, 8.0, 3.5);
    let checkpoints = vec![Checkpoint::new(
        0,
        LineSegment::new(Vec2::new(-10.0, -22.0), Vec2::new(-10.0, 22.0)),
        Vec2::X,
        0,
        true,
    )];

    Track {
        name: "Stunt City Megastructure".to_string(),
        description: "Multi-level 220m x 170m urban stunt plaza with mega kickers, skyscraper wallrides, elevated gap ramps, and aerial stunt targets.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Arena {
            boundary_hull: plaza_hull.clone(),
            floor_surface: SurfaceType::Asphalt,
            perimeter_barrier: Some(BarrierType::Steel),
        },
        spline: TrackSpline::empty(),
        geometry: TrackGeometry {
            inner_walls: Vec::new(),
            outer_walls,
            obstacles,
            surface_zones: vec![floor_zone, elevated_deck],
            jump_ramps,
            left_boundary_polyline: plaza_hull.clone(),
            right_boundary_polyline: plaza_hull,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Asphalt,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Bowman Gray Stadium (The Madhouse)
/// Historic 0.25-mile flat asphalt bullring short track in Winston-Salem, NC surveyed from OpenStreetMap (OSM):
/// claustrophobic flat turns surrounded by football stadium grandstands and continuous bumper-to-bumper action.
pub fn bowman_gray_stadium() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(25.1, -1.4), 14.0),
        TrackWaypoint::new(Vec2::new(50.2, -2.8), 14.0),
        TrackWaypoint::new(Vec2::new(75.3, -4.2), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(100.0, -1.7), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(118.5, 14.5), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(121.8, 38.8), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(106.8, 58.2), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(83.2, 66.0), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(58.1, 67.5), 14.0),
        TrackWaypoint::new(Vec2::new(33.0, 68.7), 14.0),
        TrackWaypoint::new(Vec2::new(7.9, 69.9), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-16.9, 67.3), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-35.6, 51.5), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-38.5, 27.3), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-23.7, 7.6), 14.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.6, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 16, 6.5, 2.8);

    Track {
        name: "Bowman Gray Stadium".to_string(),
        description: "The Madhouse: historic 0.25-mile flat asphalt bullring short track in Winston-Salem, NC with claustrophobic turns and relentless contact.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(40.0, -18.0), 90.0, 16.0, 0.0)
                    .with_style(GrandstandStyle::OpenBleachers)
                    .with_tiers(12)
                    .with_seat_color([0.80, 0.25, 0.20]),
                Grandstand::new(2, Vec2::new(40.0, 85.0), 90.0, 16.0, std::f32::consts::PI)
                    .with_style(GrandstandStyle::OpenBleachers)
                    .with_tiers(12)
                    .with_seat_color([0.20, 0.45, 0.85]),
            ],
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Lucas Oil Indianapolis Raceway Park (IRP)
/// Classic 0.686-mile asphalt short oval in Clermont, Indiana surveyed from OpenStreetMap (OSM):
/// 12° banking in turns 1-4, progressive transitions, and tight apron passing lines.
pub fn lucas_oil_irp() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(55.1, -3.7), 16.0).with_bank_angle(2.0),
        TrackWaypoint::new(Vec2::new(110.1, -7.5), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(162.0, 7.7), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(201.5, 45.1), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(219.0, 96.9), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(208.6, 150.4), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(174.5, 193.2), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(124.7, 215.0), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(69.7, 219.4), 16.0).with_bank_angle(2.0),
        TrackWaypoint::new(Vec2::new(14.6, 223.6), 16.0).with_bank_angle(2.0),
        TrackWaypoint::new(Vec2::new(-40.4, 227.8), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-95.5, 230.6), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-148.1, 216.1), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-187.9, 179.0), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-203.9, 126.9), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-194.3, 73.4), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-160.7, 30.5), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-109.9, 10.3), 16.0).with_bank_angle(12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-55.0, 5.0), 16.0).with_bank_angle(2.0),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.2, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 16, 7.5, 3.0);

    Track {
        name: "Lucas Oil Indianapolis Raceway Park".to_string(),
        description: "Classic 0.686-mile asphalt oval with 12-degree banking in Clermont, Indiana featuring multi-groove passing lines.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(0.0, -22.0), 120.0, 16.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(10)
                    .with_seat_color([0.85, 0.30, 0.20]),
            ],
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: North Wilkesboro Speedway
/// Historic moonshine-era 0.625-mile short track in North Carolina surveyed from OpenStreetMap (OSM):
/// distinctive downhill frontstretch, uphill backstretch, 14° turn banking, and abrasive high-tire-wear asphalt.
pub fn north_wilkesboro_speedway() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, -0.0), 16.0).with_elevation(-1.5).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(50.2, -3.3), 16.0).with_elevation(-1.5).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(100.4, -6.6), 16.0).with_elevation(-1.5).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(150.5, -8.8), 16.0).with_elevation(-1.5).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(200.4, -3.5), 16.0).with_elevation(-1.5).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(246.7, 14.9), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(277.8, 53.3), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(282.2, 102.3), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(261.7, 147.5), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(221.8, 176.2), 16.0).with_elevation(2.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(172.6, 185.8), 16.0).with_elevation(2.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(122.4, 188.3), 16.0).with_elevation(2.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(72.1, 190.8), 16.0).with_elevation(2.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(21.9, 193.0), 16.0).with_elevation(2.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-28.2, 189.4), 16.0).with_elevation(2.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-74.8, 172.0), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-105.3, 133.6), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-109.8, 83.9), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-90.3, 38.5), 16.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-48.8, 11.9), 16.0).with_bank_angle(14.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.2, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 16, 7.5, 3.0);

    Track {
        name: "North Wilkesboro Speedway".to_string(),
        description: "Historic 0.625-mile short track featuring a distinctive downhill frontstretch, uphill backstretch, and 14° banking.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(80.0, -22.0), 100.0, 16.0, 0.0)
                    .with_style(GrandstandStyle::OpenBleachers)
                    .with_tiers(8)
                    .with_seat_color([0.70, 0.50, 0.30]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(80.0, 210.0), TreeType::Oak).with_scale(1.4),
                Tree::new(2, Vec2::new(140.0, 215.0), TreeType::Pine).with_scale(1.3),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Pocono Raceway (The Tricky Triangle)
/// Legendary 2.5-mile tri-oval superspeedway in Long Pond, PA surveyed from OpenStreetMap (OSM) scaled to 0.5x (2,011.5m):
/// three distinct turn radiuses and banking angles modeled after Trenton (14°), Indianapolis (8°), and Milwaukee (6°).
pub fn pocono_raceway() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 18.0),
        TrackWaypoint::new(Vec2::new(83.7, -4.7), 18.0),
        TrackWaypoint::new(Vec2::new(167.4, -8.8), 18.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(242.9, 22.7), 18.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(282.3, 95.2), 18.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(266.8, 175.9), 18.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(218.5, 244.3), 18.0).with_bank_angle(14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(168.8, 311.9), 18.0).with_bank_angle(14.0),
        TrackWaypoint::new(Vec2::new(119.1, 379.4), 18.0),
        TrackWaypoint::new(Vec2::new(69.5, 446.9), 18.0),
        TrackWaypoint::new(Vec2::new(19.8, 514.4), 18.0),
        TrackWaypoint::new(Vec2::new(-29.4, 582.2), 18.0).with_bank_angle(8.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-79.4, 649.5), 18.0).with_bank_angle(8.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-152.9, 684.4), 18.0).with_bank_angle(8.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-229.3, 656.3), 18.0).with_bank_angle(8.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-262.3, 582.1), 18.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-256.6, 498.5), 18.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-249.5, 415.0), 18.0),
        TrackWaypoint::new(Vec2::new(-242.6, 331.5), 18.0).with_bank_angle(6.0),
        TrackWaypoint::new(Vec2::new(-235.7, 247.9), 18.0).with_bank_angle(6.0),
        TrackWaypoint::new(Vec2::new(-228.9, 164.4), 18.0).with_bank_angle(6.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-218.4, 81.5), 18.0).with_bank_angle(6.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-165.4, 19.1), 18.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-83.7, 5.0), 18.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.0, BarrierType::Steel);

    let checkpoints = generate_checkpoints(&spline, 12, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 9.0, 3.2);

    Track {
        name: "Pocono Raceway".to_string(),
        description: "The Tricky Triangle: 2.5-mile tri-oval superspeedway scaled to 0.5x (2,011.5m) with 3 distinct banked turns (14°/8°/6°) and massive straights.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(0.0, -25.0), 160.0, 18.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(14)
                    .with_seat_color([0.20, 0.40, 0.80]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(-100.0, 350.0), TreeType::Pine).with_scale(1.4),
                Tree::new(2, Vec2::new(100.0, 350.0), TreeType::Pine).with_scale(1.5),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "0.5x".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Phoenix Raceway
/// Premier 1.0-mile low-banked tri-oval in Avondale, Arizona surveyed from OpenStreetMap (OSM):
/// 8°-11° progressive banking, frontstretch start/finish dogleg cut across the asphalt apron, and dramatic desert surroundings.
pub fn phoenix_raceway() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(66.9, -4.3), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(133.8, -8.6), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(200.7, -13.0), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(267.6, -17.3), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(334.5, -21.6), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(401.0, -19.6), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(457.3, 15.1), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(486.5, 74.6), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(479.9, 140.1), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(444.0, 196.2), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(396.1, 242.9), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(345.0, 286.3), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(287.9, 320.9), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(221.3, 325.1), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(154.4, 320.6), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(87.8, 312.5), 18.0).with_bank_angle(3.0),
        TrackWaypoint::new(Vec2::new(21.3, 304.5), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-44.3, 291.7), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-98.0, 252.9), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-130.1, 194.6), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-135.3, 128.4), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-111.4, 66.1), 18.0).with_bank_angle(10.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-63.0, 20.6), 18.0).with_bank_angle(10.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 10, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 8.5, 3.2);

    Track {
        name: "Phoenix Raceway".to_string(),
        description: "1.0-mile low-banked tri-oval in Avondale, Arizona featuring the famous dogleg cut across the apron and 8°-11° progressive banking.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(180.0, -25.0), 140.0, 16.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(12)
                    .with_seat_color([0.85, 0.40, 0.15]),
            ],
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Nascar,
        module_id: Some("nascar".to_string()),
        modules: vec!["nascar".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Circuit de l'Ouest Parisien (Dreux RX)
/// Historic French rallycross championship circuit in Normandy surveyed from OpenStreetMap (OSM):
/// 1,048m mixed-surface ribbon (62% Asphalt / 38% Dirt), high-speed sweeping tarmac start, and technical loose dirt hairpins.
pub fn dreux_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(37.4, 0.4), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(74.9, 0.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(110.3, -7.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(123.8, -40.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(102.7, -70.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(71.6, -90.9), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(39.9, -110.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(3.4, -115.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-29.9, -99.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-54.5, -71.6), 13.5).with_surface(SurfaceType::Asphalt),
        TrackWaypoint::new(Vec2::new(-79.0, -43.3), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-103.6, -15.1), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-136.0, 10.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-148.0, 42.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-110.0, 14.6), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-87.6, -11.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-59.3, -36.0), 13.5).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-32.7, -62.3), 13.5).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-6.0, -88.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(29.0, -86.4), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(63.3, -71.5), 13.5).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(92.7, -50.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(78.4, -22.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(48.9, -44.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(20.2, -67.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-9.5, -49.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-29.9, -18.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.8, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-45.0, -95.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: -2.35,
            },
            Vec2::new(-0.70, -0.71),
            2.2,
            5.5,
            1.3,
            "Bois Guyon Dirt Leap",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Circuit de l'Ouest Parisien (Dreux RX)".to_string(),
        description: "French Rallycross Championship venue in Dreux featuring high-speed sweeping tarmac, technical loose dirt hairpins, and tabletop jump.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(60.0, 20.0), 60.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(8)
                    .with_seat_color([0.20, 0.40, 0.85]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(-110.0, 20.0), TreeType::Oak).with_scale(1.4),
                Tree::new(2, Vec2::new(-120.0, -50.0), TreeType::Pine).with_scale(1.3),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Blyton Park Rallycross Circuit
/// British rallycross proving ground on a former RAF airfield in Lincolnshire surveyed from OpenStreetMap (OSM):
/// 1,180m mixed-surface track (58% Asphalt / 42% Gravel), flowing flat curves, and technical jump ramp crest.
pub fn blyton_park_rx() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(33.6, -18.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(66.8, -42.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(104.2, -24.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(137.9, 0.8), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(170.4, 27.6), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(211.4, 34.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(248.8, 53.7), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(281.3, 80.2), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(315.3, 102.1), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(346.3, 80.3), 13.5).with_surface(SurfaceType::Gravel).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(351.1, 38.5), 13.5).with_surface(SurfaceType::Gravel).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(349.7, -3.5), 13.5).with_surface(SurfaceType::Gravel).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(316.9, -27.2), 13.5).with_surface(SurfaceType::Gravel).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(278.4, -44.2), 13.5).with_surface(SurfaceType::Gravel),
        TrackWaypoint::new(Vec2::new(239.9, -61.4), 13.5).with_surface(SurfaceType::Gravel),
        TrackWaypoint::new(Vec2::new(201.4, -78.7), 13.5).with_surface(SurfaceType::Gravel).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(164.8, -91.6), 13.5).with_surface(SurfaceType::Gravel).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(128.2, -111.2), 13.5).with_surface(SurfaceType::Gravel).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(89.7, -128.2), 13.5).with_surface(SurfaceType::Gravel),
        TrackWaypoint::new(Vec2::new(51.1, -145.2), 13.5).with_surface(SurfaceType::Gravel),
        TrackWaypoint::new(Vec2::new(12.3, -161.6), 13.5).with_surface(SurfaceType::Gravel).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-29.1, -169.0), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-69.2, -159.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-83.5, -122.3), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-93.3, -84.9), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-63.5, -55.4), 13.5).with_surface(SurfaceType::Asphalt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-31.7, -27.7), 13.5).with_surface(SurfaceType::Asphalt),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(120.0, -120.0),
                half_extents: Vec2::new(3.8, 5.5),
                angle: 1.57,
            },
            Vec2::new(0.0, 1.0),
            2.2,
            5.5,
            1.3,
            "Airfield Crest Jump",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
    let grid_positions = generate_grid_positions(&spline, 8, 8.5, 2.8);

    Track {
        name: "Blyton Park Rallycross Circuit".to_string(),
        description: "British classic on former RAF airfield in Lincolnshire featuring flowing flat curves, loose gravel transitions, and jump crest.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(100.0, 25.0), 70.0, 14.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(8)
                    .with_seat_color([0.85, 0.25, 0.20]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(280.0, -50.0), TreeType::Oak).with_scale(1.3),
                Tree::new(2, Vec2::new(320.0, 40.0), TreeType::Pine).with_scale(1.2),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 4,
        car_category: CarCategory::Rally,
        module_id: Some("rally".to_string()),
        modules: vec!["rally".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Laval Karting (Circuit Louis Beuvron)
/// Historic French CIK-FIA Grade 1 karting circuit in Mayenne surveyed from OpenStreetMap (OSM):
/// 1,232m technical layout with banked parabolique, rapid esses, and tight passing hairpins.
pub fn laval_kart() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(35.1, 16.6), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(75.9, 11.8), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(115.5, 0.9), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(123.6, -33.0), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(96.3, -42.8), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(73.8, -11.8), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(34.3, -3.8), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(22.4, -39.0), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(28.0, -79.7), 8.5),
        TrackWaypoint::new(Vec2::new(33.6, -120.3), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(49.7, -154.7), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(79.0, -133.8), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(73.4, -96.4), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(52.4, -64.2), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(84.2, -65.4), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(117.4, -87.5), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(146.7, -63.6), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(159.5, -24.5), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(189.9, -23.7), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(181.7, -63.2), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(156.4, -95.6), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(121.6, -117.0), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(99.1, -150.1), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(86.1, -189.0), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(50.8, -190.2), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(22.6, -160.7), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(10.8, -122.0), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(5.7, -81.3), 8.5),
        TrackWaypoint::new(Vec2::new(0.4, -40.6), 8.5).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.0, BarrierType::TireWall);

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 6.0, 2.2);

    Track {
        name: "Laval Karting (Circuit Louis Beuvron)".to_string(),
        description: "Legendary French CIK-FIA Grade 1 karting arena in Mayenne featuring banked parabolique, rapid esses, and technical chicanes.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(70.0, 25.0), 50.0, 10.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(6)
                    .with_seat_color([0.25, 0.50, 0.85]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(120.0, -100.0), TreeType::Oak).with_scale(1.3),
                Tree::new(2, Vec2::new(150.0, -150.0), TreeType::AutumnMaple).with_scale(1.2),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Kart,
        module_id: Some("kart".to_string()),
        modules: vec!["kart".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Whilton Mill Kart Circuit
/// Premier British National kart circuit in Northamptonshire surveyed from OpenStreetMap (OSM):
/// 1,200m technical course featuring Ashby hairpin, Zulu chicane, Christmas Corner, and flowing elevation drops.
pub fn whilton_mill_kart() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(37.1, 3.2), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(37.3, 39.0), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(4.6, 61.7), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-2.9, 98.4), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(9.5, 136.4), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(32.2, 168.9), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(64.7, 191.7), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(100.6, 209.4), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(138.4, 222.1), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(176.9, 218.6), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(198.8, 187.6), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(197.3, 147.8), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(192.5, 108.1), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(166.0, 89.1), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(144.0, 120.6), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(130.0, 157.5), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(93.5, 164.6), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(69.2, 134.8), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(87.8, 103.3), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(123.9, 86.2), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(150.8, 57.4), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(162.3, 19.9), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(151.8, -18.4), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(129.8, -50.9), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(91.8, -61.4), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(53.0, -51.7), 8.5),
        TrackWaypoint::new(Vec2::new(14.0, -43.1), 8.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-25.4, -39.6), 8.5).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-36.3, -6.0), 8.5).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 2.0, BarrierType::TireWall);

    let checkpoints = generate_checkpoints(&spline, 14, 3);
    let grid_positions = generate_grid_positions(&spline, 16, 6.0, 2.2);

    Track {
        name: "Whilton Mill Kart Circuit".to_string(),
        description: "Premier British National karting venue in Northamptonshire featuring challenging downhill esses, Ashby hairpin, and rapid chicanes.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(80.0, -20.0), 60.0, 10.0, 0.0)
                    .with_style(GrandstandStyle::CoveredStadium)
                    .with_tiers(6)
                    .with_seat_color([0.85, 0.25, 0.20]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(100.0, 100.0), TreeType::Oak).with_scale(1.4),
                Tree::new(2, Vec2::new(160.0, 180.0), TreeType::Pine).with_scale(1.3),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
        default_laps: 5,
        car_category: CarCategory::Kart,
        module_id: Some("kart".to_string()),
        modules: vec!["kart".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Glamis Imperial Sand Dunes
/// Open California sand bowl raid venue across rolling razorback dunes with 3 high-launch tabletop jumps and deep sand traps.
pub fn glamis_sand_dunes() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(-380.0, -200.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(-180.0, -200.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(60.0, -200.0), 16.0).with_surface(SurfaceType::Sand),
        TrackWaypoint::new(Vec2::new(260.0, -180.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(420.0, -100.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(460.0, 40.0), 16.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(390.0, 180.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(240.0, 250.0), 16.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(90.0, 220.0), 16.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(-40.0, 270.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-180.0, 280.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-350.0, 230.0), 16.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-460.0, 120.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-450.0, -30.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-400.0, -150.0), 16.0).with_surface(SurfaceType::Sand).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 6.0, BarrierType::TireWall);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(160.0, -190.0),
                half_extents: Vec2::new(6.0, 9.0),
                angle: 0.08,
            },
            Vec2::new(0.99, 0.08),
            5.5,
            16.0,
            2.2,
            "Oldsmobile Hill Leap",
        ).with_surface(SurfaceType::Sand),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(165.0, 235.0),
                half_extents: Vec2::new(6.0, 9.0),
                angle: 2.94,
            },
            Vec2::new(-0.98, 0.20),
            6.0,
            18.0,
            2.6,
            "Glamis Razorback Crest",
        ).with_surface(SurfaceType::Sand),
        JumpRamp::new(
            3,
            SurfaceShape::OrientedBox {
                center: Vec2::new(-400.0, 175.0),
                half_extents: Vec2::new(6.0, 9.0),
                angle: -2.44,
            },
            Vec2::new(-0.76, -0.65),
            5.0,
            15.0,
            2.0,
            "Dune Bowl Launch",
        ).with_surface(SurfaceType::Sand),
    ];

    let surface_zones = vec![
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(430.0, -100.0),
                radius: 22.0,
            },
            SurfaceType::Sand,
            "Turn 1 Deep Sand Trap",
        ),
        SurfaceZone::new(
            SurfaceShape::Circle {
                center: Vec2::new(-460.0, 120.0),
                radius: 24.0,
            },
            SurfaceType::Sand,
            "West Ridge Sand Bowl",
        ),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
    let grid_positions = generate_grid_positions(&spline, 12, 10.0, 3.5);

    Track {
        name: "Glamis Imperial Sand Dunes".to_string(),
        description: "Open California sand bowl with natural razorback dune crests, sweeping high-speed bowls, and triple air jumps.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones,
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: Vec::new(),
            trees: Vec::new(),
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Sand,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}

/// Preset: Crandon International Off-Road (The Big House)
/// Legendary 2,414m short-course off-road track in Wisconsin surveyed from OpenStreetMap (OSM):
/// wide high-speed clay straights, the famous Land Rush start, Barn Turn, and tabletop dirt jumps.
pub fn crandon_short_course() -> Track {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(73.7, -13.1), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(148.8, -8.2), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(222.9, 4.6), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(261.5, 55.7), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(195.2, 62.8), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(120.9, 52.6), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(46.2, 61.1), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(-11.6, 98.6), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(48.6, 116.1), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(116.3, 83.1), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(183.1, 108.5), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(237.1, 161.1), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(308.4, 166.9), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(377.7, 137.7), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(398.4, 193.2), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(388.0, 267.2), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(395.2, 342.3), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(396.0, 417.7), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(410.1, 491.7), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(425.6, 565.5), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(386.1, 547.5), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(362.7, 475.8), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(339.4, 404.1), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(316.0, 332.4), 18.0).with_surface(SurfaceType::Dirt),
        TrackWaypoint::new(Vec2::new(292.2, 260.8), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(254.4, 197.3), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(185.1, 171.5), 18.0).with_surface(SurfaceType::Dirt).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(109.8, 169.7), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(35.6, 160.4), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-31.4, 128.5), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-44.8, 58.5), 18.0).with_surface(SurfaceType::Dirt).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.0, BarrierType::Steel);

    let jump_ramps = vec![
        JumpRamp::new(
            1,
            SurfaceShape::OrientedBox {
                center: Vec2::new(150.0, 50.0),
                half_extents: Vec2::new(5.0, 8.0),
                angle: 0.15,
            },
            Vec2::new(0.99, 0.15),
            3.0,
            6.0,
            1.5,
            "Potawatomi Tabletop",
        ).with_surface(SurfaceType::Dirt),
        JumpRamp::new(
            2,
            SurfaceShape::OrientedBox {
                center: Vec2::new(280.0, 350.0),
                half_extents: Vec2::new(5.0, 8.0),
                angle: 1.57,
            },
            Vec2::new(0.0, 1.0),
            3.0,
            6.0,
            1.5,
            "Finish Line Leap",
        ).with_surface(SurfaceType::Dirt),
    ];

    let checkpoints = generate_checkpoints(&spline, 16, 3);
    let grid_positions = generate_grid_positions(&spline, 12, 10.0, 3.5);

    Track {
        name: "Crandon International Off-Road".to_string(),
        description: "The Big House: legendary 2,414m short-course off-road track in Wisconsin with wide clay straights, Barn Turn, and tabletop jumps.".to_string(),
        category: TrackCategory::Main,
        kind: TrackKind::Circuit,
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
            grandstands: vec![
                Grandstand::new(1, Vec2::new(100.0, -25.0), 120.0, 16.0, 0.0)
                    .with_style(GrandstandStyle::OpenBleachers)
                    .with_tiers(10)
                    .with_seat_color([0.85, 0.30, 0.20]),
            ],
            trees: vec![
                Tree::new(1, Vec2::new(450.0, 200.0), TreeType::Pine).with_scale(1.4),
                Tree::new(2, Vec2::new(460.0, 350.0), TreeType::Pine).with_scale(1.5),
            ],
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Dirt,
        pit_box_area: None,
        default_laps: 3,
        car_category: CarCategory::OffRoad,
        module_id: Some("extreme_offroad".to_string()),
        modules: vec!["extreme_offroad".to_string()],
        scale: "1:1".to_string(),
        wikipedia_url: None,
        osm_url: None,
        country_code: None,
        country_name: None,
        min_width: None,
        max_width: None,
        is_inspired: false,
    }
}
