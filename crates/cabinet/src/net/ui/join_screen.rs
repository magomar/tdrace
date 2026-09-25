//! # Cabinet LAN Join Screen & Server Browser
//!
//! Passive discovery server browser table listening on UDP port 7776,
//! paired with direct IP entry keypad for zero-friction local multiplayer connections.

use std::net::SocketAddr;
use macroquad::color::Color;
use macroquad::input::{is_key_pressed, mouse_position, KeyCode};
use macroquad::shapes::draw_rectangle;

use crate::input::NavGrid2D;
use crate::net::beacon::LanBeaconScanner;
use crate::net::client::{ClientEvent, LanClient};
use crate::net::ip::LocalIpResolver;
use crate::net::protocol::DEFAULT_GAME_PORT;
use crate::net::ui::client_lobby_screen::CabinetLanClientLobbyScreen;
use crate::net::ui::ip_keypad::{IpKeypad, IpKeypadAction};
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

/// Server browser and direct IP entry join screen for LAN multiplayer.
pub struct CabinetLanJoinScreen {
    /// Passive beacon discovery scanner.
    pub scanner: LanBeaconScanner,
    /// Virtual keypad widget for direct IP typing.
    pub keypad: IpKeypad,
    /// Pending connection attempt client.
    pub pending_client: Option<LanClient>,
    /// Player identity for joining rooms.
    pub player_name: String,
    pub country_code: String,
    pub car_model_id: String,
    pub color_scheme_id: String,
    /// 2D navigation router (Col 0: Browser list, Col 1: Direct keypad).
    pub nav: NavGrid2D,
    /// Notification status message.
    pub status_message: String,
    /// Search scanner pulse timer.
    pulse_timer: f32,
    /// Selected game index in discovered list.
    pub selected_browser_row: usize,
    /// If connection succeeded, holds the transition lobby screen.
    pub connected_lobby_screen: Option<CabinetLanClientLobbyScreen>,
}

impl CabinetLanJoinScreen {
    /// Creates a new LAN server browser and direct connect screen.
    pub fn new(
        player_name: impl Into<String>,
        country_code: impl Into<String>,
        car_model_id: impl Into<String>,
        color_scheme_id: impl Into<String>,
    ) -> Self {
        let scanner = LanBeaconScanner::bind(0).unwrap_or_else(|_| {
            LanBeaconScanner::bind(7776).unwrap_or_else(|_| panic!("Failed to bind LAN scanner"))
        });

        Self {
            scanner,
            keypad: IpKeypad::new("192.168.1."),
            pending_client: None,
            player_name: player_name.into(),
            country_code: country_code.into(),
            car_model_id: car_model_id.into(),
            color_scheme_id: color_scheme_id.into(),
            nav: NavGrid2D::new(vec![4, 4]),
            status_message: "Searching local network (UDP Port 7776)...".to_string(),
            pulse_timer: 0.0,
            selected_browser_row: 0,
            connected_lobby_screen: None,
        }
    }

    /// Takes the connected client lobby screen if connection succeeded.
    pub fn take_connected_lobby_screen(&mut self) -> Option<CabinetLanClientLobbyScreen> {
        self.connected_lobby_screen.take()
    }

    /// Initiates a connection handshake to a specific target address.
    pub fn connect_to(&mut self, addr: SocketAddr) {
        self.status_message = format!("Connecting to host at {}...", addr);
        match LanClient::connect(
            addr,
            &self.player_name,
            &self.country_code,
            &self.car_model_id,
            &self.color_scheme_id,
        ) {
            Ok(client) => {
                self.pending_client = Some(client);
            }
            Err(e) => {
                self.status_message = format!("Failed to initiate connection: {}", e);
            }
        }
    }

    /// Parses the keypad buffer text and initiates a connection.
    pub fn connect_via_keypad(&mut self) {
        let text = self.keypad.text().trim();
        if let Some(addr) = LocalIpResolver::parse_address(text, DEFAULT_GAME_PORT) {
            self.connect_to(addr);
        } else {
            self.status_message = format!("Invalid IP address format: '{}'", text);
        }
    }
}

impl CabinetScreen for CabinetLanJoinScreen {
    fn name(&self) -> &str {
        "CabinetLanJoinScreen"
    }

    fn is_transparent(&self) -> bool {
        false
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        self.pulse_timer += ctx.dt;

        // 1. Advance beacon scanner
        let discovered = self.scanner.update(ctx.dt);
        let num_hosts = discovered.len();

        // 2. Advance pending connection attempt if active
        if let Some(ref mut client) = self.pending_client {
            let events = client.update(ctx.dt);
            for event in events {
                match event {
                    ClientEvent::Connected { .. } => {
                        ctx.play_ui_select();
                        let client = self.pending_client.take().unwrap();
                        let lobby_screen = CabinetLanClientLobbyScreen::new(client);
                        self.connected_lobby_screen = Some(lobby_screen);
                        return ScreenAction::None;
                    }
                    ClientEvent::Disconnected(reason) => {
                        ctx.play_ui_cancel();
                        self.status_message = format!("Connection rejected: {}", reason);
                        self.pending_client = None;
                        break;
                    }
                    _ => {}
                }
            }
        }

        // 3. Handle hotkeys (Refresh / Back)
        if safe_key_pressed(KeyCode::R) {
            ctx.play_ui_move();
            self.scanner.clear();
            self.status_message = "Refreshed game list. Searching local subnet...".to_string();
        }

        if safe_key_pressed(KeyCode::Escape) || ctx.gamepad.btn_cancel_pressed || ctx.gamepad.btn_b_pressed {
            ctx.play_ui_cancel();
            return ScreenAction::Pop;
        }

        // 4. Update NavGrid dimensions
        let browser_rows = num_hosts.max(1) + 1; // Discovered hosts + Refresh button
        self.nav.set_column_len(0, browser_rows);
        self.nav.set_column_len(1, 4);

        // 5. Handle Keypad input when focused
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let pad_x = ctx.scaler.s(28.0);
        let header_h = ctx.scaler.s(48.0);
        let content_y = pad_x + header_h + ctx.scaler.s(16.0);
        let bottom_bar_h = ctx.scaler.s(42.0);
        let content_h = sh - content_y - bottom_bar_h - ctx.scaler.s(16.0);
        let col_gap = ctx.scaler.s(16.0);
        let left_w = (sw - pad_x * 2.0 - col_gap) * 0.54;
        let right_w = (sw - pad_x * 2.0 - col_gap) * 0.46;
        let right_x = pad_x + left_w + col_gap;
        let keypad_rect = (right_x + ctx.scaler.s(16.0), content_y + ctx.scaler.s(60.0), right_w - ctx.scaler.s(32.0), content_h - ctx.scaler.s(80.0));

        let keypad_action = self.keypad.handle_input(
            ctx.dt,
            ctx.gamepad.nav_left,
            ctx.gamepad.nav_right,
            ctx.gamepad.nav_up,
            ctx.gamepad.nav_down,
            ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed,
            keypad_rect,
            ctx.scaler,
        );

        match keypad_action {
            IpKeypadAction::Submit(_) => {
                ctx.play_ui_select();
                self.connect_via_keypad();
            }
            IpKeypadAction::Changed(_) => {
                ctx.play_ui_move();
            }
            IpKeypadAction::Clear => {
                ctx.play_ui_cancel();
            }
            IpKeypadAction::None => {}
        }

        // 6. Directional navigation between Browser and Direct Keypad
        if safe_key_pressed(KeyCode::Tab) {
            ctx.play_ui_move();
            if self.nav.focused_col == 0 {
                self.nav.set_focus(1, 0);
            } else {
                self.nav.set_focus(0, 0);
            }
        }

        // 7. Connect via browser selection
        if self.nav.focused_col == 0 && (safe_key_pressed(KeyCode::Enter) || ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed) {
            let row = self.nav.active_row();
            if row < num_hosts {
                let target_addr = self.scanner.discovered_hosts()[row].game_socket_addr();
                ctx.play_ui_select();
                self.connect_to(target_addr);
            } else {
                // Refresh button row
                ctx.play_ui_move();
                self.scanner.clear();
                self.status_message = "Refreshed game list. Searching local subnet...".to_string();
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

        let search_pulse = if (self.pulse_timer % 1.2) < 0.6 { "●" } else { "○" };
        let header_title = format!("🔍 JOIN LOCAL LAN GAME — SEARCHING SUBNET {} (PORT 7776)...", search_pulse);
        fonts.draw_display(
            &header_title,
            pad_x + scaler.s(16.0),
            pad_y + header_h * 0.65,
            scaler.font_s(16.0),
            Palette::WHITE,
        );

        let refresh_text = "[R] REFRESH LIST";
        let r_dim = fonts.measure_ui_bold(refresh_text, scaler.font_s(12.0));
        fonts.draw_ui_bold(
            refresh_text,
            sw - pad_x - scaler.s(16.0) - r_dim.width,
            pad_y + header_h * 0.65,
            scaler.font_s(12.0),
            Palette::NEON_CYAN,
        );

        // Split Layout: Left Panel (Server Browser), Right Panel (Direct IP Entry)
        let content_y = pad_y + header_h + scaler.s(12.0);
        let bottom_bar_h = scaler.s(42.0);
        let content_h = sh - content_y - bottom_bar_h - scaler.s(16.0);

        let col_gap = scaler.s(16.0);
        let left_w = (sw - pad_x * 2.0 - col_gap) * 0.54;
        let right_w = (sw - pad_x * 2.0 - col_gap) * 0.46;
        let right_x = pad_x + left_w + col_gap;

        // --- Left Panel: Server Browser ---
        scaler.draw_glass_card(
            pad_x,
            content_y,
            left_w,
            content_h,
            Color::new(0.05, 0.07, 0.11, 0.90),
            Palette::UI_CARD_BORDER,
            1.0,
        );

        let discovered = self.scanner.discovered_hosts();
        let browser_title = format!("FOUND LOCAL GAMES ON YOUR SUBNET ({})", discovered.len());
        fonts.draw_ui_bold(
            &browser_title,
            pad_x + scaler.s(16.0),
            content_y + scaler.s(26.0),
            scaler.font_s(14.0),
            Palette::NEON_CYAN,
        );

        let table_start_y = content_y + scaler.s(42.0);
        let item_h = scaler.s(52.0);
        let item_pad = scaler.s(8.0);

        let (focus_col, focus_row) = self.nav.active_cell();

        if discovered.is_empty() {
            // No LAN games active prompt
            let no_games_msg = "No active LAN hosts detected on your subnet.\nMake sure the host is running and connected to the same Wi-Fi/Ethernet,\nor enter their IP directly in the panel on the right.";
            fonts.draw_ui_regular(
                no_games_msg,
                pad_x + scaler.s(20.0),
                table_start_y + scaler.s(30.0),
                scaler.font_s(12.0),
                Palette::UI_TEXT_MUTED,
            );

            // Manual Refresh button
            let rf_y = table_start_y + scaler.s(110.0);
            let is_rf_focused = focus_col == 0 && focus_row == 0;
            scaler.draw_button_card(
                pad_x + scaler.s(16.0),
                rf_y,
                left_w - scaler.s(32.0),
                scaler.s(44.0),
                is_rf_focused,
                false,
                Palette::NEON_CYAN,
            );
            fonts.draw_ui_bold_centered(
                "⟳ RE-SCAN LOCAL NETWORK",
                pad_x + left_w * 0.5,
                rf_y + scaler.s(27.0),
                scaler.font_s(13.0),
                Palette::WHITE,
            );
        } else {
            // Render list of discovered hosts
            for (idx, host) in discovered.iter().enumerate() {
                let iy = table_start_y + idx as f32 * (item_h + item_pad);
                let is_focused = focus_col == 0 && focus_row == idx;
                let (mx, my) = safe_mouse_pos();
                let is_hovered = mx >= pad_x + scaler.s(12.0)
                    && mx <= pad_x + left_w - scaler.s(12.0)
                    && my >= iy
                    && my <= iy + item_h;

                scaler.draw_button_card(
                    pad_x + scaler.s(12.0),
                    iy,
                    left_w - scaler.s(24.0),
                    item_h,
                    is_focused,
                    is_hovered,
                    Palette::NEON_GREEN,
                );

                // Room Title + Host Name
                let title_line = format!("{} — \"{}\"", host.beacon.host_player_name, host.beacon.server_name);
                fonts.draw_ui_bold(
                    &title_line,
                    pad_x + scaler.s(22.0),
                    iy + item_h * 0.42,
                    scaler.font_s(12.5),
                    Palette::WHITE,
                );

                // Track Name + Discipline + Drivers Count
                let desc_line = format!(
                    "Track: {}  •  {}  •  {}/{} Drivers",
                    host.beacon.track_id, host.beacon.discipline, host.beacon.current_players, host.beacon.max_players
                );
                fonts.draw_ui_regular(
                    &desc_line,
                    pad_x + scaler.s(22.0),
                    iy + item_h * 0.80,
                    scaler.font_s(10.5),
                    Palette::UI_TEXT_MUTED,
                );

                // [JOIN] Action badge
                let join_text = "[JOIN]";
                let j_dim = fonts.measure_ui_bold(join_text, scaler.font_s(12.0));
                fonts.draw_ui_bold(
                    join_text,
                    pad_x + left_w - scaler.s(32.0) - j_dim.width,
                    iy + item_h * 0.62,
                    scaler.font_s(12.0),
                    Palette::NEON_GREEN,
                );
            }
        }

        // --- Right Panel: Direct IP Entry & Keypad ---
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
            "DIRECT IP CONNECT (CROSS-SUBNET / AP ISOLATION)",
            right_x + scaler.s(16.0),
            content_y + scaler.s(26.0),
            scaler.font_s(13.0),
            Palette::NEON_CYAN,
        );

        let prompt_text = "Can't see your friend's game? Type their Host IP:Port below:";
        fonts.draw_ui_regular(
            prompt_text,
            right_x + scaler.s(16.0),
            content_y + scaler.s(45.0),
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );

        // Render virtual numeric keypad
        let kp_y = content_y + scaler.s(55.0);
        let kp_h = content_h - scaler.s(70.0);
        self.keypad.draw(
            scaler,
            fonts,
            right_x + scaler.s(16.0),
            kp_y,
            right_w - scaler.s(32.0),
            kp_h,
        );

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

        let nav_hint = "[TAB] Switch Panel  •  [ENTER] Join Selected  •  [ESC] Back";
        let nav_dim = fonts.measure_ui_regular(nav_hint, scaler.font_s(11.0));
        fonts.draw_ui_regular(
            nav_hint,
            sw - pad_x - scaler.s(16.0) - nav_dim.width,
            bottom_y + bottom_bar_h * 0.62,
            scaler.font_s(11.0),
            Palette::UI_TEXT_MUTED,
        );
    }
}
