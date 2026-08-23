use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};
use serde::{Deserialize, Serialize};

/// Configuration for Gamepad analog stick deadzones, trigger thresholds, and sensitivity curves.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GamepadConfig {
    /// Inner deadzone threshold for left analog stick [0.0 .. 1.0].
    pub stick_deadzone: f32,
    /// Inner deadzone threshold for analog triggers [0.0 .. 1.0].
    pub trigger_deadzone: f32,
    /// Sensitivity curve exponent for analog steering (1.0 = linear, 1.15 = gentle center).
    pub steer_exponent: f32,
    /// Sensitivity scale for steering.
    pub steer_scale: f32,
}

impl Default for GamepadConfig {
    fn default() -> Self {
        Self {
            stick_deadzone: 0.12,
            trigger_deadzone: 0.05,
            steer_exponent: 1.15,
            steer_scale: 1.0,
        }
    }
}

/// Dynamic snapshot of current Gamepad button and analog axis inputs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GamepadSnapshot {
    /// Whether any gamepad is currently connected.
    pub is_connected: bool,
    /// Name or model of the active connected gamepad.
    pub gamepad_name: String,
    /// Proportional analog steering [-1.0 = left, 0.0 = center, +1.0 = right].
    pub steer: f32,
    /// Proportional analog throttle [0.0 = idle, 1.0 = full gas].
    pub throttle: f32,
    /// Proportional analog brake [0.0 = release, 1.0 = full brake].
    pub brake: f32,
    /// Handbrake button pressed (B / East or Right Bumper).
    pub handbrake: bool,
    /// Reverse button pressed (Y / North or Left Bumper).
    pub reverse: bool,

    // Button trigger events (pressed this frame)
    pub btn_start_pressed: bool,
    pub btn_back_pressed: bool,
    pub btn_a_pressed: bool,      // South (Confirm / Throttle / Select)
    pub btn_b_pressed: bool,      // East (Back / Handbrake)
    pub btn_x_pressed: bool,      // West (Brake / Mode toggle)
    pub btn_y_pressed: bool,      // North (Cam / Reset)
    pub dpad_up_pressed: bool,
    pub dpad_down_pressed: bool,
    pub dpad_left_pressed: bool,
    pub dpad_right_pressed: bool,
    pub btn_assist_toggle_pressed: bool, // Right Stick Click or Select
    pub btn_cam_toggle_pressed: bool,    // Left Stick Click or Y
}

/// Cross-platform Gamepad Manager supporting hot-plugging, analog axes, and button events.
pub struct GamepadController {
    gilrs: Option<Gilrs>,
    pub config: GamepadConfig,
    pub active_gamepad: Option<GamepadId>,
    pub snapshot: GamepadSnapshot,
}

impl Default for GamepadController {
    fn default() -> Self {
        Self::new()
    }
}

impl GamepadController {
    /// Creates a new GamepadController, attempting to initialize the native Gilrs subsystem.
    pub fn new() -> Self {
        let (gilrs, active_gamepad, gamepad_name, is_connected) = match Gilrs::new() {
            Ok(g) => {
                let first_gamepad = g.gamepads().next().map(|(id, gp)| (id, gp.name().to_string()));
                let (active, name, connected) = if let Some((id, name)) = first_gamepad {
                    (Some(id), name, true)
                } else {
                    (None, "No Gamepad Connected".to_string(), false)
                };
                (Some(g), active, name, connected)
            }
            Err(err) => {
                println!("[Gamepad] Gilrs initialization notice: {err} (falling back to keyboard/touch)");
                (None, None, "Gamepad Unavailable".to_string(), false)
            }
        };

        let mut snapshot = GamepadSnapshot::default();
        snapshot.is_connected = is_connected;
        snapshot.gamepad_name = gamepad_name;

        Self {
            gilrs,
            config: GamepadConfig::default(),
            active_gamepad,
            snapshot,
        }
    }

    /// Clears per-frame button press events.
    pub fn clear_frame_events(&mut self) {
        self.snapshot.btn_start_pressed = false;
        self.snapshot.btn_back_pressed = false;
        self.snapshot.btn_a_pressed = false;
        self.snapshot.btn_b_pressed = false;
        self.snapshot.btn_x_pressed = false;
        self.snapshot.btn_y_pressed = false;
        self.snapshot.dpad_up_pressed = false;
        self.snapshot.dpad_down_pressed = false;
        self.snapshot.dpad_left_pressed = false;
        self.snapshot.dpad_right_pressed = false;
        self.snapshot.btn_assist_toggle_pressed = false;
        self.snapshot.btn_cam_toggle_pressed = false;
    }

    /// Polls and updates gamepad state, draining all hardware events.
    pub fn update(&mut self) {
        self.clear_frame_events();

        let Some(ref mut gilrs) = self.gilrs else {
            return;
        };

        // Drain pending hardware events
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            match event {
                EventType::Connected => {
                    self.active_gamepad = Some(id);
                    if let Some(gp) = gilrs.connected_gamepad(id) {
                        self.snapshot.gamepad_name = gp.name().to_string();
                        self.snapshot.is_connected = true;
                        println!("[Gamepad] Connected: {} (ID {:?})", gp.name(), id);
                    }
                }
                EventType::Disconnected => {
                    if self.active_gamepad == Some(id) {
                        self.active_gamepad = gilrs.gamepads().next().map(|(next_id, _)| next_id);
                        if let Some(active_id) = self.active_gamepad {
                            if let Some(gp) = gilrs.connected_gamepad(active_id) {
                                self.snapshot.gamepad_name = gp.name().to_string();
                                self.snapshot.is_connected = true;
                            }
                        } else {
                            self.snapshot.is_connected = false;
                            self.snapshot.gamepad_name = "No Gamepad Connected".to_string();
                        }
                        println!("[Gamepad] Disconnected: ID {:?}", id);
                    }
                }
                EventType::ButtonPressed(btn, _) => {
                    match btn {
                        Button::Start => self.snapshot.btn_start_pressed = true,
                        Button::Select => {
                            self.snapshot.btn_back_pressed = true;
                            self.snapshot.btn_assist_toggle_pressed = true;
                        }
                        Button::South => self.snapshot.btn_a_pressed = true,
                        Button::East => self.snapshot.btn_b_pressed = true,
                        Button::West => self.snapshot.btn_x_pressed = true,
                        Button::North => {
                            self.snapshot.btn_y_pressed = true;
                            self.snapshot.btn_cam_toggle_pressed = true;
                        }
                        Button::DPadUp => self.snapshot.dpad_up_pressed = true,
                        Button::DPadDown => self.snapshot.dpad_down_pressed = true,
                        Button::DPadLeft => self.snapshot.dpad_left_pressed = true,
                        Button::DPadRight => self.snapshot.dpad_right_pressed = true,
                        Button::RightThumb => self.snapshot.btn_assist_toggle_pressed = true,
                        Button::LeftThumb => self.snapshot.btn_cam_toggle_pressed = true,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let config = self.config;

        // Sample continuous analog axes and button states from active gamepad
        if let Some(id) = self.active_gamepad {
            if let Some(gp) = gilrs.connected_gamepad(id) {
                let name = gp.name().to_string();
                let is_conn = true;

                // 1. Left Analog Stick (Steering)
                let raw_stick_x = gp.axis_data(Axis::LeftStickX).map(|d| d.value()).unwrap_or(0.0);
                let raw_dpad_x = if gp.is_pressed(Button::DPadRight) {
                    1.0
                } else if gp.is_pressed(Button::DPadLeft) {
                    -1.0
                } else {
                    0.0
                };

                let stick_steer = Self::process_axis_deadzone(raw_stick_x, config.stick_deadzone, config.steer_exponent);
                let steer = (stick_steer * config.steer_scale + raw_dpad_x).clamp(-1.0, 1.0);

                // 2. Right Trigger RT / Button A (Throttle)
                let raw_rt = gp.button_data(Button::RightTrigger2).map(|d| d.value()).unwrap_or(0.0);
                let rt_throttle = Self::process_trigger_deadzone(raw_rt, config.trigger_deadzone);
                let btn_throttle = if gp.is_pressed(Button::South) { 1.0 } else { 0.0 };
                let throttle = rt_throttle.max(btn_throttle).clamp(0.0, 1.0);

                // 3. Left Trigger LT / Button X (Brake)
                let raw_lt = gp.button_data(Button::LeftTrigger2).map(|d| d.value()).unwrap_or(0.0);
                let lt_brake = Self::process_trigger_deadzone(raw_lt, config.trigger_deadzone);
                let btn_brake = if gp.is_pressed(Button::West) { 1.0 } else { 0.0 };
                let brake = lt_brake.max(btn_brake).clamp(0.0, 1.0);

                // 4. Handbrake (Button B / East or Right Bumper RB)
                let handbrake = gp.is_pressed(Button::East) || gp.is_pressed(Button::RightTrigger);

                // 5. Reverse (Button Y / North or Left Bumper LB)
                let reverse = gp.is_pressed(Button::North) || gp.is_pressed(Button::LeftTrigger);

                self.snapshot.is_connected = is_conn;
                self.snapshot.gamepad_name = name;
                self.snapshot.steer = steer;
                self.snapshot.throttle = throttle;
                self.snapshot.brake = brake;
                self.snapshot.handbrake = handbrake;
                self.snapshot.reverse = reverse;
            }
        }
    }

    /// Applies inner deadzone and non-linear power curve to analog stick [-1.0 .. 1.0].
    pub fn process_axis_deadzone(raw: f32, deadzone: f32, exponent: f32) -> f32 {
        let abs_val = raw.abs();
        if abs_val <= deadzone {
            0.0
        } else {
            let rescaled = (abs_val - deadzone) / (1.0 - deadzone);
            raw.signum() * rescaled.powf(exponent).clamp(0.0, 1.0)
        }
    }

    /// Applies deadzone to analog trigger [0.0 .. 1.0].
    pub fn process_trigger_deadzone(raw: f32, deadzone: f32) -> f32 {
        let val = raw.max(0.0);
        if val <= deadzone {
            0.0
        } else {
            ((val - deadzone) / (1.0 - deadzone)).clamp(0.0, 1.0)
        }
    }

    /// Injects simulated gamepad state (for unit testing and headless emulation).
    pub fn inject_snapshot(&mut self, snapshot: GamepadSnapshot) {
        self.snapshot = snapshot;
    }
}
