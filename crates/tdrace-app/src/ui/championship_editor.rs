use macroquad::color::Color;
use macroquad::input::{
    is_key_down, is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton,
};
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::draw_rectangle;

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::render::color::Palette;
use crate::tournament::format::{
    ChampionshipDefinition, DriverConfig, RoundConfig,
};
use crate::ui::menu::TrackChoice;

const COLOR_LIGHT_GRAY: Color = Color::new(0.80, 0.82, 0.85, 1.0);
const COLOR_DARK_GRAY: Color = Color::new(0.45, 0.48, 0.52, 1.0);
const COLOR_TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

/// Tab views available inside the Championship Editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChampionshipEditorTab {
    #[default]
    Rules,
    Calendar,
    Grid,
    Export,
}

impl ChampionshipEditorTab {
    pub const ALL: [Self; 4] = [Self::Rules, Self::Calendar, Self::Grid, Self::Export];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Rules => "1. RULES & INFO",
            Self::Calendar => "2. CALENDAR & ROUNDS",
            Self::Grid => "3. DRIVER GRID",
            Self::Export => "4. VALIDATE & EXPORT",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Rules => Self::Calendar,
            Self::Calendar => Self::Grid,
            Self::Grid => Self::Export,
            Self::Export => Self::Rules,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Rules => Self::Export,
            Self::Calendar => Self::Rules,
            Self::Grid => Self::Calendar,
            Self::Export => Self::Grid,
        }
    }
}

/// Modal overlays inside the Championship Editor.
#[derive(Debug, Clone, PartialEq)]
pub enum ChampionshipEditorModal {
    None,
    AddTrack {
        selected_idx: usize,
    },
    SelectCarModel {
        driver_idx: usize,
        selected_idx: usize,
    },
    EditTextField {
        target: TextEditTarget,
        value: String,
        cursor_timer: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEditTarget {
    Id,
    Name,
    Description,
    DriverName(usize),
    DriverTeam(usize),
}

/// Dispatched actions from Championship Editor user input.
#[derive(Debug, Clone, PartialEq)]
pub enum ChampionshipEditorAction {
    None,
    Exit,
    SaveUser,
    SavePreset,
    LaunchTestCup(ChampionshipDefinition),
}

/// Active interactive state of the Championship Editor.
#[derive(Debug, Clone, PartialEq)]
pub struct ChampionshipEditorState {
    pub active_tab: ChampionshipEditorTab,
    pub def: ChampionshipDefinition,
    pub rules_field_idx: usize,
    pub selected_round_idx: usize,
    pub selected_driver_idx: usize,
    pub modal: ChampionshipEditorModal,
    pub status_msg: Option<(String, f32)>,
}

impl Default for ChampionshipEditorState {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ChampionshipEditorState {
    pub fn new(initial_def: Option<ChampionshipDefinition>) -> Self {
        let mut def = initial_def.unwrap_or_else(ChampionshipDefinition::default);
        if def.drivers.is_empty() {
            autofill_grid_for_module(&mut def);
        }
        Self {
            active_tab: ChampionshipEditorTab::Rules,
            def,
            rules_field_idx: 0,
            selected_round_idx: 0,
            selected_driver_idx: 0,
            modal: ChampionshipEditorModal::None,
            status_msg: None,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, duration_secs: f32) {
        self.status_msg = Some((msg.into(), duration_secs));
    }

    pub fn update(&mut self, dt: f32) {
        if let Some((_, ref mut timer)) = self.status_msg {
            *timer -= dt;
            if *timer <= 0.0 {
                self.status_msg = None;
            }
        }
        if let ChampionshipEditorModal::EditTextField {
            ref mut cursor_timer,
            ..
        } = self.modal
        {
            *cursor_timer += dt;
        }
    }
}

/// Autofills realistic driver roster and car models matching the championship's module and tier.
pub fn autofill_grid_for_module(def: &mut ChampionshipDefinition) {
    let module = def.championship.module_id.to_lowercase();
    let tier = def.championship.tier.clamp(1, 5) as u8;

    let available_models = crate::catalog::get_models_for_module_and_tier(&module, tier);
    let default_model_id = available_models
        .first()
        .map(|m| m.id.to_string())
        .unwrap_or_else(|| "gt_toyota_supra_gt4".to_string());

    let (team_prefix, bot_names) = match module.as_str() {
        "nascar" => (
            "Apex Stock Car",
            vec![
                ("dale_vance", "Dale 'The Intimidator' Vance", "Richard Childress Racing"),
                ("chase_gordon", "Chase 'Rainbow' Gordon", "Hendrick Motorsports"),
                ("richard_pettyfield", "Richard 'The King' Pettyfield", "Petty Enterprises"),
                ("rowdy_busch", "Rowdy 'Wild Thing' Busch", "Joe Gibbs Racing"),
                ("jimmie_johnson", "Jimmie 'Seven-Time' Johnson", "Hendrick Motorsports"),
                ("tony_stewart", "Tony 'Smoke' Stewart", "Stewart-Haas Racing"),
                ("bobby_allison", "Bobby 'Alabama' Allison", "Alabama Gang"),
            ],
        ),
        "rally" => (
            "Apex Rally Team",
            vec![
                ("johan_k", "Johan Kristoffersson", "KMS Volkswagen"),
                ("timmy_h", "Timmy Hansen", "Hansen Motorsport"),
                ("mattias_e", "Mattias Ekström", "EKS RX"),
                ("petter_s", "Petter Solberg", "PSRX Volkswagen"),
                ("andreas_b", "Andreas Bakkerud", "Monster Energy RX"),
                ("niclas_g", "Niclas Grönholm", "GRX Taneco"),
                ("kevin_h", "Kevin Hansen", "Hansen Motorsport"),
            ],
        ),
        "kart" => (
            "Apex Kart Racing",
            vec![
                ("marco_a", "Marco Armani", "Tony Kart Racing"),
                ("lucas_v", "Lucas Vance", "CRG Factory Team"),
                ("alex_r", "Alex Rossi", "Birel ART"),
                ("sofia_l", "Sofia Lind", "Kosmic Racing"),
                ("finn_k", "Finn Korhonen", "Sodi Kart"),
                ("leo_d", "Leo Dupont", "Energy Corse"),
                ("mateo_s", "Mateo Silva", "Parolin Motorsport"),
            ],
        ),
        "extreme_offroad" => (
            "Sand Rail Dynamics",
            vec![
                ("wyatt_c", "Wyatt 'Dust Devil' Cole", "Mojave Sandworks"),
                ("jaxson_r", "Jaxson 'Baja King' Rivera", "Baja Trophy Racing"),
                ("astrid_l", "Astrid 'Ice Queen' Lindholm", "Nordic Glacier Works"),
                ("bubba_b", "Bubba 'Mud Slinger' Beauregard", "Bayou Heavy Traction"),
                ("travis_m", "Travis 'Nitro' McGrath", "Redline Freestyle"),
                ("roxie_v", "Roxie 'Rock Hound' Vance", "Canyon Crawler Team"),
                ("sven_l", "Sven 'Blizzard' Lindqvist", "Arctic Circle Rally"),
            ],
        ),
        _ => (
            "Apex GT Racing",
            vec![
                ("max_hunter", "Max Hunter", "Red Bull GT"),
                ("charles_l", "Charles Laurent", "Scuderia GT"),
                ("lewis_v", "Lewis Vance", "Scuderia GT"),
                ("fernando_t", "Fernando Toro", "Aston GT"),
                ("george_s", "George Speed", "Mercedes-AMG GT"),
                ("lando_v", "Lando Vance", "McLaren GT"),
                ("oscar_r", "Oscar Rocket", "McLaren GT"),
            ],
        ),
    };

    let mut drivers = Vec::new();
    // 1. Player
    drivers.push(DriverConfig {
        id: "player".to_string(),
        name: "Player".to_string(),
        team: team_prefix.to_string(),
        is_player: true,
        car_model_id: Some(default_model_id.clone()),
        country: Some("ESP".to_string()),
        ai_character: None,
        livery_idx: Some(0),
    });

    // 2. Bots
    for (idx, (id, name, team)) in bot_names.into_iter().enumerate() {
        let model = available_models
            .get(idx % available_models.len().max(1))
            .map(|m| m.id.to_string())
            .unwrap_or_else(|| default_model_id.clone());

        drivers.push(DriverConfig {
            id: id.to_string(),
            name: name.to_string(),
            team: team.to_string(),
            is_player: false,
            car_model_id: Some(model),
            country: Some("INT".to_string()),
            ai_character: Some("standard".to_string()),
            livery_idx: Some((idx + 1) as u8),
        });
    }

    def.drivers = drivers;
}

/// Main rendering entrypoint for the Championship Editor studio.
pub fn render_championship_editor(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &ChampionshipEditorState,
    all_tracks: &[TrackChoice],
    is_dev: bool,
) {
    let sw = screen_width();
    let sh = screen_height();

    // Backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 1.0));

    // Header Bar
    let header_h = scaler.s(52.0);
    draw_rectangle(0.0, 0.0, sw, header_h, Color::new(0.08, 0.10, 0.16, 0.95));
    scaler.draw_glass_card(0.0, header_h - 1.0, sw, 1.0, COLOR_TRANSPARENT, Palette::UI_CARD_BORDER, 1.0);

    // Title badge
    let dev_tag = if is_dev { " • [DEV WORKBENCH]" } else { "" };
    let title_str = format!("CHAMPIONSHIP STUDIO{}", dev_tag);
    fonts.draw_ui_bold(&title_str, scaler.s(20.0), scaler.s(22.0), scaler.font_s(14.0), Palette::NEON_GOLD);
    fonts.draw_ui_regular(
        &format!("Editing: {}.toml", state.def.championship.id),
        scaler.s(20.0),
        scaler.s(40.0),
        scaler.font_s(10.5),
        Palette::UI_TEXT_MUTED,
    );

    // Tab Bar (Centered)
    let tab_w = scaler.s(150.0);
    let tab_gap = scaler.s(8.0);
    let total_tabs_w = tab_w * 4.0 + tab_gap * 3.0;
    let tabs_x = (sw - total_tabs_w) * 0.5;
    let tabs_y = scaler.s(10.0);
    let tab_h = scaler.s(32.0);

    for (idx, tab) in ChampionshipEditorTab::ALL.iter().enumerate() {
        let x = tabs_x + (idx as f32) * (tab_w + tab_gap);
        let is_sel = state.active_tab == *tab;

        let bg = if is_sel {
            Color::new(0.14, 0.20, 0.35, 0.90)
        } else {
            Color::new(0.06, 0.08, 0.12, 0.60)
        };
        let border = if is_sel {
            Palette::NEON_CYAN
        } else {
            Palette::UI_CARD_BORDER
        };
        let text_col = if is_sel { Palette::WHITE } else { Palette::UI_TEXT_MUTED };

        scaler.draw_glass_card(x, tabs_y, tab_w, tab_h, bg, border, if is_sel { 1.6 } else { 1.0 });
        fonts.draw_ui_bold(
            tab.title(),
            x + scaler.s(10.0),
            tabs_y + scaler.s(20.0),
            scaler.font_s(11.0),
            text_col,
        );
    }

    // Status Message Toast
    if let Some((ref msg, _)) = state.status_msg {
        let toast_w = scaler.s(320.0);
        let toast_h = scaler.s(32.0);
        let toast_x = sw - toast_w - scaler.s(20.0);
        let toast_y = scaler.s(10.0);
        scaler.draw_glass_card(
            toast_x,
            toast_y,
            toast_w,
            toast_h,
            Color::new(0.10, 0.28, 0.18, 0.95),
            Palette::NEON_GREEN,
            1.5,
        );
        fonts.draw_ui_bold(msg, toast_x + scaler.s(12.0), toast_y + scaler.s(20.0), scaler.font_s(11.0), Palette::WHITE);
    }

    // Body Workspace (based on active tab)
    let body_y = header_h + scaler.s(14.0);
    let body_h = sh - body_y - scaler.s(44.0);
    let pad_x = scaler.s(24.0);
    let body_w = sw - pad_x * 2.0;

    match state.active_tab {
        ChampionshipEditorTab::Rules => {
            render_tab_rules(fonts, scaler, state, pad_x, body_y, body_w, body_h);
        }
        ChampionshipEditorTab::Calendar => {
            render_tab_calendar(fonts, scaler, state, all_tracks, pad_x, body_y, body_w, body_h);
        }
        ChampionshipEditorTab::Grid => {
            render_tab_grid(fonts, scaler, state, pad_x, body_y, body_w, body_h);
        }
        ChampionshipEditorTab::Export => {
            render_tab_export(fonts, scaler, state, is_dev, pad_x, body_y, body_w, body_h);
        }
    }

    // Footer Help Bar
    render_footer(fonts, scaler, state, is_dev);

    // Modal Overlays
    render_modal(fonts, scaler, state, all_tracks);
}

fn render_tab_rules(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &ChampionshipEditorState,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
) {
    let col_w = (w - scaler.s(24.0)) * 0.5;
    let card_h = scaler.s(46.0);
    let gap = scaler.s(8.0);

    fonts.draw_ui_bold("CHAMPIONSHIP REGULATIONS & METADATA", x, y + scaler.s(14.0), scaler.font_s(13.0), Palette::NEON_GOLD);

    let module_upper = state.def.championship.module_id.to_uppercase();
    let scoring_upper = state.def.scoring.system.to_uppercase();

    let fields = [
        ("Identifier Slug", state.def.championship.id.as_str(), "[Click / Enter to Edit]"),
        ("Display Title", state.def.championship.name.as_str(), "[Click / Enter to Edit]"),
        ("Description", state.def.championship.description.as_str(), "[Click / Enter to Edit]"),
        ("Motorsport Module", module_upper.as_str(), "[Left/Right or Click to Cycle]"),
        ("Career Tier", format!("Tier {}", state.def.championship.tier).leak(), "[Left/Right or Click to Cycle]"),
        ("Default Laps", format!("{} Laps", state.def.championship.laps_per_round).leak(), "[Left/Right or Click +/-]"),
        ("Point System", scoring_upper.as_str(), "[Left/Right to Cycle]"),
        ("Fastest Lap Bonus", if state.def.scoring.fastest_lap_bonus { "ENABLED (+1 pt top 10)" } else { "DISABLED" }, "[Space/Enter to Toggle]"),
        ("Clean Race Bonus", if state.def.scoring.clean_race_bonus { "ENABLED" } else { "DISABLED" }, "[Space/Enter to Toggle]"),
    ];

    let start_y = y + scaler.s(28.0);
    for (idx, (label, val, hint)) in fields.iter().enumerate() {
        let is_left_col = idx < 5;
        let col_x = if is_left_col { x } else { x + col_w + scaler.s(24.0) };
        let row_idx = if is_left_col { idx } else { idx - 5 };
        let card_y = start_y + (row_idx as f32) * (card_h + gap);

        let is_sel = state.rules_field_idx == idx;
        let bg = if is_sel {
            Color::new(0.12, 0.16, 0.26, 0.85)
        } else {
            Color::new(0.06, 0.08, 0.12, 0.65)
        };
        let border = if is_sel { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };

        scaler.draw_glass_card(col_x, card_y, col_w, card_h, bg, border, if is_sel { 1.5 } else { 1.0 });

        fonts.draw_ui_regular(label, col_x + scaler.s(12.0), card_y + scaler.s(14.0), scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
        fonts.draw_ui_bold(val, col_x + scaler.s(12.0), card_y + scaler.s(32.0), scaler.font_s(12.0), if is_sel { Palette::WHITE } else { COLOR_LIGHT_GRAY });
        fonts.draw_ui_regular(hint, col_x + col_w - scaler.s(150.0), card_y + scaler.s(32.0), scaler.font_s(9.0), COLOR_DARK_GRAY);
    }
}

fn render_tab_calendar(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &ChampionshipEditorState,
    all_tracks: &[TrackChoice],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    fonts.draw_ui_bold(
        &format!("CHAMPIONSHIP CALENDAR ({} ROUNDS)", state.def.rounds.len()),
        x,
        y + scaler.s(14.0),
        scaler.font_s(13.0),
        Palette::NEON_GOLD,
    );

    let add_btn_w = scaler.s(150.0);
    let add_btn_h = scaler.s(28.0);
    let add_btn_x = x + w - add_btn_w;
    let add_btn_y = y;
    scaler.draw_glass_card(add_btn_x, add_btn_y, add_btn_w, add_btn_h, Color::new(0.10, 0.25, 0.18, 0.85), Palette::NEON_GREEN, 1.2);
    fonts.draw_ui_bold("+ ADD ROUND", add_btn_x + scaler.s(28.0), add_btn_y + scaler.s(18.0), scaler.font_s(11.0), Palette::WHITE);

    let start_y = y + scaler.s(34.0);
    let row_h = scaler.s(44.0);
    let gap = scaler.s(6.0);

    if state.def.rounds.is_empty() {
        fonts.draw_ui_regular(
            "No rounds in calendar yet. Click '+ ADD ROUND' or press [A] to add circuits.",
            x + scaler.s(20.0),
            start_y + scaler.s(40.0),
            scaler.font_s(12.0),
            Palette::UI_TEXT_MUTED,
        );
        return;
    }

    for (idx, round) in state.def.rounds.iter().enumerate() {
        let card_y = start_y + (idx as f32) * (row_h + gap);
        if card_y + row_h > y + h {
            break;
        }

        let is_sel = state.selected_round_idx == idx;
        let bg = if is_sel {
            Color::new(0.14, 0.18, 0.30, 0.85)
        } else {
            Color::new(0.06, 0.08, 0.12, 0.65)
        };
        let border = if is_sel { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };

        scaler.draw_glass_card(x, card_y, w, row_h, bg, border, if is_sel { 1.5 } else { 1.0 });

        // Round index badge
        fonts.draw_ui_bold(
            &format!("ROUND {:02}", idx + 1),
            x + scaler.s(14.0),
            card_y + scaler.s(26.0),
            scaler.font_s(12.0),
            Palette::NEON_GOLD,
        );

        // Track title
        let track_name = all_tracks
            .iter()
            .find(|t| t.track_id() == round.track_id)
            .map(|t| t.title().to_string())
            .unwrap_or_else(|| round.track_id.clone());

        fonts.draw_ui_bold(&track_name, x + scaler.s(110.0), card_y + scaler.s(26.0), scaler.font_s(12.5), Palette::WHITE);
        fonts.draw_ui_regular(
            &format!("slug: {}", round.track_id),
            x + scaler.s(320.0),
            card_y + scaler.s(26.0),
            scaler.font_s(10.0),
            Palette::UI_TEXT_MUTED,
        );

        // Laps
        let laps_str = format!("{} LAPS", round.laps.unwrap_or(state.def.championship.laps_per_round));
        fonts.draw_ui_bold(&laps_str, x + w - scaler.s(240.0), card_y + scaler.s(26.0), scaler.font_s(11.0), Palette::NEON_CYAN);

        // Actions hint
        fonts.draw_ui_regular(
            "[Up/Down Reorder • [D] Del • [L] Laps]",
            x + w - scaler.s(180.0),
            card_y + scaler.s(26.0),
            scaler.font_s(9.0),
            COLOR_DARK_GRAY,
        );
    }
}

fn render_tab_grid(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &ChampionshipEditorState,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    fonts.draw_ui_bold(
        &format!("GRID ROSTER ({} DRIVERS)", state.def.drivers.len()),
        x,
        y + scaler.s(14.0),
        scaler.font_s(13.0),
        Palette::NEON_GOLD,
    );

    // Autofill Grid Button
    let btn_w = scaler.s(180.0);
    let btn_h = scaler.s(28.0);
    let btn_x = x + w - btn_w;
    let btn_y = y;
    scaler.draw_glass_card(btn_x, btn_y, btn_w, btn_h, Color::new(0.12, 0.22, 0.35, 0.85), Palette::NEON_CYAN, 1.2);
    fonts.draw_ui_bold("⚡ AUTOFILL GRID", btn_x + scaler.s(26.0), btn_y + scaler.s(18.0), scaler.font_s(11.0), Palette::WHITE);

    let start_y = y + scaler.s(34.0);
    let row_h = scaler.s(38.0);
    let gap = scaler.s(5.0);

    for (idx, driver) in state.def.drivers.iter().enumerate() {
        let card_y = start_y + (idx as f32) * (row_h + gap);
        if card_y + row_h > y + h {
            break;
        }

        let is_sel = state.selected_driver_idx == idx;
        let bg = if is_sel {
            Color::new(0.14, 0.18, 0.30, 0.85)
        } else {
            Color::new(0.06, 0.08, 0.12, 0.65)
        };
        let border = if is_sel { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };

        scaler.draw_glass_card(x, card_y, w, row_h, bg, border, if is_sel { 1.5 } else { 1.0 });

        // Slot / Role badge
        let role_col = if driver.is_player { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED };
        let role_str = if driver.is_player { "[PLAYER]" } else { "[AI BOT]" };
        fonts.draw_ui_bold(role_str, x + scaler.s(12.0), card_y + scaler.s(24.0), scaler.font_s(10.5), role_col);

        // Driver name
        fonts.draw_ui_bold(&driver.name, x + scaler.s(85.0), card_y + scaler.s(24.0), scaler.font_s(11.5), Palette::WHITE);

        // Team name
        fonts.draw_ui_regular(&driver.team, x + scaler.s(240.0), card_y + scaler.s(24.0), scaler.font_s(11.0), COLOR_LIGHT_GRAY);

        // Car model
        let model_label = driver
            .car_model_id
            .as_deref()
            .and_then(crate::catalog::find_model_by_id)
            .map(|m| m.name)
            .or(driver.car_model_id.as_deref())
            .unwrap_or("Default Model");
        fonts.draw_ui_bold(model_label, x + scaler.s(450.0), card_y + scaler.s(24.0), scaler.font_s(11.0), Palette::NEON_GOLD);

        // Country
        if let Some(ref c) = driver.country {
            fonts.draw_ui_regular(c, x + w - scaler.s(160.0), card_y + scaler.s(24.0), scaler.font_s(10.0), COLOR_DARK_GRAY);
        }

        // Action hint
        fonts.draw_ui_regular("[Enter: Edit • M: Model]", x + w - scaler.s(110.0), card_y + scaler.s(24.0), scaler.font_s(8.5), COLOR_DARK_GRAY);
    }
}

fn render_tab_export(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &ChampionshipEditorState,
    is_dev: bool,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let col_w = (w - scaler.s(24.0)) * 0.5;

    // Left Column: Validation Diagnostics
    fonts.draw_ui_bold("CHAMPIONSHIP DIAGNOSTICS & STATUS", x, y + scaler.s(14.0), scaler.font_s(13.0), Palette::NEON_GOLD);

    let val_result = state.def.validate();
    let diag_box_y = y + scaler.s(28.0);
    let diag_box_h = scaler.s(180.0);

    let (diag_bg, diag_border) = match val_result {
        Ok(_) => (Color::new(0.08, 0.22, 0.14, 0.80), Palette::NEON_GREEN),
        Err(_) => (Color::new(0.24, 0.08, 0.08, 0.80), Palette::RED),
    };
    scaler.draw_glass_card(x, diag_box_y, col_w, diag_box_h, diag_bg, diag_border, 1.5);

    match val_result {
        Ok(_) => {
            fonts.draw_ui_bold("✓ VALIDATION PASSED", x + scaler.s(16.0), diag_box_y + scaler.s(26.0), scaler.font_s(13.0), Palette::NEON_GREEN);
            fonts.draw_ui_regular(
                "Championship meets all specifications. Ready to save or launch immediately in test drive mode.",
                x + scaler.s(16.0),
                diag_box_y + scaler.s(48.0),
                scaler.font_s(11.0),
                Palette::WHITE,
            );
            fonts.draw_ui_regular(&format!("• Total Rounds: {}", state.def.rounds.len()), x + scaler.s(16.0), diag_box_y + scaler.s(80.0), scaler.font_s(11.0), COLOR_LIGHT_GRAY);
            fonts.draw_ui_regular(&format!("• Grid Competitors: {}", state.def.drivers.len()), x + scaler.s(16.0), diag_box_y + scaler.s(100.0), scaler.font_s(11.0), COLOR_LIGHT_GRAY);
            fonts.draw_ui_regular(&format!("• Scoring System: {}", state.def.scoring.system.to_uppercase()), x + scaler.s(16.0), diag_box_y + scaler.s(120.0), scaler.font_s(11.0), COLOR_LIGHT_GRAY);
        }
        Err(ref errs) => {
            fonts.draw_ui_bold("✗ VALIDATION BLOCKED", x + scaler.s(16.0), diag_box_y + scaler.s(26.0), scaler.font_s(13.0), Palette::RED);
            for (i, err) in errs.iter().take(5).enumerate() {
                fonts.draw_ui_regular(
                    &format!("• {}", err),
                    x + scaler.s(16.0),
                    diag_box_y + scaler.s(52.0 + (i as f32) * 20.0),
                    scaler.font_s(10.5),
                    Palette::WHITE,
                );
            }
        }
    }

    // Action Buttons below diagnostics
    let btn_y = diag_box_y + diag_box_h + scaler.s(16.0);
    let btn_h = scaler.s(38.0);
    let btn_w = col_w;

    // Launch Test Cup
    let is_valid = val_result.is_ok();
    let launch_bg = if is_valid { Color::new(0.12, 0.35, 0.20, 0.90) } else { Color::new(0.12, 0.12, 0.12, 0.50) };
    let launch_border = if is_valid { Palette::NEON_GREEN } else { COLOR_DARK_GRAY };
    scaler.draw_glass_card(x, btn_y, btn_w, btn_h, launch_bg, launch_border, 1.5);
    fonts.draw_ui_bold("🚀 LAUNCH TEST CUP (RACE NOW)", x + scaler.s(24.0), btn_y + scaler.s(24.0), scaler.font_s(12.0), Palette::WHITE);

    // Save to User Cups
    let save_user_y = btn_y + btn_h + scaler.s(10.0);
    scaler.draw_glass_card(x, save_user_y, btn_w, btn_h, Color::new(0.10, 0.18, 0.32, 0.90), Palette::NEON_CYAN, 1.2);
    fonts.draw_ui_bold("💾 SAVE TO USER CHAMPIONSHIPS", x + scaler.s(24.0), save_user_y + scaler.s(24.0), scaler.font_s(12.0), Palette::WHITE);

    // Save to Repo Presets (Dev Mode only)
    if is_dev {
        let save_dev_y = save_user_y + btn_h + scaler.s(10.0);
        scaler.draw_glass_card(x, save_dev_y, btn_w, btn_h, Color::new(0.24, 0.16, 0.05, 0.90), Palette::NEON_GOLD, 1.2);
        fonts.draw_ui_bold("⚙️ SAVE TO REPO PRESETS [DEV ONLY]", x + scaler.s(24.0), save_dev_y + scaler.s(24.0), scaler.font_s(12.0), Palette::WHITE);
    }

    // Right Column: Formatted TOML Source Code Preview
    let right_x = x + col_w + scaler.s(24.0);
    fonts.draw_ui_bold("GENERATED TOML DOCUMENT PREVIEW", right_x, y + scaler.s(14.0), scaler.font_s(13.0), Palette::NEON_GOLD);

    let toml_box_y = y + scaler.s(28.0);
    let toml_box_h = h - scaler.s(38.0);
    scaler.draw_glass_card(right_x, toml_box_y, col_w, toml_box_h, Color::new(0.04, 0.05, 0.08, 0.90), Palette::UI_CARD_BORDER, 1.0);

    let toml_preview = state.def.to_toml().unwrap_or_else(|_| "# Formatting error".to_string());
    let mut line_y = toml_box_y + scaler.s(18.0);
    for line in toml_preview.lines().take(22) {
        let col = if line.starts_with('[') {
            Palette::NEON_GOLD
        } else if line.contains('=') {
            Palette::NEON_CYAN
        } else {
            Palette::WHITE
        };
        fonts.draw_ui_regular(line, right_x + scaler.s(14.0), line_y, scaler.font_s(10.0), col);
        line_y += scaler.s(14.0);
    }
}

fn render_footer(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &ChampionshipEditorState,
    _is_dev: bool,
) {
    let sw = screen_width();
    let sh = screen_height();
    let footer_h = scaler.s(36.0);
    let y = sh - footer_h;

    draw_rectangle(0.0, y, sw, footer_h, Color::new(0.05, 0.06, 0.10, 0.95));
    scaler.draw_glass_card(0.0, y, sw, 1.0, COLOR_TRANSPARENT, Palette::UI_CARD_BORDER, 1.0);

    let hints = match state.active_tab {
        ChampionshipEditorTab::Rules => "[Tab] Next Tab • [Up/Down] Select Field • [Enter] Edit • [Esc] Exit",
        ChampionshipEditorTab::Calendar => "[Tab] Next Tab • [A] Add Round • [Up/Down] Reorder • [D] Delete Round • [Esc] Exit",
        ChampionshipEditorTab::Grid => "[Tab] Next Tab • [Up/Down] Select Driver • [Enter] Edit Name • [M] Change Car • [Esc] Exit",
        ChampionshipEditorTab::Export => "[Tab] Next Tab • [L / F5] Launch Test Cup • [S] Save TOML • [Esc] Exit",
    };

    fonts.draw_ui_regular(hints, scaler.s(24.0), y + scaler.s(22.0), scaler.font_s(11.0), COLOR_LIGHT_GRAY);
}

fn render_modal(
    fonts: &Fonts,
    scaler: &UiScaler,
    state: &ChampionshipEditorState,
    all_tracks: &[TrackChoice],
) {
    let sw = screen_width();
    let sh = screen_height();

    match state.modal {
        ChampionshipEditorModal::None => {}
        ChampionshipEditorModal::AddTrack { selected_idx } => {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.65));

            let modal_w = scaler.s(440.0);
            let modal_h = scaler.s(360.0);
            let mx = (sw - modal_w) * 0.5;
            let my = (sh - modal_h) * 0.5;

            scaler.draw_glass_card(mx, my, modal_w, modal_h, Color::new(0.08, 0.10, 0.16, 0.98), Palette::NEON_GOLD, 2.0);
            fonts.draw_ui_bold("SELECT TRACK FOR CALENDAR", mx + scaler.s(20.0), my + scaler.s(30.0), scaler.font_s(13.0), Palette::NEON_GOLD);

            let row_h = scaler.s(32.0);
            let start_y = my + scaler.s(50.0);
            let max_visible = 8;
            let scroll_offset = selected_idx.saturating_sub(max_visible - 1);

            for (i, track) in all_tracks.iter().skip(scroll_offset).take(max_visible).enumerate() {
                let actual_idx = scroll_offset + i;
                let card_y = start_y + (i as f32) * (row_h + scaler.s(4.0));
                let is_sel = actual_idx == selected_idx;

                let bg = if is_sel { Color::new(0.15, 0.22, 0.38, 0.90) } else { Color::new(0.05, 0.07, 0.12, 0.60) };
                let border = if is_sel { Palette::NEON_CYAN } else { Palette::UI_CARD_BORDER };

                scaler.draw_glass_card(mx + scaler.s(16.0), card_y, modal_w - scaler.s(32.0), row_h, bg, border, 1.2);
                fonts.draw_ui_bold(track.title(), mx + scaler.s(28.0), card_y + scaler.s(20.0), scaler.font_s(11.5), Palette::WHITE);
                fonts.draw_ui_regular(track.tag(), mx + modal_w - scaler.s(160.0), card_y + scaler.s(20.0), scaler.font_s(9.5), COLOR_LIGHT_GRAY);
            }

            fonts.draw_ui_regular("[Up/Down] Navigate • [Enter] Add • [Esc] Cancel", mx + scaler.s(20.0), my + modal_h - scaler.s(16.0), scaler.font_s(10.0), COLOR_DARK_GRAY);
        }
        ChampionshipEditorModal::SelectCarModel { driver_idx, selected_idx } => {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.65));

            let modal_w = scaler.s(480.0);
            let modal_h = scaler.s(380.0);
            let mx = (sw - modal_w) * 0.5;
            let my = (sh - modal_h) * 0.5;

            scaler.draw_glass_card(mx, my, modal_w, modal_h, Color::new(0.08, 0.10, 0.16, 0.98), Palette::NEON_CYAN, 2.0);

            let driver_name = state.def.drivers.get(driver_idx).map(|d| d.name.as_str()).unwrap_or("Driver");
            fonts.draw_ui_bold(&format!("SELECT VEHICLE: {}", driver_name), mx + scaler.s(20.0), my + scaler.s(30.0), scaler.font_s(13.0), Palette::NEON_CYAN);

            let models = crate::catalog::get_models_for_module(&state.def.championship.module_id);
            let row_h = scaler.s(34.0);
            let start_y = my + scaler.s(50.0);
            let max_visible = 8;
            let scroll_offset = selected_idx.saturating_sub(max_visible - 1);

            for (i, m) in models.iter().skip(scroll_offset).take(max_visible).enumerate() {
                let actual_idx = scroll_offset + i;
                let card_y = start_y + (i as f32) * (row_h + scaler.s(4.0));
                let is_sel = actual_idx == selected_idx;

                let bg = if is_sel { Color::new(0.15, 0.22, 0.38, 0.90) } else { Color::new(0.05, 0.07, 0.12, 0.60) };
                let border = if is_sel { Palette::NEON_GOLD } else { Palette::UI_CARD_BORDER };

                scaler.draw_glass_card(mx + scaler.s(16.0), card_y, modal_w - scaler.s(32.0), row_h, bg, border, 1.2);
                fonts.draw_ui_bold(m.name, mx + scaler.s(28.0), card_y + scaler.s(21.0), scaler.font_s(11.5), Palette::WHITE);
                fonts.draw_ui_regular(&format!("Tier {} • {} BHP", m.tier, m.bhp), mx + modal_w - scaler.s(150.0), card_y + scaler.s(21.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
            }

            fonts.draw_ui_regular("[Up/Down] Navigate • [Enter] Assign • [Esc] Cancel", mx + scaler.s(20.0), my + modal_h - scaler.s(16.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
        }
        ChampionshipEditorModal::EditTextField {
            ref target,
            ref value,
            cursor_timer,
        } => {
            draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.65));

            let modal_w = scaler.s(420.0);
            let modal_h = scaler.s(150.0);
            let mx = (sw - modal_w) * 0.5;
            let my = (sh - modal_h) * 0.5;

            scaler.draw_glass_card(mx, my, modal_w, modal_h, Color::new(0.08, 0.10, 0.16, 0.98), Palette::NEON_CYAN, 2.0);

            let title = match target {
                TextEditTarget::Id => "EDIT CHAMPIONSHIP ID (SLUG)",
                TextEditTarget::Name => "EDIT CHAMPIONSHIP TITLE",
                TextEditTarget::Description => "EDIT DESCRIPTION",
                TextEditTarget::DriverName(_) => "EDIT DRIVER NAME",
                TextEditTarget::DriverTeam(_) => "EDIT TEAM NAME",
            };
            fonts.draw_ui_bold(title, mx + scaler.s(20.0), my + scaler.s(30.0), scaler.font_s(12.5), Palette::NEON_CYAN);

            // Input box
            let input_box_y = my + scaler.s(50.0);
            let input_box_h = scaler.s(36.0);
            scaler.draw_glass_card(mx + scaler.s(20.0), input_box_y, modal_w - scaler.s(40.0), input_box_h, Color::new(0.03, 0.04, 0.06, 0.95), Palette::WHITE, 1.5);

            let cursor = if (cursor_timer * 2.5) as i32 % 2 == 0 { "_" } else { " " };
            let display_text = format!("{}{}", value, cursor);
            fonts.draw_ui_bold(&display_text, mx + scaler.s(30.0), input_box_y + scaler.s(24.0), scaler.font_s(13.0), Palette::WHITE);

            fonts.draw_ui_regular("[Enter] Save • [Esc] Cancel", mx + scaler.s(20.0), my + modal_h - scaler.s(16.0), scaler.font_s(10.0), COLOR_DARK_GRAY);
        }
    }
}

/// Handles interactive keyboard, gamepad, and mouse events for the Championship Editor.
pub fn handle_championship_editor_input(
    state: &mut ChampionshipEditorState,
    all_tracks: &[TrackChoice],
    is_dev: bool,
) -> ChampionshipEditorAction {
    // 1. Modal Input Handling
    match state.modal {
        ChampionshipEditorModal::AddTrack { ref mut selected_idx } => {
            if is_key_pressed(KeyCode::Escape) {
                state.modal = ChampionshipEditorModal::None;
                return ChampionshipEditorAction::None;
            }
            if is_key_pressed(KeyCode::Up) {
                *selected_idx = selected_idx.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) && !all_tracks.is_empty() {
                *selected_idx = (*selected_idx + 1).min(all_tracks.len() - 1);
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                if let Some(track) = all_tracks.get(*selected_idx) {
                    let order = state.def.rounds.len() + 1;
                    state.def.rounds.push(RoundConfig {
                        order,
                        track_id: track.track_id().to_string(),
                        name: Some(track.title().to_string()),
                        laps: None,
                        weather: None,
                    });
                    state.set_status("Added circuit to calendar", 2.0);
                }
                state.modal = ChampionshipEditorModal::None;
            }
            return ChampionshipEditorAction::None;
        }
        ChampionshipEditorModal::SelectCarModel { driver_idx, ref mut selected_idx } => {
            let models = crate::catalog::get_models_for_module(&state.def.championship.module_id);
            if is_key_pressed(KeyCode::Escape) {
                state.modal = ChampionshipEditorModal::None;
                return ChampionshipEditorAction::None;
            }
            if is_key_pressed(KeyCode::Up) {
                *selected_idx = selected_idx.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) && !models.is_empty() {
                *selected_idx = (*selected_idx + 1).min(models.len() - 1);
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                if let Some(m) = models.get(*selected_idx) {
                    if let Some(d) = state.def.drivers.get_mut(driver_idx) {
                        d.car_model_id = Some(m.id.to_string());
                    }
                    state.set_status("Assigned vehicle model", 2.0);
                }
                state.modal = ChampionshipEditorModal::None;
            }
            return ChampionshipEditorAction::None;
        }
        ChampionshipEditorModal::EditTextField {
            target,
            ref mut value,
            ..
        } => {
            if is_key_pressed(KeyCode::Escape) {
                state.modal = ChampionshipEditorModal::None;
                return ChampionshipEditorAction::None;
            }
            if is_key_pressed(KeyCode::Backspace) {
                value.pop();
            }
            // Capture printable characters
            while let Some(c) = macroquad::input::get_char_pressed() {
                if !c.is_control() {
                    value.push(c);
                }
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                match target {
                    TextEditTarget::Id => {
                        state.def.championship.id = value.trim().to_lowercase().replace(' ', "_");
                    }
                    TextEditTarget::Name => {
                        state.def.championship.name = value.trim().to_string();
                    }
                    TextEditTarget::Description => {
                        state.def.championship.description = value.trim().to_string();
                    }
                    TextEditTarget::DriverName(idx) => {
                        if let Some(d) = state.def.drivers.get_mut(idx) {
                            d.name = value.trim().to_string();
                        }
                    }
                    TextEditTarget::DriverTeam(idx) => {
                        if let Some(d) = state.def.drivers.get_mut(idx) {
                            d.team = value.trim().to_string();
                        }
                    }
                }
                state.modal = ChampionshipEditorModal::None;
                state.set_status("Updated field value", 1.5);
            }
            return ChampionshipEditorAction::None;
        }
        ChampionshipEditorModal::None => {}
    }

    // 2. Global Hotkeys
    if is_key_pressed(KeyCode::Escape) {
        return ChampionshipEditorAction::Exit;
    }

    if is_key_pressed(KeyCode::Tab) {
        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            state.active_tab = state.active_tab.prev();
        } else {
            state.active_tab = state.active_tab.next();
        }
        return ChampionshipEditorAction::None;
    }

    // Number keys 1..4 switch tabs directly
    if is_key_pressed(KeyCode::Key1) { state.active_tab = ChampionshipEditorTab::Rules; }
    if is_key_pressed(KeyCode::Key2) { state.active_tab = ChampionshipEditorTab::Calendar; }
    if is_key_pressed(KeyCode::Key3) { state.active_tab = ChampionshipEditorTab::Grid; }
    if is_key_pressed(KeyCode::Key4) { state.active_tab = ChampionshipEditorTab::Export; }

    // Tab-specific interactions
    match state.active_tab {
        ChampionshipEditorTab::Rules => {
            if is_key_pressed(KeyCode::Up) {
                state.rules_field_idx = state.rules_field_idx.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) {
                state.rules_field_idx = (state.rules_field_idx + 1).min(8);
            }
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                match state.rules_field_idx {
                    3 => { // Module cycle
                        let modules = ["gt", "nascar", "rally", "kart", "extreme_offroad", "classic"];
                        let curr_pos = modules.iter().position(|&m| m == state.def.championship.module_id).unwrap_or(0);
                        let next_pos = if is_key_pressed(KeyCode::Right) {
                            (curr_pos + 1) % modules.len()
                        } else {
                            (curr_pos + modules.len() - 1) % modules.len()
                        };
                        state.def.championship.module_id = modules[next_pos].to_string();
                        autofill_grid_for_module(&mut state.def);
                    }
                    4 => { // Tier cycle (1..=5)
                        if is_key_pressed(KeyCode::Right) {
                            state.def.championship.tier = (state.def.championship.tier % 5) + 1;
                        } else {
                            state.def.championship.tier = if state.def.championship.tier <= 1 { 5 } else { state.def.championship.tier - 1 };
                        }
                        autofill_grid_for_module(&mut state.def);
                    }
                    5 => { // Default laps
                        if is_key_pressed(KeyCode::Right) {
                            state.def.championship.laps_per_round = (state.def.championship.laps_per_round + 1).min(50);
                        } else {
                            state.def.championship.laps_per_round = state.def.championship.laps_per_round.saturating_sub(1).max(1);
                        }
                    }
                    6 => { // Point system
                        let systems = ["fia", "nascar", "arcade", "motogp"];
                        let curr = systems.iter().position(|&s| s == state.def.scoring.system).unwrap_or(0);
                        let next = if is_key_pressed(KeyCode::Right) { (curr + 1) % systems.len() } else { (curr + systems.len() - 1) % systems.len() };
                        state.def.scoring.system = systems[next].to_string();
                    }
                    _ => {}
                }
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                match state.rules_field_idx {
                    0 => {
                        state.modal = ChampionshipEditorModal::EditTextField {
                            target: TextEditTarget::Id,
                            value: state.def.championship.id.clone(),
                            cursor_timer: 0.0,
                        };
                    }
                    1 => {
                        state.modal = ChampionshipEditorModal::EditTextField {
                            target: TextEditTarget::Name,
                            value: state.def.championship.name.clone(),
                            cursor_timer: 0.0,
                        };
                    }
                    2 => {
                        state.modal = ChampionshipEditorModal::EditTextField {
                            target: TextEditTarget::Description,
                            value: state.def.championship.description.clone(),
                            cursor_timer: 0.0,
                        };
                    }
                    7 => {
                        state.def.scoring.fastest_lap_bonus = !state.def.scoring.fastest_lap_bonus;
                    }
                    8 => {
                        state.def.scoring.clean_race_bonus = !state.def.scoring.clean_race_bonus;
                    }
                    _ => {}
                }
            }
        }
        ChampionshipEditorTab::Calendar => {
            if is_key_pressed(KeyCode::Up) {
                state.selected_round_idx = state.selected_round_idx.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) && !state.def.rounds.is_empty() {
                state.selected_round_idx = (state.selected_round_idx + 1).min(state.def.rounds.len() - 1);
            }
            // Add Round
            if is_key_pressed(KeyCode::A) {
                state.modal = ChampionshipEditorModal::AddTrack { selected_idx: 0 };
            }
            // Delete Round
            if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Delete) {
                if state.selected_round_idx < state.def.rounds.len() {
                    state.def.rounds.remove(state.selected_round_idx);
                    if state.selected_round_idx >= state.def.rounds.len() {
                        state.selected_round_idx = state.def.rounds.len().saturating_sub(1);
                    }
                    state.set_status("Removed round from calendar", 1.5);
                }
            }
            // Adjust per-round laps
            if is_key_pressed(KeyCode::L) {
                if let Some(r) = state.def.rounds.get_mut(state.selected_round_idx) {
                    let cur = r.laps.unwrap_or(state.def.championship.laps_per_round);
                    r.laps = Some((cur % 10) + 1);
                }
            }
        }
        ChampionshipEditorTab::Grid => {
            if is_key_pressed(KeyCode::Up) {
                state.selected_driver_idx = state.selected_driver_idx.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) && !state.def.drivers.is_empty() {
                state.selected_driver_idx = (state.selected_driver_idx + 1).min(state.def.drivers.len() - 1);
            }
            // Autofill Grid
            if is_key_pressed(KeyCode::F) {
                autofill_grid_for_module(&mut state.def);
                state.set_status("Autofilled driver grid for module", 2.0);
            }
            // Edit driver name
            if is_key_pressed(KeyCode::Enter) {
                if let Some(d) = state.def.drivers.get(state.selected_driver_idx) {
                    state.modal = ChampionshipEditorModal::EditTextField {
                        target: TextEditTarget::DriverName(state.selected_driver_idx),
                        value: d.name.clone(),
                        cursor_timer: 0.0,
                    };
                }
            }
            // Change model
            if is_key_pressed(KeyCode::M) {
                state.modal = ChampionshipEditorModal::SelectCarModel {
                    driver_idx: state.selected_driver_idx,
                    selected_idx: 0,
                };
            }
        }
        ChampionshipEditorTab::Export => {
            if is_key_pressed(KeyCode::L) || is_key_pressed(KeyCode::F5) {
                if state.def.validate().is_ok() {
                    return ChampionshipEditorAction::LaunchTestCup(state.def.clone());
                } else {
                    state.set_status("Cannot launch: Fix validation errors first!", 2.5);
                }
            }
            if is_key_pressed(KeyCode::S) {
                return ChampionshipEditorAction::SaveUser;
            }
            if is_dev && is_key_pressed(KeyCode::P) {
                return ChampionshipEditorAction::SavePreset;
            }
        }
    }

    // Mouse click handling for interactive action buttons on Export Tab
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        let sw = screen_width();
        let sh = screen_height();
        let scaler = UiScaler::new(sw, sh);
        let pad_x = scaler.s(24.0);
        let col_w = (sw - pad_x * 2.0 - scaler.s(24.0)) * 0.5;
        let btn_y = scaler.s(286.0);
        let btn_h = scaler.s(38.0);

        if state.active_tab == ChampionshipEditorTab::Export {
            // Launch Test Cup button click
            if mx >= pad_x && mx <= pad_x + col_w && my >= btn_y && my <= btn_y + btn_h {
                if state.def.validate().is_ok() {
                    return ChampionshipEditorAction::LaunchTestCup(state.def.clone());
                } else {
                    state.set_status("Cannot launch: Fix validation errors first!", 2.5);
                }
            }
            // Save User button click
            let save_y = btn_y + btn_h + scaler.s(10.0);
            if mx >= pad_x && mx <= pad_x + col_w && my >= save_y && my <= save_y + btn_h {
                return ChampionshipEditorAction::SaveUser;
            }
            // Save Preset button click
            if is_dev {
                let dev_y = save_y + btn_h + scaler.s(10.0);
                if mx >= pad_x && mx <= pad_x + col_w && my >= dev_y && my <= dev_y + btn_h {
                    return ChampionshipEditorAction::SavePreset;
                }
            }
        }
    }

    ChampionshipEditorAction::None
}
