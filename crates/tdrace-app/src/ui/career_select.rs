// Spec reference: specs/066_navigation_reorganization_category_circuit_filter_and_tiered_career_championships.md
use crate::profile::{ModuleCareerProgress, PlayerProfile, ProfileCareerStats};
use crate::render::color::Palette;
use crate::series::{ChampionshipManager, ChampionshipSession, SeriesDefinition};
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use cabinet::ui::{Accordion, AccordionItem, ScreenFooter};
use macroquad::prelude::*;
use std::collections::HashMap;

/// Earned podium trophy level for completed championships (Spec 066).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PodiumTrophy {
    Gold,
    Silver,
    Bronze,
}

impl PodiumTrophy {
    pub fn from_best_finish(pos: u32) -> Option<Self> {
        match pos {
            1 => Some(Self::Gold),
            2 => Some(Self::Silver),
            3 => Some(Self::Bronze),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Gold => "GOLD",
            Self::Silver => "SILVER",
            Self::Bronze => "BRONZE",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Gold => "🏆 GOLD",
            Self::Silver => "🥈 SILVER",
            Self::Bronze => "🥉 BRONZE",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Gold => Palette::NEON_GOLD,
            Self::Silver => Color::new(0.75, 0.82, 0.90, 1.0),
            Self::Bronze => Color::new(0.80, 0.50, 0.20, 1.0),
        }
    }
}

/// Lifecycle status tag for championship cards in Career Mode (Spec 066).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChampionshipCardStatus {
    New,
    InProgress,
    Completed,
}

/// Unified championship card for career tournament selection (Spec 066).
#[derive(Debug, Clone, PartialEq)]
pub struct CareerSelectChampionshipCard {
    pub series_id: String,
    pub series_name: String,
    pub module_id: String,
    pub module_title: String,
    pub tier: u32,
    pub tier_name: String,
    pub current_round: usize,
    pub total_rounds: usize,
    pub status: ChampionshipCardStatus,
    pub trophy: Option<PodiumTrophy>,
    pub player_points: u32,
    pub player_rank: usize,
    pub total_drivers: usize,
    pub next_track_id: String,
    pub next_track_name: String,
    pub accent_color: Color,
}

impl CareerSelectChampionshipCard {
    pub fn is_active(&self) -> bool {
        self.status == ChampionshipCardStatus::InProgress
    }

    pub fn is_completed(&self) -> bool {
        self.status == ChampionshipCardStatus::Completed
    }
}

impl Default for CareerSelectChampionshipCard {
    fn default() -> Self {
        Self {
            series_id: String::new(),
            series_name: String::new(),
            module_id: String::new(),
            module_title: String::new(),
            tier: 0,
            tier_name: String::new(),
            current_round: 0,
            total_rounds: 0,
            status: ChampionshipCardStatus::New,
            trophy: None,
            player_points: 0,
            player_rank: 0,
            total_drivers: 0,
            next_track_id: String::new(),
            next_track_name: String::new(),
            accent_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

/// Legacy discipline card descriptor retained for backwards compatibility.
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

/// Returns display title and signature neon accent color for a motorsport module.
pub fn module_meta_for(module_id: &str) -> (&'static str, Color) {
    match module_id {
        "gt" | "gt_challenge" => ("GT WORLD CHALLENGE", Palette::RED),
        "nascar" => ("NASCAR CUP SERIES", Palette::YELLOW),
        "rally" => ("RALLYCROSS WORLD CUP", Palette::NEON_GOLD),
        "kart" => ("KARTING WORLD CUP", Palette::NEON_GREEN),
        "extreme_offroad" | "offroad" => ("EXTREME OFF-ROAD", Color::new(1.0, 0.40, 0.05, 1.0)),
        "classic" => ("CLASSIC MOTORSPORT", Palette::NEON_CYAN),
        _ => ("MOTORSPORT CHAMPIONSHIP", Palette::NEON_CYAN),
    }
}

/// Formatted tier description label.
pub fn tier_name(tier: u32) -> String {
    match tier {
        1 => "TIER 1 (ROOKIE)".to_string(),
        2 => "TIER 2 (AMATEUR)".to_string(),
        3 => "TIER 3 (CONTENDER)".to_string(),
        4 => "TIER 4 (PRO)".to_string(),
        _ => "TIER 5 (LEGEND)".to_string(),
    }
}

/// Short discipline label that fits row badges.
pub fn module_badge_label(module_id: &str) -> String {
    match module_id {
        "extreme_offroad" | "offroad" => "OFF-ROAD".to_string(),
        "autocross" => "AUTOCROSS".to_string(),
        other => other.to_uppercase(),
    }
}

/// Builds the complete list of tier-gated and completed championship cards (Spec 066).
/// - Tier 1 championships are always visible.
/// - Higher tiers (2..=5) appear once unlocked by driver level or previously played.
/// - In-progress championships appear first, followed by available new championships, followed by completed replayable championships.
pub fn build_tiered_career_championship_cards(
    champ_manager: &ChampionshipManager,
    module_progress_map: &HashMap<String, ModuleCareerProgress>,
    active_progress: &ModuleCareerProgress,
    current_session: Option<&ChampionshipSession>,
) -> Vec<CareerSelectChampionshipCard> {
    let mut in_progress = Vec::new();
    let mut available_new = Vec::new();
    let mut completed = Vec::new();

    for def in champ_manager.all_series() {
        let mod_id = &def.series.module_id;
        let dummy_prog;
        let mod_progress = if let Some(p) = module_progress_map.get(mod_id) {
            p
        } else if active_progress.module_id == *mod_id {
            active_progress
        } else {
            dummy_prog = ModuleCareerProgress::default_for_module(1, mod_id);
            &dummy_prog
        };

        let unlocked_tier = mod_progress.level.max(1);

        // Check if completed in progress records
        let record = mod_progress
            .championships_completed
            .get(&def.series.id)
            .or_else(|| mod_progress.championships_completed.get(&def.series.name))
            .or_else(|| {
                mod_progress.championships_completed.values().find(|c| {
                    c.championship_id.eq_ignore_ascii_case(&def.series.id)
                        || c.championship_id.eq_ignore_ascii_case(&def.series.name)
                })
            });
        let is_completed = record.is_some();

        // Check if active in progress or current_session
        let matches_session = |s: &ChampionshipSession| -> bool {
            !s.is_completed
                && (s.name.eq_ignore_ascii_case(&def.series.name)
                    || s.name.eq_ignore_ascii_case(&def.series.id)
                    || (s.tier == def.series.tier
                        && s.name.to_lowercase().contains(&def.series.module_id))
                    || (s.tier == def.series.tier
                        && def
                            .series
                            .name
                            .to_lowercase()
                            .contains(&s.name.to_lowercase())))
        };
        let active_session = mod_progress
            .active_championship
            .as_ref()
            .filter(|s| matches_session(s))
            .or_else(|| current_session.filter(|s| matches_session(s)));
        let is_active = active_session.is_some();

        // Visibility Rule: Tier 1 is always visible. Higher/bonus tiers visible only if unlocked or played.
        let is_visible = if def.series.tier == 0 {
            unlocked_tier >= 5 || is_completed || is_active
        } else if def.series.tier == 1 {
            true
        } else {
            def.series.tier <= unlocked_tier || is_completed || is_active
        };

        if !is_visible {
            continue;
        }

        let (module_title, accent_color) = module_meta_for(mod_id);
        let tier_str = tier_name(def.series.tier);
        let total_rounds = def.rounds.len().max(1);
        let total_drivers = def.drivers.len().max(8);
        let default_track_id = def
            .rounds
            .first()
            .map(|r| r.track_id.clone())
            .unwrap_or_else(|| "monza".to_string());
        let default_track_name = default_track_id.replace('_', " ").to_uppercase();

        let (
            status,
            trophy,
            current_round,
            player_points,
            player_rank,
            next_track_id,
            next_track_name,
        ) = if let Some(champ) = active_session {
            let cur_rnd = champ.current_round.min(total_rounds);
            let standing = champ.standings.iter().find(|s| s.driver_id == "player");
            let pts = standing.map(|s| s.points).unwrap_or(0);
            let rank = champ
                .standings
                .iter()
                .position(|s| s.driver_id == "player")
                .map(|p| p + 1)
                .unwrap_or(1);
            let next_tid = champ
                .current_track_id()
                .unwrap_or(&default_track_id)
                .to_string();
            let next_tname = next_tid.replace('_', " ").to_uppercase();
            (
                ChampionshipCardStatus::InProgress,
                None,
                cur_rnd,
                pts,
                rank,
                next_tid,
                next_tname,
            )
        } else if let Some(rec) = record {
            let trop = PodiumTrophy::from_best_finish(rec.best_finish);
            (
                ChampionshipCardStatus::Completed,
                trop,
                0,
                rec.highest_points,
                rec.best_finish as usize,
                default_track_id,
                default_track_name,
            )
        } else {
            (
                ChampionshipCardStatus::New,
                None,
                0,
                0,
                0,
                default_track_id,
                default_track_name,
            )
        };

        let card = CareerSelectChampionshipCard {
            series_id: def.series.id.clone(),
            series_name: def.series.name.clone(),
            module_id: mod_id.clone(),
            module_title: module_title.to_string(),
            tier: def.series.tier,
            tier_name: tier_str,
            current_round,
            total_rounds,
            status,
            trophy,
            player_points,
            player_rank,
            total_drivers,
            next_track_id,
            next_track_name,
            accent_color,
        };

        match status {
            ChampionshipCardStatus::InProgress => in_progress.push(card),
            ChampionshipCardStatus::New => available_new.push(card),
            ChampionshipCardStatus::Completed => completed.push(card),
        }
    }

    // Sort order:
    // in_progress: sorted by tier, then series_name
    in_progress.sort_by(|a, b| {
        a.tier
            .cmp(&b.tier)
            .then_with(|| a.series_name.cmp(&b.series_name))
    });
    // available_new: sorted by tier, then module_id, then series_name
    available_new.sort_by(|a, b| {
        a.tier
            .cmp(&b.tier)
            .then_with(|| a.module_id.cmp(&b.module_id))
            .then_with(|| a.series_name.cmp(&b.series_name))
    });
    // completed: sorted by tier, then module_id, then series_name
    completed.sort_by(|a, b| {
        a.tier
            .cmp(&b.tier)
            .then_with(|| a.module_id.cmp(&b.module_id))
            .then_with(|| a.series_name.cmp(&b.series_name))
    });

    let mut result = in_progress;
    result.extend(available_new);
    result.extend(completed);
    result
}

/// Helper building cards directly from a PlayerProfile (Spec 066).
pub fn build_tiered_career_championship_cards_for_profile(
    champ_manager: &ChampionshipManager,
    profile: &PlayerProfile,
    current_session: Option<&ChampionshipSession>,
) -> Vec<CareerSelectChampionshipCard> {
    let dummy_map = HashMap::new();
    let default_prog = profile.module_progress("gt");
    build_tiered_career_championship_cards(
        champ_manager,
        &dummy_map,
        &default_prog,
        current_session,
    )
}

/// Discipline metadata descriptor for legacy catalog.
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
    ModalityMeta {
        id: "autocross",
        title: "FIA AUTOCROSS",
        subtitle: "Natural Unpaved Dirt & Buggy Racing",
        accent: Color::new(1.0, 0.45, 0.05, 1.0),
        default_series_name: "FIA Autocross World Series 2026",
    },
];

/// Legacy card builder retained for backwards compatibility with tests.
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
        let tier_name = tier_name(tier);

        let session_opt = prog
            .active_championship
            .as_ref()
            .filter(|s| !s.is_completed)
            .or_else(|| {
                current_session.filter(|s| {
                    !s.is_completed
                        && (champ_manager.series.values().any(|def| {
                            def.series.module_id == meta.id
                                && (def.series.name.eq_ignore_ascii_case(&s.name)
                                    || def.series.id.eq_ignore_ascii_case(&s.name))
                        }) || s.name.to_lowercase().contains(meta.id)
                            || meta.id.contains(&s.name.to_lowercase())
                            || s.name.eq_ignore_ascii_case(meta.default_series_name))
                })
            });

        let series_def: Option<&SeriesDefinition> = champ_manager
            .series
            .values()
            .find(|s| s.series.module_id == meta.id && s.series.tier == tier)
            .or_else(|| {
                champ_manager
                    .series
                    .values()
                    .find(|s| s.series.module_id == meta.id)
            });

        if let Some(champ) = session_opt {
            let total_rounds = champ.track_ids.len().max(1);
            let current_round = champ.current_round.min(total_rounds);
            let player_standing = champ.standings.iter().find(|s| s.driver_id == "player");
            let player_points = player_standing.map(|s| s.points).unwrap_or(0);
            let player_rank = champ
                .standings
                .iter()
                .position(|s| s.driver_id == "player")
                .map(|p| p + 1)
                .unwrap_or(1);
            let total_drivers = champ.standings.len();

            let next_track_id = champ
                .current_track_id()
                .unwrap_or(
                    champ
                        .track_ids
                        .first()
                        .map(|s| s.as_str())
                        .unwrap_or("monza"),
                )
                .to_string();
            let next_track_name = next_track_id.replace('_', " ").to_uppercase();
            let car_model_id = prog.unlocked_cars.first().cloned();

            active_cards.push(CareerSelectCard {
                module_id: meta.id.to_string(),
                series_id: series_def
                    .map(|d| d.series.id.clone())
                    .unwrap_or_else(|| meta.id.to_string()),
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
            let next_track_id = series_def
                .and_then(|d| d.rounds.first().map(|r| r.track_id.as_str()))
                .unwrap_or("monza")
                .to_string();
            let next_track_name = next_track_id.replace('_', " ").to_uppercase();
            let series_name = series_def
                .map(|d| d.series.name.clone())
                .unwrap_or_else(|| meta.default_series_name.to_string());
            let car_model_id = prog.unlocked_cars.first().cloned();

            available_cards.push(CareerSelectCard {
                module_id: meta.id.to_string(),
                series_id: series_def
                    .map(|d| d.series.id.clone())
                    .unwrap_or_else(|| meta.id.to_string()),
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

/// Computes the bounding rect for a career select card with responsive auto-scrolling viewport clamping.
pub fn career_select_card_rect(index: usize, selected_idx: usize, sw: f32, sh: f32) -> Rect {
    let scaler = UiScaler::new(sw, sh);
    let top_y = scaler.s(80.0);
    let bottom_limit = sh - scaler.s(55.0);
    let card_width = (sw * 0.90).clamp(scaler.s(640.0), scaler.s(1180.0));
    let card_x = (sw - card_width) * 0.5;

    let collapsed_h = scaler.s(52.0);
    let expanded_h = scaler.s(148.0);
    let card_gap = scaler.s(8.0);

    let raw_selected_y = top_y + (selected_idx as f32) * (collapsed_h + card_gap);
    let selected_bottom = raw_selected_y + expanded_h;

    let scroll_y = if selected_bottom > bottom_limit {
        selected_bottom - bottom_limit
    } else {
        0.0
    };

    let raw_y = if index < selected_idx {
        top_y + (index as f32) * (collapsed_h + card_gap)
    } else if index == selected_idx {
        raw_selected_y
    } else {
        raw_selected_y
            + expanded_h
            + card_gap
            + ((index - selected_idx - 1) as f32) * (collapsed_h + card_gap)
    };

    let final_y = raw_y - scroll_y;
    let height = if index == selected_idx {
        expanded_h
    } else {
        collapsed_h
    };

    Rect::new(card_x, final_y, card_width, height)
}

/// Renders the multi-career selection screen using an Expandable Accordion layout.
pub fn render_career_select_screen(
    fonts: &Fonts,
    cards: &[CareerSelectChampionshipCard],
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
        "CAREER MODE // MOTORSPORT CHAMPIONSHIP SELECTOR",
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

    // Active & Completed Progress Pill Badge
    let pill_x = scaler.s(540.0);
    let pill_w = scaler.s(240.0);
    let pill_h = scaler.s(24.0);
    let pill_y = header_y - scaler.s(18.0);

    let active_count = cards
        .iter()
        .filter(|c| c.status == ChampionshipCardStatus::InProgress)
        .count();
    let completed_count = cards
        .iter()
        .filter(|c| c.status == ChampionshipCardStatus::Completed)
        .count();

    let (pill_bg, pill_fg) = if active_count > 0 {
        (Color::new(0.95, 0.70, 0.05, 0.20), Palette::NEON_GOLD)
    } else {
        (Color::new(0.15, 0.22, 0.32, 0.35), Palette::NEON_CYAN)
    };
    draw_rectangle(pill_x, pill_y, pill_w, pill_h, pill_bg);
    draw_rectangle_lines(pill_x, pill_y, pill_w, pill_h, 1.2, pill_fg);
    let pill_text = format!(
        "ACTIVE: {}  •  COMPLETED: {}/{}",
        active_count,
        completed_count,
        cards.len()
    );
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
    scaler.draw_glass_card(
        badge_x,
        badge_y,
        badge_w,
        badge_h,
        Color::new(0.08, 0.10, 0.15, 0.85),
        Palette::NEON_CYAN,
        1.0,
    );

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

    // Viewport boundaries for card clipping
    let top_clip = scaler.s(60.0);
    let bottom_clip = sh - scaler.s(45.0);

    // =========================================================================
    // ACCORDION ROWS LIST: Collapsed rows by default, Expanded selected row.
    // Uses the platform Accordion in single-expand mode; it replaces the manual
    // collapsed(52px)/expanded(148px) height interpolation and scroll viewport offset.
    // =========================================================================
    let top_y = scaler.s(80.0);
    let bottom_limit = sh - scaler.s(55.0);
    let card_width = (sw * 0.90).clamp(scaler.s(640.0), scaler.s(1180.0));
    let card_x = (sw - card_width) * 0.5;

    let accordion_items: Vec<AccordionItem<CareerSelectChampionshipCard>> = cards
        .iter()
        .map(|card| {
            let mut item = AccordionItem::new(
                card.series_id.clone(),
                card.series_name.clone(),
                card.accent_color,
            );
            item.subtitle = Some(card.module_title.clone());
            item.tag = Some(module_badge_label(&card.module_id));
            item.data = card.clone();
            item
        })
        .collect();
    let accordion = Accordion::new(
        accordion_items,
        scaler.s(52.0),
        scaler.s(148.0),
        scaler.s(8.0),
    )
    .with_selected(selected_idx);
    let (row_rects, _scroll_y) = accordion.compute_rects(card_x, card_width, top_y, bottom_limit);

    for (idx, rect, _is_expanded) in row_rects {
        if rect.y + rect.h < top_clip || rect.y > bottom_clip {
            continue;
        }

        let card = &cards[idx];
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
            let tag_w = scaler.s(72.0);
            let tag_h = scaler.s(22.0);
            draw_rectangle(
                tag_x,
                tag_y,
                tag_w,
                tag_h,
                Color::new(
                    card.accent_color.r,
                    card.accent_color.g,
                    card.accent_color.b,
                    0.20,
                ),
            );
            draw_rectangle_lines(tag_x, tag_y, tag_w, tag_h, 1.0, card.accent_color);
            fonts.draw_ui_bold_centered(
                &module_badge_label(&card.module_id),
                tag_x + tag_w * 0.5,
                tag_y + scaler.s(15.0),
                scaler.font_s(11.0),
                card.accent_color,
            );

            let title_x = tag_x + tag_w + scaler.s(14.0);
            fonts.draw_display(
                &card.series_name,
                title_x,
                tag_y + scaler.s(18.0),
                scaler.font_s(17.0),
                Palette::WHITE,
            );

            // Top Right Status Badge Pill
            let pill_w = scaler.s(170.0);
            let pill_h = scaler.s(22.0);
            let pill_x = rect.x + rect.w - scaler.s(16.0) - pill_w;
            let (p_bg, p_border, p_fg, p_text) = match card.status {
                ChampionshipCardStatus::InProgress => (
                    Color::new(0.95, 0.70, 0.05, 0.20),
                    Palette::NEON_GOLD,
                    Palette::NEON_GOLD,
                    format!(
                        "ROUND {} / {} ACTIVE",
                        card.current_round + 1,
                        card.total_rounds
                    ),
                ),
                ChampionshipCardStatus::Completed => {
                    let trophy_str = if let Some(t) = card.trophy {
                        format!("{} • {} PTS", t.icon(), card.player_points)
                    } else {
                        "COMPLETED".to_string()
                    };
                    (
                        Color::new(0.10, 0.75, 0.35, 0.20),
                        Palette::NEON_GREEN,
                        Palette::NEON_GREEN,
                        trophy_str,
                    )
                }
                ChampionshipCardStatus::New => (
                    Color::new(0.15, 0.22, 0.32, 0.35),
                    Palette::NEON_CYAN,
                    Palette::NEON_CYAN,
                    "READY TO COMMENCE".to_string(),
                ),
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
            draw_line(
                tag_x,
                div_y,
                rect.x + rect.w - scaler.s(16.0),
                div_y,
                1.0,
                Color::new(0.20, 0.24, 0.32, 0.50),
            );

            // Middle Section: Left Details & Right Telemetry
            let body_y = div_y + scaler.s(16.0);

            // Left side: Subtitle lore and tier info
            let spec_line = format!(
                "{}  •  {} Total Rounds  •  {} Drivers",
                card.tier_name, card.total_rounds, card.total_drivers
            );
            fonts.draw_ui_regular(
                &spec_line,
                tag_x,
                body_y,
                scaler.font_s(12.0),
                Palette::UI_TEXT_MUTED,
            );

            let status_line = match card.status {
                ChampionshipCardStatus::InProgress => {
                    format!(
                        "CURRENT STANDING: P{} / {}  •  {} PTS",
                        card.player_rank, card.total_drivers, card.player_points
                    )
                }
                ChampionshipCardStatus::Completed => {
                    format!(
                        "HISTORIC RESULT: {} ({} PTS)  •  REPLAY AVAILABLE",
                        card.trophy.map(|t| t.label()).unwrap_or("FINISHED"),
                        card.player_points
                    )
                }
                ChampionshipCardStatus::New => {
                    format!(
                        "ROSTER: {} Drivers  •  Standard Tournament Scoring",
                        card.total_drivers
                    )
                }
            };
            fonts.draw_ui_bold(
                &status_line,
                tag_x,
                body_y + scaler.s(18.0),
                scaler.font_s(11.5),
                match card.status {
                    ChampionshipCardStatus::InProgress => Palette::NEON_GOLD,
                    ChampionshipCardStatus::Completed => Palette::NEON_GREEN,
                    ChampionshipCardStatus::New => Palette::NEON_CYAN,
                },
            );

            // Right side: Standings & Next Venue
            let mid_x = rect.x + rect.w * 0.50;
            match card.status {
                ChampionshipCardStatus::InProgress => {
                    let next_str = format!("NEXT EVENT: {}", card.next_track_name);
                    fonts.draw_ui_bold(
                        &next_str,
                        mid_x,
                        body_y,
                        scaler.font_s(12.0),
                        Palette::WHITE,
                    );
                    fonts.draw_ui_regular(
                        "PRESS ENTER TO RESUME CAMPAIGN",
                        mid_x,
                        body_y + scaler.s(18.0),
                        scaler.font_s(11.0),
                        Palette::UI_TEXT_MUTED,
                    );
                }
                ChampionshipCardStatus::Completed => {
                    let venue_str = format!("CIRCUIT ROTATION: {} (R1)", card.next_track_name);
                    fonts.draw_ui_bold(
                        &venue_str,
                        mid_x,
                        body_y,
                        scaler.font_s(12.0),
                        Palette::WHITE,
                    );
                    fonts.draw_ui_regular(
                        "LIFETIME TROPHY PRESERVED ON REPLAY",
                        mid_x,
                        body_y + scaler.s(18.0),
                        scaler.font_s(11.0),
                        Palette::NEON_GOLD,
                    );
                }
                ChampionshipCardStatus::New => {
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
            }

            // Bottom Action Area
            let btn_w = scaler.s(210.0);
            let btn_h = scaler.s(32.0);
            let btn_x = rect.x + rect.w - scaler.s(16.0) - btn_w;
            let btn_y = rect.y + rect.h - btn_h - scaler.s(12.0);

            let (btn_bg, btn_fg, btn_text) = match card.status {
                ChampionshipCardStatus::InProgress => {
                    (card.accent_color, Palette::BLACK, "▶  RESUME CAREER")
                }
                ChampionshipCardStatus::Completed => {
                    (Palette::NEON_GOLD, Palette::BLACK, "↻  REPLAY CHAMPIONSHIP")
                }
                ChampionshipCardStatus::New => {
                    (Palette::NEON_CYAN, Palette::BLACK, "▶  START CAREER")
                }
            };
            draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_bg);
            fonts.draw_ui_bold_centered(
                btn_text,
                btn_x + btn_w * 0.5,
                btn_y + scaler.s(21.0),
                scaler.font_s(13.0),
                btn_fg,
            );

            if card.status == ChampionshipCardStatus::InProgress {
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
            let bg_color = match card.status {
                ChampionshipCardStatus::InProgress => Color::new(0.06, 0.08, 0.13, 0.80),
                ChampionshipCardStatus::Completed => Color::new(0.04, 0.08, 0.06, 0.75),
                ChampionshipCardStatus::New => Color::new(0.05, 0.06, 0.09, 0.70),
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
            draw_rectangle_lines(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                1.0,
                Color::new(0.18, 0.22, 0.28, 0.45),
            );

            // Left Modality Accent Strip
            let strip_w = scaler.s(5.0);
            draw_rectangle(rect.x, rect.y, strip_w, rect.h, card.accent_color);

            // Modality Tag Badge
            let tag_x = rect.x + scaler.s(16.0);
            let tag_h = scaler.s(22.0);
            let tag_y = rect.y + (rect.h - tag_h) * 0.5;
            let tag_w = scaler.s(68.0);
            draw_rectangle(
                tag_x,
                tag_y,
                tag_w,
                tag_h,
                Color::new(
                    card.accent_color.r,
                    card.accent_color.g,
                    card.accent_color.b,
                    0.16,
                ),
            );
            draw_rectangle_lines(tag_x, tag_y, tag_w, tag_h, 1.0, card.accent_color);

            let tag_text = module_badge_label(&card.module_id);
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
                &card.series_name,
                title_x,
                rect.y + scaler.s(32.0),
                scaler.font_s(15.0),
                Color::new(0.85, 0.88, 0.92, 1.0),
            );

            // Right side status pill & expand indicator
            let right_x = rect.x + rect.w - scaler.s(16.0);
            let (status_text, status_col) = match card.status {
                ChampionshipCardStatus::InProgress => (
                    format!(
                        "ROUND {}/{}  •  P{} ({} PTS)",
                        card.current_round + 1,
                        card.total_rounds,
                        card.player_rank,
                        card.player_points
                    ),
                    Palette::NEON_GOLD,
                ),
                ChampionshipCardStatus::Completed => {
                    let t_icon = card.trophy.map(|t| t.icon()).unwrap_or("🏆 COMPLETED");
                    (
                        format!("{}  •  {} PTS", t_icon, card.player_points),
                        Palette::NEON_GREEN,
                    )
                }
                ChampionshipCardStatus::New => (
                    format!("{}  •  {} ROUNDS", card.tier_name, card.total_rounds),
                    Palette::UI_TEXT_MUTED,
                ),
            };

            let chip_w = scaler.s(220.0);
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

    // Bottom Navigation Bar (platform ScreenFooter)
    let nav_fs = scaler.font_s(11.5);
    let footer = {
        let mut footer = ScreenFooter::new(0.0, sh - scaler.s(42.0), sw, scaler.s(42.0));
        footer.add_prompt("W/S", "Select / Expand");
        footer.add_prompt("ENTER", "Launch / Resume / Replay");
        footer.add_prompt("R", "Reset Season");
        footer.add_prompt("ESC", "Back");
        footer
    };
    footer.render_frame();
    let footer_bounds = footer.bounds();
    let prompt_w = footer_bounds.w / footer.prompts.len().max(1) as f32;
    for (i, prompt) in footer.prompts.iter().enumerate() {
        let text = fonts.fit_ui_bold(
            &format!("[{}] {}", prompt.badge, prompt.label),
            nav_fs,
            prompt_w - scaler.s(8.0),
        );
        fonts.draw_ui_bold_centered(
            &text,
            footer_bounds.x + (i as f32 + 0.5) * prompt_w,
            footer_bounds.y + footer_bounds.h * 0.65,
            nav_fs,
            Palette::WHITE,
        );
    }
}
