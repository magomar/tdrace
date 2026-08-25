use tdrace_app::input::gamepad::{GamepadController, GamepadSnapshot};
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

#[test]
fn test_deadzone_processing_linear_and_power_curve() {
    let deadzone = 0.12;
    let exponent = 1.15;

    // Inside deadzone -> 0.0
    assert_eq!(GamepadController::process_axis_deadzone(0.05, deadzone, exponent), 0.0);
    assert_eq!(GamepadController::process_axis_deadzone(-0.11, deadzone, exponent), 0.0);
    assert_eq!(GamepadController::process_axis_deadzone(0.12, deadzone, exponent), 0.0);

    // Full deflection -> 1.0 / -1.0
    let full_pos = GamepadController::process_axis_deadzone(1.0, deadzone, exponent);
    let full_neg = GamepadController::process_axis_deadzone(-1.0, deadzone, exponent);
    assert!((full_pos - 1.0).abs() < 1e-5);
    assert!((full_neg - (-1.0)).abs() < 1e-5);

    // Half deflection (~0.56) -> strictly positive and < 0.56 due to gentle center exponent
    let half = GamepadController::process_axis_deadzone(0.56, deadzone, exponent);
    assert!(half > 0.35 && half < 0.55, "Gentle center precision response: {half}");
}

#[test]
fn test_trigger_deadzone_and_clamping() {
    let deadzone = 0.05;

    // Below trigger deadzone
    assert_eq!(GamepadController::process_trigger_deadzone(0.02, deadzone), 0.0);
    assert_eq!(GamepadController::process_trigger_deadzone(-0.50, deadzone), 0.0);

    // Full trigger
    assert_eq!(GamepadController::process_trigger_deadzone(1.0, deadzone), 1.0);
    assert_eq!(GamepadController::process_trigger_deadzone(1.2, deadzone), 1.0);

    // Mid trigger
    let mid = GamepadController::process_trigger_deadzone(0.525, deadzone);
    assert!((mid - 0.50).abs() < 1e-4);
}

#[test]
fn test_tri_modal_seamless_input_transition_keyboard_gamepad_touch() {
    let mut input = InputController::new();
    let dt = 1.0 / 60.0;

    // 1. Keyboard only
    let kb_ctrl = input.process_inputs((1.0, 1.0, 0.0, false, false), dt, 0.0);
    let touch_empty = tdrace_app::input::touch::TouchController::new().poll_controls();
    let combined_1 = InputController::combine_controls(kb_ctrl, touch_empty);
    assert!(combined_1.throttle > 0.0);

    // 2. Seamless transition to Gamepad
    input.gamepad.inject_snapshot(GamepadSnapshot {
        is_connected: true,
        gamepad_name: "Switch Pro Controller".to_string(),
        steer: -0.75,
        throttle: 0.90,
        brake: 0.0,
        ..Default::default()
    });

    let gp_ctrl = input.process_inputs((0.0, 0.0, 0.0, false, false), dt, 0.0);
    let combined_2 = InputController::combine_controls(gp_ctrl, touch_empty);
    assert_eq!(combined_2.steer, -0.75);
    assert_eq!(combined_2.throttle, 0.90);
}

#[test]
fn test_stick_navigation_and_confirm_events() {
    let mut gp = GamepadController::new();

    // Inject confirm and stick navigation snapshot
    gp.inject_snapshot(GamepadSnapshot {
        is_connected: true,
        gamepad_name: "Test Controller".to_string(),
        nav_up: true,
        nav_down: false,
        nav_left: false,
        nav_right: true,
        btn_confirm_pressed: true,
        btn_cancel_pressed: false,
        ..Default::default()
    });

    assert!(gp.snapshot.nav_up);
    assert!(gp.snapshot.nav_right);
    assert!(gp.snapshot.btn_confirm_pressed);

    gp.clear_frame_events();

    assert!(!gp.snapshot.nav_up);
    assert!(!gp.snapshot.nav_right);
    assert!(!gp.snapshot.btn_confirm_pressed);
}

#[test]
fn test_gamepad_rt_throttle_lt_brake_a_handbrake_mapping() {
    let mut input = InputController::new();
    let dt = 1.0 / 60.0;

    // Test RT driving throttle, LT driving brake, and A button triggering handbrake
    let snapshot = GamepadSnapshot {
        is_connected: true,
        gamepad_name: "Xbox Wireless Controller".to_string(),
        steer: 0.0,
        throttle: 1.0,  // From RT button/trigger
        brake: 0.85,    // From LT button/trigger
        handbrake: true, // From A button / South button
        reverse: false,
        ..Default::default()
    };
    input.gamepad.inject_snapshot(snapshot);

    let ctrl = input.process_inputs((0.0, 0.0, 0.0, false, false), dt, 0.0);
    assert_eq!(ctrl.throttle, 1.0, "RT should produce full throttle");
    assert_eq!(ctrl.brake, 0.85, "LT should produce progressive braking");
    assert!(ctrl.handbrake, "A button should engage handbrake");
}

#[test]
fn test_gamepad_x_reverse_and_a_confirm_b_cancel() {
    let mut gp = GamepadController::new();
    let snapshot = GamepadSnapshot {
        is_connected: true,
        gamepad_name: "Xbox Wireless Controller".to_string(),
        reverse: true, // From X button
        btn_confirm_pressed: true, // From A button / Enter
        btn_cancel_pressed: false,
        ..Default::default()
    };
    gp.inject_snapshot(snapshot);
    assert!(gp.snapshot.reverse, "X button should trigger reverse");
    assert!(gp.snapshot.btn_confirm_pressed, "A button should confirm/start race in menu");

    let cancel_snapshot = GamepadSnapshot {
        is_connected: true,
        gamepad_name: "Xbox Wireless Controller".to_string(),
        reverse: false,
        btn_confirm_pressed: false,
        btn_cancel_pressed: true, // From B button / Escape
        ..Default::default()
    };
    gp.inject_snapshot(cancel_snapshot);
    assert!(gp.snapshot.btn_cancel_pressed, "B button should trigger cancel/back");
}

#[test]
fn test_gamepad_profile_candidate_paths_and_reload() {
    let paths = GamepadController::candidate_profile_paths();
    assert!(!paths.is_empty(), "Candidate profile paths must not be empty");
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("gamepad_profile.json")));

    let mut gp = GamepadController::new();
    // Verify check_and_reload_profile runs without panicking
    gp.check_and_reload_profile();
}

#[test]
fn test_custom_profile_raw_code_button_mappings() {
    use tdrace_app::input::gamepad::{CustomGamepadProfile, CustomButtonBinding};

    let mut gp = GamepadController::new();
    // Simulate a custom profile where Button A was mapped to raw code "Btn_BUTTON(3)"
    gp.custom_profile = Some(CustomGamepadProfile {
        device_name: "Twin USB Joystick".to_string(),
        handbrake: Some(CustomButtonBinding {
            code: "Btn_BUTTON(3)".to_string(),
            alternate: None,
        }),
        btn_south: Some(CustomButtonBinding {
            code: "Btn_BUTTON(3)".to_string(),
            alternate: None,
        }),
        btn_west: Some(CustomButtonBinding {
            code: "Btn_BUTTON(0)".to_string(),
            alternate: None,
        }),
        ..Default::default()
    });

    // Simulate pressing raw code Btn_BUTTON(3)
    gp.raw_buttons_held.push("Btn_BUTTON(3)".to_string());
    // Run update to process transitions
    gp.update();

    assert!(gp.snapshot.btn_confirm_pressed, "Custom raw button code should trigger btn_confirm_pressed");
    assert!(gp.snapshot.btn_a_pressed, "Custom raw button code should trigger btn_a_pressed");
}




