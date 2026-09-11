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

#[test]
fn test_multi_level_zoom_cycling() {
    let mut camera = RaceCamera::new();
    assert_eq!(camera.levels.len(), 4);
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(camera.current_zoom_level().name, "Close");
    assert_eq!(camera.mode, CameraMode::SmoothFollow);

    // Intermediate 1: Medium
    let lvl1_name = camera.cycle_zoom_level().name.clone();
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(lvl1_name, "Medium");
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 10.0);
    assert_eq!(camera.max_zoom_scale, 16.5);

    // Intermediate 2: Far
    let lvl2_name = camera.cycle_zoom_level().name.clone();
    assert_eq!(camera.current_level_idx, 2);
    assert_eq!(lvl2_name, "Far");
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 7.0);
    assert_eq!(camera.max_zoom_scale, 11.5);

    // Level 3: Very Far (follow mode, not static overview)
    let lvl3_name = camera.cycle_zoom_level().name.clone();
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(lvl3_name, "Very Far");
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 5.0);
    assert_eq!(camera.max_zoom_scale, 8.0);

    // Wraparound to Close
    let lvl0_name = camera.cycle_zoom_level().name.clone();
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(lvl0_name, "Close");
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 13.5);
    assert_eq!(camera.max_zoom_scale, 22.0);
}

#[test]
fn test_camera_zoom_in_and_zoom_out() {
    let mut camera = RaceCamera::new();
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(camera.current_zoom_level().name, "Close");

    // At closest zoom (index 0), zoom_in() returns None and stays at index 0
    assert!(camera.zoom_in().is_none());
    assert_eq!(camera.current_level_idx, 0);

    // Zoom out step-by-step: 0 (Close) -> 1 (Medium) -> 2 (Far) -> 3 (Very Far)
    let lvl1 = camera.zoom_out().expect("Should zoom out to Medium");
    assert_eq!(lvl1.name, "Medium");
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 10.0);
    assert_eq!(camera.max_zoom_scale, 16.5);

    let lvl2 = camera.zoom_out().expect("Should zoom out to Far");
    assert_eq!(lvl2.name, "Far");
    assert_eq!(camera.current_level_idx, 2);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 7.0);
    assert_eq!(camera.max_zoom_scale, 11.5);

    let lvl3 = camera.zoom_out().expect("Should zoom out to Very Far");
    assert_eq!(lvl3.name, "Very Far");
    assert_eq!(camera.current_level_idx, 3);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 5.0);
    assert_eq!(camera.max_zoom_scale, 8.0);

    // At farthest zoom (index 3), zoom_out() returns None and stays at index 3
    assert!(camera.zoom_out().is_none());
    assert_eq!(camera.current_level_idx, 3);

    // Zoom in step-by-step: 3 (Very Far) -> 2 (Far) -> 1 (Medium) -> 0 (Close)
    let in2 = camera.zoom_in().expect("Should zoom in to Far");
    assert_eq!(in2.name, "Far");
    assert_eq!(camera.current_level_idx, 2);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);

    let in1 = camera.zoom_in().expect("Should zoom in to Medium");
    assert_eq!(in1.name, "Medium");
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);

    let in0 = camera.zoom_in().expect("Should zoom in to Close");
    assert_eq!(in0.name, "Close");
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.min_zoom_scale, 13.5);
    assert_eq!(camera.max_zoom_scale, 22.0);

    // Bounded again at closest
    assert!(camera.zoom_in().is_none());
    assert_eq!(camera.current_level_idx, 0);
}

#[test]
fn test_camera_skip_overview_on_tab_cycle() {
    use tdrace_app::config::ZoomLevelConfig;
    let mut camera = RaceCamera::new();
    // Inject custom levels where one is an Overview
    camera.levels = vec![
        ZoomLevelConfig {
            name: "Near".to_string(),
            mode: "follow".to_string(),
            min_zoom: 12.0,
            max_zoom: 20.0,
        },
        ZoomLevelConfig {
            name: "Overview".to_string(),
            mode: "overview".to_string(),
            min_zoom: 3.5,
            max_zoom: 3.5,
        },
        ZoomLevelConfig {
            name: "Distant".to_string(),
            mode: "follow".to_string(),
            min_zoom: 6.0,
            max_zoom: 10.0,
        },
    ];
    camera.current_level_idx = 0;

    // Cycling from Near (0) should SKIP Overview (1) and jump to Distant (2)
    let next = camera.cycle_zoom_level();
    assert_eq!(next.name, "Distant");
    assert_eq!(camera.current_level_idx, 2);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);

    // Cycling from Distant (2) should wraparound to Near (0)
    let next2 = camera.cycle_zoom_level();
    assert_eq!(next2.name, "Near");
    assert_eq!(camera.current_level_idx, 0);
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
}

#[test]
fn test_camera_paused_overview_and_resume() {
    let mut camera = RaceCamera::new();
    let track = classic_grand_prix();
    camera.setup_for_track(&track);
    camera.set_zoom_level(1); // Medium

    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.current_level_idx, 1);

    // Pause race -> should activate overview
    camera.set_paused_overview();
    assert_eq!(camera.mode, CameraMode::StaticOverview);
    assert_eq!(camera.current_pos, camera.overview_center);
    assert_eq!(camera.current_zoom, camera.overview_zoom);
    assert_eq!(camera.paused_from_follow, Some(1));

    // Resume race -> should restore follow camera and car positioning
    let mut car = Car::new(CarConfig::sports_car()).with_pose(Vec2::new(100.0, 50.0), 0.0);
    car.state.speed = 20.0;
    car.state.velocity = Vec2::new(20.0, 0.0);

    camera.resume_from_pause(Some(&car));
    assert_eq!(camera.mode, CameraMode::SmoothFollow);
    assert_eq!(camera.current_level_idx, 1);
    assert_eq!(camera.paused_from_follow, None);
    // Camera snapped directly to car + lookahead (100.0 + 20.0 * 0.40 = 108.0)
    assert!((camera.current_pos.x - 108.0).abs() < 1e-3);
    assert!((camera.current_pos.y - 50.0).abs() < 1e-3);
}

#[test]
fn test_camera_progressive_zoom_in_and_out() {
    let mut camera = RaceCamera::new();
    let initial_min = camera.min_zoom_scale;
    let initial_max = camera.max_zoom_scale;

    // Continuous Zoom In (+1.0 dir) for 0.5s at 1.0x speed multiplier
    camera.zoom_progressive(1.0, 1.0, 0.5);
    assert!(camera.min_zoom_scale > initial_min);
    assert!(camera.max_zoom_scale > initial_max);
    let zoomed_in_max = camera.max_zoom_scale;

    // Continuous Zoom Out (-1.0 dir) for 0.5s at 1.0x speed multiplier
    camera.zoom_progressive(-1.0, 1.0, 0.5);
    assert!(camera.max_zoom_scale < zoomed_in_max);
    assert!((camera.max_zoom_scale - initial_max).abs() < 0.1);

    // Fast zoom with 2.0x Shift speed multiplier
    camera.zoom_progressive(1.0, 2.0, 0.5);
    assert!(camera.max_zoom_scale > zoomed_in_max);

    // Progressive zoom clamping bounds
    camera.zoom_progressive(-1.0, 10.0, 10.0);
    assert_eq!(camera.min_zoom_scale, 1.0);
    assert_eq!(camera.max_zoom_scale, 1.0);

    camera.zoom_progressive(1.0, 10.0, 10.0);
    assert_eq!(camera.min_zoom_scale, 40.0);
    assert_eq!(camera.max_zoom_scale, 50.0);

    // Overview mode continuous progressive zoom
    camera.toggle_mode();
    assert_eq!(camera.mode, CameraMode::StaticOverview);
    let initial_overview = camera.overview_zoom;
    camera.zoom_progressive(1.0, 1.0, 0.5);
    assert!(camera.overview_zoom > initial_overview);
    camera.zoom_progressive(-1.0, 1.0, 0.5);
    assert!((camera.overview_zoom - initial_overview).abs() < 0.1);
}

#[test]
fn test_camera_cabinet_screen_shake_integration() {
    use tdrace_app::camera::ScreenShake;

    let mut camera = RaceCamera::new();
    // Test ScreenShake re-export
    let mut standalone = ScreenShake::new(20.0, 2.0);
    standalone.add_trauma(0.5);
    assert_eq!(standalone.trauma, 0.5);

    // Test RaceCamera embedded ScreenShake
    camera.add_trauma(0.75);
    assert_eq!(camera.shake.trauma, 0.75);
    assert_eq!(camera.trauma, 0.75);

    let (offset, _angle) = camera.shake.sample_shake();
    assert!(offset.length() > 0.0);

    let car = Car::new(CarConfig::sports_car());
    camera.update(&car, 0.1);
    assert_eq!(camera.trauma, camera.shake.trauma);
    assert!(camera.trauma < 0.75);

    let (_offset2, angle2) = camera.shake.sample_shake();
    assert!(angle2.abs() > 0.0);
}

