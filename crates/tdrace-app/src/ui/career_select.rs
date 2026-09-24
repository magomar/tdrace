use std::collections::HashMap;
use macroquad::prelude::*;
use crate::profile::{ModuleCareerProgress, PlayerProfile, ProfileCareerStats};
use crate::render::color::Palette;
use crate::series::{ChampionshipManager, ChampionshipSession, SeriesDefinition};
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;

/// Represents a discipline card in the multi-career selection screen.
#[derive(Debug, Clone, PartialEq)]
pub struct CareerSelectCard {
    pub module_id: String,
    pub series_id: String,
    pub series_name: String,
    pub title: String,
    pub subtitle: String,
    pub tier: u32,
    pub tier_name: String,
    pub current_round: usize,
    pub total_rounds: usize,
    pub is_active: bool,
    pub player_points: u32,
    pub player_rank: usize,
    pub total_drivers: usize,
    pub next_track_id: String,
    pub next_track_name: String,
    pub car_model_id: Option<String>,
    pub accent_color: Color,
}

/// Discipline metadata descriptor.
struct ModalityMeta {
    id: &'static str,
    title: &'static str,
    subtitle: &'static str,
    accent: Color,
    default_series_name: &'static str,
}

const MODALITY_CATALOG: &[ModalityMeta] = &[
    ModalityMeta {
        id: "gt",
        title: "GT WORLD CHALLENGE",
        subtitle: "FIA GT3 & SRO GT2 Multiclass Touring & Endurance",
        accent: Palette::RED,
        default_series_name: "GT World Challenge Championship 2026",
    },
    ModalityMeta {
        id: "nascar",
        title: "NASCAR CUP SERIES",
        subtitle: "850 BHP Pushrod V8 High-Banked Superspeedways",
        accent: Palette::YELLOW,
        default_series_name: "NASCAR Cup Series 2026",
    },
    ModalityMeta {
        id: "rally",
        title: "RALLYCROSS WORLD CUP",
        subtitle: "World RX & Euro RX Mixed Surface Stages & Jump Arenas",
        accent: Palette::NEON_GOLD,
        default_series_name: "Rallycross World Cup 2026",
    },
    ModalityMeta {
        id: "kart",
        title: "KARTING WORLD CUP",
        subtitle: "125cc Direct Steering Shifter Karts & Sprint Arenas",
        accent: Palette::NEON_GREEN,
        default_series_name: "Karting World Cup 2026",
    },
    ModalityMeta {
        id: "extreme_offroad",
        title: "EXTREME OFF-ROAD",
        subtitle: "Baja Deserts, Ice Lakes, Supercross Triples & Stunt Arenas",
        accent: Color::new(1.0, 0.40, 0.05, 1.0),
        default_series_name: "Extreme Off-Road Cup 2026",
    },
];

/// Builds the ordered list of career select cards.
/// Active championships appear first, followed by available disciplines without an active championship.
pub fn build_career_select_cards(
    champ_manager: &ChampionshipManager,
    module_progress_map: &HashMap<String, ModuleCareerProgress>,
    active_progress: &ModuleCareerProgress,
    current_session: Option<&ChampionshipSession>,
    _history: &[crate::profile::RaceHistoryEntry],
) -> (Vec<CareerSelectCard>, usize) {
    let mut active_cards = Vec::new();
    let mut available_cards = Vec::new();

    for meta in MODALITY_CATALOG {
        let dummy_prog;
        let prog = if let Some(p) = module_progress_map.get(meta.id) {
            p
        } else if active_progress.module_id == meta.id {
            active_progress
        } else {
            dummy_prog = ModuleCareerProgress::default_for_module(1, meta.id);
            &dummy_prog
        };
        let tier = prog.level.clamp(1, 5);
        let tier_name = match tier {
            1 => "TIER 1 (ROOKIE)".to_string(),
            2 => "TIER 2 (AMATEUR)".to_string(),
            3 => "TIER 3 (CONTENDER)".to_string(),
            4 => "TIER 4 (PRO)".to_string(),
            _ => "TIER 5 (LEGEND)".to_string(),
        };

        // Find active session for this module:
        // 1. First check the module's own progress (which is strictly scoped to this modality)
        // 2. Fall back to current_session if it matches this modality
        let session_opt = prog.active_championship.as_ref()
            .filter(|s| !s.is_completed)
            .or_else(|| {
                current_session.filter(|s| {
                    !s.is_completed && (
                        champ_manager.series.values().any(|def| {
                            def.series.module_id == meta.id
                                && (def.series.name.eq_ignore_ascii_case(&s.name)
                                    || def.series.id.eq_ignore_ascii_case(&s.name))
                        })
                        || s.name.to_lowercase().contains(meta.id)
                        || meta.id.contains(&s.name.to_lowercase())
                        || s.name.eq_ignore_ascii_case(meta.default_series_name)
                    )
                })
            });

        // Resolve series definition if available
        let series_def: Option<&SeriesDefinition> = champ_manager.series.values().find(|s| {
            s.series.module_id == meta.id && s.series.tier == tier
        }).or_else(|| {
            champ_manager.series.values().find(|s| s.series.module_id == meta.id)
        });

        if let Some(champ) = session_opt {
            let total_rounds = champ.track_ids.len().max(1);
            let current_round = champ.current_round.min(total_rounds);
            let player_standing = champ.standings.iter().find(|s| s.driver_id == "player");
            let player_points = player_standing.map(|s| s.points).unwrap_or(0);
            let player_rank = champ.standings.iter().position(|s| s.driver_id == "player").map(|p| p + 1).unwrap_or(1);
            let total_drivers = champ.standings.len();

            let next_track_id = champ.current_track_id().unwrap_or(
                champ.track_ids.first().map(|s| s.as_str()).unwrap_or("monza")
            ).to_string();
            let next_track_name = next_track_id.replace('_', " ").to_uppercase();

            let car_model_id = prog.unlocked_cars.first().cloned();

            active_cards.push(CareerSelectCard {
                module_id: meta.id.to_string(),
                series_id: series_def.map(|d| d.series.id.clone()).unwrap_or_else(|| meta.id.to_string()),
                series_name: champ.name.clone(),
                title: meta.title.to_string(),
                subtitle: meta.subtitle.to_string(),
                tier,
                tier_name,
                current_round,
                total_rounds,
                is_active: true,
                player_points,
                player_rank,
                total_drivers,
                next_track_id,
                next_track_name,
                car_model_id,
                accent_color: meta.accent,
            });
        } else {
            let total_rounds = series_def.map(|d| d.rounds.len()).unwrap_or(4);
            let next_track_id = series_def.and_then(|d| d.rounds.first().map(|r| r.track_id.as_str())).unwrap_or("monza").to_string();
            let next_track_name = next_track_id.replace('_', " ").to_uppercase();
            let series_name = series_def.map(|d| d.series.name.clone()).unwrap_or_else(|| meta.default_series_name.to_string());
            let car_model_id = prog.unlocked_cars.first().cloned();

            available_cards.push(CareerSelectCard {
                module_id: meta.id.to_string(),
                series_id: series_def.map(|d| d.series.id.clone()).unwrap_or_else(|| meta.id.to_string()),
                series_name,
                title: meta.title.to_string(),
                subtitle: meta.subtitle.to_string(),
                tier,
                tier_name,
                current_round: 0,
                total_rounds,
                is_active: false,
                player_points: 0,
                player_rank: 0,
                total_drivers: series_def.map(|d| d.drivers.len()).unwrap_or(8),
                next_track_id,
                next_track_name,
                car_model_id,
                accent_color: meta.accent,
            });
        }
    }

    let active_count = active_cards.len();
    active_cards.extend(available_cards);
    (active_cards, active_count)
}

/// Computes the bounding rect for a career select card given its index and current selected index in the accordion.
pub fn career_select_card_rect(index: usize, selected_idx: usize, sw: f32, sh: f32) -> Rect {
    let scaler = UiScaler::new(sw, sh);
    let top_margin = scaler.s(80.0);
    let card_width = (sw * 0.90).clamp(scaler.s(640.0), scaler.s(1180.0));
    let card_x = (sw - card_width) * 0.5;

    let collapsed_h = scaler.s(52.0);
    let expanded_h = scaler.s(148.0);
    let card_gap = scaler.s(8.0);

    let card_y = if index < selected_idx {
        top_margin + (index as f32) * (collapsed_h + card_gap)
    } else if index == selected_idx {
        top_margin + (selected_idx as f32) * (collapsed_h + card_gap)
    } else {
        top_margin + (selected_idx as f32) * (collapsed_h + card_gap)
            + expanded_h + card_gap
            + ((index - selected_idx - 1) as f32) * (collapsed_h + card_gap)
    };

    let height = if index == selected_idx {
        expanded_h
    } else {
        collapsed_h
    };

    Rect::new(card_x, card_y, card_width, height)
}

/// Renders the multi-career selection screen using an Expandable Accordion layout.
pub fn render_career_select_screen(
    fonts: &Fonts,
    cards: &[CareerSelectCard],
    active_count: usize,
    selected_idx: usize,
    active_profile: &PlayerProfile,
    _stats: &ProfileCareerStats,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep modern motorsport dark gradient backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Top Header Banner
    let header_y = scaler.s(28.0);
    let title_fs = scaler.font_s(26.0);
    let sub_fs = scaler.font_s(12.5);

    fonts.draw_display(
        "CAREER SELECTION",
        scaler.s(48.0),
        header_y,
        title_fs,
        Palette::WHITE,
    );

    fonts.draw_ui_regular(
        "PARALLEL DISCIPLINES — RESUME ONGOING SEASONS OR COMMENCE NEW CAMPAIGNS",
        scaler.s(48.0),
        header_y + scaler.s(18.0),
        sub_fs,
        Palette::UI_TEXT_MUTED,
    );

    // Active Careers Pill Badge
    let pill_x = scaler.s(370.0);
    let pill_w = scaler.s(165.0);
    let pill_h = scaler.s(24.0);
    let pill_y = header_y - scaler.s(18.0);

    let (pill_bg, pill_fg) = if active_count > 0 {
        (Color::new(0.95, 0.70, 0.05, 0.20), Palette::NEON_GOLD)
    } else {
        (Color::new(0.3, 0.35, 0.4, 0.3), Palette::UI_TEXT_MUTED)
    };
    draw_rectangle(pill_x, pill_y, pill_w, pill_h, pill_bg);
    draw_rectangle_lines(pill_x, pill_y, pill_w, pill_h, 1.2, pill_fg);
    let pill_text = format!("ACTIVE CAREERS: {}/5", active_count);
    fonts.draw_ui_bold(
        &pill_text,
        pill_x + scaler.s(12.0),
        pill_y + scaler.s(16.0),
        scaler.font_s(11.0),
        pill_fg,
    );

    // Profile Identity Badge (Top Right)
    let badge_w = scaler.s(250.0);
    let badge_h = scaler.s(42.0);
    let badge_x = sw - scaler.s(48.0) - badge_w;
    let badge_y = header_y - scaler.s(18.0);
    scaler.draw_glass_card(badge_x, badge_y, badge_w, badge_h, Color::new(0.08, 0.10, 0.15, 0.85), Palette::NEON_CYAN, 1.0);

    fonts.draw_ui_bold(
        &active_profile.alias.to_uppercase(),
        badge_x + scaler.s(14.0),
        badge_y + scaler.s(19.0),
        scaler.font_s(13.5),
        Palette::WHITE,
    );
    let profile_name_text = format!("PROFILE: {}", active_profile.name);
    fonts.draw_ui_regular(
        &profile_name_text,
        badge_x + scaler.s(14.0),
        badge_y + scaler.s(34.0),
        scaler.font_s(10.5),
        Palette::UI_TEXT_MUTED,
    );

    // =========================================================================
    // ACCORDION ROWS LIST: Collapsed rows by default, Expanded selected row
    // =========================================================================
    for (idx, card) in cards.iter().enumerate() {
        let rect = career_select_card_rect(idx, selected_idx, sw, sh);
        let is_selected = idx == selected_idx;

        if is_selected {
            // -----------------------------------------------------------------
            // EXPANDED SELECTED ROW (Height ~148px)
            // -----------------------------------------------------------------
            let bg_color = Color::new(0.10, 0.13, 0.21, 0.97);
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.2, card.accent_color);

            // Left Modality Accent Strip
            let strip_w = scaler.s(5.0);
            draw_rectangle(rect.x, rect.y, strip_w, rect.h, card.accent_color);

            // Top Header: Tag + Title + Status Pill
            let tag_x = rect.x + scaler.s(16.0);
            let tag_y = rect.y + scaler.s(14.0);
            let tag_w = scaler.s(68.0);
            let tag_h = scaler.s(22.0);
            draw_rectangle(tag_x, tag_y, tag_w, tag_h, Color::new(card.accent_color.r, card.accent_color.g, card.accent_color.b, 0.20));
            draw_rectangle_lines(tag_x, tag_y, tag_w, tag_h, 1.0, card.accent_color);
            fonts.draw_ui_bold_centered(
                &card.module_id.to_uppercase(),
                tag_x + tag_w * 0.5,
                tag_y + scaler.s(15.0),
                scaler.font_s(11.0),
                card.accent_color,
            );

            let title_x = tag_x + tag_w + scaler.s(14.0);
            fonts.draw_display(
                &card.title,
                title_x,
                tag_y + scaler.s(18.0),
                scaler.font_s(18.0),
                Palette::WHITE,
            );

            // Top Right Status Badge Pill
            let pill_w = scaler.s(160.0);
            let pill_h = scaler.s(22.0);
            let pill_x = rect.x + rect.w - scaler.s(16.0) - pill_w;
            let (p_bg, p_border, p_fg, p_text) = if card.is_active {
                (Color::new(0.95, 0.70, 0.05, 0.20), Palette::NEON_GOLD, Palette::NEON_GOLD, format!("ROUND {} / {} ACTIVE", card.current_round + 1, card.total_rounds))
            } else {
                (Color::new(0.15, 0.22, 0.32, 0.35), Palette::NEON_CYAN, Palette::NEON_CYAN, "READY TO COMMENCE".to_string())
            };
            draw_rectangle(pill_x, tag_y, pill_w, pill_h, p_bg);
            draw_rectangle_lines(pill_x, tag_y, pill_w, pill_h, 1.0, p_border);
            fonts.draw_ui_bold_centered(
                &p_text,
                pill_x + pill_w * 0.5,
                tag_y + scaler.s(15.0),
                scaler.font_s(10.5),
                p_fg,
            );

            // Divider Line
            let div_y = tag_y + tag_h + scaler.s(10.0);
            draw_line(tag_x, div_y, rect.x + rect.w - scaler.s(16.0), div_y, 1.0, Color::new(0.20, 0.24, 0.32, 0.50));

            // Middle Section: Left Details & Right Telemetry
            let body_y = div_y + scaler.s(16.0);

            // Left side: Subtitle lore and tier info
            fonts.draw_ui_regular(
                &card.subtitle,
                tag_x,
                body_y,
                scaler.font_s(12.0),
                Palette::UI_TEXT_MUTED,
            );
            let spec_line = format!("{}  •  {} Total Rounds", card.tier_name, card.total_rounds);
            fonts.draw_ui_bold(
                &spec_line,
                tag_x,
                body_y + scaler.s(18.0),
                scaler.font_s(11.5),
                if card.is_active { Palette::NEON_GOLD } else { Palette::NEON_CYAN },
            );

            // Right side: Standings & Next Venue
            let mid_x = rect.x + rect.w * 0.52;
            if card.is_active {
                let rank_str = format!("CURRENT STANDING: P{} / {}  •  {} PTS", card.player_rank, card.total_drivers, card.player_points);
                fonts.draw_ui_bold(
                    &rank_str,
                    mid_x,
                    body_y,
                    scaler.font_s(12.0),
                    Palette::WHITE,
                );
                let next_str = format!("NEXT EVENT: {}", card.next_track_name);
                fonts.draw_ui_regular(
                    &next_str,
                    mid_x,
                    body_y + scaler.s(18.0),
                    scaler.font_s(11.0),
                    Palette::UI_TEXT_MUTED,
                );
            } else {
                let opener_str = format!("OPENING VENUE: {}", card.next_track_name);
                fonts.draw_ui_bold(
                    &opener_str,
                    mid_x,
                    body_y,
                    scaler.font_s(12.0),
                    Palette::WHITE,
                );
                fonts.draw_ui_regular(
                    "ROUND 1 SEASON INAUGURAL RACE",
                    mid_x,
                    body_y + scaler.s(18.0),
                    scaler.font_s(11.0),
                    Palette::UI_TEXT_MUTED,
                );
            }

            // Bottom Action Area
            let btn_w = scaler.s(210.0);
            let btn_h = scaler.s(32.0);
            let btn_x = rect.x + rect.w - scaler.s(16.0) - btn_w;
            let btn_y = rect.y + rect.h - btn_h - scaler.s(12.0);

            let (btn_bg, btn_fg, btn_text) = if card.is_active {
                (card.accent_color, Palette::BLACK, "▶  RESUME CAREER")
            } else {
                (Palette::NEON_CYAN, Palette::BLACK, "▶  START CAREER")
            };
            draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_bg);
            fonts.draw_ui_bold_centered(
                btn_text,
                btn_x + btn_w * 0.5,
                btn_y + scaler.s(21.0),
                scaler.font_s(13.0),
                btn_fg,
            );

            if card.is_active {
                let reset_fs = scaler.font_s(10.5);
                fonts.draw_ui_regular(
                    "[R] RESET SEASON PROGRESS",
                    tag_x,
                    btn_y + scaler.s(21.0),
                    reset_fs,
                    Palette::RED,
                );
            }
        } else {
            // -----------------------------------------------------------------
            // COLLAPSED UNSELECTED ROW (Height ~52px)
            // -----------------------------------------------------------------
            let bg_color = if card.is_active {
                Color::new(0.06, 0.08, 0.13, 0.80)
            } else {
                Color::new(0.05, 0.06, 0.09, 0.70)
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, Color::new(0.18, 0.22, 0.28, 0.45));

            // Left Modality Accent Strip
            let strip_w = scaler.s(5.0);
            draw_rectangle(rect.x, rect.y, strip_w, rect.h, card.accent_color);

            // Modality Tag Badge
            let tag_x = rect.x + scaler.s(16.0);
            let tag_h = scaler.s(22.0);
            let tag_y = rect.y + (rect.h - tag_h) * 0.5;
            let tag_w = scaler.s(64.0);
            draw_rectangle(tag_x, tag_y, tag_w, tag_h, Color::new(card.accent_color.r, card.accent_color.g, card.accent_color.b, 0.16));
            draw_rectangle_lines(tag_x, tag_y, tag_w, tag_h, 1.0, card.accent_color);

            let tag_text = card.module_id.to_uppercase();
            fonts.draw_ui_bold_centered(
                &tag_text,
                tag_x + tag_w * 0.5,
                tag_y + scaler.s(15.0),
                scaler.font_s(10.5),
                card.accent_color,
            );

            // Title
            let title_x = tag_x + tag_w + scaler.s(14.0);
            fonts.draw_ui_bold(
                &card.title,
                title_x,
                rect.y + scaler.s(32.0),
                scaler.font_s(15.0),
                Color::new(0.85, 0.88, 0.92, 1.0),
            );

            // Right side status pill & expand indicator
            let right_x = rect.x + rect.w - scaler.s(16.0);
            let (status_text, status_col) = if card.is_active {
                (format!("ROUND {}/{}  •  P{} ({} PTS)", card.current_round + 1, card.total_rounds, card.player_rank, card.player_points), Palette::NEON_GOLD)
            } else {
                (format!("{}  •  {} ROUNDS", card.tier_name, card.total_rounds), Palette::UI_TEXT_MUTED)
            };

            let chip_w = scaler.s(190.0);
            let chip_x = right_x - chip_w;
            fonts.draw_ui_regular(
                &status_text,
                chip_x,
                rect.y + scaler.s(32.0),
                scaler.font_s(11.5),
                status_col,
            );
        }
    }

    // Bottom Navigation Bar
    let nav_y = sh - scaler.s(26.0);
    let nav_fs = scaler.font_s(11.5);
    draw_rectangle(0.0, sh - scaler.s(42.0), sw, scaler.s(42.0), Color::new(0.03, 0.04, 0.06, 0.95));
    draw_line(0.0, sh - scaler.s(42.0), sw, sh - scaler.s(42.0), 1.0, Color::new(0.15, 0.18, 0.25, 0.70));

    let nav_str = "[W/S / UP/DOWN] SELECT / EXPAND DISCIPLINE     [ENTER / SPACE] LAUNCH/RESUME     [R] RESET SEASON     [ESC] BACK";
    fonts.draw_ui_regular(
        nav_str,
        scaler.s(48.0),
        nav_y,
        nav_fs,
        Color::new(0.85, 0.88, 0.92, 1.0),
    );
}
