use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_line, draw_triangle};
use serde::{Deserialize, Serialize};
use tdrace_core::physics::car::Car;
use tdrace_core::track::curve::{CurveApproachStatus, CurveDirection, TrackCurve};
use tdrace_core::track::spline::TrackSpline;
use tdrace_core::track::Track;

/// Visual style of the upcoming-curve indicator. Turning it off is `curve_helper = false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CurveIndicatorStyle {
    /// Severity chevrons (`>>>`) beside the car: one chevron per degree.
    #[default]
    Chevrons,
    /// Rally pacenote icon beside the car: a small drawing of the curve's own shape.
    Pacenote,
}

impl CurveIndicatorStyle {
    /// Name stored in `config.toml`.
    pub const fn as_config_str(&self) -> &'static str {
        match self {
            Self::Chevrons => "chevrons",
            Self::Pacenote => "pacenote",
        }
    }

    /// Parses the `config.toml` name; unknown names fall back to chevrons.
    pub fn from_config_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pacenote" => Self::Pacenote,
            _ => Self::Chevrons,
        }
    }
}

/// Available color schemes for the curve approaching & braking indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveColorScheme {
    /// Classic Traffic Signal Gradient: Green -> Yellow -> Orange -> Red (Default).
    Traffic,
    /// Arcade Synthwave: Neon Cyan -> Neon Magenta -> Laser Red.
    Synthwave,
    /// High-Contrast Minimalist: Ice White -> Amber Warning -> Pure Red.
    Contrast,
    /// Motorsport Rally Pacenotes: Color fixed by curve degree, flashing Red on critical brake.
    Rally,
}

impl Default for CurveColorScheme {
    fn default() -> Self {
        Self::Traffic
    }
}

impl CurveColorScheme {
    /// Cycles to the next color scheme in the sequence.
    pub const fn next(&self) -> Self {
        match self {
            Self::Traffic => Self::Synthwave,
            Self::Synthwave => Self::Contrast,
            Self::Contrast => Self::Rally,
            Self::Rally => Self::Traffic,
        }
    }

    /// User-facing display title for HUD toast notification.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Traffic => "TRAFFIC (GREEN-YELLOW-RED)",
            Self::Synthwave => "SYNTHWAVE (CYAN-MAGENTA-RED)",
            Self::Contrast => "HIGH CONTRAST (WHITE-AMBER-RED)",
            Self::Rally => "RALLY PACENOTES (RATING-CODED)",
        }
    }
}

/// Linearly interpolates between two macroquad colors.
pub fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        c1.r + (c2.r - c1.r) * t,
        c1.g + (c2.g - c1.g) * t,
        c1.b + (c2.b - c1.b) * t,
        c1.a + (c2.a - c1.a) * t,
    )
}

/// Computes the active foreground and border/glow colors based on color scheme, dynamic urgency, and degree.
pub fn compute_curve_colors(
    scheme: CurveColorScheme,
    urgency: f32,
    degree: u8,
    alpha: f32,
) -> (Color, Color) {
    let u = urgency.clamp(0.0, 1.0);

    let (base_col, border_col) = match scheme {
        CurveColorScheme::Traffic => {
            let green = Color::new(0.18, 0.92, 0.38, 1.0); // Vibrant Neon Green
            let yellow = Color::new(1.0, 0.84, 0.12, 1.0); // Bright Neon Yellow
            let orange = Color::new(1.0, 0.48, 0.10, 1.0); // Warning Neon Orange
            let red = Color::new(0.96, 0.16, 0.20, 1.0); // Critical Red

            let c = if u < 0.35 {
                lerp_color(green, yellow, u / 0.35)
            } else if u < 0.70 {
                lerp_color(yellow, orange, (u - 0.35) / 0.35)
            } else {
                lerp_color(orange, red, (u - 0.70) / 0.30)
            };
            (c, c)
        }
        CurveColorScheme::Synthwave => {
            let cyan = Color::new(0.20, 0.90, 1.0, 1.0); // Cyber Neon Cyan
            let magenta = Color::new(1.0, 0.25, 0.80, 1.0); // Neon Magenta
            let red = Color::new(1.0, 0.10, 0.25, 1.0); // Laser Red

            let c = if u < 0.55 {
                lerp_color(cyan, magenta, u / 0.55)
            } else {
                lerp_color(magenta, red, (u - 0.55) / 0.45)
            };
            (c, c)
        }
        CurveColorScheme::Contrast => {
            let white = Color::new(0.88, 0.95, 1.0, 1.0); // Ice White
            let amber = Color::new(1.0, 0.65, 0.05, 1.0); // Amber
            let red = Color::new(0.96, 0.16, 0.18, 1.0); // Pure Red

            let c = if u < 0.50 {
                lerp_color(white, amber, u / 0.50)
            } else {
                lerp_color(amber, red, (u - 0.50) / 0.50)
            };
            (c, c)
        }
        CurveColorScheme::Rally => {
            if u >= 0.80 {
                let red = Color::new(0.96, 0.16, 0.20, 1.0);
                (red, red)
            } else {
                let deg_col = match degree {
                    1 => Color::new(0.20, 0.85, 0.55, 1.0), // Emerald
                    2 => Color::new(0.20, 0.90, 1.0, 1.0),  // Cyan
                    3 => Color::new(1.0, 0.82, 0.15, 1.0),  // Yellow
                    4 => Color::new(1.0, 0.52, 0.12, 1.0),  // Orange
                    _ => Color::new(1.0, 0.22, 0.70, 1.0),  // Rose/Magenta
                };
                (deg_col, deg_col)
            }
        }
    };

    (
        Color::new(base_col.r, base_col.g, base_col.b, base_col.a * alpha),
        Color::new(border_col.r, border_col.g, border_col.b, border_col.a * alpha),
    )
}

/// Time to curve entry (seconds) at which the indicator starts to fade in.
pub const CURVE_INDICATOR_FADE_START_S: f32 = 4.5;
/// Time to curve entry (seconds) at which the indicator is fully visible.
pub const CURVE_INDICATOR_FADE_END_S: f32 = 4.0;
/// Speed floor (m/s) for the time-to-entry estimate, so a slow or stopped car
/// still sees the turn at `CURVE_INDICATOR_FADE_START_S * 15 m/s` = 67.5 m.
pub const CURVE_INDICATOR_MIN_ETA_SPEED: f32 = 15.0;

/// Estimated time (seconds) until the car reaches `distance_to_entry` at `speed_mps`.
#[inline]
pub fn compute_curve_eta(distance_to_entry: f32, speed_mps: f32) -> f32 {
    distance_to_entry / speed_mps.max(CURVE_INDICATOR_MIN_ETA_SPEED)
}

/// Curve search distance (meters) that covers the whole fade-in window at `speed_mps`.
#[inline]
pub fn curve_indicator_lookahead(speed_mps: f32) -> f32 {
    CURVE_INDICATOR_FADE_START_S * speed_mps.max(CURVE_INDICATOR_MIN_ETA_SPEED)
}

/// Calculates the display opacity for the curve indicator based on time to curve entry and apex traversal.
pub fn compute_indicator_alpha(
    distance_to_entry: f32,
    distance_to_apex: f32,
    is_inside_curve: bool,
    speed_mps: f32,
) -> f32 {
    // Smooth fade in on approach (4.5 s -> 4.0 s before entry), so faster cars get earlier warning
    let approach_alpha = if is_inside_curve {
        1.0
    } else {
        let eta = compute_curve_eta(distance_to_entry, speed_mps);
        ((CURVE_INDICATOR_FADE_START_S - eta)
            / (CURVE_INDICATOR_FADE_START_S - CURVE_INDICATOR_FADE_END_S))
            .clamp(0.0, 1.0)
    };

    // Quick fade away after the apex / inflexion point of the curve has been traversed (< 0.0)
    let apex_fade_alpha = if distance_to_apex < 0.0 {
        let past_apex = -distance_to_apex;
        // Fades smoothly to zero across 10 meters past apex
        (1.0 - past_apex / 10.0).clamp(0.0, 1.0)
    } else {
        1.0
    };

    approach_alpha * apex_fade_alpha
}

/// Unit vector from the player car toward the side of the curve, in the car's own frame
/// (the car's left or right, not the screen's).
#[inline]
fn curve_side_vector(player_car: &Car, direction: CurveDirection) -> Vec2 {
    match direction {
        CurveDirection::Left => -player_car.right_vector(),
        CurveDirection::Right => player_car.right_vector(),
    }
}

/// Distance (world units) from the car center to the inner edge of the curve indicator:
/// half the car's track width, plus 0.5 m for the body, plus a 10 px gap on screen.
#[inline]
pub fn curve_indicator_inner_clearance(player_car: &Car, current_zoom: f32) -> f32 {
    player_car.config.track_width * 0.5 + 0.5 + 10.0 / current_zoom.max(0.5)
}

/// Computes the world-space center of the curve alert chevron block next to the player car.
///
/// Places the block on the car's own left or right side (perpendicular to its heading),
/// so the arrows stay beside the car whichever way it faces on screen.
/// Factors in the total width of the chevron block so multi-arrow clusters expand outward
/// while maintaining generous clearance from the vehicle chassis.
pub fn compute_curve_arrow_position(
    player_car: &Car,
    direction: CurveDirection,
    degree: u8,
    current_zoom: f32,
) -> Vec2 {
    let zoom = current_zoom.max(0.5);
    let car_pos = player_car.state.position;
    let elevation = player_car.total_elevation();

    // Total width of the chevron cluster
    let deg = degree.clamp(1, 5) as usize;
    let spacing = 16.0 / zoom;
    let chevron_w = 14.0 / zoom;
    let total_w = (deg as f32 - 1.0) * spacing + chevron_w;

    // Clearance between the car center and the nearest chevron
    let inner_clearance = curve_indicator_inner_clearance(player_car, zoom);

    // Center of the chevron block is offset so the innermost chevron maintains `inner_clearance`
    let lateral_dist = inner_clearance + total_w * 0.5;
    car_pos + curve_side_vector(player_car, direction) * lateral_dist + Vec2::new(0.0, elevation)
}

/// Backwards-compatible alias for `compute_curve_arrow_position` without dynamic repositioning overhead.
#[inline]
pub fn compute_smart_curve_arrow_position(
    _track: &Track,
    _all_cars: &[Car],
    player_car: &Car,
    direction: CurveDirection,
    degree: u8,
    current_zoom: f32,
) -> Vec2 {
    compute_curve_arrow_position(player_car, direction, degree, current_zoom)
}

/// Renders the simplified curve alert chevrons in world space beside the player car.
///
/// Features no background box and no text labels — only anti-aliased, glowing vector chevrons
/// on the car's own left or right side, pointing away from the car toward the curve.
pub fn render_curve_indicator(
    player_car: &Car,
    status: &CurveApproachStatus,
    scheme: CurveColorScheme,
    current_zoom: f32,
    anim_time: f32,
    scale: f32,
    brightness: f32,
) {
    let base_alpha = compute_indicator_alpha(
        status.distance_to_entry,
        status.distance_to_apex,
        status.is_inside_curve,
        player_car.state.speed,
    );
    let alpha = (base_alpha * brightness).clamp(0.0, 1.0);

    if alpha <= 0.02 {
        return;
    }

    let degree = status.curve.degree.clamp(1, 5);
    let (color, _border_color) = compute_curve_colors(scheme, status.urgency, degree, alpha);

    // Pulse scale when in critical braking envelope
    let is_critical = status.urgency >= 0.85;
    let pulse_scale = if is_critical {
        1.0 + (anim_time * 9.0).sin().abs() * 0.10
    } else {
        1.0
    };

    let zoom = current_zoom.max(0.5);

    // Car frame: `side` points from the car toward the curve, `fwd` along the car heading
    let side = curve_side_vector(player_car, status.curve.direction);
    let fwd = player_car.forward_vector();
    let inner_clearance = curve_indicator_inner_clearance(player_car, zoom);
    let origin = player_car.state.position + Vec2::new(0.0, player_car.total_elevation());

    // Vector chevron dimensions in world units (scaled by 1.0 / zoom for fixed screen size, modulated by scale)
    let chevron_w = (14.0 * pulse_scale * scale) / zoom;
    let chevron_h = (22.0 * pulse_scale * scale) / zoom;
    let chevron_thickness = (3.5 * pulse_scale * scale) / zoom;
    let spacing = (16.0 * pulse_scale * scale) / zoom;
    let shadow_offset = Vec2::new(1.6 * scale, -1.6 * scale) / zoom;

    let shadow_col = Color::new(0.0, 0.0, 0.0, (0.70 * alpha).min(1.0));
    let glow_alpha = (0.35 * alpha * brightness.min(2.0)).min(1.0);
    let glow_col = Color::new(color.r, color.g, color.b, glow_alpha);

    for i in 0..degree as usize {
        // Each chevron points away from the car: arms at `u`, tip at `u + chevron_w`
        let u = inner_clearance + (i as f32) * spacing;
        let apex = origin + side * (u + chevron_w);
        let top = origin + side * u + fwd * (chevron_h * 0.5);
        let btm = origin + side * u - fwd * (chevron_h * 0.5);

        // 1. Dark outer drop shadow
        let (st, sa, sb) = (top + shadow_offset, apex + shadow_offset, btm + shadow_offset);
        draw_line(st.x, st.y, sa.x, sa.y, chevron_thickness + 1.2 / zoom, shadow_col);
        draw_line(sa.x, sa.y, sb.x, sb.y, chevron_thickness + 1.2 / zoom, shadow_col);

        // 2. Glowing ambient stroke
        draw_line(top.x, top.y, apex.x, apex.y, chevron_thickness * 1.8, glow_col);
        draw_line(apex.x, apex.y, btm.x, btm.y, chevron_thickness * 1.8, glow_col);

        // 3. Crisp sharp primary stroke
        draw_line(top.x, top.y, apex.x, apex.y, chevron_thickness, color);
        draw_line(apex.x, apex.y, btm.x, btm.y, chevron_thickness, color);
    }
}

/// Track length (meters) drawn before the curve entry and after its exit in the pacenote icon.
const PACENOTE_LEAD_M: f32 = 12.0;
/// Number of centerline points in the pacenote icon.
const PACENOTE_SAMPLES: usize = 24;

/// Builds the pacenote icon line: the curve's own centerline, from `PACENOTE_LEAD_M` before
/// entry to `PACENOTE_LEAD_M` after exit, scaled so its longer side is `size` and centered on `center`.
///
/// The icon keeps the track's world orientation, so it matches the road on screen.
pub fn compute_pacenote_polyline(
    spline: &TrackSpline,
    curve: &TrackCurve,
    center: Vec2,
    size: f32,
) -> Vec<Vec2> {
    let mut span = curve.exit_distance - curve.entry_distance;
    if span < 0.0 {
        span += spline.total_length();
    }
    let start = curve.entry_distance - PACENOTE_LEAD_M;
    let length = span + 2.0 * PACENOTE_LEAD_M;

    let raw: Vec<Vec2> = (0..PACENOTE_SAMPLES)
        .map(|i| {
            let t = i as f32 / (PACENOTE_SAMPLES - 1) as f32;
            spline.sample_at_distance(start + length * t).point
        })
        .collect();

    let (min, max) = raw
        .iter()
        .fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let extent = (max - min).max_element().max(1e-3);
    let mid = (min + max) * 0.5;
    raw.iter().map(|p| center + (*p - mid) * (size / extent)).collect()
}

/// Renders the rally pacenote curve icon beside the player car.
///
/// A dark disc holds a drawing of the upcoming curve's shape with an arrowhead at the exit,
/// colored by the same scheme and urgency as the chevrons.
pub fn render_curve_pacenote(
    player_car: &Car,
    status: &CurveApproachStatus,
    spline: &TrackSpline,
    scheme: CurveColorScheme,
    current_zoom: f32,
    anim_time: f32,
    scale: f32,
    brightness: f32,
) {
    let base_alpha = compute_indicator_alpha(
        status.distance_to_entry,
        status.distance_to_apex,
        status.is_inside_curve,
        player_car.state.speed,
    );
    let alpha = (base_alpha * brightness).clamp(0.0, 1.0);

    if alpha <= 0.02 {
        return;
    }

    let degree = status.curve.degree.clamp(1, 5);
    let (color, _border_color) = compute_curve_colors(scheme, status.urgency, degree, alpha);

    // Pulse scale when in critical braking envelope
    let pulse_scale = if status.urgency >= 0.85 {
        1.0 + (anim_time * 9.0).sin().abs() * 0.10
    } else {
        1.0
    };

    let zoom = current_zoom.max(0.5);
    let size = (34.0 * pulse_scale * scale) / zoom;
    let plate_radius = size * 0.92;
    let thickness = (4.0 * pulse_scale * scale) / zoom;
    let outline = thickness + 3.0 / zoom;

    // Disc sits on the car's own left or right side, like the chevrons
    let side = curve_side_vector(player_car, status.curve.direction);
    let origin = player_car.state.position + Vec2::new(0.0, player_car.total_elevation());
    let center = origin + side * (curve_indicator_inner_clearance(player_car, zoom) + plate_radius);

    let points = compute_pacenote_polyline(spline, &status.curve, center, size);
    let n = points.len();
    if n < 2 {
        return;
    }

    let plate_col = Color::new(0.02, 0.03, 0.06, (0.55 * alpha).min(1.0));
    let shadow_col = Color::new(0.0, 0.0, 0.0, (0.80 * alpha).min(1.0));

    draw_circle(center.x, center.y, plate_radius, plate_col);

    // Arrowhead at the exit, pointing along the last segment
    let dir = (points[n - 1] - points[n - 2]).normalize_or_zero();
    let perp = Vec2::new(-dir.y, dir.x);
    let head_len = size * 0.28;
    let head_w = size * 0.18;
    let tip = points[n - 1] + dir * head_len * 0.6;
    let base = points[n - 1] - dir * head_len * 0.4;
    let pad = 1.5 / zoom;

    // 1. Dark outline under the line and arrowhead
    for pair in points.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, outline, shadow_col);
    }
    for p in &points {
        draw_circle(p.x, p.y, outline * 0.5, shadow_col);
    }
    let (t_o, l_o, r_o) = (
        tip + dir * pad * 2.0,
        base - dir * pad + perp * (head_w + pad * 1.5),
        base - dir * pad - perp * (head_w + pad * 1.5),
    );
    draw_triangle(mq_vec(t_o), mq_vec(l_o), mq_vec(r_o), shadow_col);

    // 2. Colored centerline with round joins, then the arrowhead
    for pair in points.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, thickness, color);
    }
    for p in &points {
        draw_circle(p.x, p.y, thickness * 0.5, color);
    }
    draw_triangle(mq_vec(tip), mq_vec(base + perp * head_w), mq_vec(base - perp * head_w), color);
}

/// Converts to macroquad's own `Vec2` (a different glam version) for `draw_triangle`.
#[inline]
fn mq_vec(v: Vec2) -> macroquad::math::Vec2 {
    macroquad::math::Vec2::new(v.x, v.y)
}
