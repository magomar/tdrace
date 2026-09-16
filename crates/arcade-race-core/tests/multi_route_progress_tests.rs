use glam::Vec2;
use arcade_race_core::track::checkpoint::{Checkpoint, MultiRouteProgressTracker, TrackProgressTracker};
use arcade_race_core::track::geometry::LineSegment;
use arcade_race_core::track::network::{
    RoadSegment, SegmentId, TrackLayout, TrackNetwork,
};
use arcade_race_core::track::spline::TrackWaypoint;
use wheelbase::{Car, CarConfig};

fn create_test_branching_network() -> (TrackNetwork, Vec<Checkpoint>) {
    // Seg 0: (0, 0) -> (60, 0) [Entry Trunk]
    let seg0 = RoadSegment::new(
        SegmentId(0),
        "Trunk Entry",
        vec![
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 12.0),
        ],
    );

    // Seg 1: (60, 0) -> (110, -25) -> (160, 0) [Main GP Branch]
    let seg1 = RoadSegment::new(
        SegmentId(1),
        "Main Branch",
        vec![
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(110.0, -25.0), 12.0),
            TrackWaypoint::new(Vec2::new(160.0, 0.0), 12.0),
        ],
    );

    // Seg 2: (60, 0) -> (110, 35) -> (160, 0) [Joker Lap Detour Branch]
    let seg2 = RoadSegment::new(
        SegmentId(2),
        "Joker Branch",
        vec![
            TrackWaypoint::new(Vec2::new(60.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(110.0, 35.0), 12.0),
            TrackWaypoint::new(Vec2::new(160.0, 0.0), 12.0),
        ],
    );

    // Seg 3: (160, 0) -> (220, 0) -> (220, -60) -> (0, -60) -> (0, 0) [Common Return Trunk]
    let seg3 = RoadSegment::new(
        SegmentId(3),
        "Common Return",
        vec![
            TrackWaypoint::new(Vec2::new(160.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(220.0, 0.0), 12.0),
            TrackWaypoint::new(Vec2::new(220.0, -60.0), 12.0),
            TrackWaypoint::new(Vec2::new(0.0, -60.0), 12.0),
            TrackWaypoint::new(Vec2::new(0.0, 0.0), 12.0),
        ],
    );

    let layout_main = TrackLayout::new(
        "main",
        "Grand Prix Circuit",
        vec![SegmentId(0), SegmentId(1), SegmentId(3)],
        SegmentId(0),
    )
    .with_checkpoints(vec![0, 1, 2, 4]);

    let layout_joker = TrackLayout::new(
        "joker",
        "Rallycross Joker Route",
        vec![SegmentId(0), SegmentId(2), SegmentId(3)],
        SegmentId(0),
    )
    .with_checkpoints(vec![0, 1, 3, 4]);

    let network = TrackNetwork {
        junctions: Vec::new(),
        segments: vec![seg0, seg1, seg2, seg3],
        layouts: vec![layout_main, layout_joker],
        default_layout_id: "main".to_string(),
    };

    let checkpoints = vec![
        // CP 0: Start / Finish line at (0, 0)
        Checkpoint::new(
            0,
            LineSegment::new(Vec2::new(0.0, -10.0), Vec2::new(0.0, 10.0)),
            Vec2::new(1.0, 0.0),
            0,
            true,
        )
        .with_segment(SegmentId(0)),
        // CP 1: Pre-split timing sector at (40, 0)
        Checkpoint::new(
            1,
            LineSegment::new(Vec2::new(40.0, -10.0), Vec2::new(40.0, 10.0)),
            Vec2::new(1.0, 0.0),
            0,
            false,
        )
        .with_segment(SegmentId(0)),
        // CP 2: Main line checkpoint at (110, -25)
        Checkpoint::new(
            2,
            LineSegment::new(Vec2::new(110.0, -35.0), Vec2::new(110.0, -15.0)),
            Vec2::new(1.0, 0.0),
            1,
            false,
        )
        .with_segment(SegmentId(1)),
        // CP 3: Joker line checkpoint at (110, 35) [is_joker: true]
        Checkpoint::new(
            3,
            LineSegment::new(Vec2::new(110.0, 25.0), Vec2::new(110.0, 45.0)),
            Vec2::new(1.0, 0.0),
            1,
            false,
        )
        .with_segment(SegmentId(2))
        .with_joker(true),
        // CP 4: Post-merge timing sector at (190, 0)
        Checkpoint::new(
            4,
            LineSegment::new(Vec2::new(190.0, -10.0), Vec2::new(190.0, 10.0)),
            Vec2::new(1.0, 0.0),
            2,
            false,
        )
        .with_segment(SegmentId(3)),
    ];

    (network, checkpoints)
}

#[test]
fn test_multi_car_free_choice_routes_and_joker_counting() {
    let (network, checkpoints) = create_test_branching_network();

    let mut tracker1 = MultiRouteProgressTracker::from_network(&network, None, 3);
    let mut tracker2 = MultiRouteProgressTracker::from_network(&network, None, 3);

    let mut car1 = Car::new(CarConfig::sports_car());
    let mut car2 = Car::new(CarConfig::sports_car());

    // --- LAP 1 ---
    // Start line crossing
    car1.state.position = Vec2::new(-2.0, 0.0);
    car2.state.position = Vec2::new(-2.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);

    car1.state.position = Vec2::new(2.0, 0.0);
    car2.state.position = Vec2::new(2.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);

    assert_eq!(tracker1.last_checkpoint_id, 0);
    assert_eq!(tracker2.last_checkpoint_id, 0);

    // CP1 Pre-split
    car1.state.position = Vec2::new(38.0, 0.0);
    car2.state.position = Vec2::new(38.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);

    car1.state.position = Vec2::new(42.0, 0.0);
    car2.state.position = Vec2::new(42.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);

    // Car 1 takes MAIN line (CP 2)
    car1.state.position = Vec2::new(108.0, -25.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    car1.state.position = Vec2::new(112.0, -25.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    assert_eq!(tracker1.active_layout_id, "main");
    assert!(!tracker1.is_joker_lap);
    assert!(!tracker1.is_in_branch);

    // Car 2 takes JOKER line (CP 3)
    car2.state.position = Vec2::new(108.0, 35.0);
    tracker2.update(&car2, &network, &checkpoints, 0.016);
    car2.state.position = Vec2::new(112.0, 35.0);
    tracker2.update(&car2, &network, &checkpoints, 0.016);
    assert_eq!(tracker2.active_layout_id, "joker");
    assert!(tracker2.is_joker_lap);
    assert!(tracker2.is_in_branch);

    // Both cross post-merge CP 4
    car1.state.position = Vec2::new(188.0, 0.0);
    car2.state.position = Vec2::new(188.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);

    car1.state.position = Vec2::new(192.0, 0.0);
    car2.state.position = Vec2::new(192.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);
    assert_eq!(tracker1.last_checkpoint_id, 4);
    assert_eq!(tracker2.last_checkpoint_id, 4);
    assert!(!tracker2.is_in_branch); // merged back onto common track

    // Finish line crossing to complete Lap 1
    car1.state.position = Vec2::new(-2.0, 0.0);
    car2.state.position = Vec2::new(-2.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);

    car1.state.position = Vec2::new(2.0, 0.0);
    car2.state.position = Vec2::new(2.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    tracker2.update(&car2, &network, &checkpoints, 0.016);

    assert!(tracker1.lap_completed);
    assert!(tracker2.lap_completed);
    assert_eq!(tracker1.current_lap, 2);
    assert_eq!(tracker2.current_lap, 2);

    // Verify Joker counts after Lap 1
    assert_eq!(tracker1.joker_laps_completed, 0);
    assert_eq!(tracker2.joker_laps_completed, 1);

    // --- LAP 2 ---
    // In Lap 2, Car 1 takes the Joker Lap!
    // CP1 Pre-split
    car1.state.position = Vec2::new(38.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    car1.state.position = Vec2::new(42.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);

    // Car 1 enters Joker branch (CP 3)
    car1.state.position = Vec2::new(108.0, 35.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    car1.state.position = Vec2::new(112.0, 35.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);

    assert_eq!(tracker1.active_layout_id, "joker");
    assert!(tracker1.is_joker_lap);

    // Post-merge CP4
    car1.state.position = Vec2::new(188.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    car1.state.position = Vec2::new(192.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);

    // Finish line CP0
    car1.state.position = Vec2::new(-2.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);
    car1.state.position = Vec2::new(2.0, 0.0);
    tracker1.update(&car1, &network, &checkpoints, 0.016);

    // Verify Car 1 completed Joker lap on Lap 2
    assert!(tracker1.lap_completed);
    assert_eq!(tracker1.current_lap, 3);
    assert_eq!(tracker1.joker_laps_completed, 1);
}

#[test]
fn test_wrong_way_detection_on_divergent_branch() {
    let (network, checkpoints) = create_test_branching_network();
    let mut tracker = MultiRouteProgressTracker::from_network(&network, None, 3);
    let mut car = Car::new(CarConfig::sports_car());

    // Position car on Joker branch facing forward
    car.state.position = Vec2::new(110.0, 35.0);
    car.state.angle = 0.0;
    tracker.current_segment_id = SegmentId(2);
    tracker.update(&car, &network, &checkpoints, 0.016);
    assert!(!tracker.is_wrong_way);

    // Spin car 180 degrees backwards
    car.state.angle = std::f32::consts::PI;
    tracker.update(&car, &network, &checkpoints, 0.016);
    assert!(tracker.is_wrong_way);
    assert!(tracker.wrong_way_timer > 0.0);

    // Backward gate crossing triggers wrong way immediately
    car.state.position = Vec2::new(112.0, 35.0);
    tracker.update(&car, &network, &checkpoints, 0.016);
    car.state.position = Vec2::new(108.0, 35.0);
    tracker.update(&car, &network, &checkpoints, 0.016);
    assert!(tracker.is_wrong_way);
}

#[test]
fn test_track_progress_tracker_update_network_sync() {
    let (network, checkpoints) = create_test_branching_network();
    let mut tracker = TrackProgressTracker::new(checkpoints.len(), 3);
    let mut car = Car::new(CarConfig::sports_car());

    // Start line crossing with update_network
    car.state.position = Vec2::new(-2.0, 0.0);
    tracker.update_network(&car, &network, &checkpoints, 0.016);
    car.state.position = Vec2::new(2.0, 0.0);
    tracker.update_network(&car, &network, &checkpoints, 0.016);

    assert!(tracker.multi_route.is_some());
    assert_eq!(tracker.last_checkpoint_idx, 0);

    // Drive forward along track
    car.state.position = Vec2::new(42.0, 0.0);
    tracker.update_network(&car, &network, &checkpoints, 0.016);
    assert_eq!(tracker.last_checkpoint_idx, 1);
    assert!(tracker.normalized_progress > 0.0);
}
