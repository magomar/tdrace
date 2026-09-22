use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use tdrace_core::track::Track;

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::game::GridParticipant;
use crate::profile::{draw_country_banner, PlayerProfile};
use crate::render::color::{CarColorScheme, Palette};
use crate::ui::menu::{CarChoice, GameMode};

use crate::render::lateral::render_car_lateral;

/// Selected active column/panel in StartingGrid pre-race setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartingGridFocus {
    LeftSetup,
    RightRoster,
}

/// Returns the rectangle (x, y, w, h) of the Garage access card on the Starting Grid.
pub fn starting_grid_garage_button_rect(sw: f32, sh: f32) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let col_w = (sw * 0.44).clamp(scaler.s(360.0), scaler.s(540.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(12.0)).max(scaler.safe_pad_x);
    let panel_y = scaler.s(60.0);
    let p1_h = scaler.s(88.0);
    let garage_card_h = scaler.s(420.0);
    let curr_y = panel_y + p1_h + scaler.s(8.0);
    (col1_x, curr_y, col_w, garage_card_h)
}

/// Returns the rectangle (x, y, w, h) of the high-visibility Launch Race button on the Starting Grid.
pub fn starting_grid_launch_button_rect(sw: f32, sh: f32) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let col_w = (sw * 0.44).clamp(scaler.s(360.0), scaler.s(540.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(12.0)).max(scaler.safe_pad_x);
    let panel_y = scaler.s(60.0);

    let p1_h = scaler.s(88.0);
    let garage_card_h = scaler.s(420.0);
    let launch_h = scaler.s(48.0);

    let curr_y = panel_y + p1_h + scaler.s(8.0) + garage_card_h + scaler.s(10.0);
    (col1_x, curr_y, col_w, launch_h)
}

/// Returns the rectangle (x, y, w, h) of the Grid Configuration card at the top of the right column.
pub fn starting_grid_grid_button_rect(sw: f32, sh: f32) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let col_w = (sw * 0.44).clamp(scaler.s(360.0), scaler.s(540.0));
    let col2_x = (sw * 0.5 + scaler.s(12.0)).min(sw - col_w - scaler.safe_pad_x);
    let panel_y = scaler.s(60.0);
    let grid_h = scaler.s(52.0);
    (col2_x, panel_y, col_w, grid_h)
}

/// Renders the 2-panel starting grid and participants showcase screen before race launch.
#[allow(clippy::too_many_arguments)]
pub fn render_starting_grid_screen(
    fonts: &Fonts,
    track: &Track,
    player_profile: &PlayerProfile,
    game_mode: GameMode,
    active_car: CarChoice,
    _predefined_car: CarChoice,
    grid_participants: &[GridParticipant],
    total_laps: u32,
    best_lap_time: Option<f32>,
    num_drivers: usize,
    max_grid_size: usize,
    gamepad_connected: bool,
    focused_panel: StartingGridFocus,
    active_card_idx: usize,
    active_roster_idx: usize,
    is_car_unlocked: bool,
    is_car_eligible: bool,
    required_tier: u8,
    unlock_level: u32,
    selected_model_id: Option<&str>,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);
    let model_opt = selected_model_id.and_then(crate::catalog::find_model_by_id);
    let player_scheme = if let Some(m) = model_opt {
        CarColorScheme {
            primary: m.primary_color,
            secondary: m.secondary_color,
            helmet: player_profile.color_scheme.helmet,
        }
    } else {
        player_profile.color_scheme
    };

    // Dark glass backdrop overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.06, 0.10, 0.90));

    // Header Title
    let title = "STARTING GRID & ROSTER SETUP";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(28.0),
        scaler.font_s(26.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    // Subtitle / Track details
    let track_len_m = track.spline.total_length().round() as i32;
    let subtitle = format!(
        "Circuit: {}  •  Mode: {}  •  Distance: {} Laps",
        track.name.to_uppercase(),
        game_mode.title().to_uppercase(),
        total_laps,
    );
    fonts.draw_ui_regular_centered(
        &subtitle,
        sw * 0.5,
        scaler.s(48.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );

    // Two Columns Geometry
    let col_w = (sw * 0.44).clamp(scaler.s(360.0), scaler.s(540.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(12.0)).max(scaler.safe_pad_x);
    let col2_x = (sw * 0.5 + scaler.s(12.0)).min(sw - col_w - scaler.safe_pad_x);
    let panel_y = scaler.s(60.0);
    let bottom_prompt_y = sh - scaler.s(24.0);

    let is_left_focused = focused_panel == StartingGridFocus::LeftSetup;
    let is_right_focused = focused_panel == StartingGridFocus::RightRoster;

    // =========================================================================
    // LEFT PANEL: Player Details, Track Details, Game Mode, Car Specs
    // =========================================================================
    let mut curr_y = panel_y;

    // Card 1: Player Profile & Circuit Dossier Card
    let p1_h = scaler.s(88.0);
    scaler.draw_glass_card(col1_x, curr_y, col_w, p1_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.2);

    // Line 1: Player Profile info
    let flag_w = scaler.s(32.0);
    let flag_h = scaler.s(18.0);
    draw_country_banner(
        player_profile.country.as_deref(),
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(10.0),
        flag_w,
        flag_h,
        None,
        &scaler,
    );

    let p_name_str = format!("{}  (\"{}\")", player_profile.name, player_profile.alias);
    fonts.draw_ui_bold(
        &p_name_str,
        col1_x + scaler.s(52.0),
        curr_y + scaler.s(23.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );

    // Team Livery Swatches on right of player row
    let swatch_w = scaler.s(13.0);
    let swatch_h = scaler.s(10.0);
    let swatch_x = col1_x + col_w - scaler.s(58.0);
    let swatch_y = curr_y + scaler.s(12.0);
    draw_rectangle(swatch_x, swatch_y, swatch_w, swatch_h, player_scheme.primary);
    draw_rectangle_lines(swatch_x, swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
    draw_rectangle(swatch_x + swatch_w + scaler.s(2.0), swatch_y, swatch_w, swatch_h, player_scheme.secondary);
    draw_rectangle_lines(swatch_x + swatch_w + scaler.s(2.0), swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
    draw_rectangle(swatch_x + (swatch_w + scaler.s(2.0)) * 2.0, swatch_y, swatch_w, swatch_h, player_scheme.helmet);
    draw_rectangle_lines(swatch_x + (swatch_w + scaler.s(2.0)) * 2.0, swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    // Divider Line
    draw_rectangle(col1_x + scaler.s(12.0), curr_y + scaler.s(38.0), col_w - scaler.s(24.0), 1.0, Color::new(0.20, 0.30, 0.45, 0.40));

    // Line 2: Track Information
    fonts.draw_ui_bold(
        &format!("CIRCUIT: {}", track.name.to_uppercase()),
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(56.0),
        scaler.font_s(12.5),
        Palette::NEON_GOLD,
    );
    fonts.draw_ui_regular(
        &format!("{}m Length  •  {} Laps  •  {}", track_len_m, total_laps, track.surface_summary_string()),
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(74.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    curr_y += p1_h + scaler.s(8.0);

    // Card 2 (Index 0): Enlarged Motorsport Garage & Active Car Card
    let is_garage_active = is_left_focused && active_card_idx == 0;
    let garage_card_h = scaler.s(420.0);
    let (mx, my) = std::panic::catch_unwind(macroquad::input::mouse_position).unwrap_or((-1000.0, -1000.0));
    let is_garage_hovered = mx >= col1_x && mx <= col1_x + col_w && my >= curr_y && my <= curr_y + garage_card_h;
    let is_garage_highlighted = is_garage_active || is_garage_hovered;

    let garage_border = if !is_car_unlocked {
        Palette::RED
    } else if is_garage_highlighted {
        Palette::NEON_GOLD
    } else {
        Palette::UI_CARD_BORDER
    };
    let garage_bg = if !is_car_unlocked {
        Color::new(0.12, 0.04, 0.04, 0.85)
    } else if is_garage_highlighted {
        Palette::UI_CARD_BG_HOVER
    } else {
        Palette::UI_CARD_BG
    };
    scaler.draw_glass_card(col1_x, curr_y, col_w, garage_card_h, garage_bg, garage_border, if is_garage_highlighted || !is_car_unlocked { 2.4 } else { 1.2 });

    let locked_header_str = format!("GARAGE: 🔒 LOCKED [LEVEL {} REQUIRED]", unlock_level);
    let garage_header_title = if !is_car_unlocked {
        &locked_header_str
    } else if is_garage_highlighted {
        if game_mode.allows_car_change() {
            "GARAGE [ACTIVE • CLICK / ENTER to open • [ / ] to switch]"
        } else {
            "GARAGE [ACTIVE • CLICK / ENTER / G to open]"
        }
    } else {
        "GARAGE [ACTIVE CAR • Click to open Garage]"
    };
    let garage_header_col = if !is_car_unlocked {
        Palette::RED
    } else if is_garage_highlighted {
        Palette::NEON_GOLD
    } else {
        Palette::UI_TEXT_MUTED
    };
    fonts.draw_ui_bold(
        garage_header_title,
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(16.0),
        scaler.font_s(11.0),
        garage_header_col,
    );

    // "OPEN GARAGE [G]" badge / button hint
    fonts.draw_ui_bold(
        "OPEN GARAGE [G]",
        col1_x + col_w - scaler.s(130.0),
        curr_y + scaler.s(16.0),
        scaler.font_s(10.0),
        Palette::NEON_CYAN,
    );

    let car_title = model_opt.map(|m| m.name).unwrap_or_else(|| active_car.title());
    let car_desc = model_opt.map(|m| m.history_bio).unwrap_or_else(|| active_car.description());

    fonts.draw_ui_bold(
        car_title,
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(34.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );

    let car_tag_str = if !is_car_unlocked {
        "🔒 LOCKED"
    } else {
        active_car.tag()
    };
    let car_tag_col = if !is_car_unlocked {
        Palette::RED
    } else {
        Palette::NEON_GOLD
    };
    fonts.draw_ui_bold(
        car_tag_str,
        col1_x + col_w - scaler.s(160.0),
        curr_y + scaler.s(34.0),
        scaler.font_s(10.0),
        car_tag_col,
    );

    fonts.draw_ui_regular(
        car_desc,
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(50.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    // 2D Lateral View Blueprint Showcase Box (Enlarged and aspect-ratio preserved)
    let stat_base_x = col1_x + scaler.s(12.0);
    let stat_bar_w = col_w - scaler.s(24.0);
    let lateral_box_y = curr_y + scaler.s(66.0);
    let lateral_box_h = scaler.s(160.0);
    scaler.draw_glass_card(
        stat_base_x,
        lateral_box_y,
        stat_bar_w,
        lateral_box_h,
        Color::new(0.03, 0.05, 0.08, 0.65),
        Color::new(0.18, 0.25, 0.35, 0.50),
        1.0,
    );
    if let Some(m) = model_opt {
        let preview_scheme = if m.module_id == "classic" {
            CarColorScheme {
                primary: m.primary_color,
                secondary: m.secondary_color,
                helmet: player_scheme.helmet,
            }
        } else {
            player_scheme
        };
        crate::render::lateral::render_real_car_lateral_by_id(
            m.id,
            &preview_scheme,
            stat_base_x + stat_bar_w * 0.50,
            lateral_box_y + lateral_box_h * 0.54,
            scaler.s(1.55),
            0.0,
            true,
        );
    } else {
        render_car_lateral(
            active_car,
            &player_scheme,
            stat_base_x + stat_bar_w * 0.50,
            lateral_box_y + lateral_box_h * 0.54,
            scaler.s(1.55),
            0.0,
            true,
        );
    }

    // 4 Performance Stat Bars
    let (spd, acc, grip, drift) = if let Some(m) = model_opt {
        (m.stats.0, m.stats.1, m.stats.2, m.stats.3)
    } else {
        active_car.stats()
    };
    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(236.0), stat_bar_w, "SPEED", spd, Palette::NEON_CYAN);
    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(254.0), stat_bar_w, "ACCEL", acc, Palette::NEON_GOLD);
    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(272.0), stat_bar_w, "GRIP", grip, Palette::NEON_GREEN);
    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(290.0), stat_bar_w, "DRIFT", drift, Palette::NEON_MAGENTA);

    // 4 Engineering / Dynamic Specs Chips
    let (spec1, spec2, spec3, spec4) = if let Some(m) = model_opt {
        (
            format!("{} BHP", m.bhp),
            format!("{} kg", m.weight_kg),
            format!("{} km/h", m.top_speed_kmh),
            m.aero_downforce.to_string(),
        )
    } else {
        let (s1, s2, s3, s4) = active_car.specs();
        (s1.to_string(), s2.to_string(), s3.to_string(), s4.to_string())
    };
    let spec_chip_w = (col_w - scaler.s(32.0)) * 0.5;
    let spec_chip_h = scaler.s(24.0);
    let chip_y1 = curr_y + scaler.s(314.0);
    let chip_y2 = curr_y + scaler.s(344.0);

    scaler.draw_glass_card(stat_base_x, chip_y1, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(&spec1, stat_base_x + scaler.s(8.0), chip_y1 + scaler.s(16.0), scaler.font_s(10.5), Palette::NEON_CYAN);

    scaler.draw_glass_card(stat_base_x + spec_chip_w + scaler.s(8.0), chip_y1, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(&spec2, stat_base_x + spec_chip_w + scaler.s(16.0), chip_y1 + scaler.s(16.0), scaler.font_s(10.5), Palette::WHITE);

    scaler.draw_glass_card(stat_base_x, chip_y2, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(&spec3, stat_base_x + scaler.s(8.0), chip_y2 + scaler.s(16.0), scaler.font_s(10.5), Palette::NEON_GOLD);

    scaler.draw_glass_card(stat_base_x + spec_chip_w + scaler.s(8.0), chip_y2, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(&spec4, stat_base_x + spec_chip_w + scaler.s(16.0), chip_y2 + scaler.s(16.0), scaler.font_s(10.5), Palette::NEON_GREEN);

    // Prompt hint at bottom of card
    if is_garage_highlighted {
        let hint_text = if game_mode.allows_car_change() {
            "Press [ENTER], [G] or CLICK to open Garage showroom & switch vehicle"
        } else {
            "Press [ENTER], [G] or CLICK to open Garage showroom"
        };
        fonts.draw_ui_regular(
            hint_text,
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(396.0),
            scaler.font_s(10.0),
            Palette::NEON_GOLD,
        );
    }

    curr_y += garage_card_h + scaler.s(10.0);

    // Card 3: High-Visibility Green "LAUNCH RACE" Action Button
    let launch_h = scaler.s(48.0);
    let is_launch_card = is_left_focused && (active_card_idx == 1 || active_card_idx == 2);
    let is_launch_hovered = mx >= col1_x && mx <= col1_x + col_w && my >= curr_y && my <= curr_y + launch_h;
    let is_launch_active = is_launch_card || is_launch_hovered;

    let (launch_bg, launch_border, launch_title, launch_sub) = if !is_car_unlocked {
        (
            Color::new(0.35, 0.10, 0.10, 0.95),
            Palette::RED,
            "🔒 VEHICLE LOCKED",
            format!("Advance Career to Level {} to Unlock", unlock_level),
        )
    } else if !is_car_eligible {
        (
            Color::new(0.35, 0.10, 0.10, 0.95),
            Palette::RED,
            "🚫 INELIGIBLE CATEGORY",
            format!("Requires Tier {} or Lower (Selected: Tier {})", required_tier, active_car.tier()),
        )
    } else if is_launch_active {
        (
            Color::new(0.12, 0.68, 0.32, 0.98),
            Palette::NEON_GREEN,
            "▶ LAUNCH RACE  [ENTER / SPACE / CLICK]",
            "SPACE / Gamepad A".to_string(),
        )
    } else {
        (
            Color::new(0.08, 0.44, 0.22, 0.92),
            Color::new(0.20, 0.78, 0.40, 0.85),
            "▶ LAUNCH RACE",
            "SPACE / Gamepad A".to_string(),
        )
    };

    draw_rectangle(col1_x, curr_y, col_w, launch_h, launch_bg);
    draw_rectangle_lines(
        col1_x,
        curr_y,
        col_w,
        launch_h,
        if is_launch_active || !is_car_unlocked || !is_car_eligible { 2.8 * scaler.scale } else { 1.6 * scaler.scale },
        launch_border,
    );

    fonts.draw_ui_bold_centered(
        launch_title,
        col1_x + col_w * 0.5,
        curr_y + scaler.s(21.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
    fonts.draw_ui_regular_centered(
        &launch_sub,
        col1_x + col_w * 0.5,
        curr_y + scaler.s(37.0),
        scaler.font_s(13.5),
        if !is_car_unlocked { Color::new(1.0, 0.8, 0.8, 0.95) } else { Color::new(0.85, 1.0, 0.90, 0.95) },
    );

    // =========================================================================
    // RIGHT PANEL: Grid Config (Top) & Starting Grid Roster (Below)
    // =========================================================================
    let grid_y = panel_y;
    let grid_h = scaler.s(52.0);
    let is_roster_locked = !game_mode.allows_roster_customization();
    let is_grid_hovered = mx >= col2_x && mx <= col2_x + col_w && my >= grid_y && my <= grid_y + grid_h;
    let is_grid_active = (is_right_focused && active_card_idx == 1) || (is_left_focused && active_card_idx == 1) || is_grid_hovered;

    let grid_border = if is_grid_active {
        Palette::NEON_CYAN
    } else {
        Palette::UI_CARD_BORDER
    };
    let grid_bg = if is_grid_active {
        Palette::UI_CARD_BG_HOVER
    } else {
        Palette::UI_CARD_BG
    };
    scaler.draw_glass_card(col2_x, grid_y, col_w, grid_h, grid_bg, grid_border, if is_grid_active { 2.4 } else { 1.2 });

    if game_mode.has_bots() {
        let grid_hdr = if is_roster_locked {
            if game_mode == GameMode::Career {
                "GRID CONFIG: 🔒 LOCKED [Championship Roster]"
            } else {
                "GRID CONFIG: 🔒 LOCKED [Official Roster]"
            }
        } else if is_grid_active {
            "GRID CONFIG [ACTIVE • ENTER / + / - to adjust]"
        } else {
            "GRID CONFIG: [Up/Down to select • Click to adjust]"
        };
        fonts.draw_ui_bold(
            grid_hdr,
            col2_x + scaler.s(12.0),
            grid_y + scaler.s(17.0),
            scaler.font_s(11.0),
            if is_grid_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
        );
        let racer_desc = if is_roster_locked && game_mode == GameMode::Career {
            format!("{} Racers (Championship Grid) • Official Season Roster Locked", num_drivers)
        } else if game_mode == GameMode::SplitScreen {
            let bot_count = num_drivers.saturating_sub(2);
            if bot_count == 0 {
                format!("2 Players (1v1 Head-to-Head Duel) • Max {} Slots", max_grid_size)
            } else {
                format!("{} Racers (2 Players + {} AI Bots) • Max {} Slots", num_drivers, bot_count, max_grid_size)
            }
        } else {
            format!("{} Racers ({} AI Opponents) • Max {} Slots", num_drivers, num_drivers.saturating_sub(1), max_grid_size)
        };
        fonts.draw_ui_bold(
            &racer_desc,
            col2_x + scaler.s(12.0),
            grid_y + scaler.s(36.0),
            scaler.font_s(13.0),
            Palette::WHITE,
        );
    } else {
        let solo_hdr = if is_grid_active {
            "SESSION STATUS [ACTIVE • Solo Track Time]"
        } else {
            "SESSION STATUS: SOLO TRACK TIME"
        };
        fonts.draw_ui_bold(
            solo_hdr,
            col2_x + scaler.s(12.0),
            grid_y + scaler.s(17.0),
            scaler.font_s(11.0),
            if is_grid_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
        );
        let status_str = match game_mode {
            GameMode::TimeTrial => {
                format!("Personal Best: {} • Shadow Car Active", best_lap_time.map(format_lap_time).unwrap_or_else(|| "No Record".to_string()))
            }
            GameMode::FreeRide => "Open Practice Session • Unlimited Laps • Zero Traffic".to_string(),
            _ => "Solo Practice".to_string(),
        };
        fonts.draw_ui_bold(
            &status_str,
            col2_x + scaler.s(12.0),
            grid_y + scaler.s(36.0),
            scaler.font_s(12.5),
            Palette::WHITE,
        );
    }

    // Roster Header (Positioned directly under Grid Config card)
    let roster_base_title = match game_mode {
        GameMode::TimeTrial => "TIME TRIAL • ROSTER & SHADOW CAR",
        GameMode::FreeRide => "FREE RIDE • PRACTICE ROSTER",
        GameMode::StandardRace => "STANDARD RACE • STARTING GRID",
        GameMode::ExperimentalRace => "CUSTOM RACE • STARTING GRID & ROSTER",
        GameMode::Career => "CAREER CHAMPIONSHIP • STARTING GRID [LOCKED]",
        GameMode::SplitScreen => "2P SPLIT SCREEN • KEYS VS GAMEPAD",
    };
    let roster_header_is_focused = is_right_focused && active_card_idx != 1;
    let roster_header = if roster_header_is_focused {
        if game_mode.allows_roster_customization() {
            format!("{} [FOCUSED • Up/Down select • [ / ] Change Car • ENTER/D Dossier]", roster_base_title)
        } else {
            format!("{} [FOCUSED • Up/Down to select slot • ENTER/D for Dossier]", roster_base_title)
        }
    } else {
        format!("{} [Right arrow to focus roster]", roster_base_title)
    };
    let roster_header_y = grid_y + grid_h + scaler.s(16.0);
    fonts.draw_ui_bold(
        &roster_header,
        col2_x,
        roster_header_y,
        scaler.font_s(13.0),
        if roster_header_is_focused { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED },
    );

    let roster_card_y = roster_header_y + scaler.s(10.0);
    let roster_card_h = (bottom_prompt_y - roster_card_y - scaler.s(8.0)).max(scaler.s(220.0));
    let roster_border = if roster_header_is_focused { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER };
    scaler.draw_glass_card(col2_x, roster_card_y, col_w, roster_card_h, Palette::UI_CARD_BG, roster_border, if roster_header_is_focused { 2.2 } else { 1.4 });

    let row_w = col_w - scaler.s(16.0);
    let row_x = col2_x + scaler.s(8.0);
    let mut row_y = roster_card_y + scaler.s(8.0);
    let row_h = scaler.s(46.0);
    let row_gap = scaler.s(5.0);

    match game_mode {
        GameMode::TimeTrial => {
            // Row 1: Player (P1)
            let is_row_sel = is_right_focused && active_card_idx != 1 && active_roster_idx == 0;
            let pb_desc = best_lap_time.map(format_lap_time).unwrap_or_else(|| "No Prior Record".to_string());
            let p_line = format!("Personal Best: {}  •  Live Driver Telemetry", pb_desc);
            render_participant_row(
                fonts,
                &scaler,
                row_x,
                row_y,
                row_w,
                row_h,
                1,
                &player_profile.name,
                &player_profile.alias,
                player_profile.country.as_deref(),
                active_car.title(),
                player_profile.color_scheme,
                &p_line,
                true,
                is_row_sel,
            );
            row_y += row_h + row_gap;

            // Row 2: Shadow / Ghost Car
            let is_ghost_sel = is_right_focused && active_card_idx != 1 && active_roster_idx == 1;
            let ghost_lap_str = best_lap_time.map(|t| format!("Ghost Target: {}  •  Live Telemetry Replay", format_lap_time(t))).unwrap_or_else(|| "No prior lap recorded  •  Recording live ghost".to_string());
            render_ghost_participant_row(
                fonts,
                &scaler,
                row_x,
                row_y,
                row_w,
                row_h,
                active_car.title(),
                &ghost_lap_str,
                is_ghost_sel,
            );
            row_y += row_h + scaler.s(14.0);

            // Explanatory info box for Time Trial mode
            let info_h = scaler.s(160.0);
            scaler.draw_glass_card(row_x, row_y, row_w, info_h, Color::new(0.05, 0.08, 0.13, 0.8), Palette::UI_CARD_BORDER, 1.0);
            fonts.draw_ui_bold("TIME TRIAL BENCHMARK & SHADOW VEHICLE", row_x + scaler.s(12.0), row_y + scaler.s(18.0), scaler.font_s(12.5), Palette::NEON_CYAN);
            let tips = [
                "• Race against your personal best time recorded in the Hall of Fame",
                "• Personal best lap is rendered live as an interactive shadow / ghost car",
                "• Real-time delta comparison (+/- seconds) displayed in racing HUD",
                "• All car choices unlocked for telemetry tuning and benchmark comparison",
            ];
            let mut tip_y = row_y + scaler.s(42.0);
            for tip in &tips {
                fonts.draw_ui_regular(tip, row_x + scaler.s(12.0), tip_y, scaler.font_s(11.0), Palette::WHITE);
                tip_y += scaler.s(24.0);
            }
        }
        GameMode::FreeRide => {
            // Row 1: Player (P1)
            let is_row_sel = is_right_focused && active_card_idx != 1 && active_roster_idx == 0;
            let p_line = "Unlimited Open Circuit Session  •  Zero Obstacle Traffic".to_string();
            render_participant_row(
                fonts,
                &scaler,
                row_x,
                row_y,
                row_w,
                row_h,
                1,
                &player_profile.name,
                &player_profile.alias,
                player_profile.country.as_deref(),
                active_car.title(),
                player_profile.color_scheme,
                &p_line,
                true,
                is_row_sel,
            );
            row_y += row_h + scaler.s(14.0);

            // Explanatory info box for Free Ride mode
            let info_h = scaler.s(160.0);
            scaler.draw_glass_card(row_x, row_y, row_w, info_h, Color::new(0.05, 0.08, 0.13, 0.8), Palette::UI_CARD_BORDER, 1.0);
            fonts.draw_ui_bold("FREE RIDE PRACTICE & CIRCUIT TUNING", row_x + scaler.s(12.0), row_y + scaler.s(18.0), scaler.font_s(12.5), Palette::NEON_GREEN);
            let tips = [
                "• Unrestricted circuit testing with zero lap limits or opponent traffic",
                "• Freely test vehicle weight transfer, slip angles, and slide recovery",
                "• Practice apex clipping zones, curb riding, and throttle control",
                "• Press [SPACE] to launch practice session on the starting grid",
            ];
            let mut tip_y = row_y + scaler.s(42.0);
            for tip in &tips {
                fonts.draw_ui_regular(tip, row_x + scaler.s(12.0), tip_y, scaler.font_s(11.0), Palette::WHITE);
                tip_y += scaler.s(24.0);
            }
        }
        GameMode::StandardRace | GameMode::ExperimentalRace | GameMode::SplitScreen | GameMode::Career => {
            for (i, participant) in grid_participants.iter().enumerate() {
                let slot = i + 1;
                let is_row_sel = is_right_focused && active_card_idx != 1 && i == active_roster_idx;
                let desc = if game_mode == GameMode::SplitScreen && i == 0 {
                    "Player 1: Keyboard (WASD / Arrows) • Grid Slot 1".to_string()
                } else if game_mode == GameMode::SplitScreen && i == 1 {
                    if gamepad_connected {
                        "Player 2: Gamepad [CONNECTED: Analog Precision]".to_string()
                    } else {
                        "Player 2: Gamepad [NOT DETECTED - Connect Controller / Fallback Arrows]".to_string()
                    }
                } else {
                    match (participant.best_lap, participant.best_circuit_time) {
                        (Some(lap), Some(circ)) => {
                            format!("Best Lap: {}  •  Circuit: {}", format_lap_time(lap), format_lap_time(circ))
                        }
                        (Some(lap), None) => format!("Best Lap: {}", format_lap_time(lap)),
                        (None, Some(circ)) => format!("Circuit: {}", format_lap_time(circ)),
                        (None, None) => {
                            if participant.is_player {
                                "No Prior Record  •  Grid Draw".to_string()
                            } else {
                                "No Prior Record  •  Rookie Draw".to_string()
                            }
                        }
                    }
                };

                let display_name = if game_mode == GameMode::SplitScreen && i == 0 {
                    format!("{} (P1 Keys)", participant.name)
                } else if game_mode == GameMode::SplitScreen && i == 1 {
                    format!("{} (P2 Gamepad)", participant.name)
                } else if participant.is_player {
                    format!("{} (You)", participant.name)
                } else {
                    participant.name.clone()
                };

                render_participant_row(
                    fonts,
                    &scaler,
                    row_x,
                    row_y,
                    row_w,
                    row_h,
                    slot,
                    &display_name,
                    &participant.alias,
                    participant.country.as_deref(),
                    &participant.car_title,
                    participant.color_scheme,
                    &desc,
                    participant.is_player,
                    is_row_sel,
                );
                row_y += row_h + row_gap;
            }
        }
    }

    // =========================================================================
    // FOOTER PROMPTS
    // =========================================================================
    let prompt = starting_grid_footer_prompt_with_mode(gamepad_connected, focused_panel, active_card_idx, game_mode.allows_roster_customization());

    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        bottom_prompt_y + scaler.s(6.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );
}

/// Returns the footer navigation and action prompt string for the starting grid screen.
pub fn starting_grid_footer_prompt(
    gamepad_connected: bool,
    focused_panel: StartingGridFocus,
    active_card_idx: usize,
) -> &'static str {
    starting_grid_footer_prompt_with_mode(gamepad_connected, focused_panel, active_card_idx, true)
}

/// Returns the footer prompt with awareness of whether the roster can be customized in the current game mode.
pub fn starting_grid_footer_prompt_with_mode(
    gamepad_connected: bool,
    focused_panel: StartingGridFocus,
    active_card_idx: usize,
    is_roster_customizable: bool,
) -> &'static str {
    if gamepad_connected {
        match focused_panel {
            StartingGridFocus::LeftSetup => match active_card_idx {
                0 => "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [A] Open Garage  |  [START] Launch  |  [B] Menu",
                1 => if is_roster_customizable {
                    "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [A/X] Adjust Bots  |  [START] Launch  |  [B] Menu"
                } else {
                    "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [ROSTER LOCKED]  |  [START] Launch  |  [B] Menu"
                },
                _ => "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [A / START] LAUNCH RACE  |  [B] Menu",
            },
            StartingGridFocus::RightRoster => {
                if active_card_idx == 1 {
                    if is_roster_customizable {
                        "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Driver  |  [A/X] Adjust Bots  |  [START] Launch  |  [B] Menu"
                    } else {
                        "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Driver  |  [ROSTER LOCKED]  |  [START] Launch  |  [B] Menu"
                    }
                } else {
                    "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Driver  |  [A/Y] View Dossier  |  [START] Launch  |  [B] Menu"
                }
            }
        }
    } else {
        match focused_panel {
            StartingGridFocus::LeftSetup => match active_card_idx {
                0 => "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ENTER] Open Garage  |  [SPACE] Launch  |  [ESC] Menu",
                1 => if is_roster_customizable {
                    "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ENTER / + / -] Adjust Bots  |  [SPACE] Launch  |  [ESC] Menu"
                } else {
                    "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ROSTER LOCKED]  |  [SPACE] Launch  |  [ESC] Menu"
                },
                _ => "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ENTER / SPACE / CLICK] LAUNCH RACE  |  [ESC] Menu",
            },
            StartingGridFocus::RightRoster => {
                if active_card_idx == 1 {
                    if is_roster_customizable {
                        "[Left/Right] Switch Panel  |  [Up/Down] Select Card/Driver  |  [ENTER / + / -] Adjust Bots  |  [SPACE] Launch  |  [ESC] Menu"
                    } else {
                        "[Left/Right] Switch Panel  |  [Up/Down] Select Card/Driver  |  [ROSTER LOCKED]  |  [SPACE] Launch  |  [ESC] Menu"
                    }
                } else if is_roster_customizable {
                    "[Left/Right] Switch Panel  |  [Up/Down] Select Driver  |  [< / >] Change Vehicle  |  [ENTER / D] View Dossier  |  [SPACE] Launch  |  [ESC] Menu"
                } else {
                    "[Left/Right] Switch Panel  |  [Up/Down] Select Driver  |  [ENTER / D] View Dossier  |  [SPACE] Launch  |  [ESC] Menu"
                }
            }
        }
    }
}

fn render_grid_stat_bar(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    pct: f32,
    fill_col: Color,
) {
    let bar_h = scaler.s(10.0);
    let label_w = scaler.s(55.0);

    fonts.draw_ui_bold(label, x, y + scaler.s(9.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);

    let bar_x = x + label_w;
    let actual_bar_w = w - label_w - scaler.s(45.0);
    draw_rectangle(bar_x, y, actual_bar_w, bar_h, Color::new(0.08, 0.10, 0.15, 0.90));
    draw_rectangle_lines(bar_x, y, actual_bar_w, bar_h, 1.0, Palette::UI_CARD_BORDER);

    let filled_w = actual_bar_w * pct.clamp(0.0, 1.0);
    draw_rectangle(bar_x, y, filled_w, bar_h, fill_col);

    let pct_str = format!("{:.0}%", pct * 100.0);
    fonts.draw_ui_bold(&pct_str, x + w - scaler.s(38.0), y + scaler.s(9.0), scaler.font_s(10.0), Palette::WHITE);
}

fn render_ghost_participant_row(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    car_name: &str,
    best_lap_str: &str,
    is_selected: bool,
) {
    let bg_color = if is_selected {
        Color::new(0.14, 0.24, 0.38, 0.95)
    } else {
        Color::new(0.06, 0.10, 0.16, 0.85)
    };
    let border_color = if is_selected {
        Palette::NEON_GOLD
    } else {
        Palette::NEON_CYAN
    };

    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, if is_selected { 2.2 } else { 1.2 }, border_color);

    // Badge: "GHOST"
    let badge_w = scaler.s(50.0);
    let badge_h = h - scaler.s(10.0);
    let badge_x = x + scaler.s(6.0);
    let badge_y = y + scaler.s(5.0);

    draw_rectangle(badge_x, badge_y, badge_w, badge_h, Palette::NEON_CYAN);
    fonts.draw_ui_bold_centered(
        "SHADOW",
        badge_x + badge_w * 0.5,
        badge_y + badge_h * 0.5 + scaler.s(4.0),
        scaler.font_s(11.0),
        Palette::BLACK,
    );

    let text_start_x = badge_x + badge_w + scaler.s(10.0);

    // Right Column: Car Model
    let car_col_w = scaler.s(175.0);
    let car_x = x + w - car_col_w;

    let sep_x = car_x - scaler.s(8.0);
    draw_rectangle(sep_x, y + scaler.s(6.0), 1.0, h - scaler.s(12.0), Color::new(0.25, 0.35, 0.45, 0.40));
    fonts.draw_ui_bold(car_name, car_x, y + scaler.s(21.0), scaler.font_s(12.5), Palette::NEON_CYAN);

    // Title & Info
    fonts.draw_ui_bold("Personal Best Benchmark", text_start_x, y + scaler.s(21.0), scaler.font_s(13.5), Palette::NEON_CYAN);
    fonts.draw_ui_regular(best_lap_str, text_start_x, y + scaler.s(37.0), scaler.font_s(11.0), Color::new(0.80, 0.95, 1.0, 0.9));
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
    country: Option<&str>,
    car_name: &str,
    scheme: CarColorScheme,
    profile_line: &str,
    is_player: bool,
    is_selected: bool,
) {
    // Row background
    let bg_color = if is_selected {
        Color::new(0.18, 0.28, 0.42, 0.98)
    } else if is_player {
        Color::new(0.12, 0.22, 0.35, 0.95)
    } else if pos % 2 == 0 {
        Color::new(0.08, 0.11, 0.17, 0.90)
    } else {
        Color::new(0.06, 0.08, 0.13, 0.90)
    };

    let border_color = if is_selected {
        Palette::NEON_GOLD
    } else if is_player {
        Palette::NEON_CYAN
    } else {
        Palette::UI_CARD_BORDER
    };

    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, if is_selected { 2.2 } else if is_player { 1.8 } else { 1.0 }, border_color);

    // Position Badge (e.g. "P1", "P2")
    let badge_w = scaler.s(36.0);
    let badge_h = h - scaler.s(10.0);
    let badge_x = x + scaler.s(6.0);
    let badge_y = y + scaler.s(5.0);

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

    // Country Banner if present
    let mut text_start_x = badge_x + badge_w + scaler.s(10.0);
    if country.is_some() || is_player {
        let flag_w = scaler.s(34.0);
        let flag_h = scaler.s(18.0);
        draw_country_banner(country, text_start_x, y + scaler.s(7.0), flag_w, flag_h, None, scaler);
        text_start_x += flag_w + scaler.s(8.0);
    }

    // Right Column: Car Model & Livery Swatches
    let car_col_w = scaler.s(175.0);
    let car_x = x + w - car_col_w;

    // Subtle vertical separator line between driver profile and vehicle column
    let sep_x = car_x - scaler.s(8.0);
    draw_rectangle(sep_x, y + scaler.s(6.0), 1.0, h - scaler.s(12.0), Color::new(0.25, 0.35, 0.45, 0.40));

    // Car title & Swatches
    fonts.draw_ui_bold(car_name, car_x, y + scaler.s(21.0), scaler.font_s(12.5), Palette::WHITE);

    let swatch_w = scaler.s(14.0);
    let swatch_h = scaler.s(10.0);
    let s_y = y + scaler.s(28.0);

    draw_rectangle(car_x, s_y, swatch_w, swatch_h, scheme.primary);
    draw_rectangle_lines(car_x, s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
    draw_rectangle(car_x + swatch_w + scaler.s(2.0), s_y, swatch_w, swatch_h, scheme.secondary);
    draw_rectangle_lines(car_x + swatch_w + scaler.s(2.0), s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
    draw_rectangle(car_x + (swatch_w + scaler.s(2.0)) * 2.0, s_y, swatch_w, swatch_h, scheme.helmet);
    draw_rectangle_lines(car_x + (swatch_w + scaler.s(2.0)) * 2.0, s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    fonts.draw_ui_regular("Team Livery", car_x + scaler.s(58.0), s_y + scaler.s(8.5), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);

    let driver_label = if is_selected {
        format!("{}  (\"{}\")  [ENTER / D: DOSSIER]", name, alias)
    } else if is_player {
        format!("{}  (\"{}\")", name, alias)
    } else {
        format!("{}  (\"{}\")", name, alias)
    };

    let name_color = if is_selected {
        Palette::NEON_GOLD
    } else if is_player {
        Palette::NEON_CYAN
    } else {
        Palette::WHITE
    };
    let stats_color = if is_selected {
        Palette::NEON_GOLD
    } else if is_player {
        Palette::NEON_GOLD
    } else {
        Color::new(0.60, 0.85, 0.95, 1.0)
    };

    // Line 1: Driver Name & Alias
    fonts.draw_ui_bold(&driver_label, text_start_x, y + scaler.s(21.0), scaler.font_s(13.5), name_color);

    // Line 2: Profile / Style stats
    fonts.draw_ui_regular(profile_line, text_start_x, y + scaler.s(37.0), scaler.font_s(10.5), stats_color);
}

