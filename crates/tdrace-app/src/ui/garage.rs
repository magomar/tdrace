use macroquad::color::Color;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines};

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::catalog::{get_models_for_module, get_models_for_module_and_tier, RealCarModel};
use crate::profile::{is_gt_garage_module, ModuleCareerProgress, PlayerProfile, GT_GARAGE_COMPOUNDS};
use crate::render::color::{CarColorScheme, Palette};
use crate::render::lateral::render_real_car_lateral_by_id;
use tdrace_core::physics::surface::CompoundId;
use cabinet::ui::{
    CardGrid, CardGridItem, FilterBar, FilterBarStyle, HeroActionButton, LayoutRect, MetricBar,
    ScreenFooter, SwatchPicker,
};

/// Viewing projection in the Garage showroom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GarageViewMode {
    #[default]
    Lateral,
    TopDownTurntable,
}

/// Returns the rectangle (x, y, w, h) of the Select Car action button in the Garage.
pub fn garage_select_button_rect(sw: f32, sh: f32) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let right_w = (sw * 0.38).clamp(scaler.s(320.0), scaler.s(450.0));
    let right_x = sw - right_w - scaler.safe_pad_x;
    let btn_h = scaler.s(52.0);
    let btn_y = sh - scaler.s(82.0);
    (right_x, btn_y, right_w, btn_h)
}

/// Returns the rectangle (x, y, w, h) of a vehicle carousel pill/card in the Garage.
pub fn garage_car_card_rect(sw: f32, sh: f32, idx: usize, total: usize) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let total_cars = total.max(1);
    let stage_w = sw * 0.56 - scaler.safe_pad_x;
    let card_w = ((stage_w - scaler.s(8.0) * (total_cars - 1) as f32) / total_cars as f32).clamp(scaler.s(60.0), scaler.s(160.0));
    let card_h = scaler.s(58.0);
    let start_x = scaler.safe_pad_x;
    let card_x = start_x + (card_w + scaler.s(8.0)) * (idx as f32);
    let card_y = sh - scaler.s(146.0);
    (card_x, card_y, card_w, card_h)
}

/// What the TYRES row of the garage shows for the car in focus (spec 098).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageTyres {
    /// No TYRES row: karts, rally, autocross, extreme off-road and the Classic module.
    Hidden,
    /// A GT car: the player picks one of the garage slicks; this is the one fitted.
    Select(CompoundId),
    /// The compound is shown but cannot be changed (the series rule of a stock car, or a locked car).
    ReadOnly { compound: CompoundId, note: &'static str },
}

/// Decides the TYRES row for the car in focus. A GT car shows the profile's choice, or its preset
/// compound when it has none. A stock car shows its preset compound read-only. While the vehicle is
/// locked for the race (`car_change_allowed == false`) a GT car shows its compound read-only too,
/// because a confirmed change would not reach the race that is already built.
pub fn garage_tyres(model: Option<&RealCarModel>, profile: &PlayerProfile, car_change_allowed: bool) -> GarageTyres {
    let Some(model) = model else {
        return GarageTyres::Hidden;
    };
    let preset = model.to_car_config().wheels[0].compound.id;
    if is_gt_garage_module(model.module_id) {
        let compound = profile.garage_compound(model.module_id, model.id).unwrap_or(preset);
        if car_change_allowed {
            GarageTyres::Select(compound)
        } else {
            GarageTyres::ReadOnly { compound, note: "vehicle locked" }
        }
    } else if model.module_id == "nascar" {
        GarageTyres::ReadOnly { compound: preset, note: "series rule" }
    } else {
        GarageTyres::Hidden
    }
}

/// The next (or previous) garage slick, wrapping around. An unknown compound starts at Medium.
pub fn garage_next_compound(current: CompoundId, forward: bool) -> CompoundId {
    let n = GT_GARAGE_COMPOUNDS.len();
    let idx = GT_GARAGE_COMPOUNDS.iter().position(|c| *c == current).unwrap_or(1);
    GT_GARAGE_COMPOUNDS[if forward { (idx + 1) % n } else { (idx + n - 1) % n }]
}

/// Short upper-case label of a compound for the TYRES row: SOFT, MEDIUM, HARD.
pub fn garage_compound_label(id: CompoundId) -> String {
    id.name().split(' ').next().unwrap_or("").to_uppercase()
}

/// One line on what a compound trades off, shown under the TYRES row.
pub fn garage_compound_tradeoff(id: CompoundId) -> &'static str {
    match id {
        CompoundId::SoftSlick => "More grip, wears faster when you slide.",
        CompoundId::HardSlick => "Less grip, forgiving and slow to wear.",
        _ => "Balanced grip and wear.",
    }
}

/// Returns the rectangle (x, y, w, h) of the SOFT / MEDIUM / HARD option `idx` (0..3) in the garage
/// TYRES row. The row is anchored to the bottom of the specification card, below the stat bars.
pub fn garage_tyre_pill_rect(sw: f32, sh: f32, idx: usize) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let right_w = (sw * 0.38).clamp(scaler.s(320.0), scaler.s(450.0));
    let right_x = sw - right_w - scaler.safe_pad_x;
    let card_bottom = scaler.s(64.0) + sh * 0.68;
    let pad = scaler.s(14.0);
    let gap = scaler.s(8.0);
    let n = GT_GARAGE_COMPOUNDS.len();
    let pill_w = (right_w - 2.0 * pad - gap * (n - 1) as f32) / n as f32;
    let x = right_x + pad + (pill_w + gap) * idx as f32;
    let y = card_bottom - scaler.s(58.0);
    (x, y, pill_w, scaler.s(24.0))
}

/// Renders the Interactive Garage & Showroom screen (`GameState::Garage`).
#[allow(clippy::too_many_arguments)]
pub fn render_garage_screen(
    fonts: &Fonts,
    active_module_id: &str,
    garage_tier: u8,
    garage_car_idx: usize,
    garage_view_mode: GarageViewMode,
    garage_turntable_angle: f32,
    garage_brake_heat: f32,
    garage_revving: bool,
    garage_rev_rpm: f32,
    garage_gallery_mode: bool,
    garage_gallery_filter: usize,
    garage_gallery_sel: usize,
    is_dev_mode: bool,
    unlocked_tier: u32,
    career_progress: Option<&ModuleCareerProgress>,
    available_credits: u64,
    module_locked: bool,
    car_change_allowed: bool,
    tyres: GarageTyres,
    owned_cars: &[String],
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // 1. Dark Showroom Backdrop with subtle carbon weave pattern
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Module branding
    let (mod_title, mod_accent) = match active_module_id {
        "gt" | "gt_challenge" | "f1" => ("GRAND TOURING CHALLENGE", Palette::NEON_CYAN),
        "rallycross" | "rally" | "rx" => ("RALLYCROSS", Palette::NEON_GOLD),
        "kart" => ("KARTING & MICRO-RACERS", Palette::NEON_GREEN),
        "nascar" => ("STOCK CAR RACING", Palette::YELLOW),
        "extreme_offroad" => ("EXTREME OFF-ROAD & STUNT ARENAS", Palette::NEON_ORANGE),
        "autocross" => ("CONTINENTAL AUTOCROSS", Palette::NEON_ORANGE),
        _ => ("CLASSIC ARCADE MOTORSPORT", Palette::NEON_GOLD),
    };

    // Global Fleet Gallery Overlay
    if garage_gallery_mode {
        render_fleet_gallery(
            fonts,
            &scaler,
            sw,
            sh,
            garage_gallery_filter,
            garage_gallery_sel,
            unlocked_tier,
            is_dev_mode,
        );
        return;
    }

    // Header Title
    let title = "MOTORSPORT GARAGE & SHOWROOM";
    fonts.draw_display_centered_with_shadow(
        title,
        sw * 0.5,
        scaler.s(28.0),
        scaler.font_s(24.0),
        Palette::NEON_GOLD,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    // Header Subtitle: Module & Category info
    let tier_models = get_models_for_module_and_tier(active_module_id, garage_tier);
    let active_model: Option<&RealCarModel> = tier_models.get(garage_car_idx).copied();
    let category_name = active_model.map(|m| m.category_name).unwrap_or("Competition Spec");

    // In LAN the module is fixed to the host discipline, so its key hint is hidden.
    let module_keys = if module_locked { "" } else { "  [◄ 1..5 ►]" };
    let is_rx_module = active_module_id == "rallycross" || active_module_id == "rally" || active_module_id == "rx";
    let max_ranked_tier: u8 = if is_rx_module { 6 } else { 5 };
    let is_heritage_tier = is_rx_module && garage_tier == 7;
    let tier_label = if is_heritage_tier {
        "HERITAGE".to_string()
    } else if garage_tier > max_ranked_tier {
        "UNRANKED".to_string()
    } else {
        format!("TIER {}", garage_tier)
    };
    let module_subtitle = if active_module_id == "classic" {
        "MODULE: CLASSIC ARCADE MOTORSPORT • FANTASY ARCADE ROSTER [◄ A / D ►]".to_string()
    } else if let Some(cp) = career_progress {
        format!(
            "MODULE: {}{}  •  {}: {}  [◄ Q/E ►]  •  DISCIPLINE XP: {} XP  •  WALLET: ${} CR",
            mod_title, module_keys, tier_label, category_name.to_uppercase(), cp.xp, available_credits
        )
    } else {
        format!(
            "MODULE: {}{}  •  {}: {}  [◄ Q/E ►]",
            mod_title, module_keys, tier_label, category_name.to_uppercase()
        )
    };
    fonts.draw_ui_regular_centered(
        &module_subtitle,
        sw * 0.5,
        scaler.s(48.0),
        scaler.font_s(12.0),
        Palette::UI_TEXT_MUTED,
    );

    // Geometry layout
    let stage_x = scaler.safe_pad_x;
    let stage_y = scaler.s(64.0);
    let stage_w = sw * 0.58 - scaler.safe_pad_x;
    let stage_h = sh * 0.58;

    // Hero Showroom Stage Card
    scaler.draw_glass_card(stage_x, stage_y, stage_w, stage_h, Palette::UI_CARD_BG, mod_accent, 1.4);

    // Ambient spotlight glow behind vehicle, kept inside the stage (it used to spill over the header)
    let center_x = stage_x + stage_w * 0.50;
    let center_y = stage_y + stage_h * 0.52;
    cabinet::ui::scaler::begin_clip_rect(stage_x + 1.0, stage_y + 1.0, stage_w - 2.0, stage_h - 2.0);
    draw_circle(center_x, center_y - scaler.s(20.0), scaler.s(220.0), Color::new(0.08, 0.16, 0.28, 0.22));
    draw_circle(center_x, center_y - scaler.s(10.0), scaler.s(150.0), Color::new(0.12, 0.22, 0.35, 0.18));
    cabinet::ui::scaler::end_clip_rect();

    let is_tier_unlocked = active_module_id == "classic" || is_dev_mode || is_heritage_tier || garage_tier as u32 <= unlocked_tier;

    // Hero Stage Header Badges
    let view_mode_str = match garage_view_mode {
        GarageViewMode::Lateral => "[2D LATERAL VIEW (ACTIVE)]  [TAB: TURNTABLE]",
        GarageViewMode::TopDownTurntable => "[360° TOP-DOWN VIEW (ACTIVE)]  [TAB: LATERAL]",
    };
    fonts.draw_ui_bold(
        view_mode_str,
        stage_x + scaler.s(16.0),
        stage_y + scaler.s(20.0),
        scaler.font_s(11.0),
        Palette::NEON_CYAN,
    );

    let unlock_badge = if is_tier_unlocked {
        "✓ UNLOCKED FOR COMPETITION"
    } else {
        "🔒 LOCKED IN CAREER"
    };
    let unlock_col = if is_tier_unlocked {
        Palette::NEON_GREEN
    } else {
        Palette::RED
    };
    let unlock_w = fonts.measure_ui_bold(unlock_badge, scaler.font_s(11.0)).width;
    fonts.draw_ui_bold(
        unlock_badge,
        stage_x + stage_w - scaler.s(16.0) - unlock_w,
        stage_y + scaler.s(20.0),
        scaler.font_s(11.0),
        unlock_col,
    );

    // Render Vehicle in Hero Stage
    if let Some(model) = active_model {
        fonts.draw_ui_bold(
            model.name,
            stage_x + scaler.s(16.0),
            stage_y + scaler.s(38.0),
            scaler.font_s(15.0),
            Palette::WHITE,
        );
        let hero_bio_w = stage_w - scaler.s(32.0);
        fonts.draw_ui_regular_multiline(
            model.history_bio,
            stage_x + scaler.s(16.0),
            stage_y + scaler.s(55.0),
            scaler.font_s(10.0),
            scaler.s(13.5),
            hero_bio_w,
            Palette::UI_TEXT_MUTED,
        );

        let scheme = CarColorScheme {
            primary: model.primary_color,
            secondary: model.secondary_color,
            helmet: Palette::NEON_GOLD,
        };

        match garage_view_mode {
            GarageViewMode::Lateral => {
                let lateral_scale = scaler.s(1.65);
                let rev_factor = if garage_revving { garage_rev_rpm } else { garage_brake_heat };
                render_real_car_lateral_by_id(
                    model.id,
                    &scheme,
                    center_x,
                    center_y,
                    lateral_scale,
                    rev_factor,
                    true,
                );
            }
            GarageViewMode::TopDownTurntable => {
                // Top-down turntable rendering with smooth rotation
                draw_circle(center_x, center_y + scaler.s(10.0), scaler.s(70.0), Color::new(0.06, 0.08, 0.12, 0.85));
                draw_circle_lines(center_x, center_y + scaler.s(10.0), scaler.s(70.0), 1.5, Color::new(0.25, 0.35, 0.50, 0.50));

                if let Some(texture) = crate::render::vehicle_assets::get_vehicle_topdown_texture(model.id, scheme.primary, scheme.secondary) {
                    let dest_w = scaler.s(220.0);
                    let dest_h = dest_w * (texture.height() / texture.width());
                    macroquad::texture::draw_texture_ex(
                        &texture,
                        center_x - dest_w * 0.5,
                        center_y + scaler.s(10.0) - dest_h * 0.5,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        macroquad::texture::DrawTextureParams {
                            dest_size: Some(macroquad::math::Vec2::new(dest_w, dest_h)),
                            rotation: garage_turntable_angle,
                            pivot: Some(macroquad::math::Vec2::new(center_x, center_y + scaler.s(10.0))),
                            ..Default::default()
                        },
                    );
                } else {
                    let turntable_scale = scaler.s(1.10);
                    render_real_car_lateral_by_id(
                        model.id,
                        &scheme,
                        center_x,
                        center_y,
                        turntable_scale,
                        garage_brake_heat,
                        false,
                    );
                }

                let angle_deg = (garage_turntable_angle * 180.0 / std::f32::consts::PI) % 360.0;
                fonts.draw_ui_regular(
                    &format!("Turntable Angle: {:.0}°", angle_deg),
                    stage_x + scaler.s(16.0),
                    stage_y + stage_h - scaler.s(45.0),
                    scaler.font_s(10.0),
                    Palette::UI_TEXT_MUTED,
                );
            }
        }
    }

    // Sound-Stage Rev Sampler at Bottom of Hero Stage
    let tacho_x = stage_x + scaler.s(20.0);
    let tacho_y = stage_y + stage_h - scaler.s(28.0);
    let tacho_w = stage_w - scaler.s(40.0);
    let tacho_h = scaler.s(8.0);

    let rev_prompt = if module_locked {
        "HOLD [SPACE] TO REV ENGINE & INSPECT EXHAUST HEAT"
    } else {
        "HOLD [SPACE] TO REV ENGINE & INSPECT EXHAUST HEAT  •  [C] FLEET GALLERY"
    };
    fonts.draw_ui_bold(
        rev_prompt,
        tacho_x,
        tacho_y - scaler.s(6.0),
        scaler.font_s(10.0),
        if garage_revving { Palette::NEON_GOLD } else { Palette::UI_TEXT_MUTED },
    );

    draw_rectangle(tacho_x, tacho_y, tacho_w, tacho_h, Color::new(0.08, 0.10, 0.14, 0.90));
    draw_rectangle_lines(tacho_x, tacho_y, tacho_w, tacho_h, 1.0, Color::new(0.20, 0.25, 0.35, 0.80));
    if garage_rev_rpm > 0.01 {
        let fill_w = tacho_w * garage_rev_rpm.clamp(0.0, 1.0);
        let bar_col = if garage_rev_rpm > 0.85 {
            Palette::RED
        } else if garage_rev_rpm > 0.60 {
            Palette::NEON_GOLD
        } else {
            Palette::NEON_CYAN
        };
        draw_rectangle(tacho_x, tacho_y, fill_w, tacho_h, bar_col);
    }

    // Vehicle Carousel / Selection Cards below Hero Stage
    for (i, model) in tier_models.iter().enumerate() {
        let (cx, cy, cw, ch) = garage_car_card_rect(sw, sh, i, tier_models.len());
        let is_card_selected = i == garage_car_idx;
        let card_bg = if is_card_selected {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_CARD_BG
        };
        let card_border = if is_card_selected {
            Palette::NEON_GOLD
        } else {
            Palette::UI_CARD_BORDER
        };
        scaler.draw_glass_card(cx, cy, cw, ch, card_bg, card_border, if is_card_selected { 2.2 } else { 1.0 });

        let num_str = format!("#{}", i + 1);
        fonts.draw_ui_bold(&num_str, cx + scaler.s(8.0), cy + scaler.s(15.0), scaler.font_s(9.5), Palette::NEON_CYAN);

        let is_rx_tier7 = (active_module_id == "rallycross" || active_module_id == "rally" || active_module_id == "rx") && model.tier == 7;
        let is_owned = active_module_id == "classic"
            || is_dev_mode
            || owned_cars.iter().any(|c| c == model.id);

        if is_owned {
            fonts.draw_ui_bold(
                if is_rx_tier7 { "HERITAGE" } else { "OWNED" },
                cx + cw - scaler.s(if is_rx_tier7 { 58.0 } else { 48.0 }),
                cy + scaler.s(15.0),
                scaler.font_s(8.5),
                if is_rx_tier7 { Palette::NEON_GOLD } else { Palette::NEON_GREEN },
            );
        } else if let Some(cp) = career_progress {
            let cost = ModuleCareerProgress::car_credit_cost_for_car(model.id, active_module_id, model.tier);
            let is_affordable = available_credits >= cost;
            let is_grassroots_starter = (active_module_id == "kart" || active_module_id == "autocross" || active_module_id == "rallycross" || active_module_id == "rally") && model.tier == 1;
            let tag = if is_grassroots_starter && is_affordable {
                format!("${} CR (AFFORDABLE)", cost)
            } else if is_grassroots_starter {
                format!("${} CR (UNAFFORDABLE)", cost)
            } else {
                format!("${} CR", cost)
            };
            let col = if is_affordable && cp.level >= model.tier as u32 { Palette::NEON_GOLD } else { Palette::RED };
            let tag_w = fonts.measure_ui_bold(&tag, scaler.font_s(7.5)).width;
            fonts.draw_ui_bold(&tag, cx + cw - tag_w - scaler.s(8.0), cy + scaler.s(15.0), scaler.font_s(7.5), col);
        }

        fonts.draw_ui_bold(
            &fonts.fit_ui_bold(model.name, scaler.font_s(10.0), cw - scaler.s(16.0)),
            cx + scaler.s(8.0),
            cy + scaler.s(30.0),
            scaler.font_s(10.0),
            Palette::WHITE,
        );

        let pi = model.performance_index().round() as u32;
        let sub_info = format!("{} BHP • {} • PI {}", model.bhp, model.drivetrain, pi);
        fonts.draw_ui_regular(
            &sub_info,
            cx + scaler.s(8.0),
            cy + scaler.s(45.0),
            scaler.font_s(8.5),
            Palette::UI_TEXT_MUTED,
        );
    }

    // Right Panel: Telemetry, Specs & Selection Action Card
    let right_w = (sw * 0.38).clamp(scaler.s(320.0), scaler.s(450.0));
    let right_x = sw - right_w - scaler.safe_pad_x;
    let right_y = stage_y;
    let specs_card_h = sh * 0.68;

    scaler.draw_glass_card(right_x, right_y, right_w, specs_card_h, Palette::UI_CARD_BG, Palette::NEON_CYAN, 1.4);

    let mut cur_ry = right_y + scaler.s(18.0);
    fonts.draw_ui_bold(
        "VEHICLE SPECIFICATION TELEMETRY",
        right_x + scaler.s(14.0),
        cur_ry,
        scaler.font_s(11.0),
        Palette::NEON_CYAN,
    );
    if let Some(model) = active_model {
        fonts.draw_ui_bold(
            model.manufacturer.to_uppercase().as_str(),
            right_x + right_w - scaler.s(120.0),
            cur_ry,
            scaler.font_s(10.0),
            Palette::NEON_GOLD,
        );
        cur_ry += scaler.s(22.0);

        // Car Title & Dynamic Price
        fonts.draw_ui_bold(
            model.name,
            right_x + scaler.s(14.0),
            cur_ry,
            scaler.font_s(14.5),
            Palette::WHITE,
        );

        let model_cost = ModuleCareerProgress::car_credit_cost_for_car(model.id, active_module_id, model.tier);
        let is_owned_active = active_module_id == "classic"
            || is_dev_mode
            || owned_cars.iter().any(|c| c == model.id);
        if !is_owned_active {
            let price_str = format!("${} CR", model_cost);
            let price_w = fonts.measure_ui_bold(&price_str, scaler.font_s(13.0)).width;
            let price_col = if available_credits >= model_cost { Palette::NEON_GREEN } else { Palette::RED };
            fonts.draw_ui_bold(
                &price_str,
                right_x + right_w - scaler.s(14.0) - price_w,
                cur_ry,
                scaler.font_s(13.0),
                price_col,
            );
        } else {
            let owned_str = if is_heritage_tier { "HERITAGE" } else { "OWNED" };
            let owned_w = fonts.measure_ui_bold(owned_str, scaler.font_s(12.0)).width;
            fonts.draw_ui_bold(
                owned_str,
                right_x + right_w - scaler.s(14.0) - owned_w,
                cur_ry,
                scaler.font_s(12.0),
                if is_heritage_tier { Palette::NEON_GOLD } else { Palette::NEON_GREEN },
            );
        }
        cur_ry += scaler.s(18.0);

        // Performance Index & Valuation Badge row
        let pi = model.performance_index().round() as u32;
        let irel = model.cohort_relative_performance_index();
        let tier_text = format!("TIER {}", model.tier);
        let tier_font_size = scaler.font_s(8.5);
        let tier_text_w = fonts.measure_ui_bold(&tier_text, tier_font_size).width;
        let tier_pill_w = tier_text_w + scaler.s(12.0);
        let pill_h = scaler.s(16.0);
        let mut pill_x = right_x + scaler.s(14.0);
        let pill_y = cur_ry;

        // Tier pill
        draw_rectangle(pill_x, pill_y, tier_pill_w, pill_h, Color::new(0.95, 0.70, 0.05, 0.15));
        draw_rectangle_lines(pill_x, pill_y, tier_pill_w, pill_h, 1.0, Palette::NEON_GOLD);
        fonts.draw_ui_bold(
            &tier_text,
            pill_x + scaler.s(6.0),
            pill_y + scaler.s(11.5),
            tier_font_size,
            Palette::NEON_GOLD,
        );
        pill_x += tier_pill_w + scaler.s(6.0);

        // Perf Index pill
        let pi_text = format!("PERF INDEX: {}", pi);
        let pi_font_size = scaler.font_s(8.5);
        let pi_text_w = fonts.measure_ui_bold(&pi_text, pi_font_size).width;
        let pi_pill_w = pi_text_w + scaler.s(12.0);
        draw_rectangle(pill_x, pill_y, pi_pill_w, pill_h, Color::new(0.0, 0.90, 0.95, 0.15));
        draw_rectangle_lines(pill_x, pill_y, pi_pill_w, pill_h, 1.0, Palette::NEON_CYAN);
        fonts.draw_ui_bold(
            &pi_text,
            pill_x + scaler.s(6.0),
            pill_y + scaler.s(11.5),
            pi_font_size,
            Palette::NEON_CYAN,
        );
        pill_x += pi_pill_w + scaler.s(6.0);

        // Valuation Badge
        if irel <= 0.15 {
            let badge_text = "[VALUE STARTER]";
            let badge_font_size = scaler.font_s(8.5);
            let badge_text_w = fonts.measure_ui_bold(badge_text, badge_font_size).width;
            let badge_w = badge_text_w + scaler.s(10.0);
            draw_rectangle(pill_x, pill_y, badge_w, pill_h, Color::new(0.05, 0.85, 0.40, 0.15));
            draw_rectangle_lines(pill_x, pill_y, badge_w, pill_h, 1.0, Palette::NEON_GREEN);
            fonts.draw_ui_bold(
                badge_text,
                pill_x + scaler.s(5.0),
                pill_y + scaler.s(11.5),
                badge_font_size,
                Palette::NEON_GREEN,
            );
        } else if irel >= 0.85 {
            let badge_text = "[COMPETITION SPEC]";
            let badge_font_size = scaler.font_s(8.5);
            let badge_text_w = fonts.measure_ui_bold(badge_text, badge_font_size).width;
            let badge_w = badge_text_w + scaler.s(10.0);
            draw_rectangle(pill_x, pill_y, badge_w, pill_h, Color::new(0.95, 0.70, 0.05, 0.15));
            draw_rectangle_lines(pill_x, pill_y, badge_w, pill_h, 1.0, Palette::NEON_GOLD);
            fonts.draw_ui_bold(
                badge_text,
                pill_x + scaler.s(5.0),
                pill_y + scaler.s(11.5),
                badge_font_size,
                Palette::NEON_GOLD,
            );
        }

        cur_ry += scaler.s(22.0);

        // Key Engine / Spec telemetry lines
        let t_spec = format!("Engine: {}  •  Power: {} BHP @ {} Nm", model.engine_desc, model.bhp, model.torque_nm);
        fonts.draw_ui_regular(&t_spec, right_x + scaler.s(14.0), cur_ry, scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
        cur_ry += scaler.s(14.0);

        let t_perf = format!("Mass: {} kg ({})  •  Top Speed: {} km/h  •  0-100: {:.1}s", model.weight_kg, model.drivetrain, model.top_speed_kmh, model.accel_0_100);
        fonts.draw_ui_regular(&t_perf, right_x + scaler.s(14.0), cur_ry, scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
        cur_ry += scaler.s(14.0);

        let t_aero = format!("Aero: {}  •  Brakes: {}", model.aero_downforce, model.brakes_desc);
        fonts.draw_ui_regular(&t_aero, right_x + scaler.s(14.0), cur_ry, scaler.font_s(9.5), Palette::UI_TEXT_MUTED);
        cur_ry += scaler.s(20.0);

        // Divider Line
        draw_line(
            right_x + scaler.s(14.0),
            cur_ry,
            right_x + right_w - scaler.s(14.0),
            cur_ry,
            1.0,
            Color::new(0.20, 0.28, 0.38, 0.50),
        );
        cur_ry += scaler.s(12.0);

        // Historical dossier paragraph
        fonts.draw_ui_bold("HISTORICAL DOSSIER & RACING PEDIGREE", right_x + scaler.s(14.0), cur_ry, scaler.font_s(10.0), Palette::NEON_GOLD);
        cur_ry += scaler.s(14.0);
        let bio_w = right_w - scaler.s(28.0);
        let lines = fonts.draw_ui_regular_multiline(
            model.history_bio,
            right_x + scaler.s(14.0),
            cur_ry,
            scaler.font_s(9.0),
            scaler.s(12.5),
            bio_w,
            Palette::UI_TEXT_MUTED,
        );
        cur_ry += (lines as f32) * scaler.s(12.5) + scaler.s(12.0);

        // Livery / paint selection palette (platform SwatchPicker)
        let livery_colors = vec![
            model.primary_color,
            model.secondary_color,
            Palette::NEON_GOLD,
        ];
        let livery_picker = SwatchPicker::new(
            right_x + scaler.s(14.0),
            cur_ry,
            scaler.s(14.0),
            scaler.s(6.0),
            livery_colors,
        );
        livery_picker.render_frame();
        fonts.draw_ui_regular(
            "LIVERY PAINT",
            right_x + scaler.s(14.0) + livery_picker.total_width() + scaler.s(12.0),
            cur_ry + scaler.s(11.0),
            scaler.font_s(9.5),
            Palette::UI_TEXT_MUTED,
        );
        cur_ry += scaler.s(22.0);

        // 6 Performance Stat Bars (platform MetricBar)
        let (spd, acc, grip, drift, brk, aero) = model.stats;
        let bar_w = right_w - scaler.s(28.0);
        let bar_base_x = right_x + scaler.s(14.0);
        let bar_h = scaler.s(16.0);

        MetricBar::stat("SPEED", spd, format!("{:.0}%", spd * 100.0), Palette::NEON_CYAN)
            .draw(&scaler, fonts, LayoutRect::new(bar_base_x, cur_ry, bar_w, bar_h));
        cur_ry += bar_h;
        MetricBar::stat("ACCEL", acc, format!("{:.0}%", acc * 100.0), Palette::NEON_GOLD)
            .draw(&scaler, fonts, LayoutRect::new(bar_base_x, cur_ry, bar_w, bar_h));
        cur_ry += bar_h;
        MetricBar::stat("GRIP", grip, format!("{:.0}%", grip * 100.0), Palette::NEON_GREEN)
            .draw(&scaler, fonts, LayoutRect::new(bar_base_x, cur_ry, bar_w, bar_h));
        cur_ry += bar_h;
        MetricBar::stat("AGILITY", drift, format!("{:.0}%", drift * 100.0), Palette::NEON_MAGENTA)
            .draw(&scaler, fonts, LayoutRect::new(bar_base_x, cur_ry, bar_w, bar_h));
        cur_ry += bar_h;
        MetricBar::stat("BRAKING", brk, format!("{:.0}%", brk * 100.0), Palette::NEON_ORANGE)
            .draw(&scaler, fonts, LayoutRect::new(bar_base_x, cur_ry, bar_w, bar_h));
        cur_ry += bar_h;
        MetricBar::stat("AERO", aero, format!("{:.0}%", aero * 100.0), Palette::WHITE)
            .draw(&scaler, fonts, LayoutRect::new(bar_base_x, cur_ry, bar_w, bar_h));
    }

    // TYRES row (spec 098): GT cars pick a slick, stock cars show their series compound.
    if tyres != GarageTyres::Hidden {
        let (first_x, pill_y, _, pill_h) = garage_tyre_pill_rect(sw, sh, 0);
        match tyres {
            GarageTyres::Select(current) => {
                let label_y = pill_y - scaler.s(7.0);
                fonts.draw_ui_bold("TYRES", first_x, label_y, scaler.font_s(10.0), Palette::NEON_GOLD);
                let hint = "[T] CHANGE";
                let hint_w = fonts.measure_ui_regular(hint, scaler.font_s(9.0)).width;
                fonts.draw_ui_regular(
                    hint,
                    right_x + right_w - scaler.s(14.0) - hint_w,
                    label_y,
                    scaler.font_s(9.0),
                    Palette::UI_TEXT_MUTED,
                );
                for (i, id) in GT_GARAGE_COMPOUNDS.iter().enumerate() {
                    let (px, py, pw, ph) = garage_tyre_pill_rect(sw, sh, i);
                    let [r, g, b, _] = id.accent_rgba();
                    let selected = *id == current;
                    let fill = if selected { Color::new(r, g, b, 0.28) } else { Palette::UI_CARD_BG };
                    let border = if selected { Color::new(r, g, b, 1.0) } else { Palette::UI_CARD_BORDER };
                    scaler.draw_glass_card(px, py, pw, ph, fill, border, if selected { 2.2 } else { 1.0 });
                    fonts.draw_ui_bold_centered(
                        &garage_compound_label(*id),
                        px + pw * 0.5,
                        py + ph * 0.5 + scaler.s(4.0),
                        scaler.font_s(10.5),
                        if selected { Palette::WHITE } else { Palette::UI_TEXT_MUTED },
                    );
                }
                fonts.draw_ui_regular(
                    garage_compound_tradeoff(current),
                    first_x,
                    pill_y + pill_h + scaler.s(15.0),
                    scaler.font_s(9.5),
                    Palette::UI_TEXT_MUTED,
                );
            }
            GarageTyres::ReadOnly { compound, note } => {
                let [r, g, b, _] = compound.accent_rgba();
                fonts.draw_ui_bold(
                    &format!("TYRES: {} ({})", garage_compound_label(compound), note),
                    first_x,
                    pill_y + pill_h * 0.5 + scaler.s(4.0),
                    scaler.font_s(11.0),
                    Color::new(r, g, b, 1.0),
                );
            }
            GarageTyres::Hidden => {}
        }
    }

    // Action Button: Select Vehicle for Race / Purchase with XP / Locked Notice
    let (btn_x, btn_y, btn_w, btn_h) = garage_select_button_rect(sw, sh);

    let active_car_id = active_model.map(|m| m.id).unwrap_or("");
    let active_car_tier = active_model.map(|m| m.tier).unwrap_or(garage_tier);
    let is_car_heritage = (active_module_id == "rallycross" || active_module_id == "rally" || active_module_id == "rx") && active_car_tier == 7;
    let is_car_owned = active_module_id == "classic"
        || is_dev_mode
        || owned_cars.iter().any(|c| c == active_car_id);

    let cost = ModuleCareerProgress::car_credit_cost_for_car(active_car_id, active_module_id, active_car_tier);
    let can_afford = available_credits >= cost;
    let tier_eligible = is_dev_mode
        || is_car_heritage
        || career_progress
            .map_or(is_tier_unlocked, |cp| cp.level >= active_car_tier as u32);

    // Hero CTA frame (platform HeroActionButton, via ScreenFooter)
    let (hero_label, hero_focused, hero_disabled) = if !car_change_allowed {
        ("VEHICLE PRESET FOR RACE".to_string(), false, true)
    } else if is_car_owned {
        ("SELECT VEHICLE FOR RACE".to_string(), true, false)
    } else if !tier_eligible {
        (format!("VEHICLE LOCKED — TIER {} REQUIRED", active_car_tier), false, true)
    } else if can_afford {
        (format!("BUY VEHICLE: ${} CR", cost), true, false)
    } else {
        (format!("INSUFFICIENT FUNDS (${} CR)", cost), false, true)
    };
    let mut cta_footer = ScreenFooter::new(btn_x, btn_y, btn_w, btn_h);
    let mut hero_btn = HeroActionButton::new(hero_label, btn_w - 32.0, btn_h);
    hero_btn.is_focused = hero_focused;
    hero_btn.is_disabled = hero_disabled;
    cta_footer.set_hero_button(hero_btn);
    cta_footer.render_frame();

    if !car_change_allowed {
        fonts.draw_ui_bold_centered(
            "🔒 VEHICLE PRESET FOR RACE  [ESC TO RETURN]",
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(12.5),
            Palette::UI_TEXT_MUTED,
        );
        fonts.draw_ui_regular_centered(
            "Vehicle category and model are locked in this mode — inspect showroom specs or press ESC",
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::UI_TEXT_MUTED,
        );
    } else if is_car_owned {
        fonts.draw_ui_bold_centered(
            "▶ SELECT VEHICLE FOR RACE  [ENTER / SPACE]",
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(12.5),
            Palette::WHITE,
        );
        fonts.draw_ui_regular_centered(
            if is_car_heritage {
                "Status: Heritage Class — homologated for Heritage Series & Quick Race"
            } else {
                "Status: Owned and homologated for active module"
            },
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::WHITE,
        );
    } else if !tier_eligible {
        let lock_title = if active_car_tier > max_ranked_tier {
            "🔒 UNRANKED VEHICLE — DEV MODE ONLY".to_string()
        } else {
            format!("🔒 VEHICLE LOCKED — CAREER TIER {} REQUIRED", active_car_tier)
        };
        fonts.draw_ui_bold_centered(
            &lock_title,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(12.0),
            Palette::RED,
        );
        let lock_hint = if (active_module_id == "gt" || active_module_id == "nascar") && active_car_tier == 1 {
            "Requires Tier 1 License & 25,000 - 70,000 Cr. Earn credentials in grassroots first."
        } else if active_car_tier > max_ranked_tier {
            "Parked outside the career tiers; drive it in dev mode"
        } else {
            "Advance career tier by earning championship podiums to unlock purchasing"
        };
        fonts.draw_ui_regular_centered(
            lock_hint,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::UI_TEXT_MUTED,
        );
    } else if can_afford {
        if let Some(rect) = cta_footer.hero_button_rect() {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.05, 0.25, 0.10, 0.35));
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, Palette::NEON_GREEN);
        }
        let is_starter = (active_module_id == "kart" || active_module_id == "autocross" || active_module_id == "rallycross" || active_module_id == "rally") && active_car_tier == 1;
        let buy_title = if is_starter {
            format!("🛒 BUY VEHICLE: ${} CR  [B / ENTER]  ★ AFFORDABLE", cost)
        } else {
            format!("🛒 BUY VEHICLE: ${} CR  [B / ENTER]", cost)
        };
        fonts.draw_ui_bold_centered(
            &buy_title,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(11.5),
            Palette::NEON_GREEN,
        );
        let buy_sub = format!(
            "Wallet Balance: ${} Cr  →  ${} Cr remaining  (Discipline XP: {} XP)",
            available_credits,
            available_credits.saturating_sub(cost),
            career_progress.map_or(0, |cp| cp.xp)
        );
        fonts.draw_ui_regular_centered(
            &buy_sub,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::WHITE,
        );
    } else {
        if let Some(rect) = cta_footer.hero_button_rect() {
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.35, 0.05, 0.05, 0.40));
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, Palette::RED);
        }
        let lock_title = format!("🛒 INSUFFICIENT FUNDS (${} CR) — WALLET: ${} CR", cost, available_credits);
        fonts.draw_ui_bold_centered(
            &lock_title,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(11.0),
            Palette::RED,
        );
        let need_cr = cost.saturating_sub(available_credits);
        let lock_sub = if (active_module_id == "gt" || active_module_id == "nascar") && active_car_tier == 1 {
            "Requires Tier 1 License & 25,000 - 70,000 Cr. Earn credentials in grassroots first.".to_string()
        } else {
            format!("Earn ${} more Credits in prize purses to purchase this vehicle", need_cr)
        };
        fonts.draw_ui_regular_centered(
            &lock_sub,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::UI_TEXT_MUTED,
        );
    }

    // Bottom Navigation Bar
    let bottom_y = sh - scaler.s(16.0);
    let nav_prompt = if !car_change_allowed {
        "USE [◄ / ►] CARS  •  [Q / E] TIERS  •  [SPACE] REV  •  [ESC] RETURN"
    } else if module_locked {
        "USE [◄ / ►] CARS  •  [Q / E] TIERS  •  [SPACE] REV  •  [B / ENTER] BUY/SELECT  •  [ESC] RETURN TO LOBBY"
    } else {
        "USE [◄ / ►] CARS  •  [Q / E] TIERS  •  [1..5] MODULES  •  [SPACE] REV  •  [B / ENTER] BUY/SELECT  •  [ESC] RETURN"
    };
    fonts.draw_ui_bold_centered(
        nav_prompt,
        sw * 0.5,
        bottom_y,
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );
}

/// The 6 motorsport modules supported in the Fleet Gallery.
pub const GALLERY_MODULES: &[(&str, &str)] = &[
    ("gt", "GT"),
    ("rallycross", "RALLYCROSS"),
    ("kart", "KART"),
    ("nascar", "STOCK CAR"),
    ("extreme_offroad", "OFF-ROAD"),
    ("autocross", "AUTOCROSS"),
];

/// Maps a gallery filter index (0..5) to its corresponding module identifier.
pub fn gallery_filter_to_module(idx: usize) -> &'static str {
    match idx {
        0 => "gt",
        1 => "rallycross",
        2 => "kart",
        3 => "nascar",
        4 => "extreme_offroad",
        5 => "autocross",
        _ => "gt",
    }
}

/// Maps a module identifier string to its gallery filter index (0..5).
pub fn module_to_gallery_filter(module_id: &str) -> usize {
    match module_id {
        "gt" | "gt_challenge" => 0,
        "rallycross" | "rally" | "rx" => 1,
        "kart" => 2,
        "nascar" => 3,
        "extreme_offroad" => 4,
        "autocross" => 5,
        _ => 0,
    }
}

/// Returns the rectangle (x, y, w, h) for a gallery module tab.
pub fn garage_gallery_tab_rect(sw: f32, sh: f32, idx: usize) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let tab_w = scaler.s(105.0);
    let total_tab_w = tab_w * GALLERY_MODULES.len() as f32;
    let start_tab_x = (sw - total_tab_w) * 0.5;
    let tx = start_tab_x + (idx as f32) * tab_w;
    (tx, scaler.s(46.0), tab_w - scaler.s(4.0), scaler.s(22.0))
}

/// Returns the rectangle (x, y, w, h) for a car card in the fleet gallery grid.
pub fn garage_gallery_card_rect(sw: f32, sh: f32, idx: usize) -> (f32, f32, f32, f32) {
    let scaler = UiScaler::new(sw, sh);
    let cols = 4;
    let grid_x = scaler.safe_pad_x;
    let grid_y = scaler.s(76.0);
    let cell_w = (sw - scaler.safe_pad_x * 2.0 - scaler.s(12.0) * (cols - 1) as f32) / cols as f32;
    let cell_h = scaler.s(92.0);
    let col = idx % cols;
    let row = idx / cols;
    let cx = grid_x + (cell_w + scaler.s(12.0)) * col as f32;
    let cy = grid_y + (cell_h + scaler.s(10.0)) * row as f32;
    (cx, cy, cell_w, cell_h)
}

/// Renders the Panoramic Fleet Gallery grid (`[C]` toggle).
#[allow(clippy::too_many_arguments)]
fn render_fleet_gallery(
    fonts: &Fonts,
    scaler: &UiScaler,
    sw: f32,
    sh: f32,
    filter_idx: usize,
    sel_idx: usize,
    unlocked_tier: u32,
    is_dev_mode: bool,
) {
    fonts.draw_display_centered_with_shadow(
        "FLEET GALLERY & GLOBAL CATALOG",
        sw * 0.5,
        scaler.s(28.0),
        scaler.font_s(24.0),
        Palette::NEON_CYAN,
        Color::new(0.0, 0.0, 0.0, 0.6),
        scaler.s(2.0),
    );

    let target_mod = gallery_filter_to_module(filter_idx);
    let filtered = get_models_for_module(target_mod);

    // Vehicle class selector (platform FilterBar, Pills; LB/RB cycling handled by game input)
    let module_labels: Vec<&str> = GALLERY_MODULES.iter().map(|(_, name)| *name).collect();
    let tab_w = scaler.s(105.0);
    let tab_bar = FilterBar::from_labels(&module_labels)
        .with_style(FilterBarStyle::Pills)
        .with_active(filter_idx)
        .with_focus(true);
    tab_bar.draw(
        &scaler,
        fonts,
        (sw - tab_w * GALLERY_MODULES.len() as f32) * 0.5,
        scaler.s(46.0),
        tab_w * GALLERY_MODULES.len() as f32,
        scaler.s(22.0),
        Palette::NEON_CYAN,
    );

    // Grid of cards (platform CardGrid, 4-column 2D matrix navigation)
    let cols = 4;
    let grid_x = scaler.safe_pad_x;
    let grid_y = scaler.s(76.0);
    let cell_w = (sw - scaler.safe_pad_x * 2.0 - scaler.s(12.0) * (cols - 1) as f32) / cols as f32;
    let cell_h = scaler.s(92.0);
    let mut grid: CardGrid<RealCarModel> = CardGrid::new(
        grid_x,
        grid_y,
        cols,
        cell_w,
        cell_h,
        scaler.s(12.0),
        scaler.s(10.0),
    );
    for model in filtered.iter().take(16) {
        grid.add_item(CardGridItem::new(model.id, (**model).clone()));
    }
    grid.selected_idx = sel_idx.min(grid.len().saturating_sub(1));
    grid.is_focused = true;

    for i in 0..grid.len() {
        let rect = grid.item_rect(i);
        let model = &grid.items[i].data;
        let is_sel = i == sel_idx;
        let is_heritage = (model.module_id == "rallycross" || model.module_id == "rally") && model.tier == 7;
        let is_unlocked = is_dev_mode || is_heritage || (model.tier as u32) <= unlocked_tier;

        grid.render_card_frame(&rect, is_sel, false);
        if !is_unlocked {
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, Palette::RED);
        }

        // Mini lateral silhouette
        let scheme = CarColorScheme {
            primary: model.primary_color,
            secondary: model.secondary_color,
            helmet: Palette::NEON_GOLD,
        };
        render_real_car_lateral_by_id(model.id, &scheme, rect.x + rect.w * 0.5, rect.y + scaler.s(28.0), scaler.s(0.65), 0.0, false);

        // Name and stats
        fonts.draw_ui_bold(model.name, rect.x + scaler.s(8.0), rect.y + scaler.s(62.0), scaler.font_s(10.0), Palette::WHITE);
        let tier_tag = if is_heritage { "HERITAGE".to_string() } else { format!("T{}", model.tier) };
        let sub = format!("{} • {} BHP • {} kg", tier_tag, model.bhp, model.weight_kg);
        fonts.draw_ui_regular(&sub, rect.x + scaler.s(8.0), rect.y + scaler.s(76.0), scaler.font_s(8.5), Palette::UI_TEXT_MUTED);

        if !is_unlocked {
            fonts.draw_ui_bold("🔒 LOCKED", rect.x + rect.w - scaler.s(65.0), rect.y + scaler.s(16.0), scaler.font_s(8.5), Palette::RED);
        }
    }

    fonts.draw_ui_bold_centered(
        "[1..5] / [TAB] / [Q / E] SWITCH MODULE  •  ARROWS / WASD NAVIGATE  •  [ENTER] SELECT  •  [C / ESC] EXIT",
        sw * 0.5,
        sh - scaler.s(18.0),
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );
}
