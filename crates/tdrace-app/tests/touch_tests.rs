use glam::Vec2;
use tdrace_app::input::touch::{
    point_in_rect, RawTouchPhase, RawTouchPoint, Rect, TouchController, TouchLayout,
};
use tdrace_core::physics::car::CarControls;

#[test]
fn test_touch_controller_initial_state_and_enable() {
    let mut tc = TouchController::new();
    assert!(!tc.enabled);
    assert_eq!(tc.layout, TouchLayout::VirtualJoystick);

    // Disabled controller returns zero controls
    let ctrl = tc.poll_controls();
    assert_eq!(ctrl, CarControls::default());

    tc.enabled = true;
    let ctrl_enabled = tc.poll_controls();
    assert_eq!(ctrl_enabled, CarControls::default());
}

#[test]
fn test_touch_layout_toggle() {
    let mut tc = TouchController::new();
    assert_eq!(tc.layout, TouchLayout::VirtualJoystick);

    tc.toggle_layout();
    assert_eq!(tc.layout, TouchLayout::SplitButtons);

    tc.toggle_layout();
    assert_eq!(tc.layout, TouchLayout::VirtualJoystick);
}

#[test]
fn test_joystick_deadzone_and_deflection() {
    let mut tc = TouchController::new();
    tc.enabled = true;
    let sw = 1280.0;
    let sh = 720.0;

    let center = tc.compute_joystick_center(sw, sh);
    tc.joystick.center = center;
    tc.joystick.deadzone_radius = 10.0;
    tc.joystick.outer_radius = 60.0;

    // 1. Touch inside deadzone (dx = 5.0, dy = 0.0) -> should produce 0.0 deflection
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 1,
            position: center + Vec2::new(5.0, 0.0),
            phase: RawTouchPhase::Started,
        }],
        sw,
        sh,
    );
    assert!(tc.joystick.is_active);
    assert_eq!(tc.joystick.deflection_x, 0.0);
    assert_eq!(tc.poll_controls().steer, 0.0);

    // 2. Touch halfway outside deadzone (dx = 35.0, dy = 0.0) -> deflection = (35 - 10)/(60 - 10) = 0.5
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 1,
            position: center + Vec2::new(35.0, 0.0),
            phase: RawTouchPhase::Moved,
        }],
        sw,
        sh,
    );
    assert!((tc.joystick.deflection_x - 0.5).abs() < 1e-4);
    assert!((tc.poll_controls().steer - 0.5).abs() < 1e-4);

    // 3. Touch at maximum reach to the left (dx = -60.0, dy = 0.0) -> deflection = -1.0
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 1,
            position: center + Vec2::new(-60.0, 0.0),
            phase: RawTouchPhase::Moved,
        }],
        sw,
        sh,
    );
    assert!((tc.joystick.deflection_x - (-1.0)).abs() < 1e-4);
    assert_eq!(tc.poll_controls().steer, -1.0);

    // 4. Release touch
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 1,
            position: center,
            phase: RawTouchPhase::Ended,
        }],
        sw,
        sh,
    );
    assert!(!tc.joystick.is_active);
    assert_eq!(tc.poll_controls().steer, 0.0);
}

#[test]
fn test_split_steering_buttons() {
    let mut tc = TouchController::new();
    tc.enabled = true;
    tc.layout = TouchLayout::SplitButtons;
    let sw = 1280.0;
    let sh = 720.0;

    let (left_rect, right_rect) = tc.compute_steer_rects(sw, sh);
    let left_center = Vec2::new(left_rect.x + left_rect.w * 0.5, left_rect.y + left_rect.h * 0.5);
    let right_center = Vec2::new(right_rect.x + right_rect.w * 0.5, right_rect.y + right_rect.h * 0.5);

    // Press Left button
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 1,
            position: left_center,
            phase: RawTouchPhase::Started,
        }],
        sw,
        sh,
    );
    assert!(tc.btn_steer_left.is_pressed);
    assert!(!tc.btn_steer_right.is_pressed);
    assert_eq!(tc.poll_controls().steer, -1.0);

    // Release Left button
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 1,
            position: left_center,
            phase: RawTouchPhase::Ended,
        }],
        sw,
        sh,
    );
    assert_eq!(tc.poll_controls().steer, 0.0);

    // Press Right button
    tc.process_touch_points(
        &[RawTouchPoint {
            id: 2,
            position: right_center,
            phase: RawTouchPhase::Started,
        }],
        sw,
        sh,
    );
    assert!(tc.btn_steer_right.is_pressed);
    assert_eq!(tc.poll_controls().steer, 1.0);

    // Press BOTH simultaneously -> steer cancels to 0.0
    tc.process_touch_points(
        &[
            RawTouchPoint {
                id: 1,
                position: left_center,
                phase: RawTouchPhase::Started,
            },
            RawTouchPoint {
                id: 2,
                position: right_center,
                phase: RawTouchPhase::Stationary,
            },
        ],
        sw,
        sh,
    );
    assert!(tc.btn_steer_left.is_pressed);
    assert!(tc.btn_steer_right.is_pressed);
    assert_eq!(tc.poll_controls().steer, 0.0);
}

#[test]
fn test_simultaneous_multi_touch_gestures() {
    let mut tc = TouchController::new();
    tc.enabled = true;
    tc.layout = TouchLayout::VirtualJoystick;
    let sw = 1280.0;
    let sh = 720.0;

    let (gas_rect, brake_rect, handbrake_rect, _) = tc.compute_pedal_rects(sw, sh);
    let gas_center = Vec2::new(gas_rect.x + gas_rect.w * 0.5, gas_rect.y + gas_rect.h * 0.5);
    let handbrake_center = Vec2::new(handbrake_rect.x + handbrake_rect.w * 0.5, handbrake_rect.y + handbrake_rect.h * 0.5);
    let joy_center = tc.compute_joystick_center(sw, sh);

    // 3 Simultaneous Fingers:
    // Finger 1 (id 10): Full Right Steer on Joystick
    // Finger 2 (id 20): Full Gas on Accelerator
    // Finger 3 (id 30): Active Handbrake
    tc.process_touch_points(
        &[
            RawTouchPoint {
                id: 10,
                position: joy_center + Vec2::new(70.0, 0.0),
                phase: RawTouchPhase::Started,
            },
            RawTouchPoint {
                id: 20,
                position: gas_center,
                phase: RawTouchPhase::Started,
            },
            RawTouchPoint {
                id: 30,
                position: handbrake_center,
                phase: RawTouchPhase::Started,
            },
        ],
        sw,
        sh,
    );

    let ctrl = tc.poll_controls();
    assert_eq!(ctrl.throttle, 1.0);
    assert_eq!(ctrl.brake, 0.0);
    assert_eq!(ctrl.steer, 1.0);
    assert!(ctrl.handbrake);
    assert!(!ctrl.reverse);

    // Now release handbrake and press brake (Finger 4, id 40)
    let brake_center = Vec2::new(brake_rect.x + brake_rect.w * 0.5, brake_rect.y + brake_rect.h * 0.5);
    tc.process_touch_points(
        &[
            RawTouchPoint {
                id: 30,
                position: handbrake_center,
                phase: RawTouchPhase::Ended,
            },
            RawTouchPoint {
                id: 40,
                position: brake_center,
                phase: RawTouchPhase::Started,
            },
        ],
        sw,
        sh,
    );

    let ctrl2 = tc.poll_controls();
    assert_eq!(ctrl2.throttle, 1.0);
    assert_eq!(ctrl2.brake, 1.0);
    assert!(ctrl2.reverse);
    assert!(!ctrl2.handbrake);
}

#[test]
fn test_point_in_rect_boundaries() {
    let r = Rect {
        x: 10.0,
        y: 20.0,
        w: 50.0,
        h: 40.0,
    };

    assert!(point_in_rect(Vec2::new(10.0, 20.0), r)); // Top-Left corner
    assert!(point_in_rect(Vec2::new(60.0, 60.0), r)); // Bottom-Right corner
    assert!(point_in_rect(Vec2::new(35.0, 40.0), r)); // Center

    assert!(!point_in_rect(Vec2::new(9.9, 20.0), r));
    assert!(!point_in_rect(Vec2::new(60.1, 40.0), r));
    assert!(!point_in_rect(Vec2::new(30.0, 19.9), r));
    assert!(!point_in_rect(Vec2::new(30.0, 60.1), r));
}
