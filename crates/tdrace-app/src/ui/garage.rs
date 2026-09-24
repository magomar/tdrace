use macroquad::color::Color;
use macroquad::input::mouse_position;
use macroquad::prelude::{screen_height, screen_width};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines};

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::catalog::{get_models_for_module, get_models_for_module_and_tier, RealCarModel};
use crate::profile::ModuleCareerProgress;
use crate::render::color::{CarColorScheme, Palette};
use crate::render::lateral::render_real_car_lateral_by_id;

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
) {
    let sw = screen_width();
    let sh = screen_height();
    let scaler = UiScaler::new(sw, sh);

    // 1. Dark Showroom Backdrop with subtle carbon weave pattern
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.04, 0.05, 0.08, 0.98));

    // Module branding
    let (mod_title, mod_accent) = match active_module_id {
        "gt" | "gt_challenge" | "f1" => ("GT WORLD CHALLENGE", Palette::NEON_CYAN),
        "rally" => ("RALLYCROSS & ALL-TERRAIN", Palette::NEON_GOLD),
        "kart" => ("KARTING & MICRO-RACERS", Palette::NEON_GREEN),
        "nascar" => ("NASCAR STOCK CAR RACING", Palette::YELLOW),
        "extreme_offroad" => ("EXTREME OFF-ROAD & STUNT ARENAS", Palette::NEON_ORANGE),
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

    let module_subtitle = if active_module_id == "classic" {
        "MODULE: CLASSIC ARCADE MOTORSPORT • FANTASY ARCADE ROSTER [◄ A / D ►]".to_string()
    } else if let Some(cp) = career_progress {
        format!(
            "MODULE: {}  [◄ 1..5 ►]  •  TIER {}: {}  [◄ Q/E ►]  •  SPENDABLE XP: {} XP",
            mod_title, garage_tier, category_name.to_uppercase(), cp.xp
        )
    } else {
        format!(
            "MODULE: {}  [◄ 1..5 ►]  •  TIER {}: {}  [◄ Q/E ►]",
            mod_title, garage_tier, category_name.to_uppercase()
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

    // Ambient spotlight glow behind vehicle
    let center_x = stage_x + stage_w * 0.50;
    let center_y = stage_y + stage_h * 0.52;
    draw_circle(center_x, center_y - scaler.s(20.0), scaler.s(220.0), Color::new(0.08, 0.16, 0.28, 0.22));
    draw_circle(center_x, center_y - scaler.s(10.0), scaler.s(150.0), Color::new(0.12, 0.22, 0.35, 0.18));

    // Hero Showroom Stage Card
    scaler.draw_glass_card(stage_x, stage_y, stage_w, stage_h, Palette::UI_CARD_BG, mod_accent, 1.4);

    let is_tier_unlocked = active_module_id == "classic" || is_dev_mode || garage_tier as u32 <= unlocked_tier;

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
    fonts.draw_ui_bold(
        unlock_badge,
        stage_x + stage_w - scaler.s(190.0),
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

    let rev_prompt = "HOLD [SPACE] TO REV ENGINE & INSPECT EXHAUST HEAT  •  [C] FLEET GALLERY";
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

        if active_module_id == "classic" {
            fonts.draw_ui_bold("OWNED", cx + cw - scaler.s(48.0), cy + scaler.s(15.0), scaler.font_s(8.5), Palette::NEON_GREEN);
        } else if let Some(cp) = career_progress {
            let is_unlocked = is_dev_mode || cp.is_car_unlocked(model.id, is_dev_mode);
            if is_unlocked {
                fonts.draw_ui_bold("OWNED", cx + cw - scaler.s(48.0), cy + scaler.s(15.0), scaler.font_s(8.5), Palette::NEON_GREEN);
            } else {
                let cost = ModuleCareerProgress::car_cost(model.tier);
                let tag = format!("{} XP", cost);
                let col = if cp.xp >= cost && cp.level >= model.tier as u32 { Palette::NEON_GOLD } else { Palette::RED };
                fonts.draw_ui_bold(&tag, cx + cw - scaler.s(55.0), cy + scaler.s(15.0), scaler.font_s(8.5), col);
            }
        }

        fonts.draw_ui_bold(
            model.name,
            cx + scaler.s(8.0),
            cy + scaler.s(30.0),
            scaler.font_s(10.0),
            Palette::WHITE,
        );

        let sub_info = format!("{} BHP • {}", model.bhp, model.drivetrain);
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

        // Car Title
        fonts.draw_ui_bold(
            model.name,
            right_x + scaler.s(14.0),
            cur_ry,
            scaler.font_s(14.5),
            Palette::WHITE,
        );
        cur_ry += scaler.s(16.0);

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

        // 6 Performance Stat Bars
        let (spd, acc, grip, drift, brk, aero) = model.stats;
        let bar_w = right_w - scaler.s(28.0);
        let bar_base_x = right_x + scaler.s(14.0);

        render_garage_stat_bar(&scaler, fonts, bar_base_x, cur_ry, bar_w, "SPEED", spd, Palette::NEON_CYAN);
        cur_ry += scaler.s(16.0);
        render_garage_stat_bar(&scaler, fonts, bar_base_x, cur_ry, bar_w, "ACCEL", acc, Palette::NEON_GOLD);
        cur_ry += scaler.s(16.0);
        render_garage_stat_bar(&scaler, fonts, bar_base_x, cur_ry, bar_w, "GRIP", grip, Palette::NEON_GREEN);
        cur_ry += scaler.s(16.0);
        render_garage_stat_bar(&scaler, fonts, bar_base_x, cur_ry, bar_w, "AGILITY", drift, Palette::NEON_MAGENTA);
        cur_ry += scaler.s(16.0);
        render_garage_stat_bar(&scaler, fonts, bar_base_x, cur_ry, bar_w, "BRAKING", brk, Palette::NEON_ORANGE);
        cur_ry += scaler.s(16.0);
        render_garage_stat_bar(&scaler, fonts, bar_base_x, cur_ry, bar_w, "AERO", aero, Palette::WHITE);
    }

    // Action Button: Select Vehicle for Race / Purchase with XP / Locked Notice
    let (btn_x, btn_y, btn_w, btn_h) = garage_select_button_rect(sw, sh);
    let (mx, my) = mouse_position();
    let is_btn_hovered = mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;

    let active_car_id = active_model.map(|m| m.id).unwrap_or("");
    let active_car_tier = active_model.map(|m| m.tier).unwrap_or(garage_tier);
    let is_car_unlocked = active_module_id == "classic"
        || is_dev_mode
        || career_progress
            .map(|cp| cp.is_car_unlocked(active_car_id, is_dev_mode))
            .unwrap_or(is_tier_unlocked);

    let cost = ModuleCareerProgress::car_cost(active_car_tier);
    let can_afford = career_progress.map_or(true, |cp| cp.xp >= cost);
    let tier_eligible = is_dev_mode
        || career_progress
            .map_or(is_tier_unlocked, |cp| cp.level >= active_car_tier as u32);

    if is_car_unlocked {
        let (btn_bg, btn_border) = if is_btn_hovered {
            (Color::new(0.12, 0.68, 0.32, 0.98), Palette::NEON_GREEN)
        } else {
            (Color::new(0.08, 0.44, 0.22, 0.92), Color::new(0.20, 0.78, 0.40, 0.85))
        };
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_bg);
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, btn_border);

        fonts.draw_ui_bold_centered(
            "▶ SELECT VEHICLE FOR RACE  [ENTER / SPACE]",
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(12.5),
            Palette::WHITE,
        );
        fonts.draw_ui_regular_centered(
            "Status: Owned and homologated for active module",
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::WHITE,
        );
    } else if !tier_eligible {
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(0.35, 0.10, 0.10, 0.95));
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, Palette::RED);

        let lock_title = format!("🔒 VEHICLE LOCKED — CAREER TIER {} REQUIRED", active_car_tier);
        fonts.draw_ui_bold_centered(
            &lock_title,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(12.0),
            Palette::RED,
        );
        fonts.draw_ui_regular_centered(
            "Advance career tier by earning championship podiums to unlock purchasing",
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::UI_TEXT_MUTED,
        );
    } else if can_afford {
        let (btn_bg, btn_border) = if is_btn_hovered {
            (Color::new(0.70, 0.52, 0.10, 0.98), Palette::NEON_GOLD)
        } else {
            (Color::new(0.42, 0.32, 0.08, 0.92), Color::new(0.85, 0.68, 0.18, 0.85))
        };
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_bg);
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, btn_border);

        let buy_title = format!("🛒 BUY VEHICLE: {} XP  [B / ENTER]", cost);
        fonts.draw_ui_bold_centered(
            &buy_title,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(12.5),
            Palette::WHITE,
        );
        let cur_xp = career_progress.map_or(0, |cp| cp.xp);
        let buy_sub = format!("Spendable Balance: {} XP  →  {} XP remaining", cur_xp, cur_xp.saturating_sub(cost));
        fonts.draw_ui_regular_centered(
            &buy_sub,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(38.0),
            scaler.font_s(10.0),
            Palette::WHITE,
        );
    } else {
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(0.28, 0.16, 0.08, 0.95));
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, Palette::NEON_GOLD);

        let cur_xp = career_progress.map_or(0, |cp| cp.xp);
        let lock_title = format!("🛒 VEHICLE PRICE: {} XP (WALLET: {} XP)", cost, cur_xp);
        fonts.draw_ui_bold_centered(
            &lock_title,
            btn_x + btn_w * 0.5,
            btn_y + scaler.s(22.0),
            scaler.font_s(12.0),
            Palette::NEON_GOLD,
        );
        let need_xp = cost.saturating_sub(cur_xp);
        let lock_sub = format!("Earn {} more XP in races to purchase this vehicle", need_xp);
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
    let nav_prompt = "USE [◄ / ►] CARS  •  [Q / E] TIERS  •  [1..5] MODULES  •  [SPACE] REV  •  [B / ENTER] BUY/SELECT  •  [ESC] RETURN";
    fonts.draw_ui_bold_centered(
        nav_prompt,
        sw * 0.5,
        bottom_y,
        scaler.font_s(11.0),
        Palette::UI_TEXT_MUTED,
    );
}

/// The 5 motorsport modules supported in the Fleet Gallery.
pub const GALLERY_MODULES: &[(&str, &str)] = &[
    ("gt", "GT"),
    ("rally", "RALLY"),
    ("kart", "KART"),
    ("nascar", "NASCAR"),
    ("extreme_offroad", "OFF-ROAD"),
];

/// Maps a gallery filter index (0..5) to its corresponding module identifier.
pub fn gallery_filter_to_module(idx: usize) -> &'static str {
    match idx {
        0 => "gt",
        1 => "rally",
        2 => "kart",
        3 => "nascar",
        4 => "extreme_offroad",
        _ => "gt",
    }
}

/// Maps a module identifier string to its gallery filter index (0..4).
pub fn module_to_gallery_filter(module_id: &str) -> usize {
    match module_id {
        "gt" | "gt_challenge" => 0,
        "rally" => 1,
        "kart" => 2,
        "nascar" => 3,
        "extreme_offroad" => 4,
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

    for (i, &(_mod_id, name)) in GALLERY_MODULES.iter().enumerate() {
        let (tx, ty, tw, th) = garage_gallery_tab_rect(sw, sh, i);
        let is_active = i == filter_idx;
        let bg = if is_active {
            Palette::NEON_CYAN
        } else {
            Color::new(0.08, 0.10, 0.15, 0.85)
        };
        let fg = if is_active {
            Palette::BLACK
        } else {
            Palette::UI_TEXT_MUTED
        };
        draw_rectangle(tx, ty, tw, th, bg);
        fonts.draw_ui_bold_centered(
            name,
            tx + tw * 0.5,
            ty + scaler.s(15.0),
            scaler.font_s(10.0),
            fg,
        );
    }

    // Grid of cards
    for (i, model) in filtered.iter().enumerate().take(16) {
        let (cx, cy, cell_w, cell_h) = garage_gallery_card_rect(sw, sh, i);

        let is_sel = i == sel_idx;
        let is_unlocked = is_dev_mode || (model.tier as u32) <= unlocked_tier;
        let card_bg = if is_sel {
            Palette::UI_CARD_BG_HOVER
        } else {
            Palette::UI_CARD_BG
        };
        let border_col = if is_sel {
            Palette::NEON_GOLD
        } else if !is_unlocked {
            Palette::RED
        } else {
            Palette::UI_CARD_BORDER
        };

        scaler.draw_glass_card(cx, cy, cell_w, cell_h, card_bg, border_col, if is_sel { 2.4 } else { 1.0 });

        // Mini lateral silhouette
        let scheme = CarColorScheme {
            primary: model.primary_color,
            secondary: model.secondary_color,
            helmet: Palette::NEON_GOLD,
        };
        render_real_car_lateral_by_id(model.id, &scheme, cx + cell_w * 0.5, cy + scaler.s(28.0), scaler.s(0.65), 0.0, false);

        // Name and stats
        fonts.draw_ui_bold(model.name, cx + scaler.s(8.0), cy + scaler.s(62.0), scaler.font_s(10.0), Palette::WHITE);
        let sub = format!("T{} • {} BHP • {} kg", model.tier, model.bhp, model.weight_kg);
        fonts.draw_ui_regular(&sub, cx + scaler.s(8.0), cy + scaler.s(76.0), scaler.font_s(8.5), Palette::UI_TEXT_MUTED);

        if !is_unlocked {
            fonts.draw_ui_bold("🔒 LOCKED", cx + cell_w - scaler.s(65.0), cy + scaler.s(16.0), scaler.font_s(8.5), Palette::RED);
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

/// Renders a single horizontal performance stat bar with label, value, and colored fill.
fn render_garage_stat_bar(
    scaler: &UiScaler,
    fonts: &Fonts,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    val: f32,
    color: Color,
) {
    let lbl_w = scaler.s(65.0);
    let bar_h = scaler.s(7.0);
    let bar_w = w - lbl_w - scaler.s(45.0);

    fonts.draw_ui_bold(
        label,
        x,
        y + scaler.s(7.0),
        scaler.font_s(9.5),
        Palette::UI_TEXT_MUTED,
    );

    let bar_x = x + lbl_w;
    draw_rectangle(bar_x, y, bar_w, bar_h, Color::new(0.08, 0.10, 0.14, 0.90));
    draw_rectangle_lines(bar_x, y, bar_w, bar_h, 1.0, Color::new(0.20, 0.25, 0.35, 0.80));

    let fill_w = bar_w * val.clamp(0.0, 1.0);
    draw_rectangle(bar_x, y, fill_w, bar_h, color);

    let pct_str = format!("{:.0}%", val * 100.0);
    fonts.draw_ui_bold(
        &pct_str,
        bar_x + bar_w + scaler.s(8.0),
        y + scaler.s(7.0),
        scaler.font_s(9.5),
        Palette::WHITE,
    );
}
