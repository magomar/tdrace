use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use macroquad::color::Color;
use macroquad::texture::{Image, Texture2D};
use tdrace_core::surface::CompoundId;

/// Extension trait adding macroquad color support to `CompoundId` (Spec 074).
pub trait CompoundColorExt {
    /// FIA-standardized color coding for compound identification.
    fn accent_color(self) -> Color;
}

impl CompoundColorExt for CompoundId {
    fn accent_color(self) -> Color {
        let [r, g, b, a] = self.accent_rgba();
        Color::new(r, g, b, a)
    }
}

static PORSCHE_LATERAL_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_vandorn_arrowhead_t2.png");
static PORSCHE_LATERAL_THUMB_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/laterals/gt/gt_vandorn_arrowhead_t2_thumb.png");
static PORSCHE_TOPDOWN_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/gt/gt_vandorn_arrowhead_t2.png");
static KART_SLICK_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/kart_slick_front.png");
static GT_SLICK_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/gt_slick_front.png");
static NASCAR_WHEEL_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/nascar_wheel_front.png");
static OFFROAD_WHEEL_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/offroad_wheel_front.png");
static RALLY_WHEEL_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/rally_wheel_front.png");
static BUGGY_ALLTERRAIN_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/buggy_allterrain_front.png");
static TRUCK_ALLTERRAIN_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/truck_allterrain_front.png");
static MONSTER_WHEEL_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/monster_wheel_front.png");
static MUD_TRACTOR_FRONT_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/wheels/mud_tractor_front.png");

static VISUAL_WHEEL_ANCHORS_JSON: &str = include_str!("../../../../artifacts/visual_wheel_anchors.json");

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
            "classic_gt_vintage" => {
                // British Racing Green bodywork
                let body = g > r * 1.08 && g > b * 1.08 && g > 0.12 && sat > 0.10;
                let accent = lum > 0.70 && sat < 0.25;
                (body, accent)
            }
            "classic_nascar" => {
                // Blue bodywork: all pixels where blue is dominant
                let body = b > r * 1.08 && b > g * 1.05 && b > 0.12 && sat > 0.10;
                let accent = r > 0.45 && g > 0.35 && b < 0.35 && sat > 0.20;
                (body, accent)
            }
            "classic_stock_vintage" => {
                // Maroon/Red muscle bodywork
                let body = r > g * 1.15 && r > b * 1.15 && r > 0.12 && sat > 0.12;
                let accent = lum > 0.70 && sat < 0.22;
                (body, accent)
            }
            "classic_offroad" => {
                // Orange body & tubular frame: red is high, green moderate, blue low
                let body = r > 0.18 && r > b * 1.15 && r > g * 1.05 && (g > 0.08 || sat > 0.35);
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            "classic_at_safari" => {
                // Tan / Khaki 4x4 bodywork
                let body = r > 0.18 && g > 0.14 && b > 0.08 && r > b * 1.15 && sat > 0.10 && sat < 0.55;
                let accent = lum > 0.65 && sat < 0.20;
                (body, accent)
            }
            "classic_kart" => {
                // Green bodywork, pods and front fairing: green is dominant
                let body = g > r * 1.08 && g > b * 1.08 && g > 0.12 && sat > 0.10;
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            "classic_kart_vintage" => {
                // Yellow nosecone and seat
                let body = r > 0.18 && g > 0.14 && (r + g) > b * 1.7 && sat > 0.15;
                let accent = lum > 0.75 && sat < 0.20;
                (body, accent)
            }
            "classic_rally" => {
                // Yellow bodywork: red and green both high, blue low
                let body = r > 0.18 && g > 0.16 && (r + g) > b * 1.8 && sat > 0.15;
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            "classic_rx_vintage" => {
                // White bodywork with blue rally stripes
                let body = lum > 0.75 && sat < 0.18;
                let accent = b > r * 1.10 && b > g * 1.05 && b > 0.15 && sat > 0.15;
                (body, accent)
            }
            "classic_ax_mudlark" => {
                // Cyan / Sky Blue bodywork & nosecone
                let body = b > 0.20 && g > 0.18 && b > r * 1.15 && sat > 0.15;
                let accent = lum > 0.65 && sat < 0.22;
                (body, accent)
            }
            "classic_ax_brawler" => {
                // Crimson Red bodywork
                let body = r > g * 1.15 && r > b * 1.15 && r > 0.12 && sat > 0.12;
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
    } else if model_id == "gt_vandorn_arrowhead_t2" {
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

/// Retrieves or dynamically generates a colorway-tinted top-down in-race or showroom texture for the specified car model.
/// Loads the canonical full-vehicle sprite with wheels for showroom and garage display.
pub fn get_vehicle_topdown_texture(
    model_id: &str,
    primary: Color,
    secondary: Color,
) -> Option<Texture2D> {
    get_vehicle_topdown_texture_impl(model_id, primary, secondary, false)
}

/// Retrieves or dynamically generates a colorway-tinted top-down chassis texture for in-game race rendering.
/// If an isolated chassis texture (`<model_id>_chassis.png`) exists (with wheels removed for modular wheel animation),
/// it is loaded. Otherwise, it gracefully falls back to the canonical `<model_id>.png`.
pub fn get_vehicle_topdown_chassis_texture(
    model_id: &str,
    primary: Color,
    secondary: Color,
) -> Option<Texture2D> {
    get_vehicle_topdown_texture_impl(model_id, primary, secondary, true)
}

fn get_vehicle_topdown_texture_impl(
    model_id: &str,
    primary: Color,
    secondary: Color,
    chassis_only: bool,
) -> Option<Texture2D> {
    let k1 = color_to_u32(primary);
    let k2 = color_to_u32(secondary);
    let base_model_id = model_id.strip_suffix("_chassis").unwrap_or(model_id);
    let is_chassis = chassis_only || model_id.ends_with("_chassis");
    let key_name = if is_chassis {
        format!("{}_chassis", base_model_id)
    } else {
        base_model_id.to_string()
    };
    let key = (key_name, k1, k2);

    let mut guard = TOPDOWN_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(tex) = map.get(&key) {
        return Some(tex.clone());
    }

    let model = crate::catalog::find_model_by_id(base_model_id);
    let module_id = model.map(|m| m.module_id).unwrap_or("gt");

    let bytes = if is_chassis {
        let chassis_rel_path = format!("textures/vehicles/topdown/{}/{}_chassis.png", module_id, base_model_id);
        find_asset_file(&chassis_rel_path).or_else(|| {
            let rel_path = format!("textures/vehicles/topdown/{}/{}.png", module_id, base_model_id);
            find_asset_file(&rel_path)
        })
    } else {
        let rel_path = format!("textures/vehicles/topdown/{}/{}.png", module_id, base_model_id);
        find_asset_file(&rel_path)
    };

    let bytes = if let Some(disk_bytes) = bytes {
        disk_bytes
    } else if base_model_id == "gt_vandorn_arrowhead_t2" {
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
        apply_vehicle_tint(&base_img, base_model_id, primary, secondary)
    };

    let texture = Texture2D::from_image(&final_img);
    map.insert(key, texture.clone());
    Some(texture)
}

/// Pre-baked computer-vision visual wheel anchor extracted from vehicle top-down & lateral artwork (Spec 095).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VisualWheelAnchor {
    pub model_id: String,
    pub module: String,
    pub base_car: String,
    pub layering: String,
    pub archetype: String,
    pub axle_x_px: f32,
    pub track_width_px: f32,
    pub tire_len_px: f32,
    pub tire_wid_px: f32,
}

static VISUAL_WHEEL_ANCHORS: OnceLock<HashMap<String, VisualWheelAnchor>> = OnceLock::new();

/// Returns all pre-baked visual wheel anchors keyed by model_id.
pub fn get_visual_wheel_anchors() -> &'static HashMap<String, VisualWheelAnchor> {
    VISUAL_WHEEL_ANCHORS.get_or_init(|| {
        serde_json::from_str(VISUAL_WHEEL_ANCHORS_JSON)
            .expect("Failed to deserialize visual_wheel_anchors.json")
    })
}

/// Retrieves the visual wheel anchor for a given model_id, if present in calibration data.
pub fn get_visual_wheel_anchor(model_id: &str) -> Option<&'static VisualWheelAnchor> {
    get_visual_wheel_anchors().get(model_id)
}

/// Returns texture padding compensation expansion factor [k_w, k_h] for a wheel texture archetype.
pub fn archetype_padding_factor(archetype: &str) -> glam::Vec2 {
    match archetype {
        "monster_wheel_front" => glam::Vec2::new(128.0 / 122.0, 256.0 / 246.0),
        "mud_tractor_front" => glam::Vec2::new(128.0 / 118.0, 256.0 / 244.0),
        "truck_allterrain_front" | "nascar_wheel_front" => glam::Vec2::new(128.0 / 116.0, 256.0 / 242.0),
        "gt_slick_front" => glam::Vec2::new(128.0 / 114.0, 256.0 / 238.0),
        "buggy_allterrain_front" => glam::Vec2::new(128.0 / 110.0, 256.0 / 238.0),
        "kart_slick_front" | "rally_wheel_front" => glam::Vec2::new(128.0 / 110.0, 256.0 / 234.0),
        "offroad_wheel_front" => glam::Vec2::new(128.0 / 90.0, 256.0 / 236.0),
        _ => glam::Vec2::new(1.0, 1.0),
    }
}

/// Normalizes an archetype name string to a static string slice identifier.
pub fn archetype_to_static_id(archetype: &str) -> &'static str {
    match archetype {
        "kart_slick_front" => "kart_slick_front",
        "gt_slick_front" => "gt_slick_front",
        "nascar_wheel_front" => "nascar_wheel_front",
        "rally_wheel_front" => "rally_wheel_front",
        "buggy_allterrain_front" => "buggy_allterrain_front",
        "truck_allterrain_front" => "truck_allterrain_front",
        "monster_wheel_front" => "monster_wheel_front",
        "mud_tractor_front" => "mud_tractor_front",
        _ => "offroad_wheel_front",
    }
}

/// Configuration for vehicles utilizing modular steered wheel rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SteeredWheelConfig {
    /// Relative path identifier or key for the wheel texture (e.g. "kart_slick_front").
    pub wheel_texture_id: &'static str,
    /// Distance from vehicle visual center to front wheel axle (meters).
    pub front_axle_offset: f32,
    /// Half of the front track width (meters).
    pub half_track_width: f32,
    /// Rendered dimensions of the individual wheel [width (thickness), height (diameter)] (meters).
    pub wheel_size: glam::Vec2,
    /// Texture padding compensation factor [k_w, k_h].
    pub texture_padding_factor: glam::Vec2,
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
        // Karting
        "classic_kart" => Some(SteeredWheelConfig {
            wheel_texture_id: "kart_slick_front",
            front_axle_offset: 0.41,
            half_track_width: 0.39,
            wheel_size: glam::Vec2::new(0.20, 0.28),
            texture_padding_factor: archetype_padding_factor("kart_slick_front"),
            layering: WheelLayerMode::OverChassis,
        }),
        "classic_kart_vintage" => Some(SteeredWheelConfig {
            wheel_texture_id: "kart_slick_front",
            front_axle_offset: 0.39,
            half_track_width: 0.36,
            wheel_size: glam::Vec2::new(0.18, 0.26),
            texture_padding_factor: archetype_padding_factor("kart_slick_front"),
            layering: WheelLayerMode::OverChassis,
        }),

        // GT / Road Racing
        "classic_gt" => Some(SteeredWheelConfig {
            wheel_texture_id: "gt_slick_front",
            front_axle_offset: 0.75,
            half_track_width: 0.48,
            wheel_size: glam::Vec2::new(0.24, 0.48),
            texture_padding_factor: archetype_padding_factor("gt_slick_front"),
            layering: WheelLayerMode::UnderChassis,
        }),
        "classic_gt_vintage" => Some(SteeredWheelConfig {
            wheel_texture_id: "gt_slick_front",
            front_axle_offset: 0.71,
            half_track_width: 0.44,
            wheel_size: glam::Vec2::new(0.22, 0.46),
            texture_padding_factor: archetype_padding_factor("gt_slick_front"),
            layering: WheelLayerMode::UnderChassis,
        }),

        // Stock Cars
        "classic_nascar" => Some(SteeredWheelConfig {
            wheel_texture_id: "nascar_wheel_front",
            front_axle_offset: 0.66,
            half_track_width: 0.51,
            wheel_size: glam::Vec2::new(0.26, 0.50),
            texture_padding_factor: archetype_padding_factor("nascar_wheel_front"),
            layering: WheelLayerMode::UnderChassis,
        }),
        "classic_stock_vintage" => Some(SteeredWheelConfig {
            wheel_texture_id: "nascar_wheel_front",
            front_axle_offset: 0.72,
            half_track_width: 0.49,
            wheel_size: glam::Vec2::new(0.26, 0.50),
            texture_padding_factor: archetype_padding_factor("nascar_wheel_front"),
            layering: WheelLayerMode::UnderChassis,
        }),

        // Rallycross
        "classic_rally" => Some(SteeredWheelConfig {
            wheel_texture_id: "rally_wheel_front",
            front_axle_offset: 0.73,
            half_track_width: 0.41,
            wheel_size: glam::Vec2::new(0.24, 0.46),
            texture_padding_factor: archetype_padding_factor("rally_wheel_front"),
            layering: WheelLayerMode::UnderChassis,
        }),
        "classic_rx_vintage" => Some(SteeredWheelConfig {
            wheel_texture_id: "rally_wheel_front",
            front_axle_offset: 0.69,
            half_track_width: 0.40,
            wheel_size: glam::Vec2::new(0.22, 0.44),
            texture_padding_factor: archetype_padding_factor("rally_wheel_front"),
            layering: WheelLayerMode::UnderChassis,
        }),

        // Autocross
        "classic_ax_mudlark" => Some(SteeredWheelConfig {
            wheel_texture_id: "offroad_wheel_front",
            front_axle_offset: 0.85,
            half_track_width: 0.55,
            wheel_size: glam::Vec2::new(0.24, 0.48),
            texture_padding_factor: archetype_padding_factor("offroad_wheel_front"),
            layering: WheelLayerMode::OverChassis,
        }),
        "classic_ax_brawler" => Some(SteeredWheelConfig {
            wheel_texture_id: "rally_wheel_front",
            front_axle_offset: 0.75,
            half_track_width: 0.44,
            wheel_size: glam::Vec2::new(0.25, 0.48),
            texture_padding_factor: archetype_padding_factor("rally_wheel_front"),
            layering: WheelLayerMode::UnderChassis,
        }),

        // All-Terrain
        "classic_offroad" => Some(SteeredWheelConfig {
            wheel_texture_id: "offroad_wheel_front",
            front_axle_offset: 0.98,
            half_track_width: 0.62,
            wheel_size: glam::Vec2::new(0.28, 0.58),
            texture_padding_factor: archetype_padding_factor("offroad_wheel_front"),
            layering: WheelLayerMode::OverChassis,
        }),
        "classic_at_safari" => Some(SteeredWheelConfig {
            wheel_texture_id: "offroad_wheel_front",
            front_axle_offset: 0.82,
            half_track_width: 0.48,
            wheel_size: glam::Vec2::new(0.26, 0.54),
            texture_padding_factor: archetype_padding_factor("offroad_wheel_front"),
            layering: WheelLayerMode::UnderChassis,
        }),

        // Legacy / Fallback
        "classic_ax_talon" => Some(SteeredWheelConfig {
            wheel_texture_id: "offroad_wheel_front",
            front_axle_offset: 1.05,
            half_track_width: 0.65,
            wheel_size: glam::Vec2::new(0.28, 0.58),
            texture_padding_factor: archetype_padding_factor("offroad_wheel_front"),
            layering: WheelLayerMode::OverChassis,
        }),
        _ => None, // Non-classic vehicles continue using monolithic sprite rendering
    }
}

/// Returns the wheel layering mode (UnderChassis vs OverChassis) for a base platform.
pub fn platform_wheel_layer_mode(platform: crate::ui::menu::CarChoice) -> WheelLayerMode {
    match platform {
        crate::ui::menu::CarChoice::CrossCar
        | crate::ui::menu::CarChoice::SuperBuggy
        | crate::ui::menu::CarChoice::DuneBuggyBaja
        | crate::ui::menu::CarChoice::MonsterTruck
        | crate::ui::menu::CarChoice::Kart
        | crate::ui::menu::CarChoice::SuperkartGP
        | crate::ui::menu::CarChoice::SandRail => WheelLayerMode::OverChassis,

        crate::ui::menu::CarChoice::GT4Clubsport
        | crate::ui::menu::CarChoice::GT3Car
        | crate::ui::menu::CarChoice::GT2Biturbo
        | crate::ui::menu::CarChoice::GT1Legend
        | crate::ui::menu::CarChoice::HypercarPrototype
        | crate::ui::menu::CarChoice::TouringAX
        | crate::ui::menu::CarChoice::TrophyTruckAWD
        | crate::ui::menu::CarChoice::MudBoggerHeavy
        | crate::ui::menu::CarChoice::RallyJuniorFWD
        | crate::ui::menu::CarChoice::RallyCar
        | crate::ui::menu::CarChoice::RallyGroupB
        | crate::ui::menu::CarChoice::RallyElectricRX
        | crate::ui::menu::CarChoice::StockCar
        | crate::ui::menu::CarChoice::StockCarTruck
        | crate::ui::menu::CarChoice::SportsCar
        | crate::ui::menu::CarChoice::DriftCar => WheelLayerMode::UnderChassis,
    }
}

/// Returns the top-down wheel texture identifier for a base platform.
pub fn platform_wheel_texture_id(platform: crate::ui::menu::CarChoice) -> &'static str {
    match platform {
        crate::ui::menu::CarChoice::Kart | crate::ui::menu::CarChoice::SuperkartGP => {
            "kart_slick_front"
        }
        crate::ui::menu::CarChoice::GT4Clubsport
        | crate::ui::menu::CarChoice::GT3Car
        | crate::ui::menu::CarChoice::GT2Biturbo
        | crate::ui::menu::CarChoice::GT1Legend
        | crate::ui::menu::CarChoice::HypercarPrototype
        | crate::ui::menu::CarChoice::SportsCar
        | crate::ui::menu::CarChoice::DriftCar => "gt_slick_front",
        crate::ui::menu::CarChoice::StockCar | crate::ui::menu::CarChoice::StockCarTruck => {
            "nascar_wheel_front"
        }
        crate::ui::menu::CarChoice::RallyJuniorFWD
        | crate::ui::menu::CarChoice::RallyCar
        | crate::ui::menu::CarChoice::RallyGroupB
        | crate::ui::menu::CarChoice::RallyElectricRX
        | crate::ui::menu::CarChoice::TouringAX => "rally_wheel_front",
        crate::ui::menu::CarChoice::SandRail
        | crate::ui::menu::CarChoice::CrossCar
        | crate::ui::menu::CarChoice::SuperBuggy
        | crate::ui::menu::CarChoice::DuneBuggyBaja => "buggy_allterrain_front",
        crate::ui::menu::CarChoice::TrophyTruckAWD => "truck_allterrain_front",
        crate::ui::menu::CarChoice::MonsterTruck => "monster_wheel_front",
        crate::ui::menu::CarChoice::MudBoggerHeavy => "mud_tractor_front",
    }
}

/// Derives default steered wheel dimensions from a vehicle's physical chassis and visual anchors.
///
/// When a pre-baked computer-vision anchor exists (Spec 095), front axle offsets, track width,
/// and wheel dimensions derive directly from the vehicle artwork's visual geometry, with wheel
/// texture routed to its authentic clustered archetype and padding compensation factor applied.
pub fn derive_steered_wheel_config(
    model_id: &str,
    car_config: &tdrace_core::physics::CarConfig,
) -> Option<SteeredWheelConfig> {
    if let Some(anchor) = get_visual_wheel_anchor(model_id) {
        let body_half_len = car_config.chassis.half_length(car_config.wheelbase);
        let m_per_px = (body_half_len * 2.0 * 1.06) / 512.0;
        let front_axle_offset = (anchor.axle_x_px - 256.0) * m_per_px;
        let half_track_width = (anchor.track_width_px * 0.5) * m_per_px;
        let rubber_width = anchor.tire_wid_px * m_per_px;
        let rubber_diameter = anchor.tire_len_px * m_per_px;
        let wheel_texture_id = archetype_to_static_id(&anchor.archetype);
        let texture_padding_factor = archetype_padding_factor(wheel_texture_id);
        let layering = if anchor.layering == "OverChassis" {
            WheelLayerMode::OverChassis
        } else {
            WheelLayerMode::UnderChassis
        };

        return Some(SteeredWheelConfig {
            wheel_texture_id,
            front_axle_offset,
            half_track_width,
            wheel_size: glam::Vec2::new(rubber_width, rubber_diameter),
            texture_padding_factor,
            layering,
        });
    }

    let (wheel_texture_id, layering) = if let Some(classic_cfg) = get_steered_wheel_config(model_id) {
        (classic_cfg.wheel_texture_id, classic_cfg.layering)
    } else if let Some(model) = crate::catalog::find_model_by_id(model_id) {
        (
            platform_wheel_texture_id(model.base_car_choice),
            platform_wheel_layer_mode(model.base_car_choice),
        )
    } else {
        ("gt_slick_front", WheelLayerMode::UnderChassis)
    };

    let texture_padding_factor = archetype_padding_factor(wheel_texture_id);
    let wheel_size = if let Some(w) = car_config.wheels.first() {
        let size_x = w.tire_width;
        let size_y = (size_x * 2.0).clamp(0.24, 1.35);
        glam::Vec2::new(size_x, size_y)
    } else {
        glam::Vec2::new(0.24, 0.48)
    };

    Some(SteeredWheelConfig {
        wheel_texture_id,
        front_axle_offset: car_config.cg_to_front,
        half_track_width: car_config.track_width * 0.5,
        wheel_size,
        texture_padding_factor,
        layering,
    })
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
    } else {
        match wheel_id {
            "kart_slick_front" => KART_SLICK_FRONT_PNG.to_vec(),
            "gt_slick_front" => GT_SLICK_FRONT_PNG.to_vec(),
            "nascar_wheel_front" => NASCAR_WHEEL_FRONT_PNG.to_vec(),
            "offroad_wheel_front" => OFFROAD_WHEEL_FRONT_PNG.to_vec(),
            "rally_wheel_front" => RALLY_WHEEL_FRONT_PNG.to_vec(),
            "buggy_allterrain_front" => BUGGY_ALLTERRAIN_FRONT_PNG.to_vec(),
            "truck_allterrain_front" => TRUCK_ALLTERRAIN_FRONT_PNG.to_vec(),
            "monster_wheel_front" => MONSTER_WHEEL_FRONT_PNG.to_vec(),
            "mud_tractor_front" => MUD_TRACTOR_FRONT_PNG.to_vec(),
            _ => return None,
        }
    };

    let base_img = Image::from_file_with_format(&bytes, None).ok()?;
    let texture = Texture2D::from_image(&base_img);
    map.insert(wheel_id.to_string(), texture.clone());
    Some(texture)
}


