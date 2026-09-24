use macroquad::color::Color;
use macroquad::math::Vec2;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::texture::{draw_texture_ex, DrawTextureParams};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::profile::{
    draw_country_banner, AssistProfile, ChampionshipAward, CountryRegistry, ModuleCareerProgress,
    PlayerProfile, ProfileCareerStats, RaceHistoryEntry, TrophyMetal,
};
use crate::render::color::{CarColorScheme, Palette};
use crate::render::lateral::render_real_car_lateral_by_id;
use crate::render::trophy_textures::draw_trophy_badge;
use crate::render::vehicle_assets::get_vehicle_lateral_texture;
use crate::series::{ChampionshipManager, ChampionshipSession, SeriesDefinition};

/// Official motorsport disciplines displayed in the 5x5 Player Profile Trophy Cabinet.
pub const CABINET_DISCIPLINES: &[(&str, &str)] = &[
    ("gt", "GT Challenge"),
    ("kart", "Karting Cup"),
    ("rally", "Rallycross"),
    ("nascar", "NASCAR Series"),
    ("extreme_offroad", "Extreme Off-Road"),
];

/// Helper to lookup an earned award for a specific discipline and tier from an awards slice.
pub fn find_award_for_slot<'a>(
    awards: &'a [ChampionshipAward],
    discipline: &str,
    tier: u32,
) -> Option<&'a ChampionshipAward> {
    let norm = crate::render::trophy_textures::normalize_discipline(discipline);
    awards.iter().find(|a| {
        crate::render::trophy_textures::normalize_discipline(&a.module_id) == norm && a.tier == tier
    })
}

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

/// Focus area within the Player Profile Manager screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProfileFocusArea {
    /// Top hero driver card focused (cycling drivers, enter opens roster manager)
    HeroCard,
    /// Tab bar focused (left/right switches tabs)
    #[default]
    Tabs,
    /// Category/module filters row focused (left/right selects active category filter)
    Filters,
    /// Content body focused (scrolling list of championships or telemetry)
    Content,
}

/// Renders the full-screen Option A: Tabbed Motorsport Telemetry Dashboard.
pub fn render_profile_manager_screen(
    fonts: &Fonts,
    profiles: &[PlayerProfile],
    selected_idx: usize,
    history: &[RaceHistoryEntry],
    stats: &ProfileCareerStats,
    active_tab: usize,
    filter_category_idx: usize,
    focus_area: ProfileFocusArea,
    championship_manager: &ChampionshipManager,
    active_championship: Option<&ChampionshipSession>,
    champ_scroll_offset: usize,
    champ_selected_idx: usize,
    is_dev_mode: bool,
    career_level: u32,
    module_progress_map: &std::collections::HashMap<String, ModuleCareerProgress>,
    awards: &[ChampionshipAward],
    cabinet_disc_idx: usize,
    cabinet_tier_idx: usize,
) {
    let focus_card = focus_area == ProfileFocusArea::HeroCard;
    let is_filter_focused = focus_area == ProfileFocusArea::Filters;
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
    let card_bg = if focus_card {
        Palette::UI_CARD_BG_HOVER
    } else {
        Color::new(0.06, 0.08, 0.13, 0.95)
    };
    let card_border = if focus_card {
        Palette::NEON_CYAN
    } else {
        Palette::UI_CARD_BORDER
    };
    let card_thickness = if focus_card { 2.4 } else { 1.2 };
    scaler.draw_glass_card(x, cur_y, full_w, hero_h, card_bg, card_border, card_thickness);

    if focus_card {
        draw_rectangle(x, cur_y, scaler.s(6.0), hero_h, Palette::NEON_CYAN);
    }

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

        // Manage Roster Action Button
        let title_dim = fonts.measure_display(&driver_title, scaler.font_s(21.0));
        let btn2_x = name_x + title_dim.width + scaler.s(12.0);
        let btn2_w = scaler.s(136.0);
        let (b2_bg, b2_border, b2_text) = if focus_card {
            (Color::new(0.08, 0.25, 0.35, 0.95), Palette::WHITE, "[ENTER] MANAGE ▶")
        } else {
            (Color::new(0.12, 0.16, 0.24, 0.90), Palette::NEON_CYAN, "[E] MANAGE ▶")
        };
        draw_rectangle(btn2_x, btn_y, btn2_w, btn_h, b2_bg);
        draw_rectangle_lines(btn2_x, btn_y, btn2_w, btn_h, if focus_card { 2.0 } else { 1.2 }, b2_border);
        fonts.draw_ui_bold_centered(b2_text, btn2_x + btn2_w * 0.5, btn_y + scaler.s(21.0), scaler.font_s(11.5), if focus_card { Palette::WHITE } else { Palette::NEON_CYAN });

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
    // 2. WIDESCREEN TAB BAR [1] OVERVIEW | [2] CAREERS | [3] CABINET | [4] CHAMPS | [5] LOGS
    // =========================================================================
    let tab_bar_h = scaler.s(34.0);
    let tab_names = [
        "[1] GLOBAL OVERVIEW",
        "[2] CAREER DISCIPLINES",
        "[3] TROPHY CABINET",
        "[4] CHAMPIONSHIPS",
        "[5] RACE TELEMETRY & LOGS",
    ];
    let tab_gap = scaler.s(8.0);
    let total_gaps = tab_gap * (tab_names.len() as f32 - 1.0);
    let tab_w = ((full_w - scaler.s(220.0) - total_gaps) / tab_names.len() as f32).max(scaler.s(96.0));

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

        fonts.draw_ui_bold_centered(name, t_x + tab_w * 0.5, cur_y + scaler.s(22.0), scaler.font_s(11.0), text_col);
    }

    // Tab switch prompt on the far right
    let tab_hint = if focus_area == ProfileFocusArea::Tabs {
        if active_tab == 3 || active_tab == 4 {
            "[◄ / ►] TABS  •  [▼] FILTERS"
        } else if active_tab == 2 {
            "[◄ / ►] TABS  •  [▼] CABINET"
        } else {
            "[◄ / ►] [1-5] TABS"
        }
    } else {
        "[◄ / ►] [1-5] TABS"
    };
    fonts.draw_ui_bold(
        tab_hint,
        x + full_w - scaler.s(205.0),
        cur_y + scaler.s(22.0),
        scaler.font_s(11.0),
        if focus_area == ProfileFocusArea::Tabs { Palette::NEON_CYAN } else { Palette::NEON_GOLD },
    );

    cur_y += tab_bar_h + scaler.s(8.0);

    // =========================================================================
    // 3. MAIN CONTENT CONTAINER
    // =========================================================================
    let footer_h = scaler.s(36.0);
    let content_h = (sh - cur_y - footer_h - scaler.s(10.0)).max(scaler.s(360.0));
    scaler.draw_glass_card(x, cur_y, full_w, content_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    match active_tab {
        0 => render_overview_tab(&scaler, fonts, x, cur_y, full_w, content_h, stats, awards),
        1 => render_disciplines_tab(&scaler, fonts, x, cur_y, full_w, content_h, stats, module_progress_map),
        2 => render_trophy_cabinet_tab(
            &scaler,
            fonts,
            x,
            cur_y,
            full_w,
            content_h,
            awards,
            cabinet_disc_idx,
            cabinet_tier_idx,
            focus_area == ProfileFocusArea::Content,
        ),
        3 => render_championships_tab(
            &scaler,
            fonts,
            x,
            cur_y,
            full_w,
            content_h,
            stats,
            history,
            sel_profile,
            championship_manager,
            active_championship,
            filter_category_idx,
            champ_scroll_offset,
            champ_selected_idx,
            focus_area == ProfileFocusArea::Content,
            is_filter_focused,
            is_dev_mode,
            career_level,
            module_progress_map,
            awards,
        ),
        4 => render_telemetry_tab(&scaler, fonts, x, cur_y, full_w, content_h, history, filter_category_idx, is_filter_focused),
        _ => render_overview_tab(&scaler, fonts, x, cur_y, full_w, content_h, stats, awards),
    }

    // =========================================================================
    // 4. FOOTER ACTION BAR
    // =========================================================================
    let foot_y = sh - scaler.s(20.0);
    let footer_prompt = match focus_area {
        ProfileFocusArea::HeroCard => {
            "[ENTER / E] Open Driver Manager  |  [▼] Focus Tabs  |  [◄ / ►] [Q] Cycle Driver  |  [ESC] Exit"
        }
        ProfileFocusArea::Tabs => {
            if active_tab == 3 || active_tab == 4 {
                "[▲] Driver Card  |  [◄ / ►] [1-5] Tabs  |  [▼] Module Filters  |  [ESC] Exit"
            } else if active_tab == 2 {
                "[▲] Driver Card  |  [◄ / ►] [1-5] Tabs  |  [▼] Cabinet Grid  |  [ESC] Exit"
            } else {
                "[▲] Driver Card  |  [◄ / ►] [1-5] Tabs  |  [E] Manage Roster  |  [ESC] Exit"
            }
        }
        ProfileFocusArea::Filters => {
            if active_tab == 3 {
                "[▲] Focus Tabs  |  [◄ / ►] Select Module Filter  |  [▼] Browse Championships  |  [ESC] Exit"
            } else {
                "[▲] Focus Tabs  |  [◄ / ►] Select Module Filter  |  [▼] Browse Telemetry Logs  |  [ESC] Exit"
            }
        }
        ProfileFocusArea::Content => {
            if active_tab == 2 {
                "[▲ / ▼] Discipline  |  [◄ / ►] Tier  |  [▲ at top] Tabs  |  [ESC] Exit"
            } else if active_tab == 3 {
                "[ENTER] Enter Cup  |  [R] Reset Cup  |  [▲ / ▼] Navigate  |  [▲ at top] Filters  |  [ESC] Exit"
            } else if active_tab == 0 {
                "[ENTER / 3] Open Trophy Cabinet  |  [▲] Tabs  |  [ESC] Exit"
            } else {
                "[▲] Focus Filters  |  [▲ / ▼] Browse Telemetry  |  [F] Quick Filter  |  [ESC] Exit"
            }
        }
    };
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
    awards: &[ChampionshipAward],
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
    i_y += row_h + scaler.s(8.0);

    // Top Honors Showcase Shelf inside Right Box
    let shelf_x = right_x + scaler.s(14.0);
    let shelf_w = col_w - scaler.s(28.0);
    let shelf_h = (col_h - (i_y - cy) - scaler.s(12.0)).max(scaler.s(68.0));

    scaler.draw_glass_card(shelf_x, i_y, shelf_w, shelf_h, Color::new(0.08, 0.11, 0.17, 0.95), Palette::NEON_CYAN, 1.0);

    // Header inside shelf
    fonts.draw_ui_bold("🏆 TOP HONORS SHOWCASE", shelf_x + scaler.s(10.0), i_y + scaler.s(15.0), scaler.font_s(11.0), Palette::NEON_GOLD);
    let cab_prompt = "[3] or [ENTER] CABINET ▶";
    let cab_dim = fonts.measure_ui_bold(cab_prompt, scaler.font_s(10.0));
    fonts.draw_ui_bold(cab_prompt, shelf_x + shelf_w - cab_dim.width - scaler.s(10.0), i_y + scaler.s(15.0), scaler.font_s(10.0), Palette::NEON_CYAN);

    // Find top 3 awards:
    // Sort awards by: Gold first (position == 1), then tier descending (5..1), then Silver, then Bronze.
    let mut top_awards: Vec<&ChampionshipAward> = awards.iter().collect();
    top_awards.sort_by(|a, b| {
        a.position.cmp(&b.position)
            .then_with(|| b.tier.cmp(&a.tier))
            .then_with(|| b.points.cmp(&a.points))
    });
    top_awards.truncate(3);

    // Stepped pedestal arrangement: [Rank 2 (Left), Rank 1 (Center), Rank 3 (Right)]
    let slot_indices: [Option<usize>; 3] = match top_awards.len() {
        0 => [None, None, None],
        1 => [None, Some(0), None],
        2 => [Some(1), Some(0), None],
        _ => [Some(1), Some(0), Some(2)],
    };

    let ped_col_w = shelf_w / 3.0;
    let ped_base_y = i_y + shelf_h - scaler.s(6.0);

    for (idx, opt_award_idx) in slot_indices.iter().enumerate() {
        let ped_cx = shelf_x + ped_col_w * (idx as f32 + 0.5);
        let is_center = idx == 1;
        let ped_h = if is_center { scaler.s(14.0) } else if idx == 0 { scaler.s(10.0) } else { scaler.s(7.0) };
        let ped_w = ped_col_w - scaler.s(16.0);
        let ped_x = ped_cx - ped_w * 0.5;
        let ped_y = ped_base_y - ped_h;

        let ped_col = if is_center { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER };
        draw_rectangle(ped_x, ped_y, ped_w, ped_h, Color::new(0.12, 0.16, 0.22, 0.90));
        draw_rectangle_lines(ped_x, ped_y, ped_w, ped_h, if is_center { 1.5 } else { 1.0 }, ped_col);

        if let Some(award_idx) = opt_award_idx {
            let award = top_awards[*award_idx];
            let badge_sz = if is_center { scaler.s(36.0) } else { scaler.s(30.0) };
            let badge_x = ped_cx - badge_sz * 0.5;
            let badge_y = ped_y - badge_sz - scaler.s(2.0);

            draw_trophy_badge(
                badge_x,
                badge_y,
                badge_sz,
                badge_sz,
                award.discipline(),
                award.tier,
                Some(award.metallic_tier()),
                false,
            );

            let rank_str = format!("T{} {}", award.tier, award.discipline().to_uppercase());
            fonts.draw_ui_bold_centered(
                &rank_str,
                ped_cx,
                ped_y + ped_h * 0.5 + scaler.s(3.5),
                scaler.font_s(8.0),
                if is_center { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
            );
        } else {
            let empty_lbl = if is_center { "PINNACLE" } else { "EMPTY" };
            fonts.draw_ui_regular_centered(
                empty_lbl,
                ped_cx,
                ped_y + ped_h * 0.5 + scaler.s(3.0),
                scaler.font_s(7.5),
                Color::new(0.40, 0.45, 0.55, 0.7),
            );
        }
    }
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
    module_progress_map: &std::collections::HashMap<String, ModuleCareerProgress>,
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
        let mod_prog = module_progress_map.get(*cat_key);
        let tier = mod_prog.map(|p| p.level.clamp(1, 5)).unwrap_or(1);
        let tier_str = format!("TIER {}", tier);
        fonts.draw_ui_bold(&tier_str, cx + card_w - scaler.s(60.0), cy + scaler.s(22.0), scaler.font_s(11.5), *accent_col);

        // Mini XP Bar towards next tier car
        let xp_progress = mod_prog.map(|p| p.level_progress_ratio()).unwrap_or(0.0);
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
        let catalog_count = crate::catalog::get_models_for_module(cat_key).len();
        let total_cars = if catalog_count > 0 { catalog_count } else { *max_cars };
        let unlocked = mod_prog.map(|p| {
            let count = p.unlocked_cars.iter().filter(|c| {
                crate::catalog::find_model_by_id(c).is_some_and(|m| m.module_id == *cat_key)
            }).count();
            if count == 0 { 1 } else { count }
        }).unwrap_or(1).min(total_cars);
        let garage_str = format!("Garage: {}/{} Cars Unlocked", unlocked, total_cars);
        fonts.draw_ui_regular(&garage_str, bar_x, sy, scaler.font_s(10.5), *accent_col);
    }
}

// =============================================================================
// TAB 2: CHAMPIONSHIPS REGISTRY & STANDINGS
// =============================================================================

/// Helper to resolve a car name string (e.g. from history entry or catalog) to a known RealCarModel.
pub fn find_car_model_by_identifier(ident: &str) -> Option<&'static crate::catalog::RealCarModel> {
    let trimmed = ident.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. Direct ID match
    if let Some(m) = crate::catalog::find_model_by_id(trimmed) {
        return Some(m);
    }

    // 2. Direct Name match (case-insensitive) in all real cars
    for m in crate::catalog::get_all_models() {
        if m.name.eq_ignore_ascii_case(trimmed) || m.id.eq_ignore_ascii_case(trimmed) {
            return Some(m);
        }
    }

    // 3. Direct match in classic arcade cars
    for m in crate::catalog::CLASSIC_ARCADE_CARS.iter() {
        if m.name.eq_ignore_ascii_case(trimmed) || m.id.eq_ignore_ascii_case(trimmed) {
            return Some(m);
        }
    }

    // 4. Match against CarChoice titles / common naming patterns
    let lower = trimmed.to_lowercase();
    if lower.contains("gt4") {
        crate::catalog::find_model_by_id("gt_toyota_supra_gt4")
            .or_else(|| crate::catalog::find_model_by_id("gt_porsche_718_gt4"))
    } else if lower.contains("gt3") {
        crate::catalog::find_model_by_id("gt_porsche_911_gt3r")
    } else if lower.contains("gt2") {
        crate::catalog::find_model_by_id("gt_porsche_911_gt2_rs")
    } else if lower.contains("gt1") {
        crate::catalog::find_model_by_id("gt_porsche_911_gt1_98")
    } else if lower.contains("hypercar") || lower.contains("lmh") || lower.contains("prototype") {
        crate::catalog::find_model_by_id("gt_ferrari_499p")
    } else if lower.contains("nascar") || lower.contains("stock car") {
        crate::catalog::find_model_by_id("nascar_monte_carlo_ss")
    } else if lower.contains("rally") {
        crate::catalog::find_model_by_id("rally_polo_rx")
            .or_else(|| crate::catalog::find_model_by_id("rally_peugeot_208_rally4"))
    } else if lower.contains("kart") {
        crate::catalog::find_model_by_id("kart_birel_art_kz2")
            .or_else(|| crate::catalog::find_model_by_id("kart_crg_hero_60"))
    } else if lower.contains("offroad") || lower.contains("sand rail") || lower.contains("buggy") {
        crate::catalog::find_model_by_id("offroad_sand_rail_buggy")
            .or_else(|| crate::catalog::find_model_by_id("offroad_baja_trophy_truck"))
    } else if lower.contains("drift") || lower.contains("sports coupe") {
        crate::catalog::find_model_by_id("classic_gt")
    } else {
        None
    }
}

/// Resolves the car model ID and display name to show for a championship row:
/// 1. Latest car used by the player in this championship (from history)
/// 2. Or the player's configured car model from the championship definition
/// 3. Or the entry car of the category for that modality
pub fn resolve_championship_car_model_id(
    champ: &SeriesDefinition,
    history: &[RaceHistoryEntry],
) -> (&'static str, String, bool) {
    // 1. Look for the latest race in history for this championship
    let latest_race = history.iter().find(|e| {
        e.championship_name.as_deref().is_some_and(|name| {
            name.eq_ignore_ascii_case(&champ.series.name)
                || name.eq_ignore_ascii_case(&champ.series.id)
                || champ.series.name.to_lowercase().contains(&name.to_lowercase())
                || name.to_lowercase().contains(&champ.series.name.to_lowercase())
        })
    });

    if let Some(entry) = latest_race {
        if let Some(model) = find_car_model_by_identifier(&entry.car_name) {
            return (model.id, model.name.to_string(), true);
        }
    }

    // 2. Try the player's configured car in the championship definition
    if let Some(player_driver) = champ.drivers.iter().find(|d| d.is_player) {
        if let Some(model_id) = &player_driver.car_model_id {
            if let Some(model) = find_car_model_by_identifier(model_id) {
                return (model.id, model.name.to_string(), false);
            }
        }
    }

    // 3. Fallback: Entry car of the category for that modality
    // First try the specific tier entry car of the championship
    if let Some(model) = crate::catalog::get_models_for_module_and_tier(&champ.series.module_id, champ.series.tier as u8).first() {
        return (model.id, model.name.to_string(), false);
    }

    // Then try Tier 1 entry car of that module
    if let Some(model) = crate::catalog::get_models_for_module_and_tier(&champ.series.module_id, 1).first() {
        return (model.id, model.name.to_string(), false);
    }

    // Then any car of that module
    if let Some(model) = crate::catalog::get_models_for_module(&champ.series.module_id).first() {
        return (model.id, model.name.to_string(), false);
    }

    // Final fallback
    ("gt_porsche_718_gt4", "Porsche 718 Cayman GT4 RS".to_string(), false)
}

/// Returns championships filtered by optional module_id and sorted strictly by tier ascending (1..=5),
/// then module_id, then series name.
pub fn get_sorted_championships<'a>(
    championship_manager: &'a ChampionshipManager,
    module_filter: Option<&str>,
) -> Vec<&'a SeriesDefinition> {
    let mut list: Vec<&SeriesDefinition> = championship_manager
        .all_sorted()
        .into_iter()
        .filter(|c| match module_filter {
            Some(mod_id) => c.series.module_id.eq_ignore_ascii_case(mod_id),
            None => true,
        })
        .collect();

    list.sort_by(|a, b| {
        a.series
            .tier
            .cmp(&b.series.tier)
            .then_with(|| a.series.module_id.cmp(&b.series.module_id))
            .then_with(|| a.series.name.cmp(&b.series.name))
    });

    list
}

/// Computes the number of championship cards visible within the given content height.
pub fn championship_visible_count(scaler: &UiScaler, content_h: f32) -> usize {
    let item_h = scaler.s(64.0);
    let item_gap = scaler.s(7.0);
    let header_space = scaler.s(82.0);
    let available_h = (content_h - header_space - scaler.s(12.0)).max(scaler.s(100.0));
    ((available_h + item_gap) / (item_h + item_gap)).floor().max(1.0) as usize
}

// =============================================================================
// TAB 2: TROPHY CABINET (5x5 DISCIPLINE-BY-TIER GRID + INSPECTION PANEL)
// =============================================================================
fn render_trophy_cabinet_tab(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    awards: &[ChampionshipAward],
    focused_disc_idx: usize,
    focused_tier_idx: usize,
    is_content_focused: bool,
) {
    let pad = scaler.s(16.0);
    let inner_w = w - pad * 2.0;
    let mut cy = y + pad;

    // 1. TOP HEADER SUMMARY BANNER
    let banner_h = scaler.s(34.0);
    scaler.draw_glass_card(x + pad, cy, inner_w, banner_h, Color::new(0.08, 0.12, 0.18, 0.90), Palette::NEON_GOLD, 1.2);

    let mut earned_slots = 0;
    for (disc, _) in CABINET_DISCIPLINES {
        for t in 1..=5 {
            if find_award_for_slot(awards, disc, t).is_some() {
                earned_slots += 1;
            }
        }
    }
    let pct = (earned_slots as f32 / 25.0 * 100.0).round() as u32;
    let gold_count = awards.iter().filter(|a| a.metallic_tier() == TrophyMetal::Gold).count();
    let silver_count = awards.iter().filter(|a| a.metallic_tier() == TrophyMetal::Silver).count();
    let bronze_count = awards.iter().filter(|a| a.metallic_tier() == TrophyMetal::Bronze).count();

    let banner_text = format!(
        "CABINET COLLECTION: {}/25 EARNED ({}%)  •  🏆 {} GOLD  •  🥈 {} SILVER  •  🥉 {} BRONZE",
        earned_slots, pct, gold_count, silver_count, bronze_count
    );
    fonts.draw_ui_bold_centered(
        &banner_text,
        x + pad + inner_w * 0.5,
        cy + scaler.s(22.0),
        scaler.font_s(13.0),
        Palette::WHITE,
    );

    cy += banner_h + scaler.s(10.0);

    // 2. MAIN LAYOUT: LEFT GRID (58%) vs RIGHT INSPECTION PANEL (42%)
    let col_gap = scaler.s(14.0);
    let grid_w = (inner_w - col_gap) * 0.58;
    let panel_w = inner_w - grid_w - col_gap;
    let body_h = (h - (cy - y) - scaler.s(14.0)).max(scaler.s(280.0));

    // Left Grid Glass Card
    let left_x = x + pad;
    scaler.draw_glass_card(left_x, cy, grid_w, body_h, Color::new(0.05, 0.07, 0.11, 0.90), Palette::UI_CARD_BORDER, 1.0);

    // Grid Header row
    let disc_label_w = scaler.s(108.0);
    let tier_col_w = (grid_w - disc_label_w - scaler.s(16.0)) / 5.0;
    let grid_top_pad = scaler.s(12.0);
    let header_y = cy + grid_top_pad;

    fonts.draw_ui_bold("DISCIPLINE", left_x + scaler.s(12.0), header_y + scaler.s(14.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

    let star_headers = ["T1 (★)", "T2 (★★)", "T3 (★★★)", "T4 (★★★★)", "T5 (★★★★★)"];
    for (t_idx, header) in star_headers.iter().enumerate() {
        let th_x = left_x + disc_label_w + t_idx as f32 * tier_col_w;
        fonts.draw_ui_bold_centered(header, th_x + tier_col_w * 0.5, header_y + scaler.s(14.0), scaler.font_s(10.0), Palette::NEON_CYAN);
    }

    // Grid Rows (5 disciplines)
    let grid_rows_y = header_y + scaler.s(20.0);
    let row_h = (body_h - grid_top_pad - scaler.s(20.0) - scaler.s(22.0)) / 5.0;

    for (row_idx, (disc_id, disc_title)) in CABINET_DISCIPLINES.iter().enumerate() {
        let r_y = grid_rows_y + row_idx as f32 * row_h;

        // Discipline Label on left
        fonts.draw_ui_bold(
            disc_title,
            left_x + scaler.s(12.0),
            r_y + row_h * 0.5 + scaler.s(4.0),
            scaler.font_s(11.0),
            Color::new(0.88, 0.92, 0.98, 1.0),
        );

        // 5 Tier Cells
        for col_idx in 0..5 {
            let tier = (col_idx + 1) as u32;
            let c_x = left_x + disc_label_w + col_idx as f32 * tier_col_w;
            let cell_pad = scaler.s(3.0);
            let cell_w = tier_col_w - cell_pad * 2.0;
            let cell_h = row_h - cell_pad * 2.0;
            let cell_x = c_x + cell_pad;
            let cell_y = r_y + cell_pad;

            let is_focused = row_idx == focused_disc_idx && col_idx == focused_tier_idx;
            let opt_award = find_award_for_slot(awards, disc_id, tier);

            let (bg_col, border_col, border_thickness) = if is_focused {
                if is_content_focused {
                    (Palette::UI_CARD_BG_HOVER, Palette::NEON_CYAN, 2.4)
                } else {
                    (Color::new(0.10, 0.14, 0.22, 0.85), Palette::NEON_CYAN, 1.5)
                }
            } else {
                (Color::new(0.06, 0.08, 0.12, 0.70), Palette::UI_CARD_BORDER, 1.0)
            };

            draw_rectangle(cell_x, cell_y, cell_w, cell_h, bg_col);
            draw_rectangle_lines(cell_x, cell_y, cell_w, cell_h, border_thickness, border_col);

            let badge_sz = (cell_h - scaler.s(12.0)).min(scaler.s(44.0)).max(scaler.s(22.0));
            let badge_x = cell_x + (cell_w - badge_sz) * 0.5;
            let badge_y = cell_y + scaler.s(2.0);

            if let Some(award) = opt_award {
                draw_trophy_badge(
                    badge_x,
                    badge_y,
                    badge_sz,
                    badge_sz,
                    disc_id,
                    tier,
                    Some(award.metallic_tier()),
                    false,
                );
                // Label under badge
                let (lbl, col) = match award.metallic_tier() {
                    TrophyMetal::Gold => ("GOLD", Palette::NEON_GOLD),
                    TrophyMetal::Silver => ("SILVER", Color::new(0.85, 0.90, 0.98, 1.0)),
                    TrophyMetal::Bronze => ("BRONZE", Color::new(0.88, 0.55, 0.35, 1.0)),
                };
                fonts.draw_ui_bold_centered(
                    lbl,
                    cell_x + cell_w * 0.5,
                    cell_y + cell_h - scaler.s(3.0),
                    scaler.font_s(8.5),
                    col,
                );
            } else {
                draw_trophy_badge(
                    badge_x,
                    badge_y,
                    badge_sz,
                    badge_sz,
                    disc_id,
                    tier,
                    None,
                    false,
                );
                fonts.draw_ui_bold_centered(
                    "LOCKED",
                    cell_x + cell_w * 0.5,
                    cell_y + cell_h - scaler.s(3.0),
                    scaler.font_s(8.0),
                    Palette::UI_TEXT_MUTED,
                );
            }
        }
    }

    // Grid Navigation footer
    let grid_foot_y = cy + body_h - scaler.s(10.0);
    fonts.draw_ui_regular(
        "[▲ / ▼] Discipline  •  [◄ / ►] Tier Slot  •  Legend: [GOLD]=1st [SILV]=2nd [BRNZ]=3rd [LOCK]=Locked",
        left_x + scaler.s(12.0),
        grid_foot_y,
        scaler.font_s(9.5),
        Palette::UI_TEXT_MUTED,
    );

    // 3. RIGHT INSPECTION & PROVENANCE PANEL
    let right_x = left_x + grid_w + col_gap;
    scaler.draw_glass_card(right_x, cy, panel_w, body_h, Color::new(0.06, 0.08, 0.13, 0.95), Palette::NEON_CYAN, 1.2);

    fonts.draw_ui_bold(
        "TROPHY INSPECTION & PROVENANCE",
        right_x + scaler.s(16.0),
        cy + scaler.s(24.0),
        scaler.font_s(13.5),
        Palette::NEON_CYAN,
    );

    let (focused_disc_id, focused_disc_title) = CABINET_DISCIPLINES[focused_disc_idx.min(4)];
    let focused_tier = (focused_tier_idx + 1).clamp(1, 5) as u32;
    let focused_award = find_award_for_slot(awards, focused_disc_id, focused_tier);

    // Illuminated preview box in inspection panel
    let preview_box_w = (panel_w - scaler.s(32.0)).min(scaler.s(210.0));
    let preview_box_h = preview_box_w;
    let preview_box_x = right_x + (panel_w - preview_box_w) * 0.5;
    let preview_box_y = cy + scaler.s(34.0);

    draw_rectangle(preview_box_x, preview_box_y, preview_box_w, preview_box_h, Color::new(0.03, 0.04, 0.07, 0.95));
    draw_rectangle_lines(preview_box_x, preview_box_y, preview_box_w, preview_box_h, 1.0, Palette::UI_CARD_BORDER);

    let sprite_sz = preview_box_w - scaler.s(16.0);
    let sprite_x = preview_box_x + (preview_box_w - sprite_sz) * 0.5;
    let sprite_y = preview_box_y + (preview_box_h - sprite_sz) * 0.5;

    if let Some(award) = focused_award {
        draw_trophy_badge(
            sprite_x,
            sprite_y,
            sprite_sz,
            sprite_sz,
            focused_disc_id,
            focused_tier,
            Some(award.metallic_tier()),
            true,
        );
    } else {
        draw_trophy_badge(
            sprite_x,
            sprite_y,
            sprite_sz,
            sprite_sz,
            focused_disc_id,
            focused_tier,
            None,
            true,
        );
    }

    // Detail rows below preview box
    let mut det_y = preview_box_y + preview_box_h + scaler.s(16.0);
    let det_row_h = scaler.s(21.0);
    let det_w = panel_w - scaler.s(32.0);

    if let Some(award) = focused_award {
        let (honor_str, honor_col) = match award.metallic_tier() {
            TrophyMetal::Gold => ("🏆 1ST PLACE [GOLD CHAMPION]", Palette::NEON_GOLD),
            TrophyMetal::Silver => ("🥈 2ND PLACE [SILVER RUNNER-UP]", Color::new(0.85, 0.90, 0.98, 1.0)),
            TrophyMetal::Bronze => ("🥉 3RD PLACE [BRONZE PODIUM]", Color::new(0.88, 0.55, 0.35, 1.0)),
        };
        let champ_title = format!("{} (Tier {})", award.championship_id.replace('_', " ").to_uppercase(), award.tier);

        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Championship", &champ_title, Palette::WHITE);
        det_y += det_row_h;
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Discipline", focused_disc_title, Palette::NEON_CYAN);
        det_y += det_row_h;
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Podium Honor", honor_str, honor_col);
        det_y += det_row_h;
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Points Scored", &format!("{} PTS", award.points), Palette::WHITE);
        det_y += det_row_h;
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Winning Car", &award.car_model_id, Palette::NEON_GREEN);
        det_y += det_row_h;
        let date_str = award.achieved_at.split('T').next().unwrap_or(&award.achieved_at);
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Achieved At", date_str, Palette::UI_TEXT_MUTED);
    } else {
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Status", "🔒 LOCKED TROPHY", Palette::UI_TEXT_MUTED);
        det_y += det_row_h;
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Discipline", focused_disc_title, Palette::NEON_CYAN);
        det_y += det_row_h;
        let stars_str = "★".repeat(focused_tier as usize);
        render_data_row(scaler, fonts, right_x + scaler.s(16.0), det_y, det_w, "Performance Tier", &format!("Tier {} ({})", focused_tier, stars_str), Palette::WHITE);
        det_y += det_row_h + scaler.s(6.0);

        let hint = format!("Compete in and podium at Tier {} of {} to claim this honor.", focused_tier, focused_disc_title);
        fonts.draw_ui_regular(
            &hint,
            right_x + scaler.s(16.0),
            det_y + scaler.s(12.0),
            scaler.font_s(11.0),
            Palette::NEON_GOLD,
        );
    }
}

fn render_championships_tab(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    _stats: &ProfileCareerStats,
    history: &[RaceHistoryEntry],
    _sel_profile: Option<&PlayerProfile>,
    championship_manager: &ChampionshipManager,
    active_championship: Option<&ChampionshipSession>,
    filter_category_idx: usize,
    champ_scroll_offset: usize,
    champ_selected_idx: usize,
    is_content_focused: bool,
    is_filter_focused: bool,
    is_dev_mode: bool,
    career_level: u32,
    module_progress_map: &std::collections::HashMap<String, ModuleCareerProgress>,
    awards: &[ChampionshipAward],
) {
    let pad = scaler.s(16.0);
    let inner_w = w - pad * 2.0;
    let mut cy = y + pad;

    fonts.draw_ui_bold(
        "CHAMPIONSHIP REGISTRY & MOTORSPORT TROPHIES",
        x + pad,
        cy + scaler.s(16.0),
        scaler.font_s(15.0),
        Palette::NEON_GOLD,
    );
    fonts.draw_ui_regular(
        "Official multi-round tournaments across circuit and stage disciplines",
        x + pad,
        cy + scaler.s(32.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );

    // --- Category Filter Pills Row ---
    let active_filter = TELEMETRY_CATEGORY_FILTERS
        .get(filter_category_idx)
        .copied()
        .unwrap_or(TELEMETRY_CATEGORY_FILTERS[0]);

    let filtered_champs = get_sorted_championships(championship_manager, active_filter.1);

    // Scroll counter on top right
    let item_h = scaler.s(64.0);
    let item_gap = scaler.s(7.0);
    let visible_count = championship_visible_count(scaler, h);
    let max_scroll = filtered_champs.len().saturating_sub(visible_count);
    let scroll = champ_scroll_offset.min(max_scroll);

    if !filtered_champs.is_empty() {
        let count_str = format!(
            "{}-{} OF {} CHAMPIONSHIPS",
            scroll + 1,
            (scroll + visible_count).min(filtered_champs.len()),
            filtered_champs.len()
        );
        fonts.draw_ui_regular(
            &count_str,
            x + pad + inner_w - scaler.s(220.0),
            cy + scaler.s(16.0),
            scaler.font_s(10.5),
            Palette::NEON_CYAN,
        );
    }

    cy += scaler.s(40.0);

    // Draw filter pills
    let pill_h = scaler.s(24.0);
    let pill_gap = scaler.s(6.0);
    let mut px = x + pad;

    let filter_label_col = if is_filter_focused {
        Palette::NEON_CYAN
    } else {
        Palette::NEON_GOLD
    };
    fonts.draw_ui_bold(
        if is_filter_focused { "FILTER [◄/►]:" } else { "FILTER:" },
        px,
        cy + scaler.s(16.0),
        scaler.font_s(11.0),
        filter_label_col,
    );
    px += scaler.s(if is_filter_focused { 84.0 } else { 55.0 });

    for (f_idx, (f_name, _)) in TELEMETRY_CATEGORY_FILTERS.iter().enumerate() {
        let is_sel = f_idx == filter_category_idx;
        let pill_w = scaler.s(if *f_name == "ALL" { 48.0 } else { 75.0 });

        let p_bg = if is_sel {
            if is_filter_focused {
                Color::new(0.10, 0.26, 0.38, 0.95)
            } else {
                Palette::UI_CARD_BG_HOVER
            }
        } else {
            Color::new(0.08, 0.10, 0.14, 0.70)
        };
        let p_border = if is_sel {
            Palette::NEON_CYAN
        } else {
            Palette::UI_CARD_BORDER
        };

        draw_rectangle(px, cy, pill_w, pill_h, p_bg);
        let border_thickness = if is_sel && is_filter_focused {
            2.4
        } else if is_sel {
            1.8
        } else {
            1.0
        };
        draw_rectangle_lines(px, cy, pill_w, pill_h, border_thickness, p_border);

        if is_sel && is_filter_focused {
            // Neon top and bottom accent lines on active focused filter pill
            draw_rectangle(px, cy, pill_w, scaler.s(2.0), Palette::NEON_CYAN);
            draw_rectangle(px, cy + pill_h - scaler.s(2.0), pill_w, scaler.s(2.0), Palette::NEON_CYAN);
        }

        let label_text = if is_sel && is_filter_focused {
            format!("◄ {} ►", f_name)
        } else {
            f_name.to_string()
        };

        fonts.draw_ui_bold_centered(
            &label_text,
            px + pill_w * 0.5,
            cy + scaler.s(16.5),
            scaler.font_s(if is_sel && is_filter_focused { 10.0 } else { 10.5 }),
            if is_sel { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
        );

        px += pill_w + pill_gap;
    }

    let filter_hint = if is_filter_focused {
        "[◄ / ►] Select Filter  •  [▼] Browse  •  [▲] Tabs"
    } else if is_content_focused {
        "[▲ / ▼] Navigate  •  [ENTER] Enter Championship  •  [▲ at top] Filters"
    } else if max_scroll > 0 {
        "[▼] Module Filters  •  [▲ / ▼ / WHEEL] Scroll"
    } else {
        "[▼] Module Filters  •  [F] Quick Filter"
    };
    fonts.draw_ui_regular(
        filter_hint,
        x + pad + inner_w - scaler.s(310.0),
        cy + scaler.s(16.5),
        scaler.font_s(11.0),
        if is_filter_focused || is_content_focused { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
    );

    cy += pill_h + scaler.s(10.0);

    if filtered_champs.is_empty() {
        fonts.draw_ui_regular_centered(
            "No official championships registered in this category. Use the Championship Editor to create custom series!",
            x + pad + inner_w * 0.5,
            cy + scaler.s(45.0),
            scaler.font_s(12.0),
            Palette::UI_TEXT_MUTED,
        );
        return;
    }

    for (rel_i, champ) in filtered_champs.iter().skip(scroll).take(visible_count).enumerate() {
        let abs_champ_idx = scroll + rel_i;
        let is_card_selected = is_content_focused && abs_champ_idx == champ_selected_idx;
        let module_tier = module_progress_map.get(&champ.series.module_id).map(|p| p.level).unwrap_or(career_level);
        let is_unlocked = is_dev_mode || champ.series.tier <= 1 || champ.series.tier <= module_tier;

        // Query history for this specific championship
        let champ_entries: Vec<&RaceHistoryEntry> = history
            .iter()
            .filter(|e| {
                e.championship_name.as_deref().is_some_and(|name| {
                    name.eq_ignore_ascii_case(&champ.series.name)
                        || name.eq_ignore_ascii_case(&champ.series.id)
                        || champ.series.name.to_lowercase().contains(&name.to_lowercase())
                        || name.to_lowercase().contains(&champ.series.name.to_lowercase())
                })
            })
            .collect();

        let champ_award = awards.iter().find(|a| {
            (a.championship_id == champ.series.id || a.championship_id == champ.series.name.to_lowercase().replace(' ', "_"))
            || (crate::render::trophy_textures::normalize_discipline(&a.module_id) == crate::render::trophy_textures::normalize_discipline(&champ.series.module_id) && a.tier == champ.series.tier)
        });

        let races_count = champ_entries.len();
        let wins = champ_entries.iter().filter(|e| e.position == 1).count();
        let podiums = champ_entries.iter().filter(|e| e.position >= 1 && e.position <= 3).count();
        let best_pos = champ_entries.iter().map(|e| e.position).min();
        let best_finish_str = best_pos.map(|p| format!("P{}", p)).unwrap_or_else(|| "P--".to_string());
        let best_lap_val = champ_entries.iter().filter_map(|e| e.best_lap).min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let best_lap_str = best_lap_val.map(format_lap_time).unwrap_or_else(|| "--:--.---".to_string());

        let total_rounds = champ.rounds.len().max(1);
        let is_matching_session = |s: &ChampionshipSession| {
            s.name.eq_ignore_ascii_case(&champ.series.name)
                || s.name.eq_ignore_ascii_case(&champ.series.id)
                || s.name.to_lowercase().contains(&champ.series.name.to_lowercase())
                || champ.series.name.to_lowercase().contains(&s.name.to_lowercase())
                || champ.series.id.to_lowercase().contains(&s.name.to_lowercase())
                || s.name.to_lowercase().contains(&champ.series.id.to_lowercase())
        };

        let module_active = module_progress_map
            .get(&champ.series.module_id)
            .and_then(|p| p.active_championship.as_ref())
            .filter(|s| is_matching_session(s));

        let effective_active = active_championship.filter(|s| is_matching_session(s)).or(module_active);
        let is_active_session = effective_active.is_some();

        let completed_rounds = if let Some(active) = effective_active {
            active.current_round.min(total_rounds)
        } else {
            races_count.min(total_rounds)
        };

        let is_completed = if let Some(active) = effective_active {
            active.is_completed
        } else {
            races_count >= total_rounds
        };

        let progress_ratio = if is_completed {
            1.0
        } else {
            (completed_rounds as f32 / total_rounds as f32).clamp(0.0, 1.0)
        };

        let (progress_label, progress_col) = if is_completed {
            (format!("COMPLETED ({}/{} ROUNDS)", total_rounds, total_rounds), Palette::NEON_GREEN)
        } else if is_active_session {
            (format!("LIVE: ROUND {}/{} ({:.0}%)", completed_rounds + 1, total_rounds, progress_ratio * 100.0), Palette::NEON_GOLD)
        } else if completed_rounds > 0 {
            (format!("PROGRESS: {}/{} ({:.0}%)", completed_rounds, total_rounds, progress_ratio * 100.0), Palette::NEON_CYAN)
        } else {
            (format!("0/{} ROUNDS (AVAILABLE)", total_rounds), Palette::UI_TEXT_MUTED)
        };

        let (status_text, status_col, status_bg, card_border) = if !is_unlocked {
            if is_card_selected {
                (
                    format!("🔒 LOCKED [TIER {} REQUIRED]", champ.series.tier),
                    Color::new(0.92, 0.45, 0.45, 1.0),
                    Color::new(0.22, 0.08, 0.08, 0.95),
                    Color::new(0.70, 0.25, 0.25, 0.95),
                )
            } else {
                (
                    format!("🔒 TIER {} REQUIRED", champ.series.tier),
                    Palette::UI_TEXT_MUTED,
                    Color::new(0.08, 0.09, 0.12, 0.70),
                    Palette::UI_CARD_BORDER,
                )
            }
        } else if is_card_selected {
            if is_active_session {
                (
                    format!("[ENTER] RESUME R{}  [R] RESET", completed_rounds + 1),
                    Palette::WHITE,
                    Color::new(0.25, 0.20, 0.05, 0.98),
                    Palette::NEON_GOLD,
                )
            } else if is_completed {
                (
                    "[ENTER] RESTART CUP ▶".to_string(),
                    Palette::WHITE,
                    Color::new(0.05, 0.22, 0.12, 0.98),
                    Palette::NEON_GREEN,
                )
            } else if completed_rounds > 0 {
                (
                    format!("[ENTER] ENTER R{}  [R] RESET", completed_rounds + 1),
                    Palette::WHITE,
                    Color::new(0.06, 0.24, 0.35, 0.98),
                    Palette::NEON_CYAN,
                )
            } else {
                (
                    "[ENTER] START CUP ▶".to_string(),
                    Palette::WHITE,
                    Color::new(0.06, 0.24, 0.35, 0.98),
                    Palette::NEON_CYAN,
                )
            }
        } else if let Some(award) = champ_award {
            let (col, bg) = match award.metallic_tier() {
                TrophyMetal::Gold => (Palette::NEON_GOLD, Color::new(0.25, 0.20, 0.05, 0.90)),
                TrophyMetal::Silver => (Color::new(0.85, 0.88, 0.95, 1.0), Color::new(0.12, 0.16, 0.24, 0.90)),
                TrophyMetal::Bronze => (Color::new(0.88, 0.55, 0.25, 1.0), Color::new(0.18, 0.10, 0.06, 0.90)),
            };
            (format!("{} [TIER {}]", award.metallic_tier().as_str().to_uppercase(), award.tier), col, bg, col)
        } else if wins > 0 && is_completed {
            ("CHAMPION [GOLD 🏆]".to_string(), Palette::NEON_GOLD, Color::new(0.25, 0.20, 0.05, 0.90), Palette::NEON_GOLD)
        } else if podiums > 0 && is_completed {
            ("PODIUM FINISHER 🥈".to_string(), Color::new(0.85, 0.88, 0.95, 1.0), Color::new(0.12, 0.16, 0.24, 0.90), Color::new(0.85, 0.88, 0.95, 0.6))
        } else if is_active_session {
            (format!("ROUND {} IN PROGRESS", completed_rounds + 1), Palette::NEON_CYAN, Color::new(0.08, 0.20, 0.28, 0.90), Palette::NEON_CYAN)
        } else if completed_rounds > 0 {
            (format!("{}/{} ROUNDS DONE", completed_rounds, total_rounds), Palette::NEON_CYAN, Color::new(0.08, 0.14, 0.20, 0.90), Palette::UI_CARD_BORDER)
        } else {
            ("AVAILABLE TO ENTER".to_string(), Palette::UI_TEXT_MUTED, Color::new(0.06, 0.08, 0.12, 0.70), Palette::UI_CARD_BORDER)
        };

        // Render card background
        let card_bg = if is_card_selected {
            if is_unlocked {
                Color::new(0.08, 0.14, 0.22, 0.95)
            } else {
                Color::new(0.12, 0.07, 0.08, 0.95)
            }
        } else if !is_unlocked {
            Color::new(0.05, 0.06, 0.08, 0.80)
        } else {
            Color::new(0.06, 0.08, 0.12, 0.90)
        };

        let border_thickness = if is_card_selected { 2.2 } else { 1.0 };
        scaler.draw_glass_card(x + pad, cy, inner_w, item_h, card_bg, card_border, border_thickness);

        if is_card_selected {
            draw_rectangle(
                x + pad,
                cy,
                scaler.s(4.5),
                item_h,
                if is_unlocked {
                    if is_active_session { Palette::NEON_GOLD } else { Palette::NEON_CYAN }
                } else {
                    Color::new(0.85, 0.35, 0.35, 1.0)
                },
            );
        }

        // ---------------------------------------------------------------------
        // 1. LEFT SIDE: CAR LATERAL THUMBNAIL (256x128 aspect ratio 2:1)
        // ---------------------------------------------------------------------
        let (model_id, display_car_name, has_raced) = resolve_championship_car_model_id(champ, history);

        let img_box_w = scaler.s(92.0);
        let img_box_h = scaler.s(50.0);
        let img_box_x = x + pad + scaler.s(7.0);
        let img_box_y = cy + (item_h - img_box_h) * 0.5;

        draw_rectangle(img_box_x, img_box_y, img_box_w, img_box_h, Color::new(0.04, 0.05, 0.08, 0.95));
        draw_rectangle_lines(img_box_x, img_box_y, img_box_w, img_box_h, 1.0, Palette::UI_CARD_BORDER);

        let car_thumb_w = scaler.s(84.0);
        let car_thumb_h = scaler.s(42.0); // Exactly 2:1 aspect ratio!
        let car_x = img_box_x + (img_box_w - car_thumb_w) * 0.5;
        let car_y = img_box_y + (img_box_h - car_thumb_h) * 0.5;

        let model = crate::catalog::find_model_by_id(model_id);
        let (factory_prim, factory_sec) = model
            .map(|m| (m.primary_color, m.secondary_color))
            .unwrap_or((Palette::WHITE, Palette::WHITE));

        // Use factory colors so is_factory evaluates to true and get_vehicle_lateral_texture
        // returns the base image directly without any red livery mask or tinting.
        if let Some(texture) = get_vehicle_lateral_texture(model_id, factory_prim, factory_sec, false) {
            let img_tint = if is_unlocked {
                Palette::WHITE
            } else {
                Color::new(0.60, 0.65, 0.70, 0.85)
            };
            draw_texture_ex(
                &texture,
                car_x,
                car_y,
                img_tint,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(car_thumb_w, car_thumb_h)),
                    ..Default::default()
                },
            );
        } else {
            let factory_scheme = CarColorScheme {
                primary: factory_prim,
                secondary: factory_sec,
                helmet: Palette::WHITE,
            };
            render_real_car_lateral_by_id(
                model_id,
                &factory_scheme,
                img_box_x + img_box_w * 0.5,
                img_box_y + img_box_h * 0.5,
                scaler.s(0.35),
                0.0,
                false,
            );
        }

        // ---------------------------------------------------------------------
        // 2. RIGHT SIDE: TROPHY BADGE (IF WON) & STATUS BADGE & STATS COUNTER
        // ---------------------------------------------------------------------
        let trophy_sz = scaler.s(48.0);
        let has_trophy = champ_award.is_some();
        let trophy_pad = if has_trophy { trophy_sz + scaler.s(8.0) } else { 0.0 };

        if let Some(award) = champ_award {
            let trophy_x = x + pad + inner_w - trophy_sz - scaler.s(8.0);
            let trophy_y = cy + (item_h - trophy_sz) * 0.5;
            draw_trophy_badge(
                trophy_x,
                trophy_y,
                trophy_sz,
                trophy_sz,
                award.discipline(),
                award.tier,
                Some(award.metallic_tier()),
                false,
            );
        }

        let badge_w = scaler.s(if is_card_selected { 165.0 } else { 145.0 });
        let badge_x = x + pad + inner_w - badge_w - scaler.s(10.0) - trophy_pad;
        let badge_y = cy + scaler.s(10.0);
        let badge_h = scaler.s(22.0);

        draw_rectangle(badge_x, badge_y, badge_w, badge_h, status_bg);
        draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, if is_card_selected { 1.8 } else { 1.0 }, status_col);
        fonts.draw_ui_bold_centered(&status_text, badge_x + badge_w * 0.5, badge_y + scaler.s(15.5), scaler.font_s(10.0), status_col);

        let stats_summary = format!("WINS: {}  •  PODIUMS: {}", wins, podiums);
        fonts.draw_ui_bold_centered(&stats_summary, badge_x + badge_w * 0.5, cy + scaler.s(48.0), scaler.font_s(10.5), Palette::WHITE);

        // ---------------------------------------------------------------------
        // 3. MIDDLE-RIGHT: PROGRESS BAR & PERSONAL BEST STATS
        // ---------------------------------------------------------------------
        let prog_w = scaler.s(140.0);
        let prog_x = badge_x - prog_w - scaler.s(16.0);

        fonts.draw_ui_bold(&progress_label, prog_x, cy + scaler.s(19.0), scaler.font_s(10.0), progress_col);

        let bar_h = scaler.s(6.0);
        let bar_y = cy + scaler.s(26.0);
        draw_rectangle(prog_x, bar_y, prog_w, bar_h, Color::new(0.10, 0.13, 0.18, 0.90));
        draw_rectangle(prog_x, bar_y, prog_w * progress_ratio, bar_h, progress_col);
        draw_rectangle_lines(prog_x, bar_y, prog_w, bar_h, 1.0, Palette::UI_CARD_BORDER);

        let best_str = format!("BEST: {}  •  LAP: {}", best_finish_str, best_lap_str);
        fonts.draw_ui_regular(&best_str, prog_x, cy + scaler.s(48.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);

        // ---------------------------------------------------------------------
        // 4. MIDDLE-LEFT: CHAMPIONSHIP TITLE, DISCIPLINE TAG & CAR/TRACK SUMMARY
        // ---------------------------------------------------------------------
        let info_x = img_box_x + img_box_w + scaler.s(12.0);

        // Row 1: Title
        let title_col = if !is_unlocked {
            Palette::UI_TEXT_MUTED
        } else if is_card_selected {
            Palette::WHITE
        } else {
            Color::new(0.92, 0.94, 0.98, 1.0)
        };
        fonts.draw_ui_bold(&champ.series.name, info_x, cy + scaler.s(19.0), scaler.font_s(13.0), title_col);

        let has_varying_laps = champ
            .rounds
            .iter()
            .any(|r| r.laps.is_some_and(|l| l != champ.series.laps_per_round));
        let laps_label = if has_varying_laps {
            "3–5 LAPS".to_string()
        } else {
            format!("{} LAPS", champ.series.laps_per_round)
        };

        // Row 2: Tag
        let disc_tag = format!(
            "{} • TIER {} • {} ROUNDS ({}) • {} PTS",
            champ.series.module_id.to_uppercase(),
            champ.series.tier,
            total_rounds,
            laps_label,
            champ.scoring.system.to_uppercase()
        );
        let tag_col = if !is_unlocked {
            Color::new(0.55, 0.55, 0.60, 0.8)
        } else {
            Palette::NEON_CYAN
        };
        fonts.draw_ui_bold(&disc_tag, info_x, cy + scaler.s(34.0), scaler.font_s(10.0), tag_col);

        // Row 3: Car name in use & Calendar preview
        let track_names: Vec<String> = champ.rounds.iter().take(3).map(|r| {
            if let Some(n) = &r.name {
                n.clone()
            } else {
                r.track_id.replace('_', " ").to_uppercase()
            }
        }).collect();
        let track_preview = if champ.rounds.len() > 3 {
            format!("{} +{} more", track_names.join(" • "), champ.rounds.len() - 3)
        } else {
            track_names.join(" • ")
        };

        let car_prefix = if has_raced { "RACED:" } else { "ENTRY:" };
        let meta_line = format!("{} {}  •  {}", car_prefix, display_car_name, track_preview);
        fonts.draw_ui_regular(&meta_line, info_x, cy + scaler.s(49.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);

        cy += item_h + item_gap;
    }

    // Scrollbar indicator
    if max_scroll > 0 {
        let scrollbar_x = x + pad + inner_w - scaler.s(3.0);
        let scrollbar_h = cy - (y + pad + scaler.s(74.0));
        let thumb_h = (scrollbar_h * (visible_count as f32 / filtered_champs.len() as f32)).max(scaler.s(20.0));
        let thumb_y = (y + pad + scaler.s(74.0)) + (scrollbar_h - thumb_h) * (scroll as f32 / max_scroll as f32);
        draw_rectangle(scrollbar_x, y + pad + scaler.s(74.0), scaler.s(3.0), scrollbar_h, Color::new(0.12, 0.15, 0.20, 0.60));
        draw_rectangle(scrollbar_x, thumb_y, scaler.s(3.0), thumb_h, Palette::NEON_CYAN);
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
    is_filter_focused: bool,
) {
    let pad = scaler.s(14.0);
    let inner_w = w - pad * 2.0;
    let mut cy = y + pad;

    // --- Category Filter Pills Row ---
    let active_filter = TELEMETRY_CATEGORY_FILTERS.get(filter_category_idx).copied().unwrap_or(TELEMETRY_CATEGORY_FILTERS[0]);
    let pill_h = scaler.s(24.0);
    let pill_gap = scaler.s(6.0);
    let mut px = x + pad;

    let filter_label_col = if is_filter_focused {
        Palette::NEON_CYAN
    } else {
        Palette::NEON_GOLD
    };
    fonts.draw_ui_bold(
        if is_filter_focused { "FILTER [◄/►]:" } else { "FILTER:" },
        px,
        cy + scaler.s(16.0),
        scaler.font_s(11.0),
        filter_label_col,
    );
    px += scaler.s(if is_filter_focused { 84.0 } else { 55.0 });

    for (f_idx, (f_name, _)) in TELEMETRY_CATEGORY_FILTERS.iter().enumerate() {
        let is_sel = f_idx == filter_category_idx;
        let pill_w = scaler.s(if *f_name == "ALL" { 48.0 } else { 75.0 });

        let p_bg = if is_sel {
            if is_filter_focused {
                Color::new(0.10, 0.26, 0.38, 0.95)
            } else {
                Palette::UI_CARD_BG_HOVER
            }
        } else {
            Color::new(0.08, 0.10, 0.14, 0.70)
        };
        let p_border = if is_sel {
            Palette::NEON_CYAN
        } else {
            Palette::UI_CARD_BORDER
        };

        draw_rectangle(px, cy, pill_w, pill_h, p_bg);
        let border_thickness = if is_sel && is_filter_focused {
            2.4
        } else if is_sel {
            1.8
        } else {
            1.0
        };
        draw_rectangle_lines(px, cy, pill_w, pill_h, border_thickness, p_border);

        if is_sel && is_filter_focused {
            // Neon top and bottom accent lines on active focused filter pill
            draw_rectangle(px, cy, pill_w, scaler.s(2.0), Palette::NEON_CYAN);
            draw_rectangle(px, cy + pill_h - scaler.s(2.0), pill_w, scaler.s(2.0), Palette::NEON_CYAN);
        }

        let label_text = if is_sel && is_filter_focused {
            format!("◄ {} ►", f_name)
        } else {
            f_name.to_string()
        };

        fonts.draw_ui_bold_centered(
            &label_text,
            px + pill_w * 0.5,
            cy + scaler.s(16.5),
            scaler.font_s(if is_sel && is_filter_focused { 10.0 } else { 10.5 }),
            if is_sel { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
        );

        px += pill_w + pill_gap;
    }

    let filter_hint = if is_filter_focused {
        "[◄ / ►] Select Filter  •  [▼] Logs  •  [▲] Tabs"
    } else {
        "[▼] Module Filters  •  [F] Quick Filter"
    };
    fonts.draw_ui_regular(
        filter_hint,
        x + pad + inner_w - scaler.s(250.0),
        cy + scaler.s(16.5),
        scaler.font_s(11.0),
        if is_filter_focused { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
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

/// Renders the Two-Column Driver Roster & Profile Manager Screen.
#[allow(clippy::too_many_arguments)]
pub fn render_player_roster_manager_screen(
    fonts: &Fonts,
    profiles: &[PlayerProfile],
    selected_idx: usize,
    active_column: usize,
    field_idx: usize,
    name_input: &str,
    alias_input: &str,
    country_idx: usize,
    livery_idx: usize,
    assist_mode: AssistProfile,
    stats: &ProfileCareerStats,
    cursor_timer: f32,
    status_msg: Option<&str>,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Dark backdrop overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.96));

    let full_w = sw * 0.96;
    let full_h = sh * 0.92;
    let x = (sw - full_w) * 0.5;
    let mut cur_y = (sh - full_h) * 0.5;

    // Header Bar
    let header_h = scaler.s(52.0);
    scaler.draw_glass_card(x, cur_y, full_w, header_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.4);
    draw_rectangle(x, cur_y, scaler.s(6.0), header_h, Palette::NEON_CYAN);

    fonts.draw_display(
        "DRIVER ROSTER & PROFILE MANAGER",
        x + scaler.s(16.0),
        cur_y + scaler.s(26.0),
        scaler.font_s(18.0),
        Palette::WHITE,
    );
    fonts.draw_ui_regular(
        "MANAGE REGISTERED RACERS, EDIT IDENTITY, CONFIGURE DRIVING ASSISTS, AND INSPECT CAREER DOSSIER",
        x + scaler.s(16.0),
        cur_y + scaler.s(43.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    if let Some(msg) = status_msg {
        let msg_dim = fonts.measure_ui_bold(msg, scaler.font_s(12.0));
        let badge_w = msg_dim.width + scaler.s(24.0);
        let badge_x = x + full_w - badge_w - scaler.s(16.0);
        let badge_y = cur_y + scaler.s(12.0);
        let badge_h = scaler.s(28.0);
        draw_rectangle(badge_x, badge_y, badge_w, badge_h, Color::new(0.08, 0.28, 0.16, 0.90));
        draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.2, Palette::NEON_GREEN);
        fonts.draw_ui_bold_centered(msg, badge_x + badge_w * 0.5, badge_y + scaler.s(18.0), scaler.font_s(12.0), Palette::NEON_GREEN);
    }

    cur_y += header_h + scaler.s(10.0);

    let footer_h = scaler.s(32.0);
    let body_h = (sh - cur_y - footer_h - scaler.s(10.0)).max(scaler.s(400.0));
    let col_left_w = full_w * 0.35;
    let gap = scaler.s(14.0);
    let col_right_w = full_w - col_left_w - gap;
    let col_left_x = x;
    let col_right_x = x + col_left_w + gap;

    // =========================================================================
    // LEFT COLUMN: DRIVER ROSTER LIST
    // =========================================================================
    let left_border = if active_column == 0 { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
    let left_thick = if active_column == 0 { 2.0 } else { 1.2 };
    scaler.draw_glass_card(col_left_x, cur_y, col_left_w, body_h, Palette::UI_CARD_BG, left_border, left_thick);

    let col_title = format!("REGISTERED DRIVERS ({})", profiles.len());
    fonts.draw_ui_bold(&col_title, col_left_x + scaler.s(14.0), cur_y + scaler.s(24.0), scaler.font_s(14.0), if active_column == 0 { Palette::NEON_CYAN } else { Palette::WHITE });
    fonts.draw_ui_regular("[▲ / ▼] Browse  •  [ENTER] Active  •  [TAB / ►] Edit", col_left_x + scaler.s(14.0), cur_y + scaler.s(40.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

    draw_rectangle(col_left_x + scaler.s(10.0), cur_y + scaler.s(48.0), col_left_w - scaler.s(20.0), 1.0, Palette::UI_CARD_BORDER);

    let list_y = cur_y + scaler.s(54.0);
    let item_h = scaler.s(52.0);
    let item_gap = scaler.s(6.0);
    let bottom_actions_h = scaler.s(42.0);
    let list_avail_h = body_h - scaler.s(54.0) - bottom_actions_h;
    let visible_items = (list_avail_h / (item_h + item_gap)).floor().max(1.0) as usize;
    let scroll_offset = if selected_idx >= visible_items {
        selected_idx - visible_items + 1
    } else {
        0
    };

    let mut item_cur_y = list_y;
    for (i, p) in profiles.iter().enumerate().skip(scroll_offset).take(visible_items) {
        let is_sel = i == selected_idx;
        let item_w = col_left_w - scaler.s(20.0);
        let item_x = col_left_x + scaler.s(10.0);

        let (ibg, iborder, ithick) = if is_sel && active_column == 0 {
            (Color::new(0.10, 0.16, 0.28, 0.95), Palette::NEON_CYAN, 2.0)
        } else if is_sel {
            (Color::new(0.08, 0.12, 0.20, 0.90), Palette::WHITE, 1.4)
        } else {
            (Color::new(0.06, 0.08, 0.12, 0.70), Palette::UI_CARD_BORDER, 1.0)
        };

        draw_rectangle(item_x, item_cur_y, item_w, item_h, ibg);
        draw_rectangle_lines(item_x, item_cur_y, item_w, item_h, ithick, iborder);

        if is_sel {
            draw_rectangle(item_x, item_cur_y, scaler.s(4.0), item_h, Palette::NEON_CYAN);
        }

        // Flag banner
        let b_w = scaler.s(36.0);
        let b_h = scaler.s(18.0);
        let b_x = item_x + scaler.s(10.0);
        let b_y = item_cur_y + scaler.s(9.0);
        draw_country_banner(p.country.as_deref(), b_x, b_y, b_w, b_h, Some(fonts), &scaler);

        // Driver Name
        let name_x = b_x + b_w + scaler.s(10.0);
        let name_size = scaler.font_s(14.0);
        let name_col = if is_sel { Palette::WHITE } else { Color::new(0.85, 0.88, 0.95, 1.0) };
        fonts.draw_display(&p.name, name_x, item_cur_y + scaler.s(22.0), name_size, name_col);

        // Callsign / Alias & Mode
        let sub_str = format!("\"{}\"  •  {}", p.alias, p.last_mode.short_name());
        fonts.draw_ui_regular(&sub_str, name_x, item_cur_y + scaler.s(40.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

        // Livery swatches on right
        let sw_s = scaler.s(10.0);
        let sw_y = item_cur_y + scaler.s(12.0);
        let sw_right = item_x + item_w - scaler.s(10.0);
        draw_rectangle(sw_right - sw_s, sw_y, sw_s, sw_s, p.color_scheme.helmet);
        draw_rectangle(sw_right - sw_s * 2.0 - scaler.s(2.0), sw_y, sw_s, sw_s, p.color_scheme.secondary);
        draw_rectangle(sw_right - sw_s * 3.0 - scaler.s(4.0), sw_y, sw_s, sw_s, p.color_scheme.primary);

        // Active Badge
        if p.is_active {
            let active_w = scaler.s(54.0);
            let active_h = scaler.s(16.0);
            let active_x = item_x + item_w - active_w - scaler.s(8.0);
            let active_y = item_cur_y + scaler.s(28.0);
            draw_rectangle(active_x, active_y, active_w, active_h, Color::new(0.08, 0.25, 0.12, 0.85));
            draw_rectangle_lines(active_x, active_y, active_w, active_h, 1.0, Palette::NEON_GREEN);
            fonts.draw_ui_bold_centered("ACTIVE", active_x + active_w * 0.5, active_y + scaler.s(12.0), scaler.font_s(9.0), Palette::NEON_GREEN);
        }

        item_cur_y += item_h + item_gap;
    }

    // Bottom Left Actions
    let btn_row_y = cur_y + body_h - bottom_actions_h;
    draw_rectangle(col_left_x + scaler.s(10.0), btn_row_y, col_left_w - scaler.s(20.0), 1.0, Palette::UI_CARD_BORDER);

    let btn_action_y = btn_row_y + scaler.s(8.0);
    let btn_action_h = scaler.s(26.0);
    let btn_n_w = scaler.s(90.0);
    let btn_del_w = scaler.s(90.0);

    draw_rectangle(col_left_x + scaler.s(14.0), btn_action_y, btn_n_w, btn_action_h, Color::new(0.12, 0.18, 0.26, 0.90));
    draw_rectangle_lines(col_left_x + scaler.s(14.0), btn_action_y, btn_n_w, btn_action_h, 1.0, Palette::NEON_CYAN);
    fonts.draw_ui_bold_centered("[N] NEW", col_left_x + scaler.s(14.0) + btn_n_w * 0.5, btn_action_y + scaler.s(18.0), scaler.font_s(11.0), Palette::NEON_CYAN);

    if profiles.len() > 1 {
        let del_x = col_left_x + scaler.s(14.0) + btn_n_w + scaler.s(8.0);
        draw_rectangle(del_x, btn_action_y, btn_del_w, btn_action_h, Color::new(0.22, 0.10, 0.12, 0.90));
        draw_rectangle_lines(del_x, btn_action_y, btn_del_w, btn_action_h, 1.0, Color::new(0.95, 0.45, 0.40, 1.0));
        fonts.draw_ui_bold_centered("[DEL] DELETE", del_x + btn_del_w * 0.5, btn_action_y + scaler.s(18.0), scaler.font_s(11.0), Color::new(0.95, 0.45, 0.40, 1.0));
    }

    // =========================================================================
    // RIGHT COLUMN: DRIVER IDENTITY SETUP & CAREER DOSSIER
    // =========================================================================
    let right_border = if active_column == 1 { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
    let right_thick = if active_column == 1 { 2.0 } else { 1.2 };
    scaler.draw_glass_card(col_right_x, cur_y, col_right_w, body_h, Palette::UI_CARD_BG, right_border, right_thick);

    let right_pad = scaler.s(16.0);
    let right_inner_w = col_right_w - right_pad * 2.0;
    let mut ry = cur_y + right_pad;

    fonts.draw_ui_bold(
        "DRIVER IDENTITY & CUSTOMIZATION",
        col_right_x + right_pad,
        ry + scaler.s(10.0),
        scaler.font_s(14.0),
        if active_column == 1 { Palette::NEON_CYAN } else { Palette::WHITE },
    );
    fonts.draw_ui_regular(
        "MODIFY NAME, ALIAS, NATIONALITY BANNER, TEAM LIVERY, AND DRIVING DIFFICULTY",
        col_right_x + right_pad,
        ry + scaler.s(26.0),
        scaler.font_s(10.5),
        Palette::UI_TEXT_MUTED,
    );

    ry += scaler.s(36.0);

    let field_h = scaler.s(38.0);
    let field_w = (right_inner_w - scaler.s(12.0)) * 0.5;
    let show_cursor = (cursor_timer * 2.5).fract() < 0.5;

    // Row 1: Field 0 (Name) and Field 1 (Alias)
    let f0_sel = active_column == 1 && field_idx == 0;
    let f1_sel = active_column == 1 && field_idx == 1;

    render_text_field(&scaler, fonts, col_right_x + right_pad, ry, field_w, field_h, "DRIVER FULL NAME", name_input, f0_sel, show_cursor && f0_sel, "Enter Name");
    render_text_field(&scaler, fonts, col_right_x + right_pad + field_w + scaler.s(12.0), ry, field_w, field_h, "CALLSIGN / ALIAS", alias_input, f1_sel, show_cursor && f1_sel, "Enter Alias");

    ry += field_h + scaler.s(14.0);

    // Row 2: Field 2 (Country Banner) and Field 3 (Livery Theme)
    let f2_sel = active_column == 1 && field_idx == 2;
    let f3_sel = active_column == 1 && field_idx == 3;

    let country_info = if country_idx > 0 && country_idx <= CountryRegistry::ALL.len() {
        Some(&CountryRegistry::ALL[country_idx - 1])
    } else {
        None
    };
    let country_title = country_info
        .map(|c| format!("{} ({})", c.name, c.code))
        .unwrap_or_else(|| "International (Worldwide)".to_string());
    let c_border = if f2_sel { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };
    draw_rectangle(col_right_x + right_pad, ry, field_w, field_h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(col_right_x + right_pad, ry, field_w, field_h, if f2_sel { 2.0 } else { 1.0 }, c_border);
    fonts.draw_ui_bold("NATIONALITY & BANNER [◄ / ►]", col_right_x + right_pad + scaler.s(10.0), ry - scaler.s(4.0), scaler.font_s(10.0), if f2_sel { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED });
    let cb_w = scaler.s(40.0);
    let cb_h = scaler.s(20.0);
    let c_code = country_info.map(|c| c.code);
    draw_country_banner(c_code, col_right_x + right_pad + scaler.s(10.0), ry + scaler.s(9.0), cb_w, cb_h, Some(fonts), &scaler);
    fonts.draw_ui_bold(&country_title, col_right_x + right_pad + cb_w + scaler.s(20.0), ry + scaler.s(24.0), scaler.font_s(12.5), Palette::WHITE);

    let livery_x = col_right_x + right_pad + field_w + scaler.s(12.0);
    let l_border = if f3_sel { Palette::NEON_MAGENTA } else { Palette::UI_CARD_BORDER };
    draw_rectangle(livery_x, ry, field_w, field_h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(livery_x, ry, field_w, field_h, if f3_sel { 2.0 } else { 1.0 }, l_border);
    fonts.draw_ui_bold("TEAM LIVERY & COLORS [◄ / ►]", livery_x + scaler.s(10.0), ry - scaler.s(4.0), scaler.font_s(10.0), if f3_sel { Palette::NEON_MAGENTA } else { Palette::UI_TEXT_MUTED });

    let scheme = CarColorScheme::from_index(livery_idx);
    let sw_w = scaler.s(20.0);
    let sw_h = scaler.s(16.0);
    let sw_x = livery_x + scaler.s(12.0);
    let sw_y = ry + scaler.s(11.0);
    draw_rectangle(sw_x, sw_y, sw_w, sw_h, scheme.primary);
    draw_rectangle_lines(sw_x, sw_y, sw_w, sw_h, 1.0, Palette::WHITE);
    draw_rectangle(sw_x + sw_w + scaler.s(4.0), sw_y, sw_w, sw_h, scheme.secondary);
    draw_rectangle_lines(sw_x + sw_w + scaler.s(4.0), sw_y, sw_w, sw_h, 1.0, Palette::WHITE);
    draw_rectangle(sw_x + (sw_w + scaler.s(4.0)) * 2.0, sw_y, sw_w, sw_h, scheme.helmet);
    draw_rectangle_lines(sw_x + (sw_w + scaler.s(4.0)) * 2.0, sw_y, sw_w, sw_h, 1.0, Palette::WHITE);
    let livery_name = format!("Theme #{}", (livery_idx % Palette::CAR_COLORS.len()) + 1);
    fonts.draw_ui_bold(&livery_name, sw_x + (sw_w + scaler.s(4.0)) * 3.0 + scaler.s(10.0), ry + scaler.s(24.0), scaler.font_s(12.5), Palette::WHITE);

    ry += field_h + scaler.s(14.0);

    // Row 3: Field 4 (Assist Mode Profile) and Field 5 (Save Button)
    let f4_sel = active_column == 1 && field_idx == 4;
    let f5_sel = active_column == 1 && field_idx == 5;

    let a_border = if f4_sel { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER };
    draw_rectangle(col_right_x + right_pad, ry, field_w, field_h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(col_right_x + right_pad, ry, field_w, field_h, if f4_sel { 2.0 } else { 1.0 }, a_border);
    fonts.draw_ui_bold("DRIVING ASSISTS / HANDLING [◄ / ►]", col_right_x + right_pad + scaler.s(10.0), ry - scaler.s(4.0), scaler.font_s(10.0), if f4_sel { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED });
    fonts.draw_ui_bold(assist_mode.title(), col_right_x + right_pad + scaler.s(14.0), ry + scaler.s(24.0), scaler.font_s(12.5), Palette::NEON_GOLD);

    let save_x = col_right_x + right_pad + field_w + scaler.s(12.0);
    let (s_bg, s_border, s_col) = if f5_sel {
        (Color::new(0.08, 0.28, 0.35, 0.95), Palette::NEON_CYAN, Palette::WHITE)
    } else {
        (Color::new(0.08, 0.16, 0.22, 0.90), Palette::UI_CARD_BORDER, Palette::NEON_CYAN)
    };
    draw_rectangle(save_x, ry, field_w, field_h, s_bg);
    draw_rectangle_lines(save_x, ry, field_w, field_h, if f5_sel { 2.0 } else { 1.2 }, s_border);
    fonts.draw_ui_bold_centered("[ENTER / A] SAVE CHANGES", save_x + field_w * 0.5, ry + scaler.s(24.0), scaler.font_s(13.0), s_col);

    ry += field_h + scaler.s(16.0);

    draw_rectangle(col_right_x + right_pad, ry, right_inner_w, 1.0, Palette::UI_CARD_BORDER);
    ry += scaler.s(12.0);

    // Lifetime Career Dossier
    fonts.draw_ui_bold("LIFETIME CAREER DOSSIER", col_right_x + right_pad, ry + scaler.s(12.0), scaler.font_s(13.5), Palette::NEON_CYAN);
    ry += scaler.s(22.0);

    let tile_gap = scaler.s(8.0);
    let tile_w = (right_inner_w - tile_gap * 5.0) / 6.0;
    let tile_h = scaler.s(48.0);

    let win_str = format!("{} ({:.0}%)", stats.wins, stats.win_rate);
    let podium_str = format!("{} ({:.0}%)", stats.podiums, stats.podium_rate);
    let clean_str = format!("{:.1}%", stats.clean_rate);

    render_kpi_tile(&scaler, fonts, col_right_x + right_pad, ry, tile_w, tile_h, "TOTAL RACES", &stats.total_races.to_string(), Palette::NEON_CYAN);
    render_kpi_tile(&scaler, fonts, col_right_x + right_pad + (tile_w + tile_gap), ry, tile_w, tile_h, "WINS (P1)", &win_str, Palette::NEON_GOLD);
    render_kpi_tile(&scaler, fonts, col_right_x + right_pad + (tile_w + tile_gap) * 2.0, ry, tile_w, tile_h, "P2 RUNNER-UP", &stats.p2_count.to_string(), Color::new(0.85, 0.88, 0.95, 1.0));
    render_kpi_tile(&scaler, fonts, col_right_x + right_pad + (tile_w + tile_gap) * 3.0, ry, tile_w, tile_h, "P3 THIRD", &stats.p3_count.to_string(), Color::new(0.88, 0.55, 0.25, 1.0));
    render_kpi_tile(&scaler, fonts, col_right_x + right_pad + (tile_w + tile_gap) * 4.0, ry, tile_w, tile_h, "PODIUMS", &podium_str, Palette::NEON_GREEN);
    render_kpi_tile(&scaler, fonts, col_right_x + right_pad + (tile_w + tile_gap) * 5.0, ry, tile_w, tile_h, "CLEAN RACE %", &clean_str, Palette::NEON_MAGENTA);

    ry += tile_h + scaler.s(12.0);

    let stat_panel_w = (right_inner_w - scaler.s(12.0)) * 0.5;
    let stat_panel_h = (cur_y + body_h - ry - scaler.s(10.0)).max(scaler.s(80.0));

    // Stunt Box
    scaler.draw_glass_card(col_right_x + right_pad, ry, stat_panel_w, stat_panel_h, Color::new(0.06, 0.08, 0.12, 0.85), Palette::NEON_GOLD, 1.0);
    fonts.draw_ui_bold("STUNT PORTFOLIO", col_right_x + right_pad + scaler.s(10.0), ry + scaler.s(18.0), scaler.font_s(11.5), Palette::NEON_GOLD);
    render_data_row(&scaler, fonts, col_right_x + right_pad + scaler.s(10.0), ry + scaler.s(36.0), stat_panel_w - scaler.s(20.0), "Accumulated Stunt Points", &format!("{} PTS", stats.total_stunt_score), Palette::NEON_GOLD);
    render_data_row(&scaler, fonts, col_right_x + right_pad + scaler.s(10.0), ry + scaler.s(54.0), stat_panel_w - scaler.s(20.0), "Total Laps Under Telemetry", &format!("{} LAPS", stats.total_laps), Palette::WHITE);

    // Incident Box
    let inc_x = col_right_x + right_pad + stat_panel_w + scaler.s(12.0);
    scaler.draw_glass_card(inc_x, ry, stat_panel_w, stat_panel_h, Color::new(0.06, 0.08, 0.12, 0.85), Palette::NEON_CYAN, 1.0);
    fonts.draw_ui_bold("SAFETY & INCIDENT RECORD", inc_x + scaler.s(10.0), ry + scaler.s(18.0), scaler.font_s(11.5), Palette::NEON_CYAN);
    render_data_row(&scaler, fonts, inc_x + scaler.s(10.0), ry + scaler.s(36.0), stat_panel_w - scaler.s(20.0), "Total Collisions", &format!("{} IMPACTS", stats.total_collisions), if stats.total_collisions == 0 { Palette::NEON_GREEN } else { Color::new(0.95, 0.45, 0.35, 1.0) });
    render_data_row(&scaler, fonts, inc_x + scaler.s(10.0), ry + scaler.s(54.0), stat_panel_w - scaler.s(20.0), "Incident-Free Races", &format!("{} / {}", stats.clean_races, stats.total_races), Palette::NEON_GREEN);

    // Footer Action Bar
    let foot_y = sh - scaler.s(18.0);
    let footer_prompt = if active_column == 0 {
        "[▲ / ▼] Select Driver  |  [ENTER] Set Active  |  [TAB / ►] Edit Details  |  [N] New Driver  |  [DEL / X] Delete  |  [ESC] Exit"
    } else {
        "[▲ / ▼] Select Field  |  [◄ / ►] Change Value  |  [ENTER] Save  |  [TAB / ◄] Back to Roster  |  [ESC] Exit"
    };
    fonts.draw_ui_bold_centered(
        footer_prompt,
        sw * 0.5,
        foot_y,
        scaler.font_s(12.5),
        Palette::WHITE,
    );
}
