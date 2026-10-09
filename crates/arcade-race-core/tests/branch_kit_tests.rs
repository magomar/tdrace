//! Spec 102: road branches built from predefined junction components (compile, guards, walls, surfaces).

use arcade_race_core::track::bake::{bake, BakeOptions};
use arcade_race_core::track::branch_kit::{BranchKitError, BranchLayout};
use arcade_race_core::track::geometry::BarrierType;
use arcade_race_core::track::junction_kit::{
    JunctionComponent, JunctionError, JunctionShape, Side,
};
use arcade_race_core::track::network::{JunctionKind, SegmentId};
use arcade_race_core::track::spline::{TrackSpline, TrackWaypoint};
use arcade_race_core::track::{validate_track, Track, ValidationSeverity};
use glam::Vec2;
use wheelbase::SurfaceType;

const TRACK_WIDTH: f32 = 12.0;
const ROAD_WIDTH: f32 = 8.0;

/// Counter-clockwise stadium: 400 m straight along +x from the origin, radius-100 m left-hand ends, one waypoint
/// every 10 m on the straights. The waypoint at `10 * i` m is at arc length `10 * i` for `i <= 40`.
fn stadium_waypoints() -> Vec<TrackWaypoint> {
    let mut pts = Vec::new();
    for i in 0..40 {
        pts.push(Vec2::new(i as f32 * 10.0, 0.0));
    }
    for i in 0..32 {
        let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 32.0;
        pts.push(Vec2::new(400.0 + 100.0 * a.cos(), 100.0 + 100.0 * a.sin()));
    }
    for i in 0..40 {
        pts.push(Vec2::new(400.0 - i as f32 * 10.0, 200.0));
    }
    for i in 0..32 {
        let a = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 32.0;
        pts.push(Vec2::new(100.0 * a.cos(), 100.0 + 100.0 * a.sin()));
    }
    pts.into_iter().map(|p| TrackWaypoint::new(p, TRACK_WIDTH).with_surface(SurfaceType::Asphalt)).collect()
}

fn junction(s: f32, divider_gap: f32, length: f32) -> JunctionComponent {
    JunctionComponent { s, kind: JunctionShape::Taper, length, divider_gap }
}

fn road_point(x: f32, y: f32, surface: SurfaceType) -> TrackWaypoint {
    TrackWaypoint::new(Vec2::new(x, y), ROAD_WIDTH).with_surface(surface)
}

/// A joker-like branch on the right of the bottom straight: split at 100 m, merge at 300 m, and a free road that
/// bends 40 m away from the main route.
fn layout() -> BranchLayout {
    BranchLayout {
        layout_id: "joker".to_string(),
        name: "Test Joker".to_string(),
        side: Side::Right,
        split: junction(100.0, 4.0, 30.0),
        merge: junction(300.0, 4.0, 30.0),
        road_waypoints: vec![
            road_point(165.0, -30.0, SurfaceType::Dirt),
            road_point(200.0, -40.0, SurfaceType::Dirt),
            road_point(235.0, -30.0, SurfaceType::Dirt),
        ],
        road_width: ROAD_WIDTH,
        nose_barrier: BarrierType::TireWall,
    }
}

/// A baked circuit with the layout compiled in: main walls, checkpoints, grid and the branch network.
fn track_with(layout: BranchLayout) -> Track {
    let mut track = Track {
        name: "Stadium".to_string(),
        spline: TrackSpline::new(stadium_waypoints(), true),
        default_surface: SurfaceType::Grass,
        ..Track::default()
    };
    track.branch_layout = Some(layout);
    bake(&mut track, &BakeOptions { rebuild: true, barrier_offset: Some(3.0), barrier_type: Some(BarrierType::TireWall), ..BakeOptions::default() })
        .expect("bake compiles the branch");
    track
}

fn bare_track() -> Track {
    Track { spline: TrackSpline::new(stadium_waypoints(), true), ..Track::default() }
}

#[test]
fn a_branch_layout_compiles_to_four_segments_two_junctions_and_both_layouts() {
    let track = bare_track();
    let compiled = layout().compile(&track).expect("compiles");
    let net = &compiled.network;
    assert_eq!(net.segments.len(), 4);
    assert_eq!(net.junctions.len(), 2);
    assert!(matches!(net.junctions[0].kind, JunctionKind::Split { .. }));
    assert!(matches!(net.junctions[1].kind, JunctionKind::Merge { .. }));
    let ids = |layout: &str| net.get_layout(layout).unwrap().segment_sequence.iter().map(|s| s.0).collect::<Vec<_>>();
    assert_eq!(ids("main"), vec![0, 1, 3]);
    assert_eq!(ids("joker"), vec![0, 2, 3]);
    assert_eq!(net.default_layout_id, "main");
    assert!(net.validate().is_ok());
    // The joker route is longer than the main one.
    assert!(net.get_layout("joker").unwrap().total_lap_length > net.get_layout("main").unwrap().total_lap_length);
}

#[test]
fn compiling_twice_gives_the_same_network() {
    let track = bare_track();
    let a = layout().compile(&track).unwrap();
    let b = layout().compile(&track).unwrap();
    assert_eq!(a, b);
}

#[test]
fn sockets_are_real_and_continuous_with_the_main_road() {
    let track = bare_track();
    let compiled = layout().compile(&track).unwrap();
    let JunctionKind::Split { ingress_socket, egress_sockets, gore_config } = &compiled.network.junctions[0].kind else {
        panic!("split")
    };
    let gore = gore_config.as_ref().unwrap();
    assert!(!gore.has_chevrons);
    assert_eq!(egress_sockets.len(), 2);
    for e in egress_sockets {
        assert!(ingress_socket.tangent.perp_dot(e.tangent).atan2(ingress_socket.tangent.dot(e.tangent)).abs() < 1e-4);
    }
    // A Taper starts centred on the main road at the anchor, 4 m to the right of nothing yet.
    assert!(egress_sockets[1].point.distance(egress_sockets[0].point) < 0.01);
    assert!((egress_sockets[1].width - ROAD_WIDTH).abs() < 1e-4);
    // The gore is a real wedge: apex on the main edge, a nose length ahead.
    assert!((gore.apex_point.y + TRACK_WIDTH * 0.5).abs() < 0.3, "apex {:?}", gore.apex_point);
    assert!(gore.gore_length > 1.0 && gore.divergence_angle > 0.0);
    let len = (gore.nose_barrier.segment.end - gore.nose_barrier.segment.start).length();
    assert!((len - 1.6).abs() < 1e-3, "nose barrier is {len} m");
}

#[test]
fn the_split_apex_lies_where_the_branch_edge_leaves_the_main_edge() {
    let track = bare_track();
    let compiled = layout().compile(&track).unwrap();
    let seg = compiled.network.get_segment(SegmentId(2)).unwrap();
    // Right side of a +x straight is -y. Branch inner edge is its left edge; the main edge is y = -6.
    let leaves = seg
        .samples
        .iter()
        .map(|s| s.point + s.normal * (s.width * 0.5))
        .find(|edge| edge.x > 100.0 && edge.y < -TRACK_WIDTH * 0.5)
        .expect("the branch leaves the main road");
    let apex = compiled.geometry.split.edge_apex;
    assert!(
        (leaves.x - apex.x).abs() < 0.5,
        "branch edge leaves at x {:.2}, apex at x {:.2}",
        leaves.x,
        apex.x
    );
}

#[test]
fn bake_installs_the_network_and_validation_finds_no_error() {
    let track = track_with(layout());
    assert!(track.network.is_some());
    let errors: Vec<_> = validate_track(&track).into_iter().filter(|d| d.severity == ValidationSeverity::Error).collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn bake_twice_changes_nothing() {
    let mut track = track_with(layout());
    let first = (track.network.clone(), track.geometry.inner_walls.clone(), track.geometry.outer_walls.clone(), track.geometry.network_walls.clone());
    bake(&mut track, &BakeOptions { rebuild: true, barrier_offset: Some(3.0), barrier_type: Some(BarrierType::TireWall), ..BakeOptions::default() }).unwrap();
    assert_eq!(first.0, track.network);
    assert_eq!(first.1, track.geometry.inner_walls);
    assert_eq!(first.2, track.geometry.outer_walls);
    assert_eq!(first.3, track.geometry.network_walls);
}

// ---------------------------------------------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn guards_reject_bad_layouts_in_order() {
    let track = bare_track();
    let mut cases: Vec<(BranchLayout, &str)> = Vec::new();

    let mut l = layout();
    l.split.s = 104.0;
    cases.push((l, "AnchorOffWaypoint"));

    let mut l = layout();
    l.split.divider_gap = 1.5;
    cases.push((l, "NoseOutsideJunction"));

    // The free road comes within 1.0 m of the main road between the noses.
    let mut l = layout();
    l.road_waypoints = vec![road_point(165.0, -20.0, SurfaceType::Dirt), road_point(200.0, -11.0, SurfaceType::Dirt), road_point(235.0, -20.0, SurfaceType::Dirt)];
    cases.push((l, "DividerTooNarrow"));

    // A bend too tight for the road: a hairpin of the free road.
    let mut l = layout();
    l.road_waypoints = vec![road_point(165.0, -30.0, SurfaceType::Dirt), road_point(172.0, -30.0, SurfaceType::Dirt), road_point(172.0, -36.0, SurfaceType::Dirt), road_point(165.0, -36.0, SurfaceType::Dirt), road_point(165.0, -42.0, SurfaceType::Dirt), road_point(235.0, -30.0, SurfaceType::Dirt)];
    cases.push((l, "RoadTooTight"));

    // The merge lies inside the split span.
    let mut l = layout();
    l.merge.s = 110.0;
    cases.push((l, "JunctionOrder"));

    let mut l = layout();
    l.split = junction(100.0, 40.0, 8.0);
    cases.push((l, "Junction(JunctionTooSteep)"));

    for (l, expected) in cases {
        let Err(err) = l.compile(&track) else { panic!("{expected}: the layout compiled") };
        let name = match &err {
            BranchKitError::AnchorOffWaypoint { .. } => "AnchorOffWaypoint".to_string(),
            BranchKitError::NoseOutsideJunction { .. } => "NoseOutsideJunction".to_string(),
            BranchKitError::DividerTooNarrow { .. } => "DividerTooNarrow".to_string(),
            BranchKitError::RoadTooTight { .. } => "RoadTooTight".to_string(),
            BranchKitError::JunctionOrder => "JunctionOrder".to_string(),
            BranchKitError::Junction(JunctionError::JunctionTooSteep { .. }) => "Junction(JunctionTooSteep)".to_string(),
            other => format!("{other:?}"),
        };
        assert_eq!(name, expected, "{err:?}");
    }
}

#[test]
fn the_layout_id_must_name_a_branch() {
    let track = bare_track();
    for id in ["", "main", "Main"] {
        let mut l = layout();
        l.layout_id = id.to_string();
        assert_eq!(l.compile(&track), Err(BranchKitError::InvalidParameter { name: "layout_id" }));
    }
}

#[test]
fn nan_parameters_fail_closed() {
    let track = bare_track();
    let mut l = layout();
    l.road_width = f32::NAN;
    assert!(l.compile(&track).is_err());
    let mut l = layout();
    l.split.length = f32::NAN;
    assert!(l.compile(&track).is_err());
    let mut l = layout();
    l.road_waypoints[0].point = Vec2::new(f32::NAN, 0.0);
    assert!(l.compile(&track).is_err());
}

#[test]
fn a_branch_can_wrap_across_the_start_finish_line() {
    // Split on the second turn (waypoint 125), merge on the first straight past the start line (waypoint 6): the
    // branch swings round the outside of the turn, through the start/finish line.
    let track = bare_track();
    let arc = |i: usize| track.spline.project_point(track.spline.waypoints[i].point).progress_distance;
    let mut l = layout();
    l.split = junction(arc(125), 4.0, 30.0);
    l.merge = junction(arc(6), 4.0, 30.0);
    l.road_waypoints = [(200.0f32, 128.0f32), (232.0, 128.0), (262.0, 128.0)]
        .iter()
        .map(|&(deg, r)| {
            let p = Vec2::new(deg.to_radians().cos() * r, 100.0 + deg.to_radians().sin() * r);
            road_point(p.x, p.y, SurfaceType::Dirt)
        })
        .collect();
    assert!(l.merge.s < l.split.s);
    let compiled = match l.compile(&track) {
        Ok(c) => c,
        Err(e) => panic!("wrap layout: {e:?}"),
    };
    assert!(compiled.geometry.wraps);
    let net = &compiled.network;
    assert!(net.validate().is_ok());
    assert_eq!(net.segments.len(), 3, "trunk, main route and branch");
    for id in ["main", "joker"] {
        assert!(net.get_layout(id).unwrap().is_closed, "{id} layout is closed");
        let spline = net.build_composite_spline_for_layout(id).expect("composite");
        assert!(spline.closed);
    }
    // The branch is one continuous spline from the split to the merge.
    let seg = net.get_segment(SegmentId(2)).unwrap();
    let gaps = seg.samples.windows(2).map(|w| w[0].point.distance(w[1].point)).fold(0.0f32, f32::max);
    assert!(gaps < 1.0, "branch has a {gaps} m jump");
    assert!(seg.samples[0].point.distance(track.spline.sample_at_distance(l.split.s).point) < 8.0);
    assert!(seg.samples[seg.samples.len() - 1].point.distance(track.spline.sample_at_distance(l.merge.s).point) < 8.0);
}

// ---------------------------------------------------------------------------------------------------------------
// Walls (Pillar IV)
// ---------------------------------------------------------------------------------------------------------------

use arcade_race_core::track::branch_kit::nose_barrier;
use arcade_race_core::track::geometry::{LineSegment, WallBarrier};

/// Gap between a road edge and its wall on the test circuit.
const G: f32 = 3.0;

fn walls_of(track: &Track) -> Vec<WallBarrier> {
    track.geometry.all_walls().copied().collect()
}

fn ends(walls: &[WallBarrier]) -> Vec<(usize, Vec2)> {
    walls.iter().enumerate().flat_map(|(i, w)| [(i, w.segment.start), (i, w.segment.end)]).collect()
}

/// True when `ray` meets a wall, counting a ray through a shared wall end.
fn ray_meets_wall(walls: &[WallBarrier], ray: &LineSegment) -> bool {
    walls.iter().any(|w| {
        w.segment.intersect_segment(ray).is_some() || [w.segment.start, w.segment.end].iter().any(|p| ray.distance_to_point(*p) < 0.02)
    })
}

#[test]
fn the_outer_wall_is_unbroken_from_the_main_road_round_the_branch() {
    let track = track_with(layout());
    let compiled = layout().compile(&track).unwrap();
    let walls = walls_of(&track);
    let env = compiled.geometry.split.outer_envelope(&track.spline, Side::Right, ROAD_WIDTH);
    let seg = compiled.network.get_segment(SegmentId(2)).unwrap();
    let (anchor, free) = (100.0f32, 130.0f32);

    // Rays outward every metre from the outer road edge, 10 m before the anchor to 10 m after the free end.
    let mut misses = Vec::new();
    let mut x = anchor - 10.0;
    while x <= free + 10.0 {
        let (edge, out) = if x < anchor {
            (Vec2::new(x, -TRACK_WIDTH * 0.5), Vec2::new(0.0, -1.0))
        } else if x <= free {
            let e = env[(x - anchor).round() as usize];
            (e.point(), e.outward)
        } else {
            // Past the free end the branch road leaves along its own heading: take its outer edge from the segment.
            let s = seg.samples.iter().find(|s| s.point.x >= x).unwrap();
            (s.point - s.normal * (s.width * 0.5), -s.normal)
        };
        let ray = LineSegment::new(edge, edge + out * (G + 1.0));
        if !ray_meets_wall(&walls, &ray) {
            misses.push(x);
        }
        x += 1.0;
    }
    assert!(misses.is_empty(), "no wall within g + 1 m at x = {misses:?}");

    // The same at the merge.
    let env = compiled.geometry.merge.outer_envelope(&track.spline, Side::Right, ROAD_WIDTH);
    let mut misses = Vec::new();
    let mut x = 300.0 - 40.0;
    while x <= 300.0 + 10.0 {
        let (edge, out) = if x < 270.0 {
            let s = seg.samples.iter().rev().find(|s| s.point.x <= x).unwrap();
            (s.point - s.normal * (s.width * 0.5), -s.normal)
        } else if x <= 300.0 {
            let e = env[(x - 270.0).round() as usize];
            (e.point(), e.outward)
        } else {
            (Vec2::new(x, -TRACK_WIDTH * 0.5), Vec2::new(0.0, -1.0))
        };
        let ray = LineSegment::new(edge, edge + out * (G + 1.0));
        if !ray_meets_wall(&walls, &ray) {
            misses.push(x);
        }
        x += 1.0;
    }
    assert!(misses.is_empty(), "no wall within g + 1 m at the merge, x = {misses:?}");
}

#[test]
fn no_wall_end_in_the_junction_region_is_open() {
    let track = track_with(layout());
    let walls = walls_of(&track);
    let all = ends(&walls);
    let mut open = Vec::new();
    for &(i, p) in &all {
        // Right of the main road, within 10 m of a junction span.
        let in_split = p.x > 90.0 && p.x < 140.0;
        let in_merge = p.x > 260.0 && p.x < 310.0;
        if !(p.y < -TRACK_WIDTH * 0.5 && (in_split || in_merge)) {
            continue;
        }
        if !all.iter().any(|&(j, q)| j != i && q.distance(p) <= 0.05) {
            open.push(p);
        }
    }
    assert!(open.is_empty(), "open wall ends: {open:?}");
}

#[test]
fn the_island_is_a_closed_loop_with_a_collidable_nose() {
    let track = track_with(layout());
    let compiled = layout().compile(&track).unwrap();
    let walls = track.geometry.network_walls.clone();
    let main = &track.spline;

    let (nose, _) = nose_barrier(&compiled.geometry.split, BarrierType::TireWall);
    let (cap, _) = nose_barrier(&compiled.geometry.merge, BarrierType::TireWall);
    let nose_i = walls.iter().position(|w| w.segment == nose.segment).expect("nose is a network wall");
    let cap_i = walls.iter().position(|w| w.segment == cap.segment).expect("cap is a network wall");
    assert!((nose.segment.length() - 1.6).abs() < 1e-3 && (cap.segment.length() - 1.6).abs() < 1e-3);

    // Walk the loop from the nose: every wall end touches exactly one other end.
    let all = ends(&walls);
    let mut seen = vec![nose_i];
    let mut frontier = vec![nose_i];
    while let Some(i) = frontier.pop() {
        for (_, p) in all.iter().filter(|(j, _)| *j == i) {
            let touching: Vec<usize> = all.iter().filter(|(j, q)| *j != i && q.distance(*p) <= 0.05).map(|(j, _)| *j).collect();
            assert_eq!(touching.len(), 1, "wall {i} end {p:?} touches {touching:?}");
            if !seen.contains(&touching[0]) {
                seen.push(touching[0]);
                frontier.push(touching[0]);
            }
        }
    }
    assert!(seen.contains(&cap_i), "the cap closes the loop with the nose");
    assert!(seen.len() >= 4, "nose, cap and two dividers: {}", seen.len());

    // The nose is perpendicular to the bisector of the two road edges within 2 degrees.
    let seg = compiled.network.get_segment(SegmentId(2)).unwrap();
    let nose_point = compiled.geometry.split.nose.unwrap();
    let branch_dir = seg.project_point(nose_point.branch_edge).tangent;
    let main_dir = main.project_point(nose_point.track_edge).tangent;
    let bisector = (main_dir + branch_dir).normalize();
    let across = nose.segment.direction();
    let angle = bisector.dot(across).clamp(-1.0, 1.0).acos().to_degrees();
    assert!((angle - 90.0).abs() <= 2.0, "nose is {angle} degrees from the bisector");

    // Each divider wall stays at least 0.3 m from its road edge (0.02 m for the branch spline's own sampling).
    for &i in &seen {
        if i == nose_i || i == cap_i {
            continue;
        }
        let w = &walls[i];
        for p in [w.segment.start, w.segment.end] {
            let main_gap = main.project_point(p).distance_to_spline - TRACK_WIDTH * 0.5;
            let branch_proj = seg.project_point(p);
            let branch_gap = branch_proj.distance_to_spline - branch_proj.track_width * 0.5;
            assert!(
                main_gap >= 0.28 && branch_gap >= 0.28,
                "divider vertex {p:?}: {main_gap} m from the main edge, {branch_gap} m from the branch edge"
            );
        }
    }
}

#[test]
fn the_main_wall_on_the_branch_side_is_replaced_between_the_anchors() {
    let track = track_with(layout());
    // Right side of the main road is `outer_walls`. None of it stands between the anchors, on either side of the cut.
    let inside: Vec<_> = track
        .geometry
        .outer_walls
        .iter()
        .filter(|w| {
            let mid = w.segment.midpoint();
            mid.y.abs() < 20.0 && mid.x > 100.5 && mid.x < 299.5
        })
        .collect();
    assert!(inside.is_empty(), "{} main wall pieces remain between the anchors", inside.len());
    // The far side is untouched.
    assert!(track
        .geometry
        .inner_walls
        .iter()
        .any(|w| w.segment.midpoint().x > 150.0 && w.segment.midpoint().y > 8.0 && w.segment.midpoint().y < 12.0));
    // The cut ends exactly at the anchor normals.
    for x in [100.0f32, 300.0] {
        assert!(
            track.geometry.outer_walls.iter().any(|w| [w.segment.start, w.segment.end]
                .iter()
                .any(|p| (p.x - x).abs() < 1e-2 && p.y < -TRACK_WIDTH * 0.5)),
            "no main wall end at x = {x}"
        );
    }
}

#[test]
fn network_walls_are_rebuilt_on_load_and_never_saved() {
    let track = track_with(layout());
    let json = track.to_json().unwrap();
    assert!(!json.contains("network_walls"));
    assert!(json.contains("branch_layout"));
    let reloaded = Track::from_json(&json).unwrap();
    assert_eq!(reloaded.branch_layout, track.branch_layout);
    assert_eq!(reloaded.geometry.network_walls, track.geometry.network_walls);
    assert_eq!(reloaded.geometry.outer_walls, track.geometry.outer_walls);
    assert!(!reloaded.geometry.network_walls.is_empty());
}

// ---------------------------------------------------------------------------------------------------------------
// Surfaces
// ---------------------------------------------------------------------------------------------------------------

#[test]
fn junction_surfaces_follow_the_ribbons_not_forced_asphalt() {
    let mut waypoints = stadium_waypoints();
    for wp in &mut waypoints {
        wp.left_runoff_surface = Some(SurfaceType::Grass);
        wp.right_runoff_surface = Some(SurfaceType::Grass);
    }
    let mut track = Track {
        name: "Stadium".to_string(),
        spline: TrackSpline::new(waypoints, true),
        default_surface: SurfaceType::Grass,
        ..Track::default()
    };
    track.branch_layout = Some(layout());
    bake(
        &mut track,
        &BakeOptions { rebuild: true, barrier_offset: Some(G), barrier_type: Some(BarrierType::TireWall), ..BakeOptions::default() },
    )
    .unwrap();
    let compiled = layout().compile(&track).unwrap();

    // In the throat, inside the main ribbon, where the branch ribbon overlaps it: the main surface.
    assert_eq!(track.sample_surface(Vec2::new(112.0, -2.0)), SurfaceType::Asphalt);
    assert_eq!(track.sample_surface(Vec2::new(285.0, -2.0)), SurfaceType::Asphalt);
    // On the branch after the apex, outside the main ribbon: the branch surface.
    let branch = compiled.network.get_segment(SegmentId(2)).unwrap().samples.iter().find(|s| s.point.x > 150.0).unwrap().point;
    assert_eq!(track.sample_surface(branch), SurfaceType::Dirt, "branch at {branch:?}");
    // In the gore, between the main edge and the branch edge once they are 1.2 m apart: the run-off surface, not
    // asphalt (the main edge is y = -6, the branch edge is its left edge).
    let seg = compiled.network.get_segment(SegmentId(2)).unwrap();
    let gore_at = |range: std::ops::Range<f32>| {
        let s = seg
            .samples
            .iter()
            .filter(|s| range.contains(&s.point.x))
            .find(|s| {
                let edge = s.point + s.normal * (s.width * 0.5);
                (edge.y + TRACK_WIDTH * 0.5 + 1.2).abs() < 0.1
            })
            .expect("a branch sample with a 1.2 m gap");
        let edge = s.point + s.normal * (s.width * 0.5);
        Vec2::new(edge.x, -TRACK_WIDTH * 0.5 - 0.6)
    };
    let gore = gore_at(100.0..140.0);
    assert_eq!(track.sample_surface(gore), SurfaceType::Grass, "split gore at {gore:?}");
    let gore = gore_at(260.0..300.0);
    assert_eq!(track.sample_surface(gore), SurfaceType::Grass, "merge gore at {gore:?}");
}
