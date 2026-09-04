use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_triangle};

use super::color::{CarColorScheme, Palette};

/// Runtime toggle flags for all four player car visibility enhancement options.
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
}

impl Default for PlayerVisibilityOptions {
    fn default() -> Self {
        Self {
            overhead_chevron: true,
            ground_aura: true,
            adaptive_visibility: true,
            roof_beacon: true,
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
    let stroke = (1.5 / zoom).max(0.05);

    // Points of inverted triangle (tip pointing down toward car)
    let p_tip = anchor;
    let p_left = anchor + Vec2::new(-half_w, height);
    let p_right = anchor + Vec2::new(half_w, height);

    // Color with alpha_scale
    let mut fill_col = color_scheme.secondary;
    // Boost vibrancy if too dark
    if fill_col.r + fill_col.g + fill_col.b < 0.6 {
        fill_col = Palette::NEON_CYAN;
    }
    fill_col.a *= alpha_scale.clamp(0.0, 1.0);

    let outline_col = Color::new(0.02, 0.02, 0.05, 0.90 * alpha_scale);

    // Drop shadow slightly offset
    let s_off = Vec2::new(0.12, -0.12);
    draw_triangle(
        macroquad::prelude::Vec2::new(p_tip.x + s_off.x, p_tip.y + s_off.y),
        macroquad::prelude::Vec2::new(p_left.x + s_off.x, p_left.y + s_off.y),
        macroquad::prelude::Vec2::new(p_right.x + s_off.x, p_right.y + s_off.y),
        Color::new(0.0, 0.0, 0.0, 0.40 * alpha_scale),
    );

    // Fill triangle
    draw_triangle(
        macroquad::prelude::Vec2::new(p_tip.x, p_tip.y),
        macroquad::prelude::Vec2::new(p_left.x, p_left.y),
        macroquad::prelude::Vec2::new(p_right.x, p_right.y),
        fill_col,
    );

    // Triangle border
    draw_line(p_tip.x, p_tip.y, p_left.x, p_left.y, stroke, outline_col);
    draw_line(p_left.x, p_left.y, p_right.x, p_right.y, stroke, outline_col);
    draw_line(p_right.x, p_right.y, p_tip.x, p_tip.y, stroke, outline_col);

    // Inner highlight bar for vector polish
    let inner_col = Color::new(1.0, 1.0, 1.0, 0.55 * alpha_scale);
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
    if aura_col.r + aura_col.g + aura_col.b < 0.6 {
        aura_col = Palette::NEON_CYAN;
    }

    // Outer soft ambient halo
    let c_outer = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.16 * alpha_scale);
    draw_circle(pos.x, pos.y, radius, c_outer);

    // Mid-level luminous glow
    let c_mid = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.28 * alpha_scale);
    draw_circle(pos.x, pos.y, radius * 0.68, c_mid);

    // Inner bright underglow core
    let c_inner = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.45 * alpha_scale);
    draw_circle(pos.x, pos.y, radius * 0.38, c_inner);

    // Subtle edge ring for crisp definition
    let ring_stroke = (1.2 / zoom).max(0.04);
    let c_ring = Color::new(aura_col.r, aura_col.g, aura_col.b, 0.35 * alpha_scale);
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





