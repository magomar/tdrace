use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle,
};

use super::color::{CarColorScheme, Palette};
use crate::camera::RaceCamera;
use crate::ui::font::Fonts;
use crate::ui::CurveColorScheme;

/// Runtime toggle flags for player car visibility and HUD driving aids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerVisibilityOptions {
    /// Option 1: Inverted neon chevron (`▼`) floating above the player car roof.
    pub overhead_chevron: bool,
    /// Option 2: Luminous ground aura / underglow disc beneath the player chassis.
    pub ground_aura: bool,
    /// Option 3: Context-aware adaptive visibility (fades when close/fast, solid when far/slow).
    pub adaptive_visibility: bool,
    /// Option 4: High-visibility diegetic roof beacon / roll-hoop T-cam strobe.
    pub roof_beacon: bool,
    /// Option 5: Approaching curve indicator and dynamic braking helper on HUD.
    pub curve_helper: bool,
    /// Active color scheme for the curve approaching helper (cycled via key 6).
    pub curve_color_scheme: CurveColorScheme,
    /// Option 6: In-race dynamic floating bot nameplates (toggled via Alt key).
    pub bot_nameplates: bool,
}

impl Default for PlayerVisibilityOptions {
    fn default() -> Self {
        Self {
            overhead_chevron: true,
            ground_aura: true,
            adaptive_visibility: true,
            roof_beacon: true,
            curve_helper: true,
            curve_color_scheme: CurveColorScheme::Traffic,
            bot_nameplates: true,
        }
    }
}

/// Renders a zoom-invariant overhead indicator (inverted chevron/triangle `▼`)
/// hovering above the player car roof.
pub fn render_player_overhead_chevron(
    pos: Vec2,
    elevation: f32,
    current_zoom: f32,
    anim_time: f32,
    color_scheme: &CarColorScheme,
    alpha_scale: f32,
) {
    if alpha_scale <= 0.01 {
        return;
    }
    let zoom = current_zoom.max(0.5);

    // Height offset: clearance above car (2.2m radius) plus constant 14px on screen + bobbing
    let bob_px = (anim_time * 6.0).sin() * 3.0;
    let offset_y = 2.2 + (16.0 + bob_px) / zoom;
    let anchor = pos + Vec2::new(0.0, offset_y + elevation);

    // Chevron dimensions (fixed screen-pixel footprint: 18px wide x 13px tall)
    let half_w = 9.0 / zoom;
    let height = 13.0 / zoom;
    let stroke = (2.0 / zoom).max(0.06);

    // Points of inverted triangle (tip pointing down toward car)
    let p_tip = anchor;
    let p_left = anchor + Vec2::new(-half_w, height);
    let p_right = anchor + Vec2::new(half_w, height);

    // Color with alpha_scale
    let mut fill_col = color_scheme.secondary;
    // Boost vibrancy if too dark or low contrast
    if fill_col.r + fill_col.g + fill_col.b < 0.85 {
        fill_col = Palette::NEON_CYAN;
    }
    fill_col.a *= alpha_scale.clamp(0.0, 1.0);

    let outline_col = Color::new(0.0, 0.0, 0.0, 0.98 * alpha_scale);

    // Drop shadow slightly offset with higher contrast
    let s_off = Vec2::new(0.14, -0.14);
    draw_triangle(
        macroquad::prelude::Vec2::new(p_tip.x + s_off.x, p_tip.y + s_off.y),
        macroquad::prelude::Vec2::new(p_left.x + s_off.x, p_left.y + s_off.y),
        macroquad::prelude::Vec2::new(p_right.x + s_off.x, p_right.y + s_off.y),
        Color::new(0.0, 0.0, 0.0, 0.65 * alpha_scale),
    );

    // Fill triangle
    draw_triangle(
        macroquad::prelude::Vec2::new(p_tip.x, p_tip.y),
        macroquad::prelude::Vec2::new(p_left.x, p_left.y),
        macroquad::prelude::Vec2::new(p_right.x, p_right.y),
        fill_col,
    );

    // High-contrast triangle border
    draw_line(p_tip.x, p_tip.y, p_left.x, p_left.y, stroke, outline_col);
    draw_line(p_left.x, p_left.y, p_right.x, p_right.y, stroke, outline_col);
    draw_line(p_right.x, p_right.y, p_tip.x, p_tip.y, stroke, outline_col);

    // Inner bright highlight bar for vector polish and contrast
    let inner_col = Color::new(1.0, 1.0, 1.0, 0.85 * alpha_scale);
    let inner_top_l = anchor + Vec2::new(-half_w * 0.55, height * 0.72);
    let inner_top_r = anchor + Vec2::new(half_w * 0.55, height * 0.72);
    draw_line(inner_top_l.x, inner_top_l.y, inner_top_r.x, inner_top_r.y, stroke * 0.8, inner_col);
}

/// Renders a luminous ground aura / underglow disc beneath the player's chassis.
///
/// Features a zoom-compensated minimum radius so the car's luminous footprint
/// remains easily trackable by peripheral vision even when zoomed far out.
pub fn render_player_ground_aura(
    pos: Vec2,
    current_zoom: f32,
    color_scheme: &CarColorScheme,
    alpha_scale: f32,
) {
    if alpha_scale <= 0.01 {
        return;
    }
    let zoom = current_zoom.max(0.5);

    // Dynamic radius: 2.4m base radius, clamped to at least 15px on screen
    let radius = 2.4f32.max(15.0 / zoom);

    let mut aura_col = color_scheme.secondary;
    if aura_col.r + aura_col.g + aura_col.b < 0.85 {
        aura_col = Palette::NEON_CYAN;
    }

    // Outer soft ambient halo (higher contrast)
    let c_outer = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.32 * alpha_scale);
    draw_circle(pos.x, pos.y, radius, c_outer);

    // Mid-level luminous glow (higher contrast)
    let c_mid = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.52 * alpha_scale);
    draw_circle(pos.x, pos.y, radius * 0.68, c_mid);

    // Inner bright underglow core (higher contrast)
    let c_inner = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.75 * alpha_scale);
    draw_circle(pos.x, pos.y, radius * 0.38, c_inner);

    // High-contrast edge ring for crisp definition
    let ring_stroke = (2.0 / zoom).max(0.06);
    let c_ring = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.75 * alpha_scale);
    draw_circle_lines(pos.x, pos.y, radius, ring_stroke, c_ring);
}

/// Computes the context-aware adaptive visibility scale factor based on camera zoom and vehicle speed.
///
/// Returns 1.0 if adaptive visibility is disabled. When enabled, smoothly fades indicators
/// when zoomed close or travelling at high speed, and amplifies prominence when zoomed out or slow/stopped.
pub fn compute_adaptive_alpha(
    enabled: bool,
    current_zoom: f32,
    car_speed: f32,
    anim_time: f32,
) -> f32 {
    if !enabled {
        return 1.0;
    }

    // Zoom scaling: fades to 0.15 at Close zoom (>= 18.0), full 1.0 at Far/Overview (<= 6.0)
    let zoom_factor = ((18.0 - current_zoom) / (18.0 - 6.0)).clamp(0.15, 1.0);

    // Speed scaling: when moving slowly (< 8.0 m/s), boost visibility and add gentle breathing pulse
    let speed_boost = if car_speed < 8.0 {
        let t = (8.0 - car_speed) / 8.0;
        t * 0.35 + (anim_time * 5.0).sin() * 0.15 * t
    } else {
        0.0
    };

    (zoom_factor + speed_boost).clamp(0.15, 1.0)
}

/// Renders a diegetic high-visibility day-glo roof beacon / roll-hoop T-cam strobe atop the player car.
///
/// Clamped to a minimum screen-pixel radius so the car's cockpit emits a visible
/// luminous beacon even at extreme overview zoom levels.
pub fn render_player_roof_beacon(
    pos: Vec2,
    fwd: Vec2,
    elevation: f32,
    current_zoom: f32,
    anim_time: f32,
    _color_scheme: &CarColorScheme,
    alpha_scale: f32,
) {
    if alpha_scale <= 0.01 {
        return;
    }
    let zoom = current_zoom.max(0.5);

    // Position at cockpit / roll-hoop cell
    let beacon_pos = pos + fwd * 0.10 + Vec2::new(0.0, elevation);

    // Minimum 4.0 screen pixels radius
    let radius = 0.22f32.max(4.0 / zoom);

    // Rhythmic strobe pulse (8 Hz)
    let pulse = ((anim_time * 8.0).sin() * 0.5 + 0.5).powf(1.5);

    // Outer soft pulsing halo
    let outer_radius = radius * 1.8;
    let halo_col = Color::new(1.0, 0.85, 0.10, (0.25 + 0.35 * pulse) * alpha_scale);
    draw_circle(beacon_pos.x, beacon_pos.y, outer_radius, halo_col);

    // Day-glo fluorescent yellow core
    let core_col = Color::new(1.0, 0.92, 0.20, 0.95 * alpha_scale);
    draw_circle(beacon_pos.x, beacon_pos.y, radius, core_col);

    // Inner bright white flash highlight
    let inner_col = Color::new(1.0, 1.0, 1.0, (0.70 + 0.30 * pulse) * alpha_scale);
    draw_circle(beacon_pos.x, beacon_pos.y, radius * 0.42, inner_col);

    // Dark protective perimeter ring
    let ring_stroke = (1.0 / zoom).max(0.04);
    let outline_col = Color::new(0.08, 0.08, 0.10, 0.85 * alpha_scale);
    draw_circle_lines(beacon_pos.x, beacon_pos.y, radius, ring_stroke, outline_col);
}

// -----------------------------------------------------------------------------
// Floating Bot Nameplates & Proximity Culling Pipeline (Spec 029)
// -----------------------------------------------------------------------------

/// Inner distance threshold (meters) where floating nameplates render at full 100% opacity.
pub const NAMEPLATE_INNER_RADIUS: f32 = 25.0;

/// Outer distance threshold (meters) where floating nameplates completely fade out.
pub const NAMEPLATE_OUTER_RADIUS: f32 = 55.0;

/// Base clearance height (meters) above car center for nameplate world anchor.
pub const NAMEPLATE_HEIGHT_CLEARANCE: f32 = 2.4;

/// Horizontal distance threshold (pixels) below which badges are considered overlapping.
pub const NAMEPLATE_DECONFLICT_W_THRESH: f32 = 75.0;

/// Vertical distance threshold (pixels) below which badges are considered overlapping.
pub const NAMEPLATE_DECONFLICT_H_THRESH: f32 = 24.0;

/// Vertical nudge offset (pixels) applied to stacked overlapping badges.
pub const NAMEPLATE_STACK_NUDGE: f32 = 20.0;

/// Runtime metadata for an in-race vehicle floating nameplate.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleNameplateItem<'a> {
    /// Car index in game.cars.
    pub car_idx: usize,
    /// Display name or alias (e.g. "Vortex", "Thunder").
    pub name: &'a str,
    /// Optional tier badge string (e.g. "T1", "T4").
    pub tier_label: Option<&'a str>,
    /// Livery accent color for badge border and tier tag.
    pub accent_color: Color,
    /// World position of the car.
    pub position: Vec2,
    /// Dynamic jump elevation.
    pub elevation: f32,
    /// Distance from the focus player car.
    pub distance_to_player: f32,
}

/// Deconflicted layout position for a projected nameplate on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct DeconflictedNameplate<'a> {
    pub item: &'a VehicleNameplateItem<'a>,
    pub screen_center: Vec2,
    pub alpha: f32,
    pub width: f32,
    pub height: f32,
}

/// Computes the proximity alpha multiplier in [0.0, 1.0] based on distance to the player car.
pub fn compute_proximity_alpha(distance: f32) -> f32 {
    if distance <= NAMEPLATE_INNER_RADIUS {
        1.0
    } else if distance >= NAMEPLATE_OUTER_RADIUS {
        0.0
    } else {
        (NAMEPLATE_OUTER_RADIUS - distance) / (NAMEPLATE_OUTER_RADIUS - NAMEPLATE_INNER_RADIUS)
    }
}

/// Computes deconflicted screen-space badge positions to prevent overlapping in dense packs.
pub fn deconflict_nameplates<'a>(
    items: &'a [VehicleNameplateItem<'a>],
    camera: &RaceCamera,
    viewport_rect: Option<(f32, f32, f32, f32)>,
    fonts: &Fonts,
    master_alpha: f32,
) -> Vec<DeconflictedNameplate<'a>> {
    if master_alpha <= 0.005 || items.is_empty() {
        return Vec::new();
    }

    let (sw, sh) = RaceCamera::get_screen_dimensions_safe();
    let (vp_x, vp_y, vp_w, vp_h) = viewport_rect.unwrap_or((0.0, 0.0, sw, sh));
    let margin = 40.0;

    // 1. Filter candidates by proximity and viewport frustum culling
    let mut candidates = Vec::with_capacity(items.len());
    for item in items {
        let prox_alpha = compute_proximity_alpha(item.distance_to_player);
        let eff_alpha = prox_alpha * master_alpha;
        if eff_alpha <= 0.005 {
            continue;
        }

        // Anchor above the vehicle (clearance + jump elevation)
        let world_anchor = item.position + Vec2::new(0.0, NAMEPLATE_HEIGHT_CLEARANCE + item.elevation);
        let screen_pos = camera.world_to_screen_with_viewport(world_anchor, sw, sh);

        // Viewport frustum culling
        if screen_pos.x < vp_x - margin
            || screen_pos.x > vp_x + vp_w + margin
            || screen_pos.y < vp_y - margin
            || screen_pos.y > vp_y + vp_h + margin
        {
            continue;
        }

        // Measure text dimensions for pill layout
        let text_dim = fonts.measure_ui_bold(item.name, 12.0);
        let tier_w = if item.tier_label.is_some() { 24.0 } else { 0.0 };
        let badge_w = (text_dim.width + tier_w + 16.0).max(48.0);
        let badge_h = 20.0;

        candidates.push((item, screen_pos, eff_alpha, badge_w, badge_h));
    }

    // 2. Sort by distance ascending (closest car gets foreground priority)
    candidates.sort_by(|a, b| {
        a.0.distance_to_player
            .partial_cmp(&b.0.distance_to_player)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 3. Stagger / deconflict overlapping positions
    let mut placed: Vec<DeconflictedNameplate<'a>> = Vec::with_capacity(candidates.len());

    for (item, screen_pos, alpha, w, h) in candidates {
        let mut final_pos = screen_pos;
        let mut stack_level = 0;

        for existing in &placed {
            let dx = (existing.screen_center.x - final_pos.x).abs();
            let dy = (existing.screen_center.y - final_pos.y).abs();
            if dx < NAMEPLATE_DECONFLICT_W_THRESH && dy < NAMEPLATE_DECONFLICT_H_THRESH {
                stack_level += 1;
                final_pos.y = existing.screen_center.y - NAMEPLATE_STACK_NUDGE;
            }
        }

        let final_alpha = if stack_level >= 2 {
            alpha * 0.6
        } else {
            alpha
        };

        placed.push(DeconflictedNameplate {
            item,
            screen_center: final_pos,
            alpha: final_alpha,
            width: w,
            height: h,
        });
    }

    placed
}

/// Renders floating bot nameplates in screen space using smart proximity and culling.
pub fn render_floating_bot_nameplates(
    fonts: &Fonts,
    camera: &RaceCamera,
    viewport_rect: Option<(f32, f32, f32, f32)>,
    nameplates: &[VehicleNameplateItem],
    master_alpha: f32,
) {
    if master_alpha <= 0.005 || nameplates.is_empty() {
        return;
    }

    let deconflicted = deconflict_nameplates(nameplates, camera, viewport_rect, fonts, master_alpha);
    for badge in deconflicted {
        let alpha = badge.alpha.clamp(0.0, 1.0);
        if alpha <= 0.01 {
            continue;
        }

        let half_w = badge.width * 0.5;
        let half_h = badge.height * 0.5;
        let rx = badge.screen_center.x - half_w;
        let ry = badge.screen_center.y - half_h;

        // Background pill
        let bg_col = Color::new(0.06, 0.08, 0.12, 0.85 * alpha);
        let border_col = Color::new(
            badge.item.accent_color.r,
            badge.item.accent_color.g,
            badge.item.accent_color.b,
            0.90 * alpha,
        );

        // Drop shadow
        draw_rectangle(rx + 1.0, ry + 1.5, badge.width, badge.height, Color::new(0.0, 0.0, 0.0, 0.50 * alpha));
        // Fill
        draw_rectangle(rx, ry, badge.width, badge.height, bg_col);
        // Border
        draw_rectangle_lines(rx, ry, badge.width, badge.height, 1.0, border_col);

        let mut text_start_x = rx + 6.0;

        // Tier tag if present
        if let Some(tier) = badge.item.tier_label {
            let tier_dim = fonts.measure_ui_bold(tier, 10.0);
            let tier_tag_w = tier_dim.width + 6.0;
            let tier_tag_h = 13.0;
            let tier_tag_y = ry + (badge.height - tier_tag_h) * 0.5;

            // Tier box
            let tier_bg = Color::new(border_col.r * 0.25, border_col.g * 0.25, border_col.b * 0.25, 0.95 * alpha);
            draw_rectangle(text_start_x, tier_tag_y, tier_tag_w, tier_tag_h, tier_bg);
            draw_rectangle_lines(text_start_x, tier_tag_y, tier_tag_w, tier_tag_h, 0.8, border_col);

            let text_y = tier_tag_y + tier_tag_h - 2.5;
            fonts.draw_ui_bold(tier, text_start_x + 3.0, text_y, 10.0, Color::new(1.0, 1.0, 1.0, alpha));

            text_start_x += tier_tag_w + 5.0;
        }

        // Driver Name
        let font_size = 12.0;
        let text_dim = fonts.measure_ui_bold(badge.item.name, font_size);
        let name_y = ry + (badge.height + text_dim.height) * 0.5 - 2.0;

        // Text drop shadow
        fonts.draw_ui_bold(badge.item.name, text_start_x + 1.0, name_y + 1.0, font_size, Color::new(0.0, 0.0, 0.0, 0.70 * alpha));
        // Text foreground
        fonts.draw_ui_bold(badge.item.name, text_start_x, name_y, font_size, Color::new(0.95, 0.96, 0.98, alpha));
    }
}

