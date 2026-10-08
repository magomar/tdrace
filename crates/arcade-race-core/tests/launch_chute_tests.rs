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

mod stamp {
    use arcade_race_core::track::network::JunctionKind;
    use arcade_race_core::track::validation::{validate_track, ValidationSeverity};
    use arcade_race_core::track::{rally_template, ChuteSide, LaunchChuteSpec, PackedGridPattern, RaceDirection, Track, TrackShape};
    use glam::Vec2;
    use wheelbase::SurfaceType;

    fn rally_oval() -> Track {
        rally_template(TrackShape::Oval, RaceDirection::Right)
    }

    fn errors(track: &Track) -> Vec<String> {
        validate_track(track)
            .into_iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect()
    }

    /// A waypoint 10-250 m before the finish line.
    fn merge_waypoint(track: &Track) -> usize {
        let mut spec = LaunchChuteSpec::new(0, ChuteSide::Left);
        let mut probe = track.clone();
        spec = probe.place_launch_chute(&spec).unwrap_or(spec);
        spec.merge_waypoint
    }

    #[test]
    fn test_stamp_builds_a_valid_chute_on_a_plain_circuit() {
        let mut track = rally_oval();
        assert!(track.network.is_none());
        let spec = track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap();

        let net = track.network.as_ref().unwrap();
        let chute = net.launch_chute.as_ref().unwrap();
        assert!(net.validate().is_ok(), "{:?}", net.validate());
        assert_eq!(errors(&track), Vec::<String>::new());

        // The chute is the run-once entry of the layout, outside the loop, and ends in a merge junction.
        let layout = net.get_layout("main").unwrap();
        assert_eq!(layout.entry_segment, Some(chute.segment_id));
        assert!(!layout.segment_sequence.contains(&chute.segment_id));
        assert_eq!(layout.segment_sequence.len(), 2, "the loop is split at the merge: {:?}", layout.segment_sequence);
        assert!(matches!(net.get_junction(chute.merge_junction_id).unwrap().kind, JunctionKind::Merge { .. }));

        // Eight packed slots on the pad, which is the starting grid.
        assert_eq!(chute.grid_slots.len(), 8);
        assert_eq!(track.grid_positions, chute.grid_slots);
        assert!(chute.pad_width == spec.pad_width && chute.surface == SurfaceType::Concrete);

        // Walls: a rear barrier at least as long as the pad, and side walls.
        assert!(chute.terminal_barrier.segment.length() >= chute.pad_width);
        assert!(!chute.side_barriers.is_empty());
        assert!(track.geometry.network_walls.contains(&chute.terminal_barrier));

        // Every checkpoint is on one of the two loop parts.
        let (main, after) = (layout.segment_sequence[0], layout.segment_sequence[1]);
        assert!(track.checkpoints.iter().all(|cp| cp.segment_id == Some(main) || cp.segment_id == Some(after)));
    }

    #[test]
    fn test_stamped_chute_merges_tangent_to_the_loop() {
        let mut track = rally_oval();
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Right)).unwrap();
        let net = track.network.as_ref().unwrap();
        let chute = net.launch_chute.as_ref().unwrap();
        let seg = net.get_segment(chute.segment_id).unwrap();
        let end = seg.samples.last().unwrap();
        let on_loop = track.spline.project_point(end.point);
        assert!(on_loop.distance_to_spline < 0.5, "chute ends on the loop: {}", on_loop.distance_to_spline);
        assert!(end.tangent.dot(on_loop.tangent) > 0.999, "tangent {:?} vs {:?}", end.tangent, on_loop.tangent);
        // The grid starts at the rear of the chute, far from the loop.
        let first = chute.grid_slots[0].position;
        assert!(track.spline.project_point(first).distance_to_spline > 15.0);
    }

    #[test]
    fn test_stamp_survives_a_json_round_trip() {
        let mut track = rally_oval();
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap();
        let saved = Track::from_json(&track.to_json().unwrap()).unwrap();
        assert_eq!(saved.launch_chute(), track.launch_chute());
        assert_eq!(saved.grid_positions, track.grid_positions);
        assert_eq!(saved.network.as_ref().unwrap().layouts, track.network.as_ref().unwrap().layouts);
        assert_eq!(errors(&saved), Vec::<String>::new());
        // The walls are rebuilt on load and match.
        assert_eq!(saved.geometry.network_walls.len(), track.geometry.network_walls.len());
    }

    #[test]
    fn test_stamp_refuses_bad_requests_and_leaves_the_track_alone() {
        let mut track = rally_oval();
        let before = track.clone();
        let k = merge_waypoint(&track);

        // A waypoint too far before the finish line (waypoint 0 is the finish line itself).
        assert!(track.stamp_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).is_err());
        let mut narrow = LaunchChuteSpec::new(k, ChuteSide::Left);
        narrow.pad_width = 10.0;
        assert!(track.stamp_launch_chute(&narrow).unwrap_err().0.contains("pad width"));
        let mut short = LaunchChuteSpec::new(k, ChuteSide::Left);
        short.pad_length = 20.0;
        assert!(track.stamp_launch_chute(&short).unwrap_err().0.contains("pad length"));
        let mut gravel = LaunchChuteSpec::new(k, ChuteSide::Left);
        gravel.surface = SurfaceType::Grass;
        assert!(track.stamp_launch_chute(&gravel).is_err());
        assert!(track.stamp_launch_chute(&LaunchChuteSpec::new(999, ChuteSide::Left)).is_err());
        assert_eq!(track, before);

        // One chute per circuit.
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap();
        assert!(track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).is_err());
    }

    #[test]
    fn test_patterns_and_surface_are_applied() {
        let mut track = rally_oval();
        let mut template = LaunchChuteSpec::new(0, ChuteSide::Left);
        template.pattern = PackedGridPattern::RallycrossThreeTwoThree;
        template.surface = SurfaceType::Asphalt;
        template.pad_width = 18.0;
        track.place_launch_chute(&template).unwrap();
        let chute = track.launch_chute().unwrap();
        assert_eq!(chute.grid_slots.len(), 8);
        assert_eq!(chute.surface, SurfaceType::Asphalt);
        assert_eq!(chute.pad_width, 18.0);
        let seg = track.network.as_ref().unwrap().get_segment(chute.segment_id).unwrap();
        assert_eq!(seg.samples[0].surface, SurfaceType::Asphalt);
        assert!((seg.samples[0].width - 18.0).abs() < 0.1);
        // The pad surface is what cars on the grid drive on.
        assert_eq!(track.sample_surface(chute.grid_slots[0].position), SurfaceType::Asphalt);
        assert_eq!(errors(&track), Vec::<String>::new());
    }

    #[test]
    fn test_remove_restores_a_plain_circuit() {
        let mut track = rally_oval();
        let plain_grid = track.grid_positions.len();
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap();
        assert!(track.remove_launch_chute());
        assert!(track.network.is_none());
        assert!(track.launch_chute().is_none());
        assert_eq!(track.grid_positions.len(), plain_grid);
        assert_eq!(errors(&track), Vec::<String>::new());
        assert!(!track.remove_launch_chute());
        // And a chute can be stamped again.
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Right)).unwrap();
        assert!(track.launch_chute().is_some());
    }

    fn codes(track: &Track) -> Vec<&'static str> {
        validate_track(track).into_iter().filter(|d| d.severity == ValidationSeverity::Error).map(|d| d.code).collect()
    }

    #[test]
    fn test_validation_rejects_a_chute_with_an_open_end() {
        let mut track = rally_oval();
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap();

        // The rear barrier no longer meets the side walls.
        let mut open = track.clone();
        let chute = open.network.as_mut().unwrap().launch_chute.as_mut().unwrap();
        chute.terminal_barrier.segment.start += Vec2::new(0.0, 3.0);
        assert!(codes(&open).contains(&"ERR_CHUTE_OPEN_END"), "{:?}", codes(&open));

        // A gap along the pad: drop the walls of one side.
        let mut gap = track.clone();
        let chute = gap.network.as_mut().unwrap().launch_chute.as_mut().unwrap();
        let rear = chute.terminal_barrier.segment.start;
        chute.side_barriers.retain(|w| w.segment.start.distance(rear) > 30.0);
        assert!(codes(&gap).contains(&"ERR_CHUTE_WALL_GAP"), "{:?}", codes(&gap));
    }

    #[test]
    fn test_validation_rejects_bad_grid_merge_and_topology() {
        let mut track = rally_oval();
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap();

        let mut outside = track.clone();
        outside.network.as_mut().unwrap().launch_chute.as_mut().unwrap().grid_slots[2].position += Vec2::new(0.0, 12.0);
        outside.grid_positions = outside.launch_chute().unwrap().grid_slots.clone();
        assert!(codes(&outside).contains(&"ERR_CHUTE_GRID"), "{:?}", codes(&outside));

        let mut loose_grid = track.clone();
        loose_grid.grid_positions.pop();
        assert!(codes(&loose_grid).contains(&"ERR_CHUTE_GRID"));

        let mut kinked = track.clone();
        let net = kinked.network.as_mut().unwrap();
        let id = net.launch_chute.as_ref().unwrap().segment_id;
        let seg = net.get_segment_mut(id).unwrap();
        let last = seg.samples.len() - 1;
        seg.samples[last].tangent = Vec2::new(0.0, 1.0);
        assert!(codes(&kinked).contains(&"ERR_CHUTE_MERGE_NOT_C1"), "{:?}", codes(&kinked));

        let mut every_lap = track.clone();
        let net = every_lap.network.as_mut().unwrap();
        let id = net.launch_chute.as_ref().unwrap().segment_id;
        net.layouts[0].segment_sequence.push(id);
        assert!(codes(&every_lap).contains(&"ERR_CHUTE_TOPOLOGY"));

        let mut no_config = track.clone();
        no_config.network.as_mut().unwrap().launch_chute = None;
        assert!(codes(&no_config).contains(&"ERR_CHUTE_NO_CONFIG"));
    }

    #[test]
    fn test_grid_slots_are_inside_the_pad_and_clear_of_walls() {
        let mut track = rally_oval();
        track.place_launch_chute(&LaunchChuteSpec::new(0, ChuteSide::Left)).unwrap();
        let chute = track.launch_chute().unwrap();
        let seg = track.network.as_ref().unwrap().get_segment(chute.segment_id).unwrap();
        for slot in &chute.grid_slots {
            let proj = seg.project_point(slot.position);
            assert!(proj.is_on_track);
            assert!(proj.distance_to_spline <= chute.pad_width * 0.5 - 1.0, "{:?}", slot);
            let heading = Vec2::new(slot.angle.cos(), slot.angle.sin());
            assert!(heading.dot(proj.tangent) > 0.95);
            for wall in track.geometry.all_walls() {
                assert!(wall.segment.distance_to_point(slot.position) > 0.8);
            }
        }
    }
}
