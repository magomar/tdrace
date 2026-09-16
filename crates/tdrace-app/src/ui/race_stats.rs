use macroquad::prelude::*;
use crate::render::color::Palette;
use crate::ui::scaler::UiScaler;
use cabinet::ui::font::Fonts;
use crate::game::PlayerRaceTelemetry;
use crate::ui::hud::format_lap_time;

/// Determines the stunt rank title and accent color based on total acrobatic stunt score.
fn determine_stunt_rank(total_score: u32) -> (&'static str, Color) {
    if total_score >= 5000 {
        ("LEGENDARY ACROBAT (S-RANK)", Palette::NEON_GOLD)
    } else if total_score >= 2500 {
        ("DRIFT & AIR DAREDEVIL (A-RANK)", Palette::NEON_MAGENTA)
    } else if total_score >= 1000 {
        ("ACROBATIC SPECIALIST (B-RANK)", Palette::NEON_CYAN)
    } else if total_score >= 300 {
        ("STUNT ENTHUSIAST (C-RANK)", Palette::NEON_GREEN)
    } else {
        ("PRECISION RACER", Color::new(0.70, 0.78, 0.88, 1.0))
    }
}

/// Renders the full-screen Detailed Race Telemetry & Acrobatic Stunt Statistics screen.
pub fn render_race_stats_screen(
    fonts: &Fonts,
    track_name: &str,
    car_name: &str,
    stats: &PlayerRaceTelemetry,
    total_time: f32,
    prev_is_hof: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep semi-transparent motorsport backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.96));

    let box_w = (sw * 0.92).clamp(scaler.s(680.0), scaler.s(1040.0));
    let box_h = (sh * 0.90).clamp(scaler.s(520.0), scaler.s(760.0));
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::NEON_GOLD, 2.5);

    // Main Header Title
    fonts.draw_display_centered_with_shadow(
        "DETAILED RACE TELEMETRY & STUNT STATISTICS",
        sw * 0.5,
        y + scaler.s(32.0),
        scaler.font_s(23.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let subtitle = format!(
        "Circuit: {}  |  Vehicle: {}  |  Total Race Time: {}",
        track_name,
        car_name,
        format_lap_time(total_time)
    );
    fonts.draw_ui_regular_centered(
        &subtitle,
        sw * 0.5,
        y + scaler.s(52.0),
        scaler.font_s(13.5),
        Palette::UI_TEXT_MUTED,
    );

    // Split Columns Setup
    let pad = scaler.s(18.0);
    let col_gap = scaler.s(14.0);
    let content_w = box_w - pad * 2.0;
    let col_w = (content_w - col_gap) * 0.5;
    let left_x = x + pad;
    let right_x = left_x + col_w + col_gap;
    let content_top_y = y + scaler.s(66.0);
    let content_h = box_h - scaler.s(96.0);

    // ─────────────────────────────────────────────────────────────────────────
    // LEFT COLUMN: LAP TIMES & SECTOR SPLITS
    // ─────────────────────────────────────────────────────────────────────────
    scaler.draw_glass_card(
        left_x,
        content_top_y,
        col_w,
        content_h,
        Color::new(0.06, 0.08, 0.12, 0.75),
        Color::new(0.20, 0.28, 0.40, 0.45),
        1.0,
    );

    fonts.draw_ui_bold(
        "LAP TIMES & SECTOR SPLITS",
        left_x + scaler.s(16.0),
        content_top_y + scaler.s(20.0),
        scaler.font_s(15.0),
        Palette::NEON_CYAN,
    );

    // Table Header
    let tbl_x = left_x + scaler.s(12.0);
    let tbl_w = col_w - scaler.s(24.0);
    let mut row_y = content_top_y + scaler.s(32.0);
    let hdr_h = scaler.s(22.0);

    draw_rectangle(tbl_x, row_y, tbl_w, hdr_h, Color::new(0.12, 0.16, 0.24, 0.90));
    fonts.draw_ui_bold("LAP", tbl_x + scaler.s(8.0), row_y + scaler.s(15.0), scaler.font_s(11.5), Palette::WHITE);
    fonts.draw_ui_bold("TIME", tbl_x + scaler.s(45.0), row_y + scaler.s(15.0), scaler.font_s(11.5), Palette::WHITE);
    fonts.draw_ui_bold("S1", tbl_x + scaler.s(125.0), row_y + scaler.s(15.0), scaler.font_s(11.5), Palette::WHITE);
    fonts.draw_ui_bold("S2", tbl_x + scaler.s(180.0), row_y + scaler.s(15.0), scaler.font_s(11.5), Palette::WHITE);
    fonts.draw_ui_bold("S3", tbl_x + scaler.s(235.0), row_y + scaler.s(15.0), scaler.font_s(11.5), Palette::WHITE);
    fonts.draw_ui_bold("STATUS / GAP", tbl_x + tbl_w - scaler.s(80.0), row_y + scaler.s(15.0), scaler.font_s(11.5), Palette::WHITE);

    row_y += hdr_h + scaler.s(4.0);

    let best_time = stats
        .best_lap_idx
        .and_then(|idx| stats.laps.get(idx))
        .map(|l| l.lap_time)
        .or_else(|| stats.laps.iter().map(|l| l.lap_time).min_by(|a, b| a.partial_cmp(b).unwrap()));

    let row_h = scaler.s(22.0);
    let max_display_laps = 10;
    for (i, lap) in stats.laps.iter().take(max_display_laps).enumerate() {
        let is_best = Some(i) == stats.best_lap_idx;
        let (row_bg, text_col) = if is_best {
            (Color::new(0.18, 0.32, 0.22, 0.85), Palette::NEON_GOLD)
        } else if i % 2 == 1 {
            (Color::new(0.08, 0.11, 0.16, 0.60), Color::new(0.85, 0.90, 0.96, 1.0))
        } else {
            (Color::new(0.06, 0.08, 0.12, 0.60), Color::new(0.85, 0.90, 0.96, 1.0))
        };

        draw_rectangle(tbl_x, row_y, tbl_w, row_h, row_bg);
        if is_best {
            draw_rectangle_lines(tbl_x, row_y, tbl_w, row_h, 1.0, Palette::NEON_GOLD);
        }

        // Lap number
        let lap_lbl = format!("L{}", lap.lap_number);
        fonts.draw_ui_bold(&lap_lbl, tbl_x + scaler.s(8.0), row_y + scaler.s(15.0), scaler.font_s(11.5), text_col);

        // Lap time
        let time_lbl = format_lap_time(lap.lap_time);
        fonts.draw_ui_bold(&time_lbl, tbl_x + scaler.s(45.0), row_y + scaler.s(15.0), scaler.font_s(12.0), text_col);

        // Sector splits
        let s1 = lap.sector_times.first().copied().unwrap_or(0.0);
        let s2 = lap.sector_times.get(1).copied().unwrap_or(0.0);
        let s3 = lap.sector_times.get(2).copied().unwrap_or(0.0);

        let s1_lbl = if s1 > 0.05 { format!("{:.2}s", s1) } else { "--".to_string() };
        let s2_lbl = if s2 > 0.05 { format!("{:.2}s", s2) } else { "--".to_string() };
        let s3_lbl = if s3 > 0.05 { format!("{:.2}s", s3) } else { "--".to_string() };

        fonts.draw_ui_regular(&s1_lbl, tbl_x + scaler.s(125.0), row_y + scaler.s(15.0), scaler.font_s(11.0), text_col);
        fonts.draw_ui_regular(&s2_lbl, tbl_x + scaler.s(180.0), row_y + scaler.s(15.0), scaler.font_s(11.0), text_col);
        fonts.draw_ui_regular(&s3_lbl, tbl_x + scaler.s(235.0), row_y + scaler.s(15.0), scaler.font_s(11.0), text_col);

        // Gap / Status
        let status_lbl = if is_best {
            "PB (BEST)".to_string()
        } else if let Some(bt) = best_time {
            let diff = lap.lap_time - bt;
            format!("+{:.2}s", diff.max(0.0))
        } else {
            "-".to_string()
        };
        let status_col = if is_best { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED };
        fonts.draw_ui_bold(&status_lbl, tbl_x + tbl_w - scaler.s(80.0), row_y + scaler.s(15.0), scaler.font_s(11.0), status_col);

        row_y += row_h + scaler.s(3.0);
    }

    if stats.laps.is_empty() {
        fonts.draw_ui_regular(
            "No full laps completed during session",
            tbl_x + scaler.s(12.0),
            row_y + scaler.s(24.0),
            scaler.font_s(12.5),
            Palette::UI_TEXT_MUTED,
        );
    }

    // Left Column Summary Card at bottom
    let sum_box_h = scaler.s(76.0);
    let sum_box_y = content_top_y + content_h - sum_box_h - scaler.s(10.0);
    draw_rectangle(tbl_x, sum_box_y, tbl_w, sum_box_h, Color::new(0.09, 0.12, 0.18, 0.85));
    draw_rectangle_lines(tbl_x, sum_box_y, tbl_w, sum_box_h, 1.0, Color::new(0.25, 0.35, 0.50, 0.6));

    let best_lap_str = best_time.map(format_lap_time).unwrap_or_else(|| "--:--.--".to_string());
    let avg_lap_str = if !stats.laps.is_empty() {
        let sum: f32 = stats.laps.iter().map(|l| l.lap_time).sum();
        format_lap_time(sum / stats.laps.len() as f32)
    } else {
        "--:--.--".to_string()
    };
    let top_speed_kmh = stats.top_speed_mps * 3.6;

    fonts.draw_ui_bold(
        &format!("Best Flying Lap: {}", best_lap_str),
        tbl_x + scaler.s(12.0),
        sum_box_y + scaler.s(22.0),
        scaler.font_s(12.5),
        Palette::NEON_GOLD,
    );
    fonts.draw_ui_bold(
        &format!("Average Lap: {}", avg_lap_str),
        tbl_x + scaler.s(12.0),
        sum_box_y + scaler.s(44.0),
        scaler.font_s(12.5),
        Color::new(0.85, 0.90, 0.96, 1.0),
    );
    fonts.draw_ui_bold(
        &format!("Top Speed: {:.1} km/h ({:.1} m/s)", top_speed_kmh, stats.top_speed_mps),
        tbl_x + scaler.s(12.0),
        sum_box_y + scaler.s(66.0),
        scaler.font_s(12.5),
        Palette::NEON_CYAN,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // RIGHT COLUMN: ACROBATIC & STUNT PERFORMANCE
    // ─────────────────────────────────────────────────────────────────────────
    scaler.draw_glass_card(
        right_x,
        content_top_y,
        col_w,
        content_h,
        Color::new(0.06, 0.08, 0.12, 0.75),
        Color::new(0.20, 0.28, 0.40, 0.45),
        1.0,
    );

    fonts.draw_ui_bold(
        "ACROBATIC & STUNT METRICS",
        right_x + scaler.s(16.0),
        content_top_y + scaler.s(20.0),
        scaler.font_s(15.0),
        Palette::NEON_GOLD,
    );

    let right_inner_x = right_x + scaler.s(14.0);
    let right_inner_w = col_w - scaler.s(28.0);
    let stunt = &stats.stunt_stats;

    // Stunt Rank Banner
    let (rank_title, rank_color) = determine_stunt_rank(stunt.total_stunt_score);
    let banner_h = scaler.s(38.0);
    let banner_y = content_top_y + scaler.s(30.0);

    draw_rectangle(right_inner_x, banner_y, right_inner_w, banner_h, Color::new(0.10, 0.16, 0.24, 0.90));
    draw_rectangle_lines(right_inner_x, banner_y, right_inner_w, banner_h, 1.5, rank_color);

    fonts.draw_ui_bold_centered(
        rank_title,
        right_inner_x + right_inner_w * 0.5,
        banner_y + scaler.s(24.0),
        scaler.font_s(14.5),
        rank_color,
    );

    // Total Stunt Score Card
    let score_card_y = banner_y + banner_h + scaler.s(10.0);
    let score_card_h = scaler.s(48.0);
    draw_rectangle(right_inner_x, score_card_y, right_inner_w, score_card_h, Color::new(0.12, 0.18, 0.14, 0.85));
    draw_rectangle_lines(right_inner_x, score_card_y, right_inner_w, score_card_h, 1.0, Palette::NEON_GOLD);

    fonts.draw_ui_regular(
        "TOTAL ACROBATIC SCORE",
        right_inner_x + scaler.s(12.0),
        score_card_y + scaler.s(20.0),
        scaler.font_s(11.5),
        Palette::UI_TEXT_MUTED,
    );
    let score_str = format!("{} PTS", stunt.total_stunt_score);
    fonts.draw_display(
        &score_str,
        right_inner_x + scaler.s(12.0),
        score_card_y + scaler.s(41.0),
        scaler.font_s(20.0),
        Palette::NEON_GOLD,
    );

    // Stunt Details Rows
    let mut detail_y = score_card_y + score_card_h + scaler.s(16.0);
    let section_h = scaler.s(20.0);

    // Section 1: Drifting
    fonts.draw_ui_bold(
        "DRIFT PERFORMANCE",
        right_inner_x,
        detail_y + scaler.s(14.0),
        scaler.font_s(13.0),
        Palette::NEON_CYAN,
    );
    detail_y += section_h;

    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Total Drift Points", &format!("{} PTS", stunt.total_drift_points), Palette::NEON_GREEN);
    detail_y += scaler.s(22.0);
    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Drift Count", &format!("{}", stunt.drift_count), Palette::WHITE);
    detail_y += scaler.s(22.0);
    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Peak Single Drift", &format!("{:.0} PTS", stunt.max_single_drift_score), Palette::WHITE);
    detail_y += scaler.s(28.0);

    // Section 2: Airborne / Jumps
    fonts.draw_ui_bold(
        "AIRBORNE JUMP PERFORMANCE",
        right_inner_x,
        detail_y + scaler.s(14.0),
        scaler.font_s(13.0),
        Palette::NEON_MAGENTA,
    );
    detail_y += section_h;

    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Ramp Jumps Completed", &format!("{}", stunt.jump_count), Palette::WHITE);
    detail_y += scaler.s(22.0);
    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Cumulative Air Time", &format!("{:.2}s", stunt.total_air_time), Palette::NEON_CYAN);
    detail_y += scaler.s(22.0);
    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Longest Single Jump", &format!("{:.2}s", stunt.longest_jump_time), Palette::WHITE);
    detail_y += scaler.s(22.0);
    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Air Stunt Bonus", &format!("{} PTS", stunt.jump_points), Palette::NEON_GREEN);
    detail_y += scaler.s(28.0);

    // Section 3: Combo Mastery
    fonts.draw_ui_bold(
        "COMBO & STREAK MULTIPLIER",
        right_inner_x,
        detail_y + scaler.s(14.0),
        scaler.font_s(13.0),
        Palette::NEON_GOLD,
    );
    detail_y += section_h;

    render_stat_row(fonts, &scaler, right_inner_x, right_inner_w, detail_y, "Peak Combo Streak", &format!("{}x STREAK", stunt.max_combo), Palette::NEON_GOLD);

    // ─────────────────────────────────────────────────────────────────────────
    // BOTTOM ACTION PROMPT
    // ─────────────────────────────────────────────────────────────────────────
    let back_target = if prev_is_hof { "Hall of Fame" } else { "Race Results" };
    let prompt = format!(
        "Press [TAB / ESC] Back to {}  |  [R] Restart Race  |  [SPACE / ENTER] Proceed",
        back_target
    );
    fonts.draw_ui_bold_centered(
        &prompt,
        sw * 0.5,
        y + box_h - scaler.s(16.0),
        scaler.font_s(14.0),
        Palette::WHITE,
    );
}

/// Helper to render a clean key-value stat row with alternating background.
fn render_stat_row(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    val: &str,
    val_col: Color,
) {
    let row_h = scaler.s(20.0);
    draw_rectangle(x, y, w, row_h, Color::new(0.08, 0.11, 0.16, 0.45));
    fonts.draw_ui_regular(
        label,
        x + scaler.s(8.0),
        y + scaler.s(14.0),
        scaler.font_s(12.0),
        Palette::UI_TEXT_MUTED,
    );
    let val_w = fonts.measure_ui_bold(val, scaler.font_s(12.0)).width;
    fonts.draw_ui_bold(
        val,
        x + w - scaler.s(12.0) - val_w,
        y + scaler.s(14.0),
        scaler.font_s(12.0),
        val_col,
    );
}
