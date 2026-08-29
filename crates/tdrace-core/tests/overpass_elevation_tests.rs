use glam::Vec2;
use tdrace_core::collision::car_collision::resolve_car_car_collision;
use tdrace_core::collision::wall::{resolve_car_obstacle_collision, resolve_car_wall_collision};
use tdrace_core::lidar::{LidarConfig, LidarHitType, LidarScanner};
use tdrace_core::physics::car::Car;
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, Obstacle, TrackGeometry, WallBarrier};
use tdrace_core::track::presets::generate_walls_from_spline;
use tdrace_core::track::spline::{TrackSpline, TrackWaypoint};
use tdrace_core::track::{Track, TrackCategory};

#[test]
fn test_spline_smooth_elevation_interpolation() {
    let waypoints = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0).with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 14.0).with_elevation(0.0),
        TrackWaypoint::new(Vec2::new(200.0, 50.0), 14.0).with_elevation(2.5),
        TrackWaypoint::new(Vec2::new(300.0, 100.0), 14.0).with_elevation(5.0),
        TrackWaypoint::new(Vec2::new(200.0, 150.0), 14.0).with_elevation(2.0),
        TrackWaypoint::new(Vec2::new(100.0, 100.0), 14.0).with_elevation(0.0),
    ];

    let spline = TrackSpline::new(waypoints, true);

    // Test sampling at various arc lengths
    let s_start = spline.sample_at_distance(10.0);
    assert!(s_start.elevation < 0.5, "Start elevation should be near 0, got {}", s_start.elevation);

    // Project point near crest
    let proj_crest = spline.project_point(Vec2::new(300.0, 100.0));
    assert!(
        (proj_crest.elevation - 5.0).abs() < 0.5,
        "Crest elevation should be close to 5.0, got {}",
        proj_crest.elevation
    );
}

#[test]
fn test_wall_elevation_filtering_underpass_and_overpass() {
    let mut car_ground = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, 50.0), 0.0);
    car_ground.state.road_elevation = 0.0;

    let bridge_wall = WallBarrier::with_elevation(
        Vec2::new(40.0, 50.0),
        Vec2::new(60.0, 50.0),
        BarrierType::Armco,
        5.0,
    );

    // Ground car passing across bridge wall segment should NOT collide
    let col = resolve_car_wall_collision(&mut car_ground, &bridge_wall);
    assert!(col.is_none(), "Ground car should not collide with elevated bridge wall");

    // Elevated car on the bridge
    let mut car_bridge = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(50.0, 50.0), 0.0);
    car_bridge.state.road_elevation = 5.0;

    // Bridge car against bridge wall SHOULD collide
    let col_bridge = resolve_car_wall_collision(&mut car_bridge, &bridge_wall);
    assert!(col_bridge.is_some(), "Bridge car should collide with bridge wall at same elevation");

    // Bridge car against ground wall should NOT collide
    let ground_wall = WallBarrier::with_elevation(
        Vec2::new(40.0, 50.0),
        Vec2::new(60.0, 50.0),
        BarrierType::Armco,
        0.0,
    );
    let col_overpass = resolve_car_wall_collision(&mut car_bridge, &ground_wall);
    assert!(col_overpass.is_none(), "Bridge car should not collide with ground wall beneath");
}

#[test]
fn test_obstacle_elevation_filtering() {
    let mut car_bridge = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(10.0, 10.0), 0.0);
    car_bridge.state.road_elevation = 4.5;

    let ground_obstacle = Obstacle::circle(1, Vec2::new(10.0, 10.0), 2.0, "Ground Tire").with_elevation(0.0);
    let col = resolve_car_obstacle_collision(&mut car_bridge, &ground_obstacle);
    assert!(col.is_none(), "Bridge car should not collide with ground obstacle beneath");

    let mut car_ground = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(10.0, 10.0), 0.0);
    car_ground.state.road_elevation = 0.0;
    let col_hit = resolve_car_obstacle_collision(&mut car_ground, &ground_obstacle);
    assert!(col_hit.is_some(), "Ground car should collide with ground obstacle at same elevation");
}

#[test]
fn test_car_car_elevation_filtering_crossing() {
    let mut car_ground = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(150.0, 75.0), 0.0);
    car_ground.state.road_elevation = 0.0;

    let mut car_bridge = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(150.0, 75.0), 1.57);
    car_bridge.state.road_elevation = 4.8;

    // Both cars occupy the exact same (X, Y) 2D position but have different elevation (overpass)
    let col = resolve_car_car_collision(&mut car_ground, &mut car_bridge, 0.5, 0.4);
    assert!(col.is_none(), "Cars at different elevations (pass-over) must not collide");

    // If elevations match, they should collide
    car_bridge.state.road_elevation = 0.0;
    let col_hit = resolve_car_car_collision(&mut car_ground, &mut car_bridge, 0.5, 0.4);
    assert!(col_hit.is_some(), "Cars at same elevation must collide");
}

#[test]
fn test_lidar_elevation_filtering() {
    let car_ground = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(0.0, 0.0), 0.0);
    // car_ground has road_elevation = 0.0

    let elevated_wall = WallBarrier::with_elevation(
        Vec2::new(20.0, -10.0),
        Vec2::new(20.0, 10.0),
        BarrierType::Armco,
        5.0,
    );

    let track = Track {
        name: "Test Overpass".to_string(),
        description: "Test".to_string(),
        category: TrackCategory::Main,
        spline: TrackSpline::new(
            vec![
                TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
                TrackWaypoint::new(Vec2::new(50.0, 0.0), 12.0),
                TrackWaypoint::new(Vec2::new(100.0, 0.0), 12.0),
            ],
            false,
        ),
        geometry: TrackGeometry {
            inner_walls: vec![elevated_wall],
            outer_walls: Vec::new(),
            obstacles: Vec::new(),
            surface_zones: Vec::new(),
            jump_ramps: Vec::new(),
            left_boundary_polyline: Vec::new(),
            right_boundary_polyline: Vec::new(),
        },
        checkpoints: Vec::new(),
        grid_positions: Vec::new(),
        default_surface: SurfaceType::Asphalt,
        pit_box_area: None,
        default_laps: 3,
        predefined_car: None,
        module_id: None,
    };

    let scanner = LidarScanner::new(LidarConfig::forward_cone_16());
    let scan = scanner.scan(&car_ground, &track, &[]);

    // All rays should miss the elevated wall because elevation difference > 2.0m
    for hit in scan {
        assert_eq!(hit.hit_type, LidarHitType::None, "LIDAR from ground should ignore elevated wall");
    }
}

#[test]
fn test_silverstone_pass_over_clearance() {
    let waypoints = vec![
        // Hamilton Straight (Start/Finish)
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 14.0),
        TrackWaypoint::new(Vec2::new(140.0, 0.0), 14.0),
        // Abbey & Farm Curve
        TrackWaypoint::new(Vec2::new(220.0, 30.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(280.0, 80.0), 14.0).with_curbs(true, false),
        // Village & The Loop
        TrackWaypoint::new(Vec2::new(260.0, 160.0), 12.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(190.0, 180.0), 12.0).with_curbs(true, false),
        // Aintree onto Wellington Straight
        TrackWaypoint::new(Vec2::new(140.0, 130.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(60.0, 120.0), 14.0),
        TrackWaypoint::new(Vec2::new(-60.0, 110.0), 14.0),
        // Brooklands & Luffield
        TrackWaypoint::new(Vec2::new(-140.0, 150.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-170.0, 220.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-120.0, 270.0), 13.0).with_curbs(true, false),
        // Copse Corner
        TrackWaypoint::new(Vec2::new(0.0, 300.0), 14.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(100.0, 350.0), 14.0).with_curbs(false, true),
        // Maggotts, Becketts & Chapel Complex
        TrackWaypoint::new(Vec2::new(180.0, 420.0), 14.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(240.0, 460.0), 13.0).with_curbs(false, true),
        TrackWaypoint::new(Vec2::new(290.0, 430.0), 13.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(320.0, 360.0), 14.0).with_curbs(false, true),
        // Hangar Straight Overpass Bridge (Passes elevated over Abbey & Farm Curve)
        TrackWaypoint::new(Vec2::new(280.0, 240.0), 15.0).with_elevation(2.5),
        TrackWaypoint::new(Vec2::new(230.0, 100.0), 15.0).with_elevation(5.0),
        // Stowe Corner Descent Ramp
        TrackWaypoint::new(Vec2::new(170.0, -40.0), 13.0)
            .with_curbs(false, true)
            .with_elevation(2.0),
        // Vale & Club Corner onto Main Straight
        TrackWaypoint::new(Vec2::new(80.0, -80.0), 12.0).with_curbs(true, false),
        TrackWaypoint::new(Vec2::new(-30.0, -60.0), 13.0).with_curbs(false, true),
    ];

    let spline = TrackSpline::new(waypoints, true);
    let (left_walls, right_walls, _, _) = generate_walls_from_spline(&spline, 5.0, BarrierType::Armco);

    // Ground position on Abbey/Farm curve under the bridge (waypoint 2)
    let ground_sample = &spline.samples[2 * 24];
    let ground_pos = ground_sample.point;
    let proj_ground = spline.project_point_continuity(ground_pos, ground_sample.distance, 50.0);
    assert_eq!(proj_ground.elevation, 0.0, "Abbey/Farm underpass should be ground level 0.0m");

    // Elevated position on Hangar Straight bridge (waypoint 19)
    let bridge_sample = &spline.samples[19 * 24];
    let bridge_pos = bridge_sample.point;
    assert!(
        bridge_sample.elevation >= 4.5,
        "bridge_sample.elevation should be >= 4.5, got {}",
        bridge_sample.elevation
    );
    let proj_bridge = spline.project_point_continuity(bridge_pos, bridge_sample.distance, 50.0);
    assert!(
        proj_bridge.elevation >= 4.5,
        "Hangar Straight crest should be elevated >= 4.5m, got {}",
        proj_bridge.elevation
    );

    // Test a car driving through the Abbey/Farm underpass (road_elevation = 0.0)
    let mut ground_car = Car::new(CarConfig::sports_car()).with_pose(ground_pos, 0.6);
    ground_car.state.road_elevation = proj_ground.elevation;

    for wall in left_walls.iter().chain(right_walls.iter()) {
        // Walls belonging to the elevated bridge (elevation > 2.5m) should not collide with ground car
        if wall.elevation > 2.5 {
            let col = resolve_car_wall_collision(&mut ground_car, wall);
            assert!(
                col.is_none(),
                "Ground car underpass should NOT collide with overhead bridge wall at elevation {}",
                wall.elevation
            );
        }
    }

    // Test a car driving over the Hangar Straight bridge (road_elevation = 5.0)
    let mut bridge_car = Car::new(CarConfig::sports_car()).with_pose(bridge_pos, -2.0);
    bridge_car.state.road_elevation = proj_bridge.elevation;

    for wall in left_walls.iter().chain(right_walls.iter()) {
        // Walls belonging to the ground track (elevation < 0.5m) should not collide with bridge car
        if wall.elevation < 0.5 {
            let col = resolve_car_wall_collision(&mut bridge_car, wall);
            assert!(
                col.is_none(),
                "Bridge car overpass should NOT collide with ground wall beneath at elevation {}",
                wall.elevation
            );
        }
    }

    // Verify that bridge walls touch the track edges (offset <= 1.6m rather than floating 5.0m away)
    for wall in left_walls.iter().chain(right_walls.iter()) {
        if wall.elevation >= 4.0 {
            let wall_mid = (wall.segment.start + wall.segment.end) * 0.5;
            let proj = spline.project_point(wall_mid);
            let dist_from_centerline = proj.lateral_offset.abs();
            let track_half_w = proj.track_width * 0.5;
            let offset_from_edge = dist_from_centerline - track_half_w;
            assert!(
                offset_from_edge <= 1.6,
                "Bridge wall must touch the track edge (offset <= 1.6m), got offset {}",
                offset_from_edge
            );
        }
    }
}
