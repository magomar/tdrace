use std::collections::HashMap;
use std::sync::Mutex;
use macroquad::color::Color;
use macroquad::texture::{Image, Texture2D};

static PORSCHE_LATERAL_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r.png");
static PORSCHE_LATERAL_THUMB_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_porsche_911_gt3r_thumb.png");
static PORSCHE_TOPDOWN_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/gt/gt_porsche_911_gt3r.png");

static LATERAL_CACHE: Mutex<Option<HashMap<(String, u32, u32, bool), Texture2D>>> = Mutex::new(None);
static TOPDOWN_CACHE: Mutex<Option<HashMap<(String, u32, u32), Texture2D>>> = Mutex::new(None);

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
                // Neutral / white / light-grey bodywork -> primary color
                let body = lum > 0.65 && sat < 0.22;
                // Accent livery graphics -> secondary color
                let accent = g > 0.58 && r > 0.45 && b < 0.45 && sat > 0.30;
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

