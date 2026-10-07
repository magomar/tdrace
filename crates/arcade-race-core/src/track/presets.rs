use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::checkpoint::Checkpoint;
use super::geometry::{
    BarrierType, JumpRamp, LineSegment, SpawnPose, SurfaceShape,
    TrackGeometry, WallBarrier,
};
use super::spline::{untangle_offset_vertices, TrackSpline, TrackWaypoint};
use super::{CarCategory, Track, TrackCategory, TrackKind};
use wheelbase::SurfaceType;

/// Trims local self-intersecting loops (swallowtail singularities) from an offset boundary polyline
/// by collapsing vertices within the loop to the intersection point, preserving vertex count and
/// 1-to-1 sample alignment.
pub fn untangle_polyline(pts: &mut Vec<Vec2>, closed: bool) {
    untangle_offset_vertices(pts.as_mut_slice(), closed);
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

        // Drop the slivers that trimming leaves, but keep an untrimmed wall of any length: on the inside of a
        // tight turn with dense samples every wall piece is shorter than 0.10 m.
        let untrimmed = segments_to_process.len() == 1 && segments_to_process[0] == wall.segment;
        for seg in segments_to_process {
            if untrimmed || seg.length() > 0.10 {
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
    let mut trimmed_left = vec![false; left_walls.len()];
    let mut trimmed_right = vec![false; right_walls.len()];
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
                // Skip segments sharing an exact endpoint (a pair already trimmed at its hit). Near-coincident
                // endpoints of non-local walls are not a connection: a figure-eight crossing can sit a few cm
                // from a vertex on both branches, and skipping it leaves the walls crossed.
                if (seg_a.start - seg_b.start).length_squared() < 1e-6
                    || (seg_a.start - seg_b.end).length_squared() < 1e-6
                    || (seg_a.end - seg_b.start).length_squared() < 1e-6
                    || (seg_a.end - seg_b.end).length_squared() < 1e-6
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
                        trimmed_left[i] = true;
                    } else {
                        right_walls[i - n_left].segment = new_seg_a;
                        trimmed_right[i - n_left] = true;
                    }

                    if j < n_left {
                        left_walls[j].segment = new_seg_b;
                        trimmed_left[j] = true;
                    } else {
                        right_walls[j - n_left].segment = new_seg_b;
                        trimmed_right[j - n_left] = true;
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

    // Drop the slivers that trimming leaves; untrimmed walls stay, however short (see trim_walls_at_crossings).
    let mut trimmed = trimmed_left.into_iter();
    left_walls.retain(|w| !trimmed.next().unwrap_or(false) || w.segment.length() > 0.10);
    let mut trimmed = trimmed_right.into_iter();
    right_walls.retain(|w| !trimmed.next().unwrap_or(false) || w.segment.length() > 0.10);
}

/// Builds boundary wall barriers along the track edges given a spline and barrier offset,
/// merging collinear segments by default to reduce physics and rendering complexity.
pub fn generate_walls_from_spline(
    spline: &TrackSpline,
    barrier_offset: f32,
    barrier_type: BarrierType,
) -> (Vec<WallBarrier>, Vec<WallBarrier>, Vec<Vec2>, Vec<Vec2>) {
    let (left_walls, right_walls, left_pts, right_pts) =
        generate_walls_from_spline_raw(spline, barrier_offset, barrier_type);
    (
        merge_collinear_walls(left_walls),
        merge_collinear_walls(right_walls),
        left_pts,
        right_pts,
    )
}

/// Builds unmerged raw boundary wall barriers along the track edges given a spline and barrier offset.
pub fn generate_walls_from_spline_raw(
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

    for (i, s) in spline.samples.iter().enumerate() {
        let elev_factor = if s.is_bridge { (s.elevation / 3.0).clamp(0.0, 1.0) } else { 0.0 };
        let curb_extra = if s.left_curb || s.right_curb { 1.35 } else { 0.75 };
        let bridge_offset = curb_extra + 0.50;

        let left_base = spline.blended_wall_distance(i, true, barrier_offset).unwrap_or(barrier_offset);
        let right_base = spline.blended_wall_distance(i, false, barrier_offset).unwrap_or(barrier_offset);

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
        if (left_pts[next_i] - left_pts[i]).length_squared() < 1e-4 {
            continue;
        }
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
        if (right_pts[next_i] - right_pts[i]).length_squared() < 1e-4 {
            continue;
        }
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

/// Merges contiguous collinear wall segments that share endpoints, have the same barrier type,
/// identical bridge flags, and matching elevation within a tight tolerance.
pub fn merge_collinear_walls(walls: Vec<WallBarrier>) -> Vec<WallBarrier> {
    if walls.len() <= 1 {
        return walls;
    }

    let mut merged: Vec<WallBarrier> = Vec::with_capacity(walls.len());
    let mut current = walls[0];

    for next in walls.into_iter().skip(1) {
        let p0 = current.segment.start;
        let p1 = current.segment.end;
        let p2 = next.segment.start;
        let p3 = next.segment.end;

        // Shared vertex only: a trimmed crossing leaves a gap of a few cm, and bridging it re-creates the crossing.
        let is_connected = (p1 - p2).length_squared() < 1e-6; // within 1 mm
        let same_type = current.barrier_type == next.barrier_type;
        let same_bridge = current.is_bridge == next.is_bridge;
        let same_elev = (current.elevation - next.elevation).abs() < 0.05;

        let v1 = p1 - p0;
        let v2 = p3 - p2;
        let len1 = v1.length();
        let len2 = v2.length();

        let mut can_merge = false;
        if is_connected && same_type && same_bridge && same_elev && len1 > 1e-4 && len2 > 1e-4 {
            let dot = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
            // Collinear if angle < 1.0 degree: cos(1.0 deg) ≈ 0.9998477
            if dot > 0.9998 {
                let v_tot = p3 - p0;
                let len_tot = v_tot.length();
                if len_tot > 1e-4 {
                    let perp_dist = (v_tot.perp_dot(p1 - p0)).abs() / len_tot;
                    if perp_dist < 0.03 {
                        can_merge = true;
                    }
                }
            }
        }

        if can_merge {
            current.segment.end = p3;
            let w1 = len1 / (len1 + len2);
            current.elevation = current.elevation * w1 + next.elevation * (1.0 - w1);
        } else {
            merged.push(current);
            current = next;
        }
    }
    merged.push(current);

    if merged.len() > 1 {
        let last = *merged.last().unwrap();
        let first = merged[0];
        let p0 = last.segment.start;
        let p1 = last.segment.end;
        let p2 = first.segment.start;
        let p3 = first.segment.end;

        let is_connected = (p1 - p2).length_squared() < 1e-6;
        let same_type = last.barrier_type == first.barrier_type;
        let same_bridge = last.is_bridge == first.is_bridge;
        let same_elev = (last.elevation - first.elevation).abs() < 0.05;

        let v1 = p1 - p0;
        let v2 = p3 - p2;
        let len1 = v1.length();
        let len2 = v2.length();

        let mut can_merge = false;
        if is_connected && same_type && same_bridge && same_elev && len1 > 1e-4 && len2 > 1e-4 {
            let dot = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
            if dot > 0.9998 {
                let v_tot = p3 - p0;
                let len_tot = v_tot.length();
                if len_tot > 1e-4 {
                    let perp_dist = (v_tot.perp_dot(p1 - p0)).abs() / len_tot;
                    if perp_dist < 0.03 {
                        can_merge = true;
                    }
                }
            }
        }

        if can_merge {
            let last_wall = merged.pop().unwrap();
            let first_wall = &mut merged[0];
            first_wall.segment.start = last_wall.segment.start;
            let w_last = len1 / (len1 + len2);
            first_wall.elevation = last_wall.elevation * w_last + first_wall.elevation * (1.0 - w_last);
        }
    }

    merged
}

/// Computes an adaptive checkpoint count clamped between 8 and 24 based on total track length:
///
/// N = clamp(floor(L / 50.0), 8, 24)
pub fn adaptive_checkpoint_count(spline: &TrackSpline) -> usize {
    let total_len = spline.total_length();
    ((total_len / 50.0).floor() as usize).clamp(8, 24)
}

/// Generates a sequence of checkpoints distributed along the track spline.
///
/// Checkpoints are distributed with speed- and curvature-aware weighting (Spec 071 §D):
/// gates are allocated proportionally to estimated traversal time dt = ds / v(s),
/// giving higher gate density in tight technical corners and braking zones, and wider
/// spacing along high-speed straights.
pub fn generate_checkpoints(
    spline: &TrackSpline,
    count: usize,
    num_sectors: usize,
) -> Vec<Checkpoint> {
    if count == 0 {
        return Vec::new();
    }
    let total_len = spline.total_length();
    if total_len < 1.0 {
        return Vec::new();
    }
    let sectors = num_sectors.max(1);

    if count == 1 {
        let sample = spline.sample_at_distance(0.0);
        let half_w = sample.width * 0.5 + 4.0;
        let gate_left = sample.point + sample.normal * half_w;
        let gate_right = sample.point - sample.normal * half_w;
        let mut cp = Checkpoint::new(
            0,
            LineSegment::new(gate_left, gate_right),
            sample.tangent,
            0,
            true,
        );
        cp.target_distance = 0.0;
        cp.elevation = sample.elevation;
        return vec![cp];
    }

    let samples = &spline.samples;
    let n = samples.len();

    let mut distances = Vec::with_capacity(count);
    distances.push(0.0);

    if n >= 4 {
        // Build cumulative traversal time along spline samples
        let mut cum_time = Vec::with_capacity(n + 1);
        let mut sample_dists = Vec::with_capacity(n + 1);
        cum_time.push(0.0);
        sample_dists.push(0.0);

        let mut current_t = 0.0;
        for i in 0..n {
            let next_i = (i + 1) % n;
            let p0 = samples[i].point;
            let p1 = samples[next_i].point;
            let ds = (p1 - p0).length();
            if ds < 1e-4 {
                continue;
            }

            let t0 = samples[i].tangent;
            let t1 = samples[next_i].tangent;
            let cross = t0.x * t1.y - t0.y * t1.x;
            let dot = t0.dot(t1);
            let turn_angle = cross.atan2(dot).abs();
            let curvature = turn_angle / ds;
            let radius = if curvature > 1e-4 { 1.0 / curvature } else { 10_000.0 };

            let mu = 1.0;
            let g = 9.81;
            let v_corner = (mu * g * radius).sqrt();
            let v_top = 50.0; // 180 km/h nominal top speed
            let v_min = 10.0; // 36 km/h hairpin minimum speed
            let speed = v_corner.clamp(v_min, v_top);

            let dt = ds / speed;
            current_t += dt;

            let d = if i + 1 == n && spline.closed {
                total_len
            } else {
                samples[next_i].distance
            };
            cum_time.push(current_t);
            sample_dists.push(d);
        }

        let total_time = current_t;
        if total_time > 1e-4 && cum_time.len() >= 2 {
            let min_gap = 15.0_f32.min(total_len / (count as f32 * 2.0));
            let mut last_d = 0.0;

            for i in 1..count {
                let target_t = (i as f32 / count as f32) * total_time;
                let idx = match cum_time.binary_search_by(|t| t.partial_cmp(&target_t).unwrap_or(std::cmp::Ordering::Equal)) {
                    Ok(k) => k,
                    Err(k) => k.saturating_sub(1),
                };
                let idx = idx.min(cum_time.len().saturating_sub(2));
                let t0 = cum_time[idx];
                let t1 = cum_time[idx + 1];
                let frac = if (t1 - t0).abs() > 1e-6 {
                    ((target_t - t0) / (t1 - t0)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let raw_d = sample_dists[idx] + frac * (sample_dists[idx + 1] - sample_dists[idx]);

                // Evaluate local curvature at raw_d to determine if on a high-speed straight
                let s_probe = spline.sample_at_distance(raw_d);
                let s_ahead = spline.sample_at_distance(raw_d + 1.0);
                let dot = s_probe.tangent.dot(s_ahead.tangent).clamp(-1.0, 1.0);
                let turn_angle = (s_probe.tangent.x * s_ahead.tangent.y - s_probe.tangent.y * s_ahead.tangent.x)
                    .atan2(dot)
                    .abs();
                let local_radius = if turn_angle > 1e-4 { 1.0 / turn_angle } else { 10_000.0 };
                let is_straight = local_radius > 400.0;

                let min_spacing = if is_straight && total_len >= 600.0 {
                    120.0_f32
                } else {
                    min_gap
                };

                let remaining_gates = (count - 1 - i) as f32;
                let max_allowed = total_len - remaining_gates * min_gap - min_gap * 0.5;
                let min_allowed = last_d + min_spacing;
                let d = if min_allowed <= max_allowed {
                    raw_d.clamp(min_allowed, max_allowed)
                } else {
                    raw_d.clamp(last_d + min_gap, max_allowed)
                };
                distances.push(d);
                last_d = d;
            }
        }
    }

    if distances.len() < count {
        distances.clear();
        for i in 0..count {
            distances.push((i as f32 / count as f32) * total_len);
        }
    }

    let mut checkpoints = Vec::with_capacity(count);
    for (i, &dist) in distances.iter().enumerate() {
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
            3,
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
            5,
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
            5,
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
    let (num_slots, spacing, stagger) = match mod_str {
        "gt" => (18, 10.0, 2.5),
        "nascar" => (16, 8.5, 3.0),
        "kart" => (14, 5.5, 1.8),
        "rally" | "extreme_offroad" => (12, 8.5, 2.8),
        _ => (10, 8.0, 2.5),
    };
    let grid_positions = generate_grid_positions(&spline, num_slots, spacing, stagger);

    let shape_name = match shape {
        TrackShape::Oval => "Oval",
        TrackShape::HorizontalEight => "Figure-8",
    };
    let dir_name = match direction {
        RaceDirection::Right => "Right",
        RaceDirection::Left => "Left",
    };
    let mod_name = match mod_str {
        "gt" => "Grand Touring Challenge",
        "kart" => "Karting",
        "rally" => "Rallycross",
        "nascar" => "Stock Car Racing",
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
        network: None,
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
            ..Default::default()
        },
        checkpoints,
        grid_positions,
        default_surface: offtrack_surface,
        pit_box_area: None,
        pit_lane: None,
        default_laps,
        car_category,
        car_model_id: None,
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
        tag: String::new(),
        category_label: String::new(),
    }.with_default_runoff_surfaces()
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_merge_collinear_walls() {
        let w1 = WallBarrier::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), BarrierType::Steel);
        let w2 = WallBarrier::new(Vec2::new(10.0, 0.0), Vec2::new(20.0, 0.0), BarrierType::Steel);
        let w3 = WallBarrier::new(Vec2::new(20.0, 0.0), Vec2::new(30.0, 0.0), BarrierType::Steel);
        // Turns 90 degrees:
        let w4 = WallBarrier::new(Vec2::new(30.0, 0.0), Vec2::new(30.0, 10.0), BarrierType::Steel);
        // Different barrier type:
        let w5 = WallBarrier::new(Vec2::new(30.0, 10.0), Vec2::new(30.0, 20.0), BarrierType::Concrete);

        let walls = vec![w1, w2, w3, w4, w5];
        let merged = merge_collinear_walls(walls);

        assert_eq!(merged.len(), 3);
        // First merged piece: 0.0 to 30.0
        assert_eq!(merged[0].segment.start, Vec2::new(0.0, 0.0));
        assert_eq!(merged[0].segment.end, Vec2::new(30.0, 0.0));
        assert_eq!(merged[0].barrier_type, BarrierType::Steel);
        // Second piece: 30.0,0.0 to 30.0,10.0
        assert_eq!(merged[1].segment.start, Vec2::new(30.0, 0.0));
        assert_eq!(merged[1].segment.end, Vec2::new(30.0, 10.0));
        // Third piece: Concrete
        assert_eq!(merged[2].segment.start, Vec2::new(30.0, 10.0));
        assert_eq!(merged[2].segment.end, Vec2::new(30.0, 20.0));
        assert_eq!(merged[2].barrier_type, BarrierType::Concrete);
    }

    #[test]
    fn test_adaptive_checkpoint_count_scaling() {
        // Short kart track (< 400m) clamps to 8 checkpoints
        let waypoints_short = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(100.0, 0.0), 10.0),
            TrackWaypoint::new(Vec2::new(100.0, 95.0), 10.0),
            TrackWaypoint::new(Vec2::new(0.0, 95.0), 10.0),
        ];
        let spline_short = TrackSpline::new(waypoints_short, true);
        assert!(spline_short.total_length() < 420.0);
        let count_short = adaptive_checkpoint_count(&spline_short);
        assert_eq!(count_short, 8, "Short track (~390m) must produce 8 checkpoints");

        // Medium circuit (~1000m) produces ~20 checkpoints
        let waypoints_med = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(300.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(300.0, 200.0), 12.0),
            TrackWaypoint::new(Vec2::new(0.0, 200.0), 12.0),
        ];
        let spline_med = TrackSpline::new(waypoints_med, true);
        let count_med = adaptive_checkpoint_count(&spline_med);
        assert_eq!(count_med, 20, "1000m circuit must produce 20 checkpoints");

        // Long circuit (> 2000m) clamps to 24 checkpoints
        let waypoints_long = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(800.0, 0.0), 14.0),
            TrackWaypoint::new(Vec2::new(800.0, 500.0), 14.0),
            TrackWaypoint::new(Vec2::new(0.0, 500.0), 14.0),
        ];
        let spline_long = TrackSpline::new(waypoints_long, true);
        assert!(spline_long.total_length() > 2000.0);
        let count_long = adaptive_checkpoint_count(&spline_long);
        assert_eq!(count_long, 24, "Long circuit (>2000m) must clamp to max 24 checkpoints");
    }

    #[test]
    fn test_speed_weighted_checkpoint_distribution_on_straight_vs_corner() {
        // Build a circuit with a long ~400m high-speed straight and tight corner complexes
        let waypoints = vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),     // Start of straight
            TrackWaypoint::new(Vec2::new(400.0, 0.0), 12.0),   // End of straight
            TrackWaypoint::new(Vec2::new(450.0, 30.0), 12.0),  // Corner entry
            TrackWaypoint::new(Vec2::new(450.0, 90.0), 12.0),  // Apex
            TrackWaypoint::new(Vec2::new(400.0, 120.0), 12.0), // Exit
            TrackWaypoint::new(Vec2::new(200.0, 120.0), 12.0), // Return straight
            TrackWaypoint::new(Vec2::new(0.0, 60.0), 12.0),    // Final turn
        ];
        let spline = TrackSpline::new(waypoints, true);
        let total_len = spline.total_length();

        let checkpoints = generate_checkpoints(&spline, 14, 3);
        assert_eq!(checkpoints.len(), 14);
        assert!(checkpoints[0].is_finish_line);
        assert_eq!(checkpoints[0].target_distance, 0.0);

        // Verify strictly monotonic ordering
        for i in 1..checkpoints.len() {
            assert!(
                checkpoints[i].target_distance > checkpoints[i - 1].target_distance,
                "Checkpoints must be strictly monotonic: cp[{}]={} <= cp[{}]={}",
                i, checkpoints[i].target_distance, i - 1, checkpoints[i - 1].target_distance
            );
            assert!(checkpoints[i].target_distance < total_len);
        }

        // On the 400m straight (dist ~0 to ~380m), verify gate spacing is wide (>= 120m)
        let straight_cps: Vec<&Checkpoint> = checkpoints
            .iter()
            .filter(|cp| cp.target_distance >= 20.0 && cp.target_distance <= 360.0)
            .collect();
        for window in straight_cps.windows(2) {
            let spacing = window[1].target_distance - window[0].target_distance;
            assert!(
                spacing >= 120.0,
                "Checkpoints along high-speed straight must be widely spaced (>= 120m), got {:.2}m",
                spacing
            );
        }
    }
}


