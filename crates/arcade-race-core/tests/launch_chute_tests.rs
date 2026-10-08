//! Spec 103: launch chute model, packed grid and entry segment.

use arcade_race_core::track::network::{
    JunctionId, LaunchChuteConfig, RoadJunction, RoadSegment, SegmentId, SocketId, SplineSocket, TrackLayout,
    TrackNetwork,
};
use arcade_race_core::track::spline::TrackWaypoint;
use arcade_race_core::track::{generate_packed_launch_grid, BarrierType, PackedGridPattern, WallBarrier};
use glam::Vec2;
use wheelbase::SurfaceType;

#[test]
fn test_packed_grid_five_three_rows() {
    let slots = generate_packed_launch_grid(Vec2::new(100.0, 20.0), Vec2::new(1.0, 0.0), 16.0, PackedGridPattern::AutocrossFiveThree, 4.0);
    assert_eq!(slots.len(), 8);

    // Row 1: slots 0..=4 share one longitudinal distance and have five distinct lateral offsets.
    for slot in &slots[0..5] {
        assert!((slot.position.x - 100.0).abs() < 1e-4);
    }
    let mut lateral: Vec<f32> = slots[0..5].iter().map(|s| s.position.y - 20.0).collect();
    assert!(lateral.windows(2).all(|w| w[0] > w[1]), "slots run from the pad's left to its right: {lateral:?}");
    lateral.sort_by(f32::total_cmp);
    assert!(lateral.windows(2).all(|w| w[1] - w[0] > 2.0));

    // Row 2: slots 5..=7 stand row_spacing behind row 1, facing the same way.
    for slot in &slots[5..8] {
        assert!((slot.position.x - 96.0).abs() < 1e-4);
    }
    assert!(slots.iter().all(|s| s.angle.abs() < 1e-6));
    assert!(slots.iter().enumerate().all(|(i, s)| s.grid_slot == i));
}

#[test]
fn test_packed_grid_patterns_have_their_row_counts() {
    for (pattern, rows) in [
        (PackedGridPattern::AutocrossFiveThree, vec![5, 3]),
        (PackedGridPattern::RallycrossThreeTwoThree, vec![3, 2, 3]),
        (PackedGridPattern::UniformFourAcross, vec![4, 4, 4]),
    ] {
        let slots = generate_packed_launch_grid(Vec2::ZERO, Vec2::new(0.0, 1.0), 16.0, pattern, 6.0);
        assert_eq!(slots.len(), pattern.slot_count());
        let mut counts: Vec<usize> = Vec::new();
        let mut last_y = f32::NAN;
        for slot in &slots {
            if last_y.is_nan() || (slot.position.y - last_y).abs() > 1e-3 {
                counts.push(0);
                last_y = slot.position.y;
            }
            *counts.last_mut().unwrap() += 1;
        }
        assert_eq!(counts, rows, "{pattern:?}");
        // Facing +Y.
        assert!(slots.iter().all(|s| (s.angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5));
    }
}

#[test]
fn test_packed_grid_stays_inside_the_pad() {
    for width in [14.0, 16.0, 18.0] {
        for pattern in [
            PackedGridPattern::AutocrossFiveThree,
            PackedGridPattern::RallycrossThreeTwoThree,
            PackedGridPattern::UniformFourAcross,
        ] {
            for slot in generate_packed_launch_grid(Vec2::ZERO, Vec2::X, width, pattern, 6.0) {
                assert!(slot.position.y.abs() <= width * 0.5 - 1.0, "{pattern:?} width {width}: {:?}", slot.position);
            }
        }
    }
}

#[test]
fn test_packed_grid_staggers_the_two_car_row() {
    let slots = generate_packed_launch_grid(Vec2::ZERO, Vec2::X, 16.0, PackedGridPattern::RallycrossThreeTwoThree, 6.0);
    let front: Vec<f32> = slots[0..3].iter().map(|s| s.position.y).collect();
    let middle: Vec<f32> = slots[3..5].iter().map(|s| s.position.y).collect();
    // No car of the middle row stands directly behind a car of the front row.
    assert!(middle.iter().all(|m| front.iter().all(|f| (m - f).abs() > 1.5)));
}

fn spur_network() -> TrackNetwork {
    let wp = |x: f32, y: f32| TrackWaypoint::new(Vec2::new(x, y), 12.0);
    let chute = RoadSegment::new(SegmentId(2), "Chute", vec![wp(-40.0, 20.0), wp(-20.0, 20.0), wp(0.0, 10.0), wp(20.0, 0.0)])
        .with_junctions(None, Some(SocketId::new(JunctionId(0), 1)));
    let lead = RoadSegment::new(SegmentId(0), "Start", vec![wp(-100.0, 0.0), wp(-50.0, 0.0), wp(20.0, 0.0)])
        .with_junctions(None, Some(SocketId::new(JunctionId(0), 0)));
    let rest = RoadSegment::new(SegmentId(1), "Rest", vec![wp(20.0, 0.0), wp(100.0, 0.0), wp(100.0, -80.0), wp(-100.0, -80.0), wp(-100.0, 0.0)])
        .with_junctions(Some(SocketId::new(JunctionId(0), 0)), None);

    let socket = SplineSocket::new(Vec2::new(20.0, 0.0), Vec2::X, 12.0);
    let merge = RoadJunction::merge(JunctionId(0), "Chute Merge", vec![socket, socket], socket, None);
    TrackNetwork {
        junctions: vec![merge],
        segments: vec![lead, rest, chute],
        layouts: vec![TrackLayout::new("main", "Main", vec![SegmentId(0), SegmentId(1)], SegmentId(0)).with_entry_segment(Some(SegmentId(2)))],
        default_layout_id: "main".to_string(),
        ..Default::default()
    }
}

#[test]
fn test_entry_segment_leads_into_the_loop() {
    let net = spur_network();
    assert!(net.validate().is_ok(), "{:?}", net.validate());
    let layout = net.get_layout("main").unwrap();
    assert_eq!(net.entry_continuation_segment(layout), Some(SegmentId(1)));

    // The first-lap route is chute, then the loop from the merge round to the merge.
    let route = net.build_entry_spline_for_layout("main").unwrap();
    assert!(!route.closed);
    assert!((route.sample_at_distance(0.0).point - Vec2::new(-40.0, 20.0)).length() < 0.5);
    let end = route.sample_at_distance(route.total_length());
    assert!((end.point - Vec2::new(20.0, 0.0)).length() < 0.5, "route ends at the merge: {:?}", end.point);
    assert!(route.total_length() > net.get_segment(SegmentId(2)).unwrap().length + net.get_segment(SegmentId(1)).unwrap().length);
}

#[test]
fn test_entry_segment_inside_the_loop_is_invalid() {
    let mut net = spur_network();
    net.layouts[0].segment_sequence.push(SegmentId(2));
    let errors = net.validate().unwrap_err();
    assert!(errors.iter().any(|e| e.contains("entry segment")), "{errors:?}");

    let mut net = spur_network();
    net.layouts[0].entry_segment = Some(SegmentId(9));
    assert!(net.validate().is_err());
}

#[test]
fn test_launch_chute_config_round_trips_through_json() {
    let mut net = spur_network();
    net.launch_chute = Some(LaunchChuteConfig {
        segment_id: SegmentId(2),
        terminal_barrier: WallBarrier::new(Vec2::new(-42.0, 12.0), Vec2::new(-42.0, 28.0), BarrierType::Concrete),
        side_barriers: vec![WallBarrier::new(Vec2::new(-42.0, 28.0), Vec2::new(-10.0, 28.0), BarrierType::Concrete)],
        merge_junction_id: JunctionId(0),
        grid_slots: generate_packed_launch_grid(Vec2::new(-30.0, 20.0), Vec2::X, 16.0, PackedGridPattern::AutocrossFiveThree, 6.0),
        surface: SurfaceType::Concrete,
        pad_width: 16.0,
    });
    let json = serde_json::to_string(&net).unwrap();
    let back: TrackNetwork = serde_json::from_str(&json).unwrap();
    assert_eq!(back.launch_chute, net.launch_chute);
    assert_eq!(back.layouts[0].entry_segment, Some(SegmentId(2)));

    // A circuit without a chute writes neither field.
    let mut plain = spur_network();
    plain.layouts[0].entry_segment = None;
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("entry_segment") && !json.contains("launch_chute"));
}
