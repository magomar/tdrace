use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::profile::{draw_country_banner, CountryRegistry, PlayerProfile, ProfileCareerStats, RaceHistoryEntry};
use crate::render::color::{CarColorScheme, Palette};

/// Renders the Player Profile Badge for menus, supporting both compact mode and enlarged navigable card mode.
pub fn render_profile_badge(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    profile: &PlayerProfile,
    stats: &ProfileCareerStats,
    is_selected: bool,
) {
    let is_compact = h < scaler.s(56.0);
    let bg_col = if is_selected {
        Palette::UI_CARD_BG_HOVER
    } else {
        Palette::UI_CARD_BG
    };
    let border_col = if is_selected {
        Palette::NEON_CYAN
    } else {
        Palette::UI_CARD_BORDER
    };
    let border_thickness = if is_selected { 2.4 } else { 1.2 };

    // Glass card backdrop
    scaler.draw_glass_card(x, y, w, h, bg_col, border_col, border_thickness);

    // Left accent bar when selected
    if is_selected {
        draw_rectangle(x, y, scaler.s(6.0), h, Palette::NEON_CYAN);
    }

    let pad_left = if is_selected { scaler.s(16.0) } else { scaler.s(12.0) };

    // Top Row: Country Banner, Name & Alias
    let banner_h = if is_compact { scaler.s(20.0) } else { scaler.s(22.0) };
    let banner_w = if is_compact { scaler.s(52.0) } else { scaler.s(56.0) };
    let banner_y = if is_compact { y + scaler.s(7.0) } else { y + scaler.s(11.0) };
    draw_country_banner(profile.country.as_deref(), x + pad_left, banner_y, banner_w, banner_h, Some(fonts), scaler);

    let name_str = format!("{} — \"{}\"", profile.name, profile.alias);
    let name_y = if is_compact { y + scaler.s(22.0) } else { y + scaler.s(27.0) };
    let name_size = if is_compact { scaler.font_s(14.5) } else { scaler.font_s(17.0) };
    fonts.draw_display(
        &name_str,
        x + pad_left + banner_w + scaler.s(16.0),
        name_y,
        name_size,
        if is_selected { Palette::WHITE } else { Color::new(0.88, 0.92, 0.97, 1.0) },
    );

    // Livery swatches on top right
    let swatch_w = if is_compact { scaler.s(14.0) } else { scaler.s(16.0) };
    let swatch_h = if is_compact { scaler.s(11.0) } else { scaler.s(13.0) };
    let swatch_x = x + w - scaler.s(if is_compact { 72.0 } else { 80.0 });
    let swatch_y = if is_compact { y + scaler.s(11.0) } else { y + scaler.s(13.0) };

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
    let bottom_y = if is_compact { y + scaler.s(41.0) } else { y + scaler.s(53.0) };
    fonts.draw_ui_regular(
        &stats_line,
        x + pad_left,
        bottom_y,
        if is_compact { scaler.font_s(11.5) } else { scaler.font_s(12.5) },
        Palette::NEON_GOLD,
    );

    let action_prompt = if is_selected {
        "PRESS [ENTER] TO MANAGE PROFILE ▶"
    } else {
        "[P] PROFILE & HISTORY"
    };
    let prompt_dim = fonts.measure_ui_bold(action_prompt, scaler.font_s(11.5));
    fonts.draw_ui_bold(
        action_prompt,
        x + w - prompt_dim.width - scaler.s(14.0),
        bottom_y,
        scaler.font_s(11.5),
        Palette::NEON_CYAN,
    );
}

/// Category filter options for telemetry tab
pub const TELEMETRY_CATEGORY_FILTERS: &[(&str, Option<&str>)] = &[
    ("ALL", None),
    ("GT", Some("gt")),
    ("NASCAR", Some("nascar")),
    ("RALLY", Some("rally")),
    ("OFF-ROAD", Some("extreme_offroad")),
    ("KART", Some("kart")),
    ("CLASSIC", Some("classic")),
];

/// Renders the full-screen Option A: Tabbed Motorsport Telemetry Dashboard.
pub fn render_profile_manager_screen(
    fonts: &Fonts,
    profiles: &[PlayerProfile],
    selected_idx: usize,
    history: &[RaceHistoryEntry],
    stats: &ProfileCareerStats,
    active_tab: usize,
    filter_category_idx: usize,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep motorsport dark backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.03, 0.04, 0.07, 0.98));

    let sel_profile = profiles.get(selected_idx).or_else(|| profiles.first());

    // Full-screen width layout: 96% of screen width, centered
    let full_w = (sw * 0.96).max(scaler.s(720.0));
    let x = (sw - full_w) * 0.5;
    let mut cur_y = scaler.s(16.0);

    // =========================================================================
    // 1. TOP FULL-WIDTH HERO DRIVER BANNER WITH INLINE SWITCHER [◄ Q / E ►]
    // =========================================================================
    let hero_h = scaler.s(74.0);
    scaler.draw_glass_card(x, cur_y, full_w, hero_h, Color::new(0.06, 0.08, 0.13, 0.95), Palette::NEON_CYAN, 1.8);

    if let Some(p) = sel_profile {
        let left_pad = scaler.s(14.0);

        // Inline Driver Cycler left button [◄ Q]
        let btn_w = scaler.s(44.0);
        let btn_h = scaler.s(32.0);
        let btn_y = cur_y + (hero_h - btn_h) * 0.5;
        draw_rectangle(x + left_pad, btn_y, btn_w, btn_h, Color::new(0.12, 0.16, 0.24, 0.90));
        draw_rectangle_lines(x + left_pad, btn_y, btn_w, btn_h, 1.2, Palette::NEON_CYAN);
        fonts.draw_ui_bold_centered("◄ Q", x + left_pad + btn_w * 0.5, btn_y + scaler.s(21.0), scaler.font_s(13.0), Palette::NEON_CYAN);

        // National Flag Banner
        let flag_x = x + left_pad + btn_w + scaler.s(12.0);
        let flag_w = scaler.s(60.0);
        let flag_h = scaler.s(32.0);
        draw_country_banner(p.country.as_deref(), flag_x, btn_y, flag_w, flag_h, Some(fonts), &scaler);

        // Driver Identity: Name & Alias
        let name_x = flag_x + flag_w + scaler.s(14.0);
        let driver_title = format!("{}  \"{}\"", p.name.to_uppercase(), p.alias);
        fonts.draw_display(
            &driver_title,
            name_x,
            cur_y + scaler.s(32.0),
            scaler.font_s(21.0),
            Palette::WHITE,
        );

        // Inline Driver Cycler right button [E ►]
        let title_dim = fonts.measure_display(&driver_title, scaler.font_s(21.0));
        let btn2_x = name_x + title_dim.width + scaler.s(12.0);
        draw_rectangle(btn2_x, btn_y, btn_w, btn_h, Color::new(0.12, 0.16, 0.24, 0.90));
        draw_rectangle_lines(btn2_x, btn_y, btn_w, btn_h, 1.2, Palette::NEON_CYAN);
        fonts.draw_ui_bold_centered("E ►", btn2_x + btn_w * 0.5, btn_y + scaler.s(21.0), scaler.font_s(13.0), Palette::NEON_CYAN);

        // Driver Metadata Subtitle
        let status_desc = if p.is_active { "PRIMARY ACTIVE DRIVER" } else { "BENCH DRIVER" };
        let meta_str = format!(
            "NATIONALITY: {}  •  PROFILE #{} OF {}  •  STATUS: {}",
            p.country_name().to_uppercase(),
            selected_idx + 1,
            profiles.len().max(1),
            status_desc
        );
        fonts.draw_ui_regular(
            &meta_str,
            name_x,
            cur_y + scaler.s(55.0),
            scaler.font_s(11.5),
            if p.is_active { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED },
        );

        // Right side: Driver Level & Team Livery
        let right_pad = scaler.s(16.0);
        let right_x = x + full_w - right_pad;

        // Active Badge
        let badge_w = scaler.s(105.0);
        let badge_h = scaler.s(24.0);
        let badge_x = right_x - badge_w;
        let badge_y = cur_y + scaler.s(12.0);
        let (badge_bg, badge_border, badge_text, badge_col) = if p.is_active {
            (Color::new(0.08, 0.25, 0.12, 0.85), Palette::NEON_GREEN, "ACTIVE RACER", Palette::NEON_GREEN)
        } else {
            (Color::new(0.15, 0.18, 0.22, 0.80), Palette::UI_CARD_BORDER, "INACTIVE", Palette::UI_TEXT_MUTED)
        };
        draw_rectangle(badge_x, badge_y, badge_w, badge_h, badge_bg);
        draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.2, badge_border);
        fonts.draw_ui_bold_centered(badge_text, badge_x + badge_w * 0.5, badge_y + scaler.s(16.0), scaler.font_s(10.5), badge_col);

        // Team Livery Swatches
        let sw_w = scaler.s(16.0);
        let sw_h = scaler.s(16.0);
        let sw_y = cur_y + scaler.s(45.0);
        let sw3_x = right_x - sw_w;
        let sw2_x = sw3_x - sw_w - scaler.s(4.0);
        let sw1_x = sw2_x - sw_w - scaler.s(4.0);

        draw_rectangle(sw1_x, sw_y, sw_w, sw_h, p.color_scheme.primary);
        draw_rectangle_lines(sw1_x, sw_y, sw_w, sw_h, 1.0, Palette::WHITE);
        draw_rectangle(sw2_x, sw_y, sw_w, sw_h, p.color_scheme.secondary);
        draw_rectangle_lines(sw2_x, sw_y, sw_w, sw_h, 1.0, Palette::WHITE);
        draw_rectangle(sw3_x, sw_y, sw_w, sw_h, p.color_scheme.helmet);
        draw_rectangle_lines(sw3_x, sw_y, sw_w, sw_h, 1.0, Palette::WHITE);

        fonts.draw_ui_regular("LIVERY:", sw1_x - scaler.s(52.0), sw_y + scaler.s(12.5), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

        // Driver Global Level & XP
        let driver_lvl = (stats.total_races / 2 + stats.wins).max(1);
        let xp_in_level = (stats.total_stunt_score + stats.total_laps * 50) % 1000;
        let lvl_str = format!("DRIVER LVL {}", driver_lvl);
        let lvl_x = sw1_x - scaler.s(180.0);
        fonts.draw_display(&lvl_str, lvl_x, cur_y + scaler.s(30.0), scaler.font_s(16.0), Palette::NEON_GOLD);

        // Mini XP Bar
        let bar_w = scaler.s(100.0);
        let bar_h = scaler.s(8.0);
        let bar_y = cur_y + scaler.s(42.0);
        draw_rectangle(lvl_x, bar_y, bar_w, bar_h, Color::new(0.12, 0.15, 0.20, 0.90));
        let progress = (xp_in_level as f32 / 1000.0).clamp(0.05, 1.0);
        draw_rectangle(lvl_x, bar_y, bar_w * progress, bar_h, Palette::NEON_GOLD);
        draw_rectangle_lines(lvl_x, bar_y, bar_w, bar_h, 1.0, Palette::UI_CARD_BORDER);

        let xp_lbl = format!("{}/1000 XP", xp_in_level);
        fonts.draw_ui_regular(&xp_lbl, lvl_x, cur_y + scaler.s(60.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
    }

    cur_y += hero_h + scaler.s(8.0);

    // =========================================================================
    // 2. WIDESCREEN TAB BAR [1] OVERVIEW | [2] CAREERS | [3] CHAMPS | [4] LOGS
    // =========================================================================
    let tab_bar_h = scaler.s(34.0);
    let tab_names = [
        "[1] GLOBAL OVERVIEW",
        "[2] CAREER DISCIPLINES",
        "[3] CHAMPIONSHIPS",
        "[4] RACE TELEMETRY & LOGS",
    ];
    let tab_gap = scaler.s(8.0);
    let total_gaps = tab_gap * (tab_names.len() as f32 - 1.0);
    let tab_w = ((full_w - scaler.s(220.0) - total_gaps) / tab_names.len() as f32).max(scaler.s(110.0));

    for (i, name) in tab_names.iter().enumerate() {
        let t_x = x + i as f32 * (tab_w + tab_gap);
        let is_active = i == active_tab;

        let bg_col = if is_active {
            Palette::UI_CARD_BG_HOVER
        } else {
            Color::new(0.06, 0.08, 0.12, 0.70)
        };
        let border_col = if is_active {
            Palette::NEON_CYAN
        } else {
            Palette::UI_CARD_BORDER
        };
        let text_col = if is_active {
            Palette::WHITE
        } else {
            Palette::UI_TEXT_MUTED
        };

        draw_rectangle(t_x, cur_y, tab_w, tab_bar_h, bg_col);
        draw_rectangle_lines(t_x, cur_y, tab_w, tab_bar_h, if is_active { 2.0 } else { 1.0 }, border_col);

        if is_active {
            // Bottom glowing neon accent bar
            draw_rectangle(t_x, cur_y + tab_bar_h - scaler.s(3.0), tab_w, scaler.s(3.0), Palette::NEON_CYAN);
        }

        fonts.draw_ui_bold_centered(name, t_x + tab_w * 0.5, cur_y + scaler.s(22.0), scaler.font_s(11.5), text_col);
    }

    // Tab switch prompt on the far right
    let tab_hint = "[TAB / 1-4] SWITCH TAB";
    fonts.draw_ui_bold(
        tab_hint,
        x + full_w - scaler.s(170.0),
        cur_y + scaler.s(22.0),
        scaler.font_s(11.0),
        Palette::NEON_GOLD,
    );

    cur_y += tab_bar_h + scaler.s(8.0);

    // =========================================================================
    // 3. MAIN CONTENT CONTAINER
    // =========================================================================
    let footer_h = scaler.s(36.0);
    let content_h = (sh - cur_y - footer_h - scaler.s(10.0)).max(scaler.s(360.0));
    scaler.draw_glass_card(x, cur_y, full_w, content_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    match active_tab {
        0 => render_overview_tab(&scaler, fonts, x, cur_y, full_w, content_h, stats),
        1 => render_disciplines_tab(&scaler, fonts, x, cur_y, full_w, content_h, stats),
        2 => render_championships_tab(&scaler, fonts, x, cur_y, full_w, content_h, stats),
        3 => render_telemetry_tab(&scaler, fonts, x, cur_y, full_w, content_h, history, filter_category_idx),
        _ => render_overview_tab(&scaler, fonts, x, cur_y, full_w, content_h, stats),
    }

    // =========================================================================
    // 4. FOOTER ACTION BAR
    // =========================================================================
    let foot_y = sh - scaler.s(20.0);
    let footer_prompt = "[◄ Q / E ►] Cycle Driver  |  [TAB / 1-4] Tabs  |  [ENTER] Set Active  |  [N] New Driver  |  [DEL / X] Delete  |  [C] Clear History  |  [ESC] Exit";
    fonts.draw_ui_bold_centered(
        footer_prompt,
        sw * 0.5,
        foot_y,
        scaler.font_s(12.5),
        Palette::WHITE,
    );
}

// =============================================================================
// TAB 0: GLOBAL OVERVIEW (6 KPI TILES + STUNTS + INCIDENTS + TROPHIES)
// =============================================================================
fn render_overview_tab(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    stats: &ProfileCareerStats,
) {
    let pad = scaler.s(16.0);
    let inner_w = w - pad * 2.0;
    let mut cy = y + pad;

    // --- Top Row: 6 KPI Stat Tiles ---
    let tile_gap = scaler.s(10.0);
    let tile_w = (inner_w - tile_gap * 5.0) / 6.0;
    let tile_h = scaler.s(54.0);

    let win_str = format!("{} ({:.0}%)", stats.wins, stats.win_rate);
    let podium_str = format!("{} ({:.0}%)", stats.podiums, stats.podium_rate);
    let clean_str = format!("{:.1}%", stats.clean_rate);

    render_kpi_tile(scaler, fonts, x + pad, cy, tile_w, tile_h, "TOTAL RACES", &stats.total_races.to_string(), Palette::NEON_CYAN);
    render_kpi_tile(scaler, fonts, x + pad + (tile_w + tile_gap), cy, tile_w, tile_h, "WINS (P1)", &win_str, Palette::NEON_GOLD);
    render_kpi_tile(scaler, fonts, x + pad + (tile_w + tile_gap) * 2.0, cy, tile_w, tile_h, "P2 RUNNER-UP", &stats.p2_count.to_string(), Color::new(0.85, 0.88, 0.95, 1.0));
    render_kpi_tile(scaler, fonts, x + pad + (tile_w + tile_gap) * 3.0, cy, tile_w, tile_h, "P3 THIRD PLACE", &stats.p3_count.to_string(), Color::new(0.88, 0.55, 0.25, 1.0));
    render_kpi_tile(scaler, fonts, x + pad + (tile_w + tile_gap) * 4.0, cy, tile_w, tile_h, "PODIUMS (P1-P3)", &podium_str, Palette::NEON_GREEN);
    render_kpi_tile(scaler, fonts, x + pad + (tile_w + tile_gap) * 5.0, cy, tile_w, tile_h, "CLEAN RACE %", &clean_str, Palette::NEON_MAGENTA);

    cy += tile_h + scaler.s(14.0);

    // --- Middle Row: Stunt Portfolio & Safety / Incident Dossier ---
    let col_gap = scaler.s(16.0);
    let col_w = (inner_w - col_gap) * 0.5;
    let col_h = (h - (cy - y) - scaler.s(14.0)).max(scaler.s(220.0));

    // Left Box: Acrobatic Stunt Portfolio
    let left_x = x + pad;
    scaler.draw_glass_card(left_x, cy, col_w, col_h, Color::new(0.06, 0.08, 0.12, 0.85), Palette::NEON_GOLD, 1.2);
    fonts.draw_ui_bold("ACROBATIC STUNT & DRIFT PORTFOLIO", left_x + scaler.s(14.0), cy + scaler.s(24.0), scaler.font_s(14.0), Palette::NEON_GOLD);

    let mut s_y = cy + scaler.s(44.0);
    let row_h = scaler.s(24.0);

    render_data_row(scaler, fonts, left_x + scaler.s(14.0), s_y, col_w - scaler.s(28.0), "Total Accumulated Stunts", &format!("{} PTS", stats.total_stunt_score), Palette::NEON_GOLD);
    s_y += row_h;
    render_data_row(scaler, fonts, left_x + scaler.s(14.0), s_y, col_w - scaler.s(28.0), "Peak Single-Race Stunt Score", &format!("{} PTS", stats.max_stunt_score), Palette::WHITE);
    s_y += row_h;
    let avg_stunt = if stats.total_races > 0 { stats.total_stunt_score / stats.total_races } else { 0 };
    render_data_row(scaler, fonts, left_x + scaler.s(14.0), s_y, col_w - scaler.s(28.0), "Average Stunt Score / Race", &format!("{} PTS", avg_stunt), Palette::NEON_CYAN);
    s_y += row_h;
    render_data_row(scaler, fonts, left_x + scaler.s(14.0), s_y, col_w - scaler.s(28.0), "Total Laps Under Telemetry", &format!("{} LAPS", stats.total_laps), Palette::WHITE);
    s_y += row_h;
    let stunt_title = if stats.total_stunt_score >= 10000 {
        "LEGENDARY STUNTMAN"
    } else if stats.total_stunt_score >= 2500 {
        "PRO DRIFTER"
    } else if stats.total_stunt_score > 0 {
        "STUNT ENTHUSIAST"
    } else {
        "NOVICE ROOKIE"
    };
    render_data_row(scaler, fonts, left_x + scaler.s(14.0), s_y, col_w - scaler.s(28.0), "Acrobatic Mastery Level", stunt_title, Palette::NEON_GREEN);
    s_y += row_h;
    render_data_row(scaler, fonts, left_x + scaler.s(14.0), s_y, col_w - scaler.s(28.0), "Circuits with Best Lap Records", &format!("{} TRACKS", stats.best_times.len()), Palette::WHITE);
    s_y += row_h;
    render_data_row(scaler, fonts, left_x + scaler.s(14.0), s_y, col_w - scaler.s(28.0), "Total Championship Podiums", &format!("{} PODIUMS", stats.podiums), Palette::NEON_MAGENTA);

    // Right Box: Safety & Incident Record + Trophy Cabinet
    let right_x = left_x + col_w + col_gap;
    scaler.draw_glass_card(right_x, cy, col_w, col_h, Color::new(0.06, 0.08, 0.12, 0.85), Palette::NEON_CYAN, 1.2);
    fonts.draw_ui_bold("SAFETY, INCIDENTS & TROPHY CABINET", right_x + scaler.s(14.0), cy + scaler.s(24.0), scaler.font_s(14.0), Palette::NEON_CYAN);

    let mut i_y = cy + scaler.s(44.0);
    render_data_row(scaler, fonts, right_x + scaler.s(14.0), i_y, col_w - scaler.s(28.0), "Total Collisions (Walls & Vehicles)", &format!("{} IMPACTS", stats.total_collisions), if stats.total_collisions == 0 { Palette::NEON_GREEN } else { Color::new(0.95, 0.45, 0.35, 1.0) });
    i_y += row_h;
    render_data_row(scaler, fonts, right_x + scaler.s(14.0), i_y, col_w - scaler.s(28.0), "Clean Incident-Free Races", &format!("{} / {}", stats.clean_races, stats.total_races), Palette::NEON_GREEN);
    i_y += row_h;
    let collision_freq = if stats.total_races > 0 { stats.total_collisions as f32 / stats.total_races as f32 } else { 0.0 };
    render_data_row(scaler, fonts, right_x + scaler.s(14.0), i_y, col_w - scaler.s(28.0), "Average Incident Frequency", &format!("{:.2} / race", collision_freq), Palette::WHITE);
    i_y += row_h;

    // Safety Rating
    let (rating_str, rating_col) = if stats.clean_rate >= 75.0 {
        ("GRADE S (Apex Master)", Palette::NEON_GREEN)
    } else if stats.clean_rate >= 50.0 {
        ("GRADE A (Pro Racer)", Palette::NEON_CYAN)
    } else if stats.clean_rate >= 25.0 {
        ("GRADE B (Club Driver)", Palette::NEON_GOLD)
    } else {
        ("GRADE C (Rookie)", Color::new(0.95, 0.45, 0.35, 1.0))
    };
    render_data_row(scaler, fonts, right_x + scaler.s(14.0), i_y, col_w - scaler.s(28.0), "Driver Safety License Grade", rating_str, rating_col);
    i_y += row_h + scaler.s(10.0);

    // Mini Trophy Showcase Bar inside Right Box
    draw_rectangle(right_x + scaler.s(14.0), i_y, col_w - scaler.s(28.0), scaler.s(48.0), Color::new(0.10, 0.14, 0.20, 0.90));
    draw_rectangle_lines(right_x + scaler.s(14.0), i_y, col_w - scaler.s(28.0), scaler.s(48.0), 1.0, Palette::UI_CARD_BORDER);

    let trophy_item_w = (col_w - scaler.s(28.0)) / 3.0;
    fonts.draw_ui_bold_centered(&format!("🏆 GOLD: {}", stats.wins), right_x + scaler.s(14.0) + trophy_item_w * 0.5, i_y + scaler.s(29.0), scaler.font_s(13.0), Palette::NEON_GOLD);
    fonts.draw_ui_bold_centered(&format!("🥈 SILVER: {}", stats.p2_count), right_x + scaler.s(14.0) + trophy_item_w * 1.5, i_y + scaler.s(29.0), scaler.font_s(13.0), Color::new(0.85, 0.88, 0.95, 1.0));
    fonts.draw_ui_bold_centered(&format!("🥉 BRONZE: {}", stats.p3_count), right_x + scaler.s(14.0) + trophy_item_w * 2.5, i_y + scaler.s(29.0), scaler.font_s(13.0), Color::new(0.88, 0.55, 0.25, 1.0));
}

// =============================================================================
// TAB 1: CAREER DISCIPLINES (6-CATEGORY GRID: GT, NASCAR, RALLY, OFF-ROAD, KART, CLASSIC)
// =============================================================================
fn render_disciplines_tab(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    stats: &ProfileCareerStats,
) {
    let pad = scaler.s(16.0);
    let inner_w = w - pad * 2.0;
    let inner_h = h - pad * 2.0;

    let categories = [
        ("gt", "GT & ENDURANCE RACING", "Grand Touring / Sports Cars", Palette::NEON_CYAN, 5),
        ("nascar", "NASCAR CUP SERIES", "Stock Cars / Oval & High-Speed Speedway", Palette::NEON_GOLD, 4),
        ("rally", "WORLD RALLYCROSS", "Multi-Surface Asphalt, Gravel & Stage Rally", Palette::NEON_GREEN, 4),
        ("extreme_offroad", "EXTREME OFF-ROAD", "Sand Rail Buggies / Desert Stunt Arena", Color::new(0.95, 0.55, 0.20, 1.0), 4),
        ("kart", "SUPERKART PRO SERIES", "CIK-FIA Benchmark Sprint Kart Circuits", Palette::NEON_MAGENTA, 4),
        ("classic", "VINTAGE CLASSIC LEGENDS", "Historic Grand Prix & Legendary Roadsters", Color::new(0.80, 0.85, 0.95, 1.0), 3),
    ];

    let cols = 3;
    let rows = 2;
    let gap = scaler.s(12.0);
    let card_w = (inner_w - gap * (cols as f32 - 1.0)) / cols as f32;
    let card_h = (inner_h - gap * (rows as f32 - 1.0)) / rows as f32;

    for (idx, (cat_key, cat_title, cat_subtitle, accent_col, max_cars)) in categories.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let cx = x + pad + col as f32 * (card_w + gap);
        let cy = y + pad + row as f32 * (card_h + gap);

        scaler.draw_glass_card(cx, cy, card_w, card_h, Color::new(0.06, 0.08, 0.12, 0.90), *accent_col, 1.2);

        // Header Accent Bar
        draw_rectangle(cx, cy, card_w, scaler.s(4.0), *accent_col);

        // Discipline Name & Subtitle
        fonts.draw_ui_bold(cat_title, cx + scaler.s(12.0), cy + scaler.s(22.0), scaler.font_s(13.0), Palette::WHITE);
        fonts.draw_ui_regular(cat_subtitle, cx + scaler.s(12.0), cy + scaler.s(36.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);

        // Fetch category stats
        let cat_stat = stats.category_stats.get(*cat_key);
        let races = cat_stat.map(|s| s.total_races).unwrap_or(0);
        let wins = cat_stat.map(|s| s.wins).unwrap_or(0);
        let win_pct = cat_stat.map(|s| s.win_rate).unwrap_or(0.0);
        let stunts = cat_stat.map(|s| s.total_stunt_score).unwrap_or(0);
        let clean_pct = cat_stat.map(|s| s.clean_rate).unwrap_or(0.0);

        // Tier Level & Progress
        let tier = (races / 3 + wins).clamp(1, 5);
        let tier_str = format!("TIER {}", tier);
        fonts.draw_ui_bold(&tier_str, cx + card_w - scaler.s(60.0), cy + scaler.s(22.0), scaler.font_s(11.5), *accent_col);

        // Mini XP Bar towards next tier car
        let xp_progress = ((races * 150 + wins * 300) % 1000) as f32 / 1000.0;
        let bar_x = cx + scaler.s(12.0);
        let bar_y = cy + scaler.s(48.0);
        let bar_w = card_w - scaler.s(24.0);
        draw_rectangle(bar_x, bar_y, bar_w, scaler.s(5.0), Color::new(0.12, 0.15, 0.20, 0.90));
        draw_rectangle(bar_x, bar_y, bar_w * xp_progress.clamp(0.05, 1.0), scaler.s(5.0), *accent_col);

        // Stats Matrix
        let mut sy = cy + scaler.s(72.0);
        let row_spacing = scaler.s(20.0);

        let row1_left = format!("Races: {}", races);
        let row1_right = format!("Wins: {} ({:.0}%)", wins, win_pct);
        fonts.draw_ui_bold(&row1_left, bar_x, sy, scaler.font_s(11.0), Palette::WHITE);
        fonts.draw_ui_bold(&row1_right, bar_x + bar_w * 0.5, sy, scaler.font_s(11.0), Palette::NEON_GOLD);

        sy += row_spacing;
        let row2_left = format!("Stunts: {} pts", stunts);
        let row2_right = format!("Clean: {:.0}%", clean_pct);
        fonts.draw_ui_regular(&row2_left, bar_x, sy, scaler.font_s(10.5), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_regular(&row2_right, bar_x + bar_w * 0.5, sy, scaler.font_s(10.5), Palette::NEON_GREEN);

        sy += row_spacing;
        let unlocked = (tier as usize).min(*max_cars);
        let garage_str = format!("Garage: {}/{} Cars Unlocked", unlocked, max_cars);
        fonts.draw_ui_regular(&garage_str, bar_x, sy, scaler.font_s(10.5), *accent_col);
    }
}

// =============================================================================
// TAB 2: CHAMPIONSHIPS REGISTRY & STANDINGS
// =============================================================================
fn render_championships_tab(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    stats: &ProfileCareerStats,
) {
    let pad = scaler.s(16.0);
    let inner_w = w - pad * 2.0;
    let mut cy = y + pad;

    fonts.draw_ui_bold("CHAMPIONSHIP REGISTRY & MOTORSPORT TROPHIES", x + pad, cy + scaler.s(16.0), scaler.font_s(15.0), Palette::NEON_GOLD);
    fonts.draw_ui_regular("Official multi-round tournaments across circuit and stage disciplines", x + pad, cy + scaler.s(32.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);

    cy += scaler.s(44.0);

    let championships = [
        ("gt", "GT World Challenge Championship", "3 Rounds: Monza GP • Spa Francorchamps • Circuit de la Sarthe", 3),
        ("nascar", "NASCAR Cup Series Championship", "4 Rounds: Daytona Tri-Oval • Talladega • Bristol Short Track • Charlotte", 4),
        ("rally", "WRC Alpine Stages Championship", "3 Rounds: Monte Carlo Tarmac • Finnish Gravel • Acropolis Mountain", 3),
        ("extreme_offroad", "Baja 1000 Desert Dune Trophy", "3 Rounds: Baja 1000 Dunes • Canyon Jump Run • Stunt Arena Freestyle", 3),
        ("kart", "CIK-FIA Karting World Cup", "3 Rounds: South Garda • Franciacorta • Campillos Kartcenter", 3),
        ("classic", "Historic Grand Prix Legends Cup", "2 Rounds: Historic Monaco • Monza Retro 1966 Banking", 2),
    ];

    let item_h = scaler.s(48.0);
    let item_gap = scaler.s(8.0);

    for (cat_key, champ_name, champ_desc, rounds) in championships.iter() {
        scaler.draw_glass_card(x + pad, cy, inner_w, item_h, Color::new(0.06, 0.08, 0.12, 0.90), Palette::UI_CARD_BORDER, 1.0);

        let cat_stat = stats.category_stats.get(*cat_key);
        let wins = cat_stat.map(|s| s.wins).unwrap_or(0);
        let podiums = cat_stat.map(|s| s.podiums).unwrap_or(0);

        // Status Badge
        let (status_text, status_col, status_bg) = if wins > 0 {
            ("CHAMPION [GOLD 🏆]", Palette::NEON_GOLD, Color::new(0.25, 0.20, 0.05, 0.90))
        } else if podiums > 0 {
            ("PODIUM FINISHER [SILVER 🥈]", Color::new(0.85, 0.88, 0.95, 1.0), Color::new(0.12, 0.16, 0.24, 0.90))
        } else {
            ("AVAILABLE TO ENTER", Palette::NEON_CYAN, Color::new(0.08, 0.14, 0.20, 0.90))
        };

        let badge_w = scaler.s(160.0);
        let badge_x = x + pad + inner_w - badge_w - scaler.s(12.0);
        let badge_y = cy + scaler.s(12.0);
        draw_rectangle(badge_x, badge_y, badge_w, scaler.s(24.0), status_bg);
        draw_rectangle_lines(badge_x, badge_y, badge_w, scaler.s(24.0), 1.0, status_col);
        fonts.draw_ui_bold_centered(status_text, badge_x + badge_w * 0.5, badge_y + scaler.s(16.5), scaler.font_s(10.5), status_col);

        // Championship Title & Rounds
        fonts.draw_ui_bold(champ_name, x + pad + scaler.s(14.0), cy + scaler.s(20.0), scaler.font_s(13.0), Palette::WHITE);
        fonts.draw_ui_regular(champ_desc, x + pad + scaler.s(14.0), cy + scaler.s(36.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

        // Round count tag
        let rnd_tag = format!("{} ROUNDS", rounds);
        fonts.draw_ui_bold(&rnd_tag, badge_x - scaler.s(90.0), cy + scaler.s(28.0), scaler.font_s(11.0), Palette::NEON_GOLD);

        cy += item_h + item_gap;
    }
}

// =============================================================================
// TAB 3: FULL-SCREEN 10-COLUMN RACE TELEMETRY & LOGS
// =============================================================================
fn render_telemetry_tab(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    history: &[RaceHistoryEntry],
    filter_category_idx: usize,
) {
    let pad = scaler.s(14.0);
    let inner_w = w - pad * 2.0;
    let mut cy = y + pad;

    // --- Category Filter Pills Row ---
    let active_filter = TELEMETRY_CATEGORY_FILTERS.get(filter_category_idx).copied().unwrap_or(TELEMETRY_CATEGORY_FILTERS[0]);
    let pill_h = scaler.s(24.0);
    let pill_gap = scaler.s(6.0);
    let mut px = x + pad;

    fonts.draw_ui_bold("FILTER:", px, cy + scaler.s(16.0), scaler.font_s(11.0), Palette::NEON_GOLD);
    px += scaler.s(55.0);

    for (f_idx, (f_name, _)) in TELEMETRY_CATEGORY_FILTERS.iter().enumerate() {
        let is_sel = f_idx == filter_category_idx;
        let pill_w = scaler.s(if *f_name == "ALL" { 48.0 } else { 75.0 });

        let p_bg = if is_sel {
            Palette::UI_CARD_BG_HOVER
        } else {
            Color::new(0.08, 0.10, 0.14, 0.70)
        };
        let p_border = if is_sel {
            Palette::NEON_CYAN
        } else {
            Palette::UI_CARD_BORDER
        };

        draw_rectangle(px, cy, pill_w, pill_h, p_bg);
        draw_rectangle_lines(px, cy, pill_w, pill_h, if is_sel { 1.8 } else { 1.0 }, p_border);

        fonts.draw_ui_bold_centered(
            f_name,
            px + pill_w * 0.5,
            cy + scaler.s(16.5),
            scaler.font_s(10.5),
            if is_sel { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
        );

        px += pill_w + pill_gap;
    }

    let filter_hint = "[◄ / ►] Change Category Filter";
    fonts.draw_ui_regular(
        filter_hint,
        x + pad + inner_w - scaler.s(180.0),
        cy + scaler.s(16.5),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    cy += pill_h + scaler.s(10.0);

    // Filter history records
    let filtered_history: Vec<&RaceHistoryEntry> = history
        .iter()
        .filter(|entry| match active_filter.1 {
            Some(cat) => entry.category.eq_ignore_ascii_case(cat),
            None => true,
        })
        .collect();

    // --- 10-Column Table Header ---
    let hdr_h = scaler.s(22.0);
    draw_rectangle(x + pad, cy, inner_w, hdr_h, Color::new(0.10, 0.14, 0.22, 0.95));
    draw_rectangle_lines(x + pad, cy, inner_w, hdr_h, 1.0, Palette::UI_CARD_BORDER);

    // 10 Columns: POS, CAT, TRACK, CAR, TIME, BEST LAP, STUNTS, COLLISIONS, CLEAN?, DATE
    let c_pos = x + pad + scaler.s(8.0);
    let c_cat = x + pad + scaler.s(45.0);
    let c_track = x + pad + scaler.s(115.0);
    let c_car = x + pad + scaler.s(280.0);
    let c_time = x + pad + inner_w - scaler.s(430.0);
    let c_best = x + pad + inner_w - scaler.s(320.0);
    let c_stunt = x + pad + inner_w - scaler.s(225.0);
    let c_col = x + pad + inner_w - scaler.s(155.0);
    let c_clean = x + pad + inner_w - scaler.s(85.0);
    let c_date = x + pad + inner_w - scaler.s(35.0);

    let hdr_y = cy + scaler.s(15.0);
    let hdr_font_s = scaler.font_s(10.5);
    fonts.draw_ui_bold("POS", c_pos, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("CAT", c_cat, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("CIRCUIT / TRACK", c_track, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("VEHICLE", c_car, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("TOTAL TIME", c_time, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("BEST LAP", c_best, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("STUNTS", c_stunt, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("IMPACTS", c_col, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("CLEAN?", c_clean, hdr_y, hdr_font_s, Palette::WHITE);
    fonts.draw_ui_bold("DATE", c_date, hdr_y, hdr_font_s, Palette::WHITE);

    cy += hdr_h + scaler.s(2.0);

    // --- Data Rows ---
    let row_h = scaler.s(21.0);
    if filtered_history.is_empty() {
        fonts.draw_ui_regular_centered(
            "No telemetry records matching this filter. Complete races to log multi-level career telemetry!",
            x + pad + inner_w * 0.5,
            cy + scaler.s(35.0),
            scaler.font_s(12.0),
            Palette::UI_TEXT_MUTED,
        );
    } else {
        for (idx, entry) in filtered_history.iter().take(11).enumerate() {
            let row_bg = if idx % 2 == 0 {
                Color::new(0.08, 0.10, 0.15, 0.60)
            } else {
                Color::new(0.06, 0.08, 0.12, 0.60)
            };

            draw_rectangle(x + pad, cy, inner_w, row_h, row_bg);

            let row_y = cy + scaler.s(14.5);
            let val_font_s = scaler.font_s(10.5);

            // 1. POS
            let (pos_str, pos_col) = match entry.position {
                1 => ("P1", Palette::NEON_GOLD),
                2 => ("P2", Color::new(0.85, 0.88, 0.95, 1.0)),
                3 => ("P3", Color::new(0.88, 0.55, 0.25, 1.0)),
                _ => ("P--", Palette::UI_TEXT_MUTED),
            };
            let dyn_pos = if entry.position > 3 { format!("P{}", entry.position) } else { pos_str.to_string() };
            fonts.draw_ui_bold(&dyn_pos, c_pos, row_y, val_font_s, pos_col);

            // 2. CAT
            let cat_label = entry.category.to_uppercase();
            fonts.draw_ui_bold(&cat_label, c_cat, row_y, scaler.font_s(9.5), Palette::NEON_CYAN);

            // 3. TRACK
            let track_clean = entry.track_id.replace('_', " ").to_uppercase();
            fonts.draw_ui_regular(&track_clean, c_track, row_y, val_font_s, Palette::WHITE);

            // 4. VEHICLE
            fonts.draw_ui_regular(&entry.car_name, c_car, row_y, val_font_s, Palette::UI_TEXT_MUTED);

            // 5. TOTAL TIME
            let total_str = format_lap_time(entry.total_time);
            fonts.draw_ui_bold(&total_str, c_time, row_y, val_font_s, Palette::NEON_GREEN);

            // 6. BEST LAP
            let best_str = format_lap_time(entry.best_lap.unwrap_or(0.0));
            fonts.draw_ui_bold(&best_str, c_best, row_y, val_font_s, Palette::NEON_CYAN);

            // 7. STUNTS
            let stunt_str = format!("{} pts", entry.stunt_score);
            fonts.draw_ui_regular(&stunt_str, c_stunt, row_y, val_font_s, Palette::NEON_GOLD);

            // 8. COLLISIONS
            let col_str = format!("{}", entry.collisions);
            fonts.draw_ui_bold(&col_str, c_col, row_y, val_font_s, if entry.collisions == 0 { Palette::NEON_GREEN } else { Color::new(0.95, 0.45, 0.35, 1.0) });

            // 9. CLEAN?
            let clean_str = if entry.collisions == 0 { "YES" } else { "NO" };
            fonts.draw_ui_bold(clean_str, c_clean, row_y, val_font_s, if entry.collisions == 0 { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED });

            // 10. DATE
            let date_str = entry.created_at.split(' ').next().unwrap_or(&entry.created_at);
            fonts.draw_ui_regular(date_str, c_date - scaler.s(25.0), row_y, scaler.font_s(9.5), Palette::UI_TEXT_MUTED);

            cy += row_h + scaler.s(1.5);
        }
    }
}

fn render_kpi_tile(
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
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(x, y, w, h, 1.0, Palette::UI_CARD_BORDER);

    fonts.draw_ui_bold(label, x + scaler.s(8.0), y + scaler.s(16.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
    fonts.draw_ui_bold(val, x + scaler.s(8.0), y + scaler.s(39.0), scaler.font_s(14.5), val_col);
}

fn render_data_row(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    val: &str,
    val_col: Color,
) {
    fonts.draw_ui_regular(label, x, y, scaler.font_s(11.5), Palette::WHITE);
    let val_dim = fonts.measure_ui_bold(val, scaler.font_s(11.5));
    fonts.draw_ui_bold(val, x + w - val_dim.width, y, scaler.font_s(11.5), val_col);
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
