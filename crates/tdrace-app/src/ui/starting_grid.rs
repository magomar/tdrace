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

/// Selected active column/panel in StartingGrid pre-race setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartingGridFocus {
    LeftSetup,
    RightRoster,
}

/// Returns the rectangle (x, y, w, h) of the high-visibility Launch Race button on the Starting Grid.
pub fn starting_grid_launch_button_rect(sw: f32, sh: f32) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let col_w = (sw * 0.44).clamp(scaler.s(360.0), scaler.s(540.0));
    let col1_x = (sw * 0.5 - col_w - scaler.s(12.0)).max(scaler.safe_pad_x);
    let panel_y = scaler.s(60.0);

    let p1_h = scaler.s(88.0);
    let mode_h = scaler.s(72.0);
    let car_card_h = scaler.s(188.0);
    let grid_h = scaler.s(48.0);
    let launch_h = scaler.s(48.0);

    let curr_y = panel_y + p1_h + scaler.s(8.0) + mode_h + scaler.s(8.0) + car_card_h + scaler.s(8.0) + grid_h + scaler.s(10.0);
    (col1_x, curr_y, col_w, launch_h)
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
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

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
    draw_rectangle(swatch_x, swatch_y, swatch_w, swatch_h, player_profile.color_scheme.primary);
    draw_rectangle_lines(swatch_x, swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
    draw_rectangle(swatch_x + swatch_w + scaler.s(2.0), swatch_y, swatch_w, swatch_h, player_profile.color_scheme.secondary);
    draw_rectangle_lines(swatch_x + swatch_w + scaler.s(2.0), swatch_y, swatch_w, swatch_h, 1.0, Palette::WHITE);
    draw_rectangle(swatch_x + (swatch_w + scaler.s(2.0)) * 2.0, swatch_y, swatch_w, swatch_h, player_profile.color_scheme.helmet);
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

    // Card 2: Game Mode Selector Card
    let is_mode_active = is_left_focused && active_card_idx == 0;
    let mode_h = scaler.s(72.0);
    let mode_border = if is_mode_active {
        Palette::NEON_GOLD
    } else {
        Palette::UI_CARD_BORDER
    };
    let mode_bg = if is_mode_active {
        Palette::UI_CARD_BG_HOVER
    } else {
        Palette::UI_CARD_BG
    };
    scaler.draw_glass_card(col1_x, curr_y, col_w, mode_h, mode_bg, mode_border, if is_mode_active { 2.4 } else { 1.2 });

    let mode_header_label = if is_mode_active {
        "GAME MODE [ACTIVE • ENTER / SPACE to cycle]"
    } else {
        "GAME MODE: [Up/Down to select card]"
    };
    fonts.draw_ui_bold(
        mode_header_label,
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(16.0),
        scaler.font_s(11.0),
        if is_mode_active { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED },
    );
    fonts.draw_ui_bold(
        game_mode.tag(),
        col1_x + col_w - scaler.s(180.0),
        curr_y + scaler.s(16.0),
        scaler.font_s(9.5),
        Palette::NEON_CYAN,
    );

    fonts.draw_ui_bold(
        game_mode.title(),
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(36.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
    fonts.draw_ui_regular(
        game_mode.description(),
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(54.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    curr_y += mode_h + scaler.s(8.0);

    // Card 3: Vehicle Selection & Specs Card
    let is_car_active = is_left_focused && active_card_idx == 1;
    let car_card_h = scaler.s(214.0);
    let car_border_col = if is_car_active {
        if game_mode.allows_car_change() { Palette::NEON_GREEN } else { Palette::NEON_CYAN }
    } else {
        Palette::UI_CARD_BORDER
    };
    let car_bg = if is_car_active {
        Palette::UI_CARD_BG_HOVER
    } else {
        Palette::UI_CARD_BG
    };
    scaler.draw_glass_card(col1_x, curr_y, col_w, car_card_h, car_bg, car_border_col, if is_car_active { 2.4 } else { 1.2 });

    let car_header_title = if is_car_active {
        if game_mode.allows_car_change() {
            "CAR SELECTION [ACTIVE • ENTER / < / > to switch]"
        } else {
            "CAR SPEC: ENFORCED PREDEFINED [Locked]"
        }
    } else {
        if game_mode.allows_car_change() {
            "CAR SELECTION: [Up/Down to select card]"
        } else {
            "CAR SPEC: ENFORCED PREDEFINED [Locked]"
        }
    };
    let car_header_col = if is_car_active {
        if game_mode.allows_car_change() { Palette::NEON_GREEN } else { Palette::NEON_CYAN }
    } else {
        Palette::UI_TEXT_MUTED
    };
    fonts.draw_ui_bold(
        car_header_title,
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(16.0),
        scaler.font_s(11.0),
        car_header_col,
    );
    fonts.draw_ui_bold(
        active_car.tag(),
        col1_x + col_w - scaler.s(160.0),
        curr_y + scaler.s(16.0),
        scaler.font_s(10.0),
        Palette::NEON_GOLD,
    );

    fonts.draw_ui_bold(
        active_car.title(),
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(34.0),
        scaler.font_s(16.5),
        Palette::WHITE,
    );
    fonts.draw_ui_regular(
        active_car.description(),
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(49.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    // 4 Performance Stat Bars
    let (spd, acc, grip, drift) = active_car.stats();
    let stat_bar_w = col_w - scaler.s(24.0);
    let stat_base_x = col1_x + scaler.s(12.0);

    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(64.0), stat_bar_w, "SPEED", spd, Palette::NEON_CYAN);
    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(79.0), stat_bar_w, "ACCEL", acc, Palette::NEON_GOLD);
    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(94.0), stat_bar_w, "GRIP", grip, Palette::NEON_GREEN);
    render_grid_stat_bar(&scaler, fonts, stat_base_x, curr_y + scaler.s(109.0), stat_bar_w, "DRIFT", drift, Palette::NEON_MAGENTA);

    // 4 Engineering / Dynamic Specs Chips
    let (spec1, spec2, spec3, spec4) = active_car.specs();
    let spec_chip_w = (col_w - scaler.s(32.0)) * 0.5;
    let spec_chip_h = scaler.s(22.0);
    let chip_y1 = curr_y + scaler.s(134.0);
    let chip_y2 = curr_y + scaler.s(160.0);

    scaler.draw_glass_card(stat_base_x, chip_y1, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(spec1, stat_base_x + scaler.s(8.0), chip_y1 + scaler.s(15.0), scaler.font_s(10.0), Palette::NEON_CYAN);

    scaler.draw_glass_card(stat_base_x + spec_chip_w + scaler.s(8.0), chip_y1, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(spec2, stat_base_x + spec_chip_w + scaler.s(16.0), chip_y1 + scaler.s(15.0), scaler.font_s(10.0), Palette::WHITE);

    scaler.draw_glass_card(stat_base_x, chip_y2, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(spec3, stat_base_x + scaler.s(8.0), chip_y2 + scaler.s(15.0), scaler.font_s(10.0), Palette::NEON_GOLD);

    scaler.draw_glass_card(stat_base_x + spec_chip_w + scaler.s(8.0), chip_y2, spec_chip_w, spec_chip_h, Color::new(0.06, 0.08, 0.12, 0.8), Palette::UI_CARD_BORDER, 1.0);
    fonts.draw_ui_bold(spec4, stat_base_x + spec_chip_w + scaler.s(16.0), chip_y2 + scaler.s(15.0), scaler.font_s(10.0), Palette::NEON_GREEN);

    curr_y += car_card_h + scaler.s(8.0);

    // Card 4: Grid Configuration / Session Status Card
    let is_grid_active = is_left_focused && active_card_idx == 2;
    let grid_h = scaler.s(48.0);
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
    scaler.draw_glass_card(col1_x, curr_y, col_w, grid_h, grid_bg, grid_border, if is_grid_active { 2.4 } else { 1.2 });

    if game_mode.has_bots() {
        let grid_hdr = if is_grid_active {
            "GRID CONFIG [ACTIVE • ENTER / + / - to adjust]"
        } else {
            "GRID CONFIG: [Up/Down to select card]"
        };
        fonts.draw_ui_bold(
            grid_hdr,
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(16.0),
            scaler.font_s(10.5),
            if is_grid_active { Palette::NEON_CYAN } else { Palette::UI_TEXT_MUTED },
        );
        fonts.draw_ui_bold(
            &format!("{} Racers ({} AI Opponents) • Max {} Slots", num_drivers, num_drivers.saturating_sub(1), max_grid_size),
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(34.0),
            scaler.font_s(12.5),
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
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(16.0),
            scaler.font_s(10.5),
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
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(34.0),
            scaler.font_s(12.0),
            Palette::WHITE,
        );
    }

    curr_y += grid_h + scaler.s(10.0);

    // Card 5 (Index 3): High-Visibility Green "LAUNCH RACE" Action Button
    let launch_h = scaler.s(48.0);
    let is_launch_card = is_left_focused && active_card_idx == 3;
    let (mx, my) = std::panic::catch_unwind(macroquad::input::mouse_position).unwrap_or((-1000.0, -1000.0));
    let is_launch_hovered = mx >= col1_x && mx <= col1_x + col_w && my >= curr_y && my <= curr_y + launch_h;
    let is_launch_active = is_launch_card || is_launch_hovered;

    let launch_bg = if is_launch_active {
        Color::new(0.12, 0.68, 0.32, 0.98)
    } else {
        Color::new(0.08, 0.44, 0.22, 0.92)
    };
    let launch_border = if is_launch_active {
        Palette::NEON_GREEN
    } else {
        Color::new(0.20, 0.78, 0.40, 0.85)
    };

    draw_rectangle(col1_x, curr_y, col_w, launch_h, launch_bg);
    draw_rectangle_lines(
        col1_x,
        curr_y,
        col_w,
        launch_h,
        if is_launch_active { 2.8 * scaler.scale } else { 1.6 * scaler.scale },
        launch_border,
    );

    let launch_title = if is_launch_active {
        "▶ LAUNCH RACE  [ENTER / SPACE / CLICK]"
    } else {
        "▶ LAUNCH RACE"
    };
    fonts.draw_ui_bold_centered(
        launch_title,
        col1_x + col_w * 0.5,
        curr_y + scaler.s(21.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );
    fonts.draw_ui_regular_centered(
        "SPACE / Gamepad A",
        col1_x + col_w * 0.5,
        curr_y + scaler.s(37.0),
        scaler.font_s(13.5),
        Color::new(0.85, 1.0, 0.90, 0.95),
    );

    // =========================================================================
    // RIGHT PANEL: Starting Grid & Roster
    // =========================================================================
    let roster_base_title = match game_mode {
        GameMode::TimeTrial => "TIME TRIAL • ROSTER & SHADOW CAR",
        GameMode::FreeRide => "FREE RIDE • PRACTICE ROSTER",
        GameMode::StandardRace | GameMode::ExperimentalRace => "STARTING GRID & ROSTER",
    };
    let roster_header = if is_right_focused {
        format!("{} [FOCUSED • Up/Down to select slot • ENTER/D for Dossier]", roster_base_title)
    } else {
        format!("{} [Right arrow to focus roster]", roster_base_title)
    };
    fonts.draw_ui_bold(
        &roster_header,
        col2_x,
        panel_y + scaler.s(13.0),
        scaler.font_s(13.5),
        if is_right_focused { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED },
    );

    let roster_card_y = panel_y + scaler.s(22.0);
    let roster_card_h = (bottom_prompt_y - roster_card_y - scaler.s(8.0)).max(scaler.s(240.0));
    let roster_border = if is_right_focused { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER };
    scaler.draw_glass_card(col2_x, roster_card_y, col_w, roster_card_h, Palette::UI_CARD_BG, roster_border, if is_right_focused { 2.2 } else { 1.4 });

    let row_w = col_w - scaler.s(16.0);
    let row_x = col2_x + scaler.s(8.0);
    let mut row_y = roster_card_y + scaler.s(8.0);
    let row_h = scaler.s(46.0);
    let row_gap = scaler.s(5.0);

    match game_mode {
        GameMode::TimeTrial => {
            // Row 1: Player (P1)
            let is_row_sel = is_right_focused && active_roster_idx == 0;
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
            let is_ghost_sel = is_right_focused && active_roster_idx == 1;
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
            let is_row_sel = is_right_focused && active_roster_idx == 0;
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
                "• Press [SPACE / ENTER] to launch practice session on the starting grid",
            ];
            let mut tip_y = row_y + scaler.s(42.0);
            for tip in &tips {
                fonts.draw_ui_regular(tip, row_x + scaler.s(12.0), tip_y, scaler.font_s(11.0), Palette::WHITE);
                tip_y += scaler.s(24.0);
            }
        }
        GameMode::StandardRace | GameMode::ExperimentalRace => {
            for (i, participant) in grid_participants.iter().enumerate() {
                let slot = i + 1;
                let is_row_sel = is_right_focused && i == active_roster_idx;
                let desc = match (participant.best_lap, participant.best_circuit_time) {
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
                };

                let display_name = if participant.is_player {
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
    let prompt = if gamepad_connected {
        match focused_panel {
            StartingGridFocus::LeftSetup => match active_card_idx {
                0 => "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [A/X] Cycle Mode  |  [START] Launch  |  [B] Menu",
                1 => "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [A/X] Change Car  |  [START] Launch  |  [B] Menu",
                2 => "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [A/X] Adjust Bots  |  [START] Launch  |  [B] Menu",
                _ => "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Card  |  [A / START] LAUNCH RACE  |  [B] Menu",
            },
            StartingGridFocus::RightRoster => {
                "[D-Pad L/R] Switch Panel  |  [Up/Down] Select Driver  |  [A/Y] View Dossier  |  [START] Launch  |  [B] Menu"
            }
        }
    } else {
        match focused_panel {
            StartingGridFocus::LeftSetup => match active_card_idx {
                0 => "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ENTER/SPACE/TAB] Cycle Mode  |  [SPACE] Launch  |  [ESC] Menu",
                1 => "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ENTER / < / >] Change Vehicle  |  [SPACE] Launch  |  [ESC] Menu",
                2 => "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ENTER / + / -] Adjust Bots  |  [SPACE] Launch  |  [ESC] Menu",
                _ => "[Left/Right] Switch Panel  |  [Up/Down] Select Card  |  [ENTER / SPACE / CLICK] LAUNCH RACE  |  [ESC] Menu",
            },
            StartingGridFocus::RightRoster => {
                "[Left/Right] Switch Panel  |  [Up/Down] Select Driver  |  [ENTER / D] View Dossier  |  [SPACE] Launch  |  [ESC] Menu"
            }
        }
    };

    fonts.draw_ui_bold_centered(
        prompt,
        sw * 0.5,
        bottom_prompt_y + scaler.s(6.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );
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

