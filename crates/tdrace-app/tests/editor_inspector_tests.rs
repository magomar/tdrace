//! Track Studio inspector tests for specs/086_compact_track_studio_inspector_with_unified_single_and_multiselection_editing.md.

use glam::Vec2;
use tdrace_app::editor::state::dev_selection;
use tdrace_app::editor::Selection;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{BarrierType, JumpRamp, LineSegment, Obstacle, PitBox, PitLane, SurfaceShape, SurfaceZone};
use tdrace_core::track::spline::TrackSpline;
use tdrace_core::track::Track;

const FIXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor_inspector_fixture.json");

/// Builds the inspector fixture: a rounded 240 m x 140 m loop with every editor entity kind.
fn build_fixture() -> Track {
    let points = [
        Vec2::new(-100.0, -70.0),
        Vec2::new(0.0, -70.0),
        Vec2::new(100.0, -70.0),
        Vec2::new(120.0, -50.0),
        Vec2::new(120.0, 50.0),
        Vec2::new(100.0, 70.0),
        Vec2::new(0.0, 70.0),
        Vec2::new(-100.0, 70.0),
        Vec2::new(-120.0, 50.0),
        Vec2::new(-120.0, -50.0),
    ];
    let mut track = Track {
        name: "Editor Inspector Fixture".to_string(),
        spline: TrackSpline::from_points(&points, 12.0, true),
        ..Track::default()
    };
    track.rebuild_geometry(8.0, BarrierType::Steel);

    let g = &mut track.geometry;
    for (i, (x, surface)) in [(-40.0, SurfaceType::Gravel), (40.0, SurfaceType::Water)].into_iter().enumerate() {
        let shape = SurfaceShape::OrientedBox { center: Vec2::new(x, 20.0), half_extents: Vec2::new(10.0, 6.0), angle: 0.0 };
        g.surface_zones.push(SurfaceZone::new(shape, surface, format!("Fixture Zone {}", i + 1)));
    }
    for i in 0..3 {
        let center = Vec2::new(-20.0 + 20.0 * i as f32, -20.0);
        g.obstacles.push(Obstacle::oriented_box(i + 1, center, Vec2::new(2.0, 2.0), 0.0, format!("Fixture Box {}", i + 1)));
    }
    for (i, x) in [-50.0, 50.0].into_iter().enumerate() {
        let shape = SurfaceShape::OrientedBox { center: Vec2::new(x, 70.0), half_extents: Vec2::new(6.0, 4.0), angle: 0.0 };
        g.jump_ramps.push(JumpRamp::new(i + 1, shape, Vec2::NEG_X, 4.0, 15.0, 1.8, format!("Fixture Ramp {}", i + 1)));
    }

    let pit_points = [Vec2::new(-80.0, -55.0), Vec2::new(0.0, -55.0), Vec2::new(80.0, -55.0)];
    track.pit_lane = Some(PitLane::new(
        TrackSpline::from_points(&pit_points, 6.0, false),
        6.0,
        PitLane::DEFAULT_ROAD_SPEED_LIMIT,
        vec![PitBox::new(Vec2::new(0.0, -55.0), Vec2::X, 3.0, 0.0)],
        LineSegment::new(Vec2::new(-80.0, -58.0), Vec2::new(-80.0, -52.0)),
        LineSegment::new(Vec2::new(80.0, -58.0), Vec2::new(80.0, -52.0)),
    ));
    track.auto_generate_checkpoints(8, 3);
    track.auto_generate_grid_default();
    track
}

/// Regenerates the fixture file: `cargo test -p tdrace-app --test editor_inspector_tests -- --ignored`.
#[test]
#[ignore]
fn regenerate_editor_inspector_fixture() {
    let json = build_fixture().to_json().expect("fixture serializes");
    std::fs::write(FIXTURE_PATH, json).expect("fixture written");
}

#[test]
fn fixture_has_every_entity_kind() {
    let track = Track::load_from_file(FIXTURE_PATH).expect("fixture loads");
    assert!(track.spline.waypoints.len() >= 3);
    assert_eq!(track.geometry.surface_zones.len(), 2);
    assert_eq!(track.geometry.obstacles.len(), 3);
    assert_eq!(track.geometry.jump_ramps.len(), 2);
    assert!(!track.checkpoints.is_empty());
    assert!(!track.grid_positions.is_empty());
    assert!(track.pit_lane.is_some());
}

#[test]
fn dev_selection_picks_each_kind_from_the_fixture() {
    let track = Track::load_from_file(FIXTURE_PATH).expect("fixture loads");
    assert_eq!(dev_selection("waypoint", &track), Selection::Waypoint(0));
    assert_eq!(dev_selection("waypoints", &track), Selection::MultipleWaypoints(vec![0, 1, 2]));
    assert_eq!(dev_selection("zone", &track), Selection::SurfaceZone(0));
    assert_eq!(dev_selection("obstacle", &track), Selection::Obstacle(0));
    assert_eq!(dev_selection("ramp", &track), Selection::JumpRamp(0));
    assert_eq!(dev_selection("checkpoint", &track), Selection::Checkpoint(0));
    assert_eq!(dev_selection("grid", &track), Selection::GridSlot(0));
    assert_eq!(dev_selection("pit", &track), Selection::PitBox);
    assert_eq!(dev_selection("none", &track), Selection::None);
    assert_eq!(dev_selection("zones", &track).selected_surface_zone_indices(), vec![0, 1]);
    assert_eq!(dev_selection("obstacles", &track).selected_obstacle_indices(), vec![0, 1, 2]);
    assert_eq!(dev_selection("ramps", &track).selected_jump_ramp_indices(), vec![0, 1]);
    let mixed = dev_selection("mixed", &track);
    assert_eq!(mixed.selected_waypoint_indices(), vec![0, 1]);
    assert_eq!(mixed.selected_surface_zone_indices(), vec![0]);
    assert_eq!(mixed.selected_jump_ramp_indices(), vec![0]);
}

#[test]
fn dev_selection_drops_missing_indices() {
    let track = Track::default();
    assert_eq!(dev_selection("ramps", &track), Selection::None);
    assert_eq!(dev_selection("pit", &track), Selection::None);
    assert_eq!(dev_selection("unknown-kind", &track), Selection::None);
}
