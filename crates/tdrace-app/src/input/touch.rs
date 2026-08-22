use glam::Vec2;
use macroquad::color::Color;
use macroquad::input::{is_mouse_button_down, mouse_position, touches, MouseButton, TouchPhase};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines};
use macroquad::text::{draw_text, measure_text};
use serde::{Deserialize, Serialize};
use tdrace_core::physics::car::CarControls;

/// Layout modes for touch steering controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TouchLayout {
    /// Virtual analog steering stick on the left.
    VirtualJoystick,
    /// Split Left / Right steering buttons on the left.
    SplitButtons,
}

/// Abstract touch phase for platform-independent handling & testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawTouchPhase {
    Started,
    Moved,
    Stationary,
    Ended,
    Cancelled,
}

/// Abstract touch point for platform-independent multi-touch tracking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawTouchPoint {
    pub id: u64,
    pub position: Vec2,
    pub phase: RawTouchPhase,
}

/// State of an individual interactive touch button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchButtonState {
    pub is_pressed: bool,
    pub active_touch_id: Option<u64>,
    pub press_anim: f32, // 0.0 to 1.0 animation pulse
}

impl Default for TouchButtonState {
    fn default() -> Self {
        Self {
            is_pressed: false,
            active_touch_id: None,
            press_anim: 0.0,
        }
    }
}

/// Virtual analog stick tracking state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualJoystickState {
    pub center: Vec2,
    pub current_knob_pos: Vec2,
    pub active_touch_id: Option<u64>,
    pub is_active: bool,
    pub outer_radius: f32,
    pub knob_radius: f32,
    pub deadzone_radius: f32,
    pub deflection_x: f32, // -1.0 to 1.0
    pub deflection_y: f32, // -1.0 to 1.0
}

impl Default for VirtualJoystickState {
    fn default() -> Self {
        Self {
            center: Vec2::new(120.0, 560.0),
            current_knob_pos: Vec2::new(120.0, 560.0),
            active_touch_id: None,
            is_active: false,
            outer_radius: 65.0,
            knob_radius: 28.0,
            deadzone_radius: 8.0,
            deflection_x: 0.0,
            deflection_y: 0.0,
        }
    }
}

/// Mobile Multi-Touch Input Controller.
///
/// Features:
/// - Zero-lag multi-touch tracking supporting simultaneous multi-finger gestures.
/// - Virtual Joystick mode with configurable deadzone and progressive deflection.
/// - Split Left/Right steering buttons mode.
/// - Dedicated Gas (accelerator), Brake (service brake + reverse), and Handbrake buttons.
/// - Automatic touch / mobile detection with fallback and manual overrides.
/// - Customizable UI opacity and haptic-style visual pulse feedback.
#[derive(Debug, Clone, PartialEq)]
pub struct TouchController {
    /// Active layout style.
    pub layout: TouchLayout,
    /// Master visibility toggle for touch overlay.
    pub enabled: bool,
    /// Auto-detect touch events and display overlay.
    pub auto_detect: bool,
    /// Base alpha transparency of touch controls (0.0 to 1.0).
    pub opacity: f32,

    // Controls
    pub joystick: VirtualJoystickState,
    pub btn_steer_left: TouchButtonState,
    pub btn_steer_right: TouchButtonState,
    pub btn_gas: TouchButtonState,
    pub btn_brake: TouchButtonState,
    pub btn_handbrake: TouchButtonState,
    pub btn_toggle_layout: TouchButtonState,

    // Internal metrics
    pub simulated_mouse_enabled: bool,
    pub last_screen_size: Vec2,
}

impl Default for TouchController {
    fn default() -> Self {
        Self::new()
    }
}

impl TouchController {
    pub fn new() -> Self {
        Self {
            layout: TouchLayout::VirtualJoystick,
            enabled: false,
            auto_detect: true,
            opacity: 0.70,

            joystick: VirtualJoystickState::default(),
            btn_steer_left: TouchButtonState::default(),
            btn_steer_right: TouchButtonState::default(),
            btn_gas: TouchButtonState::default(),
            btn_brake: TouchButtonState::default(),
            btn_handbrake: TouchButtonState::default(),
            btn_toggle_layout: TouchButtonState::default(),

            simulated_mouse_enabled: true,
            last_screen_size: Vec2::new(1280.0, 720.0),
        }
    }

    /// Toggles between Virtual Joystick and Split Buttons layouts.
    pub fn toggle_layout(&mut self) {
        self.layout = match self.layout {
            TouchLayout::VirtualJoystick => TouchLayout::SplitButtons,
            TouchLayout::SplitButtons => TouchLayout::VirtualJoystick,
        };
        self.reset_touches();
    }

    /// Resets all touch tracking states.
    pub fn reset_touches(&mut self) {
        self.joystick.is_active = false;
        self.joystick.active_touch_id = None;
        self.joystick.current_knob_pos = self.joystick.center;
        self.joystick.deflection_x = 0.0;
        self.joystick.deflection_y = 0.0;

        self.btn_steer_left = TouchButtonState::default();
        self.btn_steer_right = TouchButtonState::default();
        self.btn_gas = TouchButtonState::default();
        self.btn_brake = TouchButtonState::default();
        self.btn_handbrake = TouchButtonState::default();
        self.btn_toggle_layout = TouchButtonState::default();
    }

    /// Updates touch states using real macroquad hardware touches and optional mouse simulation.
    pub fn update_from_macroquad(&mut self, screen_w: f32, screen_h: f32, dt: f32) {
        self.last_screen_size = Vec2::new(screen_w, screen_h);
        self.update_press_animations(dt);

        let real_touches = touches();
        if !real_touches.is_empty() {
            if self.auto_detect && !self.enabled {
                self.enabled = true;
            }

            let raw_points: Vec<RawTouchPoint> = real_touches
                .iter()
                .map(|t| RawTouchPoint {
                    id: t.id,
                    position: Vec2::new(t.position.x, t.position.y),
                    phase: match t.phase {
                        TouchPhase::Started => RawTouchPhase::Started,
                        TouchPhase::Moved => RawTouchPhase::Moved,
                        TouchPhase::Stationary => RawTouchPhase::Stationary,
                        TouchPhase::Ended => RawTouchPhase::Ended,
                        TouchPhase::Cancelled => RawTouchPhase::Cancelled,
                    },
                })
                .collect();

            self.process_touch_points(&raw_points, screen_w, screen_h);
            return;
        }

        // Fallback desktop mouse touch simulation (for testing on desktop)
        if self.simulated_mouse_enabled && self.enabled {
            let (mx, my) = mouse_position();
            let mouse_pos = Vec2::new(mx, my);
            let mouse_down = is_mouse_button_down(MouseButton::Left);

            let mouse_touch_id = 999999;
            if mouse_down {
                let phase = if self.joystick.active_touch_id == Some(mouse_touch_id)
                    || self.btn_gas.active_touch_id == Some(mouse_touch_id)
                    || self.btn_brake.active_touch_id == Some(mouse_touch_id)
                    || self.btn_handbrake.active_touch_id == Some(mouse_touch_id)
                    || self.btn_steer_left.active_touch_id == Some(mouse_touch_id)
                    || self.btn_steer_right.active_touch_id == Some(mouse_touch_id)
                {
                    RawTouchPhase::Moved
                } else {
                    RawTouchPhase::Started
                };

                self.process_touch_points(
                    &[RawTouchPoint {
                        id: mouse_touch_id,
                        position: mouse_pos,
                        phase,
                    }],
                    screen_w,
                    screen_h,
                );
            } else {
                // Release mouse touch if active
                self.process_touch_points(
                    &[RawTouchPoint {
                        id: mouse_touch_id,
                        position: mouse_pos,
                        phase: RawTouchPhase::Ended,
                    }],
                    screen_w,
                    screen_h,
                );
            }
        }
    }

    /// Platform-independent touch processing method (used by tests and runtime).
    pub fn process_touch_points(&mut self, touches: &[RawTouchPoint], screen_w: f32, screen_h: f32) {
        self.last_screen_size = Vec2::new(screen_w, screen_h);

        // Update button hit-boxes according to screen dimensions
        let (gas_rect, brake_rect, handbrake_rect, layout_rect) = self.compute_pedal_rects(screen_w, screen_h);
        let (left_rect, right_rect) = self.compute_steer_rects(screen_w, screen_h);
        let joy_center = self.compute_joystick_center(screen_w, screen_h);
        self.joystick.center = joy_center;

        for touch in touches {
            let pos = touch.position;
            let id = touch.id;

            match touch.phase {
                RawTouchPhase::Started => {
                    // Check Layout Toggle button (top-left or controls bar)
                    if point_in_rect(pos, layout_rect) {
                        self.toggle_layout();
                        self.btn_toggle_layout.press_anim = 1.0;
                        continue;
                    }

                    // Check Right side pedals
                    if point_in_rect(pos, gas_rect) {
                        self.btn_gas.is_pressed = true;
                        self.btn_gas.active_touch_id = Some(id);
                        self.btn_gas.press_anim = 1.0;
                    } else if point_in_rect(pos, brake_rect) {
                        self.btn_brake.is_pressed = true;
                        self.btn_brake.active_touch_id = Some(id);
                        self.btn_brake.press_anim = 1.0;
                    } else if point_in_rect(pos, handbrake_rect) {
                        self.btn_handbrake.is_pressed = true;
                        self.btn_handbrake.active_touch_id = Some(id);
                        self.btn_handbrake.press_anim = 1.0;
                    }

                    // Check Left side steering
                    match self.layout {
                        TouchLayout::VirtualJoystick => {
                            // If touch starts on left 45% of screen
                            if pos.x < screen_w * 0.45 && pos.y > screen_h * 0.35 {
                                self.joystick.active_touch_id = Some(id);
                                self.joystick.is_active = true;
                                self.update_joystick_pos(pos);
                            }
                        }
                        TouchLayout::SplitButtons => {
                            if point_in_rect(pos, left_rect) {
                                self.btn_steer_left.is_pressed = true;
                                self.btn_steer_left.active_touch_id = Some(id);
                                self.btn_steer_left.press_anim = 1.0;
                            } else if point_in_rect(pos, right_rect) {
                                self.btn_steer_right.is_pressed = true;
                                self.btn_steer_right.active_touch_id = Some(id);
                                self.btn_steer_right.press_anim = 1.0;
                            }
                        }
                    }
                }
                RawTouchPhase::Moved | RawTouchPhase::Stationary => {
                    // Update joystick if this touch owns it
                    if self.joystick.active_touch_id == Some(id) {
                        self.update_joystick_pos(pos);
                    }

                    // Update split buttons if touch dragged across
                    if self.layout == TouchLayout::SplitButtons {
                        if self.btn_steer_left.active_touch_id == Some(id) {
                            self.btn_steer_left.is_pressed = point_in_rect(pos, left_rect);
                        }
                        if self.btn_steer_right.active_touch_id == Some(id) {
                            self.btn_steer_right.is_pressed = point_in_rect(pos, right_rect);
                        }
                    }

                    // Update pedals
                    if self.btn_gas.active_touch_id == Some(id) {
                        self.btn_gas.is_pressed = point_in_rect(pos, gas_rect);
                    }
                    if self.btn_brake.active_touch_id == Some(id) {
                        self.btn_brake.is_pressed = point_in_rect(pos, brake_rect);
                    }
                    if self.btn_handbrake.active_touch_id == Some(id) {
                        self.btn_handbrake.is_pressed = point_in_rect(pos, handbrake_rect);
                    }
                }
                RawTouchPhase::Ended | RawTouchPhase::Cancelled => {
                    // Release joystick
                    if self.joystick.active_touch_id == Some(id) {
                        self.joystick.is_active = false;
                        self.joystick.active_touch_id = None;
                        self.joystick.current_knob_pos = self.joystick.center;
                        self.joystick.deflection_x = 0.0;
                        self.joystick.deflection_y = 0.0;
                    }

                    // Release buttons
                    if self.btn_steer_left.active_touch_id == Some(id) {
                        self.btn_steer_left.is_pressed = false;
                        self.btn_steer_left.active_touch_id = None;
                    }
                    if self.btn_steer_right.active_touch_id == Some(id) {
                        self.btn_steer_right.is_pressed = false;
                        self.btn_steer_right.active_touch_id = None;
                    }
                    if self.btn_gas.active_touch_id == Some(id) {
                        self.btn_gas.is_pressed = false;
                        self.btn_gas.active_touch_id = None;
                    }
                    if self.btn_brake.active_touch_id == Some(id) {
                        self.btn_brake.is_pressed = false;
                        self.btn_brake.active_touch_id = None;
                    }
                    if self.btn_handbrake.active_touch_id == Some(id) {
                        self.btn_handbrake.is_pressed = false;
                        self.btn_handbrake.active_touch_id = None;
                    }
                }
            }
        }
    }

    /// Updates joystick deflection calculation with deadzone and saturation clamping.
    fn update_joystick_pos(&mut self, pos: Vec2) {
        let center = self.joystick.center;
        let delta = pos - center;
        let dist = delta.length();

        if dist < self.joystick.deadzone_radius {
            self.joystick.current_knob_pos = center;
            self.joystick.deflection_x = 0.0;
            self.joystick.deflection_y = 0.0;
            return;
        }

        let max_r = self.joystick.outer_radius;
        let deadzone = self.joystick.deadzone_radius;
        let clamped_dist = dist.min(max_r);
        let dir = if dist > 1e-4 { delta / dist } else { Vec2::ZERO };

        self.joystick.current_knob_pos = center + dir * clamped_dist;

        // Normalized deflection magnitude after deadzone subtraction
        let active_span = (max_r - deadzone).max(1.0);
        let norm_mag = ((clamped_dist - deadzone) / active_span).clamp(0.0, 1.0);

        self.joystick.deflection_x = (dir.x * norm_mag).clamp(-1.0, 1.0);
        self.joystick.deflection_y = (dir.y * norm_mag).clamp(-1.0, 1.0);
    }

    /// Decays visual press animations.
    fn update_press_animations(&mut self, dt: f32) {
        let decay = dt * 4.0;
        self.btn_gas.press_anim = (self.btn_gas.press_anim - decay).max(0.0);
        self.btn_brake.press_anim = (self.btn_brake.press_anim - decay).max(0.0);
        self.btn_handbrake.press_anim = (self.btn_handbrake.press_anim - decay).max(0.0);
        self.btn_steer_left.press_anim = (self.btn_steer_left.press_anim - decay).max(0.0);
        self.btn_steer_right.press_anim = (self.btn_steer_right.press_anim - decay).max(0.0);
        self.btn_toggle_layout.press_anim = (self.btn_toggle_layout.press_anim - decay).max(0.0);
    }

    /// Produces standard `CarControls` from current touch state.
    pub fn poll_controls(&self) -> CarControls {
        if !self.enabled {
            return CarControls::default();
        }

        let mut steer = 0.0f32;
        match self.layout {
            TouchLayout::VirtualJoystick => {
                steer = self.joystick.deflection_x;
            }
            TouchLayout::SplitButtons => {
                if self.btn_steer_left.is_pressed {
                    steer -= 1.0;
                }
                if self.btn_steer_right.is_pressed {
                    steer += 1.0;
                }
            }
        }

        let throttle = if self.btn_gas.is_pressed { 1.0 } else { 0.0 };
        let brake = if self.btn_brake.is_pressed { 1.0 } else { 0.0 };
        let reverse = self.btn_brake.is_pressed;
        let handbrake = self.btn_handbrake.is_pressed;

        CarControls {
            throttle,
            steer: steer.clamp(-1.0, 1.0),
            brake,
            handbrake,
            reverse,
        }
    }

    // --- Geometry Layout Helpers ---

    pub fn compute_joystick_center(&self, _sw: f32, sh: f32) -> Vec2 {
        Vec2::new(130.0, sh - 130.0)
    }

    pub fn compute_steer_rects(&self, _sw: f32, sh: f32) -> (Rect, Rect) {
        let btn_w = 90.0;
        let btn_h = 110.0;
        let y = sh - 150.0;
        let left_rect = Rect {
            x: 25.0,
            y,
            w: btn_w,
            h: btn_h,
        };
        let right_rect = Rect {
            x: 130.0,
            y,
            w: btn_w,
            h: btn_h,
        };
        (left_rect, right_rect)
    }

    pub fn compute_pedal_rects(&self, sw: f32, sh: f32) -> (Rect, Rect, Rect, Rect) {
        let gas_w = 85.0;
        let gas_h = 135.0;
        let gas_rect = Rect {
            x: sw - gas_w - 25.0,
            y: sh - gas_h - 25.0,
            w: gas_w,
            h: gas_h,
        };

        let brake_w = 80.0;
        let brake_h = 110.0;
        let brake_rect = Rect {
            x: sw - gas_w - brake_w - 45.0,
            y: sh - brake_h - 25.0,
            w: brake_w,
            h: brake_h,
        };

        let handbrake_w = 110.0;
        let handbrake_h = 45.0;
        let handbrake_rect = Rect {
            x: sw - handbrake_w - 25.0,
            y: sh - gas_h - handbrake_h - 45.0,
            w: handbrake_w,
            h: handbrake_h,
        };

        let layout_toggle_rect = Rect {
            x: 25.0,
            y: 25.0,
            w: 110.0,
            h: 36.0,
        };

        (gas_rect, brake_rect, handbrake_rect, layout_toggle_rect)
    }

    /// Renders the touch control interface with glowing feedback and haptic visuals.
    pub fn render(&self, sw: f32, sh: f32) {
        if !self.enabled {
            return;
        }

        let alpha = self.opacity.clamp(0.0, 1.0);
        let (gas_rect, brake_rect, handbrake_rect, layout_rect) = self.compute_pedal_rects(sw, sh);

        // 1. Layout Toggle Button (Top Left)
        self.render_layout_toggle_btn(layout_rect, alpha);

        // 2. Left Side Steering (Joystick vs Split Buttons)
        match self.layout {
            TouchLayout::VirtualJoystick => {
                self.render_joystick(alpha);
            }
            TouchLayout::SplitButtons => {
                let (left_rect, right_rect) = self.compute_steer_rects(sw, sh);
                self.render_steer_button(left_rect, "◀", self.btn_steer_left, alpha);
                self.render_steer_button(right_rect, "▶", self.btn_steer_right, alpha);
            }
        }

        // 3. Right Side Pedals (Gas, Brake, Handbrake)
        self.render_gas_pedal(gas_rect, alpha);
        self.render_brake_pedal(brake_rect, alpha);
        self.render_handbrake_btn(handbrake_rect, alpha);
    }

    fn render_layout_toggle_btn(&self, r: Rect, alpha: f32) {
        let is_pressed = self.btn_toggle_layout.press_anim > 0.05;
        let bg_col = if is_pressed {
            Color::new(0.2, 0.5, 0.9, alpha * 0.9)
        } else {
            Color::new(0.08, 0.10, 0.15, alpha * 0.75)
        };
        let border_col = Color::new(0.4, 0.7, 1.0, alpha * 0.9);

        draw_rectangle(r.x, r.y, r.w, r.h, bg_col);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, border_col);

        let label = match self.layout {
            TouchLayout::VirtualJoystick => "STICK 🕹️",
            TouchLayout::SplitButtons => "BUTTONS ◀▶",
        };
        let m = measure_text(label, None, 14, 1.0);
        draw_text(
            label,
            r.x + (r.w - m.width) * 0.5,
            r.y + (r.h + 8.0) * 0.5,
            14.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn render_joystick(&self, alpha: f32) {
        let center = self.joystick.center;
        let outer_r = self.joystick.outer_radius;
        let knob_r = self.joystick.knob_radius;
        let knob_pos = self.joystick.current_knob_pos;
        let is_active = self.joystick.is_active;

        // Outer base ring
        let base_bg = Color::new(0.06, 0.08, 0.12, alpha * 0.65);
        let base_border = if is_active {
            Color::new(0.2, 0.85, 1.0, alpha * 0.95)
        } else {
            Color::new(0.3, 0.4, 0.55, alpha * 0.7)
        };

        draw_circle(center.x, center.y, outer_r, base_bg);
        draw_circle_lines(center.x, center.y, outer_r, 2.5, base_border);
        draw_circle_lines(center.x, center.y, self.joystick.deadzone_radius, 1.0, Color::new(0.3, 0.5, 0.7, alpha * 0.35));

        // Deflection indicator line
        if is_active {
            draw_line(center.x, center.y, knob_pos.x, knob_pos.y, 3.0, Color::new(0.2, 0.85, 1.0, alpha * 0.75));
        }

        // Thumb Knob
        let knob_bg = if is_active {
            Color::new(0.2, 0.75, 1.0, alpha * 0.9)
        } else {
            Color::new(0.25, 0.32, 0.45, alpha * 0.8)
        };
        let knob_border = Color::new(1.0, 1.0, 1.0, alpha * 0.95);

        draw_circle(knob_pos.x, knob_pos.y, knob_r, knob_bg);
        draw_circle_lines(knob_pos.x, knob_pos.y, knob_r, 2.0, knob_border);

        // Knob directional marker
        draw_circle(knob_pos.x, knob_pos.y, 4.0, Color::new(1.0, 1.0, 1.0, alpha));
    }

    fn render_steer_button(&self, r: Rect, icon: &str, state: TouchButtonState, alpha: f32) {
        let is_down = state.is_pressed;
        let bg_col = if is_down {
            Color::new(0.15, 0.65, 1.0, alpha * 0.95)
        } else {
            Color::new(0.08, 0.10, 0.15, alpha * 0.75)
        };
        let border_col = if is_down {
            Color::new(0.8, 0.95, 1.0, alpha)
        } else {
            Color::new(0.3, 0.45, 0.65, alpha * 0.8)
        };

        draw_rectangle(r.x, r.y, r.w, r.h, bg_col);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, border_col);

        let m = measure_text(icon, None, 36, 1.0);
        draw_text(
            icon,
            r.x + (r.w - m.width) * 0.5,
            r.y + (r.h + 20.0) * 0.5,
            36.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn render_gas_pedal(&self, r: Rect, alpha: f32) {
        let is_down = self.btn_gas.is_pressed;
        let bg_col = if is_down {
            Color::new(0.1, 0.85, 0.35, alpha * 0.95)
        } else {
            Color::new(0.06, 0.14, 0.08, alpha * 0.75)
        };
        let border_col = if is_down {
            Color::new(0.4, 1.0, 0.6, alpha)
        } else {
            Color::new(0.2, 0.7, 0.35, alpha * 0.8)
        };

        draw_rectangle(r.x, r.y, r.w, r.h, bg_col);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.5, border_col);

        // Pedal grip ridges
        let ridge_count = 5;
        let step_y = r.h / (ridge_count + 1) as f32;
        for i in 1..=ridge_count {
            let ry = r.y + step_y * i as f32;
            draw_line(r.x + 12.0, ry, r.x + r.w - 12.0, ry, 2.0, Color::new(1.0, 1.0, 1.0, alpha * 0.5));
        }

        let m = measure_text("GAS", None, 18, 1.0);
        draw_text(
            "GAS",
            r.x + (r.w - m.width) * 0.5,
            r.y + r.h - 14.0,
            18.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn render_brake_pedal(&self, r: Rect, alpha: f32) {
        let is_down = self.btn_brake.is_pressed;
        let bg_col = if is_down {
            Color::new(0.9, 0.15, 0.15, alpha * 0.95)
        } else {
            Color::new(0.15, 0.06, 0.06, alpha * 0.75)
        };
        let border_col = if is_down {
            Color::new(1.0, 0.4, 0.4, alpha)
        } else {
            Color::new(0.8, 0.25, 0.25, alpha * 0.8)
        };

        draw_rectangle(r.x, r.y, r.w, r.h, bg_col);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.5, border_col);

        // Pedal grip ridges
        let ridge_count = 4;
        let step_y = r.h / (ridge_count + 1) as f32;
        for i in 1..=ridge_count {
            let ry = r.y + step_y * i as f32;
            draw_line(r.x + 10.0, ry, r.x + r.w - 10.0, ry, 2.0, Color::new(1.0, 1.0, 1.0, alpha * 0.5));
        }

        let m = measure_text("BRAKE", None, 16, 1.0);
        draw_text(
            "BRAKE",
            r.x + (r.w - m.width) * 0.5,
            r.y + r.h - 12.0,
            16.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }

    fn render_handbrake_btn(&self, r: Rect, alpha: f32) {
        let is_down = self.btn_handbrake.is_pressed;
        let bg_col = if is_down {
            Color::new(1.0, 0.6, 0.1, alpha * 0.95)
        } else {
            Color::new(0.16, 0.10, 0.04, alpha * 0.75)
        };
        let border_col = if is_down {
            Color::new(1.0, 0.8, 0.3, alpha)
        } else {
            Color::new(0.9, 0.5, 0.15, alpha * 0.8)
        };

        draw_rectangle(r.x, r.y, r.w, r.h, bg_col);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, border_col);

        let label = "DRIFT / E-BRAKE";
        let m = measure_text(label, None, 13, 1.0);
        draw_text(
            label,
            r.x + (r.w - m.width) * 0.5,
            r.y + (r.h + 8.0) * 0.5,
            13.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }
}

/// Simple 2D bounding rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[inline]
pub fn point_in_rect(p: Vec2, r: Rect) -> bool {
    p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h
}
