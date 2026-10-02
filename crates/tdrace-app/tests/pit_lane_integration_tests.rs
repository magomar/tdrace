//! Pit lane integration tests: Speed limiter enforcement, governor clamping, and HUD integration (Spec 062).

use glam::Vec2;
use race_kit::{DriveControls, RaceEvent, RaceRules, RaceWorld};
use tdrace_core::track::{Checkpoint, LineSegment, PitBox, PitLane, TrackSpline};
use tdrace_core::{Body2D, Car, CarConfig};

#[test]
fn test_pit_limiter_speed_clamping() {
    let mut track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");

    // Define pit road spline, stalls, and entry/exit gates
    let entry_gate = LineSegment::new(Vec2::new(10.0, -10.0), Vec2::new(10.0, 10.0));
    let exit_gate = LineSegment::new(Vec2::new(100.0, -10.0), Vec2::new(100.0, 10.0));
    let pit_spline = TrackSpline::from_points(
        &[Vec2::new(10.0, 0.0), Vec2::new(55.0, 0.0), Vec2::new(100.0, 0.0)],
        6.0,
        false,
    );
    let pit_box = PitBox::new(Vec2::new(50.0, 0.0), Vec2::new(1.0, 0.0), 3.0, 0.0);

    let speed_limit = PitLane::DEFAULT_ROAD_SPEED_LIMIT; // 16.67 m/s = 60 km/h
    track.pit_lane = Some(PitLane::new(
        pit_spline,
        6.0,
        speed_limit,
        vec![pit_box],
        entry_gate,
        exit_gate,
    ));

    // Register checkpoints with pit entry/exit flags
    track.checkpoints = vec![
        Checkpoint::new(0, entry_gate, Vec2::new(1.0, 0.0), 0, false).with_pit_flags(true, false),
        Checkpoint::new(1, exit_gate, Vec2::new(1.0, 0.0), 0, false).with_pit_flags(false, true),
    ];

    let mut world: RaceWorld<Car> = RaceWorld::new(RaceRules::default());
    let mut car = Car::new(CarConfig::sports_car());

    // Scenario: Vehicle racing at ~150 km/h (41.67 m/s) on the main straight approaching pit entry
    car.state.position = Vec2::new(5.0, 0.0);
    car.state.velocity = Vec2::new(41.67, 0.0);
    car.state.speed = 41.67;

    let tracker = tdrace_core::track::checkpoint::TrackProgressTracker::new(track.checkpoints.len(), 1);
    world.spawn(car, tracker);

    // Initial state: not pitting, limiter inactive
    assert!(!world.pit_states[0].is_limiter_active());

    // Step 1: Vehicle crosses pit entry gate at high speed with full throttle
    let full_throttle = vec![DriveControls::new(1.0, 0.0, 0.0, false)];
    let events = world.step(&track, &full_throttle, 0.2).to_vec();

    // Verify PitEntry event fired
    assert!(
        events.iter().any(|ev| matches!(ev, RaceEvent::PitEntry { car: 0 })),
        "Vehicle crossing is_pit_entry gate must trigger PitEntry event"
    );

    // Speed limiter must now be authoritatively active
    assert!(
        world.pit_states[0].is_limiter_active(),
        "Pit limiter must be engaged inside the pit lane"
    );

    // Speed must be clamped to <= 60 km/h (16.67 m/s)
    let speed_after_entry = world.vehicles[0].speed();
    assert!(
        speed_after_entry <= speed_limit + 0.05,
        "Vehicle speed ({:.2} m/s) must be clamped to pit speed limit ({:.2} m/s)",
        speed_after_entry,
        speed_limit
    );

    // Step 2: Continuous driving with 100% throttle inside pit lane
    for _ in 0..10 {
        world.step(&track, &full_throttle, 0.1);
        let current_speed = world.vehicles[0].speed();
        assert!(
            current_speed <= speed_limit + 0.05,
            "Speed limiter override prevention failed: speed reached {:.2} m/s under full throttle",
            current_speed
        );
    }

    // Step 3: Advance past pit exit gate
    world.vehicles[0].state.position = Vec2::new(98.0, 0.0);
    let exit_events = world.step(&track, &full_throttle, 0.2).to_vec();

    assert!(
        exit_events.iter().any(|ev| matches!(ev, RaceEvent::PitExit { car: 0 })),
        "Crossing pit exit gate must trigger PitExit event"
    );
    assert!(
        !world.pit_states[0].is_limiter_active(),
        "Pit limiter must disengage after crossing pit exit gate"
    );

    // Step 4: With limiter released, full throttle accelerates beyond 60 km/h
    for _ in 0..15 {
        world.step(&track, &full_throttle, 0.1);
    }
    assert!(
        world.vehicles[0].speed() > speed_limit + 2.0,
        "Vehicle must regain full acceleration authority after exiting pit lane"
    );
}
