use glam::Vec2;
use tdrace_app::camera::{CameraMode, RaceCamera};
use tdrace_core::{Car, CarConfig};
use tdrace_core::track::presets::{classic_grand_prix, oval_speedway};

#[test]
fn test_camera_modes_and_toggle() {
    let mut camera = RaceCamera::new();
    assert_eq!(camera.mode, CameraMode::SmoothFollow);

    camera.toggle_mode();
    assert_eq!(camera.mode, CameraMode::StaticOverview);

    camera.toggle_mode();
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
}

#[test]
fn test_camera_setup_for_all_presets() {
    let mut camera = RaceCamera::new();
    let gp = classic_grand_prix();
    camera.setup_for_track(&gp);

    assert!(camera.overview_zoom > 0.0);
    assert_ne!(camera.overview_center, Vec2::ZERO);

    let oval = oval_speedway();
    camera.setup_for_track(&oval);
    assert!(camera.overview_zoom > 0.0);
}

#[test]
fn test_camera_smooth_follow_and_speed_zoom() {
    let mut camera = RaceCamera::new();
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(100.0, 50.0), 0.0);
    car.state.speed = 40.0;
    car.state.velocity = Vec2::new(40.0, 0.0);

    // Initial position
    camera.current_pos = Vec2::ZERO;

    for _ in 0..60 {
        camera.update(&car, 0.016);
    }

    // Camera should smoothly move towards car + lookahead (100.0 + 40.0 * 0.4 = 116.0)
    assert!(camera.current_pos.x > 80.0);
    assert!(camera.current_pos.y > 35.0);

    // Speed zoom should zoom out at 40 m/s
    assert!(camera.current_zoom < camera.max_zoom_scale);
}

#[test]
fn test_camera_screen_shake_and_decay() {
    let mut camera = RaceCamera::new();
    assert_eq!(camera.trauma, 0.0);

    camera.add_trauma(0.8);
    assert!((camera.trauma - 0.8).abs() < 1e-4);

    let car = Car::new(CarConfig::sports_car());
    camera.update(&car, 0.2);
    assert!(camera.trauma < 0.8);
    assert!(camera.trauma > 0.0);

    camera.update(&car, 1.0);
    assert_eq!(camera.trauma, 0.0);
}

#[test]
fn test_camera_coordinate_conversions() {
    let mut camera = RaceCamera::new();
    camera.current_pos = Vec2::new(50.0, 30.0);
    camera.current_zoom = 20.0;

    let world_pt = Vec2::new(55.0, 35.0);
    let screen_pt = camera.world_to_screen(world_pt);
    let roundtrip_world = camera.screen_to_world(screen_pt);

    assert!((world_pt.x - roundtrip_world.x).abs() < 1e-3);
    assert!((world_pt.y - roundtrip_world.y).abs() < 1e-3);
}
