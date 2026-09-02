#[cfg(feature = "gamepad")]
use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};
use serde::{Deserialize, Serialize};

/// Configuration for Gamepad analog stick deadzones, trigger thresholds, and sensitivity curves.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GamepadConfig {
    /// Inner deadzone threshold for left analog stick [0.0 .. 1.0].
    pub stick_deadzone: f32,
    /// Inner deadzone threshold for analog triggers [0.0 .. 1.0].
    pub trigger_deadzone: f32,
    /// Sensitivity curve exponent for analog steering/turning (1.0 = linear, 1.15 = gentle center).
    pub steer_exponent: f32,
    /// Sensitivity scale for steering/turning.
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
    /// Handbrake button pressed (A / South or Right Bumper).
    pub handbrake: bool,
    /// Reverse button pressed (Y / North or Left Bumper).
    pub reverse: bool,

    // Button trigger events (pressed this frame)
    pub btn_start_pressed: bool,
    pub btn_back_pressed: bool,
    pub btn_a_pressed: bool,      // South (Confirm / Handbrake / Select)
    pub btn_b_pressed: bool,      // East (Cancel / Back)
    pub btn_x_pressed: bool,      // West (Action 1 / Mode toggle)
    pub btn_y_pressed: bool,      // North (Action 2 / Cam)
    pub dpad_up_pressed: bool,
    pub dpad_down_pressed: bool,
    pub dpad_left_pressed: bool,
    pub dpad_right_pressed: bool,
    pub btn_assist_toggle_pressed: bool,
    pub btn_cam_toggle_pressed: bool,

    // Navigational triggers (D-Pad OR Analog Stick flicks)
    pub nav_up: bool,
    pub nav_down: bool,
    pub nav_left: bool,
    pub nav_right: bool,
    pub btn_confirm_pressed: bool, // Universal Confirm (South / East / Start)
    pub btn_cancel_pressed: bool,  // Universal Cancel (East / South / Back)
}

/// Cross-platform Gamepad Manager supporting hot-plugging, analog axes, and button events.
pub struct GamepadManager {
    #[cfg(feature = "gamepad")]
    gilrs: Option<Gilrs>,
    pub config: GamepadConfig,
    #[cfg(feature = "gamepad")]
    pub active_gamepad: Option<GamepadId>,
    pub snapshot: GamepadSnapshot,
    pub raw_buttons_held: Vec<String>,
    prev_buttons_held: Vec<String>,
    prev_stick_x: f32,
    prev_stick_y: f32,
    prev_south: bool,
    prev_east: bool,
    prev_west: bool,
    prev_north: bool,
    prev_start: bool,
    prev_select: bool,
    prev_dpad_up: bool,
    prev_dpad_down: bool,
    prev_dpad_left: bool,
    prev_dpad_right: bool,
    prev_thumb_r: bool,
    prev_thumb_l: bool,
}

impl Default for GamepadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GamepadManager {
    /// Creates a new GamepadManager, attempting to initialize the native subsystem.
    pub fn new() -> Self {
        #[cfg(feature = "gamepad")]
        let (gilrs, active_gamepad, gamepad_name, is_connected) = match Gilrs::new() {
            Ok(g) => {
                let first_gamepad = g
                    .gamepads()
                    .find(|(_, gp)| gp.is_connected())
                    .or_else(|| g.gamepads().next())
                    .map(|(id, gp)| (id, gp.name().to_string()));
                let (active, name, connected) = if let Some((id, name)) = first_gamepad {
                    (Some(id), name, true)
                } else {
                    (None, "No Gamepad Connected".to_string(), false)
                };
                (Some(g), active, name, connected)
            }
            Err(_) => (None, None, "Gamepad Unavailable".to_string(), false),
        };

        #[cfg(not(feature = "gamepad"))]
        let (gamepad_name, is_connected) = ("Gamepad Disabled".to_string(), false);

        let mut snapshot = GamepadSnapshot::default();
        snapshot.is_connected = is_connected;
        snapshot.gamepad_name = gamepad_name;

        Self {
            #[cfg(feature = "gamepad")]
            gilrs,
            config: GamepadConfig::default(),
            #[cfg(feature = "gamepad")]
            active_gamepad,
            snapshot,
            raw_buttons_held: Vec::new(),
            prev_buttons_held: Vec::new(),
            prev_stick_x: 0.0,
            prev_stick_y: 0.0,
            prev_south: false,
            prev_east: false,
            prev_west: false,
            prev_north: false,
            prev_start: false,
            prev_select: false,
            prev_dpad_up: false,
            prev_dpad_down: false,
            prev_dpad_left: false,
            prev_dpad_right: false,
            prev_thumb_r: false,
            prev_thumb_l: false,
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
        self.snapshot.nav_up = false;
        self.snapshot.nav_down = false;
        self.snapshot.nav_left = false;
        self.snapshot.nav_right = false;
        self.snapshot.btn_confirm_pressed = false;
        self.snapshot.btn_cancel_pressed = false;
    }

    /// Polls and updates gamepad state, draining all hardware events.
    pub fn update(&mut self) {
        self.clear_frame_events();

        #[cfg(feature = "gamepad")]
        {
            let Some(ref mut gilrs) = self.gilrs else {
                return;
            };

            // Auto-detect active connected gamepad
            if self.active_gamepad.is_none()
                || self
                    .active_gamepad
                    .map_or(true, |id| gilrs.connected_gamepad(id).is_none())
            {
                if let Some((id, gp)) = gilrs
                    .gamepads()
                    .find(|(_, gp)| gp.is_connected())
                    .or_else(|| gilrs.gamepads().next())
                {
                    self.active_gamepad = Some(id);
                    self.snapshot.gamepad_name = gp.name().to_string();
                    self.snapshot.is_connected = true;
                } else {
                    self.active_gamepad = None;
                    self.snapshot.is_connected = false;
                    self.snapshot.gamepad_name = "No Gamepad Connected".to_string();
                }
            }

            let mut pressed_codes = Vec::new();
            let mut btn_start = false;
            let mut btn_select = false;
            let mut btn_south = false;
            let mut btn_east = false;
            let mut btn_west = false;
            let mut btn_north = false;
            let mut dpad_u = false;
            let mut dpad_d = false;
            let mut dpad_l = false;
            let mut dpad_r = false;
            let mut thumb_r = false;
            let mut thumb_l = false;

            while let Some(Event { id, event, .. }) = gilrs.next_event() {
                self.active_gamepad = Some(id);
                self.snapshot.is_connected = true;
                if let Some(gp) = gilrs.connected_gamepad(id) {
                    self.snapshot.gamepad_name = gp.name().to_string();
                }

                match event {
                    EventType::ButtonPressed(btn, code) => {
                        let code_str = if btn != Button::Unknown {
                            format!("{btn:?}")
                        } else {
                            format!("Btn_{code}")
                        };
                        if !self.raw_buttons_held.contains(&code_str) {
                            self.raw_buttons_held.push(code_str.clone());
                        }
                        pressed_codes.push(code_str);

                        match btn {
                            Button::Start => btn_start = true,
                            Button::Select => btn_select = true,
                            Button::South => btn_south = true,
                            Button::East => btn_east = true,
                            Button::West => btn_west = true,
                            Button::North => btn_north = true,
                            Button::DPadUp => dpad_u = true,
                            Button::DPadDown => dpad_d = true,
                            Button::DPadLeft => dpad_l = true,
                            Button::DPadRight => dpad_r = true,
                            Button::RightThumb => thumb_r = true,
                            Button::LeftThumb => thumb_l = true,
                            _ => {}
                        }
                    }
                    EventType::ButtonReleased(btn, code) => {
                        let code_str = if btn != Button::Unknown {
                            format!("{btn:?}")
                        } else {
                            format!("Btn_{code}")
                        };
                        self.raw_buttons_held.retain(|b| b != &code_str);
                    }
                    _ => {}
                }
            }

            for b in &self.raw_buttons_held {
                if !self.prev_buttons_held.contains(b) && !pressed_codes.contains(b) {
                    pressed_codes.push(b.clone());
                }
            }
            self.prev_buttons_held = self.raw_buttons_held.clone();

            let config = self.config;
            let mut steer = 0.0;
            let mut throttle = 0.0;
            let mut brake = 0.0;
            let mut handbrake = false;
            let mut reverse = false;
            let mut stick_up = false;
            let mut stick_down = false;
            let mut stick_left = false;
            let mut stick_right = false;

            if let Some(id) = self.active_gamepad {
                let maybe_gp = gilrs.connected_gamepad(id).or_else(|| {
                    gilrs.gamepads().find(|(gid, _)| *gid == id).map(|(_, gp)| gp)
                });

                if let Some(gp) = maybe_gp {
                    self.snapshot.is_connected = true;
                    self.snapshot.gamepad_name = gp.name().to_string();

                    let curr_south = gp.is_pressed(Button::South);
                    let curr_east = gp.is_pressed(Button::East);
                    let curr_west = gp.is_pressed(Button::West);
                    let curr_north = gp.is_pressed(Button::North);
                    let curr_start = gp.is_pressed(Button::Start);
                    let curr_select = gp.is_pressed(Button::Select);
                    let dpad_y_axis = gp.axis_data(Axis::DPadY).map(|d| d.value()).unwrap_or(0.0);
                    let dpad_x_axis = gp.axis_data(Axis::DPadX).map(|d| d.value()).unwrap_or(0.0);
                    let curr_dpad_u = gp.is_pressed(Button::DPadUp) || dpad_y_axis > 0.5;
                    let curr_dpad_d = gp.is_pressed(Button::DPadDown) || dpad_y_axis < -0.5;
                    let curr_dpad_l = gp.is_pressed(Button::DPadLeft) || dpad_x_axis < -0.5;
                    let curr_dpad_r = gp.is_pressed(Button::DPadRight) || dpad_x_axis > 0.5;
                    let curr_thumb_r = gp.is_pressed(Button::RightThumb);
                    let curr_thumb_l = gp.is_pressed(Button::LeftThumb);

                    if curr_south && !self.prev_south { btn_south = true; }
                    if curr_east && !self.prev_east { btn_east = true; }
                    if curr_west && !self.prev_west { btn_west = true; }
                    if curr_north && !self.prev_north { btn_north = true; }
                    if curr_start && !self.prev_start { btn_start = true; }
                    if curr_select && !self.prev_select { btn_select = true; }
                    if curr_dpad_u && !self.prev_dpad_up { dpad_u = true; }
                    if curr_dpad_d && !self.prev_dpad_down { dpad_d = true; }
                    if curr_dpad_l && !self.prev_dpad_left { dpad_l = true; }
                    if curr_dpad_r && !self.prev_dpad_right { dpad_r = true; }
                    if curr_thumb_r && !self.prev_thumb_r { thumb_r = true; }
                    if curr_thumb_l && !self.prev_thumb_l { thumb_l = true; }

                    self.prev_south = curr_south;
                    self.prev_east = curr_east;
                    self.prev_west = curr_west;
                    self.prev_north = curr_north;
                    self.prev_start = curr_start;
                    self.prev_select = curr_select;
                    self.prev_dpad_up = curr_dpad_u;
                    self.prev_dpad_down = curr_dpad_d;
                    self.prev_dpad_left = curr_dpad_l;
                    self.prev_dpad_right = curr_dpad_r;
                    self.prev_thumb_r = curr_thumb_r;
                    self.prev_thumb_l = curr_thumb_l;

                    let raw_stick_x = gp.axis_data(Axis::LeftStickX).map(|d| d.value()).unwrap_or(0.0);
                    let raw_stick_y = gp.axis_data(Axis::LeftStickY).map(|d| d.value()).unwrap_or(0.0);

                    stick_up = raw_stick_y > 0.45 && self.prev_stick_y <= 0.45;
                    stick_down = raw_stick_y < -0.45 && self.prev_stick_y >= -0.45;
                    stick_left = raw_stick_x < -0.45 && self.prev_stick_x >= -0.45;
                    stick_right = raw_stick_x > 0.45 && self.prev_stick_x <= 0.45;

                    self.prev_stick_x = raw_stick_x;
                    self.prev_stick_y = raw_stick_y;

                    let raw_dpad_x = if curr_dpad_r {
                        1.0
                    } else if curr_dpad_l {
                        -1.0
                    } else if dpad_x_axis.abs() > 0.2 {
                        dpad_x_axis.signum()
                    } else {
                        0.0
                    };

                    let stick_steer = Self::process_axis_deadzone(raw_stick_x, config.stick_deadzone, config.steer_exponent);
                    steer = (stick_steer * config.steer_scale + raw_dpad_x).clamp(-1.0, 1.0);

                    let raw_rt_btn = gp.button_data(Button::RightTrigger2).map(|d| d.value()).unwrap_or(0.0);
                    let is_rt_pressed = if gp.is_pressed(Button::RightTrigger2) { 1.0 } else { 0.0 };
                    throttle = Self::process_trigger_deadzone(raw_rt_btn.max(is_rt_pressed), config.trigger_deadzone).clamp(0.0, 1.0);

                    let raw_lt_btn = gp.button_data(Button::LeftTrigger2).map(|d| d.value()).unwrap_or(0.0);
                    let is_lt_pressed = if gp.is_pressed(Button::LeftTrigger2) { 1.0 } else { 0.0 };
                    brake = Self::process_trigger_deadzone(raw_lt_btn.max(is_lt_pressed), config.trigger_deadzone).clamp(0.0, 1.0);

                    handbrake = curr_south;
                    reverse = curr_west;
                }
            }

            self.snapshot.steer = steer;
            self.snapshot.throttle = throttle;
            self.snapshot.brake = brake;
            self.snapshot.handbrake = handbrake;
            self.snapshot.reverse = reverse;

            self.snapshot.btn_start_pressed = btn_start;
            self.snapshot.btn_back_pressed = btn_select;
            self.snapshot.btn_a_pressed = btn_south;
            self.snapshot.btn_b_pressed = btn_east;
            self.snapshot.btn_x_pressed = btn_west;
            self.snapshot.btn_y_pressed = btn_north;
            self.snapshot.dpad_up_pressed = dpad_u;
            self.snapshot.dpad_down_pressed = dpad_d;
            self.snapshot.dpad_left_pressed = dpad_l;
            self.snapshot.dpad_right_pressed = dpad_r;
            self.snapshot.btn_assist_toggle_pressed = thumb_r || btn_select;
            self.snapshot.btn_cam_toggle_pressed = thumb_l;

            self.snapshot.nav_up = dpad_u || stick_up;
            self.snapshot.nav_down = dpad_d || stick_down;
            self.snapshot.nav_left = dpad_l || stick_left;
            self.snapshot.nav_right = dpad_r || stick_right;
            self.snapshot.btn_confirm_pressed = btn_south || btn_start;
            self.snapshot.btn_cancel_pressed = btn_east || btn_select;
        }
    }

    /// Applies inner deadzone and non-linear power curve to analog stick [-1.0 .. 1.0].
    pub fn process_axis_deadzone(raw: f32, deadzone: f32, exponent: f32) -> f32 {
        let abs_val = raw.abs();
        if abs_val <= deadzone {
            0.0
        } else {
            let normalized = (abs_val - deadzone) / (1.0 - deadzone);
            raw.signum() * normalized.powf(exponent)
        }
    }

    /// Applies deadzone to trigger [0.0 .. 1.0].
    pub fn process_trigger_deadzone(raw: f32, deadzone: f32) -> f32 {
        if raw <= deadzone {
            0.0
        } else {
            (raw - deadzone) / (1.0 - deadzone)
        }
    }
}
