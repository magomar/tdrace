use glam::Vec2;

use super::checkpoint::Checkpoint;
use super::geometry::{
    BarrierType, JumpRamp, LineSegment, Obstacle, SpawnPose, SurfaceShape, SurfaceZone,
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
        // Taper barrier offset down to touch the track/curbs on elevated bridge sections
        let elev_factor = (s.elevation / 2.5).clamp(0.0, 1.0);
        let curb_extra = if s.left_curb || s.right_curb { 1.35 } else { 0.0 };
        let bridge_offset = curb_extra + 0.15;
        let offset = barrier_offset * (1.0 - elev_factor) + bridge_offset * elev_factor;

        let half_w = s.width * 0.5 + offset;
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
        let elev = (spline.samples[i % n].elevation + spline.samples[next_i % n].elevation) * 0.5;
        left_walls.push(WallBarrier::with_elevation(left_pts[i], left_pts[next_i], barrier_type, elev));
    }

    let seg_count_right = if spline.closed { right_pts.len() } else { right_pts.len().saturating_sub(1) };
    for i in 0..seg_count_right {
        let next_i = (i + 1) % right_pts.len();
        let elev = (spline.samples[i % n].elevation + spline.samples[next_i % n].elevation) * 0.5;
        right_walls.push(WallBarrier::with_elevation(
            right_pts[i],
            right_pts[next_i],
            barrier_type,
            elev,
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

    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(185.0, 22.0), 1.2, "Apex Tire Stack T1"),
        Obstacle::circle(2, Vec2::new(225.0, -2.0), 1.2, "Apex Tire Stack T2"),
    ];

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
            obstacles,
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

    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(125.0, 45.0), 1.5, "Drift Clipping Zone 1"),
        Obstacle::circle(2, Vec2::new(80.0, 95.0), 1.5, "Drift Clipping Zone 2"),
        Obstacle::circle(3, Vec2::new(-28.0, 35.0), 1.5, "Drift Clipping Zone 3"),
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
            obstacles,
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

    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(100.0, 210.0), 1.5, "Stadium Apex Pylon"),
        Obstacle::circle(2, Vec2::new(-45.0, 20.0), 1.5, "Carousel Apex Pylon"),
        Obstacle::circle(3, Vec2::new(-50.0, -30.0), 1.2, "Tire Stack 3"),
        Obstacle::circle(4, Vec2::new(-50.0, -30.0), 1.2, "Tire Stack 4"),
        Obstacle::circle(5, Vec2::new(-50.0, -30.0), 1.2, "Tire Stack 5"),
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
            obstacles,
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

    // Obstacles: Towering natural mountain cliff monoliths and rock formations framing the pass
    let obstacles = vec![
        Obstacle::circle(1, Vec2::new(30.0, 170.0), 2.5, "Canyon Rock Wall East"),
        Obstacle::circle(2, Vec2::new(-25.0, 185.0), 3.0, "North Pass Monolith"),
        Obstacle::circle(3, Vec2::new(-75.0, 145.0), 2.5, "Gorge Cliff Face"),
        Obstacle::circle(4, Vec2::new(85.0, 45.0), 3.5, "Central Peak Rock Formation"),
    ];

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
            obstacles,
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

