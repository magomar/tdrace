use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{
    draw_circle, draw_circle_lines, draw_line, draw_triangle,
};

use super::color::{CarColorScheme, Palette};
use crate::camera::RaceCamera;
use crate::ui::font::Fonts;
use crate::ui::CurveColorScheme;

/// Runtime toggle flags and visual parameters for player car visibility and HUD driving aids.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerVisibilityOptions {
    /// Option 1: Frontal directional chevron (`▲`) seated on the front of the player car's ground aura.
    pub frontal_chevron: bool,
    /// Size scale multiplier for the frontal chevron.
    pub frontal_chevron_scale: f32,
    /// Brightness multiplier for the frontal chevron.
    pub frontal_chevron_brightness: f32,
    /// Option 2: Luminous ground aura / underglow disc beneath the player chassis.
    pub ground_aura: bool,
    /// Radius ratio multiplier for the ground aura disc.
    pub ground_aura_radius_ratio: f32,
    /// Brightness multiplier for the ground aura disc.
    pub ground_aura_brightness: f32,
    /// Option 3: Context-aware adaptive visibility (fades when close/fast, solid when far/slow).
    pub adaptive_visibility: bool,
    /// Option 4: High-visibility diegetic roof beacon / roll-hoop T-cam strobe.
    pub roof_beacon: bool,
    /// Brightness multiplier for the roof beacon.
    pub roof_beacon_brightness: f32,
    /// Option 5: Approaching curve indicator and dynamic braking helper on HUD.
    pub curve_helper: bool,
    /// Size scale multiplier for the curve indicator.
    pub curve_helper_scale: f32,
    /// Brightness multiplier for the curve indicator.
    pub curve_helper_brightness: f32,
    /// Active color scheme for the curve approaching helper (cycled via key 6).
    pub curve_color_scheme: CurveColorScheme,
    /// Option 6: In-race dynamic floating bot nameplates (toggled via Alt key).
    pub bot_nameplates: bool,
    /// Maximum number of visible floating pilot labels (3 to 24).
    pub bot_nameplates_max_visible: usize,
    /// Opacity multiplier for floating pilot labels (0.20 to 1.00).
    pub bot_nameplates_opacity: f32,
    /// Compact pilot label mode (true = name only, false = full name + tier badge).
    pub bot_nameplates_compact: bool,
    /// Option 7: Expanding sonar / radar ping ripples when switching cameras or spinning out.
    pub sonar_ping: bool,
    /// Option 8: Mathematically ideal racing line visual aid display mode.
    pub racing_line_mode: tdrace_core::track::RacingLineMode,
    /// Opacity multiplier for the racing line ribbon overlay (0.20 to 1.00).
    pub racing_line_opacity: f32,
}

impl Default for PlayerVisibilityOptions {
    fn default() -> Self {
        Self {
            frontal_chevron: true,
            frontal_chevron_scale: 1.0,
            frontal_chevron_brightness: 1.0,
            ground_aura: true,
            ground_aura_radius_ratio: 1.0,
            ground_aura_brightness: 1.0,
            adaptive_visibility: true,
            roof_beacon: false,
            roof_beacon_brightness: 1.0,
            curve_helper: true,
            curve_helper_scale: 1.0,
            curve_helper_brightness: 1.0,
            curve_color_scheme: CurveColorScheme::Traffic,
            bot_nameplates: true,
            bot_nameplates_max_visible: 24,
            bot_nameplates_opacity: 0.80,
            bot_nameplates_compact: false,
            sonar_ping: true,
            racing_line_mode: tdrace_core::track::RacingLineMode::Full,
            racing_line_opacity: 0.70,
        }
    }
}

impl From<&crate::config::PlayerHelpersConfig> for PlayerVisibilityOptions {
    fn from(cfg: &crate::config::PlayerHelpersConfig) -> Self {
        let curve_color_scheme = match cfg.curve_color_scheme.to_lowercase().as_str() {
            "synthwave" => CurveColorScheme::Synthwave,
            "contrast" => CurveColorScheme::Contrast,
            "rally" => CurveColorScheme::Rally,
            _ => CurveColorScheme::Traffic,
        };
        Self {
            frontal_chevron: cfg.frontal_chevron,
            frontal_chevron_scale: cfg.frontal_chevron_scale,
            frontal_chevron_brightness: cfg.frontal_chevron_brightness,
            ground_aura: cfg.ground_aura,
            ground_aura_radius_ratio: cfg.ground_aura_radius_ratio,
            ground_aura_brightness: cfg.ground_aura_brightness,
            adaptive_visibility: cfg.adaptive_visibility,
            roof_beacon: cfg.roof_beacon,
            roof_beacon_brightness: cfg.roof_beacon_brightness,
            curve_helper: cfg.curve_helper,
            curve_helper_scale: cfg.curve_helper_scale,
            curve_helper_brightness: cfg.curve_helper_brightness,
            curve_color_scheme,
            bot_nameplates: cfg.bot_nameplates,
            bot_nameplates_max_visible: cfg.bot_nameplates_max_visible,
            bot_nameplates_opacity: cfg.bot_nameplates_opacity,
            bot_nameplates_compact: cfg.bot_nameplates_compact,
            sonar_ping: cfg.radar_sonar_ping,
            racing_line_mode: cfg.racing_line_mode,
            racing_line_opacity: cfg.racing_line_opacity,
        }
    }
}

impl From<&PlayerVisibilityOptions> for crate::config::PlayerHelpersConfig {
    fn from(opts: &PlayerVisibilityOptions) -> Self {
        let curve_color_scheme = match opts.curve_color_scheme {
            CurveColorScheme::Traffic => "traffic".to_string(),
            CurveColorScheme::Synthwave => "synthwave".to_string(),
            CurveColorScheme::Contrast => "contrast".to_string(),
            CurveColorScheme::Rally => "rally".to_string(),
        };
        Self {
            frontal_chevron: opts.frontal_chevron,
            frontal_chevron_scale: opts.frontal_chevron_scale,
            frontal_chevron_brightness: opts.frontal_chevron_brightness,
            ground_aura: opts.ground_aura,
            ground_aura_radius_ratio: opts.ground_aura_radius_ratio,
            ground_aura_brightness: opts.ground_aura_brightness,
            roof_beacon: opts.roof_beacon,
            roof_beacon_brightness: opts.roof_beacon_brightness,
            curve_helper: opts.curve_helper,
            curve_helper_scale: opts.curve_helper_scale,
            curve_helper_brightness: opts.curve_helper_brightness,
            curve_color_scheme,
            adaptive_visibility: opts.adaptive_visibility,
            radar_sonar_ping: opts.sonar_ping,
            bot_nameplates: opts.bot_nameplates,
            bot_nameplates_max_visible: opts.bot_nameplates_max_visible,
            bot_nameplates_opacity: opts.bot_nameplates_opacity,
            bot_nameplates_compact: opts.bot_nameplates_compact,
            racing_line_mode: opts.racing_line_mode,
            racing_line_opacity: opts.racing_line_opacity,
        }
    }
}

/// Number of arc segments used to conform the chevron base to the circular aura perimeter.
pub const PLAYER_CHEVRON_ARC_SEGMENTS: usize = 6;

/// Computed world-space geometry for the player's forward-pointing aura chevron.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerChevronGeometry {
    /// Forward unit direction vector of the vehicle.
    pub forward: Vec2,
    /// Radius of the ground aura ring where the chevron base is anchored.
    pub aura_radius: f32,
    /// World-space tip of the chevron pointing ahead of the car.
    pub tip: Vec2,
    /// Leftmost base vertex on the aura perimeter ring.
    pub base_left: Vec2,
    /// Rightmost base vertex on the aura perimeter ring.
    pub base_right: Vec2,
    /// Arc-conforming base vertices along the aura ring from left to right.
    pub base_arc: [Vec2; PLAYER_CHEVRON_ARC_SEGMENTS + 1],
    /// Angular half-spread (radians) of the chevron base along the aura perimeter.
    pub half_angle: f32,
    /// Radial protrusion length of the chevron tip beyond `aura_radius`.
    pub length: f32,
}

/// Computes the zoom-compensated radius of the player's ground aura ring in world meters.
#[inline]
pub fn player_aura_radius(current_zoom: f32, radius_ratio: f32) -> f32 {
    let zoom = current_zoom.max(0.5);
    let ratio = radius_ratio.max(0.2);
    (2.4f32 * ratio).max((15.0 * ratio) / zoom)
}

/// Computes the world-space geometry of the forward-pointing chevron seated on the front
/// perimeter of the player's ground aura ring.
pub fn compute_player_chevron_geometry(
    pos: Vec2,
    fwd: Vec2,
    current_zoom: f32,
    scale: f32,
    aura_radius_ratio: f32,
) -> PlayerChevronGeometry {
    let zoom = current_zoom.max(0.5);
    let s = scale.max(0.25);
    let dir = if fwd.length_squared() > 1e-6 {
        fwd.normalize()
    } else {
        Vec2::X
    };
    let right = Vec2::new(-dir.y, dir.x);
    let aura_radius = player_aura_radius(zoom, aura_radius_ratio);

    // Protrusion length and base half-width scale with the aura ring and maintain a minimum screen-pixel footprint when zoomed out
    let length = (aura_radius * 0.42 * s).max((10.0 * s) / zoom);
    let half_w = (aura_radius * 0.32 * s)
        .max((7.0 * s) / zoom)
        .min(aura_radius * 0.82);
    let half_angle = (half_w / aura_radius).clamp(0.05, 0.95).asin();

    let tip = pos + dir * (aura_radius + length);
    let mut base_arc = [Vec2::ZERO; PLAYER_CHEVRON_ARC_SEGMENTS + 1];
    for (k, slot) in base_arc.iter_mut().enumerate() {
        let t = -1.0 + 2.0 * (k as f32 / PLAYER_CHEVRON_ARC_SEGMENTS as f32);
        let theta = t * half_angle;
        *slot = pos + dir * (aura_radius * theta.cos()) + right * (aura_radius * theta.sin());
    }

    PlayerChevronGeometry {
        forward: dir,
        aura_radius,
        tip,
        base_left: base_arc[0],
        base_right: base_arc[PLAYER_CHEVRON_ARC_SEGMENTS],
        base_arc,
        half_angle,
        length,
    }
}

/// Renders a forward-pointing directional chevron seated on the front perimeter of the
/// player's ground aura ring, indicating both car location and heading when sprites are small.
#[allow(clippy::too_many_arguments)]
pub fn render_player_frontal_chevron(
    pos: Vec2,
    fwd: Vec2,
    current_zoom: f32,
    color_scheme: &CarColorScheme,
    alpha_scale: f32,
    scale: f32,
    brightness: f32,
    aura_radius_ratio: f32,
) {
    let effective_alpha = (alpha_scale * brightness).max(0.0);
    if effective_alpha <= 0.01 {
        return;
    }
    let zoom = current_zoom.max(0.5);
    let geom = compute_player_chevron_geometry(pos, fwd, zoom, scale, aura_radius_ratio);
    let dir = geom.forward;
    let right = Vec2::new(-dir.y, dir.x);

    // Match the ground aura color selection so the ring and forward pointer form a unified indicator
    let mut aura_col = color_scheme.secondary;
    if aura_col.r + aura_col.g + aura_col.b < 0.85 {
        aura_col = Palette::NEON_CYAN;
    }

    let m = |v: Vec2| macroquad::prelude::Vec2::new(v.x, v.y);

    // 1. Soft outer ambient halo matching the ground aura's outer glow layer
    let halo_col = Color::new(
        aura_col.r,
        aura_col.g,
        aura_col.b,
        (0.32 * effective_alpha).min(1.0),
    );
    let halo_tip = pos + dir * (geom.aura_radius + geom.length * 1.22);
    let halo_half_angle = (geom.half_angle * 1.20).min(1.15);
    for k in 0..PLAYER_CHEVRON_ARC_SEGMENTS {
        let t0 = -1.0 + 2.0 * (k as f32 / PLAYER_CHEVRON_ARC_SEGMENTS as f32);
        let t1 = -1.0 + 2.0 * ((k + 1) as f32 / PLAYER_CHEVRON_ARC_SEGMENTS as f32);
        let a0 = t0 * halo_half_angle;
        let a1 = t1 * halo_half_angle;
        let b0 = pos + dir * (geom.aura_radius * a0.cos()) + right * (geom.aura_radius * a0.sin());
        let b1 = pos + dir * (geom.aura_radius * a1.cos()) + right * (geom.aura_radius * a1.sin());
        draw_triangle(m(halo_tip), m(b0), m(b1), halo_col);
    }

    // 2. Main luminous chevron fill conforming to the outer circumference of the aura ring
    let fill_col = Color::new(
        aura_col.r,
        aura_col.g,
        aura_col.b,
        (0.68 * effective_alpha).min(1.0),
    );
    for k in 0..PLAYER_CHEVRON_ARC_SEGMENTS {
        draw_triangle(
            m(geom.tip),
            m(geom.base_arc[k]),
            m(geom.base_arc[k + 1]),
            fill_col,
        );
    }

    // 3. Inner bright core highlight matching the multi-layered luminous aura aesthetic
    let core_col = Color::new(
        (aura_col.r + 1.0) * 0.5,
        (aura_col.g + 1.0) * 0.5,
        (aura_col.b + 1.0) * 0.5,
        (0.50 * effective_alpha).min(1.0),
    );
    let core_tip = pos + dir * (geom.aura_radius + geom.length * 0.78);
    let core_base_r = geom.aura_radius + geom.length * 0.12;
    let core_half_w = (geom.aura_radius * geom.half_angle.sin()) * 0.45;
    let core_left = pos + dir * core_base_r - right * core_half_w;
    let core_right = pos + dir * core_base_r + right * core_half_w;
    draw_triangle(m(core_tip), m(core_left), m(core_right), core_col);

    // 4. Crisp leading-edge flank strokes continuing the aura perimeter ring to the tip
    let stroke = ((2.0 * scale.max(0.5)) / zoom).max(0.06);
    let edge_col = Color::new(
        aura_col.r,
        aura_col.g,
        aura_col.b,
        (0.78 * effective_alpha).min(1.0),
    );
    draw_line(
        geom.base_left.x,
        geom.base_left.y,
        geom.tip.x,
        geom.tip.y,
        stroke,
        edge_col,
    );
    draw_line(
        geom.base_right.x,
        geom.base_right.y,
        geom.tip.x,
        geom.tip.y,
        stroke,
        edge_col,
    );
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
    radius_ratio: f32,
    brightness: f32,
) {
    let effective_alpha = (alpha_scale * brightness).max(0.0);
    if effective_alpha <= 0.01 {
        return;
    }
    let zoom = current_zoom.max(0.5);

    // Dynamic radius: 2.4m base radius * ratio, clamped to at least (15.0 * ratio) px on screen
    let radius = player_aura_radius(zoom, radius_ratio);

    let mut aura_col = color_scheme.secondary;
    if aura_col.r + aura_col.g + aura_col.b < 0.85 {
        aura_col = Palette::NEON_CYAN;
    }

    // Outer soft ambient halo (higher contrast)
    let c_outer = Color::new(aura_col.r, aura_col.g, aura_col.b, (0.32 * effective_alpha).min(1.0));
    draw_circle(pos.x, pos.y, radius, c_outer);

    // Mid-level luminous glow (higher contrast)
    let c_mid = Color::new(aura_col.r, aura_col.g, aura_col.b, (0.52 * effective_alpha).min(1.0));
    draw_circle(pos.x, pos.y, radius * 0.68, c_mid);

    // Inner bright underglow core (higher contrast)
    let c_inner = Color::new(aura_col.r, aura_col.g, aura_col.b, (0.75 * effective_alpha).min(1.0));
    draw_circle(pos.x, pos.y, radius * 0.38, c_inner);

    // High-contrast edge ring for crisp definition
    let ring_stroke = (2.0 / zoom).max(0.06);
    let c_ring = Color::new(aura_col.r, aura_col.g, aura_col.b, (0.75 * effective_alpha).min(1.0));
    draw_circle_lines(pos.x, pos.y, radius, ring_stroke, c_ring);
}

/// Radius (in world meters) around the player car within which other vehicles
/// contribute to situational crowding for adaptive visibility scaling.
pub const ADAPTIVE_TRAFFIC_RADIUS: f32 = 22.0;

/// Weighted proximity sum at which traffic crowding saturates (`1.0`).
pub const ADAPTIVE_TRAFFIC_SATURATION: f32 = 2.4;

/// Relative multipliers applied to the user's configured visual aid settings
/// (brightness and size/radius) when Adaptive Visibility is active.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveVisibilityFactors {
    /// Relative multiplier applied to configured brightness (`1.0` when disabled; `[0.50, 1.50]` when enabled).
    pub alpha_multiplier: f32,
    /// Relative multiplier applied to configured radius/scale (`1.0` when disabled; `[0.90, 1.25]` when enabled).
    pub scale_multiplier: f32,
}

/// Computes a normalized `[0.0, 1.0]` traffic crowding factor around `player_pos`
/// based on the proximity of other vehicles within [`ADAPTIVE_TRAFFIC_RADIUS`].
pub fn compute_traffic_crowding(
    player_pos: Vec2,
    other_positions: impl IntoIterator<Item = Vec2>,
) -> f32 {
    let mut weighted_sum = 0.0f32;
    for pos in other_positions {
        let dist = player_pos.distance(pos);
        if dist < ADAPTIVE_TRAFFIC_RADIUS {
            let w = (1.0 - dist / ADAPTIVE_TRAFFIC_RADIUS).clamp(0.0, 1.0);
            weighted_sum += w * (0.5 + 0.5 * w);
        }
    }
    (weighted_sum / ADAPTIVE_TRAFFIC_SATURATION).clamp(0.0, 1.0)
}

/// Computes relative brightness and scale multipliers around the user's configured settings (`1.0`).
///
/// - When `enabled == false`: returns `(1.0, 1.0)` so the exact Settings values are used directly.
/// - When `enabled == true`: scales in a `[0.50, 1.50]` relative range around `1.0`:
///   - Decreases below `1.0` (down to `0.50` alpha / `0.90` scale) when visibility is very clear
///     (close zoom, racing speed, no nearby traffic).
///   - Increases above `1.0` (up to `1.50` alpha / `1.25` scale) when camera zoom pulls out,
///     when surrounded by nearby vehicles (`traffic_crowding`), or when stopped/recovering.
pub fn compute_adaptive_visibility(
    enabled: bool,
    current_zoom: f32,
    car_speed: f32,
    anim_time: f32,
    traffic_crowding: f32,
) -> AdaptiveVisibilityFactors {
    if !enabled {
        return AdaptiveVisibilityFactors {
            alpha_multiplier: 1.0,
            scale_multiplier: 1.0,
        };
    }

    let crowding = traffic_crowding.clamp(0.0, 1.0);

    // 1. Zoom relative factor around neutral Medium-Close zoom (14.0):
    //    - Close zoom (14.0..20.0+): diminishes below 1.0 down to 0.50 when visibility is clear
    //    - Far/Overview zoom (14.0..5.0): boosts above 1.0 up to 1.35 as sprites shrink
    let base_zoom_mult = if current_zoom >= 14.0 {
        let t = ((current_zoom - 14.0) / (20.0 - 14.0)).clamp(0.0, 1.0);
        1.0 - t * 0.50
    } else {
        let t = ((14.0 - current_zoom) / (14.0 - 5.0)).clamp(0.0, 1.0);
        1.0 + t * 0.35
    };

    // 2. Traffic crowding: cancels close-zoom dimming and adds up to +0.35 prominence in packs
    let zoom_with_traffic_floor = if base_zoom_mult < 1.0 {
        base_zoom_mult + (1.0 - base_zoom_mult) * crowding
    } else {
        base_zoom_mult
    };
    let traffic_boost = crowding * 0.35;

    // 3. Low-speed / spin-recovery boost (< 8.0 m/s) with gentle breathing pulse
    let speed_boost = if car_speed < 8.0 {
        let t = ((8.0 - car_speed.max(0.0)) / 8.0).clamp(0.0, 1.0);
        t * 0.20 + (anim_time * 5.0).sin() * 0.08 * t
    } else {
        0.0
    };

    let alpha_multiplier = (zoom_with_traffic_floor + traffic_boost + speed_boost).clamp(0.50, 1.50);

    // Relative size/radius scaling around 1.0:
    // Slightly compact (down to 0.90x) when visibility is clear, expanding up to 1.25x in far zoom or dense traffic
    let delta = alpha_multiplier - 1.0;
    let scale_multiplier = if delta < 0.0 {
        (1.0 + delta * 0.20).clamp(0.90, 1.0)
    } else {
        (1.0 + delta * 0.50).clamp(1.0, 1.25)
    };

    AdaptiveVisibilityFactors {
        alpha_multiplier,
        scale_multiplier,
    }
}

/// Computes the context-aware adaptive visibility alpha multiplier based on camera zoom and vehicle speed.
///
/// Returns `1.0` if adaptive visibility is disabled. When enabled, scales relative to `1.0`
/// (decreasing below `1.0` when close and clear, and boosting above `1.0` when zoomed out or slow/stopped).
pub fn compute_adaptive_alpha(
    enabled: bool,
    current_zoom: f32,
    car_speed: f32,
    anim_time: f32,
) -> f32 {
    compute_adaptive_visibility(enabled, current_zoom, car_speed, anim_time, 0.0).alpha_multiplier
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
    brightness: f32,
) {
    let effective_alpha = (alpha_scale * brightness).max(0.0);
    if effective_alpha <= 0.01 {
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
    let halo_col = Color::new(1.0, 0.85, 0.10, ((0.25 + 0.35 * pulse) * effective_alpha).min(1.0));
    draw_circle(beacon_pos.x, beacon_pos.y, outer_radius, halo_col);

    // Day-glo fluorescent yellow core
    let core_col = Color::new(1.0, 0.92, 0.20, (0.95 * effective_alpha).min(1.0));
    draw_circle(beacon_pos.x, beacon_pos.y, radius, core_col);

    // Inner bright white flash highlight
    let inner_col = Color::new(1.0, 1.0, 1.0, ((0.70 + 0.30 * pulse) * effective_alpha).min(1.0));
    draw_circle(beacon_pos.x, beacon_pos.y, radius * 0.42, inner_col);

    // Dark protective perimeter ring
    let ring_stroke = (1.0 / zoom).max(0.04);
    let outline_col = Color::new(0.08, 0.08, 0.10, (0.85 * effective_alpha).min(1.0));
    draw_circle_lines(beacon_pos.x, beacon_pos.y, radius, ring_stroke, outline_col);
}

/// Renders an expanding dual-shockwave radar / sonar ping ripple centered at `origin`.
///
/// Features a primary neon cyan wave and a secondary golden echo wave triggered after 18% progress.
/// Zero draw calls are issued when `timer <= 0.0` or `max_duration <= 0.0`.
pub fn render_player_sonar_ping(
    origin: Vec2,
    current_zoom: f32,
    timer: f32,
    max_duration: f32,
) {
    if timer <= 0.0 || max_duration <= 0.0 {
        return;
    }

    let p = (1.0 - (timer / max_duration).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let zoom = current_zoom.max(0.5);

    // Max shockwave radius: at least 12.0m, or 65.0/zoom in overview mode
    let max_radius = 12.0f32.max(65.0 / zoom);
    let stroke = ((2.8) / zoom).max(0.08);

    // 1. Primary Wave: radius 0.5 + p * max_radius, alpha (1.0 - p)^1.6
    let r1 = 0.5 + p * max_radius;
    let alpha1 = (1.0 - p).powf(1.6);

    if alpha1 > 0.005 {
        // Ambient soft halo behind primary wavefront
        let halo_col = Color::new(0.20, 0.90, 1.0, 0.20 * alpha1);
        draw_circle_lines(origin.x, origin.y, r1, stroke * 2.4, halo_col);

        // Core bright neon cyan wavefront
        let wave_col = Color::new(0.20, 0.90, 1.0, 0.95 * alpha1);
        draw_circle_lines(origin.x, origin.y, r1, stroke, wave_col);
    }

    // 2. Secondary Echo Wave: triggers when p > 0.18
    if p > 0.18 {
        let p_echo = (p - 0.18) / 0.82;
        let r2 = 0.5 + p_echo * (0.75 * max_radius);
        let alpha2 = (1.0 - p_echo).powf(1.8);

        if alpha2 > 0.005 {
            let echo_halo = Color::new(1.0, 0.85, 0.20, 0.15 * alpha2);
            draw_circle_lines(origin.x, origin.y, r2, stroke * 1.8, echo_halo);

            let echo_col = Color::new(1.0, 0.85, 0.20, 0.85 * alpha2);
            draw_circle_lines(origin.x, origin.y, r2, stroke * 0.8, echo_col);
        }
    }

    // 3. Center Origin Blip / Core Spark
    if p < 0.35 {
        let center_alpha = (1.0 - p / 0.35) * alpha1;
        draw_circle(
            origin.x,
            origin.y,
            (1.6 / zoom).max(0.18),
            Color::new(1.0, 1.0, 1.0, 0.90 * center_alpha),
        );
    }
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

/// Maximum number of bot nameplates displayed simultaneously on screen.
pub const MAX_VISIBLE_NAMEPLATES: usize = 5;

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
    /// Racing number (spec 092), `0` for none. The plate reads `#NN NAME`.
    pub race_number: u8,
}

impl VehicleNameplateItem<'_> {
    /// Text of the plate: the racing number before the name when the car has one.
    pub fn label(&self) -> String {
        if crate::ai::is_valid_race_number(self.race_number) {
            format!("#{} {}", self.race_number, self.name)
        } else {
            self.name.to_string()
        }
    }

    /// Compact label: name only without race number.
    pub fn compact_label(&self) -> String {
        self.name.to_string()
    }
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
    deconflict_nameplates_with_limit(
        items,
        camera,
        viewport_rect,
        fonts,
        master_alpha,
        MAX_VISIBLE_NAMEPLATES,
        false,
    )
}

/// Computes deconflicted screen-space badge positions with customizable max_visible limit and compact formatting.
pub fn deconflict_nameplates_with_limit<'a>(
    items: &'a [VehicleNameplateItem<'a>],
    camera: &RaceCamera,
    viewport_rect: Option<(f32, f32, f32, f32)>,
    fonts: &Fonts,
    master_alpha: f32,
    max_visible: usize,
    compact: bool,
) -> Vec<DeconflictedNameplate<'a>> {
    if master_alpha <= 0.005 || items.is_empty() || max_visible == 0 {
        return Vec::new();
    }

    let (sw, sh) = RaceCamera::get_screen_dimensions_safe();
    let (vp_x, vp_y, vp_w, vp_h) = viewport_rect.unwrap_or((0.0, 0.0, sw, sh));
    let margin = 20.0;

    // 1. Filter candidates strictly by active viewport bounds
    let mut candidates = Vec::with_capacity(items.len());
    for item in items {
        // Project car and anchor to screen space
        let world_anchor = item.position + Vec2::new(0.0, NAMEPLATE_HEIGHT_CLEARANCE + item.elevation);
        let screen_pos = camera.world_to_screen_with_viewport(world_anchor, sw, sh);
        let car_screen = camera.world_to_screen_with_viewport(item.position + Vec2::new(0.0, item.elevation), sw, sh);

        // Viewport culling: only racers visible within active screen viewport are candidates
        let in_viewport = (car_screen.x >= vp_x - margin
            && car_screen.x <= vp_x + vp_w + margin
            && car_screen.y >= vp_y - margin
            && car_screen.y <= vp_y + vp_h + margin)
            || (screen_pos.x >= vp_x - margin
                && screen_pos.x <= vp_x + vp_w + margin
                && screen_pos.y >= vp_y - margin
                && screen_pos.y <= vp_y + vp_h + margin);

        if !in_viewport {
            continue;
        }

        // Measure text dimensions for pill layout
        let label_str = if compact {
            item.compact_label()
        } else {
            item.label()
        };
        let text_dim = fonts.measure_ui_bold(&label_str, 12.0);
        let tier_w = if !compact && item.tier_label.is_some() { 24.0 } else { 0.0 };
        let badge_w = (text_dim.width + tier_w + 16.0).max(48.0);
        let badge_h = 20.0;

        candidates.push((item, screen_pos, master_alpha, badge_w, badge_h));
    }

    // 2. Sort by distance ascending (closest racers get priority)
    candidates.sort_by(|a, b| {
        a.0.distance_to_player
            .partial_cmp(&b.0.distance_to_player)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 3. Never show more than max_visible closest racers simultaneously
    candidates.truncate(max_visible);

    // 4. Stagger / deconflict overlapping positions
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

        // Clamp badge position inside viewport bounds with padding
        final_pos.x = final_pos.x.clamp(vp_x + w * 0.5 + 4.0, vp_x + vp_w - w * 0.5 - 4.0);
        final_pos.y = final_pos.y.clamp(vp_y + h * 0.5 + 4.0, vp_y + vp_h - h * 0.5 - 4.0);

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
    render_floating_bot_nameplates_ext(
        fonts,
        camera,
        viewport_rect,
        nameplates,
        master_alpha,
        MAX_VISIBLE_NAMEPLATES,
        false,
    );
}

/// Renders floating bot nameplates with customizable max visible limit and compact detail mode.
pub fn render_floating_bot_nameplates_ext(
    fonts: &Fonts,
    camera: &RaceCamera,
    viewport_rect: Option<(f32, f32, f32, f32)>,
    nameplates: &[VehicleNameplateItem],
    master_alpha: f32,
    max_visible: usize,
    compact: bool,
) {
    if master_alpha <= 0.005 || nameplates.is_empty() || max_visible == 0 {
        return;
    }

    // Nameplates are projected in screen space with fixed pixel metrics; drive the platform
    // chip with a reference-resolution scaler (scale == 1.0) so it renders 1:1.
    let scaler = crate::ui::scaler::UiScaler::new(1280.0, 720.0);

    let deconflicted = deconflict_nameplates_with_limit(
        nameplates,
        camera,
        viewport_rect,
        fonts,
        master_alpha,
        max_visible,
        compact,
    );
    for badge in deconflicted {
        let alpha = badge.alpha.clamp(0.0, 1.0);
        if alpha <= 0.01 {
            continue;
        }

        let half_w = badge.width * 0.5;
        let half_h = badge.height * 0.5;
        let rx = badge.screen_center.x - half_w;
        let ry = badge.screen_center.y - half_h;

        let mut text_start_x = rx + 6.0;

        // Tier tag chip (only in full/non-compact mode)
        if !compact {
            if let Some(tier) = badge.item.tier_label {
                let tier_dim = fonts.measure_ui_bold(tier, 10.0);
                let tier_tag_w = tier_dim.width + 6.0;
                let tier_tag_h = 13.0;
                let tier_tag_y = ry + (badge.height - tier_tag_h) * 0.5;

                cabinet::ui::draw_chip(
                    &scaler,
                    fonts,
                    text_start_x,
                    tier_tag_y,
                    tier_tag_w,
                    tier_tag_h,
                    tier,
                    Color::new(1.0, 1.0, 1.0, alpha),
                );

                text_start_x += tier_tag_w + 5.0;
            }
        }

        // Driver Name badge chip
        let name_w = (rx + badge.width - text_start_x).max(1.0);
        let display_label = if compact {
            badge.item.compact_label()
        } else {
            badge.item.label()
        };
        cabinet::ui::draw_chip(
            &scaler,
            fonts,
            text_start_x,
            ry,
            name_w,
            badge.height,
            &display_label,
            Color::new(0.95, 0.96, 0.98, alpha),
        );
    }
}

/// Renders a pulsing overhead chevron indicator above the designated team pit stall (Spec 062).
pub fn render_pit_box_chevron(
    stall_pos: Vec2,
    current_zoom: f32,
    anim_time: f32,
    color_scheme: &CarColorScheme,
) {
    let zoom = current_zoom.max(0.5);
    let pulse = (anim_time * 5.0).sin().abs() * 0.35 + 0.65;
    let bob = (anim_time * 6.0).sin() * (4.0 / zoom);
    let anchor = stall_pos + Vec2::new(0.0, 4.0 + (16.0 / zoom) + bob);

    let half_w = 12.0 / zoom;
    let height = 16.0 / zoom;

    let p_tip = anchor;
    let p_left = anchor + Vec2::new(-half_w, height);
    let p_right = anchor + Vec2::new(half_w, height);

    let mut col = color_scheme.primary;
    col.a = pulse;
    let border = Color::new(0.0, 0.0, 0.0, 0.9 * pulse);

    // Drop shadow
    let shadow_off = Vec2::new(1.0 / zoom, 1.5 / zoom);
    let m = |v: Vec2| macroquad::math::Vec2::new(v.x, v.y);
    draw_triangle(m(p_tip + shadow_off), m(p_left + shadow_off), m(p_right + shadow_off), Color::new(0.0, 0.0, 0.0, 0.4 * pulse));
    draw_triangle(m(p_tip), m(p_left), m(p_right), col);
    draw_line(p_tip.x, p_tip.y, p_left.x, p_left.y, (2.0 / zoom).max(0.1), border);
    draw_line(p_left.x, p_left.y, p_right.x, p_right.y, (2.0 / zoom).max(0.1), border);
    draw_line(p_right.x, p_right.y, p_tip.x, p_tip.y, (2.0 / zoom).max(0.1), border);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_visibility_options_defaults_and_conversion() {
        let opts = PlayerVisibilityOptions::default();
        assert!(opts.frontal_chevron);
        assert!(opts.ground_aura);
        assert!(opts.adaptive_visibility);
        assert!(!opts.roof_beacon);
        assert!(opts.curve_helper);
        assert!(opts.bot_nameplates);
        assert_eq!(opts.bot_nameplates_max_visible, 24);
        assert!((opts.bot_nameplates_opacity - 0.80).abs() < 1e-4);
        assert!(!opts.bot_nameplates_compact);
        assert!(opts.sonar_ping);

        let cfg = crate::config::PlayerHelpersConfig::from(&opts);
        assert!(cfg.radar_sonar_ping);
        assert_eq!(cfg.bot_nameplates_max_visible, 24);
        assert!((cfg.bot_nameplates_opacity - 0.80).abs() < 1e-4);
        assert!(!cfg.bot_nameplates_compact);

        let roundtrip = PlayerVisibilityOptions::from(&cfg);
        assert_eq!(roundtrip, opts);
    }

    #[test]
    fn test_render_player_sonar_ping_bounds() {
        // Zero or negative timer should return early without executing draw commands or crashing
        render_player_sonar_ping(Vec2::ZERO, 10.0, 0.0, 0.75);
        render_player_sonar_ping(Vec2::ZERO, 10.0, -0.5, 0.75);
        render_player_sonar_ping(Vec2::ZERO, 10.0, 0.5, 0.0);
    }

    #[test]
    fn test_player_chevron_geometry_follows_heading_and_aura_ring() {
        let pos = Vec2::new(40.0, -25.0);
        for &angle in &[0.0f32, 0.65, std::f32::consts::FRAC_PI_2, 2.35, -1.2] {
            let fwd = Vec2::new(angle.cos(), angle.sin());
            let geom = compute_player_chevron_geometry(pos, fwd, 14.0, 1.0, 1.0);

            // Forward direction and tip alignment
            assert!((geom.forward - fwd).length() < 1e-5);
            let tip_vec = geom.tip - pos;
            let tip_dir = tip_vec.normalize();
            assert!((tip_dir - fwd).length() < 1e-5, "Chevron tip must point in car forward direction");
            assert!(
                (tip_vec.length() - (geom.aura_radius + geom.length)).abs() < 1e-4,
                "Chevron tip must extend beyond aura radius by chevron length"
            );

            // Every base arc vertex must sit directly on the aura perimeter circle
            for pt in &geom.base_arc {
                let r = (*pt - pos).length();
                assert!(
                    (r - geom.aura_radius).abs() < 1e-4,
                    "Base arc vertex must lie on aura ring radius {}, got {}",
                    geom.aura_radius,
                    r
                );
            }
        }
    }
}


