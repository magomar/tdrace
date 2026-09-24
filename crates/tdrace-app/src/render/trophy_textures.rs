use std::collections::HashMap;
use std::sync::Mutex;
use macroquad::color::{Color, WHITE};
use macroquad::math::Vec2;
use macroquad::shapes::{draw_circle, draw_rectangle, draw_rectangle_lines};
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
    let (bg_col, border_col) = match metal {
        Some(TrophyMetal::Gold) => (Color::new(0.20, 0.16, 0.05, 0.90), Palette::NEON_GOLD),
        Some(TrophyMetal::Silver) => (Color::new(0.12, 0.15, 0.20, 0.90), Color::new(0.85, 0.90, 0.95, 1.0)),
        Some(TrophyMetal::Bronze) => (Color::new(0.18, 0.10, 0.06, 0.90), Color::new(0.85, 0.55, 0.35, 1.0)),
        None => (Color::new(0.06, 0.08, 0.12, 0.60), Color::new(0.25, 0.30, 0.38, 0.80)),
    };

    draw_rectangle(x, y, w, h, bg_col);
    draw_rectangle_lines(x, y, w, h, 1.5, border_col);

    let cx = x + w * 0.5;
    let cy = y + h * 0.5;

    if metal.is_some() {
        // Draw trophy cup icon silhouette
        let cup_w = w * 0.45;
        let cup_h = h * 0.35;
        draw_rectangle(cx - cup_w * 0.5, cy - cup_h * 0.4, cup_w, cup_h, border_col);
        // Pedestal
        draw_rectangle(cx - cup_w * 0.3, cy + cup_h * 0.6, cup_w * 0.6, cup_h * 0.3, border_col);

        // Draw star dots
        let num_stars = tier.clamp(1, 5);
        let star_r = (w * 0.04).clamp(1.5, 4.0);
        let star_spacing = star_r * 2.8;
        let start_x = cx - (num_stars as f32 - 1.0) * star_spacing * 0.5;
        for i in 0..num_stars {
            draw_circle(start_x + i as f32 * star_spacing, y + h * 0.18, star_r, border_col);
        }
    } else {
        // Locked padlock silhouette
        let lock_w = w * 0.30;
        let lock_h = h * 0.30;
        draw_rectangle(cx - lock_w * 0.5, cy - lock_h * 0.2, lock_w, lock_h, border_col);
        draw_circle(cx, cy - lock_h * 0.4, lock_w * 0.4, border_col);
        draw_circle(cx, cy - lock_h * 0.4, lock_w * 0.25, bg_col);
    }
}
