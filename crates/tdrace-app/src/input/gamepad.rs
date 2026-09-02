use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};
use serde::{Deserialize, Serialize};

pub use cabinet::input::gamepad::{GamepadConfig, GamepadSnapshot};

/// Optional loaded gamepad profile mapping from `gamepad-mapper`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomGamepadProfile {
    #[serde(default)]
    pub device_name: String,
    #[serde(default, alias = "left_stick_x")]
    pub steering: Option<CustomAxisBinding>,
    #[serde(default, alias = "right_trigger")]
    pub throttle: Option<CustomTriggerBinding>,
    #[serde(default, alias = "left_trigger")]
    pub brake: Option<CustomTriggerBinding>,
    #[serde(default)]
    pub handbrake: Option<CustomButtonBinding>,
    #[serde(default)]
    pub reverse: Option<CustomButtonBinding>,
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
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CustomButtonBinding {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub alternate: Option<String>,
}

/// Cross-platform Gamepad Manager supporting hot-plugging, analog axes, and button events.
pub struct GamepadController {
    gilrs: Option<Gilrs>,
    pub config: GamepadConfig,
    pub active_gamepad: Option<GamepadId>,
    pub snapshot: GamepadSnapshot,
    pub custom_profile: Option<CustomGamepadProfile>,
    profile_path: Option<std::path::PathBuf>,
    profile_last_modified: Option<std::time::SystemTime>,
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
            Err(err) => {
                println!("[Gamepad] Gilrs initialization notice: {err} (falling back to keyboard/touch)");
                (None, None, "Gamepad Unavailable".to_string(), false)
            }
        };

        let mut snapshot = GamepadSnapshot::default();
        snapshot.is_connected = is_connected;
        snapshot.gamepad_name = gamepad_name;

        let loaded = Self::find_and_load_profile();
        let mut config = GamepadConfig::default();
        let mut custom_profile = None;
        let mut profile_path = None;
        let mut profile_last_modified = None;

        if let Some((prof, path, mtime)) = loaded {
            println!("[Gamepad] Loaded newest mapping profile on startup from {:?}", path);
            Self::sync_to_config_dir(&path);
            if let Some(ref st) = prof.steering {
                if st.deadzone > 0.0 {
                    config.stick_deadzone = st.deadzone;
                }
                if st.scale > 0.0 {
                    config.steer_scale = st.scale;
                }
            }
            if let Some(ref th) = prof.throttle {
                if th.deadzone > 0.0 {
                    config.trigger_deadzone = th.deadzone;
                }
            }
            custom_profile = Some(prof);
            profile_path = Some(path);
            profile_last_modified = Some(mtime);
        }

        Self {
            gilrs,
            config,
            active_gamepad,
            snapshot,
            custom_profile,
            profile_path,
            profile_last_modified,
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

    /// Candidate search paths for gamepad mapping profiles in order of priority.
    pub fn candidate_profile_paths() -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        // 1. Working directory
        paths.push(std::path::PathBuf::from("gamepad_profile.json"));
        // 2. Sibling directory ../gamepad-mapper/gamepad_profile.json
        paths.push(std::path::PathBuf::from("../gamepad-mapper/gamepad_profile.json"));
        // 3. User config ~/.config/tdrace/gamepad_profile.json
        if let Some(home) = std::env::var_os("HOME") {
            let mut p = std::path::PathBuf::from(home);
            p.push(".config");
            p.push("tdrace");
            p.push("gamepad_profile.json");
            paths.push(p);
        }
        paths
    }

    /// Finds and parses the most recently modified profile among all candidate locations.
    pub fn find_and_load_profile() -> Option<(CustomGamepadProfile, std::path::PathBuf, std::time::SystemTime)> {
        let mut newest: Option<(CustomGamepadProfile, std::path::PathBuf, std::time::SystemTime)> = None;

        for path in Self::candidate_profile_paths() {
            if path.exists() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(profile) = serde_json::from_str::<CustomGamepadProfile>(&content) {
                                let is_newer = match &newest {
                                    Some((_, _, newest_mtime)) => modified > *newest_mtime,
                                    None => true,
                                };
                                if is_newer {
                                    newest = Some((profile, path, modified));
                                }
                            }
                        }
                    }
                }
            }
        }
        newest
    }

    /// Synchronizes/copies the given profile file to ~/.config/tdrace/gamepad_profile.json.
    pub fn sync_to_config_dir(src_path: &std::path::Path) {
        if let Some(home) = std::env::var_os("HOME") {
            let mut target_dir = std::path::PathBuf::from(home);
            target_dir.push(".config");
            target_dir.push("tdrace");
            let _ = std::fs::create_dir_all(&target_dir);
            let target_file = target_dir.join("gamepad_profile.json");

            // Only copy if source is different from destination
            if let (Ok(canonical_src), Ok(canonical_target)) = (src_path.canonicalize(), target_file.canonicalize()) {
                if canonical_src == canonical_target {
                    return;
                }
            }
            if src_path != target_file.as_path() {
                if let Ok(_) = std::fs::copy(src_path, &target_file) {
                    println!("[Gamepad] Synced newest mapping profile from {:?} to {:?}", src_path, target_file);
                }
            }
        }
    }

    /// Checks if a newer mapping profile exists on disk and dynamically reloads and syncs it.
    pub fn check_and_reload_profile(&mut self) {
        if let Some((profile, path, modified)) = Self::find_and_load_profile() {
            let need_reload = match (self.profile_path.as_ref(), self.profile_last_modified) {
                (Some(curr_path), Some(curr_mod)) => *curr_path != path || curr_mod < modified,
                _ => true,
            };

            if need_reload {
                println!("[Gamepad] Detected newer mapping profile at {:?}, reloading live!", path);
                Self::sync_to_config_dir(&path);
                if let Some(ref st) = profile.steering {
                    if st.deadzone > 0.0 {
                        self.config.stick_deadzone = st.deadzone;
                    }
                    if st.scale > 0.0 {
                        self.config.steer_scale = st.scale;
                    }
                }
                if let Some(ref th) = profile.throttle {
                    if th.deadzone > 0.0 {
                        self.config.trigger_deadzone = th.deadzone;
                    }
                }
                self.custom_profile = Some(profile);
                self.profile_path = Some(path);
                self.profile_last_modified = Some(modified);
            }
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

    /// Checks if a custom button binding or standard fallback is pressed (edge-triggered) this frame.
    fn is_binding_pressed_this_frame(
        binding: &Option<CustomButtonBinding>,
        pressed_codes: &[String],
        standard_pressed: bool,
    ) -> bool {
        if let Some(b) = binding {
            if pressed_codes.contains(&b.code) {
                return true;
            }
            if let Some(ref alt) = b.alternate {
                if pressed_codes.contains(alt) {
                    return true;
                }
            }
        }
        standard_pressed
    }

    /// Checks if a custom button binding or standard fallback is currently held down.
    fn is_binding_held(
        binding: &Option<CustomButtonBinding>,
        raw_buttons_held: &[String],
        gp: &gilrs::Gamepad,
        standard_held: bool,
    ) -> bool {
        if let Some(b) = binding {
            if raw_buttons_held.contains(&b.code) {
                return true;
            }
            if let Some(ref alt) = b.alternate {
                if raw_buttons_held.contains(alt) {
                    return true;
                }
            }
            if sample_input_value(gp, &b.code) > 0.5 {
                return true;
            }
            if let Some(ref alt) = b.alternate {
                if sample_input_value(gp, alt) > 0.5 {
                    return true;
                }
            }
        }
        standard_held
    }

    /// Polls and updates gamepad state, draining all hardware events.
    pub fn update(&mut self) {
        self.clear_frame_events();

        let Some(ref mut gilrs) = self.gilrs else {
            return;
        };

        // Auto-detect active connected gamepad if none is currently selected
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

        // Drain pending hardware events
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            self.active_gamepad = Some(id);
            self.snapshot.is_connected = true;
            if let Some(gp) = gilrs.connected_gamepad(id) {
                self.snapshot.gamepad_name = gp.name().to_string();
            }

            match event {
                EventType::Connected => {
                    if let Some(gp) = gilrs.connected_gamepad(id) {
                        println!("[Gamepad] Connected: {} (ID {:?})", gp.name(), id);
                    }
                }
                EventType::Disconnected => {
                    if self.active_gamepad == Some(id) {
                        self.active_gamepad = gilrs
                            .gamepads()
                            .find(|(_, gp)| gp.is_connected())
                            .map(|(next_id, _)| next_id);
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
                EventType::ButtonChanged(btn, val, code) => {
                    let code_str = if btn != Button::Unknown {
                        format!("{btn:?}")
                    } else {
                        format!("Btn_{code}")
                    };
                    if val > 0.5 {
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
                    } else if val < 0.2 {
                        self.raw_buttons_held.retain(|b| b != &code_str);
                    }
                }
                _ => {}
            }
        }

        // Track new button presses from raw_buttons_held transitions
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

        // Sample continuous analog axes and button states from active gamepad
        if let Some(id) = self.active_gamepad {
            let maybe_gp = gilrs.connected_gamepad(id).or_else(|| {
                gilrs.gamepads().find(|(gid, _)| *gid == id).map(|(_, gp)| gp)
            });

            if let Some(gp) = maybe_gp {
                self.snapshot.is_connected = true;
                self.snapshot.gamepad_name = gp.name().to_string();

                // State polling for continuous state
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

                // Edge-triggered fallback detection (guarantees detection across all driver types)
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

                // 1. Left Analog Stick (Steering + Menu Flick Navigation)
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

                let stick_steer =
                    Self::process_axis_deadzone(raw_stick_x, config.stick_deadzone, config.steer_exponent);
                steer = (stick_steer * config.steer_scale + raw_dpad_x).clamp(-1.0, 1.0);

                // 2. Throttle (Right Trigger or Custom Profile Binding)
                let raw_rt = if let Some(ref prof) = self.custom_profile {
                    if let Some(ref th) = prof.throttle {
                        sample_input_value(&gp, &th.primary_code)
                    } else {
                        let raw_rt_btn = gp.button_data(Button::RightTrigger2).map(|d| d.value()).unwrap_or(0.0);
                        let is_rt_pressed = if gp.is_pressed(Button::RightTrigger2) { 1.0 } else { 0.0 };
                        raw_rt_btn.max(is_rt_pressed)
                    }
                } else {
                    let raw_rt_btn = gp.button_data(Button::RightTrigger2).map(|d| d.value()).unwrap_or(0.0);
                    let is_rt_pressed = if gp.is_pressed(Button::RightTrigger2) { 1.0 } else { 0.0 };
                    raw_rt_btn.max(is_rt_pressed)
                };
                throttle =
                    Self::process_trigger_deadzone(raw_rt, config.trigger_deadzone).clamp(0.0, 1.0);

                // 3. Brake (Left Trigger or Custom Profile Binding)
                let raw_lt = if let Some(ref prof) = self.custom_profile {
                    if let Some(ref br) = prof.brake {
                        sample_input_value(&gp, &br.primary_code)
                    } else {
                        let raw_lt_btn = gp.button_data(Button::LeftTrigger2).map(|d| d.value()).unwrap_or(0.0);
                        let is_lt_pressed = if gp.is_pressed(Button::LeftTrigger2) { 1.0 } else { 0.0 };
                        raw_lt_btn.max(is_lt_pressed)
                    }
                } else {
                    let raw_lt_btn = gp.button_data(Button::LeftTrigger2).map(|d| d.value()).unwrap_or(0.0);
                    let is_lt_pressed = if gp.is_pressed(Button::LeftTrigger2) { 1.0 } else { 0.0 };
                    raw_lt_btn.max(is_lt_pressed)
                };
                brake =
                    Self::process_trigger_deadzone(raw_lt, config.trigger_deadzone).clamp(0.0, 1.0);

                // 4. Handbrake (Button A / South or Custom Profile Binding)
                handbrake = Self::is_binding_held(
                    &self.custom_profile.as_ref().and_then(|p| p.btn_south.clone().or_else(|| p.handbrake.clone())),
                    &self.raw_buttons_held,
                    &gp,
                    curr_south,
                );

                // 5. Reverse (Button X / West or Custom Profile Binding)
                reverse = Self::is_binding_held(
                    &self.custom_profile.as_ref().and_then(|p| p.btn_west.clone().or_else(|| p.reverse.clone())),
                    &self.raw_buttons_held,
                    &gp,
                    curr_west,
                );
            }
        }

        let is_a_pressed = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.btn_south.clone().or_else(|| p.handbrake.clone())),
            &pressed_codes,
            btn_south,
        );
        let is_b_pressed = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.btn_east.clone()),
            &pressed_codes,
            btn_east,
        );
        let is_x_pressed = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.btn_west.clone().or_else(|| p.reverse.clone())),
            &pressed_codes,
            btn_west,
        );
        let is_y_pressed = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.btn_north.clone()),
            &pressed_codes,
            btn_north,
        );
        let is_start_pressed = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.btn_start.clone()),
            &pressed_codes,
            btn_start,
        );
        let is_select_pressed = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.btn_select.clone()),
            &pressed_codes,
            btn_select,
        );
        let is_dpad_u = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.dpad_up.clone()),
            &pressed_codes,
            dpad_u,
        );
        let is_dpad_d = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.dpad_down.clone()),
            &pressed_codes,
            dpad_d,
        );
        let is_dpad_l = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.dpad_left.clone()),
            &pressed_codes,
            dpad_l,
        );
        let is_dpad_r = Self::is_binding_pressed_this_frame(
            &self.custom_profile.as_ref().and_then(|p| p.dpad_right.clone()),
            &pressed_codes,
            dpad_r,
        );

        self.snapshot.steer = steer;
        self.snapshot.throttle = throttle;
        self.snapshot.brake = brake;
        self.snapshot.handbrake = handbrake;
        self.snapshot.reverse = reverse;

        self.snapshot.btn_start_pressed = is_start_pressed;
        self.snapshot.btn_back_pressed = is_select_pressed;
        self.snapshot.btn_a_pressed = is_a_pressed;
        self.snapshot.btn_b_pressed = is_b_pressed;
        self.snapshot.btn_x_pressed = is_x_pressed;
        self.snapshot.btn_y_pressed = is_y_pressed;
        self.snapshot.dpad_up_pressed = is_dpad_u;
        self.snapshot.dpad_down_pressed = is_dpad_d;
        self.snapshot.dpad_left_pressed = is_dpad_l;
        self.snapshot.dpad_right_pressed = is_dpad_r;
        self.snapshot.btn_assist_toggle_pressed = thumb_r || is_select_pressed;
        self.snapshot.btn_cam_toggle_pressed = thumb_l;

        self.snapshot.nav_up = is_dpad_u || stick_up;
        self.snapshot.nav_down = is_dpad_d || stick_down;
        self.snapshot.nav_left = is_dpad_l || stick_left;
        self.snapshot.nav_right = is_dpad_r || stick_right;
        self.snapshot.btn_confirm_pressed = is_a_pressed || is_start_pressed;
        self.snapshot.btn_cancel_pressed = is_b_pressed || is_select_pressed;
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

/// Helper function to sample named axis or button value from Gilrs gamepad.
fn sample_input_value(gp: &gilrs::Gamepad, code: &str) -> f32 {
    match code {
        "RightTrigger2" => {
            let btn_val = gp.button_data(Button::RightTrigger2).map(|d| d.value()).unwrap_or(0.0);
            let is_p = if gp.is_pressed(Button::RightTrigger2) { 1.0 } else { 0.0 };
            btn_val.max(is_p)
        }
        "LeftTrigger2" => {
            let btn_val = gp.button_data(Button::LeftTrigger2).map(|d| d.value()).unwrap_or(0.0);
            let is_p = if gp.is_pressed(Button::LeftTrigger2) { 1.0 } else { 0.0 };
            btn_val.max(is_p)
        }
        "RightTrigger" => if gp.is_pressed(Button::RightTrigger) { 1.0 } else { 0.0 },
        "LeftTrigger" => if gp.is_pressed(Button::LeftTrigger) { 1.0 } else { 0.0 },
        "South" => if gp.is_pressed(Button::South) { 1.0 } else { 0.0 },
        "East" => if gp.is_pressed(Button::East) { 1.0 } else { 0.0 },
        "West" => if gp.is_pressed(Button::West) { 1.0 } else { 0.0 },
        "North" => if gp.is_pressed(Button::North) { 1.0 } else { 0.0 },
        "LeftStickX" => gp.axis_data(Axis::LeftStickX).map(|d| d.value()).unwrap_or(0.0),
        "LeftStickY" => gp.axis_data(Axis::LeftStickY).map(|d| d.value()).unwrap_or(0.0),
        "RightStickX" => gp.axis_data(Axis::RightStickX).map(|d| d.value()).unwrap_or(0.0),
        "RightStickY" => gp.axis_data(Axis::RightStickY).map(|d| d.value()).unwrap_or(0.0),
        "DPadUp" => if gp.is_pressed(Button::DPadUp) { 1.0 } else { 0.0 },
        "DPadDown" => if gp.is_pressed(Button::DPadDown) { 1.0 } else { 0.0 },
        "DPadLeft" => if gp.is_pressed(Button::DPadLeft) { 1.0 } else { 0.0 },
        "DPadRight" => if gp.is_pressed(Button::DPadRight) { 1.0 } else { 0.0 },
        "Start" => if gp.is_pressed(Button::Start) { 1.0 } else { 0.0 },
        "Select" => if gp.is_pressed(Button::Select) { 1.0 } else { 0.0 },
        _ => 0.0,
    }
}

