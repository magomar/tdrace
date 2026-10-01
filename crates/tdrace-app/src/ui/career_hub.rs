use macroquad::prelude::*;
use crate::catalog::get_models_for_module_and_tier;
use crate::profile::{draw_country_banner, ModuleCareerProgress, PlayerProfile};
use crate::tournament::ChampionshipSession;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::render::color::Palette;
use cabinet::ui::{
    ColumnAlign, DataColumn, DataRow, DataTable, FilterBar, FilterBarStyle, LayoutRect, MetricBar,
    ScreenFooter, SplitPane, VStack, ValueStepper,
};

/// Human-readable title for a GT tier championship.
pub fn gt_tier_title(tier: u32) -> &'static str {
    match tier {
        1 => "GT4 Clubman Sprint Cup",
        2 => "FIA GT3 European Challenge",
        3 => "SRO GT2 Power Masters",
        4 => "Le Mans 90s Heritage Trophy",
        _ => "World Endurance Hypercar Grand Prix",
    }
}

/// Vehicle class description for a GT tier.
pub fn gt_tier_class(tier: u32) -> &'static str {
    match tier {
        1 => "GT4 Clubsport (500 BHP • Mechanical Grip)",
        2 => "FIA GT3 Pro (565 BHP • High Downforce)",
        3 => "SRO GT2 Masters (640–700 BHP • High Speed)",
        4 => "GT1 Heritage (600–650 BHP • Historic Legends)",
        _ => "Hypercar Prototype (680–1000+ BHP • Le Mans Apex)",
    }
}

/// Returns the mandatory 3 new tracks for each GT tier (5 starter circuits for Tier 1).
pub fn gt_mandatory_tracks(tier: u32) -> &'static [&'static str] {
    match tier {
        1 => &["red_bull_ring", "zandvoort", "nurburgring_gp", "portimao_gp", "montreal"],
        2 => &["monza", "silverstone", "catalunya"],
        3 => &["spa", "cota", "bahrain"],
        4 => &["suzuka", "interlagos", "bathurst"],
        _ => &["le_mans_sarthe", "monaco", "marina_bay"],
    }
}

/// Returns the default curated calendar for a GT tier (5, 7, 9, 10, 12 circuits).
pub fn gt_default_calendar(tier: u32) -> Vec<String> {
    match tier {
        1 => vec![
            "red_bull_ring".to_string(),
            "zandvoort".to_string(),
            "nurburgring_gp".to_string(),
            "portimao_gp".to_string(),
            "montreal".to_string(),
        ],
        2 => vec![
            "monza".to_string(),
            "silverstone".to_string(),
            "catalunya".to_string(),
            "red_bull_ring".to_string(),
            "zandvoort".to_string(),
            "nurburgring_gp".to_string(),
            "portimao_gp".to_string(),
        ],
        3 => vec![
            "spa".to_string(),
            "cota".to_string(),
            "bahrain".to_string(),
            "monza".to_string(),
            "silverstone".to_string(),
            "catalunya".to_string(),
            "red_bull_ring".to_string(),
            "nurburgring_gp".to_string(),
            "montreal".to_string(),
        ],
        4 => vec![
            "suzuka".to_string(),
            "interlagos".to_string(),
            "bathurst".to_string(),
            "spa".to_string(),
            "monza".to_string(),
            "silverstone".to_string(),
            "catalunya".to_string(),
            "cota".to_string(),
            "nurburgring_gp".to_string(),
            "red_bull_ring".to_string(),
        ],
        _ => vec![
            "le_mans_sarthe".to_string(),
            "monaco".to_string(),
            "marina_bay".to_string(),
            "spa".to_string(),
            "monza".to_string(),
            "silverstone".to_string(),
            "suzuka".to_string(),
            "bathurst".to_string(),
            "interlagos".to_string(),
            "catalunya".to_string(),
            "cota".to_string(),
            "red_bull_ring".to_string(),
        ],
    }
}

/// All eligible circuits from previous tiers available for selection in optional slots.
pub fn gt_eligible_previous_tracks(tier: u32) -> Vec<&'static str> {
    let mut eligible = Vec::new();
    if tier >= 2 {
        eligible.extend_from_slice(&["red_bull_ring", "zandvoort", "nurburgring_gp", "portimao_gp", "montreal"]);
    }
    if tier >= 3 {
        eligible.extend_from_slice(&["monza", "silverstone", "catalunya"]);
    }
    if tier >= 4 {
        eligible.extend_from_slice(&["spa", "cota", "bahrain"]);
    }
    if tier >= 5 {
        eligible.extend_from_slice(&["suzuka", "interlagos", "bathurst"]);
    }
    eligible
}

/// Checks if a circuit in a tier's calendar is a mandatory track.
pub fn is_slot_mandatory(tier: u32, track_id: &str) -> bool {
    gt_mandatory_tracks(tier).contains(&track_id)
}

/// Cycles an optional calendar slot to the previous or next eligible circuit.
pub fn cycle_calendar_slot(tier: u32, calendar: &mut [String], slot_idx: usize, forward: bool) -> bool {
    if slot_idx >= calendar.len() {
        return false;
    }
    let cur_track = calendar[slot_idx].clone();
    if is_slot_mandatory(tier, &cur_track) {
        return false; // Cannot swap mandatory track
    }

    let all_eligible = gt_eligible_previous_tracks(tier);
    if all_eligible.is_empty() {
        return false;
    }

    // Filter out tracks already present in OTHER slots of the calendar
    let available: Vec<&'static str> = all_eligible
        .into_iter()
        .filter(|&t| t == cur_track || !calendar.iter().any(|c| c == t))
        .collect();

    if available.len() <= 1 {
        return false;
    }

    let cur_pos = available.iter().position(|&t| t == cur_track).unwrap_or(0);
    let mut stepper = ValueStepper::new("Slot", 0usize, available.len() - 1, 1usize, cur_pos);
    let advanced = if forward { stepper.step_up() } else { stepper.step_down() };
    // ValueStepper clamps at the bounds; emulate wrap-around to the opposite end.
    let next_pos = if advanced {
        stepper.value
    } else if forward {
        0
    } else {
        available.len() - 1
    };

    calendar[slot_idx] = available[next_pos].to_string();
    true
}

/// Human-readable circuit title, from the official catalog (spec 042).
pub fn track_title(track_id: &str) -> &'static str {
    tdrace_core::catalog::find(track_id, None).map_or("Motorsport Circuit", |c| c.name)
}

/// 2-Letter ISO country code for circuit flag banner, from the official catalog (spec 042).
pub fn track_country(track_id: &str) -> &'static str {
    tdrace_core::catalog::find(track_id, None)
        .map(|c| c.country_code)
        .filter(|code| !code.is_empty())
        .unwrap_or("--")
}

/// Approximate circuit length in meters.
pub fn track_length_meters(track_id: &str) -> u32 {
    match track_id {
        "red_bull_ring" => 2159,
        "zandvoort" => 2129,
        "nurburgring_gp" => 2574,
        "portimao_gp" => 2326,
        "montreal" => 2181,
        "monza" => 2897,
        "silverstone" => 2946,
        "catalunya" => 2329,
        "spa" => 3502,
        "cota" => 2757,
        "bahrain" => 2706,
        "suzuka" => 2904,
        "interlagos" => 2155,
        "bathurst" => 3107,
        "le_mans_sarthe" => 6813,
        "monaco" => 1669,
        "marina_bay" => 2532,
        _ => 2500,
    }
}

fn draw_ui_bold_right(fonts: &Fonts, text: &str, right_x: f32, y: f32, size: f32, color: Color) {
    let dim = fonts.measure_ui_bold(text, size);
    fonts.draw_ui_bold(text, right_x - dim.width, y, size, color);
}

fn draw_ui_regular_right(fonts: &Fonts, text: &str, right_x: f32, y: f32, size: f32, color: Color) {
    let dim = fonts.measure_ui_regular(text, size);
    fonts.draw_ui_regular(text, right_x - dim.width, y, size, color);
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(b as char);
    }
    out
}

/// Focus area within the GT Career Hub screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CareerHubFocus {
    /// Championship Tier Tab bar focused (Left/Right arrow keys & Gamepad switch tiers)
    #[default]
    Tabs,
    /// Championship Calendar focused (Up/Down navigates slots, Up from slot 0 returns to Tabs)
    Calendar,
}

/// Primary Career Hub screen renderer.
#[allow(clippy::too_many_arguments)]
pub fn render_career_hub_screen(
    fonts: &Fonts,
    profile: &PlayerProfile,
    career: &ModuleCareerProgress,
    selected_tier: u32,
    selected_slot: usize,
    calendar: &[String],
    champ: Option<&ChampionshipSession>,
    showing_standings: bool,
    active_car_model_id: Option<&str>,
    is_gamepad: bool,
    focus_area: CareerHubFocus,
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // Deep motorsport dark backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    let full_w = (sw * 0.96).max(scaler.s(760.0));
    let x = (sw - full_w) * 0.5;
    let mut cur_y = scaler.s(14.0);

    // =========================================================================
    // 1. TOP HERO HEADER: DRIVER BANNER & CAREER WALLET
    // =========================================================================
    let header_h = scaler.s(56.0);
    scaler.draw_glass_card(x, cur_y, full_w, header_h, Color::new(0.06, 0.08, 0.13, 0.95), Palette::UI_CARD_BORDER, 1.2);

    let pad_x = scaler.s(14.0);

    // Country Flag Banner
    let flag_w = scaler.s(44.0);
    let flag_h = scaler.s(28.0);
    let flag_y = cur_y + (header_h - flag_h) * 0.5;
    draw_country_banner(profile.country.as_deref(), x + pad_x, flag_y, flag_w, flag_h, Some(fonts), &scaler);

    // Driver Identity
    let driver_name = profile.name.to_uppercase();
    let alias_tag = if profile.alias.is_empty() { "PILOT" } else { &profile.alias };
    let title_line = format!("{} [\"{}\"]", driver_name, alias_tag);
    let sub_line = "Team: Apex GT Racing  •  Discipline: GRAN TURISMO CAREER";

    let text_x = x + pad_x + flag_w + scaler.s(12.0);
    fonts.draw_ui_bold(&title_line, text_x, cur_y + scaler.s(22.0), scaler.font_s(14.0), Palette::WHITE);
    fonts.draw_ui_regular(sub_line, text_x, cur_y + scaler.s(40.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

    // Career XP Wallet (Right Aligned)
    let right_x = x + full_w - scaler.s(16.0);
    let xp_str = format!("{} XP", format_number(career.xp));
    draw_ui_bold_right(fonts, &xp_str, right_x, cur_y + scaler.s(23.0), scaler.font_s(16.0), Palette::NEON_GOLD);

    let trophy_str = format!("🏆 {}  🥈 {}  🥉 {}", career.trophies_gold, career.trophies_silver, career.trophies_bronze);
    draw_ui_bold_right(fonts, &trophy_str, right_x, cur_y + scaler.s(42.0), scaler.font_s(11.5), Palette::NEON_CYAN);

    cur_y += header_h + scaler.s(10.0);

    // =========================================================================
    // 2. TIER SELECTOR TAB BAR [TIER 1 .. 6] (platform FilterBar, Shelf style)
    // =========================================================================
    let tab_bar_h = scaler.s(40.0);
    let tier_count = career.max_tier();

    let is_tabs_focused = focus_area == CareerHubFocus::Tabs;

    let tier_labels: Vec<String> = (1..=tier_count)
        .map(|t| {
            let base = match t {
                1 => "1. GT4 CLUBMAN",
                2 => "2. FIA GT3",
                3 => "3. SRO GT2",
                4 => "4. LE MANS 90s",
                _ => "5. WEC HYPERCAR",
            };
            if t <= career.level {
                base.to_string()
            } else {
                format!("🔒 {}", base)
            }
        })
        .collect();
    let tier_label_refs: Vec<&str> = tier_labels.iter().map(|s| s.as_str()).collect();
    let tier_bar = FilterBar::from_labels(&tier_label_refs)
        .with_style(FilterBarStyle::Shelf)
        .with_gap(8.0)
        .with_active(selected_tier.saturating_sub(1) as usize)
        .with_focus(is_tabs_focused);
    tier_bar.draw(&scaler, fonts, x, cur_y, full_w, tab_bar_h, Palette::NEON_CYAN);

    cur_y += tab_bar_h + scaler.s(12.0);

    // =========================================================================
    // 3. MAIN DUAL-COLUMN BODY
    // =========================================================================
    let body_h = (sh - cur_y - scaler.s(60.0)).max(scaler.s(410.0));
    let col_gap = scaler.s(14.0);
    let split_pane = SplitPane::new(LayoutRect::new(x, cur_y, full_w, body_h), 0.38, col_gap);
    let left_rect = split_pane.left_rect();
    let right_rect = split_pane.right_rect();
    let left_w = left_rect.w;
    let right_w = right_rect.w;
    let left_x = left_rect.x;
    let right_x = right_rect.x;

    // -------------------------------------------------------------------------
    // LEFT COLUMN: TIER OVERVIEW, PROMOTION STATUS, & ACTIVE CAR
    // -------------------------------------------------------------------------
    scaler.draw_glass_card(left_x, cur_y, left_w, body_h, Color::new(0.05, 0.07, 0.11, 0.95), Palette::UI_CARD_BORDER, 1.2);

    let mut ly = cur_y + scaler.s(16.0);
    let left_inner_x = left_x + scaler.s(16.0);
    let left_inner_w = left_w - scaler.s(32.0);

    // Active Tier Badge & Title
    let tier_str = format!("TIER {} CHAMPIONSHIP", selected_tier);
    fonts.draw_ui_bold(&tier_str, left_inner_x, ly + scaler.s(14.0), scaler.font_s(11.0), Palette::NEON_GOLD);
    fonts.draw_ui_bold(gt_tier_title(selected_tier), left_inner_x, ly + scaler.s(34.0), scaler.font_s(15.0), Palette::WHITE);
    fonts.draw_ui_regular(gt_tier_class(selected_tier), left_inner_x, ly + scaler.s(52.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);

    ly += scaler.s(68.0);
    draw_rectangle(left_inner_x, ly, left_inner_w, 1.0, Palette::UI_CARD_BORDER);
    ly += scaler.s(14.0);

    // Career Progression Gates
    fonts.draw_ui_bold("TIER STATUS & PROMOTION", left_inner_x, ly + scaler.s(12.0), scaler.font_s(12.0), Palette::NEON_CYAN);
    ly += scaler.s(22.0);

    if selected_tier < career.level {
        let comp_msg = "✓ TIER COMPLETED — REPLAY CUP FOR XP & TROPHIES";
        fonts.draw_ui_bold(comp_msg, left_inner_x, ly + scaler.s(14.0), scaler.font_s(11.0), Palette::NEON_GREEN);
        ly += scaler.s(24.0);
    } else if selected_tier == career.level {
        if career.level >= career.max_tier() {
            fonts.draw_ui_bold("★ PINNACLE TIER REACHED — MAXIMUM APEX", left_inner_x, ly + scaler.s(14.0), scaler.font_s(11.0), Palette::NEON_GOLD);
            ly += scaler.s(24.0);
        } else {
            let next_tier = career.level + 1;
            let req_xp = ModuleCareerProgress::tier_license_xp_for_module(&career.module_id, next_tier);
            let has_podium = career.has_podium_in_tier(career.level);
            let has_xp = career.xp >= req_xp;

            let podium_check = if has_podium { "✓ Tier Podium Earned" } else { "✗ Requires 1+ Tier Championship Podium" };
            let xp_check = format!("{}/{} XP (License Threshold: {} XP)", format_number(career.xp), format_number(req_xp), format_number(req_xp));

            let pod_col = if has_podium { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED };
            let xp_col = if has_xp { Palette::NEON_GREEN } else { Palette::NEON_GOLD };

            fonts.draw_ui_regular(podium_check, left_inner_x, ly + scaler.s(14.0), scaler.font_s(11.0), pod_col);
            fonts.draw_ui_regular(&xp_check, left_inner_x, ly + scaler.s(32.0), scaler.font_s(11.0), xp_col);
            ly += scaler.s(42.0);

            // Progress Bar towards tier promotion (platform MetricBar)
            let progress_ratio = (career.xp as f32 / req_xp as f32).clamp(0.0, 1.0);
            let license_bar = MetricBar::progress(
                "LICENSE XP",
                progress_ratio,
                format!("{:.0}%", progress_ratio * 100.0),
                Palette::NEON_CYAN,
            );
            license_bar.draw(&scaler, fonts, LayoutRect::new(left_inner_x, ly, left_inner_w, scaler.s(18.0)));
            ly += scaler.s(22.0);

            if career.can_advance_tier() {
                scaler.draw_glass_card(left_inner_x, ly, left_inner_w, scaler.s(34.0), Color::new(0.08, 0.22, 0.14, 0.95), Palette::NEON_GREEN, 1.5);
                fonts.draw_ui_bold_centered("PROMOTION READY! PRESS [P] TO ADVANCE TIER", left_inner_x + left_inner_w * 0.5, ly + scaler.s(22.0), scaler.font_s(10.5), Palette::NEON_GREEN);
                ly += scaler.s(44.0);
            }
        }
    } else {
        let lock_msg = format!("LOCKED — UNLOCK BY PROMOTING FROM TIER {}", selected_tier - 1);
        fonts.draw_ui_bold(&lock_msg, left_inner_x, ly + scaler.s(14.0), scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
        ly += scaler.s(24.0);
    }

    draw_rectangle(left_inner_x, ly, left_inner_w, 1.0, Palette::UI_CARD_BORDER);
    ly += scaler.s(14.0);

    // Active Vehicle Card for Selected Tier
    fonts.draw_ui_bold("ASSIGNED VEHICLE & GARAGE", left_inner_x, ly + scaler.s(12.0), scaler.font_s(12.0), Palette::NEON_CYAN);
    ly += scaler.s(22.0);

    let tier_models = get_models_for_module_and_tier(&career.module_id, selected_tier as u8);
    let unlocked_count = tier_models.iter().filter(|m| career.is_car_unlocked(m.id, false)).count();
    let total_cars = tier_models.len();

    let car_summary = format!("{}/{} Tier {} Models Unlocked", unlocked_count, total_cars, selected_tier);
    fonts.draw_ui_regular(&car_summary, left_inner_x, ly + scaler.s(12.0), scaler.font_s(11.0), Palette::WHITE);
    ly += scaler.s(22.0);

    // Active car spec block
    let active_model = active_car_model_id
        .and_then(crate::catalog::find_model_by_id)
        .filter(|m| m.tier == selected_tier as u8)
        .or_else(|| tier_models.iter().find(|m| career.is_car_unlocked(m.id, false)).copied())
        .or_else(|| tier_models.first().copied());

    if let Some(m) = active_model {
        let is_unlocked = career.is_car_unlocked(m.id, false);
        let car_card_h = scaler.s(72.0);
        scaler.draw_glass_card(left_inner_x, ly, left_inner_w, car_card_h, Color::new(0.08, 0.11, 0.16, 0.90), Palette::UI_CARD_BORDER, 1.0);

        let name_col = if is_unlocked { Palette::WHITE } else { Palette::UI_TEXT_MUTED };
        fonts.draw_ui_bold(m.name, left_inner_x + scaler.s(10.0), ly + scaler.s(20.0), scaler.font_s(11.5), name_col);

        let spec_line = format!("{} BHP • {} kg • Top Speed: {} km/h • {}", m.bhp, m.weight_kg, m.top_speed_kmh, m.drivetrain);
        fonts.draw_ui_regular(&spec_line, left_inner_x + scaler.s(10.0), ly + scaler.s(38.0), scaler.font_s(10.0), Palette::UI_TEXT_MUTED);

        if is_unlocked {
            fonts.draw_ui_bold("[ACTIVE CAR]", left_inner_x + scaler.s(10.0), ly + scaler.s(56.0), scaler.font_s(10.0), Palette::NEON_GREEN);
        } else {
            let cost_str = format!("Available to buy: ${} Credits", format_number(ModuleCareerProgress::car_credit_cost(selected_tier as u8)));
            fonts.draw_ui_bold(&cost_str, left_inner_x + scaler.s(10.0), ly + scaler.s(56.0), scaler.font_s(10.0), Palette::NEON_GOLD);
        }
        ly += car_card_h + scaler.s(10.0);
    }

    // Garage Button Hint
    let garage_btn_h = scaler.s(28.0);
    scaler.draw_glass_card(left_inner_x, ly, left_inner_w, garage_btn_h, Color::new(0.10, 0.14, 0.22, 0.90), Palette::NEON_CYAN, 1.0);
    fonts.draw_ui_bold_centered("[G] INSPECT IN GARAGE SHOWROOM", left_inner_x + left_inner_w * 0.5, ly + scaler.s(18.5), scaler.font_s(10.5), Palette::NEON_CYAN);

    // -------------------------------------------------------------------------
    // RIGHT COLUMN: CHAMPIONSHIP CALENDAR & STANDINGS
    // -------------------------------------------------------------------------
    scaler.draw_glass_card(right_x, cur_y, right_w, body_h, Color::new(0.05, 0.07, 0.11, 0.95), Palette::UI_CARD_BORDER, 1.2);

    let mut ry = cur_y + scaler.s(16.0);
    let right_inner_x = right_x + scaler.s(16.0);
    let right_inner_w = right_w - scaler.s(32.0);

    // Right Column Sub-Header
    let round_count = calendar.len();
    let subhead_text = if showing_standings {
        format!("CHAMPIONSHIP STANDINGS — {} ROUNDS", round_count)
    } else {
        format!("CHAMPIONSHIP CALENDAR — {} ROUNDS", round_count)
    };
    fonts.draw_ui_bold(&subhead_text, right_inner_x, ry + scaler.s(14.0), scaler.font_s(14.0), Palette::WHITE);

    // Standings toggle badge
    let tab_hint = if showing_standings { "[TAB] SHOW CALENDAR" } else { "[TAB] SHOW STANDINGS" };
    draw_ui_bold_right(fonts, tab_hint, right_inner_x + right_inner_w, ry + scaler.s(14.0), scaler.font_s(11.0), Palette::NEON_CYAN);

    ry += scaler.s(28.0);
    draw_rectangle(right_inner_x, ry, right_inner_w, 1.0, Palette::UI_CARD_BORDER);
    ry += scaler.s(10.0);

    if showing_standings {
        // Standings View
        if let Some(c) = champ {
            fonts.draw_ui_bold("CURRENT DRIVER STANDINGS", right_inner_x, ry + scaler.s(16.0), scaler.font_s(13.0), Palette::NEON_GOLD);
            ry += scaler.s(26.0);

            // Standings Table (platform DataTable: POS / DRIVER / TEAM / WINS / POINTS)
            let table_h = (cur_y + body_h - ry - scaler.s(24.0)).max(scaler.s(120.0));
            let mut table = DataTable::new(right_inner_x, ry, right_inner_w, scaler.s(22.0), scaler.s(20.0));
            type Standing = (usize, crate::series::SeriesStandingEntry);
            table.add_column(DataColumn::new("pos", "POS", 7.0, ColumnAlign::Left, |r: &Standing| format!("#{}", r.0)));
            table.add_column(DataColumn::new("driver", "DRIVER", 30.0, ColumnAlign::Left, |r| r.1.driver_name.clone()));
            table.add_column(DataColumn::new("team", "TEAM", 35.0, ColumnAlign::Left, |r| r.1.team_name.clone()));
            table.add_column(DataColumn::new("wins", "WINS", 10.0, ColumnAlign::Right, |r| r.1.wins.to_string()));
            table.add_column(DataColumn::new("points", "POINTS", 18.0, ColumnAlign::Right, |r| format!("{} PTS", r.1.points)));
            table.set_rows(
                c.standings
                    .iter()
                    .take(8)
                    .enumerate()
                    .map(|(i, entry)| {
                        DataRow::new(entry.driver_id.clone(), (i + 1, entry.clone()))
                            .with_rank(i + 1)
                            .with_player(entry.driver_id == "player")
                    })
                    .collect(),
            );
            table.draw(&scaler, fonts, LayoutRect::new(right_inner_x, ry, right_inner_w, table_h));
        } else {
            fonts.draw_ui_regular("No active season standings yet. Launch Round 1 to start competing!", right_inner_x, ry + scaler.s(24.0), scaler.font_s(12.0), Palette::UI_TEXT_MUTED);
        }
    } else {
        // Calendar List View
        let active_round = champ.map(|c| c.current_round).unwrap_or(0);
        let season_active = champ.is_some() && !champ.unwrap().is_completed;

        let visible_slots = calendar.len();
        let slot_h = ((body_h - scaler.s(80.0)) / visible_slots as f32).clamp(scaler.s(24.0), scaler.s(36.0));
        let slot_gap = scaler.s(4.0);

        let is_calendar_focused = focus_area == CareerHubFocus::Calendar;
        let calendar_stack = VStack::new_uniform(right_inner_x, ry, right_inner_w, slot_h, slot_gap);

        for (idx, track_id) in calendar.iter().enumerate() {
            let slot_y = calendar_stack.item_rect(idx).y;
            let is_cur_slot = idx == selected_slot;
            let is_mandatory = is_slot_mandatory(selected_tier, track_id);
            let is_played = season_active && idx < active_round;
            let is_up_next = season_active && idx == active_round;

            let (bg, border, thickness) = if is_cur_slot {
                if is_calendar_focused {
                    (Color::new(0.12, 0.18, 0.28, 0.95), Palette::NEON_CYAN, 2.0)
                } else {
                    (Color::new(0.08, 0.11, 0.16, 0.85), Color::new(0.25, 0.40, 0.50, 0.70), 1.2)
                }
            } else if is_up_next {
                (Color::new(0.08, 0.16, 0.22, 0.90), Palette::NEON_GREEN, 1.4)
            } else if is_played {
                (Color::new(0.05, 0.08, 0.06, 0.80), Color::new(0.20, 0.35, 0.20, 0.60), 1.0)
            } else {
                (Color::new(0.06, 0.08, 0.12, 0.80), Palette::UI_CARD_BORDER, 1.0)
            };

            scaler.draw_glass_card(right_inner_x, slot_y, right_inner_w, slot_h, bg, border, thickness);

            // Round Number Tag
            let rnd_tag = format!("RND {:02}", idx + 1);
            fonts.draw_ui_bold(&rnd_tag, right_inner_x + scaler.s(10.0), slot_y + slot_h * 0.65, scaler.font_s(10.5), Palette::NEON_GOLD);

            // Country Flag
            let country = track_country(track_id);
            draw_country_banner(Some(country), right_inner_x + scaler.s(60.0), slot_y + (slot_h - scaler.s(16.0)) * 0.5, scaler.s(26.0), scaler.s(16.0), Some(fonts), &scaler);

            // Circuit Name & Length
            let title = track_title(track_id);
            let length_km = track_length_meters(track_id) as f32 / 1000.0;
            let name_str = format!("{} ({:.2} km)", title, length_km);
            fonts.draw_ui_bold(&name_str, right_inner_x + scaler.s(96.0), slot_y + slot_h * 0.65, scaler.font_s(11.0), Palette::WHITE);

            // Right status badge
            let right_badge_x = right_inner_x + right_inner_w - scaler.s(10.0);
            if is_played {
                draw_ui_bold_right(fonts, "[✓ FINISHED]", right_badge_x, slot_y + slot_h * 0.65, scaler.font_s(10.0), Palette::NEON_GREEN);
            } else if is_up_next {
                draw_ui_bold_right(fonts, "[► UP NEXT]", right_badge_x, slot_y + slot_h * 0.65, scaler.font_s(10.5), Palette::NEON_CYAN);
            } else if is_mandatory {
                draw_ui_bold_right(fonts, "[MANDATORY NEW]", right_badge_x, slot_y + slot_h * 0.65, scaler.font_s(10.0), Palette::NEON_GOLD);
            } else if is_cur_slot && !season_active {
                draw_ui_bold_right(fonts, "[◄ SWAP: < / > ►]", right_badge_x, slot_y + slot_h * 0.65, scaler.font_s(10.0), Palette::NEON_CYAN);
            } else {
                fonts.draw_ui_regular("[SELECTABLE]", right_badge_x - scaler.s(70.0), slot_y + slot_h * 0.65, scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
            }
        }

        if !season_active && selected_tier > 1 {
            let hint_y = calendar_stack.item_rect(visible_slots).y;
            let swap_hint = "Navigate optional slots with [UP/DOWN] and press [< / >] to swap circuits.";
            fonts.draw_ui_regular(swap_hint, right_inner_x, hint_y + scaler.s(16.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);
        }
    }

    // =========================================================================
    // 4. BOTTOM ACTION & NAVIGATION BAR
    // =========================================================================
    let footer_y = sh - scaler.s(44.0);
    let is_unlocked = selected_tier <= career.level;
    let season_in_progress = champ.is_some() && !champ.unwrap().is_completed;

    let can_rerun = champ.as_ref().map(|c| c.current_round > 0).unwrap_or(false);

    if !is_unlocked {
        draw_bottom_bar(x, full_w, footer_y, &scaler, fonts, "TIER LOCKED — COMPLETE LOWER TIERS TO UNLOCK", Palette::UI_TEXT_MUTED, is_gamepad, false);
    } else if season_in_progress {
        let cur_rnd = champ.unwrap().current_round + 1;
        let tot_rnd = champ.unwrap().total_rounds();
        let track_name = champ.unwrap().current_track_id().map(track_title).unwrap_or("Next Round");
        let txt = format!("CONTINUE CHAMPIONSHIP — ROUND {}/{} ({})", cur_rnd, tot_rnd, track_name);
        draw_bottom_bar(x, full_w, footer_y, &scaler, fonts, &txt, Palette::NEON_GREEN, is_gamepad, can_rerun);
    } else {
        let first_track = calendar.first().map(|s| track_title(s)).unwrap_or("Round 1");
        let txt = format!("ENTER CHAMPIONSHIP CUP — ROUND 1 ({})", first_track);
        draw_bottom_bar(x, full_w, footer_y, &scaler, fonts, &txt, Palette::NEON_CYAN, is_gamepad, can_rerun);
    }
}

fn draw_bottom_bar(
    x: f32,
    w: f32,
    y: f32,
    scaler: &UiScaler,
    fonts: &Fonts,
    action_text: &str,
    text_col: Color,
    is_gamepad: bool,
    can_rerun: bool,
) {
    let bar_h = scaler.s(36.0);
    let footer = ScreenFooter::new(x, y, w, bar_h);
    footer.render_frame();

    let confirm_btn = if is_gamepad { "[A]" } else { "[ENTER / SPACE]" };
    let full_action = format!("{} {}", confirm_btn, action_text);
    fonts.draw_ui_bold(&full_action, x + scaler.s(16.0), y + scaler.s(23.0), scaler.font_s(12.5), text_col);

    let shortcuts = if is_gamepad {
        if can_rerun {
            "[D-Pad / Sticks] Tier/Slot  •  [LB/RB] Tier  •  [Y] Standings  •  [X] Re-run Round  •  [B] Back"
        } else {
            "[D-Pad / Sticks] Tier/Slot  •  [LB/RB] Tier  •  [Y] Standings  •  [B] Back"
        }
    } else {
        if can_rerun {
            "[◄/►] Tier  •  [▲/▼] Slot  •  [< / >] Swap  •  [TAB] Standings  •  [R] Re-run Round  •  [ESC] Back"
        } else {
            "[◄/►] Tier  •  [▲/▼] Slot  •  [< / >] Swap  •  [TAB] Standings  •  [ESC] Back"
        }
    };
    draw_ui_regular_right(fonts, shortcuts, x + w - scaler.s(16.0), y + scaler.s(23.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);
}
