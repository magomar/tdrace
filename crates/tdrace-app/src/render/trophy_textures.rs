use std::collections::HashMap;
use std::sync::Mutex;
use macroquad::color::{Color, WHITE};
use macroquad::math::Vec2;
use macroquad::texture::{draw_texture_ex, DrawTextureParams, Image, Texture2D};

use crate::profile::TrophyMetal;
use crate::render::color::Palette;

static LOCKED_TROPHY_128_PNG: &[u8] = include_bytes!("../../../../assets/icons/trophies/trophy_locked-128.png");
static LOCKED_TROPHY_256_PNG: &[u8] = include_bytes!("../../../../assets/icons/trophies/trophy_locked-256.png");

static TROPHY_CACHE: Mutex<Option<HashMap<String, Texture2D>>> = Mutex::new(None);

/// Normalizes user or TOML discipline identifiers to the canonical 5 motorsport modules.
pub fn normalize_discipline(disc: &str) -> &'static str {
    match disc {
        "gt" | "gt_challenge" => "gt",
        "kart" | "karting" => "kart",
        "rally" | "rallycross" => "rally",
        "nascar" => "nascar",
        "extreme_offroad" | "offroad" => "extreme_offroad",
        _ => "gt",
    }
}

/// Computes the authentic icon asset filename matching Spec 027.
pub fn trophy_filename(discipline: &str, tier: u32, metal: Option<TrophyMetal>, high_dpi: bool) -> String {
    let size_suffix = if high_dpi { "256" } else { "128" };
    match metal {
        Some(m) => {
            let disc = normalize_discipline(discipline);
            let t = tier.clamp(1, 5);
            format!("{}_t{}_{}-{}.png", disc, t, m.as_str(), size_suffix)
        }
        None => format!("trophy_locked-{}.png", size_suffix),
    }
}

fn find_trophy_asset_bytes(filename: &str) -> Option<Vec<u8>> {
    let candidates = [
        format!("assets/icons/trophies/{}", filename),
        format!("../assets/icons/trophies/{}", filename),
        format!("../../assets/icons/trophies/{}", filename),
        format!("../../../assets/icons/trophies/{}", filename),
    ];
    for c in &candidates {
        if let Ok(data) = std::fs::read(c) {
            return Some(data);
        }
    }
    None
}

/// Retrieves or decodes a championship trophy texture for the given discipline, tier, and podium metal.
/// When metal is None, returns the locked carbon silhouette texture.
pub fn get_trophy_texture(
    discipline: &str,
    tier: u32,
    metal: Option<TrophyMetal>,
    high_dpi: bool,
) -> Option<Texture2D> {
    if crate::storage::is_test_environment() {
        return None;
    }

    let filename = trophy_filename(discipline, tier, metal, high_dpi);
    let mut guard = TROPHY_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);

    if let Some(tex) = map.get(&filename) {
        return Some(tex.clone());
    }

    let bytes = if filename == "trophy_locked-128.png" {
        LOCKED_TROPHY_128_PNG.to_vec()
    } else if filename == "trophy_locked-256.png" {
        LOCKED_TROPHY_256_PNG.to_vec()
    } else if let Some(b) = find_trophy_asset_bytes(&filename) {
        b
    } else {
        return None;
    };

    let img = Image::from_file_with_format(&bytes, None).ok()?;
    let tex = Texture2D::from_image(&img);
    map.insert(filename, tex.clone());
    Some(tex)
}

/// Draws an authentic championship trophy badge or locked silhouette inside the specified rectangle.
pub fn draw_trophy_badge(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    discipline: &str,
    tier: u32,
    metal: Option<TrophyMetal>,
    high_dpi: bool,
) {
    if let Some(tex) = get_trophy_texture(discipline, tier, metal, high_dpi) {
        draw_texture_ex(
            &tex,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(w, h)),
                ..Default::default()
            },
        );
    } else {
        draw_procedural_trophy_fallback(x, y, w, h, discipline, tier, metal);
    }
}

/// Procedural fallback when running in headless tests or if image decoding fails.
fn draw_procedural_trophy_fallback(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    _discipline: &str,
    tier: u32,
    metal: Option<TrophyMetal>,
) {
    // Reuse the platform chip for a consistent arcade badge silhouette on the fallback path.
    let scaler = cabinet::ui::UiScaler::new(1280.0, 720.0);
    let fonts = cabinet::ui::Fonts::default();

    match metal {
        Some(m) => {
            let (label, text_color): (String, Color) = match m {
                TrophyMetal::Gold => (
                    format!("GOLD T{}", tier.clamp(1, 5)),
                    Palette::NEON_GOLD,
                ),
                TrophyMetal::Silver => (
                    format!("SILVER T{}", tier.clamp(1, 5)),
                    Color::new(0.85, 0.90, 0.95, 1.0),
                ),
                TrophyMetal::Bronze => (
                    format!("BRONZE T{}", tier.clamp(1, 5)),
                    Color::new(0.85, 0.55, 0.35, 1.0),
                ),
            };
            cabinet::ui::draw_chip(&scaler, &fonts, x, y, w, h, &label, text_color);
        }
        None => {
            cabinet::ui::draw_chip(
                &scaler,
                &fonts,
                x,
                y,
                w,
                h,
                "LOCKED",
                Color::new(0.25, 0.30, 0.38, 0.80),
            );
        }
    }
}
