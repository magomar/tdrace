use macroquad::prelude::*;
use crate::render::color::Palette;
use crate::ui::scaler::UiScaler;
use cabinet::ui::font::Fonts;
use cabinet::ui::{ColumnAlign, DataColumn, DataRow, DataTable, KpiTile, LayoutRect, ScreenFooter, SplitPane};
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

/// A single lap's split times, formatted for the lap-splits `DataTable`.
#[derive(Clone)]
struct LapSplit {
    lap: String,
    time: String,
    s1: String,
    s2: String,
    s3: String,
    status: String,
}

/// Builds display-ready lap split rows from the player's lap telemetry.
fn build_lap_splits(stats: &PlayerRaceTelemetry, max_laps: usize) -> Vec<LapSplit> {
    let best_time = stats
        .best_lap_idx
        .and_then(|idx| stats.laps.get(idx))
        .map(|l| l.lap_time)
        .or_else(|| stats.laps.iter().map(|l| l.lap_time).min_by(|a, b| a.partial_cmp(b).unwrap()));

    stats
        .laps
        .iter()
        .take(max_laps)
        .enumerate()
        .map(|(i, lap)| {
            let is_best = Some(i) == stats.best_lap_idx;
            let s1 = lap.sector_times.first().copied().unwrap_or(0.0);
            let s2 = lap.sector_times.get(1).copied().unwrap_or(0.0);
            let s3 = lap.sector_times.get(2).copied().unwrap_or(0.0);
            let s1_lbl = if s1 > 0.05 { format!("{:.2}s", s1) } else { "--".to_string() };
            let s2_lbl = if s2 > 0.05 { format!("{:.2}s", s2) } else { "--".to_string() };
            let s3_lbl = if s3 > 0.05 { format!("{:.2}s", s3) } else { "--".to_string() };
            let lap_lbl = if is_best {
                format!("★L{}", lap.lap_number)
            } else {
                format!("L{}", lap.lap_number)
            };
            let status_lbl = if is_best {
                "PB (BEST)".to_string()
            } else if let Some(bt) = best_time {
                let diff = lap.lap_time - bt;
                format!("+{:.2}s", diff.max(0.0))
            } else {
                "-".to_string()
            };
            LapSplit {
                lap: lap_lbl,
                time: format_lap_time(lap.lap_time),
                s1: s1_lbl,
                s2: s2_lbl,
                s3: s3_lbl,
                status: status_lbl,
            }
        })
        .collect()
}

/// Renders the full-screen Detailed Race Telemetry & Acrobatic Stunt Statistics screen.
pub fn render_race_stats_screen(
    fonts: &Fonts,
    track_name: &str,
    car_name: &str,
    stats: &PlayerRaceTelemetry,
    total_time: f32,
    prev_is_hof: bool,
    stunt_scoring_enabled: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep semi-transparent motorsport backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.96));

    let box_w = (sw * 0.92).clamp(scaler.s(680.0), scaler.s(1040.0));
    let box_h = (sh * 0.84).clamp(scaler.s(520.0), scaler.s(760.0));
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

    // Split Columns Setup (platform SplitPane)
    let pad = scaler.s(18.0);
    let content_w = box_w - pad * 2.0;
    let content_top_y = y + scaler.s(66.0);
    let content_h = box_h - scaler.s(96.0);
    let split_pane = SplitPane::new(
        LayoutRect::new(x + pad, content_top_y, content_w, content_h),
        0.5,
        scaler.s(14.0),
    );
    let left_x = split_pane.left_rect().x;
    let right_x = split_pane.right_rect().x;
    let col_w = split_pane.left_rect().w;

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

    // Lap splits DataTable (platform DataTable<LapSplit>)
    let tbl_x = left_x + scaler.s(12.0);
    let tbl_w = col_w - scaler.s(24.0);

    // Left Column Summary Card (anchored at bottom)
    let sum_box_h = scaler.s(76.0);
    let sum_box_y = content_top_y + content_h - sum_box_h - scaler.s(10.0);

    if stats.laps.is_empty() {
        fonts.draw_ui_regular(
            "No full laps completed during session",
            tbl_x + scaler.s(12.0),
            content_top_y + scaler.s(56.0),
            scaler.font_s(12.5),
            Palette::UI_TEXT_MUTED,
        );
    } else {
        let table_bounds = LayoutRect::new(
            tbl_x,
            content_top_y + scaler.s(32.0),
            tbl_w,
            (sum_box_y - scaler.s(8.0)) - (content_top_y + scaler.s(32.0)),
        );
        let mut table = DataTable::<LapSplit>::new(0.0, 0.0, table_bounds.w, scaler.s(24.0), scaler.s(22.0));
        table.add_column(DataColumn::new("lap", "LAP", 10.0, ColumnAlign::Left, |r| r.lap.clone()));
        table.add_column(DataColumn::new("time", "TIME", 22.0, ColumnAlign::Left, |r| r.time.clone()));
        table.add_column(DataColumn::new("s1", "S1", 18.0, ColumnAlign::Right, |r| r.s1.clone()));
        table.add_column(DataColumn::new("s2", "S2", 18.0, ColumnAlign::Right, |r| r.s2.clone()));
        table.add_column(DataColumn::new("s3", "S3", 18.0, ColumnAlign::Right, |r| r.s3.clone()));
        table.add_column(DataColumn::new("status", "STATUS / GAP", 14.0, ColumnAlign::Right, |r| r.status.clone()));
        let laps = build_lap_splits(stats, 10);
        table.set_rows(laps.iter().enumerate().map(|(i, s)| DataRow::new(i.to_string(), s.clone())).collect());
        table.draw(&scaler, fonts, table_bounds);
    }

    // Left Column Summary Card at bottom
    draw_rectangle(tbl_x, sum_box_y, tbl_w, sum_box_h, Color::new(0.09, 0.12, 0.18, 0.85));
    draw_rectangle_lines(tbl_x, sum_box_y, tbl_w, sum_box_h, 1.0, Color::new(0.25, 0.35, 0.50, 0.6));

    let best_time = stats
        .best_lap_idx
        .and_then(|idx| stats.laps.get(idx))
        .map(|l| l.lap_time)
        .or_else(|| stats.laps.iter().map(|l| l.lap_time).min_by(|a, b| a.partial_cmp(b).unwrap()));

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
    let (rank_title, rank_color) = if stunt_scoring_enabled {
        determine_stunt_rank(stunt.total_stunt_score)
    } else {
        ("CIRCUIT TIMING MODE (STUNT SCORING INACTIVE)", Color::new(0.60, 0.70, 0.80, 0.85))
    };
    let banner_h = scaler.s(38.0);
    let banner_y = content_top_y + scaler.s(30.0);

    draw_rectangle(right_inner_x, banner_y, right_inner_w, banner_h, Color::new(0.10, 0.16, 0.24, 0.90));
    draw_rectangle_lines(right_inner_x, banner_y, right_inner_w, banner_h, 1.5, rank_color);

    fonts.draw_ui_bold_centered(
        rank_title,
        right_inner_x + right_inner_w * 0.5,
        banner_y + scaler.s(24.0),
        if stunt_scoring_enabled { scaler.font_s(14.5) } else { scaler.font_s(12.5) },
        rank_color,
    );

    // Total Stunt Score KPI tile (platform KpiTile)
    let total_score_str = if stunt_scoring_enabled {
        format!("{} PTS", stunt.total_stunt_score)
    } else {
        "OFF".to_string()
    };
    let total_tile = KpiTile::new(
        if stunt_scoring_enabled { "TOTAL ACROBATIC SCORE" } else { "TOTAL ACROBATIC SCORE (INACTIVE)" },
        total_score_str,
        if stunt_scoring_enabled { Palette::NEON_GOLD } else { Color::new(0.60, 0.70, 0.80, 0.85) },
    )
    .with_subtext(rank_title);
    let total_tile_h = scaler.s(88.0);
    let total_tile_y = banner_y + banner_h + scaler.s(10.0);
    total_tile.draw(&scaler, fonts, LayoutRect::new(right_inner_x, total_tile_y, right_inner_w, total_tile_h));

    // Stunt metric KPI tiles (platform KpiTile)
    let tile_gap = scaler.s(10.0);
    let tile_w = (right_inner_w - tile_gap) * 0.5;
    let tile_h = scaler.s(72.0);
    let tiles_y = total_tile_y + total_tile_h + scaler.s(12.0);
    let tiles = [
        KpiTile::new("DRIFT POINTS", format!("{} PTS", stunt.total_drift_points), Palette::NEON_GREEN)
            .with_subtext(format!("{} DRIFTS", stunt.drift_count)),
        KpiTile::new("AIR STUNT BONUS", format!("{} PTS", stunt.jump_points), Palette::NEON_MAGENTA)
            .with_subtext(format!("{:.2}s AIR", stunt.total_air_time)),
        KpiTile::new("PEAK COMBO", format!("{}x STREAK", stunt.max_combo), Palette::NEON_GOLD),
        KpiTile::new("LONGEST JUMP", format!("{:.2}s", stunt.longest_jump_time), Palette::NEON_CYAN)
            .with_subtext(format!("{} JUMPS", stunt.jump_count)),
    ];
    for (i, tile) in tiles.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let tx = right_inner_x + col as f32 * (tile_w + tile_gap);
        let ty = tiles_y + row as f32 * (tile_h + tile_gap);
        tile.draw(&scaler, fonts, LayoutRect::new(tx, ty, tile_w, tile_h));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BOTTOM ACTION BAR (platform ScreenFooter)
    // ─────────────────────────────────────────────────────────────────────────
    let back_target = if prev_is_hof { "Hall of Fame" } else { "Race Results" };
    let footer_h = scaler.s(56.0);
    let mut footer = ScreenFooter::new(0.0, sh - footer_h, sw, footer_h);
    footer.add_prompt("TAB / ESC", &format!("Back to {}", back_target));
    footer.add_prompt("R", "Restart Race");
    footer.add_prompt("SPACE / ENTER", "Proceed");
    footer.render_frame();
    let prompt_w = sw / footer.prompts.len().max(1) as f32;
    for (i, prompt) in footer.prompts.iter().enumerate() {
        let size = scaler.font_s(13.0);
        let text = fonts.fit_ui_bold(&format!("[{}] {}", prompt.badge, prompt.label), size, prompt_w - scaler.s(8.0));
        fonts.draw_ui_bold_centered(&text, (i as f32 + 0.5) * prompt_w, sh - footer_h + footer_h * 0.65, size, Palette::WHITE);
    }
}
