use macroquad::prelude::*;
use crate::catalog::get_models_for_module_and_tier;
use crate::profile::{draw_country_banner, ModuleCareerProgress, PlayerProfile};
use crate::tournament::ChampionshipSession;
use crate::ui::font::Fonts;
use crate::ui::scaler::UiScaler;
use crate::render::color::Palette;

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
    let next_pos = if forward {
        (cur_pos + 1) % available.len()
    } else {
        (cur_pos + available.len() - 1) % available.len()
    };

    calendar[slot_idx] = available[next_pos].to_string();
    true
}

/// Human-readable circuit title.
pub fn track_title(track_id: &str) -> &'static str {
    match track_id {
        "red_bull_ring" => "Red Bull Ring (Spielberg)",
        "zandvoort" => "Circuit Zandvoort",
        "nurburgring_gp" => "Nürburgring GP-Strecke",
        "portimao_gp" => "Autódromo do Algarve (Portimão)",
        "montreal" => "Circuit Gilles Villeneuve",
        "monza" => "Monza Autodromo Nazionale",
        "silverstone" => "Silverstone GP Circuit",
        "catalunya" => "Circuit de Barcelona-Catalunya",
        "spa" => "Circuit de Spa-Francorchamps",
        "cota" => "Circuit of the Americas (COTA)",
        "bahrain" => "Bahrain International Circuit",
        "suzuka" => "Suzuka International Racing Course",
        "interlagos" => "Autódromo de Interlagos",
        "bathurst" => "Mount Panorama (Bathurst)",
        "le_mans_sarthe" => "Circuit de la Sarthe (24h Le Mans)",
        "monaco" => "Circuit de Monaco",
        "marina_bay" => "Marina Bay Street Circuit",
        "madring" => "Circuito Madring",
        _ => "Motorsport Circuit",
    }
}

/// 2-Letter ISO country code for circuit flag banner.
pub fn track_country(track_id: &str) -> &'static str {
    match track_id {
        "red_bull_ring" => "AT",
        "zandvoort" => "NL",
        "nurburgring_gp" => "DE",
        "portimao_gp" => "PT",
        "montreal" => "CA",
        "monza" => "IT",
        "silverstone" => "GB",
        "catalunya" => "ES",
        "spa" => "BE",
        "cota" => "US",
        "bahrain" => "BH",
        "suzuka" => "JP",
        "interlagos" => "BR",
        "bathurst" => "AU",
        "le_mans_sarthe" => "FR",
        "monaco" => "MC",
        "marina_bay" => "SG",
        "madring" => "ES",
        _ => "--",
    }
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
    // 2. TIER SELECTOR TAB BAR [TIER 1 .. 5] WITH REPLAY SELECTOR [◄ Q / E ►]
    // =========================================================================
    let tab_bar_h = scaler.s(40.0);
    let tier_count = 5;
    let tab_gap = scaler.s(8.0);
    let tab_w = (full_w - tab_gap * (tier_count as f32 - 1.0)) / tier_count as f32;

    for t in 1..=tier_count {
        let tx = x + (t - 1) as f32 * (tab_w + tab_gap);
        let is_selected = t == selected_tier;
        let is_unlocked = t <= career.level;

        let (bg, border, text_col) = if is_selected {
            (Color::new(0.12, 0.18, 0.28, 0.95), Palette::NEON_CYAN, Palette::NEON_CYAN)
        } else if is_unlocked {
            (Color::new(0.06, 0.08, 0.12, 0.90), Palette::UI_CARD_BORDER, Palette::WHITE)
        } else {
            (Color::new(0.03, 0.04, 0.06, 0.80), Color::new(0.15, 0.18, 0.24, 0.50), Palette::UI_TEXT_MUTED)
        };

        draw_rectangle(tx, cur_y, tab_w, tab_bar_h, bg);
        draw_rectangle_lines(tx, cur_y, tab_w, tab_bar_h, if is_selected { 2.0 } else { 1.0 }, border);

        let tier_label = match t {
            1 => "1. GT4 CLUBMAN",
            2 => "2. FIA GT3",
            3 => "3. SRO GT2",
            4 => "4. LE MANS 90s",
            _ => "5. WEC HYPERCAR",
        };

        let display_label = if is_unlocked {
            tier_label.to_string()
        } else {
            format!("🔒 {}", tier_label)
        };

        fonts.draw_ui_bold_centered(
            &display_label,
            tx + tab_w * 0.5,
            cur_y + scaler.s(24.0),
            scaler.font_s(11.0),
            text_col,
        );

        if is_selected {
            draw_rectangle(tx, cur_y + tab_bar_h - scaler.s(3.0), tab_w, scaler.s(3.0), Palette::NEON_CYAN);
        }
    }

    cur_y += tab_bar_h + scaler.s(12.0);

    // =========================================================================
    // 3. MAIN DUAL-COLUMN BODY
    // =========================================================================
    let body_h = (sh - cur_y - scaler.s(60.0)).max(scaler.s(410.0));
    let col_gap = scaler.s(14.0);
    let left_w = full_w * 0.38;
    let right_w = full_w - left_w - col_gap;
    let left_x = x;
    let right_x = x + left_w + col_gap;

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
        if career.level >= 5 {
            fonts.draw_ui_bold("★ PINNACLE TIER REACHED — WORLD ENDURANCE APEX", left_inner_x, ly + scaler.s(14.0), scaler.font_s(11.0), Palette::NEON_GOLD);
            ly += scaler.s(24.0);
        } else {
            let next_tier = career.level + 1;
            let next_cost = ModuleCareerProgress::car_cost(next_tier as u8);
            let has_podium = (career.trophies_gold + career.trophies_silver + career.trophies_bronze) > 0;
            let has_xp = career.xp >= next_cost;

            let podium_check = if has_podium { "✓ Podium Finish Earned" } else { "✗ Requires 1+ Championship Podium" };
            let xp_check = format!("{}/{} XP (Target: {} XP)", format_number(career.xp), format_number(next_cost), format_number(next_cost));

            let pod_col = if has_podium { Palette::NEON_GREEN } else { Palette::UI_TEXT_MUTED };
            let xp_col = if has_xp { Palette::NEON_GREEN } else { Palette::NEON_GOLD };

            fonts.draw_ui_regular(podium_check, left_inner_x, ly + scaler.s(14.0), scaler.font_s(11.0), pod_col);
            fonts.draw_ui_regular(&xp_check, left_inner_x, ly + scaler.s(32.0), scaler.font_s(11.0), xp_col);
            ly += scaler.s(42.0);

            // Progress Bar towards tier promotion
            let progress_ratio = (career.xp as f32 / next_cost as f32).clamp(0.0, 1.0);
            draw_rectangle(left_inner_x, ly, left_inner_w, scaler.s(6.0), Color::new(0.12, 0.15, 0.20, 0.90));
            draw_rectangle(left_inner_x, ly, left_inner_w * progress_ratio, scaler.s(6.0), Palette::NEON_CYAN);
            ly += scaler.s(14.0);

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

    let tier_models = get_models_for_module_and_tier("gt", selected_tier as u8);
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
            let cost_str = format!("Available to buy: {} XP", format_number(ModuleCareerProgress::car_cost(selected_tier as u8)));
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
    let subhead_text = format!("CHAMPIONSHIP CALENDAR — {} ROUNDS", round_count);
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

            // Table Header
            fonts.draw_ui_bold("POS", right_inner_x + scaler.s(8.0), ry, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
            fonts.draw_ui_bold("DRIVER", right_inner_x + scaler.s(50.0), ry, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
            fonts.draw_ui_bold("TEAM", right_inner_x + scaler.s(220.0), ry, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
            fonts.draw_ui_bold("WINS", right_inner_x + right_inner_w - scaler.s(120.0), ry, scaler.font_s(11.0), Palette::UI_TEXT_MUTED);
            draw_ui_bold_right(fonts, "POINTS", right_inner_x + right_inner_w, ry, scaler.font_s(11.0), Palette::NEON_GOLD);
            ry += scaler.s(14.0);
            draw_rectangle(right_inner_x, ry, right_inner_w, 1.0, Palette::UI_CARD_BORDER);
            ry += scaler.s(8.0);

            for (i, entry) in c.standings.iter().enumerate().take(8) {
                let row_col = if entry.driver_id == "player" { Palette::NEON_CYAN } else { Palette::WHITE };
                let pos_str = format!("#{}", i + 1);
                fonts.draw_ui_bold(&pos_str, right_inner_x + scaler.s(8.0), ry + scaler.s(12.0), scaler.font_s(11.0), row_col);
                fonts.draw_ui_bold(&entry.driver_name, right_inner_x + scaler.s(50.0), ry + scaler.s(12.0), scaler.font_s(11.0), row_col);
                fonts.draw_ui_regular(&entry.team_name, right_inner_x + scaler.s(220.0), ry + scaler.s(12.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);
                fonts.draw_ui_bold(&entry.wins.to_string(), right_inner_x + right_inner_w - scaler.s(110.0), ry + scaler.s(12.0), scaler.font_s(11.0), Palette::WHITE);
                draw_ui_bold_right(fonts, &format!("{} PTS", entry.points), right_inner_x + right_inner_w, ry + scaler.s(12.0), scaler.font_s(11.5), Palette::NEON_GOLD);
                ry += scaler.s(22.0);
            }
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

        for (idx, track_id) in calendar.iter().enumerate() {
            let is_cur_slot = idx == selected_slot;
            let is_mandatory = is_slot_mandatory(selected_tier, track_id);
            let is_played = season_active && idx < active_round;
            let is_up_next = season_active && idx == active_round;

            let (bg, border, thickness) = if is_cur_slot {
                (Color::new(0.12, 0.18, 0.28, 0.95), Palette::NEON_CYAN, 1.8)
            } else if is_up_next {
                (Color::new(0.08, 0.16, 0.22, 0.90), Palette::NEON_GREEN, 1.4)
            } else if is_played {
                (Color::new(0.05, 0.08, 0.06, 0.80), Color::new(0.20, 0.35, 0.20, 0.60), 1.0)
            } else {
                (Color::new(0.06, 0.08, 0.12, 0.80), Palette::UI_CARD_BORDER, 1.0)
            };

            scaler.draw_glass_card(right_inner_x, ry, right_inner_w, slot_h, bg, border, thickness);

            // Round Number Tag
            let rnd_tag = format!("RND {:02}", idx + 1);
            fonts.draw_ui_bold(&rnd_tag, right_inner_x + scaler.s(10.0), ry + slot_h * 0.65, scaler.font_s(10.5), Palette::NEON_GOLD);

            // Country Flag
            let country = track_country(track_id);
            draw_country_banner(Some(country), right_inner_x + scaler.s(60.0), ry + (slot_h - scaler.s(16.0)) * 0.5, scaler.s(26.0), scaler.s(16.0), Some(fonts), &scaler);

            // Circuit Name & Length
            let title = track_title(track_id);
            let length_km = track_length_meters(track_id) as f32 / 1000.0;
            let name_str = format!("{} ({:.2} km)", title, length_km);
            fonts.draw_ui_bold(&name_str, right_inner_x + scaler.s(96.0), ry + slot_h * 0.65, scaler.font_s(11.0), Palette::WHITE);

            // Right status badge
            let right_badge_x = right_inner_x + right_inner_w - scaler.s(10.0);
            if is_played {
                draw_ui_bold_right(fonts, "[✓ FINISHED]", right_badge_x, ry + slot_h * 0.65, scaler.font_s(10.0), Palette::NEON_GREEN);
            } else if is_up_next {
                draw_ui_bold_right(fonts, "[► UP NEXT]", right_badge_x, ry + slot_h * 0.65, scaler.font_s(10.5), Palette::NEON_CYAN);
            } else if is_mandatory {
                draw_ui_bold_right(fonts, "[MANDATORY NEW]", right_badge_x, ry + slot_h * 0.65, scaler.font_s(10.0), Palette::NEON_GOLD);
            } else if is_cur_slot && !season_active {
                draw_ui_bold_right(fonts, "[◄ SWAP: < / > ►]", right_badge_x, ry + slot_h * 0.65, scaler.font_s(10.0), Palette::NEON_CYAN);
            } else {
                fonts.draw_ui_regular("[SELECTABLE]", right_badge_x - scaler.s(70.0), ry + slot_h * 0.65, scaler.font_s(10.0), Palette::UI_TEXT_MUTED);
            }

            ry += slot_h + slot_gap;
        }

        if !season_active && selected_tier > 1 {
            let swap_hint = "Navigate optional slots with [UP/DOWN] and press [< / >] to swap circuits.";
            fonts.draw_ui_regular(swap_hint, right_inner_x, ry + scaler.s(16.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);
        }
    }

    // =========================================================================
    // 4. BOTTOM ACTION & NAVIGATION BAR
    // =========================================================================
    let footer_y = sh - scaler.s(44.0);
    let is_unlocked = selected_tier <= career.level;
    let season_in_progress = champ.is_some() && !champ.unwrap().is_completed;

    if !is_unlocked {
        draw_bottom_bar(x, full_w, footer_y, &scaler, fonts, "TIER LOCKED — COMPLETE LOWER TIERS TO UNLOCK", Palette::UI_TEXT_MUTED, Palette::UI_CARD_BORDER, is_gamepad);
    } else if season_in_progress {
        let cur_rnd = champ.unwrap().current_round + 1;
        let tot_rnd = champ.unwrap().total_rounds();
        let track_name = champ.unwrap().current_track_id().map(track_title).unwrap_or("Next Round");
        let txt = format!("CONTINUE CHAMPIONSHIP — ROUND {}/{} ({})", cur_rnd, tot_rnd, track_name);
        draw_bottom_bar(x, full_w, footer_y, &scaler, fonts, &txt, Palette::NEON_GREEN, Palette::NEON_GREEN, is_gamepad);
    } else {
        let first_track = calendar.first().map(|s| track_title(s)).unwrap_or("Round 1");
        let txt = format!("ENTER CHAMPIONSHIP CUP — ROUND 1 ({})", first_track);
        draw_bottom_bar(x, full_w, footer_y, &scaler, fonts, &txt, Palette::NEON_CYAN, Palette::NEON_CYAN, is_gamepad);
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
    border_col: Color,
    is_gamepad: bool,
) {
    let bar_h = scaler.s(36.0);
    scaler.draw_glass_card(x, y, w, bar_h, Color::new(0.06, 0.08, 0.13, 0.95), border_col, 1.2);

    let confirm_btn = if is_gamepad { "[A]" } else { "[ENTER / SPACE]" };
    let full_action = format!("{} {}", confirm_btn, action_text);
    fonts.draw_ui_bold(&full_action, x + scaler.s(16.0), y + scaler.s(23.0), scaler.font_s(12.5), text_col);

    let shortcuts = if is_gamepad {
        "[LB/RB] Tier  •  [D-Pad] Slot/Swap  •  [Y] Standings  •  [X] Reset  •  [B] Back"
    } else {
        "[Q/E] Tier  •  [UP/DN] Slot  •  [< / >] Swap  •  [TAB] Standings  •  [X] Reset  •  [ESC] Back"
    };
    draw_ui_regular_right(fonts, shortcuts, x + w - scaler.s(16.0), y + scaler.s(23.0), scaler.font_s(10.5), Palette::UI_TEXT_MUTED);
}
