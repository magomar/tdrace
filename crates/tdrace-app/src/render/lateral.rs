use std::collections::HashMap;
use std::sync::Mutex;
use macroquad::color::Color;
use macroquad::math::Vec2;
use macroquad::shapes::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle,
};
use macroquad::texture::{draw_texture_ex, DrawTextureParams, Image, Texture2D};

use super::color::{CarColorScheme, Palette};
use crate::module::VehicleVisualType;
use crate::ui::menu::CarChoice;

static PORSCHE_LATERAL_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r.png");
static PORSCHE_LATERAL_THUMB_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r_thumb.png");
static PORSCHE_LATERAL_CACHE: Mutex<Option<HashMap<(u32, u32, bool), Texture2D>>> = Mutex::new(None);

#[inline]
fn color_to_u32(c: Color) -> u32 {
    let r = (c.r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (c.g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (c.b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

/// Retrieves or dynamically generates a colorway-tinted lateral texture for the Porsche 911 GT3 R.
/// Uses full 1024px asset when `high_res` is true (garage stage) and 256px thumbnail when false (menus/cards).
pub fn get_tinted_porsche_lateral(primary: Color, secondary: Color, high_res: bool) -> Texture2D {
    let k1 = color_to_u32(primary);
    let k2 = color_to_u32(secondary);

    let mut guard = PORSCHE_LATERAL_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(tex) = map.get(&(k1, k2, high_res)) {
        return tex.clone();
    }

    let png_bytes = if high_res {
        PORSCHE_LATERAL_PNG
    } else {
        PORSCHE_LATERAL_THUMB_PNG
    };

    let base_img = Image::from_file_with_format(png_bytes, None)
        .expect("failed to load porsche lateral PNG");
    let mut tinted = base_img.clone();
    for pixel in tinted.bytes.chunks_exact_mut(4) {
        let a = pixel[3];
        if a < 15 {
            continue;
        }
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let sat = if max_c > 0.001 { (max_c - min_c) / max_c } else { 0.0 };
        let lum = (r + g + b) / 3.0;

        if lum > 0.65 && sat < 0.22 {
            pixel[0] = ((primary.r * lum * 1.05).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = ((primary.g * lum * 1.05).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = ((primary.b * lum * 1.05).clamp(0.0, 1.0) * 255.0) as u8;
        } else if g > 0.58 && r > 0.45 && b < 0.45 && sat > 0.30 {
            pixel[0] = ((secondary.r * lum * 1.15).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = ((secondary.g * lum * 1.15).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = ((secondary.b * lum * 1.15).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }

    let texture = Texture2D::from_image(&tinted);
    map.insert((k1, k2, high_res), texture.clone());
    texture
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelStyle {
    AlloyGT,
    CenterlockAero,
    StockCarSteel,
    RallyGravel,
    StuddedIce,
    MudTractorChevron,
    MonsterJam66,
    KartSmall,
    LawnmowerTurf,
}

/// Adapts the module roster visual type to `render_car_lateral`.
#[allow(clippy::too_many_arguments)]
pub fn render_lateral_car(
    center_x: f32,
    center_y: f32,
    scale: f32,
    visual_type: VehicleVisualType,
    color_scheme: &CarColorScheme,
    brake_heat: f32,
    revving: bool,
    draw_reflection: bool,
) {
    let car_choice = match visual_type {
        VehicleVisualType::OpenWheel { .. } => CarChoice::Kart,
        VehicleVisualType::GoKart { .. } => CarChoice::Kart,
        VehicleVisualType::RallyHatch { .. } => CarChoice::RallyCar,
        VehicleVisualType::StockCar { .. } => CarChoice::StockCar,
        VehicleVisualType::SandRail { .. } => CarChoice::SandRail,
        VehicleVisualType::TouringGT { .. } => CarChoice::GT3Car,
    };
    render_car_lateral(
        car_choice,
        color_scheme,
        center_x,
        center_y,
        scale,
        if revving { 1.0 } else { brake_heat },
        draw_reflection,
    );
}

/// Renders an authentic, model-specific 2D lateral (side-profile) vector illustration.
/// Uses `model_id` to select the exact bodywork silhouette, aerodynamics, and livery details.
pub fn render_real_car_lateral_by_id(
    model_id: &str,
    color_scheme: &CarColorScheme,
    center_x: f32,
    center_y: f32,
    scale: f32,
    rev_intensity: f32,
    draw_reflection: bool,
) {
    let s = scale.max(0.2);
    let primary = color_scheme.primary;
    let secondary = color_scheme.secondary;
    let helmet_col = color_scheme.helmet;

    // Base dimensions
    let half_len = 80.0 * s;
    let ground_y = center_y + 18.0 * s;
    let wheel_y = ground_y - 12.0 * s;

    // Determine model-specific wheelbase, wheel radius, and wheel style
    let (wf_x, wr_x, r_wheel, wheel_style) = get_wheel_geometry(model_id, center_x, ground_y, half_len, s);

    // Optional Showroom Mirror Floor Reflection
    if draw_reflection {
        let refl_fade = Color::new(primary.r * 0.6, primary.g * 0.6, primary.b * 0.6, 0.15);
        let refl_w_col = Color::new(0.08, 0.09, 0.11, 0.22);
        draw_rectangle(
            center_x - half_len * 0.75,
            ground_y + 2.0 * s,
            half_len * 1.50,
            12.0 * s,
            refl_fade,
        );
        draw_circle(wf_x, ground_y + 6.0 * s, r_wheel * 0.75, refl_w_col);
        draw_circle(wr_x, ground_y + 6.0 * s, r_wheel * 0.75, refl_w_col);
    }

    // Floor contact baseline
    draw_line(
        center_x - half_len * 1.08,
        ground_y,
        center_x + half_len * 1.08,
        ground_y,
        1.0 * s,
        Color::new(0.25, 0.35, 0.50, 0.35),
    );

    // If model-specific high-resolution lateral sprite is available, render tinted texture directly
    // Always use full-resolution lateral asset (high_res = true) to guarantee authentic 2:1 aspect ratio and avoid squashed thumbnail distortion
    if let Some(texture) = crate::render::vehicle_assets::get_vehicle_lateral_texture(model_id, primary, secondary, true) {
        let dest_w = half_len * 2.36;
        let dest_h = dest_w * (texture.height() / texture.width());
        let car_y = ground_y - dest_h * 0.98;
        draw_texture_ex(
            &texture,
            center_x - dest_w * 0.5,
            car_y,
            Color::new(1.0, 1.0, 1.0, 1.0),
            DrawTextureParams {
                dest_size: Some(Vec2::new(dest_w, dest_h)),
                ..Default::default()
            },
        );

        // Dynamic Rev Exhaust Backfire Sparks
        if rev_intensity > 0.05 {
            let exh_x = center_x - half_len * 0.92;
            let exh_y = ground_y - 8.0 * s;
            let flame_len = (18.0 * rev_intensity * s).min(28.0 * s);
            draw_triangle(
                Vec2::new(exh_x, exh_y - 2.5 * s),
                Vec2::new(exh_x, exh_y + 2.5 * s),
                Vec2::new(exh_x - flame_len, exh_y),
                Color::new(1.0, 0.45, 0.10, (0.6 + 0.4 * rev_intensity).min(1.0)),
            );
            draw_triangle(
                Vec2::new(exh_x, exh_y - 1.2 * s),
                Vec2::new(exh_x, exh_y + 1.2 * s),
                Vec2::new(exh_x - flame_len * 0.6, exh_y),
                Color::new(1.0, 0.90, 0.40, 1.0),
            );
        }
        return;
    }

    // Render Model-Specific Body Silhouette
    render_specific_body(model_id, center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);

    // Render Wheels with appropriate styling
    render_lateral_wheel_styled(wf_x, wheel_y, r_wheel, s, rev_intensity, wheel_style);
    render_lateral_wheel_styled(wr_x, wheel_y, r_wheel, s, rev_intensity, wheel_style);

    // Dynamic Rev Exhaust Backfire Sparks
    if rev_intensity > 0.05 {
        let exh_x = center_x - half_len * 0.90;
        let exh_y = ground_y - 8.0 * s;
        let flame_len = (18.0 * rev_intensity * s).min(28.0 * s);
        draw_triangle(
            Vec2::new(exh_x, exh_y - 2.5 * s),
            Vec2::new(exh_x, exh_y + 2.5 * s),
            Vec2::new(exh_x - flame_len, exh_y),
            Color::new(1.0, 0.45, 0.10, (0.6 + 0.4 * rev_intensity).min(1.0)),
        );
        draw_triangle(
            Vec2::new(exh_x, exh_y - 1.2 * s),
            Vec2::new(exh_x, exh_y + 1.2 * s),
            Vec2::new(exh_x - flame_len * 0.6, exh_y),
            Color::new(1.0, 0.95, 0.60, 0.95),
        );
    }
}

/// Fallback / Category-level side profile vector illustration.
pub fn render_car_lateral(
    car_choice: CarChoice,
    color_scheme: &CarColorScheme,
    center_x: f32,
    center_y: f32,
    scale: f32,
    rev_intensity: f32,
    draw_reflection: bool,
) {
    let dummy_id = match car_choice {
        CarChoice::GT4Clubsport => "gt_porsche_718_gt4",
        CarChoice::GT3Car => "gt_porsche_911_gt3r",
        CarChoice::GT2Biturbo => "gt_porsche_911_gt2_rs",
        CarChoice::GT1Legend => "gt_mclaren_f1_gtr_lt",
        CarChoice::HypercarPrototype => "gt_ferrari_499p",
        CarChoice::Kart => "kart_tony_kart_racer_ok",
        CarChoice::RallyCar => "rally_hyundai_i20_rx",
        CarChoice::StockCar => "nascar_arca_chevy_ss",
        CarChoice::SandRail => "offroad_sand_rail_buggy",
        CarChoice::SportsCar | CarChoice::DriftCar => "gt_bmw_m4_gt4",
    };
    render_real_car_lateral_by_id(
        dummy_id,
        color_scheme,
        center_x,
        center_y,
        scale,
        rev_intensity,
        draw_reflection,
    );
}

fn get_wheel_geometry(
    id: &str,
    cx: f32,
    _gy: f32,
    hl: f32,
    s: f32,
) -> (f32, f32, f32, WheelStyle) {
    match id {
        // Classic Arcade Fantasy Roster
        "classic_gt" => (cx + hl * 0.52, cx - hl * 0.48, 12.5 * s, WheelStyle::AlloyGT),
        "classic_nascar" => (cx + hl * 0.54, cx - hl * 0.50, 13.5 * s, WheelStyle::StockCarSteel),
        "classic_offroad" => (cx + hl * 0.55, cx - hl * 0.48, 16.0 * s, WheelStyle::MudTractorChevron),
        "classic_kart" => (cx + hl * 0.46, cx - hl * 0.44, 9.0 * s, WheelStyle::KartSmall),
        "classic_rally" => (cx + hl * 0.50, cx - hl * 0.48, 13.0 * s, WheelStyle::RallyGravel),

        // GT4
        "gt_porsche_718_gt4" => (cx + hl * 0.50, cx - hl * 0.50, 12.0 * s, WheelStyle::AlloyGT),
        "gt_bmw_m4_gt4" => (cx + hl * 0.56, cx - hl * 0.48, 12.5 * s, WheelStyle::AlloyGT),
        "gt_aston_vantage_gt4" => (cx + hl * 0.54, cx - hl * 0.50, 12.5 * s, WheelStyle::AlloyGT),
        "gt_toyota_supra_gt4" => (cx + hl * 0.48, cx - hl * 0.48, 12.0 * s, WheelStyle::AlloyGT),

        // GT3
        "gt_porsche_911_gt3r" => (cx + hl * 0.52, cx - hl * 0.48, 12.8 * s, WheelStyle::CenterlockAero),
        "gt_ferrari_296_gt3" => (cx + hl * 0.54, cx - hl * 0.52, 12.8 * s, WheelStyle::CenterlockAero),
        "gt_amg_gt3_evo" => (cx + hl * 0.58, cx - hl * 0.48, 13.0 * s, WheelStyle::AlloyGT),
        "gt_audi_r8_gt3_evo2" => (cx + hl * 0.52, cx - hl * 0.50, 12.8 * s, WheelStyle::CenterlockAero),

        // GT2
        "gt_porsche_911_gt2_rs" => (cx + hl * 0.52, cx - hl * 0.48, 13.0 * s, WheelStyle::CenterlockAero),
        "gt_brabham_bt62_gt2" => (cx + hl * 0.55, cx - hl * 0.52, 12.5 * s, WheelStyle::CenterlockAero),
        "gt_maserati_mc20_gt2" => (cx + hl * 0.53, cx - hl * 0.50, 12.8 * s, WheelStyle::CenterlockAero),
        "gt_audi_r8_gt2" => (cx + hl * 0.52, cx - hl * 0.50, 12.8 * s, WheelStyle::CenterlockAero),

        // GT1
        "gt_porsche_911_gt1_98" => (cx + hl * 0.56, cx - hl * 0.54, 13.0 * s, WheelStyle::CenterlockAero),
        "gt_mclaren_f1_gtr_lt" => (cx + hl * 0.54, cx - hl * 0.56, 13.0 * s, WheelStyle::CenterlockAero),
        "gt_mercedes_clk_gtr" => (cx + hl * 0.56, cx - hl * 0.52, 13.0 * s, WheelStyle::AlloyGT),
        "gt_nissan_r390_gt1" => (cx + hl * 0.54, cx - hl * 0.52, 12.8 * s, WheelStyle::CenterlockAero),

        // Hypercars
        "gt_ferrari_499p" | "gt_porsche_963" | "gt_toyota_gr010" | "gt_cadillac_v_series_r" => {
            (cx + hl * 0.56, cx - hl * 0.52, 13.2 * s, WheelStyle::CenterlockAero)
        }

        // NASCAR Stock Cars
        "nascar_monte_carlo_ss" | "nascar_mustang_street_stock" | "nascar_dodge_dart_street_stock" => {
            (cx + hl * 0.52, cx - hl * 0.50, 13.5 * s, WheelStyle::StockCarSteel)
        }
        "nascar_super_late_model" | "nascar_mustang_super_late_model" | "nascar_late_model_stock_car"
        | "nascar_arca_chevy_ss" | "nascar_toyota_camry_arca" | "nascar_ford_fusion_arca"
        | "nascar_silverado_truck" | "nascar_f150_truck" | "nascar_tundra_truck" => {
            (cx + hl * 0.54, cx - hl * 0.50, 13.5 * s, WheelStyle::StockCarSteel)
        }
        "nascar_corvette_ta1" | "nascar_mustang_ta1" | "nascar_challenger_ta1" => {
            (cx + hl * 0.55, cx - hl * 0.50, 13.5 * s, WheelStyle::AlloyGT)
        }

        // Rallycross & All-Terrain
        "rally_peugeot_208_rally4" | "rally_fiesta_rally4" | "rally_clio_rally4" => {
            (cx + hl * 0.48, cx - hl * 0.46, 12.5 * s, WheelStyle::RallyGravel)
        }
        "rally_hyundai_i20_rx" | "rally_polo_rx" | "rally_audi_s1_rx" => {
            (cx + hl * 0.50, cx - hl * 0.48, 13.0 * s, WheelStyle::RallyGravel)
        }
        "rally_audi_sport_quattro_s1" | "rally_peugeot_205_t16" | "rally_lancia_delta_s4" => {
            (cx + hl * 0.46, cx - hl * 0.44, 13.0 * s, WheelStyle::RallyGravel)
        }
        "rally_toyota_hilux_t1_plus" | "rally_audi_rs_q_etron" | "rally_prodrive_hunter_t1" => {
            (cx + hl * 0.52, cx - hl * 0.48, 16.5 * s, WheelStyle::MudTractorChevron)
        }
        "rally_sst_super_truck" | "rally_sst_robby_gordon" | "rally_sst_traxxas_edition" => {
            (cx + hl * 0.50, cx - hl * 0.48, 15.0 * s, WheelStyle::RallyGravel)
        }

        // Extreme Off-Road
        "offroad_sand_rail_buggy" | "offroad_polaris_rzr_pro_r" | "offroad_vw_sand_rail" => {
            (cx + hl * 0.52, cx - hl * 0.48, 14.5 * s, WheelStyle::RallyGravel)
        }
        "offroad_baja_trophy_truck" | "offroad_bettantown_trophy_truck" | "offroad_mason_awd_truck" => {
            (cx + hl * 0.52, cx - hl * 0.50, 16.5 * s, WheelStyle::RallyGravel)
        }
        "offroad_subaru_ice_racer" | "offroad_audi_quattro_ice" | "offroad_lancer_evo_ice" => {
            (cx + hl * 0.50, cx - hl * 0.48, 13.0 * s, WheelStyle::StuddedIce)
        }
        "offroad_mega_mud_truck" | "offroad_chevy_k30_mud_bogger" | "offroad_ford_f250_high_riser" => {
            (cx + hl * 0.52, cx - hl * 0.50, 19.0 * s, WheelStyle::MudTractorChevron)
        }
        "offroad_grave_crusher" | "offroad_max_d_monster" | "offroad_bigfoot_crusher" => {
            (cx + hl * 0.54, cx - hl * 0.50, 24.0 * s, WheelStyle::MonsterJam66)
        }

        // Karting
        "kart_crg_hero_60" | "kart_birel_c28" | "kart_tony_kart_neos" => {
            (cx + hl * 0.42, cx - hl * 0.42, 8.5 * s, WheelStyle::KartSmall)
        }
        "kart_tony_kart_racer_ok" | "kart_crg_kt2_ok" | "kart_birel_ry30_ok" => {
            (cx + hl * 0.45, cx - hl * 0.45, 9.2 * s, WheelStyle::KartSmall)
        }
        "kart_birel_art_kz2" | "kart_crg_road_rebel_kz" | "kart_tony_kart_racer_kz" => {
            (cx + hl * 0.45, cx - hl * 0.45, 9.5 * s, WheelStyle::KartSmall)
        }
        "kart_honda_mean_mower" | "kart_john_deere_racing_mower" | "kart_viking_t6_tractor" => {
            (cx + hl * 0.45, cx - hl * 0.45, 11.0 * s, WheelStyle::LawnmowerTurf)
        }
        "kart_anderson_cs250" | "kart_ms_superkart_250" | "kart_viper_250_twin" => {
            (cx + hl * 0.48, cx - hl * 0.48, 10.0 * s, WheelStyle::KartSmall)
        }
        _ => (cx + hl * 0.52, cx - hl * 0.50, 12.8 * s, WheelStyle::AlloyGT),
    }
}

fn render_lateral_wheel_styled(x: f32, y: f32, r: f32, s: f32, rev_intensity: f32, style: WheelStyle) {
    // 1. Black tire rubber
    draw_circle(x, y, r, Color::new(0.10, 0.11, 0.13, 1.0));
    draw_circle_lines(x, y, r, 1.2 * s, Color::new(0.20, 0.22, 0.25, 1.0));

    match style {
        WheelStyle::MonsterJam66 => {
            // Massive 66" deep V chevron tread lugs
            for i in 0..10 {
                let a = (i as f32) * std::f32::consts::PI * 2.0 / 10.0;
                let lx = x + a.cos() * r;
                let ly = y + a.sin() * r;
                let inner_x = x + a.cos() * (r * 0.82);
                let inner_y = y + a.sin() * (r * 0.82);
                draw_line(inner_x, inner_y, lx, ly, 3.0 * s, Color::new(0.35, 0.38, 0.42, 1.0));
            }
            let r_rim = r * 0.45;
            draw_circle(x, y, r_rim, Color::new(0.15, 0.70, 0.20, 1.0));
            draw_circle(x, y, r_rim * 0.5, Color::new(0.10, 0.12, 0.15, 1.0));
        }
        WheelStyle::MudTractorChevron => {
            // Tractor paddle chevron lugs
            for i in 0..8 {
                let a = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
                let lx = x + a.cos() * r;
                let ly = y + a.sin() * r;
                draw_line(x + a.cos() * (r * 0.85), y + a.sin() * (r * 0.85), lx, ly, 2.5 * s, Color::new(0.30, 0.32, 0.35, 1.0));
            }
            let r_rim = r * 0.55;
            draw_circle(x, y, r_rim, Color::new(0.85, 0.85, 0.90, 1.0));
            draw_circle(x, y, r_rim * 0.4, Color::new(0.12, 0.14, 0.18, 1.0));
        }
        WheelStyle::StuddedIce => {
            // Visible tungsten metal studs around perimeter
            for i in 0..12 {
                let a = (i as f32) * std::f32::consts::PI * 2.0 / 12.0;
                let sx = x + a.cos() * (r * 0.96);
                let sy = y + a.sin() * (r * 0.96);
                draw_circle(sx, sy, 1.2 * s, Color::new(0.95, 0.95, 1.0, 1.0));
            }
            // Gold OZ rally rim
            let r_rim = r * 0.68;
            draw_circle(x, y, r_rim, Color::new(0.85, 0.70, 0.15, 1.0));
            draw_circle(x, y, r_rim * 0.4, Color::new(0.12, 0.12, 0.15, 1.0));
        }
        WheelStyle::StockCarSteel => {
            // Yellow lettering on sidewall
            draw_circle_lines(x, y, r * 0.88, 0.8 * s, Color::new(0.95, 0.80, 0.10, 0.70));
            // Black steel wheel with 5 lug nuts
            let r_rim = r * 0.68;
            draw_circle(x, y, r_rim, Color::new(0.12, 0.12, 0.15, 1.0));
            draw_circle_lines(x, y, r_rim, 1.0 * s, Color::new(0.90, 0.75, 0.10, 0.80));
            for i in 0..5 {
                let a = (i as f32) * std::f32::consts::PI * 2.0 / 5.0;
                draw_circle(x + a.cos() * (r_rim * 0.5), y + a.sin() * (r_rim * 0.5), 1.2 * s, Palette::WHITE);
            }
        }
        WheelStyle::KartSmall | WheelStyle::LawnmowerTurf => {
            let r_rim = r * 0.60;
            draw_circle(x, y, r_rim, Color::new(0.70, 0.72, 0.76, 1.0));
            draw_circle(x, y, 2.5 * s, Color::new(0.12, 0.12, 0.15, 1.0));
        }
        _ => {
            // Standard Alloy / Centerlock Aero with brake disc and caliper
            draw_circle_lines(x, y, r * 0.88, 0.8 * s, Color::new(0.40, 0.42, 0.46, 0.60));
            let r_rim = r * 0.72;
            draw_circle(x, y, r_rim, Color::new(0.18, 0.20, 0.24, 1.0));

            // Brake rotor disc
            let r_rotor = r_rim * 0.78;
            draw_circle(x, y, r_rotor, Color::new(0.35, 0.38, 0.42, 1.0));
            draw_circle_lines(x, y, r_rotor, 0.8 * s, Color::new(0.55, 0.58, 0.62, 0.80));

            // Glowing Brake Caliper
            let cal_w = 4.0 * s;
            let cal_h = 7.0 * s;
            let cal_col = if rev_intensity > 0.05 {
                Color::new(1.0, 0.20 + 0.60 * rev_intensity, 0.05, 0.85 + 0.15 * rev_intensity)
            } else {
                Palette::RED
            };
            draw_rectangle(x + r_rotor * 0.45, y - cal_h * 0.5, cal_w, cal_h, cal_col);

            // Alloy spokes
            for i in 0..6 {
                let angle = (i as f32) * std::f32::consts::PI / 3.0;
                let sx = x + angle.cos() * (r_rim * 0.90);
                let sy = y + angle.sin() * (r_rim * 0.90);
                draw_line(x, y, sx, sy, 1.4 * s, Color::new(0.85, 0.88, 0.92, 0.90));
            }

            // Center hub nut
            draw_circle(x, y, 3.2 * s, Color::new(0.95, 0.95, 0.98, 1.0));
            draw_circle(x, y, 1.6 * s, Color::new(0.15, 0.16, 0.18, 1.0));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_specific_body(
    id: &str,
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    match id {
        // --- Classic Arcade Fantasy Roster ---
        "classic_gt" => render_porsche_718_gt4(cx, cy, gy, hl, s, primary, secondary, helmet),
        "classic_nascar" => render_nascar_monte_carlo(cx, cy, gy, hl, s, primary, secondary, helmet),
        "classic_offroad" => render_offroad_sand_rail(cx, cy, gy, hl, s, primary, secondary, helmet),
        "classic_kart" => render_kart_birel_kz2(cx, cy, gy, hl, s, primary, secondary, helmet),
        "classic_rally" => render_rally_audi_quattro_s1(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- GT4 Models ---
        "gt_porsche_718_gt4" => render_porsche_718_gt4(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_bmw_m4_gt4" => render_bmw_m4_gt4(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_aston_vantage_gt4" => render_aston_vantage_gt4(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_toyota_supra_gt4" => render_toyota_supra_gt4(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- GT3 Models ---
        "gt_porsche_911_gt3r" => render_porsche_911_gt3r(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_ferrari_296_gt3" => render_ferrari_296_gt3(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_amg_gt3_evo" => render_amg_gt3_evo(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_audi_r8_gt3_evo2" => render_audi_r8_gt3(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- GT2 Models ---
        "gt_porsche_911_gt2_rs" => render_porsche_911_gt2_rs(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_brabham_bt62_gt2" => render_brabham_bt62(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_maserati_mc20_gt2" => render_maserati_mc20_gt2(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_audi_r8_gt2" => render_audi_r8_gt2(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- GT1 Models ---
        "gt_porsche_911_gt1_98" => render_porsche_911_gt1(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_mclaren_f1_gtr_lt" => render_mclaren_f1_gtr_lt(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_mercedes_clk_gtr" => render_mercedes_clk_gtr(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_nissan_r390_gt1" => render_nissan_r390_gt1(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- Hypercars ---
        "gt_ferrari_499p" => render_ferrari_499p(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_porsche_963" => render_porsche_963(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_toyota_gr010" => render_toyota_gr010(cx, cy, gy, hl, s, primary, secondary, helmet),
        "gt_cadillac_v_series_r" => render_cadillac_v_series_r(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- NASCAR Stock Cars ---
        "nascar_monte_carlo_ss" => render_nascar_monte_carlo(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_mustang_street_stock" => render_nascar_mustang_ss(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_dodge_dart_street_stock" => render_nascar_dodge_dart_ss(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_super_late_model" => render_nascar_late_model(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_mustang_super_late_model" => render_nascar_mustang_late_model(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_late_model_stock_car" => render_nascar_perimeter_late_model(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_arca_chevy_ss" => render_nascar_arca(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_toyota_camry_arca" => render_nascar_camry_arca(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_ford_fusion_arca" => render_nascar_fusion_arca(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_silverado_truck" => render_nascar_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_f150_truck" => render_nascar_f150_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_tundra_truck" => render_nascar_tundra_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_corvette_ta1" => render_nascar_corvette_ta1(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_mustang_ta1" => render_nascar_mustang_ta1(cx, cy, gy, hl, s, primary, secondary, helmet),
        "nascar_challenger_ta1" => render_nascar_challenger_ta1(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- Rallycross & All-Terrain ---
        "rally_peugeot_208_rally4" => render_rally_peugeot_208(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_fiesta_rally4" => render_rally_fiesta_rally4(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_clio_rally4" => render_rally_clio_rally4(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_hyundai_i20_rx" => render_rally_hyundai_rx(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_polo_rx" => render_rally_polo_rx(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_audi_s1_rx" => render_rally_audi_s1_rx(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_audi_sport_quattro_s1" => render_rally_audi_quattro_s1(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_peugeot_205_t16" => render_rally_peugeot_205_t16(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_lancia_delta_s4" => render_rally_lancia_delta_s4(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_toyota_hilux_t1_plus" => render_rally_hilux_t1(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_audi_rs_q_etron" => render_rally_audi_rs_q_etron(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_prodrive_hunter_t1" => render_rally_prodrive_hunter(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_sst_super_truck" => render_rally_sst_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_sst_robby_gordon" => render_rally_sst_robby_gordon(cx, cy, gy, hl, s, primary, secondary, helmet),
        "rally_sst_traxxas_edition" => render_rally_sst_traxxas(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- Extreme Off-Road ---
        "offroad_sand_rail_buggy" => render_offroad_sand_rail(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_polaris_rzr_pro_r" => render_offroad_polaris_rzr(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_vw_sand_rail" => render_offroad_vw_baja_buggy(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_baja_trophy_truck" => render_offroad_trophy_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_bettantown_trophy_truck" => render_offroad_bettantown_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_mason_awd_truck" => render_offroad_mason_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_subaru_ice_racer" => render_offroad_subaru_ice(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_audi_quattro_ice" => render_offroad_audi_ice(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_lancer_evo_ice" => render_offroad_lancer_ice(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_mega_mud_truck" => render_offroad_mud_truck(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_chevy_k30_mud_bogger" => render_offroad_chevy_k30_bogger(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_ford_f250_high_riser" => render_offroad_ford_f250_riser(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_grave_crusher" => render_offroad_monster_jam(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_max_d_monster" => render_offroad_max_d(cx, cy, gy, hl, s, primary, secondary, helmet),
        "offroad_bigfoot_crusher" => render_offroad_bigfoot(cx, cy, gy, hl, s, primary, secondary, helmet),

        // --- Karting ---
        "kart_crg_hero_60" => render_kart_cadet_60(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_birel_c28" => render_kart_birel_c28(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_tony_kart_neos" => render_kart_tony_neos(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_tony_kart_racer_ok" => render_kart_tony_ok(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_crg_kt2_ok" => render_kart_crg_kt2(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_birel_ry30_ok" => render_kart_birel_ry30(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_birel_art_kz2" => render_kart_birel_kz2(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_crg_road_rebel_kz" => render_kart_crg_road_rebel(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_tony_kart_racer_kz" => render_kart_tony_kz(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_honda_mean_mower" => render_kart_mean_mower(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_john_deere_racing_mower" => render_kart_john_deere_mower(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_viking_t6_tractor" => render_kart_viking_tractor(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_anderson_cs250" => render_kart_superkart_250(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_ms_superkart_250" => render_kart_ms_superkart(cx, cy, gy, hl, s, primary, secondary, helmet),
        "kart_viper_250_twin" => render_kart_viper_superkart(cx, cy, gy, hl, s, primary, secondary, helmet),

        // Fallback GT Sports Car
        _ => render_porsche_718_gt4(cx, cy, gy, hl, s, primary, secondary, helmet),
    }
}

// =========================================================================
// GT4 IMPLEMENTATIONS
// =========================================================================
fn render_porsche_718_gt4(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.86;
    let sill_y = gy - 6.8 * s;
    let belt_y = cy + 2.0 * s;
    let roof_y = cy - 16.5 * s;

    // Compact mid-engine fastback body
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.45, belt_y), primary);
    draw_rectangle(cx - hl * 0.60, belt_y, hl * 1.05, sill_y - belt_y, primary);
    draw_triangle(Vec2::new(cx - hl * 0.60, belt_y), Vec2::new(tail_x, sill_y), Vec2::new(cx - hl * 0.60, sill_y), primary);

    // Front splitter & Cayman side intake scoop behind door
    draw_rectangle(nose_x - 10.0 * s, sill_y + 4.0 * s, 14.0 * s, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_triangle(Vec2::new(cx - hl * 0.25, belt_y + 1.0 * s), Vec2::new(cx - hl * 0.35, belt_y + 8.0 * s), Vec2::new(cx - hl * 0.22, belt_y + 8.0 * s), Color::new(0.08, 0.08, 0.10, 1.0));

    // RS side script stripe along bottom sill
    draw_rectangle(tail_x + 12.0 * s, sill_y + 1.0 * s, (nose_x - tail_x) - 24.0 * s, 2.2 * s, secondary);

    // Cabin / Windows (Compact mid-engine canopy with quarter scoop)
    let a_pillar_x = cx + hl * 0.22;
    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(a_pillar_x, roof_y), Vec2::new(cx - hl * 0.10, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.10, roof_y, 20.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_triangle(Vec2::new(cx - hl * 0.32, belt_y), Vec2::new(cx - hl * 0.10, roof_y), Vec2::new(cx - hl * 0.10, belt_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.02, cy - 6.0 * s, 4.0 * s, helmet);

    // GT4 RS Swan-Neck Rear Wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 1.0 * s, 1.8 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 6.0 * s, roof_y, 18.0 * s, 2.5 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    draw_rectangle(wing_x - 6.0 * s, roof_y - 2.0 * s, 2.0 * s, 6.0 * s, secondary);
}

fn render_bmw_m4_gt4(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.96;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 7.0 * s;
    let belt_y = cy + 1.5 * s;
    let roof_y = cy - 18.0 * s;

    // Muscular front-engine coupe with tall vertical nose
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.52, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.17, sill_y - belt_y, primary);
    draw_triangle(Vec2::new(cx - hl * 0.65, belt_y), Vec2::new(tail_x, sill_y), Vec2::new(cx - hl * 0.65, sill_y), primary);

    // Signature vertical twin-kidney nose graphic
    draw_rectangle(nose_x - 4.0 * s, belt_y + 1.0 * s, 4.0 * s, 8.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    // BMW M-Tricolor accent stripe along shoulder line
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 18.0 * s, 2.8 * s, secondary);
    draw_rectangle(tail_x + 10.0 * s, belt_y + 1.0 * s, (nose_x - tail_x) - 18.0 * s, 1.4 * s, Color::new(0.15, 0.65, 0.95, 1.0));

    // Cabin / Hofmeister Kink
    let a_pillar_x = cx + hl * 0.26;
    draw_triangle(Vec2::new(cx + hl * 0.40, belt_y), Vec2::new(a_pillar_x, roof_y), Vec2::new(cx - hl * 0.15, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.15, roof_y, 24.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.05, cy - 6.5 * s, 4.2 * s, helmet);

    // High GT4 Rear Wing
    let wing_x = tail_x + 4.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 2.0 * s, 1.8 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 8.0 * s, roof_y + 1.0 * s, 20.0 * s, 2.5 * s, Color::new(0.08, 0.08, 0.10, 1.0));
}

fn render_aston_vantage_gt4(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 6.8 * s;
    let belt_y = cy + 2.0 * s;
    let roof_y = cy - 17.0 * s;

    // Low shark nose and sweeping front-mid hood
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.50, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.15, sill_y - belt_y, primary);

    // Lime AMR racing accent line
    draw_rectangle(tail_x + 8.0 * s, sill_y + 4.0 * s, (nose_x - tail_x) - 12.0 * s, 2.0 * s, secondary);
    // Side strake air gill
    draw_line(cx + hl * 0.25, belt_y + 4.0 * s, cx + hl * 0.12, belt_y + 4.0 * s, 2.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));

    // Greenhouse
    draw_triangle(Vec2::new(cx + hl * 0.38, belt_y), Vec2::new(cx + hl * 0.22, roof_y), Vec2::new(cx - hl * 0.15, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.15, roof_y, 22.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.05, cy - 6.0 * s, 4.0 * s, helmet);

    // Vantage pedestal GT wing
    let wing_x = tail_x + 5.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 3.0 * s, 1.6 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 6.0 * s, roof_y + 2.0 * s, 18.0 * s, 2.5 * s, Color::new(0.08, 0.08, 0.10, 1.0));
}

fn render_toyota_supra_gt4(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.86;
    let sill_y = gy - 6.8 * s;
    let belt_y = cy + 2.2 * s;
    let roof_y = cy - 16.5 * s;

    // Pointy nosecone and double-bubble roof
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.44, belt_y), Vec2::new(cx - hl * 0.45, belt_y), primary);
    draw_rectangle(cx - hl * 0.60, belt_y, hl * 1.04, sill_y - belt_y, primary);

    // Gazoo Racing two-tone graphic
    draw_triangle(Vec2::new(cx - hl * 0.40, belt_y), Vec2::new(tail_x, sill_y), Vec2::new(cx - hl * 0.20, sill_y), secondary);

    // Double-bubble roof hump
    draw_circle(cx - hl * 0.05, roof_y, 8.0 * s, primary);
    draw_circle(cx - hl * 0.02, cy - 5.5 * s, 4.0 * s, helmet);

    // Ducktail GT4 Wing
    let wing_x = tail_x + 5.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 2.0 * s, 1.8 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 6.0 * s, roof_y + 1.0 * s, 18.0 * s, 2.5 * s, secondary);
}

// =========================================================================
// GT3 IMPLEMENTATIONS
// =========================================================================
fn render_porsche_911_gt3r(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.8 * s;
    let roof_y = cy - 18.0 * s;

    // Seamless 911 teardrop slope from roof to tail
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.45, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);
    draw_triangle(Vec2::new(cx - hl * 0.65, belt_y), Vec2::new(tail_x, sill_y), Vec2::new(cx - hl * 0.65, sill_y), primary);

    // Dual front hood extraction nostrils
    draw_rectangle(cx + hl * 0.45, belt_y + 1.0 * s, 8.0 * s, 2.5 * s, Color::new(0.08, 0.08, 0.10, 1.0));

    // Racing door number plate
    draw_rectangle(cx - hl * 0.10, belt_y + 1.5 * s, 16.0 * s, 12.0 * s, Palette::WHITE);
    draw_rectangle_lines(cx - hl * 0.10, belt_y + 1.5 * s, 16.0 * s, 12.0 * s, 1.0 * s, secondary);

    // Continuous 911 rear fastback roofline
    draw_triangle(Vec2::new(cx + hl * 0.36, belt_y), Vec2::new(cx + hl * 0.20, roof_y), Vec2::new(cx - hl * 0.15, roof_y), Color::new(0.12, 0.15, 0.20, 0.88));
    draw_triangle(Vec2::new(cx - hl * 0.55, belt_y), Vec2::new(cx - hl * 0.15, roof_y), Vec2::new(cx - hl * 0.15, belt_y), Color::new(0.12, 0.15, 0.20, 0.88));
    draw_circle(cx - hl * 0.05, cy - 6.5 * s, 4.2 * s, helmet);

    // Massive GT3 R Swan-Neck Rear Wing
    let wing_x = tail_x + 8.0 * s;
    draw_line(wing_x, belt_y, wing_x - 3.0 * s, roof_y + 1.0 * s, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 1.0 * s, 22.0 * s, 3.0 * s, secondary);
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 2.5 * s, 8.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
}

fn render_ferrari_296_gt3(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.98;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 6.0 * s;
    let belt_y = cy + 2.5 * s;
    let roof_y = cy - 16.5 * s;

    // Ultra-low 120° V6 mid-engine profile with flying buttresses
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.15, sill_y - belt_y, primary);

    // Dual front dive planes / canards
    draw_line(nose_x - 8.0 * s, belt_y + 3.0 * s, nose_x + 2.0 * s, belt_y, 1.8 * s, secondary);
    draw_line(nose_x - 10.0 * s, belt_y + 6.0 * s, nose_x - 1.0 * s, belt_y + 3.0 * s, 1.8 * s, secondary);

    // Giallo Modena yellow racing accents
    draw_rectangle(tail_x + 12.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 24.0 * s, 3.2 * s, secondary);

    // Cockpit & Flying Buttress trailing pillars
    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(cx + hl * 0.20, roof_y), Vec2::new(cx - hl * 0.15, roof_y), Color::new(0.12, 0.15, 0.20, 0.88));
    draw_triangle(Vec2::new(cx - hl * 0.50, belt_y), Vec2::new(cx - hl * 0.15, roof_y), Vec2::new(cx - hl * 0.15, belt_y), primary); // Flying buttress!
    draw_circle(cx - hl * 0.05, cy - 5.5 * s, 4.2 * s, helmet);

    // High downforce swan-neck wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x - 2.0 * s, roof_y, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 2.0 * s, 24.0 * s, 3.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 5.0 * s, 2.5 * s, 8.0 * s, secondary);
}

fn render_amg_gt3_evo(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 1.00;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.5 * s;
    let roof_y = cy - 18.0 * s;

    // Extra long hood and massive Panamericana vertical nose grille
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.58, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.23, sill_y - belt_y, primary);

    // Panamericana grille vertical slats
    draw_rectangle(nose_x - 5.0 * s, belt_y + 1.0 * s, 5.0 * s, 9.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    for i in 0..3 {
        draw_line(nose_x - 4.0 * s + (i as f32) * 1.5 * s, belt_y + 2.0 * s, nose_x - 4.0 * s + (i as f32) * 1.5 * s, belt_y + 9.0 * s, 1.0 * s, Palette::WHITE);
    }
    // Petronas turquoise accent stripe
    draw_rectangle(tail_x + 10.0 * s, sill_y + 4.5 * s, (nose_x - tail_x) - 15.0 * s, 2.2 * s, secondary);

    // Cab-backward greenhouse
    let a_pillar_x = cx + hl * 0.20;
    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(a_pillar_x, roof_y), Vec2::new(cx - hl * 0.20, roof_y), Color::new(0.12, 0.15, 0.20, 0.88));
    draw_rectangle(cx - hl * 0.20, roof_y, 22.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.88));
    draw_circle(cx - hl * 0.08, cy - 6.5 * s, 4.2 * s, helmet);

    // Swan-neck GT wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 1.0 * s, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y, 22.0 * s, 3.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
}

fn render_audi_r8_gt3(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 2.0 * s;
    let roof_y = cy - 17.5 * s;

    // Mid-engine wedge with signature Audi sideblades
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.46, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.11, sill_y - belt_y, primary);

    // Carbon Sideblade behind door
    draw_rectangle(cx - hl * 0.35, belt_y - 2.0 * s, 10.0 * s, 14.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    draw_line(cx - hl * 0.35, belt_y - 2.0 * s, cx - hl * 0.35, belt_y + 12.0 * s, 2.0 * s, secondary);

    // Roof intake scoop
    draw_triangle(Vec2::new(cx - hl * 0.05, roof_y - 4.0 * s), Vec2::new(cx + hl * 0.10, roof_y - 1.0 * s), Vec2::new(cx - hl * 0.15, roof_y - 1.0 * s), secondary);
    draw_circle(cx - hl * 0.05, cy - 6.0 * s, 4.2 * s, helmet);

    // Rear GT wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 1.0 * s, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y, 22.0 * s, 3.0 * s, primary);
}

// =========================================================================
// GT2 IMPLEMENTATIONS
// =========================================================================
fn render_porsche_911_gt2_rs(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_porsche_911_gt3r(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Large rear fender intercooler air scoop
    draw_rectangle(cx - hl * 0.35, cy + 6.0 * s, 8.0 * s, 5.0 * s, Color::new(0.05, 0.05, 0.07, 0.95));
}

fn render_brabham_bt62(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 1.00;
    let tail_x = cx - hl * 0.94;
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.0 * s;
    let roof_y = cy - 15.0 * s;

    // Ultralight low track monocoque
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.42, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.25, sill_y - belt_y, primary);

    // Historic Gold racing stripe
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 20.0 * s, 3.5 * s, secondary);
    draw_circle(cx - hl * 0.05, cy - 5.0 * s, 4.0 * s, helmet);

    // Enormous rear wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x - 3.0 * s, roof_y - 2.0 * s, 2.2 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 3.0 * s, 26.0 * s, 3.5 * s, secondary);
}

fn render_maserati_mc20_gt2(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.96;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 6.0 * s;
    let belt_y = cy + 2.5 * s;
    let roof_y = cy - 16.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Roof air scoop feeding Nettuno V6
    draw_triangle(Vec2::new(cx - hl * 0.05, roof_y - 4.5 * s), Vec2::new(cx + hl * 0.10, roof_y - 1.0 * s), Vec2::new(cx - hl * 0.15, roof_y - 1.0 * s), secondary);
    draw_circle(cx - hl * 0.05, cy - 5.5 * s, 4.0 * s, helmet);

    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 1.0 * s, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y, 22.0 * s, 3.0 * s, secondary);
}

fn render_audi_r8_gt2(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_audi_r8_gt3(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Elevated giant ram-air roof snorkel
    let roof_y = cy - 17.5 * s;
    draw_rectangle(cx - hl * 0.18, roof_y - 7.0 * s, 14.0 * s, 7.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    draw_triangle(Vec2::new(cx + hl * 0.05, roof_y - 7.0 * s), Vec2::new(cx - hl * 0.18, roof_y - 7.0 * s), Vec2::new(cx - hl * 0.18, roof_y), secondary);
}

// =========================================================================
// GT1 IMPLEMENTATIONS
// =========================================================================
fn render_porsche_911_gt1(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 1.02;
    let tail_x = cx - hl * 0.98;
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.5 * s;
    let roof_y = cy - 14.0 * s;

    // Ultra-low elongated Le Mans 1998 winner body
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.40, belt_y), Vec2::new(cx - hl * 0.60, belt_y), primary);
    draw_rectangle(cx - hl * 0.75, belt_y, hl * 1.30, sill_y - belt_y, primary);

    // Mobil 1 blue swoosh livery graphic
    draw_triangle(Vec2::new(cx - hl * 0.30, belt_y), Vec2::new(tail_x + 10.0 * s, sill_y), Vec2::new(cx + hl * 0.10, sill_y), secondary);
    draw_circle(cx - hl * 0.05, cy - 4.0 * s, 4.0 * s, helmet);

    let wing_x = tail_x + 4.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.2 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y - 4.0 * s, 24.0 * s, 3.0 * s, secondary);
}

fn render_mclaren_f1_gtr_lt(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 1.00;
    let tail_x = cx - hl * 1.04; // Longtail rear extension!
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.2 * s;
    let roof_y = cy - 14.5 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.42, belt_y), Vec2::new(cx - hl * 0.65, belt_y), primary);
    draw_rectangle(cx - hl * 0.80, belt_y, hl * 1.35, sill_y - belt_y, primary);

    // Central roof intake scoop
    draw_triangle(Vec2::new(cx - hl * 0.05, roof_y - 3.5 * s), Vec2::new(cx + hl * 0.10, roof_y), Vec2::new(cx - hl * 0.15, roof_y), secondary);
    draw_circle(cx - hl * 0.02, cy - 4.5 * s, 4.0 * s, helmet);

    // Giant longtail rear wing on twin uprights
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 3.0 * s, 2.2 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y - 5.0 * s, 24.0 * s, 3.0 * s, secondary);
}

fn render_mercedes_clk_gtr(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.98;
    let tail_x = cx - hl * 0.94;
    let sill_y = gy - 6.0 * s;
    let belt_y = cy + 2.5 * s;
    let roof_y = cy - 15.5 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.50, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.20, sill_y - belt_y, primary);

    // Classic 90s horizontal Mercedes grille front
    draw_rectangle(nose_x - 6.0 * s, belt_y + 2.0 * s, 6.0 * s, 6.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_circle(cx - hl * 0.05, cy - 5.0 * s, 4.0 * s, helmet);

    let wing_x = tail_x + 5.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y - 2.0 * s, 22.0 * s, 3.0 * s, secondary);
}

fn render_nissan_r390_gt1(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.98;
    let tail_x = cx - hl * 0.96;
    let sill_y = gy - 6.0 * s;
    let belt_y = cy + 2.8 * s;
    let roof_y = cy - 15.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.44, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.20, sill_y - belt_y, primary);

    // Calsonic blue and red graphics
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 20.0 * s, 3.0 * s, secondary);
    draw_circle(cx - hl * 0.05, cy - 5.0 * s, 4.0 * s, helmet);

    let wing_x = tail_x + 5.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y - 2.0 * s, 22.0 * s, 3.0 * s, primary);
}

// =========================================================================
// HYPERCAR PROTOTYPES (LMH / LMDh)
// =========================================================================
fn render_ferrari_499p(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 1.00;
    let tail_x = cx - hl * 0.96;
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.0 * s;
    let roof_y = cy - 16.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.42, belt_y), Vec2::new(cx - hl * 0.60, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.25, sill_y - belt_y, primary);

    // Front horizontal blade LED light bar
    draw_line(nose_x - 12.0 * s, belt_y + 3.0 * s, nose_x, belt_y + 2.0 * s, 2.0 * s, Palette::NEON_GOLD);

    // Cockpit bubble
    draw_circle(cx + hl * 0.02, cy - 6.0 * s, 10.0 * s, Color::new(0.10, 0.14, 0.18, 0.92));
    draw_circle(cx - hl * 0.02, cy - 6.0 * s, 4.0 * s, helmet);

    // Central Dorsal Shark Fin along spine
    draw_triangle(Vec2::new(cx + hl * 0.05, roof_y), Vec2::new(cx - hl * 0.55, roof_y - 2.0 * s), Vec2::new(cx - hl * 0.55, belt_y), secondary);

    // High Mount Hypercar Bi-Plane Wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.2 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 26.0 * s, 3.2 * s, primary);
    draw_rectangle(wing_x - 10.0 * s, roof_y + 1.0 * s, 22.0 * s, 2.0 * s, secondary);
}

fn render_porsche_963(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 1.00;
    let tail_x = cx - hl * 0.96;
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.0 * s;
    let roof_y = cy - 16.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.42, belt_y), Vec2::new(cx - hl * 0.60, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.25, sill_y - belt_y, primary);

    // 4-point Porsche LED headlights in front fender
    for i in 0..2 {
        for j in 0..2 {
            draw_circle(nose_x - 8.0 * s + (i as f32) * 3.0 * s, belt_y + 3.0 * s + (j as f32) * 2.5 * s, 1.0 * s, Palette::WHITE);
        }
    }

    draw_circle(cx + hl * 0.02, cy - 6.0 * s, 10.0 * s, Color::new(0.10, 0.14, 0.18, 0.92));
    draw_circle(cx - hl * 0.02, cy - 6.0 * s, 4.0 * s, helmet);

    // Large dorsal shark fin
    draw_triangle(Vec2::new(cx + hl * 0.05, roof_y), Vec2::new(cx - hl * 0.55, roof_y - 2.0 * s), Vec2::new(cx - hl * 0.55, belt_y), secondary);

    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.2 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 26.0 * s, 3.2 * s, primary);
}

fn render_toyota_gr010(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_porsche_963(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Gazoo Racing two-tone red & matte black split
    draw_triangle(Vec2::new(cx - hl * 0.40, cy + 3.0 * s), Vec2::new(cx - hl * 0.96, gy - 5.5 * s), Vec2::new(cx - hl * 0.20, gy - 5.5 * s), secondary);
}

fn render_cadillac_v_series_r(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 1.00;
    let tail_x = cx - hl * 0.96;
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.0 * s;
    let roof_y = cy - 16.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.42, belt_y), Vec2::new(cx - hl * 0.60, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.25, sill_y - belt_y, primary);

    // Cadillac vertical blade LED light bar
    draw_line(nose_x - 4.0 * s, belt_y + 1.0 * s, nose_x - 4.0 * s, belt_y + 7.0 * s, 2.0 * s, Palette::WHITE);

    draw_circle(cx + hl * 0.02, cy - 6.0 * s, 10.0 * s, Color::new(0.10, 0.14, 0.18, 0.92));
    draw_circle(cx - hl * 0.02, cy - 6.0 * s, 4.0 * s, helmet);

    draw_triangle(Vec2::new(cx + hl * 0.05, roof_y), Vec2::new(cx - hl * 0.55, roof_y - 2.0 * s), Vec2::new(cx - hl * 0.55, belt_y), secondary);

    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.2 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 26.0 * s, 3.2 * s, primary);
}

// =========================================================================
// NASCAR STOCK CAR IMPLEMENTATIONS
// =========================================================================
fn render_nascar_monte_carlo(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.92;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 7.0 * s;
    let belt_y = cy + 1.0 * s;
    let roof_y = cy - 18.0 * s;

    // Boxy 80s muscle notchback body
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 7.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Front/rear chrome/steel crash bumpers
    draw_rectangle(nose_x - 4.0 * s, sill_y + 2.0 * s, 6.0 * s, 4.0 * s, Color::new(0.85, 0.88, 0.92, 1.0));
    draw_rectangle(tail_x - 2.0 * s, sill_y + 2.0 * s, 6.0 * s, 4.0 * s, Color::new(0.85, 0.88, 0.92, 1.0));

    // Door Number #24 plate
    draw_rectangle(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, Palette::WHITE);
    draw_rectangle_lines(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, 1.2 * s, secondary);

    // Upright notchback greenhouse
    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(cx + hl * 0.18, roof_y), Vec2::new(cx - hl * 0.20, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.20, roof_y, 22.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);

    // Flat trunk deck with low spoiler
    draw_line(tail_x + 2.0 * s, belt_y, tail_x - 2.0 * s, belt_y - 5.0 * s, 2.5 * s, Color::new(0.10, 0.10, 0.12, 1.0));
}

fn render_nascar_mustang_ss(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 7.0 * s;
    let belt_y = cy + 1.2 * s;
    let roof_y = cy - 18.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 7.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.48, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.13, sill_y - belt_y, primary);

    // White racing hood stripes
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 20.0 * s, 2.5 * s, secondary);

    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(cx + hl * 0.20, roof_y), Vec2::new(cx - hl * 0.20, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.20, roof_y, 22.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);
}

fn render_nascar_dodge_dart_ss(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.92;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 7.0 * s;
    let belt_y = cy + 1.0 * s;
    let roof_y = cy - 18.0 * s;

    // Classic 1970s Mopar A-body notchback profile
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 7.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.44, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.09, sill_y - belt_y, primary);

    // Front hood scoop & classic bumblebee tail stripe
    draw_rectangle(cx + hl * 0.20, belt_y - 3.0 * s, 10.0 * s, 3.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(tail_x + 8.0 * s, belt_y - 2.0 * s, 8.0 * s, sill_y - belt_y + 4.0 * s, secondary);

    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(cx + hl * 0.16, roof_y), Vec2::new(cx - hl * 0.22, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.22, roof_y, 24.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);
}

fn render_nascar_late_model(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.96;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.0 * s;

    // Asymmetric wedge nose & high quarter panels
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.48, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.18, sill_y - belt_y, primary);

    // Tall aggressive rear blade spoiler
    draw_line(tail_x + 2.0 * s, belt_y, tail_x - 6.0 * s, belt_y - 12.0 * s, 3.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));

    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);
    draw_rectangle(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, secondary);
}

fn render_nascar_mustang_late_model(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.96;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.50, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.20, sill_y - belt_y, primary);

    // Front carbon splitter & double rear blade spoiler
    draw_rectangle(nose_x - 10.0 * s, sill_y + 4.0 * s, 14.0 * s, 2.5 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    draw_line(tail_x + 2.0 * s, belt_y, tail_x - 8.0 * s, belt_y - 14.0 * s, 3.2 * s, secondary);

    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);
    draw_rectangle(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, secondary);
}

fn render_nascar_perimeter_late_model(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.95;
    let tail_x = cx - hl * 0.89;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.46, belt_y), Vec2::new(cx - hl * 0.52, belt_y), primary);
    draw_rectangle(cx - hl * 0.68, belt_y, hl * 1.14, sill_y - belt_y, primary);

    // Yellow tubular perimeter chassis bars along roof
    draw_line(cx + hl * 0.18, cy - 18.0 * s, cx - hl * 0.22, cy - 18.0 * s, 2.5 * s, Palette::NEON_GOLD);
    draw_line(tail_x + 2.0 * s, belt_y, tail_x - 5.0 * s, belt_y - 11.0 * s, 2.8 * s, Color::new(0.08, 0.08, 0.10, 1.0));

    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);
    draw_rectangle(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, secondary);
}

fn render_nascar_arca(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_stock_car(cx, cy, gy, hl, s, primary, secondary, helmet);
}

fn render_nascar_camry_arca(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_stock_car(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Trapezoidal front nose graphic & TRD red side stripe
    let nose_x = cx + hl * 0.94;
    let sill_y = gy - 6.5 * s;
    draw_triangle(Vec2::new(nose_x, sill_y + 2.0 * s), Vec2::new(nose_x - 8.0 * s, sill_y - 6.0 * s), Vec2::new(nose_x - 8.0 * s, sill_y + 2.0 * s), Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(cx - hl * 0.50, cy + 2.0 * s, hl * 0.80, 2.5 * s, secondary);
}

fn render_nascar_fusion_arca(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_stock_car(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Hexagonal front grille graphic & dual hood heat extractors
    let nose_x = cx + hl * 0.94;
    let sill_y = gy - 6.5 * s;
    draw_rectangle(nose_x - 6.0 * s, sill_y - 4.0 * s, 5.0 * s, 5.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_line(cx + hl * 0.30, cy + 1.0 * s, cx + hl * 0.15, cy + 1.0 * s, 2.0 * s, secondary);
}

fn render_nascar_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 6.8 * s;
    let belt_y = cy + 1.0 * s;
    let roof_y = cy - 20.0 * s;

    // Pickup truck body with flat bed tonneau cover
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.8 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.45, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Upright Truck Cab
    let a_pillar_x = cx + hl * 0.20;
    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(a_pillar_x, roof_y), Vec2::new(cx - hl * 0.10, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.10, roof_y, 18.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.02, cy - 8.0 * s, 4.4 * s, helmet);

    // Flat bed with tall rear tailgate spoiler
    draw_rectangle(cx - hl * 0.65, belt_y - 1.5 * s, hl * 0.55, 2.0 * s, Color::new(0.08, 0.08, 0.10, 1.0)); // tonneau
    draw_line(tail_x + 2.0 * s, belt_y, tail_x - 5.0 * s, belt_y - 10.0 * s, 3.0 * s, secondary);
}

fn render_nascar_f150_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_nascar_truck(cx, cy, gy, hl, s, primary, secondary, helmet);
    // C-clamp LED headlights & Ford Blue Oval accent
    let nose_x = cx + hl * 0.94;
    let sill_y = gy - 6.8 * s;
    draw_line(nose_x - 3.0 * s, sill_y - 4.0 * s, nose_x - 1.0 * s, sill_y + 2.0 * s, 2.5 * s, Palette::WHITE);
    draw_circle(nose_x - 8.0 * s, sill_y + 1.0 * s, 2.5 * s, Color::new(0.10, 0.35, 0.85, 1.0));
}

fn render_nascar_tundra_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_nascar_truck(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Bold hexagonal grille & red TRD side decals
    let nose_x = cx + hl * 0.94;
    let sill_y = gy - 6.8 * s;
    draw_rectangle(nose_x - 6.0 * s, sill_y - 3.0 * s, 5.0 * s, 6.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(cx - hl * 0.40, cy + 2.0 * s, 20.0 * s, 2.5 * s, Palette::RED);
}

fn render_nascar_corvette_ta1(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.98;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 6.0 * s;
    let belt_y = cy + 2.0 * s;
    let roof_y = cy - 17.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.52, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.22, sill_y - belt_y, primary);

    // Boom-tube side exhaust in front of rear wheel
    draw_rectangle(cx - hl * 0.35, sill_y + 2.0 * s, 10.0 * s, 3.0 * s, Color::new(0.40, 0.42, 0.46, 1.0));
    draw_circle(cx - hl * 0.05, cy - 6.0 * s, 4.2 * s, helmet);

    // Giant High-Mount TA1 Rear Wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.4 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 24.0 * s, 3.5 * s, secondary);
}

fn render_nascar_mustang_ta1(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.97;
    let tail_x = cx - hl * 0.91;
    let sill_y = gy - 6.0 * s;
    let belt_y = cy + 2.0 * s;
    let roof_y = cy - 17.5 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.50, belt_y), Vec2::new(cx - hl * 0.52, belt_y), primary);
    draw_rectangle(cx - hl * 0.68, belt_y, hl * 1.18, sill_y - belt_y, primary);

    // Large hood cowl induction bulge & side boom-tube exhaust
    draw_rectangle(cx + hl * 0.15, belt_y - 3.0 * s, 18.0 * s, 3.5 * s, secondary);
    draw_rectangle(cx - hl * 0.35, sill_y + 2.0 * s, 10.0 * s, 3.0 * s, Color::new(0.40, 0.42, 0.46, 1.0));
    draw_circle(cx - hl * 0.05, cy - 6.5 * s, 4.2 * s, helmet);

    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.4 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 24.0 * s, 3.5 * s, secondary);
}

fn render_nascar_challenger_ta1(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.96;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 6.0 * s;
    let belt_y = cy + 1.8 * s;
    let roof_y = cy - 17.5 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.48, belt_y), Vec2::new(cx - hl * 0.54, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.18, sill_y - belt_y, primary);

    // Dual snorkel hood scoops & side exhaust
    draw_rectangle(cx + hl * 0.22, belt_y - 2.5 * s, 8.0 * s, 2.5 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(cx - hl * 0.35, sill_y + 2.0 * s, 10.0 * s, 3.0 * s, Color::new(0.40, 0.42, 0.46, 1.0));
    draw_circle(cx - hl * 0.05, cy - 6.5 * s, 4.2 * s, helmet);

    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.4 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 24.0 * s, 3.5 * s, secondary);
}

fn render_rally_peugeot_208(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_rally(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Peugeot "fang" front LED light
    let nose_x = cx + hl * 0.88;
    let sill_y = gy - 7.5 * s;
    draw_line(nose_x - 4.0 * s, sill_y - 2.0 * s, nose_x - 2.0 * s, sill_y + 4.0 * s, 2.0 * s, Palette::NEON_CYAN);
}

fn render_rally_hyundai_rx(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_rally(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Double-tier RX rear wing
    let tail_x = cx - hl * 0.84;
    let roof_y = cy - 20.0 * s;
    draw_rectangle(tail_x - 10.0 * s, roof_y - 4.0 * s, 12.0 * s, 2.0 * s, secondary);
}

fn render_rally_fiesta_rally4(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_rally(cx, cy, gy, hl, s, primary, secondary, helmet);
    // M-Sport angular front styling & RS roof wing
    let nose_x = cx + hl * 0.88;
    let sill_y = gy - 7.5 * s;
    draw_line(nose_x - 6.0 * s, sill_y - 3.0 * s, nose_x - 1.0 * s, sill_y, 2.0 * s, Palette::WHITE);
    draw_rectangle(cx - hl * 0.40, cy + 2.0 * s, hl * 0.70, 2.2 * s, secondary);
}

fn render_rally_clio_rally4(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_rally(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Renault C-shape LED lights & Renault Sport yellow accents
    let nose_x = cx + hl * 0.88;
    let sill_y = gy - 7.5 * s;
    draw_circle_lines(nose_x - 4.0 * s, sill_y - 1.0 * s, 3.0 * s, 1.5 * s, Palette::NEON_GOLD);
    draw_rectangle(tail_x_val(cx, hl) + 6.0 * s, sill_y + 1.0 * s, hl * 0.90, 2.0 * s, secondary);
}

fn render_rally_polo_rx(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_rally(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Large front intercooler & dual roof NACA ducts
    let nose_x = cx + hl * 0.88;
    let sill_y = gy - 7.5 * s;
    draw_rectangle(nose_x - 10.0 * s, sill_y + 1.0 * s, 8.0 * s, 4.0 * s, Color::new(0.60, 0.62, 0.66, 1.0));
}

fn render_rally_audi_s1_rx(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_rally(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Blistered quattro box arches & red/black EKS livery
    let tail_x = cx - hl * 0.84;
    let roof_y = cy - 20.0 * s;
    draw_rectangle(tail_x - 12.0 * s, roof_y - 5.0 * s, 14.0 * s, 2.5 * s, secondary);
    draw_rectangle(cx - hl * 0.30, cy + 2.0 * s, hl * 0.60, 3.0 * s, Palette::RED);
}

fn render_rally_audi_quattro_s1(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.92;
    let tail_x = cx - hl * 0.86;
    let sill_y = gy - 7.5 * s;
    let belt_y = cy + 1.5 * s;
    let roof_y = cy - 19.0 * s;

    // Short wheelbase Group B beast
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Giant Front Snowplow Splitter
    draw_rectangle(nose_x - 14.0 * s, sill_y + 3.0 * s, 20.0 * s, 4.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));

    // Yellow and White Audi Sport graphics
    draw_rectangle(tail_x + 8.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 16.0 * s, 3.5 * s, secondary);
    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);

    // Enormous Bi-Plane Group B Rear Wing
    draw_line(tail_x, roof_y + 2.0 * s, tail_x - 8.0 * s, roof_y - 8.0 * s, 2.8 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(tail_x - 16.0 * s, roof_y - 10.0 * s, 20.0 * s, 3.5 * s, primary);
    draw_rectangle(tail_x - 14.0 * s, roof_y - 4.0 * s, 16.0 * s, 2.5 * s, secondary);
}

fn render_rally_peugeot_205_t16(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, _secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.86;
    let sill_y = gy - 7.5 * s;
    let belt_y = cy + 1.5 * s;
    let roof_y = cy - 18.5 * s;

    // Iconic Group B mid-engine rear clamshell
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.44, belt_y), Vec2::new(cx - hl * 0.48, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.09, sill_y - belt_y, primary);

    // Peugeot Talbot Sport tricolor stripes (Blue, Yellow, Red)
    draw_rectangle(tail_x + 8.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 16.0 * s, 1.5 * s, Color::new(0.15, 0.55, 0.95, 1.0));
    draw_rectangle(tail_x + 8.0 * s, belt_y - 0.5 * s, (nose_x - tail_x) - 16.0 * s, 1.5 * s, Palette::NEON_GOLD);
    draw_rectangle(tail_x + 8.0 * s, belt_y + 1.0 * s, (nose_x - tail_x) - 16.0 * s, 1.5 * s, Palette::RED);

    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);

    // Massive Evo 2 high rear wing
    draw_line(tail_x, roof_y + 2.0 * s, tail_x - 6.0 * s, roof_y - 8.0 * s, 2.5 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(tail_x - 14.0 * s, roof_y - 10.0 * s, 18.0 * s, 3.2 * s, primary);
}

fn render_rally_lancia_delta_s4(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, _secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 7.5 * s;
    let belt_y = cy + 1.5 * s;
    let roof_y = cy - 19.0 * s;

    // Twincharged Group B monster with roof scoop and rear wing
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.44, belt_y), Vec2::new(cx - hl * 0.52, belt_y), primary);
    draw_rectangle(cx - hl * 0.68, belt_y, hl * 1.12, sill_y - belt_y, primary);

    // Martini racing stripes
    draw_rectangle(tail_x + 8.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 16.0 * s, 2.0 * s, Color::new(0.15, 0.65, 0.95, 1.0));
    draw_rectangle(tail_x + 8.0 * s, belt_y, (nose_x - tail_x) - 16.0 * s, 2.0 * s, Palette::RED);
    draw_rectangle(tail_x + 8.0 * s, belt_y + 2.0 * s, (nose_x - tail_x) - 16.0 * s, 2.0 * s, Color::new(0.08, 0.15, 0.45, 1.0));

    // High roof intercooler scoop
    draw_triangle(Vec2::new(cx - hl * 0.10, roof_y - 5.0 * s), Vec2::new(cx + hl * 0.05, roof_y), Vec2::new(cx - hl * 0.20, roof_y), Color::new(0.12, 0.12, 0.15, 1.0));

    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);

    let wing_x = tail_x + 2.0 * s;
    draw_line(wing_x, belt_y, wing_x - 4.0 * s, roof_y - 4.0 * s, 2.2 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 6.0 * s, 16.0 * s, 3.0 * s, primary);
}

fn render_rally_hilux_t1(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, _secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 12.0 * s; // High Dakar clearance!
    let belt_y = cy - 2.0 * s;
    let _roof_y = cy - 22.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 8.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.45, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Snorkel air intake running up A-pillar
    draw_line(cx + hl * 0.35, sill_y, cx + hl * 0.15, cy - 24.0 * s, 2.5 * s, Color::new(0.12, 0.12, 0.15, 1.0));

    // Spare tires visible in rear bed
    draw_circle(cx - hl * 0.40, belt_y + 2.0 * s, 8.0 * s, Color::new(0.12, 0.14, 0.18, 1.0));
    draw_circle(cx - hl * 0.05, cy - 9.0 * s, 4.4 * s, helmet);
}

fn render_rally_audi_rs_q_etron(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 12.0 * s;
    let belt_y = cy - 2.0 * s;
    let roof_y = cy - 22.0 * s;

    // Futuristic Dakar prototype with central dorsal fin
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 8.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.15, sill_y - belt_y, primary);

    // Central roof dorsal fin & neon orange high-voltage accents
    draw_triangle(Vec2::new(cx + hl * 0.05, roof_y), Vec2::new(cx - hl * 0.45, roof_y - 3.0 * s), Vec2::new(cx - hl * 0.45, belt_y), secondary);
    draw_line(nose_x - 8.0 * s, belt_y + 2.0 * s, nose_x, belt_y + 2.0 * s, 2.5 * s, Palette::NEON_ORANGE);

    draw_circle(cx - hl * 0.05, cy - 9.0 * s, 4.4 * s, helmet);
}

fn render_rally_prodrive_hunter(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 12.0 * s;
    let belt_y = cy - 2.0 * s;

    // Sculpted aerodynamic body designed by Ian Callum
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 8.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.48, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.68, belt_y, hl * 1.16, sill_y - belt_y, primary);

    // Red Bull / Bahrain Raid Xtreme graphics
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 20.0 * s, 3.5 * s, secondary);
    draw_circle(cx - hl * 0.35, belt_y + 2.0 * s, 8.0 * s, Color::new(0.12, 0.14, 0.18, 1.0));
    draw_circle(cx - hl * 0.05, cy - 9.0 * s, 4.4 * s, helmet);
}

fn render_offroad_sand_rail(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_sand_rail(cx, cy, gy, hl, s, primary, secondary, helmet);
}

fn render_offroad_polaris_rzr(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.88;
    let tail_x = cx - hl * 0.84;
    let sill_y = gy - 10.0 * s;
    let cage_roof_y = cy - 21.0 * s;

    // Angular modern factory UTV spaceframe
    draw_line(nose_x, sill_y, cx - hl * 0.40, sill_y, 3.0 * s, primary);
    draw_line(cx + hl * 0.30, sill_y, cx + hl * 0.08, cage_roof_y, 2.5 * s, primary);
    draw_line(cx + hl * 0.08, cage_roof_y, cx - hl * 0.35, cage_roof_y, 2.5 * s, primary);
    draw_line(cx - hl * 0.35, cage_roof_y, tail_x, sill_y, 2.5 * s, primary);

    // Front high LED blade bar & beadlock colors
    draw_rectangle(nose_x - 8.0 * s, sill_y - 4.0 * s, 8.0 * s, 2.5 * s, Palette::WHITE);
    draw_rectangle(cx - hl * 0.05, cage_roof_y - 3.0 * s, 14.0 * s, 2.5 * s, secondary);

    draw_circle(cx - hl * 0.05, cy - 8.5 * s, 4.4 * s, helmet);
}

fn render_offroad_vw_baja_buggy(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.86;
    let tail_x = cx - hl * 0.82;
    let sill_y = gy - 8.5 * s;
    let roof_y = cy - 18.0 * s;

    // Rounded classic Beetle bubble roof and exposed rear flat-4 stinger exhaust
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_circle(cx - hl * 0.05, roof_y + 4.0 * s, 16.0 * s, primary);

    // Exposed chrome rear stinger exhaust pipe
    draw_line(tail_x, sill_y, tail_x - 12.0 * s, sill_y - 12.0 * s, 2.5 * s, Color::new(0.85, 0.88, 0.92, 1.0));

    // Vintage round front headlights
    draw_circle(nose_x - 4.0 * s, sill_y - 4.0 * s, 3.5 * s, Palette::NEON_GOLD);

    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);
    draw_rectangle(cx - hl * 0.20, sill_y - 2.0 * s, 24.0 * s, 3.0 * s, secondary);
}

fn render_offroad_subaru_ice(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.92;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 7.0 * s;
    let belt_y = cy + 1.2 * s;
    let roof_y = cy - 18.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.46, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.11, sill_y - belt_y, primary);

    // Subaru iconic gold STI side graphic
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 20.0 * s, 3.0 * s, secondary);
    // Roof air vent scoop
    draw_triangle(Vec2::new(cx - hl * 0.05, roof_y - 4.0 * s), Vec2::new(cx + hl * 0.08, roof_y - 1.0 * s), Vec2::new(cx - hl * 0.12, roof_y - 1.0 * s), primary);

    draw_circle(cx - hl * 0.05, cy - 6.5 * s, 4.2 * s, helmet);

    // Iconic tall STI rear wing
    let wing_x = tail_x + 4.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 1.0 * s, 2.2 * s, primary);
    draw_rectangle(wing_x - 8.0 * s, roof_y, 18.0 * s, 3.0 * s, secondary);
}

fn render_offroad_audi_ice(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.86;
    let sill_y = gy - 7.5 * s;
    let belt_y = cy + 1.5 * s;
    let _roof_y = cy - 18.5 * s;

    // Classic 1980s angular Audi quattro box flares on studded ice tires
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.44, belt_y), Vec2::new(cx - hl * 0.48, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.09, sill_y - belt_y, primary);

    // Front bumper yellow fog lamp pods
    draw_circle(nose_x - 3.0 * s, sill_y - 3.0 * s, 3.5 * s, Palette::NEON_GOLD);

    draw_rectangle(tail_x + 8.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 16.0 * s, 3.0 * s, secondary);
    draw_circle(cx - hl * 0.05, cy - 6.5 * s, 4.2 * s, helmet);
}

fn render_offroad_lancer_ice(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.92;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 7.0 * s;
    let belt_y = cy + 1.2 * s;
    let roof_y = cy - 18.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.46, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.11, sill_y - belt_y, primary);

    // Ralliart red/black splash graphics & roof vortex generator teeth
    draw_triangle(Vec2::new(tail_x + 10.0 * s, belt_y), Vec2::new(tail_x + 30.0 * s, sill_y), Vec2::new(tail_x + 10.0 * s, sill_y), Palette::RED);
    for i in 0..4 {
        draw_rectangle(cx - hl * 0.15 + (i as f32) * 4.0 * s, roof_y - 2.0 * s, 1.5 * s, 2.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    }

    draw_circle(cx - hl * 0.05, cy - 6.5 * s, 4.2 * s, helmet);

    let wing_x = tail_x + 4.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y + 1.0 * s, 2.2 * s, primary);
    draw_rectangle(wing_x - 8.0 * s, roof_y, 18.0 * s, 3.0 * s, secondary);
}

fn render_offroad_chevy_k30_bogger(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.92;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 16.0 * s;
    let belt_y = cy - 6.0 * s;
    let roof_y = cy - 26.0 * s;

    // 1970s Squarebody pickup on monster lift kit
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 8.0 * s, primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Dual vertical chrome exhaust stacks behind cab
    draw_line(cx - hl * 0.20, belt_y, cx - hl * 0.20, roof_y - 6.0 * s, 3.5 * s, Color::new(0.85, 0.88, 0.92, 1.0));

    draw_circle(cx - hl * 0.08, cy - 12.0 * s, 4.4 * s, helmet);
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 20.0 * s, 3.0 * s, secondary);
}

fn render_rally_sst_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 11.0 * s;
    let belt_y = cy - 1.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 7.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.45, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Long-travel suspension shocks visible in wheel arches
    draw_line(cx + hl * 0.50, sill_y, cx + hl * 0.50, gy - 15.0 * s, 2.5 * s, secondary);
    draw_line(cx - hl * 0.48, sill_y, cx - hl * 0.48, gy - 15.0 * s, 2.5 * s, secondary);
    draw_circle(cx - hl * 0.05, cy - 8.5 * s, 4.4 * s, helmet);
}

fn render_rally_sst_robby_gordon(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, _secondary: Color, helmet: Color) {
    render_rally_sst_truck(cx, cy, gy, hl, s, primary, Palette::NEON_GREEN, helmet);
    // Speed Energy neon green roof light pod
    let roof_y = cy - 21.0 * s;
    draw_rectangle(cx - hl * 0.05, roof_y - 4.0 * s, 14.0 * s, 3.0 * s, Palette::NEON_GREEN);
}

fn render_rally_sst_traxxas(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_rally_sst_truck(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Traxxas Slash splash graphic
    draw_triangle(Vec2::new(cx - hl * 0.20, cy - 2.0 * s), Vec2::new(cx + hl * 0.15, cy + 4.0 * s), Vec2::new(cx - hl * 0.40, cy + 4.0 * s), Palette::NEON_CYAN);
}

fn render_offroad_trophy_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, _secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 13.0 * s;
    let belt_y = cy - 2.0 * s;
    let roof_y = cy - 23.0 * s;

    // Full-size Baja 1000 Trophy Truck on 40" tires
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 8.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.45, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Front high-clearance skid plate
    draw_line(nose_x, sill_y + 4.0 * s, nose_x - 12.0 * s, gy - 2.0 * s, 3.0 * s, Color::new(0.85, 0.88, 0.92, 1.0));

    // Dual rear spare tires on bed rack
    draw_circle(cx - hl * 0.35, belt_y + 2.0 * s, 9.0 * s, Color::new(0.12, 0.14, 0.18, 1.0));
    draw_circle(cx - hl * 0.50, belt_y + 2.0 * s, 9.0 * s, Color::new(0.12, 0.14, 0.18, 1.0));

    // Roof LED lightbar
    draw_rectangle(cx - hl * 0.05, roof_y - 4.0 * s, 16.0 * s, 3.5 * s, Palette::NEON_GOLD);
    draw_circle(cx - hl * 0.05, cy - 10.0 * s, 4.4 * s, helmet);
}

fn render_offroad_bettantown_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_offroad_trophy_truck(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Quad roof pod lights
    let roof_y = cy - 23.0 * s;
    for i in 0..4 {
        draw_circle(cx - hl * 0.08 + (i as f32) * 5.0 * s, roof_y - 4.0 * s, 2.2 * s, Palette::NEON_GOLD);
    }
}

fn render_offroad_mason_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_offroad_trophy_truck(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Aggressive dual hood air extractors & Mason blue side graphic
    draw_rectangle(cx + hl * 0.20, cy - 4.0 * s, 12.0 * s, 3.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(cx - hl * 0.45, cy - 1.0 * s, hl * 0.70, 3.0 * s, secondary);
}

fn render_offroad_mud_truck(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 16.0 * s; // Super-tall lift kit!
    let belt_y = cy - 6.0 * s;
    let roof_y = cy - 26.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 8.0 * s, primary);
    draw_rectangle(cx - hl * 0.60, belt_y, hl * 1.00, sill_y - belt_y, primary);

    // Supercharged engine blower sticking up through hood
    draw_rectangle(cx + hl * 0.25, belt_y - 8.0 * s, 10.0 * s, 8.0 * s, Color::new(0.85, 0.88, 0.92, 1.0));
    draw_circle(cx + hl * 0.32, belt_y - 6.0 * s, 3.0 * s, Palette::RED);

    // Open roll cage
    draw_line(cx + hl * 0.20, belt_y, cx + hl * 0.05, roof_y, 2.5 * s, secondary);
    draw_line(cx + hl * 0.05, roof_y, cx - hl * 0.35, roof_y, 2.5 * s, secondary);
    draw_line(cx - hl * 0.35, roof_y, cx - hl * 0.50, belt_y, 2.5 * s, secondary);

    draw_circle(cx - hl * 0.08, cy - 12.0 * s, 4.4 * s, helmet);
}

fn render_offroad_monster_jam(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.88;
    let tail_x = cx - hl * 0.86;
    let sill_y = gy - 20.0 * s; // Gargantuan 66" tire height!
    let belt_y = cy - 10.0 * s;

    // 1950s Panel Van body on top of giant tubular chassis
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 9.0 * s, primary);
    draw_rectangle(tail_x, belt_y, nose_x - tail_x, sill_y - belt_y, primary);

    // Green flame graphics on black body
    draw_triangle(Vec2::new(cx + hl * 0.30, belt_y), Vec2::new(cx - hl * 0.20, sill_y), Vec2::new(cx - hl * 0.40, belt_y), secondary);
    draw_triangle(Vec2::new(cx + hl * 0.10, belt_y), Vec2::new(cx - hl * 0.40, sill_y), Vec2::new(cx - hl * 0.60, belt_y), secondary);

    // Heavy 4-link tubular spaceframe and nitrogen shock struts
    draw_line(cx + hl * 0.40, sill_y + 4.0 * s, cx + hl * 0.54, gy - 24.0 * s, 3.0 * s, secondary);
    draw_line(cx - hl * 0.35, sill_y + 4.0 * s, cx - hl * 0.50, gy - 24.0 * s, 3.0 * s, secondary);

    // Driver central cockpit
    draw_circle(cx, cy - 16.0 * s, 4.8 * s, helmet);
}

fn render_offroad_ford_f250_riser(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_offroad_mud_truck(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Heavy duty front bull bar & dual steering stabilizers
    let nose_x = cx + hl * 0.90;
    let sill_y = gy - 16.0 * s;
    draw_line(nose_x + 4.0 * s, sill_y + 8.0 * s, nose_x + 4.0 * s, sill_y - 6.0 * s, 3.5 * s, Color::new(0.85, 0.88, 0.92, 1.0));
}

fn render_offroad_max_d(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.88;
    let tail_x = cx - hl * 0.86;
    let sill_y = gy - 20.0 * s;
    let belt_y = cy - 10.0 * s;
    let roof_y = cy - 28.0 * s;

    // Maximum Destruction futuristic silver armored body with roof spikes
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 9.0 * s, primary);
    draw_rectangle(tail_x, belt_y, nose_x - tail_x, sill_y - belt_y, primary);

    // Armored roof spikes
    for i in 0..3 {
        draw_triangle(
            Vec2::new(cx - hl * 0.20 + (i as f32) * 8.0 * s, roof_y),
            Vec2::new(cx - hl * 0.16 + (i as f32) * 8.0 * s, roof_y - 6.0 * s),
            Vec2::new(cx - hl * 0.12 + (i as f32) * 8.0 * s, roof_y),
            Color::new(0.85, 0.88, 0.92, 1.0),
        );
    }

    draw_circle(cx, cy - 16.0 * s, 4.8 * s, helmet);
    draw_rectangle(cx - hl * 0.30, belt_y + 2.0 * s, hl * 0.60, 3.5 * s, secondary);
}

fn render_offroad_bigfoot(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.90;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 20.0 * s;
    let belt_y = cy - 10.0 * s;
    let roof_y = cy - 28.0 * s;

    // Bigfoot classic 1970s Ford F-250 pickup cab on 66" tires
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 9.0 * s, primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.10, sill_y - belt_y, primary);

    // Chrome double roll bar in bed with 5 KC daylighter lights
    draw_line(cx - hl * 0.25, belt_y, cx - hl * 0.25, roof_y - 4.0 * s, 3.5 * s, Color::new(0.85, 0.88, 0.92, 1.0));
    for i in 0..5 {
        draw_circle(cx - hl * 0.35 + (i as f32) * 4.0 * s, roof_y - 6.0 * s, 2.0 * s, Palette::NEON_GOLD);
    }

    draw_circle(cx - hl * 0.08, cy - 16.0 * s, 4.8 * s, helmet);
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 20.0 * s, 3.5 * s, secondary);
}

fn render_kart_cadet_60(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_kart(cx, cy, gy, hl, s, primary, secondary, helmet);
    // 60cc air-cooled cylinder head
    let sill_y = gy - 4.5 * s;
    draw_rectangle(cx - hl * 0.30, sill_y - 8.0 * s, 6.0 * s, 6.0 * s, Color::new(0.50, 0.52, 0.56, 1.0));
}

fn render_kart_birel_c28(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_kart(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Red/white Freeline bodywork & mini chain guard
    let sill_y = gy - 4.5 * s;
    draw_rectangle(cx - hl * 0.32, sill_y - 6.0 * s, 6.0 * s, 4.0 * s, Palette::RED);
}

fn render_kart_tony_neos(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_kart(cx, cy, gy, hl, s, primary, secondary, helmet);
    // OTK Green/White graphics & compact floor tray
    let sill_y = gy - 4.5 * s;
    draw_rectangle(cx - hl * 0.20, sill_y - 2.0 * s, hl * 0.40, 2.0 * s, Palette::NEON_GREEN);
}

fn render_kart_tony_ok(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_kart(cx, cy, gy, hl, s, primary, secondary, helmet);
}

fn render_kart_crg_kt2(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_kart(cx, cy, gy, hl, s, primary, secondary, helmet);
    // CRG Black/Orange aero side pods
    let sill_y = gy - 4.5 * s;
    draw_rectangle(cx - hl * 0.25, sill_y - 4.0 * s, hl * 0.50, 4.0 * s, Palette::NEON_ORANGE);
}

fn render_kart_birel_ry30(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_kart(cx, cy, gy, hl, s, primary, secondary, helmet);
    let sill_y = gy - 4.5 * s;
    draw_rectangle(cx - hl * 0.25, sill_y - 4.0 * s, hl * 0.50, 4.0 * s, Palette::RED);
}

fn render_kart_birel_kz2(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_lateral_kart(cx, cy, gy, hl, s, primary, secondary, helmet);
    // 6-speed sequential shifter lever and front handbrake
    let sill_y = gy - 4.5 * s;
    draw_line(cx + hl * 0.10, sill_y - 2.0 * s, cx + hl * 0.08, cy - 6.0 * s, 1.8 * s, Palette::WHITE);
    draw_circle(cx + hl * 0.08, cy - 6.0 * s, 2.0 * s, Palette::RED);
}

fn render_kart_crg_road_rebel(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_kart_birel_kz2(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Gold magnesium components & shifter setup
    let sill_y = gy - 4.5 * s;
    draw_circle(cx - hl * 0.35, sill_y - 4.0 * s, 3.0 * s, Palette::NEON_GOLD);
}

fn render_kart_tony_kz(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_kart_birel_kz2(cx, cy, gy, hl, s, primary, secondary, helmet);
    // OTK Green shifter styling
    let sill_y = gy - 4.5 * s;
    draw_rectangle(cx - hl * 0.20, sill_y - 3.0 * s, hl * 0.40, 3.0 * s, Palette::NEON_GREEN);
}

fn render_kart_mean_mower(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, _secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.85;
    let tail_x = cx - hl * 0.75;
    let sill_y = gy - 6.0 * s;
    let hood_y = cy - 8.0 * s;

    // Lawn tractor front hood box
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_rectangle(cx - hl * 0.10, hood_y, nose_x - (cx - hl * 0.10), sill_y - hood_y, primary);

    // Front headlights and grill
    draw_rectangle(nose_x - 3.0 * s, hood_y + 2.0 * s, 3.0 * s, 5.0 * s, Palette::WHITE);

    // High upright tractor seat
    draw_rectangle(cx - hl * 0.35, cy - 10.0 * s, 8.0 * s, 12.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_circle(cx - hl * 0.15, cy - 14.0 * s, 4.6 * s, helmet);

    // Side grass discharge chute
    draw_rectangle(cx - hl * 0.10, sill_y + 1.0 * s, 14.0 * s, 3.5 * s, Color::new(0.08, 0.08, 0.10, 1.0));
}

fn render_kart_john_deere_mower(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_kart_mean_mower(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Classic John Deere yellow racing stripes
    draw_rectangle(cx - hl * 0.05, cy - 4.0 * s, hl * 0.70, 2.5 * s, Palette::NEON_GOLD);
}

fn render_kart_viking_tractor(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_kart_mean_mower(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Viking metallic orange / grey styling & twin LED lights
    let nose_x = cx + hl * 0.85;
    draw_circle(nose_x - 4.0 * s, cy - 6.0 * s, 2.5 * s, Palette::NEON_CYAN);
}

fn render_kart_superkart_250(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.95;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 5.0 * s;
    let belt_y = cy + 3.0 * s;
    let roof_y = cy - 12.0 * s;

    // Full aerodynamic carbon body shell encapsulating legs
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y + 1.0 * s), Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.00, sill_y - belt_y, primary);

    // Cockpit windscreen bubble
    draw_triangle(Vec2::new(cx + hl * 0.25, belt_y), Vec2::new(cx + hl * 0.10, roof_y + 2.0 * s), Vec2::new(cx - hl * 0.15, roof_y + 2.0 * s), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.02, cy - 4.0 * s, 4.2 * s, helmet);

    // Multi-element aerodynamic rear wing
    let wing_x = tail_x + 5.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y - 2.0 * s, 20.0 * s, 2.8 * s, secondary);
}

fn render_kart_ms_superkart(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_kart_superkart_250(cx, cy, gy, hl, s, primary, secondary, helmet);
    // MS Kart Red/White livery accent
    draw_rectangle(cx - hl * 0.40, cy + 2.0 * s, hl * 0.60, 2.5 * s, Palette::RED);
}

fn render_kart_viper_superkart(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    render_kart_superkart_250(cx, cy, gy, hl, s, primary, secondary, helmet);
    // Viper Stealth carbon black & twin expansion exhausts
    let tail_x = cx - hl * 0.90;
    draw_line(tail_x, gy - 5.0 * s, tail_x - 8.0 * s, gy - 7.0 * s, 2.2 * s, Color::new(0.40, 0.42, 0.46, 1.0));
    draw_line(tail_x, gy - 7.0 * s, tail_x - 8.0 * s, gy - 9.0 * s, 2.2 * s, Color::new(0.40, 0.42, 0.46, 1.0));
}

#[inline]
fn tail_x_val(cx: f32, hl: f32) -> f32 {
    cx - hl * 0.84
}

// =========================================================================
// BASE ARCHETYPE LATERAL RENDERERS
// =========================================================================
fn render_lateral_stock_car(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.0 * s;
    let roof_y = cy - 18.5 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.48, belt_y), Vec2::new(cx - hl * 0.55, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.18, sill_y - belt_y, primary);
    draw_triangle(Vec2::new(cx - hl * 0.70, belt_y), Vec2::new(tail_x, sill_y + 1.0 * s), Vec2::new(cx - hl * 0.70, sill_y), primary);

    draw_rectangle(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, Palette::WHITE);
    draw_rectangle_lines(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, 1.2 * s, secondary);

    let a_pillar_x = cx + hl * 0.22;
    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(a_pillar_x, roof_y), Vec2::new(cx - hl * 0.20, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.20, roof_y, (a_pillar_x - (cx - hl * 0.20)).abs().max(10.0 * s), belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));

    for i in 0..4 {
        let lx = cx - hl * 0.10 + (i as f32) * 4.0 * s;
        draw_line(lx, roof_y + 2.0 * s, lx, belt_y - 1.0 * s, 1.0 * s, Color::new(0.08, 0.08, 0.10, 0.90));
    }
    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);
    draw_line(tail_x + 2.0 * s, belt_y, tail_x - 4.0 * s, belt_y - 8.0 * s, 2.5 * s, Color::new(0.10, 0.10, 0.12, 1.0));
}

fn render_lateral_rally(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.88;
    let tail_x = cx - hl * 0.84;
    let sill_y = gy - 7.5 * s;
    let belt_y = cy + 1.0 * s;
    let roof_y = cy - 20.0 * s;

    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(Vec2::new(nose_x, sill_y), Vec2::new(cx + hl * 0.45, belt_y), Vec2::new(cx - hl * 0.50, belt_y), primary);
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.15, sill_y - belt_y, primary);

    draw_rectangle(tail_x + 8.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 16.0 * s, 3.5 * s, secondary);

    draw_triangle(Vec2::new(cx + hl * 0.35, belt_y), Vec2::new(cx + hl * 0.18, roof_y), Vec2::new(cx - hl * 0.15, roof_y), Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.15, roof_y, 22.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.05, cy - 8.0 * s, 4.2 * s, helmet);

    // Roof Scoop
    draw_triangle(Vec2::new(cx - hl * 0.02, roof_y - 6.0 * s), Vec2::new(cx + hl * 0.12, roof_y - 2.0 * s), Vec2::new(cx - hl * 0.12, roof_y - 2.0 * s), secondary);

    // Rear Spoiler
    draw_line(tail_x, roof_y, tail_x - 6.0 * s, roof_y - 6.0 * s, 2.5 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(tail_x - 12.0 * s, roof_y - 8.0 * s, 16.0 * s, 3.0 * s, secondary);
}

fn render_lateral_sand_rail(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, _secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.86;
    let tail_x = cx - hl * 0.82;
    let sill_y = gy - 9.0 * s;
    let cage_roof_y = cy - 20.0 * s;

    draw_line(nose_x, sill_y, cx - hl * 0.40, sill_y, 2.5 * s, primary);
    draw_line(cx + hl * 0.35, sill_y, cx + hl * 0.10, cage_roof_y, 2.5 * s, primary);
    draw_line(cx + hl * 0.10, cage_roof_y, cx - hl * 0.35, cage_roof_y, 2.5 * s, primary);
    draw_line(cx - hl * 0.35, cage_roof_y, cx - hl * 0.50, sill_y, 2.5 * s, primary);
    draw_line(cx - hl * 0.35, cage_roof_y, tail_x, sill_y, 2.0 * s, primary);

    draw_circle(cx - hl * 0.05, cy - 8.0 * s, 4.4 * s, helmet);
    draw_rectangle(cx - hl * 0.05, cage_roof_y - 3.5 * s, 14.0 * s, 3.0 * s, Palette::NEON_GOLD);

    draw_line(tail_x + 2.0 * s, sill_y - 8.0 * s, tail_x - 12.0 * s, cage_roof_y - 12.0 * s, 1.0 * s, Palette::WHITE);
    draw_triangle(Vec2::new(tail_x - 12.0 * s, cage_roof_y - 12.0 * s), Vec2::new(tail_x - 6.0 * s, cage_roof_y - 8.0 * s), Vec2::new(tail_x - 12.0 * s, cage_roof_y - 4.0 * s), Palette::NEON_ORANGE);
}

fn render_lateral_kart(cx: f32, cy: f32, gy: f32, hl: f32, s: f32, primary: Color, secondary: Color, helmet: Color) {
    let nose_x = cx + hl * 0.75;
    let tail_x = cx - hl * 0.75;
    let sill_y = gy - 4.5 * s;

    draw_line(tail_x, sill_y, nose_x, sill_y, 3.0 * s, Color::new(0.25, 0.28, 0.32, 1.0));
    draw_triangle(Vec2::new(nose_x + 5.0 * s, sill_y + 1.0 * s), Vec2::new(nose_x - 12.0 * s, sill_y - 8.0 * s), Vec2::new(nose_x - 12.0 * s, sill_y + 1.0 * s), primary);
    draw_rectangle(nose_x - 18.0 * s, sill_y - 12.0 * s, 5.0 * s, 12.0 * s, secondary);
    draw_rectangle(cx - hl * 0.28, sill_y - 5.0 * s, hl * 0.56, 6.0 * s, primary);

    draw_line(cx + hl * 0.18, sill_y, cx + hl * 0.05, cy - 8.0 * s, 2.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_circle(cx + hl * 0.05, cy - 8.0 * s, 3.5 * s, Color::new(0.15, 0.16, 0.18, 1.0));

    draw_rectangle(cx - hl * 0.25, cy - 4.0 * s, 8.0 * s, 12.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_triangle(Vec2::new(cx - hl * 0.18, sill_y), Vec2::new(cx - hl * 0.05, cy - 10.0 * s), Vec2::new(cx + hl * 0.08, sill_y), secondary);
    draw_circle(cx - hl * 0.04, cy - 16.0 * s, 5.5 * s, helmet);
}
