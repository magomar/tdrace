use glam::Vec2;

use super::checkpoint::Checkpoint;
use super::geometry::{
    BarrierType, LineSegment, Obstacle, SpawnPose, SurfaceShape, SurfaceZone, TrackGeometry,
    WallBarrier,
};
use super::spline::{TrackSpline, TrackWaypoint};
use super::Track;
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
        let half_w = s.width * 0.5 + barrier_offset;
        left_pts.push(s.point + s.normal * half_w);
        right_pts.push(s.point - s.normal * half_w);
    }

    untangle_polyline(&mut left_pts, spline.closed);
    untangle_polyline(&mut right_pts, spline.closed);

    let mut left_walls = Vec::new();
    let mut right_walls = Vec::new();

    let seg_count_left = if spline.closed { left_pts.len() } else { left_pts.len().saturating_sub(1) };
    for i in 0..seg_count_left {
        let next_i = (i + 1) % left_pts.len();
        left_walls.push(WallBarrier::new(left_pts[i], left_pts[next_i], barrier_type));
    }

    let seg_count_right = if spline.closed { right_pts.len() } else { right_pts.len().saturating_sub(1) };
    for i in 0..seg_count_right {
        let next_i = (i + 1) % right_pts.len();
        right_walls.push(WallBarrier::new(
            right_pts[i],
            right_pts[next_i],
            barrier_type,
        ));
    }

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
    let mut slots = Vec::with_capacity(num_slots);
    let total_len = spline.total_length();

    for i in 0..num_slots {
        // Place slots behind start line (at negative offset along spline)
        let slot_dist = total_len - 15.0 - (i as f32 * spacing);
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
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
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

    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(185.0, 22.0), 1.2, "Apex Tire Stack T1"),
        Obstacle::circle(2, Vec2::new(225.0, -2.0), 1.2, "Apex Tire Stack T2"),
    ];

    let mut checkpoints = generate_checkpoints(&spline, 12, 3);
    // Add Pit Entry / Exit checkpoints
    let mut pit_entry = Checkpoint::new(
        100,
        LineSegment::new(Vec2::new(-25.0, -5.0), Vec2::new(-25.0, -18.0)),
        Vec2::new(1.0, 0.0),
        0,
        false,
    )
    .with_pit_flags(true, false);
    pit_entry.target_distance = spline.total_length() - 30.0;

    let mut pit_exit = Checkpoint::new(
        101,
        LineSegment::new(Vec2::new(135.0, -5.0), Vec2::new(135.0, -18.0)),
        Vec2::new(1.0, 0.0),
        0,
        false,
    )
    .with_pit_flags(false, true);
    pit_exit.target_distance = 135.0;

    checkpoints.push(pit_entry);
    checkpoints.push(pit_exit);

    let grid_positions = generate_grid_positions(&spline, 8, 8.0, 2.5);

    Track {
        name: "Classic Grand Prix".to_string(),
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles,
            surface_zones,
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
    }
}

/// Preset 2: Oval Speedway
/// High speed 2-turn oval with perimeter concrete walls, tight wall collisions, asphalt apron.
pub fn oval_speedway() -> Track {
    let waypoints = vec![
        // Front Straight
        TrackWaypoint::new(Vec2::new(0.0, -60.0), 18.0),
        TrackWaypoint::new(Vec2::new(150.0, -60.0), 18.0),
        // Turn 1 & 2 (East Banked Curve)
        TrackWaypoint::new(Vec2::new(230.0, -25.0), 20.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(250.0, 30.0), 20.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(230.0, 85.0), 20.0).with_curbs(true, true),
        // Back Straight
        TrackWaypoint::new(Vec2::new(150.0, 120.0), 18.0),
        TrackWaypoint::new(Vec2::new(0.0, 120.0), 18.0),
        // Turn 3 & 4 (West Banked Curve)
        TrackWaypoint::new(Vec2::new(-80.0, 85.0), 20.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(-100.0, 30.0), 20.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(-80.0, -25.0), 20.0).with_curbs(true, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 1.5, BarrierType::Concrete);

    let checkpoints = generate_checkpoints(&spline, 8, 2);
    let grid_positions = generate_grid_positions(&spline, 12, 7.0, 3.0);

    Track {
        name: "Oval Speedway".to_string(),
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
    }
}

/// Preset 3: Drift Park
/// Technical hairpin turns, wide sliding transitions, sand traps, clipping point curbs.
pub fn drift_park() -> Track {
    let waypoints = vec![
        // Launch Straight & Start/Finish
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
        TrackWaypoint::new(Vec2::new(80.0, 0.0), 14.0),
        // Hairpin 1 (Right Hand 180)
        TrackWaypoint::new(Vec2::new(130.0, 25.0), 15.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(130.0, 75.0), 15.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(70.0, 90.0), 15.0).with_curbs(false, true),
        // Transition S-Turn
        TrackWaypoint::new(Vec2::new(10.0, 70.0), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-40.0, 110.0), 15.0).with_curbs(false, true),
        // Hairpin 2 (Left Hand 180)
        TrackWaypoint::new(Vec2::new(-80.0, 160.0), 15.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-120.0, 130.0), 15.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-90.0, 70.0), 14.0).with_curbs(true, false),
        // Carousel / Donut Section (Wide, sweeping multi-apex drift corner)
        TrackWaypoint::new(Vec2::new(-45.0, 60.0), 14.5).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-15.0, 40.0), 15.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(-20.0, 15.0), 15.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(-65.0, 15.0), 14.5).with_curbs(false, true),
        // Switchback & Final Corner onto Launch Straight
        TrackWaypoint::new(Vec2::new(-105.0, -5.0), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-80.0, -25.0), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-30.0, -15.0), 14.0).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let mut surface_zones = Vec::new();
    // Sand traps outside hairpin 1 & 2 runoffs (outside track boundaries)
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(140.0, 10.0),
            max: Vec2::new(180.0, 95.0),
        },
        SurfaceType::Sand,
        "Drift Sand Trap 1",
    ));
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(-170.0, 90.0),
            max: Vec2::new(-130.0, 180.0),
        },
        SurfaceType::Sand,
        "Drift Sand Trap 2",
    ));

    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(100.0, 50.0), 1.5, "Drift Clipping Zone 1"),
        Obstacle::circle(2, Vec2::new(-80.0, 115.0), 1.5, "Drift Clipping Zone 2"),
        Obstacle::circle(3, Vec2::new(-28.0, 35.0), 1.5, "Drift Clipping Zone 3"),
    ];

    let checkpoints = generate_checkpoints(&spline, 10, 3);
    let grid_positions = generate_grid_positions(&spline, 6, 8.0, 2.5);

    Track {
        name: "Drift Park".to_string(),
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles,
            surface_zones,
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
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
    let grid_positions = generate_grid_positions(&spline, 6, 5.5, 1.8);

    Track {
        name: "Kart Arena".to_string(),
        spline,
        geometry: TrackGeometry {
            inner_walls: left_walls,
            outer_walls: right_walls,
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            left_boundary_polyline: left_poly,
            right_boundary_polyline: right_poly,
        },
        checkpoints,
        grid_positions,
        default_surface: SurfaceType::Grass,
        pit_box_area: None,
    }
}
