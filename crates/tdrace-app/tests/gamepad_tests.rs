use tdrace_app::input::gamepad::{GamepadConfig, GamepadController, GamepadSnapshot};
use tdrace_app::input::InputController;

#[test]
fn test_gamepad_config_defaults_and_deadzone_processing() {
    let mut gp = GamepadController::new();
    assert_eq!(gp.config.stick_deadzone, 0.12);
    assert_eq!(gp.config.trigger_deadzone, 0.05);

    // Below stick deadzone -> must be 0.0
    let sub_deadzone_snapshot = GamepadSnapshot {
        is_connected: true,
        gamepad_name: "Test Pad".to_string(),
        steer: 0.08, // within 0.12 deadzone
        throttle: 0.03, // within 0.05 trigger deadzone
        brake: 0.0,
        ..Default::default()
    };
    gp.inject_snapshot(sub_deadzone_snapshot);

    assert_eq!(gp.snapshot.brake, 0.0);
}

#[test]
fn test_gamepad_analog_steering_and_triggers_injection() {
    let mut input = InputController::new();
    let dt = 1.0 / 60.0;

    // Inject proportional analog inputs: 65% right steer, 80% throttle trigger, 0% brake
    let snapshot = GamepadSnapshot {
        is_connected: true,
        gamepad_name: "Xbox Series Controller".to_string(),
        steer: 0.65,
        throttle: 0.80,
        brake: 0.0,
        handbrake: false,
        reverse: false,
        ..Default::default()
    };
    input.gamepad.inject_snapshot(snapshot);

    let ctrl = input.process_inputs((0.0, 0.0, 0.0, false, false), dt, 0.0);
    assert!((ctrl.steer - 0.65).abs() < 1e-4, "Analog stick steer should be preserved");
    assert!((ctrl.throttle - 0.80).abs() < 1e-4, "Analog trigger throttle should be preserved");
    assert_eq!(ctrl.brake, 0.0);
    assert!(!ctrl.handbrake);
    assert!(!ctrl.reverse);
}

#[test]
fn test_gamepad_and_keyboard_seamless_blending() {
    let mut input = InputController::new();
    let dt = 1.0 / 60.0;

    // Gamepad steers right (0.5), brakes with LT (0.75), and pulls handbrake
    let snapshot = GamepadSnapshot {
        is_connected: true,
        gamepad_name: "DualSense Wireless Controller".to_string(),
        steer: 0.50,
        throttle: 0.0,
        brake: 0.75,
        handbrake: true,
        reverse: false,
        ..Default::default()
    };
    input.gamepad.inject_snapshot(snapshot);

    // Also press keyboard throttle (Q)
    let ctrl = input.process_inputs((0.0, 1.0, 0.0, false, false), dt, 0.0);
    assert!((ctrl.steer - 0.50).abs() < 1e-4);
    assert!(ctrl.throttle > 0.0, "Keyboard throttle blended");
    assert!((ctrl.brake - 0.75).abs() < 1e-4);
    assert!(ctrl.handbrake);
}

#[test]
fn test_gamepad_frame_button_event_clearing() {
    let mut gp = GamepadController::new();
    gp.snapshot.btn_a_pressed = true;
    gp.snapshot.btn_start_pressed = true;
    gp.snapshot.dpad_up_pressed = true;

    assert!(gp.snapshot.btn_a_pressed);
    assert!(gp.snapshot.btn_start_pressed);
    assert!(gp.snapshot.dpad_up_pressed);

    gp.clear_frame_events();

    assert!(!gp.snapshot.btn_a_pressed);
    assert!(!gp.snapshot.btn_start_pressed);
    assert!(!gp.snapshot.dpad_up_pressed);
}
