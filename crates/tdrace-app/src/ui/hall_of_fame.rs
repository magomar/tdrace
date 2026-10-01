use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

use cabinet::ui::{
    ColumnAlign, DataColumn, DataRow, DataTable, LayoutRect, ModalContainer, ScreenFooter,
    TextInputWidget,
};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::db::HallOfFameEntry;
use crate::render::color::Palette;

/// Congratulations metadata earned by the player upon race completion.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerCongrats {
    pub is_personal_best: bool,
    pub personal_best_lap: Option<f32>,
    pub hof_rank: Option<usize>,      // 1..=10 if qualified for Top 10
    pub race_position: Option<usize>, // 1, 2, 3 if on race podium
}

impl PlayerCongrats {
    pub fn has_achievements(&self) -> bool {
        self.is_personal_best || self.hof_rank.is_some() || self.race_position.is_some()
    }
}

/// Display payload pairing a Hall of Fame record with its ordinal rank so the
/// platform `DataTable` can render rank badges and the "P1..P10" position column.
#[derive(Clone)]
struct HofRow {
    rank: usize,
    name: String,
    car: String,
    total: String,
    lap: String,
    date: String,
}

/// Trims a SQLite `YYYY-MM-DD HH:MM:SS` timestamp to its date component.
fn hof_short_date(created_at: &str) -> String {
    created_at.split(' ').next().unwrap_or(created_at).to_string()
}

/// Builds the Hall of Fame leaderboard as a `DataTable` with rank badges and
/// player-row highlighting (up to 10 slots, vacant slots rendered as placeholders).
fn hall_of_fame_table(entries: &[HallOfFameEntry], highlight_id: Option<i64>) -> DataTable<HofRow> {
    let mut table = DataTable::new(0.0, 0.0, 1.0, 32.0, 28.0);
    table.add_column(DataColumn::new(
        "pos",
        "POS",
        7.0,
        ColumnAlign::Left,
        |r: &HofRow| format!("P{}", r.rank),
    ));
    table.add_column(DataColumn::new(
        "driver",
        "DRIVER",
        26.0,
        ColumnAlign::Left,
        |r| r.name.clone(),
    ));
    table.add_column(DataColumn::new(
        "vehicle",
        "VEHICLE",
        24.0,
        ColumnAlign::Left,
        |r| r.car.clone(),
    ));
    table.add_column(DataColumn::new(
        "total",
        "TOTAL TIME",
        20.0,
        ColumnAlign::Right,
        |r| r.total.clone(),
    ));
    table.add_column(DataColumn::new(
        "lap",
        "BEST LAP",
        18.0,
        ColumnAlign::Right,
        |r| r.lap.clone(),
    ));
    table.add_column(DataColumn::new(
        "date",
        "DATE",
        12.0,
        ColumnAlign::Right,
        |r| r.date.clone(),
    ));

    let rows: Vec<DataRow<HofRow>> = (0..10)
        .map(|i| {
            let rank = i + 1;
            match entries.get(i) {
                Some(e) => {
                    let is_player = e.id.is_some() && e.id == highlight_id;
                    DataRow::new(
                        e.id.map(|id| id.to_string())
                            .unwrap_or_else(|| format!("vacant-{}", rank)),
                        HofRow {
                            rank,
                            name: if is_player {
                                format!("{} (You)", e.player_name)
                            } else {
                                e.player_name.clone()
                            },
                            car: e.car_name.clone(),
                            total: format_lap_time(e.total_time),
                            lap: format_lap_time(e.best_lap.unwrap_or(0.0)),
                            date: hof_short_date(&e.created_at),
                        },
                    )
                    .with_rank(rank)
                    .with_player(is_player)
                }
                None => DataRow::new(
                    format!("vacant-{}", rank),
                    HofRow {
                        rank,
                        name: "--- VACANT ---".to_string(),
                        car: "--".to_string(),
                        total: "--:--.---".to_string(),
                        lap: "--:--.---".to_string(),
                        date: "--".to_string(),
                    },
                )
                .with_rank(rank),
            }
        })
        .collect();

    table.set_rows(rows);
    table
}

/// Renders the arcade modal dialog prompting the player to enter their name for the Hall of Fame (kept for compatibility).
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

    let box_w = (sw * 0.70).clamp(scaler.s(440.0), scaler.s(640.0));
    let box_h = scaler.s(360.0);
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    // Uniform modal chrome (platform ModalContainer: dim, frame, title, divider)
    let modal = ModalContainer::new(
        "NEW RECORD! TOP 10 QUALIFIED!",
        LayoutRect::new(x, y, box_w, box_h),
    );
    modal.draw(&scaler, fonts, sw, sh);

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
    fonts.draw_ui_bold_centered(
        "ENTER DRIVER NAME:",
        sw * 0.5,
        y + scaler.s(174.0),
        scaler.font_s(16.0),
        Palette::WHITE,
    );

    // Text Input Box (platform TextInputWidget frame + text + cursor)
    let input_w = (box_w - scaler.s(100.0)).clamp(scaler.s(280.0), scaler.s(420.0));
    let input_h = scaler.s(48.0);
    let input_x = (sw - input_w) * 0.5;
    let input_y = y + scaler.s(190.0);

    let mut input = TextInputWidget::new(input_x, input_y, input_w, input_h, 12, "DRIVER NAME");
    input.set_text(input_name);
    input.render_frame();

    let font_size = scaler.font_s(22.0);
    let text_y = input_y + scaler.s(32.0);
    let show_cursor = (cursor_timer * 2.5).fract() < 0.5;

    if input_name.is_empty() {
        if show_cursor {
            let cursor_w = scaler.s(2.0);
            let cursor_h = scaler.s(24.0);
            let cursor_x = sw * 0.5 - cursor_w * 0.5;
            let cursor_y = input_y + (input_h - cursor_h) * 0.5;
            draw_rectangle(cursor_x, cursor_y, cursor_w, cursor_h, Palette::NEON_CYAN);
        }
    } else {
        let dim = fonts.measure_ui_bold(input_name, font_size);
        let text_x = sw * 0.5 - dim.width * 0.5;
        fonts.draw_ui_bold(input_name, text_x, text_y, font_size, Palette::NEON_CYAN);

        if show_cursor {
            let cursor_w = scaler.s(2.0);
            let cursor_h = scaler.s(24.0);
            let cursor_x = text_x + dim.width + scaler.s(3.0);
            let cursor_y = input_y + (input_h - cursor_h) * 0.5;
            draw_rectangle(cursor_x, cursor_y, cursor_w, cursor_h, Palette::NEON_CYAN);
        }
    }

    let chars_count = format!("{}/12", input_name.len());
    fonts.draw_ui_regular(
        &chars_count,
        input_x + input_w - scaler.s(42.0),
        input_y + input_h + scaler.s(18.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );

    let help_line = "[ENTER / GAMEPAD A] Confirm & Save | [BACKSPACE] Erase | [ESC] Skip";
    fonts.draw_ui_regular_centered(
        help_line,
        sw * 0.5,
        y + box_h - scaler.s(24.0),
        scaler.font_s(13.0),
        Palette::UI_TEXT_MUTED,
    );
}

/// Renders the full Hall of Fame Leaderboard screen with historical top 10 results
/// and celebratory achievement banners.
pub fn render_hall_of_fame_screen(
    fonts: &Fonts,
    track_name: &str,
    entries: &[HallOfFameEntry],
    highlight_id: Option<i64>,
    congrats: Option<&PlayerCongrats>,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep modern motorsport gradient backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.96));

    let box_w = (sw * 0.88).clamp(scaler.s(580.0), scaler.s(940.0));
    let box_h = (sh * 0.90).clamp(scaler.s(500.0), scaler.s(760.0));
    let x = (sw - box_w) * 0.5;
    let y = (sh - box_h) * 0.5;

    let has_congrats = congrats.is_some_and(|c| c.has_achievements());
    let card_border = if has_congrats {
        Palette::NEON_GOLD
    } else {
        Palette::UI_CARD_BORDER
    };

    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, card_border, 2.5);

    // Header Title
    let title = "HALL OF FAME — TOP 10 HISTORICAL BEST";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        y + scaler.s(32.0),
        scaler.font_s(26.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let track_label = format!("Circuit: {} | All-Time Session Records", track_name);
    fonts.draw_ui_regular_centered(
        &track_label,
        sw * 0.5,
        y + scaler.s(52.0),
        scaler.font_s(14.0),
        Palette::UI_TEXT_MUTED,
    );

    // Optional Congratulations Banner
    let mut table_start_y = y + scaler.s(68.0);
    if let Some(c) = congrats {
        if c.has_achievements() {
            let banner_w = box_w - scaler.s(40.0);
            let banner_h = scaler.s(38.0);
            let banner_x = x + scaler.s(20.0);
            let banner_y = y + scaler.s(60.0);

            draw_rectangle(
                banner_x,
                banner_y,
                banner_w,
                banner_h,
                Color::new(0.10, 0.20, 0.14, 0.95),
            );
            draw_rectangle_lines(
                banner_x,
                banner_y,
                banner_w,
                banner_h,
                1.5,
                Palette::NEON_GOLD,
            );

            // Assemble badges summary string
            let mut badges = Vec::new();

            if let Some(pos) = c.race_position {
                match pos {
                    1 => badges.push("1ST PLACE VICTORY!".to_string()),
                    2 => badges.push("2ND PLACE PODIUM".to_string()),
                    3 => badges.push("3RD PLACE PODIUM".to_string()),
                    _ => {}
                }
            }

            if let Some(rank) = c.hof_rank {
                if rank == 1 {
                    badges.push("ALL-TIME TRACK RECORD (#1)!".to_string());
                } else if rank <= 3 {
                    badges.push(format!("HALL OF FAME PODIUM (RANK #{})", rank));
                } else {
                    badges.push(format!("TOP 10 QUALIFIED (RANK #{})", rank));
                }
            }

            if c.is_personal_best {
                if let Some(best_lap) = c.personal_best_lap {
                    badges.push(format!("PERSONAL BEST ({})", format_lap_time(best_lap)));
                } else {
                    badges.push("NEW PERSONAL BEST!".to_string());
                }
            }

            let full_text = format!("CONGRATULATIONS!  {}", badges.join("  •  "));
            fonts.draw_ui_bold_centered(
                &full_text,
                sw * 0.5,
                banner_y + scaler.s(23.0),
                scaler.font_s(14.0),
                Palette::NEON_GOLD,
            );

            table_start_y = y + scaler.s(106.0);
        }
    }

    // Hall of Fame leaderboard (platform DataTable with rank badges + player highlight)
    let table_w = box_w - scaler.s(40.0);
    let table_x = x + scaler.s(20.0);
    let table_top = table_start_y - scaler.s(16.0);

    let mut table = hall_of_fame_table(entries, highlight_id);
    table.row_height = scaler.s(25.0);
    table.header_height = scaler.s(24.0);
    table.draw(
        &scaler,
        fonts,
        LayoutRect::new(
            table_x,
            table_top,
            table_w,
            (y + box_h - scaler.s(44.0)) - table_top,
        ),
    );

    // Bottom action footer (platform ScreenFooter)
    let footer_h = scaler.s(36.0);
    let mut footer = ScreenFooter::new(x, y + box_h - footer_h, box_w, footer_h);
    footer.add_prompt("SPACE/ENTER", "Main Menu");
    footer.add_prompt("TAB", "Stats");
    footer.add_prompt("R", "Restart");
    footer.add_prompt("ESC", "Results");
    footer.render_frame();

    let width = box_w / footer.prompts.len().max(1) as f32;
    let size = scaler.font_s(10.0);
    for (i, prompt) in footer.prompts.iter().enumerate() {
        let text = fonts.fit_ui_bold(
            &format!("[{}] {}", prompt.badge, prompt.label),
            size,
            width - scaler.s(8.0),
        );
        fonts.draw_ui_bold_centered(
            &text,
            x + (i as f32 + 0.5) * width,
            y + box_h - footer_h + footer_h * 0.65,
            size,
            Palette::WHITE,
        );
    }
}
