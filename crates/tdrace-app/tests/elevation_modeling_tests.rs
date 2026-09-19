use tdrace_app::module::f1::F1GameModule;
use tdrace_app::module::kart::KartGameModule;
use tdrace_core::car::{Car, CarControls};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::presets::kart_arena;
use glam::Vec2;

#[test]
fn test_crossover_bridge_detection_pfi_and_suzuka() {
    // 1. Suzuka: famous figure-8 crossover
    let suzuka_track = F1GameModule::track_suzuka();
    let suzuka_bridge_samples: Vec<_> = suzuka_track.spline.samples.iter().filter(|s| s.is_bridge).collect();
    assert!(
        !suzuka_bridge_samples.is_empty(),
        "Suzuka figure-8 crossover overpass must be detected as bridge"
    );
    for s in &suzuka_bridge_samples {
        assert!(s.elevation >= 1.2, "Suzuka bridge samples must have elevated clearance");
    }

    // Underpass on Suzuka must NOT be flagged as bridge
    let suzuka_non_bridge: Vec<_> = suzuka_track.spline.samples.iter().filter(|s| !s.is_bridge).collect();
    assert!(
        suzuka_non_bridge.iter().any(|s| s.elevation < 0.5),
        "Suzuka ground/underpass sections must have is_bridge = false"
    );

    // Wall barriers on Suzuka bridge should have is_bridge = true
    let suzuka_bridge_walls: Vec<_> = suzuka_track
        .geometry
        .all_walls()
        .filter(|w| w.is_bridge)
        .collect();
    assert!(
        !suzuka_bridge_walls.is_empty(),
        "Suzuka overpass must have bridge walls"
    );

    // 2. PFI: famous kart flyover crossover bridge
    let pfi = KartGameModule::track_pfi();
    let pfi_bridge_samples: Vec<_> = pfi.spline.samples.iter().filter(|s| s.is_bridge).collect();
    assert!(
        !pfi_bridge_samples.is_empty(),
        "PFI flyover crossover bridge must be detected as bridge"
    );
    for s in &pfi_bridge_samples {
        assert!(s.elevation >= 1.2, "PFI bridge samples must have elevated clearance");
    }
}

#[test]
fn test_natural_elevation_tracks_have_zero_bridges() {
    // Test natural mountain / elevation tracks: Spa, Nurburgring GP, Bathurst (Mount Panorama)
    let nurburgring = F1GameModule::track_nurburgring_gp();
    assert!(
        nurburgring.spline.samples.iter().any(|s| s.elevation.abs() > 0.5),
        "Nurburgring GP must have real elevation profile"
    );
    assert!(
        !nurburgring.spline.samples.iter().any(|s| s.is_bridge),
        "Nurburgring GP is natural terrain and must have zero bridge samples"
    );
    assert!(
        !nurburgring.geometry.all_walls().any(|w| w.is_bridge),
        "Nurburgring GP must have zero bridge wall barriers"
    );

    // Check grade slope and vertical curvature exist along natural terrain
    let has_slopes = nurburgring.spline.samples.iter().any(|s| s.grade_slope.abs() > 0.005);
    assert!(has_slopes, "Nurburgring GP must calculate non-zero longitudinal grade slopes");

    let has_curvatures = nurburgring.spline.samples.iter().any(|s| s.vertical_curvature.abs() > 1e-4);
    assert!(has_curvatures, "Nurburgring GP must calculate non-zero vertical road curvatures");

    // Mount Panorama (Bathurst): famous mountain circuit
    let bathurst = F1GameModule::track_bathurst();
    let max_bathurst_elev = bathurst.spline.samples.iter().map(|s| s.elevation).fold(0.0f32, f32::max);
    assert!(
        max_bathurst_elev > 4.0,
        "Bathurst must have massive natural elevation (got {:.1}m)",
        max_bathurst_elev
    );
    assert!(
        !bathurst.spline.samples.iter().any(|s| s.is_bridge),
        "Bathurst Mount Panorama has no crossover bridges - all elevation is natural ground!"
    );
    assert!(
        !bathurst.geometry.all_walls().any(|w| w.is_bridge),
        "Bathurst barriers must be natural ground barriers without viaduct drop shadows"
    );

    // Kart arena has no bridges
    let arena = kart_arena();
    assert!(!arena.spline.samples.iter().any(|s| s.is_bridge));
}

#[test]
fn test_3d_physics_grade_slope_and_crest_dynamics() {
    let mut car_flat = Car::new(CarConfig::sports_car());
    let mut car_uphill = Car::new(CarConfig::sports_car());

    // Set uphill incline of 0.10 rad (~5.7 deg) along forward direction
    car_uphill.state.road_grade_slope = 0.10;
    car_uphill.state.track_forward = Vec2::new(1.0, 0.0);

    let ctrl = CarControls::new(1.0, 0.0, 0.0, false);
    let dt = 1.0 / 60.0;

    for _ in 0..60 {
        car_flat.step(&ctrl, SurfaceType::Asphalt, dt);
        car_uphill.step(&ctrl, SurfaceType::Asphalt, dt);
    }

    // Uphill car experiences grade resistance force: F = -m g sin(theta)
    assert!(
        car_flat.state.speed > car_uphill.state.speed + 0.5,
        "Flat car must accelerate faster than uphill car"
    );

    // Uphill car transfers weight to rear axle (grade pitch)
    let front_load_uphill: f32 = car_uphill.state.wheels[0].normal_load + car_uphill.state.wheels[1].normal_load;
    let rear_load_uphill: f32 = car_uphill.state.wheels[2].normal_load + car_uphill.state.wheels[3].normal_load;
    assert!(
        rear_load_uphill > front_load_uphill,
        "Uphill acceleration must transfer weight to rear axle"
    );
}
