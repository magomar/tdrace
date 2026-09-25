//! # LAN Multiplayer Hub UI
//!
//! Visual interface for selecting Host or Join modality in local network multiplayer.

use macroquad::color::Color;
use macroquad::input::mouse_position;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::window::{screen_height, screen_width};

use cabinet::ui::font::Fonts;
use cabinet::ui::scaler::UiScaler;
use cabinet::ui::theme::Palette;

/// Renders the LAN Hub Screen with two interactive arcade cards (0: Host, 1: Join).
/// Returns which card was clicked via mouse, if any.
pub fn render_lan_hub_screen(
    fonts: &Fonts,
    selected_idx: usize,
) -> Option<usize> {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark sleek cyber background
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.03, 0.04, 0.07, 1.0));

    // Top Header Banner
    let top_y = scaler.s(32.0);
    fonts.draw_display_centered_with_shadow(
        "LAN MULTIPLAYER",
        sw * 0.5,
        top_y,
        scaler.font_s(34.0),
        Palette::NEON_CYAN,
        Color::new(0.0, 0.0, 0.0, 0.7),
        scaler.s(2.0),
    );

    fonts.draw_ui_regular_centered(
        "ZERO-CONFIGURATION LOCAL NETWORK RACING • PEER-TO-PEER UDP (PORTS 7776 / 7777)",
        sw * 0.5,
        top_y + scaler.s(26.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    let (mx, my) = mouse_position();
    let mut clicked_card = None;

    // Two big interactive cards side-by-side
    let card_gap = scaler.s(24.0);
    let card_w = (scaler.s(440.0)).min((sw - scaler.s(80.0) - card_gap) * 0.5);
    let card_h = (scaler.s(360.0)).min(sh - top_y - scaler.s(100.0));
    let total_w = card_w * 2.0 + card_gap;
    let start_x = (sw - total_w) * 0.5;
    let card_y = top_y + scaler.s(60.0);

    let cards = [
        (
            0,
            "HOST LAN GAME",
            "PORT 7777",
            Palette::NEON_CYAN,
            "Authoritative Session Host",
            "Spawn an authoritative race server. Broadcasts UDP advertisement beacons on port 7776 across the local subnet and accepts connecting drivers.",
            &["• Automatic Subnet Beacon", "• Up to 8 Racers", "• Circuit & Lap Control", "• Low-Latency Dead Reckoning"],
            "[ 1 ]  HOST ROOM",
        ),
        (
            1,
            "JOIN LAN GAME",
            "AUTO & DIRECT IP",
            Palette::NEON_GOLD,
            "Participant Client",
            "Discover active game sessions on your Wi-Fi or wired Ethernet network automatically, or connect directly using a host IP address.",
            &["• Real-time Server Browser", "• Direct IP Keypad (Game Port 7777)", "• Vehicle & Livery Picker", "• Instant Ready Check"],
            "[ 2 ]  JOIN ROOM",
        ),
    ];

    for (idx, title, badge, accent, subtitle, desc, features, btn_label) in cards {
        let x = start_x + (card_w + card_gap) * (idx as f32);
        let y = card_y;

        let is_focused = selected_idx == idx;
        let is_hovered = mx >= x && mx <= x + card_w && my >= y && my <= y + card_h;

        if is_hovered && macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
            clicked_card = Some(idx);
        }

        let bg_col = if is_focused || is_hovered {
            Color::new(0.08, 0.11, 0.17, 1.0)
        } else {
            Color::new(0.05, 0.07, 0.11, 1.0)
        };

        let border_col = if is_focused {
            accent
        } else if is_hovered {
            Color::new(0.85, 0.88, 0.95, 1.0)
        } else {
            Color::new(0.18, 0.22, 0.30, 1.0)
        };

        // Base card rect
        draw_rectangle(x, y, card_w, card_h, bg_col);
        draw_rectangle_lines(x, y, card_w, card_h, scaler.s(if is_focused { 2.5 } else { 1.0 }), border_col);

        // Top accent stripe
        draw_rectangle(x, y, card_w, scaler.s(4.0), accent);

        // Card Header: Badge & Title
        let header_y = y + scaler.s(22.0);
        let badge_w = scaler.s(120.0);
        let badge_h = scaler.s(20.0);
        let badge_x = x + card_w - badge_w - scaler.s(16.0);
        draw_rectangle(badge_x, header_y, badge_w, badge_h, Color::new(accent.r * 0.2, accent.g * 0.2, accent.b * 0.2, 0.8));
        draw_rectangle_lines(badge_x, header_y, badge_w, badge_h, scaler.s(1.0), accent);
        fonts.draw_ui_bold_centered(
            badge,
            badge_x + badge_w * 0.5,
            header_y + scaler.s(4.0),
            scaler.font_s(10.0),
            accent,
        );

        fonts.draw_ui_bold(
            title,
            x + scaler.s(18.0),
            header_y + scaler.s(4.0),
            scaler.font_s(18.0),
            if is_focused { Palette::WHITE } else { Color::new(0.85, 0.88, 0.95, 1.0) },
        );

        fonts.draw_ui_bold(
            subtitle,
            x + scaler.s(18.0),
            header_y + scaler.s(28.0),
            scaler.font_s(12.0),
            accent,
        );

        // Description
        let desc_y = header_y + scaler.s(56.0);
        let words = desc.split_whitespace().collect::<Vec<_>>();
        let mut line = String::new();
        let mut line_y = desc_y;
        for word in words {
            let test_line = if line.is_empty() { word.to_string() } else { format!("{} {}", line, word) };
            if test_line.len() > 42 {
                fonts.draw_ui_regular(
                    &line,
                    x + scaler.s(18.0),
                    line_y,
                    scaler.font_s(12.0),
                    Palette::UI_TEXT_MUTED,
                );
                line = word.to_string();
                line_y += scaler.s(18.0);
            } else {
                line = test_line;
            }
        }
        if !line.is_empty() {
            fonts.draw_ui_regular(
                &line,
                x + scaler.s(18.0),
                line_y,
                scaler.font_s(12.0),
                Palette::UI_TEXT_MUTED,
            );
            line_y += scaler.s(22.0);
        }

        // Features list
        let mut feat_y = line_y.max(y + scaler.s(160.0));
        for &feature in features {
            fonts.draw_ui_regular(
                feature,
                x + scaler.s(22.0),
                feat_y,
                scaler.font_s(11.5),
                Color::new(0.70, 0.76, 0.85, 1.0),
            );
            feat_y += scaler.s(18.0);
        }

        // Action Button at card bottom
        let btn_h = scaler.s(36.0);
        let btn_y = y + card_h - btn_h - scaler.s(16.0);
        let btn_w = card_w - scaler.s(36.0);
        let btn_x = x + scaler.s(18.0);

        let btn_bg = if is_focused {
            accent
        } else if is_hovered {
            Color::new(0.20, 0.25, 0.35, 1.0)
        } else {
            Color::new(0.12, 0.15, 0.22, 1.0)
        };
        let btn_fg = if is_focused {
            Palette::BLACK
        } else {
            Palette::WHITE
        };

        draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_bg);
        if !is_focused {
            draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, scaler.s(1.0), accent);
        }
        fonts.draw_ui_bold_centered(
            btn_label,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(9.0),
            scaler.font_s(14.0),
            btn_fg,
        );
    }

    // Bottom Navigation Help Footer
    let footer_y = sh - scaler.s(36.0);
    fonts.draw_ui_regular_centered(
        "[▲/▼ or ◀/▶] Navigate  •  [ENTER / SPACE / A] Select  •  [ESC / B] Back to Menu",
        sw * 0.5,
        footer_y,
        scaler.font_s(12.5),
        Palette::UI_TEXT_MUTED,
    );

    clicked_card
}
