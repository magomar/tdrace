//! Spec 112: Track geometry tests for adaptive median barriers and wall suppression diagnostics.

use arcade_race_core::track::geometry::{BarrierType, LineSegment, WallSuppressionRecord};
use arcade_race_core::track::junction_kit::Side;
use arcade_race_core::track::network::{
    clearance_gap, generate_adaptive_median_barriers, RoadSegment, SegmentId,
};
use arcade_race_core::track::spline::TrackWaypoint;
use glam::Vec2;

#[test]
fn test_adaptive_median_barrier_in_diverging_gore() {
    // Road 1 straight along y = 0, width 8.0 m (edges at y = -4.0 and +4.0 m)
    let wps1 = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
        TrackWaypoint::new(Vec2::new(50.0, 0.0), 8.0),
        TrackWaypoint::new(Vec2::new(100.0, 0.0), 8.0),
    ];
    let seg1 = RoadSegment::new(SegmentId(1), "Trunk / Main", wps1);

    // Road 2 starts at (0, 0) and diverges at ~20 degrees (tan(20 deg) ~= 0.364)
    // Width 8.0 m.
    let wps2 = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
        TrackWaypoint::new(Vec2::new(50.0, 18.2), 8.0),
        TrackWaypoint::new(Vec2::new(100.0, 36.4), 8.0),
    ];
    let seg2 = RoadSegment::new(SegmentId(2), "Diverging Branch", wps2);

    let barriers = generate_adaptive_median_barriers(&seg1, &seg2, BarrierType::TireWall);
    assert!(!barriers.is_empty(), "Should generate adaptive median barriers in diverging gore");

    for b in &barriers {
        // Each barrier segment must maintain >= 0.3 m clearance from both road edges
        let mid = b.segment.midpoint();
        let proj1 = seg1.project_point(mid);
        let proj2 = seg2.project_point(mid);

        let clr1 = proj1.distance_to_spline - proj1.track_width * 0.5;
        let clr2 = proj2.distance_to_spline - proj2.track_width * 0.5;

        assert!(
            clr1 >= 0.29,
            "Barrier point {:?} must maintain at least 0.3 m clearance from Road 1 (got {:.2} m)",
            mid,
            clr1
        );
        assert!(
            clr2 >= 0.29,
            "Barrier point {:?} must maintain at least 0.3 m clearance from Road 2 (got {:.2} m)",
            mid,
            clr2
        );
    }

    // Verify first barrier starts where clearance gap G(t) >= 2.2 m
    let first_pt = barriers[0].segment.start;
    let proj1 = seg1.project_point(first_pt);
    let gap = clearance_gap(&seg1, &seg2, proj1.progress_distance).expect("gap computed");
    assert!(
        gap >= 2.15,
        "Adaptive median barrier should start when G(t) >= 2.2 m (got {:.2} m)",
        gap
    );
}

#[test]
fn test_adaptive_median_barrier_bypassed_if_angle_exceeds_60_degrees() {
    // Road 1 along y = 0
    let wps1 = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
        TrackWaypoint::new(Vec2::new(50.0, 0.0), 8.0),
    ];
    let seg1 = RoadSegment::new(SegmentId(1), "Road 1", wps1);

    // Road 2 diverges at 90 degrees (along +y) -> angle 90 > 60 deg
    let wps2 = vec![
        TrackWaypoint::new(Vec2::new(0.0, 0.0), 8.0),
        TrackWaypoint::new(Vec2::new(0.0, 50.0), 8.0),
    ];
    let seg2 = RoadSegment::new(SegmentId(2), "Sharp Turn Branch", wps2);

    let barriers = generate_adaptive_median_barriers(&seg1, &seg2, BarrierType::TireWall);
    assert!(
        barriers.is_empty(),
        "Should bypass adaptive median barrier when divergence angle > 60 deg"
    );
}

#[test]
fn test_wall_suppression_record_properties() {
    let rec = WallSuppressionRecord::new(
        Side::Left,
        LineSegment::new(Vec2::new(10.0, 5.0), Vec2::new(20.0, 5.0)),
        SegmentId(3),
        10.0,
        20.0,
    );
    assert_eq!(rec.side, Side::Left);
    assert_eq!(rec.conflicting_segment, SegmentId(3));
    assert!(rec.contains_station(15.0, 1.0));
    assert!(rec.contains_station(9.5, 1.0));
    assert!(!rec.contains_station(25.0, 1.0));
    assert!((rec.distance_to_point(Vec2::new(15.0, 8.0)) - 3.0).abs() < 1e-4);
}
