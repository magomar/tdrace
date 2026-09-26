//! # Cabinet LAN Client Lobby Screen
//!
//! Participant waiting room lobby screen displaying the room settings,
//! connected racers table, local vehicle/livery selectors, ready status toggle,
//! and synchronized starting grid countdown.

use macroquad::color::Color;
use macroquad::input::{is_key_pressed, mouse_position, KeyCode};
use macroquad::shapes::draw_rectangle;

use crate::input::NavGrid2D;
use crate::net::client::{ClientEvent, ClientState, LanClient};
use crate::net::protocol::LobbySlot;
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

/// Participant client waiting room lobby screen.
pub struct CabinetLanClientLobbyScreen {
    /// Active client connection to the host.
    pub client: LanClient,
    /// Available vehicle models for cycling: (id, display_name)
    pub car_models: Vec<(String, String)>,
    pub selected_car_idx: usize,
    /// Available livery colors for cycling: (id, display_name, color)
    pub liveries: Vec<(String, String, Color)>,
    pub selected_livery_idx: usize,
    /// Local driver ready status.
    pub is_ready: bool,
    /// 2D navigation grid (Col 0: Slots table, Col 1: Player controls).
    pub nav: NavGrid2D,
    /// Status message banner.
    pub status_message: String,
    /// Countdown remaining timer in seconds, if active.
    pub countdown_remaining_sec: Option<f32>,
    /// Pulse timer for animations.
    pulse_timer: f32,
}

impl CabinetLanClientLobbyScreen {
    /// Creates a client lobby screen wrapping a connected `LanClient`.
    pub fn new(client: LanClient) -> Self {
        let car_models = vec![
            ("scuderia_gt".to_string(), "Scuderia 296 GT3".to_string()),
            ("stuttgart_gt".to_string(), "Stuttgart 911 GT3 R".to_string()),
            ("bavarian_m4".to_string(), "Bavarian M4 GT3".to_string()),
            ("silverstone_vantage".to_string(), "Silverstone Vantage GT3".to_string()),
            ("shifter_kart_125".to_string(), "125cc Shifter Kart".to_string()),
        ];

        let liveries = vec![
            ("corsa_red".to_string(), "Rosso Corsa".to_string(), Palette::NEON_RED),
            ("viper_green".to_string(), "Viper Green".to_string(), Palette::NEON_GREEN),
            ("speed_yellow".to_string(), "Speed Yellow".to_string(), Palette::NEON_GOLD),
            ("matte_cyan".to_string(), "Matte Cyan".to_string(), Palette::NEON_CYAN),
            ("stealth_black".to_string(), "Stealth Black".to_string(), Color::new(0.20, 0.22, 0.28, 1.0)),
        ];

        Self {
            client,
            car_models,
            selected_car_idx: 0,
            liveries,
            selected_livery_idx: 0,
            is_ready: false,
            nav: NavGrid2D::new(vec![8, 4]),
            status_message: "Connected to host lobby. Choose car and mark READY!".to_string(),
            countdown_remaining_sec: None,
            pulse_timer: 0.0,
        }
    }

    /// Returns a reference to the underlying client.
    pub fn client(&self) -> &LanClient {
        &self.client
    }

    /// Returns a mutable reference to the underlying client.
    pub fn client_mut(&mut self) -> &mut LanClient {
        &mut self.client
    }

    /// Consumes the screen and returns the inner client.
    pub fn into_client(self) -> LanClient {
        self.client
    }

    /// Returns true if countdown has completed and race is active.
    pub fn is_in_race(&self) -> bool {
        matches!(self.client.state(), ClientState::InRace { .. })
    }

    /// Cycles local car model.
    pub fn cycle_car(&mut self) {
        if !self.car_models.is_empty() {
            self.selected_car_idx = (self.selected_car_idx + 1) % self.car_models.len();
            self.sync_slot_update();
        }
    }

    /// Cycles local livery color.
    pub fn cycle_livery(&mut self) {
        if !self.liveries.is_empty() {
            self.selected_livery_idx = (self.selected_livery_idx + 1) % self.liveries.len();
            self.sync_slot_update();
        }
    }

    /// Toggles local driver ready check.
    pub fn toggle_ready(&mut self) {
        self.is_ready = !self.is_ready;
        self.sync_slot_update();
    }

    /// Transmits slot customization update packet to host.
    pub fn sync_slot_update(&mut self) {
        let (car_id, _) = &self.car_models[self.selected_car_idx];
        let (livery_id, _, _) = &self.liveries[self.selected_livery_idx];
        let _ = self.client.send_slot_update(car_id.clone(), livery_id.clone(), self.is_ready);
    }
}

impl CabinetScreen for CabinetLanClientLobbyScreen {
    fn name(&self) -> &str {
        "CabinetLanClientLobbyScreen"
    }

    fn is_transparent(&self) -> bool {
        false
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        self.pulse_timer += ctx.dt;

        // Advance countdown timer if active
        if let Some(ref mut time) = self.countdown_remaining_sec {
            *time -= ctx.dt;
            if *time <= 0.0 {
                self.countdown_remaining_sec = None;
            }
        }

        // Pump client network events
        let events = self.client.update(ctx.dt);
        for event in events {
            match event {
                ClientEvent::Connected { room_name, track_id, slot_id } => {
                    self.status_message = format!("Joined '{}' ({}) in Slot {}!", room_name, track_id, slot_id + 1);
                }
                ClientEvent::LobbyUpdated { track_id, laps, .. } => {
                    self.status_message = format!("Host updated track: {} ({} Laps)", track_id, laps);
                }
                ClientEvent::CountdownStarted { starts_in_millis, .. } => {
                    ctx.play_ui_select();
                    self.countdown_remaining_sec = Some(starts_in_millis as f32 / 1000.0);
                    self.status_message = "Host launched starting countdown! Prepare for race...".to_string();
                }
                ClientEvent::Disconnected(_reason) => {
                    ctx.play_ui_cancel();
                    return ScreenAction::Pop;
                }
                ClientEvent::WorldSnapshot(_) => {}
            }
        }

        // Check if disconnected
        if !self.client.is_connected() {
            return ScreenAction::Pop;
        }

        // Cancel / Disconnect (ESC or Gamepad B)
        if safe_key_pressed(KeyCode::Escape) || ctx.gamepad.btn_cancel_pressed || ctx.gamepad.btn_b_pressed {
            ctx.play_ui_cancel();
            let _ = self.client.disconnect();
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

        // Confirm button action (ENTER / Gamepad A)
        let is_confirmed = self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed);

        if self.nav.focused_col == 1 && is_confirmed {
            match self.nav.active_row() {
                0 => {
                    // Car model
                    ctx.play_ui_move();
                    self.cycle_car();
                }
                1 => {
                    // Livery color
                    ctx.play_ui_move();
                    self.cycle_livery();
                }
                2 => {
                    // Ready toggle
                    ctx.play_ui_select();
                    self.toggle_ready();
                }
                3 => {
                    // Leave room
                    ctx.play_ui_cancel();
                    let _ = self.client.disconnect();
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

        let (room_name, track_name, laps) = match self.client.state() {
            ClientState::InLobby { room_name, track_id, laps, .. } => (room_name.as_str(), track_id.as_str(), *laps),
            _ => ("LAN Game", "Grand Prix", 5),
        };

        let title = format!("🌐 LAN MULTIPLAYER LOBBY — \"{}\"", room_name.to_uppercase());
        fonts.draw_display(
            &title,
            pad_x + scaler.s(16.0),
            pad_y + header_h * 0.65,
            scaler.font_s(17.0),
            Palette::WHITE,
        );

        let ping_text = format!("HOST: {} ({} ms)", self.client.host_addr(), self.client.ping_ms());
        let ping_dim = fonts.measure_ui_bold(&ping_text, scaler.font_s(12.5));
        fonts.draw_ui_bold(
            &ping_text,
            sw - pad_x - scaler.s(16.0) - ping_dim.width,
            pad_y + header_h * 0.65,
            scaler.font_s(12.5),
            Palette::NEON_CYAN,
        );

        // Subheader Banner (Track and Laps info)
        let sub_y = pad_y + header_h + scaler.s(10.0);
        let sub_h = scaler.s(44.0);
        scaler.draw_glass_card(
            pad_x,
            sub_y,
            sw - pad_x * 2.0,
            sub_h,
            Color::new(0.05, 0.07, 0.12, 0.95),
            Palette::UI_CARD_BORDER,
            1.2,
        );

        let info_line = format!("TRACK: {}  •  LAPS: {} LAPS  •  HOST-AUTHORITATIVE SAT", track_name, laps);
        fonts.draw_ui_bold(
            &info_line,
            pad_x + scaler.s(16.0),
            sub_y + sub_h * 0.65,
            scaler.font_s(13.0),
            Palette::WHITE,
        );

        // Split Layout: Left Panel (Slots Roster), Right Panel (Player Customization)
        let content_y = sub_y + sub_h + scaler.s(12.0);
        let bottom_bar_h = scaler.s(42.0);
        let content_h = sh - content_y - bottom_bar_h - scaler.s(16.0);

        let col_gap = scaler.s(16.0);
        let left_w = (sw - pad_x * 2.0 - col_gap) * 0.58;
        let right_w = (sw - pad_x * 2.0 - col_gap) * 0.42;
        let right_x = pad_x + left_w + col_gap;

        // --- Left Panel: Connected Drivers Roster ---
        scaler.draw_glass_card(
            pad_x,
            content_y,
            left_w,
            content_h,
            Color::new(0.05, 0.07, 0.11, 0.90),
            Palette::UI_CARD_BORDER,
            1.0,
        );

        let slots_in_lobby: Vec<LobbySlot> = match self.client.state() {
            ClientState::InLobby { slots, .. } => slots.clone(),
            _ => Vec::new(),
        };

        let roster_header = format!("CONNECTED DRIVERS ({} / 8 SLOTS FILLED)", slots_in_lobby.len());
        fonts.draw_ui_bold(
            &roster_header,
            pad_x + scaler.s(16.0),
            content_y + scaler.s(26.0),
            scaler.font_s(14.0),
            Palette::NEON_CYAN,
        );

        let slot_pad = scaler.s(6.0);
        let slot_start_y = content_y + scaler.s(36.0);
        let slot_h = (content_h - scaler.s(44.0) - slot_pad * 7.0) / 8.0;

        let my_slot_id = self.client.assigned_slot_id().unwrap_or(255);

        for i in 0..8 {
            let sy = slot_start_y + i as f32 * (slot_h + slot_pad);
            let slot_opt = slots_in_lobby.iter().find(|s| s.slot_id == i as u8);
            let is_my_slot = i as u8 == my_slot_id;

            let (bg_col, border_col) = if is_my_slot {
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
                if is_my_slot { 1.8 } else { 1.0 },
            );

            let slot_badge = if i == 0 {
                "SLOT 1 [HOST]".to_string()
            } else if is_my_slot {
                format!("SLOT {} [YOU]", i + 1)
            } else {
                format!("SLOT {}", i + 1)
            };

            if let Some(slot) = slot_opt {
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
            } else {
                let empty_line = format!("{} [ OPEN SLOT — WAITING... ]", slot_badge);
                fonts.draw_ui_regular(
                    &empty_line,
                    pad_x + scaler.s(22.0),
                    sy + slot_h * 0.62,
                    scaler.font_s(11.0),
                    Color::new(0.40, 0.48, 0.60, 0.70),
                );
            }
        }

        // --- Right Panel: Player Customization & Ready Check ---
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
            "DRIVER LOADOUT & READY CHECK",
            right_x + scaler.s(16.0),
            content_y + scaler.s(26.0),
            scaler.font_s(14.0),
            Palette::NEON_CYAN,
        );

        let ctrl_pad = scaler.s(12.0);
        let ctrl_start_y = content_y + scaler.s(45.0);
        let ctrl_item_h = scaler.s(48.0);

        let (_, car_name) = &self.car_models[self.selected_car_idx];
        let (_, livery_name, livery_color) = &self.liveries[self.selected_livery_idx];

        let ready_button_title = if self.is_ready {
            "STATUS: READY ⭐ [PRESS ENTER]"
        } else {
            "STATUS: SELECTING CAR... [PRESS ENTER]"
        };

        let items_meta = [
            ("VEHICLE", car_name.as_str(), "[CYCLE]"),
            ("LIVERY", livery_name.as_str(), "[CYCLE]"),
            ("READY CHECK", ready_button_title, ""),
            ("LEAVE LOBBY", "Disconnect & Return", "[ESC]"),
        ];

        let (focus_col, focus_row) = self.nav.active_cell();

        for (idx, (label, val, shortcut)) in items_meta.iter().enumerate() {
            let by = ctrl_start_y + idx as f32 * (ctrl_item_h + ctrl_pad);
            let is_focused = focus_col == 1 && focus_row == idx;
            let (mx, my) = safe_mouse_pos();
            let is_hovered = mx >= right_x + scaler.s(14.0)
                && mx <= right_x + right_w - scaler.s(14.0)
                && my >= by
                && my <= by + ctrl_item_h;

            let accent = if idx == 2 {
                if self.is_ready { Palette::NEON_GREEN } else { Palette::NEON_GOLD }
            } else if idx == 3 {
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
            let title_line = if idx == 2 {
                ready_button_title.to_string()
            } else if idx == 3 {
                "✕ LEAVE ROOM".to_string()
            } else {
                format!("{}: {}", label, val)
            };

            let title_color = if idx == 2 && self.is_ready {
                Palette::NEON_GREEN
            } else if is_focused || is_hovered {
                Palette::WHITE
            } else {
                Palette::UI_TEXT_MUTED
            };

            fonts.draw_ui_bold(
                &title_line,
                right_x + scaler.s(24.0),
                text_y,
                scaler.font_s(13.0),
                title_color,
            );

            if !shortcut.is_empty() && idx != 2 && idx != 3 {
                let sc_dim = fonts.measure_ui_bold(shortcut, scaler.font_s(11.0));
                fonts.draw_ui_bold(
                    shortcut,
                    right_x + right_w - scaler.s(26.0) - sc_dim.width,
                    text_y,
                    scaler.font_s(11.0),
                    accent,
                );
            }

            // Draw livery color swatch on livery row
            if idx == 1 {
                let swatch_x = right_x + scaler.s(24.0) + fonts.measure_ui_bold(&title_line, scaler.font_s(13.0)).width + scaler.s(10.0);
                draw_rectangle(
                    swatch_x,
                    by + ctrl_item_h * 0.35,
                    scaler.s(14.0),
                    scaler.s(14.0),
                    *livery_color,
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

        let nav_hint = "[ARROWS / WASD] Navigate  •  [ENTER] Toggle / Cycle  •  [ESC] Leave";
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
