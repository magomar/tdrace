//! Tournament bracket screen (Spec 104): the weekend tree, the player's slot and the launch button.
//!
//! [`tournament_bracket_view`] builds what the screen shows from the series session, so the content is
//! testable without a window. [`render_tournament_bracket_screen`] draws it.

use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};

use super::font::Fonts;
use super::hud::format_lap_time;
use super::scaler::UiScaler;
use crate::render::color::Palette;
use crate::series::{RoundDriverResult, SeriesSession, TournamentStage, ADVANCERS_PER_GROUP, DNF_TIME};

/// One driver on the bracket.
#[derive(Debug, Clone, PartialEq)]
pub struct BracketCard {
    pub driver_id: String,
    pub name: String,
    pub team: String,
    /// Catalog name of the car the series gives the driver, when it declares one.
    pub car: Option<String>,
    /// Finishing position once the race is run.
    pub position: Option<usize>,
    /// Elapsed time once the race is run.
    pub time: Option<f32>,
    pub is_player: bool,
    /// `Some(true)` when the driver went on to the next stage, `Some(false)` when not. `None` before
    /// the race is run and in the finals.
    pub advanced: Option<bool>,
}

/// One race of the bracket: eight cards, or none for a stage not drawn yet.
#[derive(Debug, Clone, PartialEq)]
pub struct BracketGroup {
    pub label: String,
    pub laps: u32,
    /// The race the player drives next: drawn with the amber border.
    pub is_player_race: bool,
    pub cards: Vec<BracketCard>,
}

/// One stage of the bracket.
#[derive(Debug, Clone, PartialEq)]
pub struct BracketColumn {
    pub title: String,
    pub is_active: bool,
    pub groups: Vec<BracketGroup>,
}

/// Everything the bracket screen shows.
#[derive(Debug, Clone, PartialEq)]
pub struct TournamentBracketView {
    pub title: String,
    pub subtitle: String,
    pub columns: Vec<BracketColumn>,
    /// `(column, group)` of the race the player drives next.
    pub player_slot: Option<(usize, usize)>,
    pub launch_label: String,
}

fn plural_title(stage: TournamentStage) -> &'static str {
    match stage {
        TournamentStage::QualifyingHeat => "QUALIFYING HEATS",
        TournamentStage::Quarterfinal => "QUARTERFINALS",
        TournamentStage::Semifinal => "SEMIFINALS",
        TournamentStage::GrandFinal => "FINALS",
        TournamentStage::ConsolationSemifinal => "CONSOLATION",
        TournamentStage::ConsolationBFinal => "B-FINAL",
    }
}

fn group_label(stage: TournamentStage, group: usize) -> String {
    match stage {
        TournamentStage::QualifyingHeat => format!("HEAT {}", group + 1),
        TournamentStage::Quarterfinal => format!("QUARTERFINAL {}", group + 1),
        TournamentStage::Semifinal => format!("SEMIFINAL {}", group + 1),
        other => other.title().to_string(),
    }
}

/// Builds the bracket of the weekend running in `champ`. `None` when there is no weekend.
pub fn tournament_bracket_view(champ: &SeriesSession) -> Option<TournamentBracketView> {
    let weekend = champ.tournament.as_ref()?;
    let config = weekend.config;
    let final_round = config.round_count() - 1;
    let player_race = weekend.player_race();

    let card = |id: &str, result: Option<&RoundDriverResult>, advanced: Option<bool>| {
        let entry = champ.standings.iter().find(|s| s.driver_id == id);
        BracketCard {
            driver_id: id.to_string(),
            name: entry.map(|e| e.driver_name.clone()).or_else(|| result.map(|r| r.driver_name.clone())).unwrap_or_else(|| id.to_string()),
            team: entry.map(|e| e.team_name.clone()).or_else(|| result.map(|r| r.team_name.clone())).unwrap_or_default(),
            car: entry
                .and_then(|e| e.car_model_id.as_deref())
                .and_then(crate::catalog::find_model_by_id)
                .map(|m| m.name.to_string()),
            position: result.map(|r| r.finish_position),
            time: result.map(|r| r.total_time),
            is_player: id == weekend.player_id,
            advanced,
        }
    };
    let result_group = |label: String, laps: u32, results: &[RoundDriverResult], mark_advance: bool| BracketGroup {
        label,
        laps,
        is_player_race: false,
        cards: results
            .iter()
            .map(|r| card(&r.driver_id, Some(r), mark_advance.then_some(r.finish_position <= ADVANCERS_PER_GROUP)))
            .collect(),
    };

    let mut columns = Vec::new();
    let mut player_slot = None;
    for round in 0..=final_round {
        let stage = config.stage_for_round(round);
        let mut groups = Vec::new();
        if round < weekend.active_round {
            for (g, results) in weekend.round_results(round).iter().enumerate() {
                groups.push(result_group(group_label(stage, g), config.laps_for(stage), results, true));
            }
            if let Some((_, results)) = weekend.consolation_results.iter().find(|(r, _)| *r == round) {
                groups.push(result_group(TournamentStage::ConsolationSemifinal.title().to_string(), config.semi_laps, results, false));
            }
        } else if round == weekend.active_round {
            for race in &weekend.active_races {
                let is_player_race = player_race == Some(race);
                if is_player_race {
                    player_slot = Some((columns.len(), groups.len()));
                }
                groups.push(BracketGroup {
                    label: group_label(race.stage, race.group),
                    laps: race.laps,
                    is_player_race,
                    cards: race.driver_ids.iter().map(|id| card(id, None, None)).collect(),
                });
            }
        } else {
            for g in 0..config.groups_in_round(round) {
                groups.push(BracketGroup { label: group_label(stage, g), laps: config.laps_for(stage), is_player_race: false, cards: Vec::new() });
            }
            if round == final_round {
                groups.push(BracketGroup {
                    label: TournamentStage::ConsolationBFinal.title().to_string(),
                    laps: config.semi_laps,
                    is_player_race: false,
                    cards: Vec::new(),
                });
            }
        }
        columns.push(BracketColumn { title: plural_title(stage).to_string(), is_active: round == weekend.active_round, groups });
    }

    let launch_label = match player_race {
        Some(race) => format!("LAUNCH RACE — {} • {} LAPS", race.badge(), race.laps),
        None => "WEEKEND COMPLETE".to_string(),
    };
    Some(TournamentBracketView {
        title: champ.name.to_uppercase(),
        subtitle: format!("ROUND {} OF {} • {}-DRIVER TOURNAMENT WEEKEND", champ.current_round + 1, champ.total_rounds(), config.total_drivers),
        columns,
        player_slot,
        launch_label,
    })
}

/// The badge the race HUD shows for the race the player drives: `HEAT 2/4`, `GRAND FINAL`.
pub fn stage_badge(champ: &SeriesSession) -> Option<String> {
    champ.tournament_race().map(|r| r.badge())
}

fn amber() -> Color {
    Palette::NEON_GOLD
}

fn draw_group(fonts: &Fonts, scaler: &UiScaler, x: f32, y: f32, w: f32, group: &BracketGroup) -> f32 {
    let row_h = scaler.s(15.0);
    let head_h = scaler.s(20.0);
    let rows = group.cards.len().max(1);
    let h = head_h + rows as f32 * row_h + scaler.s(6.0);
    let border = if group.is_player_race { amber() } else { Palette::UI_CARD_BORDER };
    scaler.draw_glass_card(x, y, w, h, Palette::UI_CARD_BG, border, if group.is_player_race { 3.0 } else { 1.2 });
    fonts.draw_ui_bold(&format!("{} • {} LAPS", group.label, group.laps), x + scaler.s(8.0), y + scaler.s(14.0), scaler.font_s(11.0), border);
    if group.cards.is_empty() {
        fonts.draw_ui_regular("TO BE DECIDED", x + scaler.s(8.0), y + head_h + row_h * 0.8, scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
    }
    for (i, c) in group.cards.iter().enumerate() {
        let ry = y + head_h + i as f32 * row_h;
        if c.is_player {
            draw_rectangle(x + scaler.s(3.0), ry, w - scaler.s(6.0), row_h, Color::new(amber().r, amber().g, amber().b, 0.22));
            draw_rectangle_lines(x + scaler.s(3.0), ry, w - scaler.s(6.0), row_h, scaler.s(1.2), amber());
        }
        let color = match c.advanced {
            Some(true) => Palette::NEON_GREEN,
            Some(false) => Palette::UI_TEXT_MUTED,
            None => Palette::WHITE,
        };
        let pos = c.position.map(|p| format!("{:>2}", p)).unwrap_or_else(|| format!("{:>2}", i + 1));
        let mut line = format!("{}  {}", pos, c.name);
        if let Some(car) = &c.car {
            line = format!("{} • {}", line, car);
        }
        let right = c.time.map(|t| if t >= DNF_TIME { "DNF".to_string() } else { format_lap_time(t) }).unwrap_or_else(|| c.team.clone());
        let text_w = w - scaler.s(16.0) - scaler.s(86.0);
        let left_text = fonts.fit_ui_bold(&line, scaler.font_s(10.0), text_w);
        fonts.draw_ui_bold(&left_text, x + scaler.s(8.0), ry + row_h * 0.78, scaler.font_s(10.0), color);
        let right_text = fonts.fit_ui_regular(&right, scaler.font_s(9.5), scaler.s(84.0));
        fonts.draw_ui_regular(&right_text, x + w - scaler.s(88.0), ry + row_h * 0.78, scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
    }
    h
}

/// Draws the bracket screen: header, the stage columns with their cards and the launch prompt.
pub fn render_tournament_bracket_screen(fonts: &Fonts, champ: &SeriesSession) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));
    let Some(view) = tournament_bracket_view(champ) else {
        return;
    };

    fonts.draw_display_centered_with_shadow(&view.title, sw * 0.5, scaler.s(40.0), scaler.font_s(30.0), Palette::NEON_GOLD, Color::new(0.0, 0.0, 0.0, 0.6), scaler.s(2.0));
    fonts.draw_ui_bold_centered(&view.subtitle, sw * 0.5, scaler.s(64.0), scaler.font_s(13.0), Palette::NEON_CYAN);

    let margin = scaler.s(24.0);
    let gap = scaler.s(14.0);
    let cols = view.columns.len().max(1) as f32;
    let col_w = ((sw - 2.0 * margin - gap * (cols - 1.0)) / cols).min(scaler.s(440.0));
    let total_w = col_w * cols + gap * (cols - 1.0);
    let left = (sw - total_w) * 0.5;
    let top = scaler.s(84.0);
    for (ci, column) in view.columns.iter().enumerate() {
        let x = left + ci as f32 * (col_w + gap);
        let head_col = if column.is_active { amber() } else { Palette::UI_TEXT_MUTED };
        fonts.draw_ui_bold(&column.title, x, top + scaler.s(12.0), scaler.font_s(13.0), head_col);
        let mut y = top + scaler.s(20.0);
        for group in &column.groups {
            y += draw_group(fonts, &scaler, x, y, col_w, group) + scaler.s(6.0);
        }
    }

    let btn_w = scaler.s(520.0).min(sw - 2.0 * margin);
    let btn_h = scaler.s(40.0);
    let btn_x = (sw - btn_w) * 0.5;
    let btn_y = sh - scaler.s(96.0);
    draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(amber().r * 0.25, amber().g * 0.25, amber().b * 0.25, 0.95));
    draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, scaler.s(2.5), amber());
    fonts.draw_ui_bold_centered(&format!("[ {} ]", view.launch_label), sw * 0.5, btn_y + btn_h * 0.64, scaler.font_s(16.0), amber());
    fonts.draw_ui_bold_centered("[Enter / A] Launch Race     [Esc / B] Exit", sw * 0.5, sh - scaler.s(30.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
}
