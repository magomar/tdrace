use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::ai::DriverCharacter;
use crate::render::color::Palette;

/// Renders the full-screen Driver Cards Dossier and Roster Browser.
pub fn render_driver_cards_screen(fonts: &Fonts, drivers: &[DriverCharacter], selected_idx: usize) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    let default_roster = DriverCharacter::all();
    let roster = if drivers.is_empty() { default_roster.as_slice() } else { drivers };
    let driver = &roster[selected_idx % roster.len()];

    // Deep motorsport dark backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header Title
    let title = "MOTORSPORT DRIVER DOSSIER & ROSTER";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(42.0),
        scaler.font_s(32.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = "Predefined AI Opponents | Operationalized Handling Styles & Driver Profiles";
    fonts.draw_ui_regular_centered(
        subtitle,
        sw * 0.5,
        scaler.s(66.0),
        scaler.font_s(15.0),
        Palette::UI_TEXT_MUTED,
    );

    // Main Card Dimensions
    let box_w = (sw * 0.88).clamp(scaler.s(580.0), scaler.s(920.0));
    let box_h = (sh * 0.74).clamp(scaler.s(400.0), scaler.s(580.0));
    let x = (sw - box_w) * 0.5;
    let y = scaler.s(88.0);

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.2);

    // Top Banner inside Card: Driver Name, Alias, and Index Badge
    let banner_h = scaler.s(58.0);
    draw_rectangle(x + scaler.s(16.0), y + scaler.s(16.0), box_w - scaler.s(32.0), banner_h, Color::new(0.10, 0.14, 0.22, 0.95));
    draw_rectangle_lines(x + scaler.s(16.0), y + scaler.s(16.0), box_w - scaler.s(32.0), banner_h, 1.5, Palette::UI_CARD_BORDER);

    // Driver Index Badge
    let badge_str = format!("#{} OF {}", (selected_idx % roster.len()) + 1, roster.len());
    fonts.draw_ui_bold(&badge_str, x + scaler.s(32.0), y + scaler.s(42.0), scaler.font_s(14.0), Palette::NEON_GOLD);

    // Driver Alias / Full Name
    let name_str = format!("{} — \"{}\"", driver.name, driver.alias);
    fonts.draw_ui_bold(&name_str, x + scaler.s(120.0), y + scaler.s(42.0), scaler.font_s(22.0), Palette::WHITE);

    // Two-Column Layout inside Card
    let content_y = y + scaler.s(88.0);
    let col_gap = scaler.s(20.0);
    let col_w = (box_w - scaler.s(32.0) - col_gap) * 0.5;
    let col1_x = x + scaler.s(16.0);
    let col2_x = col1_x + col_w + col_gap;
    let col_h = box_h - scaler.s(110.0);

    // --- Left Column: Bio, Vehicle & Livery ---
    scaler.draw_glass_card(col1_x, content_y, col_w, col_h, Color::new(0.06, 0.08, 0.12, 0.85), Palette::UI_CARD_BORDER, 1.2);

    let mut left_y = content_y + scaler.s(24.0);
    fonts.draw_ui_bold("DRIVER PROFILE & BACKGROUND", col1_x + scaler.s(16.0), left_y, scaler.font_s(15.0), Palette::NEON_CYAN);
    left_y += scaler.s(22.0);

    // Bio text
    let bio_font_size = scaler.font_s(13.0);
    let bio_max_w = col_w - scaler.s(32.0);
    let bio_lines = fonts.wrap_text(driver.bio, bio_font_size, bio_max_w);
    let line_height = scaler.s(16.0);
    for (i, line) in bio_lines.iter().enumerate() {
        fonts.draw_ui_regular(
            line,
            col1_x + scaler.s(16.0),
            left_y + (i as f32 * line_height),
            bio_font_size,
            Color::new(0.85, 0.90, 0.96, 1.0),
        );
    }
    left_y += scaler.s(60.0);

    // Preferred Car Section
    fonts.draw_ui_bold("PREFERRED VEHICLE", col1_x + scaler.s(16.0), left_y, scaler.font_s(14.0), Palette::NEON_GOLD);
    left_y += scaler.s(18.0);

    let car_desc = format!("{} ({})", driver.preferred_car.title(), driver.preferred_car.tag());
    fonts.draw_ui_bold(&car_desc, col1_x + scaler.s(16.0), left_y, scaler.font_s(15.0), Palette::WHITE);
    left_y += scaler.s(16.0);
    fonts.draw_ui_regular(driver.preferred_car.description(), col1_x + scaler.s(16.0), left_y, scaler.font_s(11.5), Palette::UI_TEXT_MUTED);
    left_y += scaler.s(45.0);

    // Custom Car Livery Swatches
    fonts.draw_ui_bold("CUSTOM TEAM LIVERY", col1_x + scaler.s(16.0), left_y, scaler.font_s(14.0), Palette::NEON_MAGENTA);
    left_y += scaler.s(18.0);

    let swatch_w = scaler.s(32.0);
    let swatch_h = scaler.s(20.0);

    // Primary Body Swatch

    let s1_x = col1_x + scaler.s(16.0);
    draw_rectangle(s1_x, left_y, swatch_w, swatch_h, driver.color_scheme.primary);
    draw_rectangle_lines(s1_x, left_y, swatch_w, swatch_h, 1.5, Palette::WHITE);
    fonts.draw_ui_regular("Body", s1_x + swatch_w + scaler.s(6.0), left_y + scaler.s(15.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);

    // Stripe / Secondary Swatch
    let s2_x = s1_x + scaler.s(85.0);
    draw_rectangle(s2_x, left_y, swatch_w, swatch_h, driver.color_scheme.secondary);
    draw_rectangle_lines(s2_x, left_y, swatch_w, swatch_h, 1.5, Palette::WHITE);
    fonts.draw_ui_regular("Stripe", s2_x + swatch_w + scaler.s(6.0), left_y + scaler.s(15.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);

    // Helmet / Visor Swatch
    let s3_x = s2_x + scaler.s(90.0);
    draw_rectangle(s3_x, left_y, swatch_w, swatch_h, driver.color_scheme.helmet);
    draw_rectangle_lines(s3_x, left_y, swatch_w, swatch_h, 1.5, Palette::WHITE);
    fonts.draw_ui_regular("Helmet", s3_x + swatch_w + scaler.s(6.0), left_y + scaler.s(15.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);

    // --- Right Column: Driving Style & Operationalized Parameters ---
    scaler.draw_glass_card(col2_x, content_y, col_w, col_h, Color::new(0.06, 0.08, 0.12, 0.85), Palette::UI_CARD_BORDER, 1.2);

    let mut right_y = content_y + scaler.s(24.0);
    fonts.draw_ui_bold("OPERATIONALIZED DRIVING STYLE", col2_x + scaler.s(16.0), right_y, scaler.font_s(15.0), Palette::NEON_GREEN);
    right_y += scaler.s(22.0);

    // Skill Stat Bars
    render_character_stat_bar(scaler, fonts, col2_x + scaler.s(16.0), right_y, "PACE & SPEED", driver.stats.speed, Palette::NEON_CYAN);
    right_y += scaler.s(28.0);
    render_character_stat_bar(scaler, fonts, col2_x + scaler.s(16.0), right_y, "OVERTAKE AGGRESSION", driver.stats.aggression, Palette::RED);
    right_y += scaler.s(28.0);
    render_character_stat_bar(scaler, fonts, col2_x + scaler.s(16.0), right_y, "APEX PRECISION", driver.stats.precision, Palette::NEON_GOLD);
    right_y += scaler.s(28.0);
    render_character_stat_bar(scaler, fonts, col2_x + scaler.s(16.0), right_y, "DEFENSIVE POSITION", driver.stats.defense, Palette::NEON_GREEN);
    right_y += scaler.s(36.0);

    // Operationalized AI Parameters Table
    fonts.draw_ui_bold("AI TELEMETRY PARAMETERS", col2_x + scaler.s(16.0), right_y, scaler.font_s(13.5), Palette::UI_TEXT_MUTED);
    right_y += scaler.s(20.0);

    let param_box_w = col_w - scaler.s(32.0);
    let param_box_h = scaler.s(72.0);
    draw_rectangle(col2_x + scaler.s(16.0), right_y, param_box_w, param_box_h, Color::new(0.10, 0.12, 0.17, 0.90));
    draw_rectangle_lines(col2_x + scaler.s(16.0), right_y, param_box_w, param_box_h, 1.0, Palette::UI_CARD_BORDER);

    let p1 = format!("• Corner Speed Factor: {:.2}x", driver.profile.speed_factor);
    let p2 = format!("• Lookahead Horizon: {:.2}s", driver.profile.lookahead_time);
    let p3 = format!("• Steering Gain (Kp): {:.1}", driver.profile.steering_kp);
    let p4 = format!("• Braking Distance Margin: {:.2}x", driver.profile.brake_margin);

    fonts.draw_ui_regular(&p1, col2_x + scaler.s(24.0), right_y + scaler.s(20.0), scaler.font_s(12.0), Color::new(0.85, 0.90, 0.96, 1.0));
    fonts.draw_ui_regular(&p2, col2_x + scaler.s(24.0), right_y + scaler.s(40.0), scaler.font_s(12.0), Color::new(0.85, 0.90, 0.96, 1.0));
    fonts.draw_ui_regular(&p3, col2_x + param_box_w * 0.5 + scaler.s(10.0), right_y + scaler.s(20.0), scaler.font_s(12.0), Color::new(0.85, 0.90, 0.96, 1.0));
    fonts.draw_ui_regular(&p4, col2_x + param_box_w * 0.5 + scaler.s(10.0), right_y + scaler.s(40.0), scaler.font_s(12.0), Color::new(0.85, 0.90, 0.96, 1.0));

    // Footer Navigation Controls
    let nav_prompt = "[LEFT / A] Previous Driver | [RIGHT / D] Next Driver | [ESC / SPACE] Close Dossier";
    fonts.draw_ui_bold_centered(
        nav_prompt,
        sw * 0.5,
        sh - scaler.s(20.0),
        scaler.font_s(15.0),
        Palette::WHITE,
    );
}

fn render_character_stat_bar(
    scaler: UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    label: &str,
    val: f32,
    color: Color,
) {
    fonts.draw_ui_bold(label, x, y + scaler.s(7.0), scaler.font_s(11.0), Palette::WHITE);
    let bar_w = scaler.s(140.0);
    let bar_h = scaler.s(10.0);
    let bar_x = x + scaler.s(160.0);

    draw_rectangle(bar_x, y, bar_w, bar_h, Color::new(0.12, 0.14, 0.20, 0.95));
    draw_rectangle(bar_x, y, bar_w * val.clamp(0.0, 1.0), bar_h, color);
    draw_rectangle_lines(bar_x, y, bar_w, bar_h, 1.0, Palette::UI_CARD_BORDER);

    let pct_str = format!("{:.0}%", val * 100.0);
    fonts.draw_ui_regular(&pct_str, bar_x + bar_w + scaler.s(8.0), y + scaler.s(8.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
}
