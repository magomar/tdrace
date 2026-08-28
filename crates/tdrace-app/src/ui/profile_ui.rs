use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::profile::{draw_country_banner, CountryRegistry, PlayerProfile, ProfileCareerStats, RaceHistoryEntry};
use crate::render::color::{CarColorScheme, Palette};

/// Renders the compact Player Profile Badge for the Main Menu.
pub fn render_profile_badge(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    profile: &PlayerProfile,
    stats: &ProfileCareerStats,
) {
    // Glass card backdrop
    scaler.draw_glass_card(x, y, w, h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.8);

    // Top Row: Country Banner, Name & Alias
    let banner_h = scaler.s(20.0);
    let banner_w = scaler.s(52.0);
    draw_country_banner(profile.country.as_deref(), x + scaler.s(12.0), y + scaler.s(7.0), banner_w, banner_h, Some(fonts), scaler);

    let name_str = format!("{} — \"{}\"", profile.name, profile.alias);
    fonts.draw_ui_bold(
        &name_str,
        x + banner_w + scaler.s(20.0),
        y + scaler.s(22.0),
        scaler.font_s(14.5),
        Palette::WHITE,
    );

    // Livery swatches on top right
    let swatch_w = scaler.s(14.0);
    let swatch_h = scaler.s(11.0);
    let swatch_x = x + w - scaler.s(72.0);
    let swatch_y = y + scaler.s(11.0);

    draw_rectangle(swatch_x, swatch_y, swatch_w, swatch_h, profile.color_scheme.primary);
    draw_rectangle_lines(swatch_x, swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    draw_rectangle(swatch_x + swatch_w + scaler.s(3.0), swatch_y, swatch_w, swatch_h, profile.color_scheme.secondary);
    draw_rectangle_lines(swatch_x + swatch_w + scaler.s(3.0), swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    draw_rectangle(swatch_x + (swatch_w + scaler.s(3.0)) * 2.0, swatch_y, swatch_w, swatch_h, profile.color_scheme.helmet);
    draw_rectangle_lines(swatch_x + (swatch_w + scaler.s(3.0)) * 2.0, swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    // Bottom Row: Career Summary Stats & Switch prompt
    let win_pct = stats.win_rate.round() as i32;
    let stats_line = format!(
        "Races: {}  •  Wins: {} ({}%)  •  Podiums: {}  •  Laps: {}",
        stats.total_races, stats.wins, win_pct, stats.podiums, stats.total_laps
    );
    fonts.draw_ui_regular(
        &stats_line,
        x + scaler.s(12.0),
        y + scaler.s(41.0),
        scaler.font_s(11.5),
        Palette::NEON_GOLD,
    );

    let action_prompt = "[P] PROFILE & HISTORY";
    let prompt_dim = fonts.measure_ui_bold(action_prompt, scaler.font_s(11.5));
    fonts.draw_ui_bold(
        action_prompt,
        x + w - prompt_dim.width - scaler.s(14.0),
        y + scaler.s(41.0),
        scaler.font_s(11.5),
        Palette::NEON_CYAN,
    );
}

/// Renders the full-screen Profile Management and Career History dossier.
pub fn render_profile_manager_screen(
    fonts: &Fonts,
    profiles: &[PlayerProfile],
    selected_idx: usize,
    history: &[RaceHistoryEntry],
    stats: &ProfileCareerStats,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep motorsport backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Header Title
    let title = "DRIVER PROFILES & CAREER HISTORY";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(38.0),
        scaler.font_s(30.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = "Driver Identity | National Banners | Custom Team Liveries | Career Race Telemetry";
    fonts.draw_ui_regular_centered(
        subtitle,
        sw * 0.5,
        scaler.s(60.0),
        scaler.font_s(14.0),
        Palette::UI_TEXT_MUTED,
    );

    let sel_profile = profiles.get(selected_idx).or_else(|| profiles.first());

    // Main Card Dimensions
    let box_w = (sw * 0.90).clamp(scaler.s(680.0), scaler.s(1040.0));
    let box_h = (sh * 0.76).clamp(scaler.s(440.0), scaler.s(620.0));
    let x = (sw - box_w) * 0.5;
    let y = scaler.s(74.0);

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.0);

    // Two-Column Layout
    let col_gap = scaler.s(16.0);
    let col1_w = (box_w * 0.36).clamp(scaler.s(220.0), scaler.s(340.0));
    let col2_w = box_w - col1_w - col_gap - scaler.s(24.0);
    let col1_x = x + scaler.s(12.0);
    let col2_x = col1_x + col1_w + col_gap;
    let col_h = box_h - scaler.s(24.0);
    let col_y = y + scaler.s(12.0);

    // --- Left Column: Profiles List ---
    scaler.draw_glass_card(col1_x, col_y, col1_w, col_h, Color::new(0.06, 0.08, 0.12, 0.90), Palette::UI_CARD_BORDER, 1.2);

    let mut list_y = col_y + scaler.s(22.0);
    fonts.draw_ui_bold("DRIVER ROSTER [Up/Down]", col1_x + scaler.s(14.0), list_y, scaler.font_s(14.0), Palette::NEON_CYAN);
    list_y += scaler.s(14.0);

    let item_h = scaler.s(52.0);
    for (i, p) in profiles.iter().enumerate() {
        let is_sel = i == selected_idx;
        let is_active = p.is_active;

        let item_bg = if is_sel {
            Palette::UI_CARD_BG_HOVER
        } else {
            Color::new(0.08, 0.10, 0.15, 0.70)
        };
        let item_border = if is_sel {
            Palette::NEON_CYAN
        } else if is_active {
            Palette::NEON_GOLD
        } else {
            Palette::UI_CARD_BORDER
        };

        draw_rectangle(col1_x + scaler.s(8.0), list_y, col1_w - scaler.s(16.0), item_h, item_bg);
        draw_rectangle_lines(col1_x + scaler.s(8.0), list_y, col1_w - scaler.s(16.0), item_h, if is_sel { 2.0 } else { 1.0 }, item_border);

        // Active indicator / index badge
        let badge_text = if is_active { "ACTIVE" } else { "IDLE" };
        let badge_col = if is_active { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED };
        fonts.draw_ui_bold(badge_text, col1_x + scaler.s(14.0), list_y + scaler.s(16.0), scaler.font_s(10.0), badge_col);

        // Country banner
        let b_w = scaler.s(36.0);
        let b_h = scaler.s(18.0);
        draw_country_banner(p.country.as_deref(), col1_x + col1_w - b_w - scaler.s(55.0), list_y + scaler.s(6.0), b_w, b_h, None, &scaler);

        // Livery swatches
        let sw_x = col1_x + col1_w - scaler.s(45.0);
        let sw_y = list_y + scaler.s(9.0);
        let sw_w = scaler.s(10.0);
        let sw_h = scaler.s(12.0);
        draw_rectangle(sw_x, sw_y, sw_w, sw_h, p.color_scheme.primary);
        draw_rectangle(sw_x + sw_w + scaler.s(2.0), sw_y, sw_w, sw_h, p.color_scheme.secondary);
        draw_rectangle(sw_x + (sw_w + scaler.s(2.0)) * 2.0, sw_y, sw_w, sw_h, p.color_scheme.helmet);

        // Profile Name / Alias
        let name_label = format!("{} (\"{}\")", p.name, p.alias);
        let text_col = if is_sel { Palette::WHITE } else { Color::new(0.85, 0.90, 0.95, 1.0) };
        fonts.draw_ui_bold(&name_label, col1_x + scaler.s(14.0), list_y + scaler.s(36.0), scaler.font_s(13.0), text_col);

        list_y += item_h + scaler.s(6.0);
    }

    // --- Right Column: Active/Selected Profile Dossier & Career History ---
    scaler.draw_glass_card(col2_x, col_y, col2_w, col_h, Color::new(0.06, 0.08, 0.12, 0.90), Palette::UI_CARD_BORDER, 1.2);

    if let Some(p) = sel_profile {
        let mut r_y = col_y + scaler.s(22.0);

        // Top Banner inside card
        let banner_box_h = scaler.s(58.0);
        draw_rectangle(col2_x + scaler.s(12.0), r_y, col2_w - scaler.s(24.0), banner_box_h, Color::new(0.10, 0.14, 0.22, 0.95));
        draw_rectangle_lines(col2_x + scaler.s(12.0), r_y, col2_w - scaler.s(24.0), banner_box_h, 1.5, Palette::UI_CARD_BORDER);

        // Country banner
        let cb_w = scaler.s(70.0);
        let cb_h = scaler.s(36.0);
        draw_country_banner(p.country.as_deref(), col2_x + scaler.s(24.0), r_y + scaler.s(11.0), cb_w, cb_h, Some(fonts), &scaler);

        // Name & Alias
        let title_name = format!("{} — \"{}\"", p.name, p.alias);
        fonts.draw_ui_bold(&title_name, col2_x + cb_w + scaler.s(40.0), r_y + scaler.s(28.0), scaler.font_s(18.0), Palette::WHITE);

        let country_desc = format!("Nationality: {}  •  Status: {}", p.country_name(), if p.is_active { "Active Primary Driver" } else { "Bench / Inactive" });
        fonts.draw_ui_regular(&country_desc, col2_x + cb_w + scaler.s(40.0), r_y + scaler.s(46.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);

        // Livery swatches on top right of banner
        let l_w = scaler.s(22.0);
        let l_h = scaler.s(18.0);
        let l_x = col2_x + col2_w - scaler.s(120.0);
        let l_y = r_y + scaler.s(18.0);

        draw_rectangle(l_x, l_y, l_w, l_h, p.color_scheme.primary);
        draw_rectangle_lines(l_x, l_y, l_w, l_h, 1.0, Palette::WHITE);
        draw_rectangle(l_x + l_w + scaler.s(4.0), l_y, l_w, l_h, p.color_scheme.secondary);
        draw_rectangle_lines(l_x + l_w + scaler.s(4.0), l_y, l_w, l_h, 1.0, Palette::WHITE);
        draw_rectangle(l_x + (l_w + scaler.s(4.0)) * 2.0, l_y, l_w, l_h, p.color_scheme.helmet);
        draw_rectangle_lines(l_x + (l_w + scaler.s(4.0)) * 2.0, l_y, l_w, l_h, 1.0, Palette::WHITE);

        fonts.draw_ui_regular("Team Livery", l_x, l_y + l_h + scaler.s(12.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);

        r_y += banner_box_h + scaler.s(14.0);

        // Career Statistics Summary Grid
        fonts.draw_ui_bold("CAREER STATISTICS", col2_x + scaler.s(14.0), r_y, scaler.font_s(13.5), Palette::NEON_GOLD);
        r_y += scaler.s(8.0);

        let stat_card_w = (col2_w - scaler.s(24.0) - scaler.s(18.0)) * 0.25;
        let stat_card_h = scaler.s(46.0);

        render_mini_stat_card(&scaler, fonts, col2_x + scaler.s(12.0), r_y, stat_card_w, stat_card_h, "RACES", &stats.total_races.to_string(), Palette::NEON_CYAN);
        render_mini_stat_card(&scaler, fonts, col2_x + scaler.s(12.0) + (stat_card_w + scaler.s(6.0)), r_y, stat_card_w, stat_card_h, "WINS (P1)", &stats.wins.to_string(), Palette::NEON_GREEN);
        let win_str = format!("{:.0}%", stats.win_rate);
        render_mini_stat_card(&scaler, fonts, col2_x + scaler.s(12.0) + (stat_card_w + scaler.s(6.0)) * 2.0, r_y, stat_card_w, stat_card_h, "WIN RATE", &win_str, Palette::NEON_GOLD);
        render_mini_stat_card(&scaler, fonts, col2_x + scaler.s(12.0) + (stat_card_w + scaler.s(6.0)) * 3.0, r_y, stat_card_w, stat_card_h, "PODIUMS", &stats.podiums.to_string(), Palette::NEON_MAGENTA);

        r_y += stat_card_h + scaler.s(16.0);

        // Recent Race History Table
        fonts.draw_ui_bold("RECENT RACE HISTORY & TELEMETRY", col2_x + scaler.s(14.0), r_y, scaler.font_s(13.5), Palette::NEON_CYAN);
        r_y += scaler.s(8.0);

        let table_w = col2_w - scaler.s(24.0);
        let table_x = col2_x + scaler.s(12.0);
        let hdr_h = scaler.s(22.0);

        draw_rectangle(table_x, r_y, table_w, hdr_h, Color::new(0.12, 0.16, 0.24, 0.95));
        draw_rectangle_lines(table_x, r_y, table_w, hdr_h, 1.0, Palette::UI_CARD_BORDER);

        fonts.draw_ui_bold("POS", table_x + scaler.s(10.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::WHITE);
        fonts.draw_ui_bold("TRACK", table_x + scaler.s(55.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::WHITE);
        fonts.draw_ui_bold("VEHICLE", table_x + scaler.s(190.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::WHITE);
        fonts.draw_ui_bold("TOTAL TIME", table_x + table_w - scaler.s(210.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::WHITE);
        fonts.draw_ui_bold("BEST LAP", table_x + table_w - scaler.s(120.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::WHITE);
        fonts.draw_ui_bold("DATE", table_x + table_w - scaler.s(45.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::WHITE);

        r_y += hdr_h + scaler.s(3.0);

        let row_h = scaler.s(22.0);
        if history.is_empty() {
            fonts.draw_ui_regular_centered(
                "No race records found for this driver profile yet. Complete a race to log telemetry!",
                table_x + table_w * 0.5,
                r_y + scaler.s(25.0),
                scaler.font_s(12.5),
                Palette::UI_TEXT_MUTED,
            );
        } else {
            for (idx, entry) in history.iter().take(6).enumerate() {
                let row_bg = if idx % 2 == 0 {
                    Color::new(0.08, 0.10, 0.15, 0.60)
                } else {
                    Color::new(0.06, 0.08, 0.12, 0.60)
                };

                draw_rectangle(table_x, r_y, table_w, row_h, row_bg);

                let pos_str = match entry.position {
                    1 => "P1".to_string(),
                    2 => "P2".to_string(),
                    3 => "P3".to_string(),
                    _ => format!("P{}", entry.position),
                };
                let pos_col = match entry.position {
                    1 => Palette::NEON_GOLD,
                    2 => Color::new(0.85, 0.88, 0.95, 1.0),
                    3 => Color::new(0.88, 0.55, 0.25, 1.0),
                    _ => Palette::UI_TEXT_MUTED,
                };
                fonts.draw_ui_bold(&pos_str, table_x + scaler.s(8.0), r_y + scaler.s(15.0), scaler.font_s(11.5), pos_col);

                let track_clean = entry.track_id.replace('_', " ");
                fonts.draw_ui_regular(&track_clean, table_x + scaler.s(55.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::WHITE);
                fonts.draw_ui_regular(&entry.car_name, table_x + scaler.s(190.0), r_y + scaler.s(15.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);

                let total_str = format_lap_time(entry.total_time);
                fonts.draw_ui_bold(&total_str, table_x + table_w - scaler.s(210.0), r_y + scaler.s(15.0), scaler.font_s(11.5), Palette::NEON_GREEN);

                let best_str = format_lap_time(entry.best_lap.unwrap_or(0.0));
                fonts.draw_ui_bold(&best_str, table_x + table_w - scaler.s(120.0), r_y + scaler.s(15.0), scaler.font_s(11.5), Palette::NEON_CYAN);

                let date_str = entry.created_at.split(' ').next().unwrap_or(&entry.created_at);
                fonts.draw_ui_regular(date_str, table_x + table_w - scaler.s(50.0), r_y + scaler.s(15.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);

                r_y += row_h + scaler.s(2.0);
            }
        }
    }

    // Footer Navigation & Actions
    let action_prompt = "[ENTER / A] Set Active  |  [E] Edit Profile  |  [N / X] New Profile  |  [DEL / Y] Delete Profile  |  [ESC / B] Main Menu";
    fonts.draw_ui_bold_centered(
        action_prompt,
        sw * 0.5,
        sh - scaler.s(18.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );
}

fn render_mini_stat_card(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    val: &str,
    val_col: Color,
) {
    draw_rectangle(x, y, w, h, Color::new(0.10, 0.12, 0.18, 0.90));
    draw_rectangle_lines(x, y, w, h, 1.0, Palette::UI_CARD_BORDER);

    fonts.draw_ui_bold(label, x + scaler.s(8.0), y + scaler.s(16.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold(val, x + scaler.s(8.0), y + scaler.s(36.0), scaler.font_s(16.0), val_col);
}

/// Renders the interactive Profile Creation or Editing Screen / Modal.
#[allow(clippy::too_many_arguments)]
pub fn render_profile_create_screen(
    fonts: &Fonts,
    active_field: usize,
    name_input: &str,
    alias_input: &str,
    country_idx: usize,
    livery_idx: usize,
    cursor_timer: f32,
    is_editing: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark backdrop overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.03, 0.04, 0.07, 0.95));

    let box_w = (sw * 0.75).clamp(scaler.s(480.0), scaler.s(680.0));
    let box_h = scaler.s(460.0);
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 2.2);

    // Modal Header
    let title = if is_editing {
        "EDIT DRIVER PROFILE"
    } else {
        "CREATE NEW DRIVER PROFILE"
    };
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        y + scaler.s(40.0),
        scaler.font_s(26.0),
        Palette::NEON_CYAN,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = if is_editing {
        "Modify driver identity, national banner, and team livery color scheme"
    } else {
        "Customize driver identity, national banner, and team livery color scheme"
    };
    fonts.draw_ui_regular_centered(
        subtitle,
        sw * 0.5,
        y + scaler.s(62.0),
        scaler.font_s(13.5),
        Palette::UI_TEXT_MUTED,
    );

    let mut field_y = y + scaler.s(90.0);
    let field_h = scaler.s(46.0);
    let field_w = box_w - scaler.s(60.0);
    let field_x = x + scaler.s(30.0);

    let show_cursor = (cursor_timer * 2.5).fract() < 0.5;

    // Field 0: Full Name
    let f0_sel = active_field == 0;
    render_text_field(
        &scaler,
        fonts,
        field_x,
        field_y,
        field_w,
        field_h,
        "DRIVER FULL NAME",
        name_input,
        f0_sel,
        show_cursor && f0_sel,
        "e.g. Mario Gomez",
    );
    field_y += field_h + scaler.s(22.0);

    // Field 1: Alias / Racing Handle
    let f1_sel = active_field == 1;
    render_text_field(
        &scaler,
        fonts,
        field_x,
        field_y,
        field_w,
        field_h,
        "RACING ALIAS / CALLSIGN",
        alias_input,
        f1_sel,
        show_cursor && f1_sel,
        "e.g. Apex Legend",
    );
    field_y += field_h + scaler.s(22.0);

    // Field 2: Nationality / Country Banner Selector
    let f2_sel = active_field == 2;
    let country_info = if country_idx > 0 && country_idx <= CountryRegistry::ALL.len() {
        Some(&CountryRegistry::ALL[country_idx - 1])
    } else {
        None
    };

    let country_title = country_info
        .map(|c| format!("{} ({})", c.name, c.code))
        .unwrap_or_else(|| "International / Worldwide (No Banner)".to_string());

    let country_border = if f2_sel { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
    draw_rectangle(field_x, field_y, field_w, field_h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(field_x, field_y, field_w, field_h, if f2_sel { 2.0 } else { 1.0 }, country_border);

    fonts.draw_ui_bold("NATIONALITY & BANNER [Left/Right]", field_x + scaler.s(12.0), field_y - scaler.s(5.0), scaler.font_s(11.0), if f2_sel { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED });

    let cb_w = scaler.s(54.0);
    let cb_h = scaler.s(26.0);
    let c_code = country_info.map(|c| c.code);
    draw_country_banner(c_code, field_x + scaler.s(16.0), field_y + scaler.s(10.0), cb_w, cb_h, Some(fonts), &scaler);

    fonts.draw_ui_bold(&country_title, field_x + cb_w + scaler.s(28.0), field_y + scaler.s(28.0), scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_regular("[Left / Right]", field_x + field_w - scaler.s(90.0), field_y + scaler.s(28.0), scaler.font_s(12.0), Palette::NEON_GOLD);

    field_y += field_h + scaler.s(22.0);

    // Field 3: Team Livery / Car Colors Selector
    let f3_sel = active_field == 3;
    let livery_border = if f3_sel { Palette::NEON_MAGENTA } else { Palette::UI_CARD_BORDER };
    draw_rectangle(field_x, field_y, field_w, field_h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(field_x, field_y, field_w, field_h, if f3_sel { 2.0 } else { 1.0 }, livery_border);

    fonts.draw_ui_bold("TEAM LIVERY & CAR COLORS [Left/Right]", field_x + scaler.s(12.0), field_y - scaler.s(5.0), scaler.font_s(11.0), if f3_sel { Palette::NEON_MAGENTA } else { Palette::UI_TEXT_MUTED });

    let scheme = CarColorScheme::from_index(livery_idx);
    let sw_w = scaler.s(28.0);
    let sw_h = scaler.s(20.0);
    let sw_x = field_x + scaler.s(16.0);
    let sw_y = field_y + scaler.s(13.0);

    draw_rectangle(sw_x, sw_y, sw_w, sw_h, scheme.primary);
    draw_rectangle_lines(sw_x, sw_y, sw_w, sw_h, 1.0, Palette::WHITE);

    draw_rectangle(sw_x + sw_w + scaler.s(6.0), sw_y, sw_w, sw_h, scheme.secondary);
    draw_rectangle_lines(sw_x + sw_w + scaler.s(6.0), sw_y, sw_w, sw_h, 1.0, Palette::WHITE);

    draw_rectangle(sw_x + (sw_w + scaler.s(6.0)) * 2.0, sw_y, sw_w, sw_h, scheme.helmet);
    draw_rectangle_lines(sw_x + (sw_w + scaler.s(6.0)) * 2.0, sw_y, sw_w, sw_h, 1.0, Palette::WHITE);

    let livery_name = format!("Livery Theme #{}", (livery_idx % Palette::CAR_COLORS.len()) + 1);
    fonts.draw_ui_bold(&livery_name, sw_x + (sw_w + scaler.s(6.0)) * 3.0 + scaler.s(10.0), field_y + scaler.s(28.0), scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_regular("[Left / Right]", field_x + field_w - scaler.s(90.0), field_y + scaler.s(28.0), scaler.font_s(12.0), Palette::NEON_MAGENTA);

    // Footer Prompts
    let help_line = if is_editing {
        "[TAB / UP / DOWN] Next Field  |  [ENTER / A] Save Changes  |  [ESC / B] Cancel"
    } else {
        "[TAB / UP / DOWN] Next Field  |  [ENTER / A] Save & Activate  |  [ESC / B] Cancel"
    };
    fonts.draw_ui_bold_centered(
        help_line,
        sw * 0.5,
        y + box_h - scaler.s(20.0),
        scaler.font_s(13.5),
        Palette::WHITE,
    );
}

fn render_text_field(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    text: &str,
    is_focused: bool,
    show_cursor: bool,
    placeholder: &str,
) {
    let border_col = if is_focused { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(x, y, w, h, if is_focused { 2.0 } else { 1.0 }, border_col);

    fonts.draw_ui_bold(label, x + scaler.s(12.0), y - scaler.s(5.0), scaler.font_s(11.0), if is_focused { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED });

    let font_size = scaler.font_s(15.0);
    let text_y = y + scaler.s(29.0);

    if text.is_empty() {
        fonts.draw_ui_regular(placeholder, x + scaler.s(16.0), text_y, font_size, Palette::UI_TEXT_MUTED);
        if show_cursor {
            let cursor_w = scaler.s(2.0);
            let cursor_h = scaler.s(20.0);
            let cursor_y = y + (h - cursor_h) * 0.5;
            draw_rectangle(x + scaler.s(16.0), cursor_y, cursor_w, cursor_h, Palette::NEON_CYAN);
        }
    } else {
        fonts.draw_ui_bold(text, x + scaler.s(16.0), text_y, font_size, Palette::WHITE);
        if show_cursor {
            let dim = fonts.measure_ui_bold(text, font_size);
            let cursor_w = scaler.s(2.0);
            let cursor_h = scaler.s(20.0);
            let cursor_x = x + scaler.s(16.0) + dim.width + scaler.s(2.0);
            let cursor_y = y + (h - cursor_h) * 0.5;
            draw_rectangle(cursor_x, cursor_y, cursor_w, cursor_h, Palette::NEON_CYAN);
        }
    }
}
