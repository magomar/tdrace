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
    let mode_h = scaler.s(72.0);
    scaler.draw_glass_card(col1_x, curr_y, col_w, mode_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 1.4);

    fonts.draw_ui_bold(
        "GAME MODE: [Tab to cycle] or Gamepad [X]",
        col1_x + scaler.s(12.0),
        curr_y + scaler.s(16.0),
        scaler.font_s(11.0),
        Palette::NEON_GOLD,
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
    let car_card_h = scaler.s(214.0);
    let car_border_col = if game_mode.allows_car_change() { Palette::NEON_GREEN } else { Palette::UI_CARD_BORDER };
    scaler.draw_glass_card(col1_x, curr_y, col_w, car_card_h, Palette::UI_CARD_BG, car_border_col, 1.4);

    let car_header_title = if game_mode.allows_car_change() {
        "CAR SELECTION: [ < / > ] [Left/Right]"
    } else {
        "CAR SPEC: ENFORCED PREDEFINED [Locked]"
    };
    let car_header_col = if game_mode.allows_car_change() { Palette::NEON_GREEN } else { Palette::NEON_CYAN };
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
    let grid_h = scaler.s(48.0);
    scaler.draw_glass_card(col1_x, curr_y, col_w, grid_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.2);

    if game_mode.has_bots() {
        fonts.draw_ui_bold(
            "GRID CONFIG: [Up/Down to adjust bots]",
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(16.0),
            scaler.font_s(10.5),
            Palette::NEON_CYAN,
        );
        fonts.draw_ui_bold(
            &format!("{} Racers ({} AI Opponents) • Max {} Slots", num_drivers, num_drivers.saturating_sub(1), max_grid_size),
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(34.0),
            scaler.font_s(12.5),
            Palette::WHITE,
        );
    } else {
        fonts.draw_ui_bold(
            "SESSION STATUS: SOLO TRACK TIME",
            col1_x + scaler.s(12.0),
            curr_y + scaler.s(16.0),
            scaler.font_s(10.5),
            Palette::NEON_CYAN,
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

    // =========================================================================
    // RIGHT PANEL: Starting Grid & Roster
    // =========================================================================
    let roster_header = match game_mode {
        GameMode::TimeTrial => "TIME TRIAL • ROSTER & SHADOW CAR",
        GameMode::FreeRide => "FREE RIDE • PRACTICE ROSTER",
        GameMode::StandardRace | GameMode::ExperimentalRace => "STARTING GRID & ROSTER [D for Dossiers]",
    };
    fonts.draw_ui_bold(
        roster_header,
        col2_x,
        panel_y + scaler.s(13.0),
        scaler.font_s(14.5),
        Palette::NEON_GOLD,
    );

    let roster_card_y = panel_y + scaler.s(22.0);
    let roster_card_h = (bottom_prompt_y - roster_card_y - scaler.s(8.0)).max(scaler.s(240.0));
    scaler.draw_glass_card(col2_x, roster_card_y, col_w, roster_card_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 1.8);

    let row_w = col_w - scaler.s(16.0);
    let row_x = col2_x + scaler.s(8.0);
    let mut row_y = roster_card_y + scaler.s(8.0);
    let row_h = scaler.s(46.0);
    let row_gap = scaler.s(5.0);

    match game_mode {
        GameMode::TimeTrial => {
            // Row 1: Player (P1)
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
            );
            row_y += row_h + row_gap;

            // Row 2: Shadow / Ghost Car
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
            );
            row_y += row_h + scaler.s(14.0);

            // Explanatory info box for Time Trial mode
            let info_h = scaler.s(160.0);
            scaler.draw_glass_card(row_x, row_y, row_w, info_h, Color::new(0.05, 0.08, 0.13, 0.8), Palette::UI_CARD_BORDER, 1.0);
            fonts.draw_ui_bold("TIME TRIAL SHADOW CAR TELEMETRY", row_x + scaler.s(12.0), row_y + scaler.s(18.0), scaler.font_s(12.5), Palette::NEON_CYAN);
            let tips = [
                "• The shadow car renders semi-transparently using your all-time best lap",
                "• Accurately compare cornering lines, braking markers, and exit speeds",
                "• Setting a new fastest lap automatically updates the active shadow benchmark",
                "• Press [SPACE / ENTER] to initiate the 3-2-1 launch countdown",
            ];
            let mut tip_y = row_y + scaler.s(42.0);
            for tip in &tips {
                fonts.draw_ui_regular(tip, row_x + scaler.s(12.0), tip_y, scaler.font_s(11.0), Palette::WHITE);
                tip_y += scaler.s(24.0);
            }
        }
        GameMode::FreeRide => {
            // Row 1: Player (P1)
            let p_line = "Open Practice Session  •  Unlimited Free Laps".to_string();
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
                );
                row_y += row_h + row_gap;
            }
        }
    }

    // =========================================================================
    // FOOTER PROMPTS
    // =========================================================================
    let prompt = if gamepad_connected {
        "[A/START] Launch Race  |  [X] Game Mode  |  [D-Pad L/R] Select Car  |  [Up/Down] Bots  |  [Y] Dossier  |  [B] Menu"
    } else {
        "[SPACE/ENTER] Launch Race  |  [TAB] Game Mode  |  [ < / > ] Select Car  |  [Up/Down] Bots  |  [D] Dossiers  |  [ESC] Menu"
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
    val: f32,
    color: Color,
) {
    fonts.draw_ui_bold(label, x, y + scaler.s(7.0), scaler.font_s(9.0), Palette::UI_TEXT_MUTED);
    let bar_x = x + scaler.s(40.0);
    let bar_w = (w - scaler.s(76.0)).max(30.0);
    let bar_h = scaler.s(6.0);

    draw_rectangle(bar_x, y, bar_w, bar_h, Color::new(0.1, 0.12, 0.16, 0.9));
    draw_rectangle(bar_x, y, bar_w * val.clamp(0.0, 1.0), bar_h, color);

    let pct_str = format!("{:.0}%", (val * 100.0).clamp(0.0, 100.0));
    fonts.draw_ui_bold(&pct_str, bar_x + bar_w + scaler.s(6.0), y + scaler.s(7.0), scaler.font_s(9.0), color);
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
) {
    let bg_color = Color::new(0.06, 0.14, 0.22, 0.90);
    let border_color = Palette::NEON_CYAN;

    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, 1.4, border_color);

    // Ghost Badge
    let badge_w = scaler.s(48.0);
    let badge_h = h - scaler.s(10.0);
    let badge_x = x + scaler.s(6.0);
    let badge_y = y + scaler.s(5.0);

    draw_rectangle(badge_x, badge_y, badge_w, badge_h, Palette::NEON_CYAN);
    fonts.draw_ui_bold_centered(
        "GHOST",
        badge_x + badge_w * 0.5,
        badge_y + badge_h * 0.5 + scaler.s(4.5),
        scaler.font_s(11.0),
        Palette::BLACK,
    );

    let text_start_x = badge_x + badge_w + scaler.s(10.0);

    // Car column on right
    let car_col_w = scaler.s(170.0);
    let car_x = x + w - car_col_w;

    let sep_x = car_x - scaler.s(8.0);
    draw_rectangle(sep_x, y + scaler.s(6.0), 1.0, h - scaler.s(12.0), Color::new(0.25, 0.35, 0.45, 0.40));

    fonts.draw_ui_bold(car_name, car_x, y + scaler.s(21.0), scaler.font_s(12.5), Palette::WHITE);
    fonts.draw_ui_regular("Shadow Car", car_x, y + scaler.s(36.0), scaler.font_s(10.0), Palette::NEON_CYAN);

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

    draw_rectangle(car_x + swatch_w + scaler.s(3.0), s_y, swatch_w, swatch_h, scheme.secondary);
    draw_rectangle_lines(car_x + swatch_w + scaler.s(3.0), s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    draw_rectangle(car_x + (swatch_w + scaler.s(3.0)) * 2.0, s_y, swatch_w, swatch_h, scheme.helmet);
    draw_rectangle_lines(car_x + (swatch_w + scaler.s(3.0)) * 2.0, s_y, swatch_w, swatch_h, 1.0, Palette::WHITE);

    fonts.draw_ui_regular("Team Livery", car_x + scaler.s(58.0), s_y + scaler.s(8.5), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);

    // Middle Column: Driver Name & Profile / Style
    let driver_label = if is_player {
        format!("{}  (\"{}\")", name, alias)
    } else {
        format!("{}  (\"{}\")", name, alias)
    };

    let name_color = if is_player { Palette::NEON_CYAN } else { Palette::WHITE };
    let stats_color = if is_player { Palette::NEON_GOLD } else { Color::new(0.60, 0.85, 0.95, 1.0) };

    // Line 1: Driver Name & Alias
    fonts.draw_ui_bold(&driver_label, text_start_x, y + scaler.s(21.0), scaler.font_s(13.5), name_color);

    // Line 2: Profile / Style stats
    fonts.draw_ui_regular(profile_line, text_start_x, y + scaler.s(37.0), scaler.font_s(10.5), stats_color);
}

