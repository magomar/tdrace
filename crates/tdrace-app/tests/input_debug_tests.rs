use tdrace_app::input::{DebugOverlays, InputController};
use tdrace_core::{Car, CarConfig};

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
    let track = tdrace_core::catalog::official_track("classic", "classic_grand_prix");
    let car = Car::new(CarConfig::sports_car());
    let opponents = vec![Car::new(CarConfig::sports_car())];

    let hits = controller.lidar_scanner.scan(&car, &track, &opponents);
    assert_eq!(hits.len(), 32); // 32 beams by default
}

/// Scenario: Headless test runner executes input polling without panic unwinding (Spec 084)
#[test]
fn test_headless_input_polling_without_panic() {
    use macroquad::input::KeyCode;
    use tdrace_app::input::is_key_down;

    // In headless test environments without an active Macroquad context,
    // input polling must return safe defaults (false) without panicking.
    assert!(!is_key_down(KeyCode::Left));
    assert!(!is_key_down(KeyCode::Right));
    assert!(!is_key_down(KeyCode::Up));
    assert!(!is_key_down(KeyCode::Down));
}

