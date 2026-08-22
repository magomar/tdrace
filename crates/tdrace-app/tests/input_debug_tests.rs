use tdrace_app::input::{DebugOverlays, InputController};
use tdrace_core::{Car, CarConfig};
use tdrace_core::track::presets::classic_grand_prix;

#[test]
fn test_debug_overlays_default_and_flags() {
    let mut overlays = DebugOverlays::default();
    assert!(!overlays.lidar);
    assert!(!overlays.checkpoints);
    assert!(!overlays.collision_obb);
    assert!(!overlays.ai_paths);
    assert!(!overlays.telemetry);

    overlays.lidar = true;
    overlays.telemetry = true;
    assert!(overlays.lidar);
    assert!(overlays.telemetry);
}

#[test]
fn test_input_controller_lidar_scan() {
    let controller = InputController::new();
    let track = classic_grand_prix();
    let car = Car::new(CarConfig::sports_car());
    let opponents = vec![Car::new(CarConfig::sports_car())];

    let hits = controller.lidar_scanner.scan(&car, &track, &opponents);
    assert_eq!(hits.len(), 32); // 32 beams by default
}
