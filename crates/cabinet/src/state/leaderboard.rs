use macroquad::color::Color;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use crate::input::NavGrid2D;
use crate::profile::country::draw_country_banner;
use crate::records::leaderboard::{HallOfFame, RecordMetric};
use crate::state::stack::{CabinetContext, CabinetScreen, ScreenAction};
use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Layout rectangles for the leaderboard modal.
#[derive(Debug, Clone, Copy)]
pub struct LeaderboardLayout {
    pub box_x: f32,
    pub box_y: f32,
    pub box_w: f32,
    pub box_h: f32,
    pub close_btn_rect: (f32, f32, f32, f32),
    pub row_start_y: f32,
    pub row_h: f32,
    pub max_visible_rows: usize,
}

/// Computes responsive layout for the Leaderboard modal.
pub fn leaderboard_modal_layout(sw: f32, sh: f32) -> LeaderboardLayout {
    let scaler = UiScaler::new(sw, sh);
    let box_w = (sw * 0.65).clamp(scaler.s(640.0), scaler.s(820.0));
    let box_h = (sh * 0.78).clamp(scaler.s(440.0), scaler.s(560.0));
    let box_x = (sw - box_w) * 0.5;
    let box_y = (sh - box_h) * 0.5;

    let btn_w = scaler.s(220.0);
    let btn_h = scaler.s(44.0);
    let btn_x = box_x + (box_w - btn_w) * 0.5;
    let btn_y = box_y + box_h - btn_h - scaler.s(20.0);

    let row_start_y = box_y + scaler.s(105.0);
    let row_h = scaler.s(34.0);
    let max_visible_rows = (((btn_y - scaler.s(12.0)) - row_start_y) / row_h).floor() as usize;

    LeaderboardLayout {
        box_x,
        box_y,
        box_w,
        box_h,
        close_btn_rect: (btn_x, btn_y, btn_w, btn_h),
        row_start_y,
        row_h,
        max_visible_rows: max_visible_rows.max(5),
    }
}

/// Formats a leaderboard score according to the category metric.
pub fn format_metric_score(score: f64, metric: RecordMetric) -> String {
    match metric {
        RecordMetric::LowestTime => {
            let total_sec = score as f32;
            let mins = (total_sec / 60.0).floor() as u32;
            let secs = total_sec % 60.0;
            if mins > 0 {
                format!("{}:{:06.3}", mins, secs)
            } else {
                format!("{:.3}s", secs)
            }
        }
        RecordMetric::HighestScore => {
            let pts = score.round() as i64;
            let s = pts.to_string();
            let mut out = String::new();
            let len = s.len();
            for (i, ch) in s.chars().enumerate() {
                if i > 0 && (len - i).is_multiple_of(3) {
                    out.push(',');
                }
                out.push(ch);
            }
            format!("{} PTS", out)
        }
    }
}

/// Generic arcade leaderboard and Hall of Fame screen modal.
///
/// Features:
/// - Renders sorted `HallOfFame` records with ranking badges (P1 Gold, P2 Silver, P3 Bronze).
/// - Dynamic metric formatting (Time `M:SS.mmm` vs Points `1,250,000 PTS`).
/// - Flag rendering for player nationalities using `CountryRegistry`.
/// - Highlight rank indicator to spotlight the player's newly earned record.
/// - Scroll navigation for catalogs with > 8-10 entries.
/// - Full 2D navigation and Gamepad `A`/`B`/`Start` support.
pub struct LeaderboardModal {
    pub title: String,
    pub hall_of_fame: HallOfFame,
    pub highlight_rank: Option<usize>, // 1-indexed (e.g. 1 = P1)
    pub scroll_offset: usize,
    pub nav: NavGrid2D, // Row navigation
}

impl LeaderboardModal {
    /// Creates a leaderboard modal with a custom title and HallOfFame data.
    pub fn new(title: impl Into<String>, hall_of_fame: HallOfFame) -> Self {
        let entry_count = hall_of_fame.entries.len();
        Self {
            title: title.into(),
            hall_of_fame,
            highlight_rank: None,
            scroll_offset: 0,
            nav: NavGrid2D::new(vec![entry_count.max(1) + 1]), // rows + close button
        }
    }

    /// Highlights a specific rank position (1-indexed).
    pub fn with_highlight(mut self, rank: usize) -> Self {
        self.highlight_rank = Some(rank);
        if rank > 0 && rank <= self.hall_of_fame.entries.len() {
            self.nav.set_focus(0, rank - 1);
        }
        self
    }
}

impl CabinetScreen for LeaderboardModal {
    fn name(&self) -> &str {
        "LeaderboardModal"
    }

    fn is_transparent(&self) -> bool {
        true
    }

    fn update(&mut self, ctx: &mut CabinetContext) -> ScreenAction {
        let total_items = self.hall_of_fame.entries.len() + 1; // entries + close button
        if self.nav.column_lengths[0] != total_items {
            self.nav.set_column_len(0, total_items);
        }

        self.nav.handle_standard_inputs(
            ctx.gamepad.nav_left,
            ctx.gamepad.nav_right,
            ctx.gamepad.nav_up,
            ctx.gamepad.nav_down,
        );

        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let layout = leaderboard_modal_layout(sw, sh);

        // Adjust scroll offset to keep focused item in view
        let selected_row = self.nav.cursor_rows[0];
        if selected_row < self.hall_of_fame.entries.len() {
            if selected_row < self.scroll_offset {
                self.scroll_offset = selected_row;
            } else if selected_row >= self.scroll_offset + layout.max_visible_rows {
                self.scroll_offset = selected_row.saturating_sub(layout.max_visible_rows - 1);
            }
        }

        let close_hovered = NavGrid2D::check_mouse_hover(layout.close_btn_rect);
        if close_hovered {
            self.nav.set_focus(0, self.hall_of_fame.entries.len());
        }

        let close_clicked = NavGrid2D::check_mouse_click(layout.close_btn_rect);
        if close_clicked || self.nav.is_cancelled(ctx.gamepad.btn_cancel_pressed || ctx.gamepad.btn_b_pressed) {
            return ScreenAction::Pop;
        }

        if self.nav.is_confirmed(ctx.gamepad.btn_confirm_pressed || ctx.gamepad.btn_a_pressed) {
            return ScreenAction::Pop;
        }

        if ctx.gamepad.btn_start_pressed {
            return ScreenAction::Pop;
        }

        ScreenAction::None
    }

    fn draw(&self, ctx: &CabinetContext) {
        let sw = ctx.scaler.screen_w;
        let sh = ctx.scaler.screen_h;
        let scaler = ctx.scaler;
        let fonts = ctx.fonts;

        // Dark dim backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.80));

        let layout = leaderboard_modal_layout(sw, sh);

        // Glassmorphism main card
        scaler.draw_glass_card(
            layout.box_x,
            layout.box_y,
            layout.box_w,
            layout.box_h,
            Palette::UI_CARD_BG,
            Palette::NEON_GOLD,
            2.2,
        );

        // Header Title
        fonts.draw_display_centered_with_shadow(
            &self.title,
            sw * 0.5,
            layout.box_y + scaler.s(36.0),
            scaler.font_s(26.0),
            Palette::NEON_GOLD,
            Color::new(0.0, 0.0, 0.0, 0.6),
            scaler.s(2.0),
        );

        // Category Subtitle
        let cat_label = format!("CATEGORY: {}", self.hall_of_fame.category_id.to_uppercase());
        fonts.draw_ui_bold_centered(
            &cat_label,
            sw * 0.5,
            layout.box_y + scaler.s(58.0),
            scaler.font_s(11.5),
            Palette::UI_TEXT_MUTED,
        );

        // Table Header Columns
        let table_x = layout.box_x + scaler.s(24.0);
        let table_w = layout.box_w - scaler.s(48.0);
        let th_y = layout.box_y + scaler.s(88.0);

        fonts.draw_ui_bold("RANK", table_x + scaler.s(8.0), th_y, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold("PLAYER / CALLSIGN", table_x + scaler.s(70.0), th_y, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold("NAT", table_x + scaler.s(310.0), th_y, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold("RECORD SCORE", table_x + scaler.s(390.0), th_y, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold("DATE", table_x + table_w - scaler.s(100.0), th_y, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);

        draw_rectangle(table_x, th_y + scaler.s(5.0), table_w, scaler.s(1.0), Palette::UI_CARD_BORDER);

        // Entries Rendering
        if self.hall_of_fame.entries.is_empty() {
            fonts.draw_ui_regular_centered(
                "NO RECORD ENTRIES YET - COMPLETE A RUN TO SET A BENCHMARK!",
                sw * 0.5,
                layout.row_start_y + scaler.s(45.0),
                scaler.font_s(14.0),
                Palette::UI_TEXT_MUTED,
            );
        } else {
            let visible_count = layout.max_visible_rows.min(self.hall_of_fame.entries.len() - self.scroll_offset);
            for i in 0..visible_count {
                let entry_idx = self.scroll_offset + i;
                let entry = &self.hall_of_fame.entries[entry_idx];
                let rank = entry_idx + 1;
                let row_y = layout.row_start_y + (i as f32 * layout.row_h);

                let is_highlighted = self.highlight_rank == Some(rank);
                let is_selected = self.nav.cursor_rows[0] == entry_idx;

                // Row backdrop card
                let row_bg = if is_highlighted {
                    Color::new(0.20, 0.90, 0.45, 0.18)
                } else if is_selected {
                    Color::new(0.12, 0.18, 0.28, 0.85)
                } else if i % 2 == 0 {
                    Color::new(0.08, 0.11, 0.17, 0.50)
                } else {
                    Color::new(0.05, 0.07, 0.11, 0.40)
                };

                let border_col = if is_highlighted {
                    Palette::NEON_GREEN
                } else if is_selected {
                    Palette::NEON_CYAN
                } else {
                    Color::new(0.15, 0.20, 0.30, 0.40)
                };

                draw_rectangle(table_x, row_y, table_w, layout.row_h - scaler.s(3.0), row_bg);
                if is_highlighted || is_selected {
                    draw_rectangle_lines(table_x, row_y, table_w, layout.row_h - scaler.s(3.0), 1.5, border_col);
                }

                // Rank badge / text
                let (rank_str, rank_col) = match rank {
                    1 => ("P1 ★".to_string(), Palette::NEON_GOLD),
                    2 => ("P2".to_string(), Color::new(0.85, 0.90, 0.95, 1.0)),
                    3 => ("P3".to_string(), Color::new(0.90, 0.60, 0.35, 1.0)),
                    _ => (format!("P{}", rank), Palette::UI_TEXT_MUTED),
                };
                fonts.draw_ui_bold(&rank_str, table_x + scaler.s(8.0), row_y + scaler.s(21.0), scaler.font_s(13.0), rank_col);

                // Player name and callsign
                let name_display = if !entry.player_alias.is_empty() {
                    format!("{} [{}]", entry.player_name, entry.player_alias)
                } else {
                    entry.player_name.clone()
                };
                let name_col = if is_highlighted { Palette::NEON_GREEN } else { Palette::WHITE };
                fonts.draw_ui_bold(&name_display, table_x + scaler.s(70.0), row_y + scaler.s(21.0), scaler.font_s(13.0), name_col);

                // National Flag
                draw_country_banner(
                    entry.country.as_deref(),
                    table_x + scaler.s(310.0),
                    row_y + scaler.s(7.0),
                    scaler.s(48.0),
                    scaler.s(16.0),
                    Some(fonts),
                    scaler,
                );

                // Record Score / Lap Time
                let score_str = format_metric_score(entry.score, self.hall_of_fame.metric);
                let score_col = if rank == 1 { Palette::NEON_GOLD } else { Palette::NEON_CYAN };
                fonts.draw_ui_bold(&score_str, table_x + scaler.s(390.0), row_y + scaler.s(21.0), scaler.font_s(13.5), score_col);

                // Date stamp
                fonts.draw_ui_regular(&entry.timestamp, table_x + table_w - scaler.s(100.0), row_y + scaler.s(21.0), scaler.font_s(11.5), Palette::UI_TEXT_MUTED);
            }
        }

        // Close Button
        let close_focused = self.nav.cursor_rows[0] >= self.hall_of_fame.entries.len();
        let close_hovered = NavGrid2D::check_mouse_hover(layout.close_btn_rect);
        let (bx, by, bw, bh) = layout.close_btn_rect;
        scaler.draw_button_card(bx, by, bw, bh, close_focused, close_hovered, Palette::NEON_CYAN);
        fonts.draw_ui_bold_centered(
            "CLOSE [ESC / ENTER]",
            bx + bw * 0.5,
            by + bh * 0.60,
            scaler.font_s(14.0),
            Palette::WHITE,
        );
    }
}
