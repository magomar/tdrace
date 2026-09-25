#[cfg(feature = "gamepad")]
use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs, GilrsBuilder};
use serde::{Deserialize, Serialize};

/// Supplemental SDL gamecontrollerdb mappings for generic controllers and adapter revisions
/// not bundled in Gilrs' default database.
pub const SUPPLEMENTAL_SDL_MAPPINGS: &str = "\
03000000100800000100000010010000,Twin USB PS2 Adapter,a:b2,b:b1,back:b8,dpdown:h0.4,dpleft:h0.8,dpright:h0.2,dpup:h0.1,leftshoulder:b6,leftstick:b10,lefttrigger:b4,leftx:a0,lefty:a1,rightshoulder:b7,rightstick:b11,righttrigger:b5,rightx:a3,righty:a2,start:b9,x:b3,y:b0,platform:Linux,\n\
03000000100800000100000011010000,Twin USB PS2 Adapter,a:b2,b:b1,back:b8,dpdown:h0.4,dpleft:h0.8,dpright:h0.2,dpup:h0.1,leftshoulder:b6,leftstick:b10,lefttrigger:b4,leftx:a0,lefty:a1,rightshoulder:b7,rightstick:b11,righttrigger:b5,rightx:a3,righty:a2,start:b9,x:b3,y:b0,platform:Linux,\n\
03000000100800000100000010010000,shanwan Twin USB Joystick,a:b2,b:b1,back:b8,dpdown:h0.4,dpleft:h0.8,dpright:h0.2,dpup:h0.1,leftshoulder:b6,leftstick:b10,lefttrigger:b4,leftx:a0,lefty:a1,rightshoulder:b7,rightstick:b11,righttrigger:b5,rightx:a3,righty:a2,start:b9,x:b3,y:b0,platform:Linux,\n\
03000000100800000100000011010000,shanwan Twin USB Joystick,a:b2,b:b1,back:b8,dpdown:h0.4,dpleft:h0.8,dpright:h0.2,dpup:h0.1,leftshoulder:b6,leftstick:b10,lefttrigger:b4,leftx:a0,lefty:a1,rightshoulder:b7,rightstick:b11,righttrigger:b5,rightx:a3,righty:a2,start:b9,x:b3,y:b0,platform:Linux,\n\
";

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
    pub btn_rb_pressed: bool,      // Right Bumper / R1
    pub btn_lb_pressed: bool,      // Left Bumper / L1

    // Button held states (continuous)
    pub btn_a_down: bool,
    pub btn_b_down: bool,
    pub btn_x_down: bool,
    pub btn_y_down: bool,
    pub dpad_up_down: bool,
    pub dpad_down_down: bool,
    pub dpad_left_down: bool,
    pub dpad_right_down: bool,
    pub btn_rb_down: bool,
    pub btn_lb_down: bool,
    pub stick_y: f32,

    // Navigational triggers (D-Pad OR Analog Stick flicks)
    pub nav_up: bool,
    pub nav_down: bool,
    pub nav_left: bool,
    pub nav_right: bool,
    pub btn_confirm_pressed: bool, // Universal Confirm (South / East / Start)
    pub btn_cancel_pressed: bool,  // Universal Cancel (East / South / Back)
}

/// Optional loaded gamepad profile mapping from `gamepad-mapper`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomGamepadProfile {
    #[serde(default)]
    pub device_name: String,
    #[serde(default, alias = "left_stick_x")]
    pub steering: Option<CustomAxisBinding>,
    #[serde(default, alias = "left_stick_y")]
    pub stick_y: Option<CustomAxisBinding>,
    #[serde(default)]
    pub right_stick_x: Option<CustomAxisBinding>,
    #[serde(default)]
    pub right_stick_y: Option<CustomAxisBinding>,
    #[serde(default, alias = "right_trigger")]
    pub throttle: Option<CustomTriggerBinding>,
    #[serde(default, alias = "left_trigger")]
    pub brake: Option<CustomTriggerBinding>,
    #[serde(default, alias = "btn_a")]
    pub btn_south: Option<CustomButtonBinding>,
    #[serde(default, alias = "btn_b")]
    pub btn_east: Option<CustomButtonBinding>,
    #[serde(default, alias = "btn_x")]
    pub btn_west: Option<CustomButtonBinding>,
    #[serde(default, alias = "btn_y")]
    pub btn_north: Option<CustomButtonBinding>,
    #[serde(default)]
    pub bumper_left: Option<CustomButtonBinding>,
    #[serde(default)]
    pub bumper_right: Option<CustomButtonBinding>,
    #[serde(default)]
    pub dpad_up: Option<CustomButtonBinding>,
    #[serde(default)]
    pub dpad_down: Option<CustomButtonBinding>,
    #[serde(default)]
    pub dpad_left: Option<CustomButtonBinding>,
    #[serde(default)]
    pub dpad_right: Option<CustomButtonBinding>,
    #[serde(default)]
    pub btn_start: Option<CustomButtonBinding>,
    #[serde(default)]
    pub btn_select: Option<CustomButtonBinding>,
    #[serde(default)]
    pub stick_l3: Option<CustomButtonBinding>,
    #[serde(default)]
    pub stick_r3: Option<CustomButtonBinding>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomAxisBinding {
    #[serde(default)]
    pub axis_name: String,
    #[serde(default)]
    pub inverted: bool,
    #[serde(default)]
    pub deadzone: f32,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    pub fallback_btn_pos: Option<String>,
    #[serde(default)]
    pub fallback_btn_neg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomTriggerBinding {
    #[serde(default)]
    pub primary_code: String,
    #[serde(default)]
    pub is_axis: bool,
    #[serde(default)]
    pub inverted: bool,
    #[serde(default)]
    pub deadzone: f32,
    #[serde(default)]
    pub alternate_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomButtonBinding {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub alternate: Option<String>,
}

/// Cross-platform Gamepad Manager supporting hot-plugging, analog axes, and button events.
pub struct GamepadManager {
    #[cfg(feature = "gamepad")]
    gilrs: Option<Gilrs>,
    pub config: GamepadConfig,
    #[cfg(feature = "gamepad")]
    pub active_gamepad: Option<GamepadId>,
    pub snapshot: GamepadSnapshot,
    pub custom_profile: Option<CustomGamepadProfile>,
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
    prev_rb: bool,
    prev_lb: bool,
    raw_codes_held: Vec<u32>,
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
        let (gilrs, active_gamepad, gamepad_name, is_connected) = match GilrsBuilder::default()
            .add_mappings(SUPPLEMENTAL_SDL_MAPPINGS)
            .build()
        {
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
        snapshot.gamepad_name = gamepad_name.clone();

        let custom_profile = Self::find_and_load_profile_for_device(if is_connected {
            Some(&gamepad_name)
        } else {
            None
        });

        Self {
            #[cfg(feature = "gamepad")]
            gilrs,
            config: GamepadConfig::default(),
            #[cfg(feature = "gamepad")]
            active_gamepad,
            snapshot,
            custom_profile,
            raw_buttons_held: Vec::new(),
            raw_codes_held: Vec::new(),
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
            prev_rb: false,
            prev_lb: false,
        }
    }

    /// Candidate search paths for gamepad mapping profiles in order of priority.
    pub fn candidate_profile_paths() -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        // 1. Current working directory
        paths.push(std::path::PathBuf::from("gamepad_profile.json"));
        // 2. Sibling directories
        paths.push(std::path::PathBuf::from("../gamepad-mapper/gamepad_profile.json"));
        paths.push(std::path::PathBuf::from("../tdrace/gamepad_profile.json"));
        paths.push(std::path::PathBuf::from("../asteroids/gamepad_profile.json"));
        // 3. User config directories
        if let Some(home) = std::env::var_os("HOME") {
            let h = std::path::PathBuf::from(home);
            paths.push(h.join(".config").join("gamepad-mapper").join("gamepad_profile.json"));
            paths.push(h.join(".config").join("tdrace").join("gamepad_profile.json"));
            paths.push(h.join(".config").join("asteroids").join("gamepad_profile.json"));
        }
        paths
    }

    /// Finds and parses the most recently modified profile among candidate locations.
    pub fn find_and_load_profile() -> Option<CustomGamepadProfile> {
        Self::find_and_load_profile_for_device(None)
    }

    /// Finds and parses the best-matching profile for the specified device, prioritizing matched devices over generic profiles.
    pub fn find_and_load_profile_for_device(target_device: Option<&str>) -> Option<CustomGamepadProfile> {
        let mut candidates = Vec::new();

        for path in Self::candidate_profile_paths() {
            if path.exists() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(profile) = serde_json::from_str::<CustomGamepadProfile>(&content) {
                                candidates.push((profile, path, modified));
                            }
                        }
                    }
                }
            }
        }

        candidates.into_iter().max_by(|a, b| {
            let score_a = Self::profile_match_score(&a.0, target_device);
            let score_b = Self::profile_match_score(&b.0, target_device);
            score_a.cmp(&score_b).then_with(|| a.2.cmp(&b.2))
        }).map(|(profile, _, _)| profile)
    }

    fn profile_match_score(profile: &CustomGamepadProfile, target_device: Option<&str>) -> u8 {
        let dev = profile.device_name.trim();
        if dev.is_empty() {
            return 0;
        }
        if let Some(target) = target_device {
            let target_lower = target.to_lowercase();
            let dev_lower = dev.to_lowercase();
            if !target_lower.is_empty() && target_lower != "no gamepad connected" && target_lower != "gamepad unavailable" {
                if target_lower.contains(&dev_lower) || dev_lower.contains(&target_lower) {
                    return 2;
                }
            }
        }
        if dev != "Standard Gamepad" {
            1
        } else {
            0
        }
    }

    /// Checks if a custom button binding is currently held down.
    pub fn is_binding_active(
        binding: &Option<CustomButtonBinding>,
        raw_buttons_held: &[String],
    ) -> bool {
        if let Some(b) = binding {
            if raw_buttons_held.iter().any(|raw| {
                raw == &b.code
                    || raw.trim_start_matches("Btn_") == b.code.trim_start_matches("Btn_")
                    || b.alternate.as_ref().map_or(false, |alt| {
                        raw == alt || raw.trim_start_matches("Btn_") == alt.trim_start_matches("Btn_")
                    })
            }) {
                return true;
            }
        }
        false
    }

    /// Checks if a custom button binding was pressed this frame.
    pub fn is_binding_pressed(
        binding: &Option<CustomButtonBinding>,
        pressed_codes: &[String],
    ) -> bool {
        if let Some(b) = binding {
            if pressed_codes.iter().any(|p| {
                p == &b.code
                    || p.trim_start_matches("Btn_") == b.code.trim_start_matches("Btn_")
                    || b.alternate.as_ref().map_or(false, |alt| {
                        p == alt || p.trim_start_matches("Btn_") == alt.trim_start_matches("Btn_")
                    })
            }) {
                return true;
            }
        }
        false
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
        self.snapshot.btn_rb_pressed = false;
        self.snapshot.btn_lb_pressed = false;
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
            let mut btn_rb = false;
            let mut btn_lb = false;

            while let Some(Event { id, event, .. }) = gilrs.next_event() {
                self.active_gamepad = Some(id);
                self.snapshot.is_connected = true;
                if let Some(gp) = gilrs.connected_gamepad(id) {
                    self.snapshot.gamepad_name = gp.name().to_string();
                }

                match event {
                    EventType::ButtonPressed(btn, code) => {
                        let raw_code = code.into_u32();
                        let evdev_code = (raw_code & 0xFFFF) as u32;
                        if !self.raw_codes_held.contains(&evdev_code) {
                            self.raw_codes_held.push(evdev_code);
                        }
                        if !self.raw_codes_held.contains(&raw_code) {
                            self.raw_codes_held.push(raw_code);
                        }

                        let code_str = format!("Btn_{code}");
                        let raw_str = format!("Btn_{evdev_code}");
                        let display_code = format!("{code}");
                        for s in [&code_str, &raw_str, &display_code] {
                            if !self.raw_buttons_held.contains(s) {
                                self.raw_buttons_held.push(s.clone());
                            }
                            pressed_codes.push(s.clone());
                        }

                        if btn != Button::Unknown {
                            let btn_str = format!("{btn:?}");
                            if !self.raw_buttons_held.contains(&btn_str) {
                                self.raw_buttons_held.push(btn_str.clone());
                            }
                            pressed_codes.push(btn_str);
                        }

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
                            Button::RightTrigger => btn_rb = true,
                            Button::LeftTrigger => btn_lb = true,
                            _ => {}
                        }

                        // Linux evdev fallback for generic unmapped controllers
                        match evdev_code {
                            290 | 304 => btn_south = true,
                            289 | 305 => btn_east = true,
                            291 | 308 => btn_west = true,
                            288 | 307 => btn_north = true,
                            297 | 315 => btn_start = true,
                            296 | 314 => btn_select = true,
                            295 => btn_rb = true,
                            294 => btn_lb = true,
                            _ => {}
                        }
                    }
                    EventType::ButtonReleased(btn, code) => {
                        let raw_code = code.into_u32();
                        let evdev_code = (raw_code & 0xFFFF) as u32;
                        self.raw_codes_held.retain(|&c| c != evdev_code && c != raw_code);

                        let code_str = format!("Btn_{code}");
                        let raw_str = format!("Btn_{evdev_code}");
                        let display_code = format!("{code}");
                        let btn_str = format!("{btn:?}");
                        self.raw_buttons_held.retain(|b| b != &code_str && b != &raw_str && b != &display_code && b != &btn_str);
                    }
                    EventType::ButtonChanged(btn, val, code) => {
                        let raw_code = code.into_u32();
                        let evdev_code = (raw_code & 0xFFFF) as u32;
                        let code_str = format!("Btn_{code}");
                        let raw_str = format!("Btn_{evdev_code}");
                        let display_code = format!("{code}");
                        let btn_str = format!("{btn:?}");

                        if val > 0.5 {
                            if !self.raw_codes_held.contains(&evdev_code) {
                                self.raw_codes_held.push(evdev_code);
                            }
                            if !self.raw_codes_held.contains(&raw_code) {
                                self.raw_codes_held.push(raw_code);
                            }
                            for s in [&code_str, &raw_str, &display_code] {
                                if !self.raw_buttons_held.contains(s) {
                                    self.raw_buttons_held.push(s.clone());
                                }
                                pressed_codes.push(s.clone());
                            }
                            if btn != Button::Unknown {
                                if !self.raw_buttons_held.contains(&btn_str) {
                                    self.raw_buttons_held.push(btn_str.clone());
                                }
                                pressed_codes.push(btn_str);
                            }

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
                                Button::RightTrigger => btn_rb = true,
                                Button::LeftTrigger => btn_lb = true,
                                _ => {}
                            }

                            match evdev_code {
                                290 | 304 => btn_south = true,
                                289 | 305 => btn_east = true,
                                291 | 308 => btn_west = true,
                                288 | 307 => btn_north = true,
                                297 | 315 => btn_start = true,
                                296 | 314 => btn_select = true,
                                295 => btn_rb = true,
                                294 => btn_lb = true,
                                _ => {}
                            }
                        } else if val < 0.2 {
                            self.raw_codes_held.retain(|&c| c != evdev_code && c != raw_code);
                            self.raw_buttons_held.retain(|b| b != &code_str && b != &raw_str && b != &display_code && b != &btn_str);
                        }
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

            if let Some(ref prof) = self.custom_profile {
                if Self::is_binding_pressed(&prof.btn_south, &pressed_codes) { btn_south = true; }
                if Self::is_binding_pressed(&prof.btn_east, &pressed_codes) { btn_east = true; }
                if Self::is_binding_pressed(&prof.btn_west, &pressed_codes) { btn_west = true; }
                if Self::is_binding_pressed(&prof.btn_north, &pressed_codes) { btn_north = true; }
                if Self::is_binding_pressed(&prof.btn_start, &pressed_codes) { btn_start = true; }
                if Self::is_binding_pressed(&prof.btn_select, &pressed_codes) { btn_select = true; }
                if Self::is_binding_pressed(&prof.bumper_right, &pressed_codes) { btn_rb = true; }
                if Self::is_binding_pressed(&prof.bumper_left, &pressed_codes) { btn_lb = true; }
                if Self::is_binding_pressed(&prof.dpad_up, &pressed_codes) { dpad_u = true; }
                if Self::is_binding_pressed(&prof.dpad_down, &pressed_codes) { dpad_d = true; }
                if Self::is_binding_pressed(&prof.dpad_left, &pressed_codes) { dpad_l = true; }
                if Self::is_binding_pressed(&prof.dpad_right, &pressed_codes) { dpad_r = true; }
            }

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

            let mut btn_a_down = false;
            let mut btn_b_down = false;
            let mut btn_x_down = false;
            let mut btn_y_down = false;
            let mut dpad_up_down = false;
            let mut dpad_down_down = false;
            let mut dpad_left_down = false;
            let mut dpad_right_down = false;
            let mut btn_rb_down = false;
            let mut btn_lb_down = false;
            let mut stick_y = 0.0;

            if let Some(id) = self.active_gamepad {
                let maybe_gp = gilrs.connected_gamepad(id).or_else(|| {
                    gilrs.gamepads().find(|(gid, _)| *gid == id).map(|(_, gp)| gp)
                });

                if let Some(gp) = maybe_gp {
                    self.snapshot.is_connected = true;
                    self.snapshot.gamepad_name = gp.name().to_string();

                    let mut curr_south = gp.is_pressed(Button::South)
                        || self.raw_codes_held.contains(&290)
                        || self.raw_codes_held.contains(&304);
                    let mut curr_east = gp.is_pressed(Button::East)
                        || self.raw_codes_held.contains(&289)
                        || self.raw_codes_held.contains(&305);
                    let mut curr_west = gp.is_pressed(Button::West)
                        || self.raw_codes_held.contains(&291)
                        || self.raw_codes_held.contains(&308);
                    let mut curr_north = gp.is_pressed(Button::North)
                        || self.raw_codes_held.contains(&288)
                        || self.raw_codes_held.contains(&307);
                    let mut curr_start = gp.is_pressed(Button::Start)
                        || self.raw_codes_held.contains(&297)
                        || self.raw_codes_held.contains(&315);
                    let mut curr_select = gp.is_pressed(Button::Select)
                        || self.raw_codes_held.contains(&296)
                        || self.raw_codes_held.contains(&314);
                    let dpad_y_axis = gp.axis_data(Axis::DPadY).map(|d| d.value()).unwrap_or(0.0);
                    let dpad_x_axis = gp.axis_data(Axis::DPadX).map(|d| d.value()).unwrap_or(0.0);
                    let curr_dpad_u = gp.is_pressed(Button::DPadUp) || dpad_y_axis > 0.5;
                    let curr_dpad_d = gp.is_pressed(Button::DPadDown) || dpad_y_axis < -0.5;
                    let curr_dpad_l = gp.is_pressed(Button::DPadLeft) || dpad_x_axis < -0.5;
                    let curr_dpad_r = gp.is_pressed(Button::DPadRight) || dpad_x_axis > 0.5;
                    let curr_thumb_r = gp.is_pressed(Button::RightThumb);
                    let curr_thumb_l = gp.is_pressed(Button::LeftThumb);
                    let mut curr_rb = gp.is_pressed(Button::RightTrigger)
                        || self.raw_codes_held.contains(&295)
                        || self.raw_codes_held.contains(&311);
                    let mut curr_lb = gp.is_pressed(Button::LeftTrigger)
                        || self.raw_codes_held.contains(&294)
                        || self.raw_codes_held.contains(&310);

                    if let Some(ref prof) = self.custom_profile {
                        if Self::is_binding_active(&prof.btn_south, &self.raw_buttons_held) { curr_south = true; }
                        if Self::is_binding_active(&prof.btn_east, &self.raw_buttons_held) { curr_east = true; }
                        if Self::is_binding_active(&prof.btn_west, &self.raw_buttons_held) { curr_west = true; }
                        if Self::is_binding_active(&prof.btn_north, &self.raw_buttons_held) { curr_north = true; }
                        if Self::is_binding_active(&prof.btn_start, &self.raw_buttons_held) { curr_start = true; }
                        if Self::is_binding_active(&prof.btn_select, &self.raw_buttons_held) { curr_select = true; }
                        if Self::is_binding_active(&prof.bumper_right, &self.raw_buttons_held) { curr_rb = true; }
                        if Self::is_binding_active(&prof.bumper_left, &self.raw_buttons_held) { curr_lb = true; }
                    }

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
                    if curr_rb && !self.prev_rb { btn_rb = true; }
                    if curr_lb && !self.prev_lb { btn_lb = true; }

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
                    self.prev_rb = curr_rb;
                    self.prev_lb = curr_lb;

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
                    let is_rt_evdev = if self.raw_codes_held.contains(&293) { 1.0 } else { 0.0 };
                    let is_rt_profile = if let Some(ref prof) = self.custom_profile {
                        if let Some(ref th) = prof.throttle {
                            if self.raw_buttons_held.iter().any(|b| b == &th.primary_code || b.trim_start_matches("Btn_") == th.primary_code.trim_start_matches("Btn_")) {
                                1.0
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    let raw_rt = raw_rt_btn.max(is_rt_pressed).max(is_rt_evdev).max(is_rt_profile);
                    throttle = Self::process_trigger_deadzone(raw_rt, config.trigger_deadzone).clamp(0.0, 1.0);

                    let raw_lt_btn = gp.button_data(Button::LeftTrigger2).map(|d| d.value()).unwrap_or(0.0);
                    let is_lt_pressed = if gp.is_pressed(Button::LeftTrigger2) { 1.0 } else { 0.0 };
                    let is_lt_evdev = if self.raw_codes_held.contains(&292) { 1.0 } else { 0.0 };
                    let is_lt_profile = if let Some(ref prof) = self.custom_profile {
                        if let Some(ref br) = prof.brake {
                            if self.raw_buttons_held.iter().any(|b| b == &br.primary_code || b.trim_start_matches("Btn_") == br.primary_code.trim_start_matches("Btn_")) {
                                1.0
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    let raw_lt = raw_lt_btn.max(is_lt_pressed).max(is_lt_evdev).max(is_lt_profile);
                    brake = Self::process_trigger_deadzone(raw_lt, config.trigger_deadzone).clamp(0.0, 1.0);

                    handbrake = curr_south;
                    reverse = curr_west;

                    btn_a_down = curr_south;
                    btn_b_down = curr_east;
                    btn_x_down = curr_west;
                    btn_y_down = curr_north;
                    dpad_up_down = curr_dpad_u;
                    dpad_down_down = curr_dpad_d;
                    dpad_left_down = curr_dpad_l;
                    dpad_right_down = curr_dpad_r;
                    btn_rb_down = curr_rb;
                    btn_lb_down = curr_lb;
                    stick_y = raw_stick_y;
                }
            }

            self.snapshot.steer = steer;
            self.snapshot.throttle = throttle;
            self.snapshot.brake = brake;
            self.snapshot.handbrake = handbrake;
            self.snapshot.reverse = reverse;

            self.snapshot.btn_a_down = btn_a_down;
            self.snapshot.btn_b_down = btn_b_down;
            self.snapshot.btn_x_down = btn_x_down;
            self.snapshot.btn_y_down = btn_y_down;
            self.snapshot.dpad_up_down = dpad_up_down;
            self.snapshot.dpad_down_down = dpad_down_down;
            self.snapshot.dpad_left_down = dpad_left_down;
            self.snapshot.dpad_right_down = dpad_right_down;
            self.snapshot.btn_rb_pressed = btn_rb;
            self.snapshot.btn_rb_down = btn_rb_down;
            self.snapshot.btn_lb_pressed = btn_lb;
            self.snapshot.btn_lb_down = btn_lb_down;
            self.snapshot.stick_y = stick_y;

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

    /// Checks whether a Gilrs button is currently held down on the active gamepad.
    pub fn is_button_down(&self, btn: Button) -> bool {
        #[cfg(feature = "gamepad")]
        {
            if let (Some(ref gilrs), Some(id)) = (&self.gilrs, self.active_gamepad) {
                if let Some(gp) = gilrs.connected_gamepad(id) {
                    return gp.is_pressed(btn);
                }
            }
        }
        false
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_gamepad_profile_loading() {
        let profile = GamepadManager::find_and_load_profile_for_device(Some("shanwan Twin USB Joystick"));
        assert!(profile.is_some(), "Custom profile should be found in candidate paths");
        let prof = profile.unwrap();
        assert_eq!(prof.device_name, "shanwan Twin USB Joystick");
        assert!(prof.btn_south.is_some());
        assert_eq!(prof.btn_south.as_ref().unwrap().code, "Btn_KEY(290)");
        assert!(prof.right_stick_x.is_some());
        assert_eq!(prof.right_stick_x.as_ref().unwrap().axis_name, "LeftZ");
        assert!(prof.right_stick_y.is_some());
        assert_eq!(prof.right_stick_y.as_ref().unwrap().axis_name, "RightZ");
        assert!(prof.right_stick_y.as_ref().unwrap().inverted);
    }

    #[test]
    fn test_binding_active_matching() {
        let binding = Some(CustomButtonBinding {
            code: "Btn_KEY(290)".to_string(),
            alternate: None,
        });
        // Test matching with Btn_KEY(290)
        assert!(GamepadManager::is_binding_active(&binding, &["Btn_KEY(290)".to_string()]));
        // Test matching with KEY(290)
        assert!(GamepadManager::is_binding_active(&binding, &["KEY(290)".to_string()]));
    }

    #[test]
    fn test_hardware_gamepad_presence() {
        let gm = GamepadManager::new();
        println!("GM is_connected: {}", gm.snapshot.is_connected);
        println!("GM gamepad_name: {}", gm.snapshot.gamepad_name);
        println!("GM custom_profile: {:?}", gm.custom_profile.as_ref().map(|p| &p.device_name));
        #[cfg(feature = "gamepad")]
        if let Some(ref g) = gm.gilrs {
            for (id, gp) in g.gamepads() {
                println!("Found gamepad {:?}: name='{}', connected={}, power_info={:?}", id, gp.name(), gp.is_connected(), gp.power_info());
            }
        }
    }
}


