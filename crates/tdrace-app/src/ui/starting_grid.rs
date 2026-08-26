use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use tdrace_core::track::Track;

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::ai::DriverCharacter;
use crate::render::color::{CarColorScheme, Palette};

/// Renders the starting grid and participants showcase screen before race launch.
#[allow(clippy::too_many_arguments)]
pub fn render_starting_grid_screen(
    fonts: &Fonts,
    track: &Track,
    player_car_title: &str,
    player_scheme: CarColorScheme,
    opponents: &[DriverCharacter],
    total_laps: u32,
    _is_time_attack: bool,
    gamepad_connected: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark glass backdrop overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.06, 0.10, 0.88));

    // Header Title
    let title = "🏁 STARTING GRID & RACE PARTICIPANTS";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(42.0),
        scaler.font_s(30.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    // Subtitle / Track details
    let total_racers = 1 + opponents.len();
    let track_len_m = track.spline.total_length().round() as i32;
    let subtitle = format!(
        "Circuit: {}  •  Length: {}m  •  Distance: {} Laps  •  Grid Size: {} Racers",
        track.name.to_uppercase(),
        track_len_m,
        total_laps,
        total_racers
    );
    fonts.draw_ui_regular_centered(
        &subtitle,
        sw * 0.5,
        scaler.s(68.0),
        scaler.font_s(14.0),
        Palette::UI_TEXT_MUTED,
    );

    // Grid Container Box
    let card_w = (sw * 0.90).clamp(scaler.s(600.0), scaler.s(960.0));
    let num_rows = total_racers;
    let row_h = scaler.s(52.0);
    let row_gap = scaler.s(8.0);
    let list_h = (num_rows as f32 * (row_h + row_gap)) + scaler.s(20.0);
    let box_h = list_h.clamp(scaler.s(240.0), scaler.s(520.0));
    let x = (sw - card_w) * 0.5;
    let y = scaler.s(92.0);

    scaler.draw_glass_card(x, y, card_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    // Draw Participant Rows
    let mut row_y = y + scaler.s(14.0);

    // 1. Slot 1 (Pole Position - Player)
    render_participant_row(
        fonts,
        &scaler,
        x + scaler.s(12.0),
        row_y,
        card_w - scaler.s(24.0),
        row_h,
        1,
        "Player 1 (You)",
        "Human Driver",
        player_car_title,
        player_scheme,
        "User Controlled  •  Pole Position",
        true,
    );
    row_y += row_h + row_gap;

    // 2. Slots 2..N (AI Opponent Characters)
    for (i, character) in opponents.iter().enumerate() {
        let slot = i + 2;
        let style_desc = format!(
            "Pace: {:.0}% • Aggr: {:.0}% • {}",
            character.stats.speed * 100.0,
            character.stats.aggression * 100.0,
            character.bio
        );

        render_participant_row(
            fonts,
            &scaler,
            x + scaler.s(12.0),
            row_y,
            card_w - scaler.s(24.0),
            row_h,
            slot,
            character.name,
            character.alias,
            character.preferred_car.title(),
            character.color_scheme,
            &style_desc,
            false,
        );
        row_y += row_h + row_gap;
    }

    // Footer Prompts
    let prompt = if gamepad_connected {
        "🎮 [A / START] Launch Race Countdown  |  [Y / D] Driver Dossier  |  [B / ESC] Back to Menu"
    } else {
        "▶ PRESS [SPACE / ENTER] TO START RACE  |  [D] View Driver Dossiers  |  [ESC] Back to Menu"
    };

    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        sh - scaler.s(24.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_participant_row(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    pos: usize,
    name: &str,
    alias: &str,
    car_name: &str,
    scheme: CarColorScheme,
    tagline: &str,
    is_player: bool,
) {
    // Row background
    let bg_color = if is_player {
        Color::new(0.12, 0.22, 0.35, 0.95)
    } else if pos % 2 == 0 {
        Color::new(0.08, 0.11, 0.17, 0.90)
    } else {
        Color::new(0.06, 0.08, 0.13, 0.90)
    };

    let border_color = if is_player {
        Palette::NEON_CYAN
    } else {
        Palette::UI_CARD_BORDER
    };

    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, if is_player { 1.8 } else { 1.0 }, border_color);

    // Position Badge (e.g. "P1", "P2")
    let badge_w = scaler.s(38.0);
    let badge_h = h - scaler.s(12.0);
    let badge_x = x + scaler.s(8.0);
    let badge_y = y + scaler.s(6.0);

    let (badge_bg, badge_text_col) = match pos {
        1 => (Palette::NEON_GOLD, Palette::BLACK),
        2 => (Color::new(0.85, 0.88, 0.95, 1.0), Palette::BLACK), // Silver
        3 => (Color::new(0.88, 0.55, 0.25, 1.0), Palette::BLACK), // Bronze
        _ => (Color::new(0.16, 0.20, 0.28, 0.95), Palette::WHITE),
    };

    draw_rectangle(badge_x, badge_y, badge_w, badge_h, badge_bg);
    let pos_label = format!("P{}", pos);
    fonts.draw_ui_bold_centered(
        &pos_label,
        badge_x + badge_w * 0.5,
        badge_y + badge_h * 0.5 + scaler.s(5.0),
        scaler.font_s(14.0),
        badge_text_col,
    );

    // Driver Name & Alias
    let name_x = badge_x + badge_w + scaler.s(14.0);
    let driver_label = if is_player {
        name.to_string()
    } else {
        format!("{}  (\"{}\")", name, alias)
    };

    let name_color = if is_player { Palette::NEON_CYAN } else { Palette::WHITE };
    fonts.draw_ui_bold(&driver_label, name_x, y + scaler.s(22.0), scaler.font_s(15.0), name_color);

    // Tagline / Driving Style subtext
    fonts.draw_ui_regular(tagline, name_x, y + scaler.s(40.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);

    // Car Model Badge
    let car_x = x + w - scaler.s(240.0);
    fonts.draw_ui_bold(car_name, car_x, y + scaler.s(24.0), scaler.font_s(13.5), Palette::WHITE);

    // Livery Swatches
    let swatch_w = scaler.s(16.0);
    let swatch_h = scaler.s(12.0);
    let s_y = y + scaler.s(30.0);

    draw_rectangle(car_x, s_y, swatch_w, swatch_h, scheme.primary);
    draw_rectangle_lines(car_x, s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    draw_rectangle(car_x + swatch_w + scaler.s(4.0), s_y, swatch_w, swatch_h, scheme.secondary);
    draw_rectangle_lines(car_x + swatch_w + scaler.s(4.0), s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    draw_rectangle(car_x + (swatch_w + scaler.s(4.0)) * 2.0, s_y, swatch_w, swatch_h, scheme.helmet);
    draw_rectangle_lines(car_x + (swatch_w + scaler.s(4.0)) * 2.0, s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    fonts.draw_ui_regular("Team Livery", car_x + scaler.s(66.0), s_y + scaler.s(10.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);
}
