use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::db::HallOfFameEntry;
use crate::render::color::Palette;

/// Renders the arcade modal dialog prompting the player to enter their name for the Hall of Fame.
pub fn render_name_input_modal(
    fonts: &Fonts,
    track_name: &str,
    input_name: &str,
    total_time: f32,
    best_lap: Option<f32>,
    cursor_timer: f32,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark semi-transparent overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.02, 0.03, 0.05, 0.85));

    let box_w = (sw * 0.70).clamp(scaler.s(440.0), scaler.s(640.0));
    let box_h = scaler.s(360.0);
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    // Glowing victory card
    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 2.5);

    // Modal Header
    let trophy_title = "🏆 NEW RECORD! TOP 10 QUALIFIED! 🏆";
    fonts.draw_display_centered_with_shadow(
        trophy_title,
        sw * 0.5,
        y + scaler.s(42.0),
        scaler.font_s(26.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.7),
        scaler.s(2.0),
    );

    let track_subtitle = format!("Circuit: {}", track_name);
    fonts.draw_ui_regular_centered(
        &track_subtitle,
        sw * 0.5,
        y + scaler.s(68.0),
        scaler.font_s(15.0),
        Palette::UI_TEXT_MUTED,
    );

    // Time summary card
    let stat_box_w = box_w - scaler.s(60.0);
    let stat_box_h = scaler.s(52.0);
    let stat_box_x = x + scaler.s(30.0);
    let stat_box_y = y + scaler.s(92.0);

    draw_rectangle(
        stat_box_x,
        stat_box_y,
        stat_box_w,
        stat_box_h,
        Color::new(0.10, 0.14, 0.22, 0.90),
    );
    draw_rectangle_lines(
        stat_box_x,
        stat_box_y,
        stat_box_w,
        stat_box_h,
        1.5,
        Palette::UI_CARD_BORDER,
    );

    let time_str = format!("TOTAL TIME: {}", format_lap_time(total_time));
    fonts.draw_ui_bold(
        &time_str,
        stat_box_x + scaler.s(20.0),
        stat_box_y + scaler.s(32.0),
        scaler.font_s(16.0),
        Palette::NEON_GREEN,
    );

    if let Some(lap) = best_lap {
        let lap_str = format!("BEST LAP: {}", format_lap_time(lap));
        fonts.draw_ui_bold(
            &lap_str,
            stat_box_x + stat_box_w - scaler.s(200.0),
            stat_box_y + scaler.s(32.0),
            scaler.font_s(16.0),
            Palette::NEON_CYAN,
        );
    }

    // Name Input Label
    let prompt_lbl = "ENTER DRIVER NAME:";
    fonts.draw_ui_bold_centered(
        prompt_lbl,
        sw * 0.5,
        y + scaler.s(174.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );

    // Text Input Box
    let input_w = (box_w - scaler.s(100.0)).clamp(scaler.s(280.0), scaler.s(420.0));
    let input_h = scaler.s(48.0);
    let input_x = (sw - input_w) * 0.5;
    let input_y = y + scaler.s(190.0);

    draw_rectangle(input_x, input_y, input_w, input_h, Color::new(0.06, 0.08, 0.12, 0.98));
    draw_rectangle_lines(input_x, input_y, input_w, input_h, 2.0, Palette::NEON_CYAN);

    // Blinking cursor
    let show_cursor = (cursor_timer * 2.5).fract() < 0.5;
    let display_text = if input_name.is_empty() {
        if show_cursor { "_" } else { "" }
    } else if show_cursor {
        &format!("{}_", input_name)
    } else {
        input_name
    };

    let text_col = if input_name.is_empty() {
        Palette::UI_TEXT_MUTED
    } else {
        Palette::NEON_CYAN
    };

    fonts.draw_ui_bold_centered(
        display_text,
        sw * 0.5,
        input_y + scaler.s(32.0),
        scaler.font_s(22.0),
        text_col,
    );

    // Max length indicator
    let chars_count = format!("{}/12", input_name.len());
    fonts.draw_ui_regular(
        &chars_count,
        input_x + input_w - scaler.s(42.0),
        input_y + input_h + scaler.s(18.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    // Helper Action Prompts
    let help_line = "[ENTER / GAMEPAD A] Confirm & Save | [BACKSPACE] Erase | [ESC] Skip";
    fonts.draw_ui_regular_centered(
        help_line,
        sw * 0.5,
        y + box_h - scaler.s(24.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );
}

/// Renders the full Hall of Fame Leaderboard screen with historical top 10 results.
pub fn render_hall_of_fame_screen(
    fonts: &Fonts,
    track_name: &str,
    entries: &[HallOfFameEntry],
    highlight_id: Option<i64>,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep modern motorsport gradient backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.96));

    let box_w = (sw * 0.86).clamp(scaler.s(560.0), scaler.s(920.0));
    let box_h = (sh * 0.86).clamp(scaler.s(460.0), scaler.s(720.0));
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 2.5);

    // Header Title
    let title = "🏛️ HALL OF FAME — TOP 10 HISTORICAL BEST";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        y + scaler.s(40.0),
        scaler.font_s(28.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let track_label = format!("Track: {} | Best 3-Lap Session Times", track_name);
    fonts.draw_ui_regular_centered(
        &track_label,
        sw * 0.5,
        y + scaler.s(65.0),
        scaler.font_s(15.0),
        Palette::UI_TEXT_MUTED,
    );

    // Table Header
    let mut row_y = y + scaler.s(98.0);
    let hdr_h = scaler.s(26.0);
    let table_w = box_w - scaler.s(40.0);
    let table_x = x + scaler.s(20.0);

    draw_rectangle(table_x, row_y - scaler.s(18.0), table_w, hdr_h, Color::new(0.12, 0.16, 0.25, 0.95));
    draw_rectangle_lines(table_x, row_y - scaler.s(18.0), table_w, hdr_h, 1.0, Palette::UI_CARD_BORDER);

    fonts.draw_ui_bold("POS", table_x + scaler.s(16.0), row_y, scaler.font_s(13.0), Palette::WHITE);
    fonts.draw_ui_bold("DRIVER", table_x + scaler.s(70.0), row_y, scaler.font_s(13.0), Palette::WHITE);
    fonts.draw_ui_bold("VEHICLE", table_x + scaler.s(240.0), row_y, scaler.font_s(13.0), Palette::WHITE);
    fonts.draw_ui_bold("TOTAL TIME", table_x + table_w - scaler.s(320.0), row_y, scaler.font_s(13.0), Palette::WHITE);
    fonts.draw_ui_bold("BEST LAP", table_x + table_w - scaler.s(190.0), row_y, scaler.font_s(13.0), Palette::WHITE);
    fonts.draw_ui_bold("DATE", table_x + table_w - scaler.s(90.0), row_y, scaler.font_s(13.0), Palette::WHITE);

    row_y += scaler.s(22.0);

    let row_h = scaler.s(27.0);
    for rank in 1..=10 {
        let entry = entries.get(rank - 1);
        let is_highlighted = entry.and_then(|e| e.id).is_some() && entry.and_then(|e| e.id) == highlight_id;

        let (row_bg, text_col) = if is_highlighted {
            (Color::new(0.16, 0.38, 0.22, 0.95), Palette::NEON_GREEN)
        } else if rank % 2 == 1 {
            (Color::new(0.08, 0.10, 0.15, 0.65), Color::new(0.85, 0.90, 0.96, 1.0))
        } else {
            (Color::new(0.06, 0.08, 0.12, 0.65), Color::new(0.85, 0.90, 0.96, 1.0))
        };

        draw_rectangle(table_x, row_y - scaler.s(15.0), table_w, row_h, row_bg);

        let pos_str = match rank {
            1 => "🥇 P1".to_string(),
            2 => "🥈 P2".to_string(),
            3 => "🥉 P3".to_string(),
            _ => format!("P{}", rank),
        };

        let pos_col = match rank {
            1 => Palette::NEON_GOLD,
            2 => Color::new(0.80, 0.85, 0.92, 1.0),
            3 => Color::new(0.85, 0.55, 0.35, 1.0),
            _ => Palette::UI_TEXT_MUTED,
        };

        fonts.draw_ui_bold(&pos_str, table_x + scaler.s(16.0), row_y + scaler.s(3.0), scaler.font_s(13.0), pos_col);

        if let Some(e) = entry {
            let driver_display = if is_highlighted {
                format!("★ {}", e.player_name)
            } else {
                e.player_name.clone()
            };
            fonts.draw_ui_bold(&driver_display, table_x + scaler.s(70.0), row_y + scaler.s(3.0), scaler.font_s(13.5), text_col);
            fonts.draw_ui_regular(&e.car_name, table_x + scaler.s(240.0), row_y + scaler.s(3.0), scaler.font_s(13.0), text_col);

            let total_str = format_lap_time(e.total_time);
            fonts.draw_ui_bold(&total_str, table_x + table_w - scaler.s(320.0), row_y + scaler.s(3.0), scaler.font_s(13.5), text_col);

            let lap_str = format_lap_time(e.best_lap.unwrap_or(0.0));
            fonts.draw_ui_bold(&lap_str, table_x + table_w - scaler.s(190.0), row_y + scaler.s(3.0), scaler.font_s(13.0), text_col);

            // Short date (YYYY-MM-DD)
            let date_str = e.created_at.split(' ').next().unwrap_or(&e.created_at);
            fonts.draw_ui_regular(date_str, table_x + table_w - scaler.s(90.0), row_y + scaler.s(3.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
        } else {
            fonts.draw_ui_regular("--- VACANT ---", table_x + scaler.s(70.0), row_y + scaler.s(3.0), scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
            fonts.draw_ui_regular("--", table_x + scaler.s(240.0), row_y + scaler.s(3.0), scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
            fonts.draw_ui_regular("--:--.---", table_x + table_w - scaler.s(320.0), row_y + scaler.s(3.0), scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
            fonts.draw_ui_regular("--:--.---", table_x + table_w - scaler.s(190.0), row_y + scaler.s(3.0), scaler.font_s(13.0), Palette::UI_TEXT_MUTED);
            fonts.draw_ui_regular("--", table_x + table_w - scaler.s(90.0), row_y + scaler.s(3.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
        }

        row_y += row_h + scaler.s(4.0);
    }

    // Bottom Action Prompt
    let prompt = "Press [SPACE / ENTER] or [R] to Race Again | [M] for Main Menu";
    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        y + box_h - scaler.s(20.0),
        scaler.font_s(15.0),
        Palette::WHITE,
    );
}
