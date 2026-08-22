use glam::Vec2;

use super::checkpoint::Checkpoint;
use super::geometry::{
    BarrierType, LineSegment, Obstacle, SpawnPose, SurfaceShape, SurfaceZone, TrackGeometry,
    WallBarrier,
};
use super::spline::{TrackSpline, TrackWaypoint};
use super::Track;
use crate::physics::surface::SurfaceType;

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

    let mut left_walls = Vec::new();
    let mut right_walls = Vec::new();

    let seg_count = if spline.closed { n } else { n - 1 };
    for i in 0..seg_count {
        let next_i = (i + 1) % n;
        left_walls.push(WallBarrier::new(left_pts[i], left_pts[next_i], barrier_type));
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
    let (left_walls, mut right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 4.0, BarrierType::Armco);

    // Pit lane parallel to main straight (x: -10 to 140, y: -12.0)
    let pit_wall = WallBarrier::new(
        Vec2::new(-10.0, -7.0),
        Vec2::new(130.0, -7.0),
        BarrierType::Concrete,
    );
    right_walls.push(pit_wall);

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
        // Launch Straight
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
        // Donut / Roundabout Section
        TrackWaypoint::new(Vec2::new(-20.0, 30.0), 16.0).with_curbs(true, true),
        TrackWaypoint::new(Vec2::new(-10.0, -20.0), 14.0).with_curbs(true, false),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, left_poly, right_poly) =
        generate_walls_from_spline(&spline, 3.5, BarrierType::TireWall);

    let mut surface_zones = Vec::new();
    // Sand traps outside hairpin 1 & 2
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(135.0, 10.0),
            max: Vec2::new(180.0, 95.0),
        },
        SurfaceType::Sand,
        "Drift Sand Trap 1",
    ));
    surface_zones.push(SurfaceZone::new(
        SurfaceShape::Aabb {
            min: Vec2::new(-170.0, 90.0),
            max: Vec2::new(-125.0, 180.0),
        },
        SurfaceType::Sand,
        "Drift Sand Trap 2",
    ));

    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(100.0, 50.0), 1.5, "Drift Clipping Zone 1"),
        Obstacle::circle(2, Vec2::new(-80.0, 115.0), 1.5, "Drift Clipping Zone 2"),
    ];

    let checkpoints = generate_checkpoints(&spline, 10, 3);
    let grid_positions = generate_grid_positions(&spline, 6, 9.0, 2.5);

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
