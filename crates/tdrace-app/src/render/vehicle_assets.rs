use std::collections::HashMap;
use std::sync::Mutex;
use macroquad::color::Color;
use macroquad::texture::{Image, Texture2D};

static PORSCHE_LATERAL_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r.png");
static PORSCHE_LATERAL_THUMB_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r_thumb.png");
static PORSCHE_TOPDOWN_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/gt/gt_porsche_911_gt3r.png");
static KART_SLICK_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/kart_slick_front.png");

static LATERAL_CACHE: Mutex<Option<HashMap<(String, u32, u32, bool), Texture2D>>> = Mutex::new(None);
static TOPDOWN_CACHE: Mutex<Option<HashMap<(String, u32, u32), Texture2D>>> = Mutex::new(None);
static WHEEL_TEXTURE_CACHE: Mutex<Option<HashMap<String, Texture2D>>> = Mutex::new(None);

#[inline]
pub fn color_to_u32(c: Color) -> u32 {
    let r = (c.r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (c.g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (c.b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn color_approx_eq(c1: Color, c2: Color) -> bool {
    (c1.r - c2.r).abs() < 0.03 && (c1.g - c2.g).abs() < 0.03 && (c1.b - c2.b).abs() < 0.03
}

fn find_asset_file(rel_path: &str) -> Option<Vec<u8>> {
    let candidates = [
        format!("assets/{}", rel_path),
        format!("../assets/{}", rel_path),
        format!("../../assets/{}", rel_path),
        format!("../../../assets/{}", rel_path),
    ];
    for c in &candidates {
        if let Ok(data) = std::fs::read(c) {
            return Some(data);
        }
    }
    None
}

/// Dynamic tinting pass converting neutral or archetype bodywork to `primary` and accent stripes to `secondary`.
pub fn apply_vehicle_tint(base_img: &Image, model_id: &str, primary: Color, secondary: Color) -> Image {
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

        let (is_body, is_accent) = match model_id {
            "classic_gt" => {
                // Red bodywork: all pixels where red is dominant, excluding neutral shadows and glass
                let body = r > g * 1.08 && r > b * 1.08 && r > 0.12 && sat > 0.10;
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            "classic_nascar" => {
                // Blue bodywork: all pixels where blue is dominant
                let body = b > r * 1.08 && b > g * 1.05 && b > 0.12 && sat > 0.10;
                let accent = r > 0.45 && g > 0.35 && b < 0.35 && sat > 0.20;
                (body, accent)
            }
            "classic_offroad" => {
                // Orange body & tubular frame: red is high, green moderate, blue low
                let body = r > 0.18 && r > b * 1.15 && r > g * 1.05 && (g > 0.08 || sat > 0.35);
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            "classic_kart" => {
                // Green bodywork, pods and front fairing: green is dominant
                let body = g > r * 1.08 && g > b * 1.08 && g > 0.12 && sat > 0.10;
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            "classic_rally" => {
                // Yellow bodywork: red and green both high, blue low
                let body = r > 0.18 && g > 0.16 && (r + g) > b * 1.8 && sat > 0.15;
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            _ => {
                let (cat_body, cat_accent) = if let Some(m) = crate::catalog::find_model_by_id(model_id) {
                    let dr_p = (r - m.primary_color.r).abs();
                    let dg_p = (g - m.primary_color.g).abs();
                    let db_p = (b - m.primary_color.b).abs();
                    let dist_p = (dr_p * dr_p + dg_p * dg_p + db_p * db_p).sqrt();
                    let is_p = dist_p < 0.28 && sat > 0.10;

                    let dr_s = (r - m.secondary_color.r).abs();
                    let dg_s = (g - m.secondary_color.g).abs();
                    let db_s = (b - m.secondary_color.b).abs();
                    let dist_s = (dr_s * dr_s + dg_s * dg_s + db_s * db_s).sqrt();
                    let is_s = dist_s < 0.28 && sat > 0.10;

                    (is_p, is_s)
                } else {
                    (false, false)
                };

                let body = (lum > 0.65 && sat < 0.22) || cat_body;
                let accent = (!body && (g > 0.58 && r > 0.45 && b < 0.45 && sat > 0.30)) || cat_accent;
                (body, accent)
            }
        };

        if is_body {
            let val = (max_c * 1.35).clamp(0.0, 1.0);
            pixel[0] = ((primary.r * val).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = ((primary.g * val).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = ((primary.b * val).clamp(0.0, 1.0) * 255.0) as u8;
        } else if is_accent {
            let val = (lum * 1.25).clamp(0.0, 1.0);
            pixel[0] = ((secondary.r * val).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = ((secondary.g * val).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = ((secondary.b * val).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    tinted
}

/// Retrieves or dynamically generates a colorway-tinted lateral texture for the specified car model.
/// When `high_res` is true, loads 1024px full texture for garage turntable.
/// When `high_res` is false, loads 256px thumbnail for menus and starting grid cards.
pub fn get_vehicle_lateral_texture(
    model_id: &str,
    primary: Color,
    secondary: Color,
    high_res: bool,
) -> Option<Texture2D> {
    let k1 = color_to_u32(primary);
    let k2 = color_to_u32(secondary);
    let key = (model_id.to_string(), k1, k2, high_res);

    let mut guard = LATERAL_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(tex) = map.get(&key) {
        return Some(tex.clone());
    }

    let model = crate::catalog::find_model_by_id(model_id);
    let module_id = model.map(|m| m.module_id).unwrap_or("gt");
    let file_suffix = if high_res { "" } else { "_thumb" };
    let rel_path = format!("textures/vehicles/laterals/{}/{}{}.png", module_id, model_id, file_suffix);

    let bytes = if let Some(disk_bytes) = find_asset_file(&rel_path) {
        disk_bytes
    } else if model_id == "gt_porsche_911_gt3r" {
        if high_res {
            PORSCHE_LATERAL_PNG.to_vec()
        } else {
            PORSCHE_LATERAL_THUMB_PNG.to_vec()
        }
    } else {
        return None;
    };

    let base_img = Image::from_file_with_format(&bytes, None).ok()?;
    let is_factory = model.map(|m| {
        color_approx_eq(m.primary_color, primary) && color_approx_eq(m.secondary_color, secondary)
    }).unwrap_or(false);

    let final_img = if is_factory {
        base_img
    } else {
        apply_vehicle_tint(&base_img, model_id, primary, secondary)
    };

    let texture = Texture2D::from_image(&final_img);
    map.insert(key, texture.clone());
    Some(texture)
}

/// Retrieves or dynamically generates a colorway-tinted top-down in-race texture for the specified car model.
pub fn get_vehicle_topdown_texture(
    model_id: &str,
    primary: Color,
    secondary: Color,
) -> Option<Texture2D> {
    let k1 = color_to_u32(primary);
    let k2 = color_to_u32(secondary);
    let key = (model_id.to_string(), k1, k2);

    let mut guard = TOPDOWN_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(tex) = map.get(&key) {
        return Some(tex.clone());
    }

    let model = crate::catalog::find_model_by_id(model_id);
    let module_id = model.map(|m| m.module_id).unwrap_or("gt");
    let rel_path = format!("textures/vehicles/topdown/{}/{}.png", module_id, model_id);

    let bytes = if let Some(disk_bytes) = find_asset_file(&rel_path) {
        disk_bytes
    } else if model_id == "gt_porsche_911_gt3r" {
        PORSCHE_TOPDOWN_PNG.to_vec()
    } else {
        return None;
    };

    let base_img = Image::from_file_with_format(&bytes, None).ok()?;
    let is_factory = model.map(|m| {
        color_approx_eq(m.primary_color, primary) && color_approx_eq(m.secondary_color, secondary)
    }).unwrap_or(false);

    let final_img = if is_factory {
        base_img
    } else {
        apply_vehicle_tint(&base_img, model_id, primary, secondary)
    };

    let texture = Texture2D::from_image(&final_img);
    map.insert(key, texture.clone());
    Some(texture)
}

/// Configuration for vehicles utilizing modular steered wheel rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SteeredWheelConfig {
    /// Relative path identifier or key for the wheel texture (e.g. "kart_slick_front").
    pub wheel_texture_id: &'static str,
    /// Distance from vehicle CG to front wheel axle (meters).
    pub front_axle_offset: f32,
    /// Half of the front track width (meters).
    pub half_track_width: f32,
    /// Rendered dimensions of the individual wheel [width (thickness), height (diameter)] (meters).
    pub wheel_size: glam::Vec2,
    /// Z-layering mode relative to the chassis body.
    pub layering: WheelLayerMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelLayerMode {
    /// Wheels render beneath the chassis body (ideal for closed GT/NASCAR/Rally).
    UnderChassis,
    /// Wheels render above or alongside the chassis body (ideal for open karts/buggies).
    OverChassis,
}

/// Returns the steered wheel configuration for a model, if modular wheel animation is enabled.
pub fn get_steered_wheel_config(model_id: &str) -> Option<SteeredWheelConfig> {
    match model_id {
        "classic_kart" => Some(SteeredWheelConfig {
            wheel_texture_id: "kart_slick_front",
            front_axle_offset: 0.62,
            half_track_width: 0.52,
            wheel_size: glam::Vec2::new(0.24, 0.44),
            layering: WheelLayerMode::OverChassis,
        }),
        _ => None, // 84 legacy vehicles continue using monolithic sprite rendering
    }
}

/// Retrieves or loads a standalone high-resolution top-down wheel texture.
pub fn get_wheel_texture(wheel_id: &str) -> Option<Texture2D> {
    let mut guard = WHEEL_TEXTURE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(tex) = map.get(wheel_id) {
        return Some(tex.clone());
    }

    let rel_path = format!("textures/vehicles/topdown/wheels/{}.png", wheel_id);
    let bytes = if let Some(disk_bytes) = find_asset_file(&rel_path) {
        disk_bytes
    } else if wheel_id == "kart_slick_front" {
        KART_SLICK_FRONT_PNG.to_vec()
    } else {
        return None;
    };

    let base_img = Image::from_file_with_format(&bytes, None).ok()?;
    let texture = Texture2D::from_image(&base_img);
    map.insert(wheel_id.to_string(), texture.clone());
    Some(texture)
}

