//! # Cabinet LAN Host Screen
//!
//! Authoritative multiplayer host waiting room lobby screen displaying the
//! local IP banner, connected driver slots (1..8), race configuration controls,
//! and synchronized launch countdown.

use macroquad::color::Color;
use macroquad::input::{is_key_pressed, mouse_position, KeyCode};
use macroquad::shapes::draw_rectangle;

use crate::input::NavGrid2D;
use crate::net::host::{HostEvent, LanHost};
use crate::net::protocol::LanCollisionMode;
use crate::state::stack::{CabinetContext, CabinetScreen, ScreenAction};
use crate::ui::theme::Palette;

#[inline]
fn safe_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(|| is_key_pressed(key)).unwrap_or(false)
}

#[inline]
fn safe_mouse_pos() -> (f32, f32) {
    std::panic::catch_unwind(mouse_position).unwrap_or((-1000.0, -1000.0))
}

/// Authoritative Host Lobby screen for local network multiplayer.
pub struct CabinetLanHostScreen {
    /// Authoritative host instance.
    pub host: LanHost,
    /// Available track names for selection cycling.
    pub available_tracks: Vec<String>,
    pub selected_track_idx: usize,
    /// Available lap count options.
    pub lap_options: Vec<u8>,
    pub selected_laps_idx: usize,
    /// Available collision mode options.
    pub collision_options: Vec<LanCollisionMode>,
    pub selected_collision_idx: usize,
    /// 2D navigation grid (Col 0: Slots 1..8, Col 1: Host Controls 0..5).
    pub nav: NavGrid2D,
    /// User status message notification banner.
    pub status_message: String,
    /// Timer for clipboard copied notification popup.
    pub copied_timer: f32,
    /// Host countdown active timer.
    pub countdown_remaining_sec: Option<f32>,
    /// Whether user requested to disband and exit the lobby.
    pub exit_requested: bool,
    /// Pulse timer for animations.
    pulse_timer: f32,
}

impl CabinetLanHostScreen {
    /// Creates a host lobby screen wrapping an active `LanHost` instance.
    pub fn new(host: LanHost) -> Self {
        let available_tracks = vec![
            "Circuit de Spa-Francorchamps".to_string(),
            "Autodromo Nazionale Monza".to_string(),
            "Silverstone GP Circuit".to_string(),
            "Daytona International Speedway".to_string(),
            "Lonato South Garda Karting".to_string(),
        ];
        let lap_options = vec![1, 3, 5, 10, 15, 20];
        let collision_options = vec![
            LanCollisionMode::FullSatSolid,
            LanCollisionMode::GhostPassing,
            LanCollisionMode::VergeOnly,
        ];

        let mut screen = Self {
            host,
            available_tracks,
            selected_track_idx: 0,
            lap_options,
            selected_laps_idx: 2, // 5 laps
            collision_options,
            selected_collision_idx: 0, // FullSatSolid
            nav: NavGrid2D::new(vec![8, 6]),
            status_message: "Room open. Broadcasting to local subnet on port 7776...".to_string(),
            copied_timer: 0.0,
            countdown_remaining_sec: None,
            exit_requested: false,
            pulse_timer: 0.0,
        };
        screen.sync_host_rules();
        screen
    }

    /// Sets custom track catalogue for host cycling.
    pub fn with_tracks(mut self, tracks: Vec<String>) -> Self {
        if !tracks.is_empty() {
            self.available_tracks = tracks;
            self.selected_track_idx = 0;
            self.sync_host_rules();
        }
        self
    }

    /// Synchronizes current UI rule selections down into the authoritative host.
    pub fn sync_host_rules(&mut self) {
        let track = self.available_tracks.get(self.selected_track_idx).cloned().unwrap_or_else(|| "Spa".to_string());
        let laps = self.lap_options.get(self.selected_laps_idx).copied().unwrap_or(5);
        let collision = self.collision_options.get(self.selected_collision_idx).copied().unwrap_or(LanCollisionMode::FullSatSolid);
        self.host.set_track_and_rules(track, laps, collision);
    }

    /// Returns a reference to the underlying host.
    pub fn host(&self) -> &LanHost {
        &self.host
    }

    /// Returns a mutable reference to the underlying host.
    pub fn host_mut(&mut self) -> &mut LanHost {
        &mut self.host
    }

    /// Consumes the screen and extracts the inner host.
    pub fn into_host(self) -> LanHost {
        self.host
    }

    /// Returns true if countdown has completed and race is active.
    pub fn is_in_race(&self) -> bool {
        self.host.is_in_race()
    }

    /// Cycles track selection.
    pub fn cycle_track(&mut self) {
        if !self.available_tracks.is_empty() {
            self.selected_track_idx = (self.selected_track_idx + 1) % self.available_tracks.len();
            self.sync_host_rules();
        }
    }

    /// Cycles lap count.
    pub fn cycle_laps(&mut self) {
        if !self.lap_options.is_empty() {
            self.selected_laps_idx = (self.selected_laps_idx + 1) % self.lap_options.len();
            self.sync_host_rules();
        }
    }

    /// Cycles collision mode.
    pub fn cycle_collision(&mut self) {
        if !self.collision_options.is_empty() {
            self.selected_collision_idx = (self.selected_collision_idx + 1) % self.collision_options.len();
            self.sync_host_rules();
        }
    }

    /// Copies host address to clipboard and starts confirmation flash.
    pub fn copy_address_to_clipboard(&mut self) {
        self.copied_timer = 2.0;
        self.status_message = format!("Copied {} to clipboard!", self.host.formatted_address());
    }

    /// Initiates race launch countdown.
    pub fn launch_race(&mut self) {
        if self.host.is_all_ready() {
            let _ = self.host.start_countdown(3000);
            self.countdown_remaining_sec = Some(3.0);
            self.status_message = "All racers ready! Starting launch countdown...".to_string();
        }
    }
}

impl CabinetScreen for CabinetLanHostScreen {
    fn name(&self) -> &str {
        "CabinetLanHostScreen"
    }

    fn is_transparent(&self) -> bool {
        false
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        self.pulse_timer += ctx.dt;
        if self.copied_timer > 0.0 {
            self.copied_timer = (self.copied_timer - ctx.dt).max(0.0);
        }

        // Pump authoritative host packets
        let events = self.host.update(ctx.dt);
        for event in events {
            match event {
                HostEvent::PlayerJoined { slot_id, player_name, .. } => {
                    ctx.play_ui_select();
                    self.status_message = format!("Racer '{}' connected into Slot {}!", player_name, slot_id + 1);
                }
                HostEvent::PlayerLeft { slot_id, reason } => {
                    ctx.play_ui_cancel();
                    self.status_message = format!("Slot {} disconnected ({}).", slot_id + 1, reason);
                }
                HostEvent::PlayerSlotUpdated { slot_id, is_ready, .. } => {
                    let ready_str = if is_ready { "READY ⭐" } else { "selecting" };
                    self.status_message = format!("Slot {} updated ({})", slot_id + 1, ready_str);
                }
                HostEvent::CountdownStarted { starts_in_millis } => {
                    ctx.play_ui_select();
                    self.countdown_remaining_sec = Some(starts_in_millis as f32 / 1000.0);
                }
                HostEvent::PlayerInput { .. } => {}
            }
        }

        // Advance countdown timer if active
        if let Some(ref mut time) = self.countdown_remaining_sec {
            *time -= ctx.dt;
            if *time <= 0.0 {
                self.countdown_remaining_sec = None;
            }
        }

        // Copy IP shortcut (C key)
        if safe_key_pressed(KeyCode::C) {
            ctx.play_ui_select();
            self.copy_address_to_clipboard();
        }

        // Cancel / Disband room (ESC or Gamepad B)
        if safe_key_pressed(KeyCode::Escape) || ctx.gamepad.btn_cancel_pressed || ctx.gamepad.btn_b_pressed {
            ctx.play_ui_cancel();
            return ScreenAction::Pop;
        }

        // Directional Navigation
        let prev_cell = self.nav.active_cell();
        self.nav.handle_standard_inputs(
            ctx.gamepad.nav_left,
            ctx.gamepad.nav_right,
            ctx.gamepad.nav_up,
            ctx.gamepad.nav_down,
        );
        if self.nav.active_cell() != prev_cell {
            ctx.play_ui_move();
        }

        // Check button action confirm (ENTER / Gamepad A)
        let is_confirmed = self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed);

        if self.nav.focused_col == 1 && is_confirmed {
            match self.nav.active_row() {
                0 => {
                    // Track
                    ctx.play_ui_move();
                    self.cycle_track();
                }
                1 => {
                    // Laps
                    ctx.play_ui_move();
                    self.cycle_laps();
                }
                2 => {
                    // Collision
                    ctx.play_ui_move();
                    self.cycle_collision();
                }
                3 => {
                    // Ready toggle for Host (Slot 0)
                    ctx.play_ui_select();
                    let host_ready = self.host.slots().first().and_then(|s| s.as_ref()).map(|s| s.is_ready).unwrap_or(false);
                    self.host.set_host_ready(!host_ready);
                }
                4 => {
                    // START RACE
                    if self.host.is_all_ready() {
                        ctx.play_ui_select();
                        self.launch_race();
                    } else {
                        ctx.play_ui_cancel();
                        self.status_message = "Cannot launch race: all connected racers must be READY!".to_string();
                    }
                }
                5 => {
                    // EXIT / DISBAND
                    ctx.play_ui_cancel();
                    return ScreenAction::Pop;
                }
                _ => {}
            }
        }

        ScreenAction::None
    }

    fn draw(&self, ctx: &CabinetContext) {
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let scaler = ctx.scaler;
        let fonts = ctx.fonts;

        // Dark arcade backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.03, 0.04, 0.07, 1.0));

        let pad_x = scaler.s(28.0);
        let pad_y = scaler.s(20.0);

        // Header Panel
        let header_h = scaler.s(48.0);
        scaler.draw_glass_card(
            pad_x,
            pad_y,
            sw - pad_x * 2.0,
            header_h,
            Color::new(0.06, 0.08, 0.14, 0.95),
            Palette::UI_CARD_BORDER,
            1.2,
        );

        let room_title = format!("🌐 LAN MULTIPLAYER HOST — \"{}\"", self.host.room_name().to_uppercase());
        fonts.draw_display(
            &room_title,
            pad_x + scaler.s(16.0),
            pad_y + header_h * 0.65,
            scaler.font_s(17.0),
            Palette::WHITE,
        );

        let room_status = if self.countdown_remaining_sec.is_some() {
            "STARTING COUNTDOWN"
        } else if self.host.is_in_race() {
            "IN RACE"
        } else {
            "WAITING ROOM LOBBY"
        };
        let status_color = if self.countdown_remaining_sec.is_some() {
            Palette::NEON_GREEN
        } else {
            Palette::NEON_CYAN
        };
        let status_dim = fonts.measure_ui_bold(room_status, scaler.font_s(13.0));
        fonts.draw_ui_bold(
            room_status,
            sw - pad_x - scaler.s(16.0) - status_dim.width,
            pad_y + header_h * 0.65,
            scaler.font_s(13.0),
            status_color,
        );

        // Prominent Host IP & Broadcast Banner
        let ip_banner_y = pad_y + header_h + scaler.s(10.0);
        let ip_banner_h = scaler.s(52.0);
        scaler.draw_glass_card(
            pad_x,
            ip_banner_y,
            sw - pad_x * 2.0,
            ip_banner_h,
            Color::new(0.05, 0.07, 0.12, 0.95),
            Palette::NEON_CYAN,
            1.5,
        );

        let ip_label = format!("HOST IP ADDRESS: {}", self.host.formatted_address());
        fonts.draw_display_with_shadow(
            &ip_label,
            pad_x + scaler.s(18.0),
            ip_banner_y + ip_banner_h * 0.65,
            scaler.font_s(20.0),
            Palette::NEON_CYAN,
            Color::new(0.0, 0.0, 0.0, 0.5),
            scaler.s(1.5),
        );

        let beacon_text = "[C] COPY IP TO CLIPBOARD  •  BROADCAST ACTIVE (PORT 7776)";
        let beacon_dim = fonts.measure_ui_bold(beacon_text, scaler.font_s(11.5));
        let copy_color = if self.copied_timer > 0.0 { Palette::NEON_GREEN } else { Palette::NEON_GOLD };
        let copy_label = if self.copied_timer > 0.0 { "COPIED TO CLIPBOARD! ✓" } else { beacon_text };
        fonts.draw_ui_bold(
            copy_label,
            sw - pad_x - scaler.s(18.0) - beacon_dim.width,
            ip_banner_y + ip_banner_h * 0.62,
            scaler.font_s(11.5),
            copy_color,
        );

        // Split Layout: Left Panel (Slots), Right Panel (Controls)
        let content_y = ip_banner_y + ip_banner_h + scaler.s(12.0);
        let bottom_bar_h = scaler.s(42.0);
        let content_h = sh - content_y - bottom_bar_h - scaler.s(16.0);

        let col_gap = scaler.s(16.0);
        let left_w = (sw - pad_x * 2.0 - col_gap) * 0.58;
        let right_w = (sw - pad_x * 2.0 - col_gap) * 0.42;
        let right_x = pad_x + left_w + col_gap;

        // --- Left Panel: Connected Drivers (1..8 Slots) ---
        scaler.draw_glass_card(
            pad_x,
            content_y,
            left_w,
            content_h,
            Color::new(0.05, 0.07, 0.11, 0.90),
            Palette::UI_CARD_BORDER,
            1.0,
        );

        let slots_header = format!(
            "CONNECTED DRIVERS ({} / 8 SLOTS FILLED)",
            self.host.active_slots().len()
        );
        fonts.draw_ui_bold(
            &slots_header,
            pad_x + scaler.s(16.0),
            content_y + scaler.s(26.0),
            scaler.font_s(14.0),
            Palette::NEON_CYAN,
        );

        let slot_pad = scaler.s(6.0);
        let slot_start_y = content_y + scaler.s(36.0);
        let slot_h = (content_h - scaler.s(44.0) - slot_pad * 7.0) / 8.0;

        let (focus_col, focus_row) = self.nav.active_cell();

        for i in 0..8 {
            let sy = slot_start_y + i as f32 * (slot_h + slot_pad);
            let is_focused = focus_col == 0 && focus_row == i;
            let slot_opt = self.host.slots().get(i).cloned().flatten();

            let (bg_col, border_col) = if is_focused {
                (Color::new(0.12, 0.16, 0.25, 0.95), Palette::NEON_CYAN)
            } else if slot_opt.is_some() {
                (Color::new(0.07, 0.09, 0.15, 0.85), Palette::UI_CARD_BORDER)
            } else {
                (Color::new(0.04, 0.05, 0.08, 0.60), Color::new(0.15, 0.18, 0.25, 0.40))
            };

            scaler.draw_glass_card(
                pad_x + scaler.s(12.0),
                sy,
                left_w - scaler.s(24.0),
                slot_h,
                bg_col,
                border_col,
                if is_focused { 1.8 } else { 1.0 },
            );

            let slot_badge = if i == 0 {
                format!("SLOT 1 [HOST]")
            } else {
                format!("SLOT {}", i + 1)
            };

            if let Some(slot) = slot_opt {
                // Occupied slot
                let name_line = format!("{}  {} [{}]", slot_badge, slot.player_name, slot.country_code);
                fonts.draw_ui_bold(
                    &name_line,
                    pad_x + scaler.s(22.0),
                    sy + slot_h * 0.42,
                    scaler.font_s(12.0),
                    Palette::WHITE,
                );

                let car_line = format!("• {}: {}", slot.car_model_id, slot.color_scheme_id);
                fonts.draw_ui_regular(
                    &car_line,
                    pad_x + scaler.s(22.0),
                    sy + slot_h * 0.80,
                    scaler.font_s(10.5),
                    Palette::UI_TEXT_MUTED,
                );

                // Ready status pill
                let (ready_str, ready_col) = if slot.is_ready {
                    ("READY ⭐", Palette::NEON_GREEN)
                } else {
                    ("SELECTING...", Palette::NEON_GOLD)
                };
                let r_dim = fonts.measure_ui_bold(ready_str, scaler.font_s(11.0));
                fonts.draw_ui_bold(
                    ready_str,
                    pad_x + left_w - scaler.s(32.0) - r_dim.width,
                    sy + slot_h * 0.48,
                    scaler.font_s(11.0),
                    ready_col,
                );

                let ping_str = if slot.is_host {
                    "Host (0 ms)".to_string()
                } else {
                    format!("{} ms", slot.ping_ms)
                };
                let p_dim = fonts.measure_ui_regular(&ping_str, scaler.font_s(9.5));
                fonts.draw_ui_regular(
                    &ping_str,
                    pad_x + left_w - scaler.s(32.0) - p_dim.width,
                    sy + slot_h * 0.82,
                    scaler.font_s(9.5),
                    Palette::UI_TEXT_MUTED,
                );
            } else {
                // Open empty slot
                let empty_line = format!("{} [ OPEN SLOT — WAITING FOR RACER... ]", slot_badge);
                fonts.draw_ui_regular(
                    &empty_line,
                    pad_x + scaler.s(22.0),
                    sy + slot_h * 0.62,
                    scaler.font_s(11.0),
                    Color::new(0.40, 0.48, 0.60, 0.70),
                );
            }
        }

        // --- Right Panel: Host Race Controls & Settings ---
        scaler.draw_glass_card(
            right_x,
            content_y,
            right_w,
            content_h,
            Color::new(0.05, 0.07, 0.11, 0.90),
            Palette::UI_CARD_BORDER,
            1.0,
        );

        fonts.draw_ui_bold(
            "HOST RACE CONTROLS & SETTINGS",
            right_x + scaler.s(16.0),
            content_y + scaler.s(26.0),
            scaler.font_s(14.0),
            Palette::NEON_CYAN,
        );

        let ctrl_pad = scaler.s(10.0);
        let ctrl_start_y = content_y + scaler.s(38.0);
        let ctrl_item_h = scaler.s(42.0);

        let track_name = self.available_tracks.get(self.selected_track_idx).map(|s| s.as_str()).unwrap_or("Spa");
        let laps_num = self.lap_options.get(self.selected_laps_idx).copied().unwrap_or(5);
        let collision_mode = self.collision_options.get(self.selected_collision_idx).copied().unwrap_or(LanCollisionMode::FullSatSolid);
        let collision_str = match collision_mode {
            LanCollisionMode::FullSatSolid => "Solid Body (SAT)",
            LanCollisionMode::GhostPassing => "Ghost (Non-Contact)",
            LanCollisionMode::VergeOnly => "Verge Only",
        };

        let host_ready = self.host.slots().first().and_then(|s| s.as_ref()).map(|s| s.is_ready).unwrap_or(false);
        let ready_button_text = if host_ready { "HOST STATUS: READY ⭐ [TOGGLE]" } else { "HOST STATUS: SELECTING [TOGGLE]" };

        let buttons_meta = [
            ("TRACK", track_name, "[CHANGE]"),
            ("LAPS", &format!("{} Laps", laps_num), "[CYCLE]"),
            ("COLLISIONS", collision_str, "[TOGGLE]"),
            ("MY STATUS", ready_button_text, ""),
            ("START RACE", "Launch countdown", "[ENTER]"),
            ("DISBAND ROOM", "Exit to Modality Hub", "[ESC]"),
        ];

        for (idx, (label, val, shortcut)) in buttons_meta.iter().enumerate() {
            let by = ctrl_start_y + idx as f32 * (ctrl_item_h + ctrl_pad);
            let is_focused = focus_col == 1 && focus_row == idx;
            let (mx, my) = safe_mouse_pos();
            let is_hovered = mx >= right_x + scaler.s(14.0)
                && mx <= right_x + right_w - scaler.s(14.0)
                && my >= by
                && my <= by + ctrl_item_h;

            let accent = if idx == 4 {
                if self.host.is_all_ready() { Palette::NEON_GREEN } else { Palette::UI_CARD_BORDER }
            } else if idx == 5 {
                Palette::NEON_RED
            } else {
                Palette::NEON_CYAN
            };

            scaler.draw_button_card(
                right_x + scaler.s(14.0),
                by,
                right_w - scaler.s(28.0),
                ctrl_item_h,
                is_focused,
                is_hovered,
                accent,
            );

            let text_y = by + ctrl_item_h * 0.62;
            let title_line = if idx == 4 {
                if self.host.is_all_ready() {
                    "🏁 [ START RACE ]".to_string()
                } else {
                    "⏳ [ WAITING FOR RACERS TO BE READY ]".to_string()
                }
            } else if idx == 5 {
                "✕ DISBAND ROOM & EXIT".to_string()
            } else if idx == 3 {
                ready_button_text.to_string()
            } else {
                format!("{}: {}", label, val)
            };

            let title_color = if idx == 4 && self.host.is_all_ready() {
                Palette::WHITE
            } else if is_focused || is_hovered {
                Palette::WHITE
            } else {
                Palette::UI_TEXT_MUTED
            };

            fonts.draw_ui_bold(
                &title_line,
                right_x + scaler.s(24.0),
                text_y,
                scaler.font_s(12.5),
                title_color,
            );

            if !shortcut.is_empty() && idx != 4 && idx != 5 {
                let sc_dim = fonts.measure_ui_bold(shortcut, scaler.font_s(11.0));
                fonts.draw_ui_bold(
                    shortcut,
                    right_x + right_w - scaler.s(26.0) - sc_dim.width,
                    text_y,
                    scaler.font_s(11.0),
                    accent,
                );
            }
        }

        // Bottom Notification Bar
        let bottom_y = sh - bottom_bar_h - scaler.s(8.0);
        scaler.draw_glass_card(
            pad_x,
            bottom_y,
            sw - pad_x * 2.0,
            bottom_bar_h,
            Color::new(0.04, 0.06, 0.10, 0.95),
            Palette::UI_CARD_BORDER,
            1.0,
        );

        let bottom_text = format!("📢 {}", self.status_message);
        fonts.draw_ui_bold(
            &bottom_text,
            pad_x + scaler.s(16.0),
            bottom_y + bottom_bar_h * 0.62,
            scaler.font_s(12.0),
            Palette::NEON_GOLD,
        );

        let nav_hint = "[▲/▼/◄/►] Navigate  •  [ENTER] Activate  •  [ESC] Disband";
        let nav_dim = fonts.measure_ui_regular(nav_hint, scaler.font_s(11.0));
        fonts.draw_ui_regular(
            nav_hint,
            sw - pad_x - scaler.s(16.0) - nav_dim.width,
            bottom_y + bottom_bar_h * 0.62,
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        // Synchronized Launch Countdown Overlay
        if let Some(time) = self.countdown_remaining_sec {
            let overlay_w = scaler.s(480.0);
            let overlay_h = scaler.s(180.0);
            let (ox, oy, _, _) = scaler.centered_rect(overlay_w, overlay_h);

            scaler.draw_glass_card(
                ox,
                oy,
                overlay_w,
                overlay_h,
                Color::new(0.05, 0.08, 0.15, 0.98),
                Palette::NEON_GREEN,
                2.5,
            );

            fonts.draw_display_centered(
                "GRID LAUNCH COUNTDOWN",
                ox + overlay_w * 0.5,
                oy + scaler.s(45.0),
                scaler.font_s(18.0),
                Palette::WHITE,
            );

            let count_int = (time.ceil() as u32).max(1);
            let count_str = format!("{}", count_int);
            fonts.draw_display_centered_with_shadow(
                &count_str,
                ox + overlay_w * 0.5,
                oy + scaler.s(120.0),
                scaler.font_s(64.0),
                Palette::NEON_GREEN,
                Palette::BLACK,
                scaler.s(3.0),
            );
        }
    }
}
