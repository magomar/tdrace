use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::draw_line;
use serde::{Deserialize, Serialize};
use tdrace_core::physics::car::Car;
use tdrace_core::track::curve::{CurveApproachStatus, CurveDirection};
use tdrace_core::track::Track;

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

/// Calculates the display opacity for the curve indicator based on approach distance and apex traversal.
pub fn compute_indicator_alpha(
    distance_to_entry: f32,
    distance_to_apex: f32,
    is_inside_curve: bool,
) -> f32 {
    // Smooth fade in on approach (150m -> 130m)
    let approach_alpha = if is_inside_curve {
        1.0
    } else if distance_to_entry > 130.0 {
        ((150.0 - distance_to_entry) / 20.0).clamp(0.0, 1.0)
    } else {
        1.0
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

/// Computes the smart world-space position for curve alert arrows adjacent to the player car.
///
/// Places arrows to the left or right of the player car based on curve direction,
/// and searches for a vertical offset (moving slightly up or down) that avoids
/// overlaying the circuit track ribbon or nearby opponent vehicles.
pub fn compute_smart_curve_arrow_position(
    track: &Track,
    all_cars: &[Car],
    player_car: &Car,
    direction: CurveDirection,
    current_zoom: f32,
) -> Vec2 {
    let zoom = current_zoom.max(0.5);
    let car_pos = player_car.state.position;
    let elevation = player_car.total_elevation();

    // Horizontal offset: left or right of car with comfortable clearance
    let side_sign = match direction {
        CurveDirection::Left => -1.0,
        CurveDirection::Right => 1.0,
    };
    let lateral_dist = (38.0 / zoom).clamp(2.6, 6.0);
    let base_x = car_pos.x + side_sign * lateral_dist;
    let base_y = car_pos.y + elevation;

    // Vertical candidate shifts: test level (0.0), slightly up, slightly down, then further up/down
    let step_y = 16.0 / zoom;
    let y_shifts = [
        0.0,
        step_y,
        -step_y,
        2.0 * step_y,
        -2.0 * step_y,
        3.0 * step_y,
        -3.0 * step_y,
    ];

    let mut best_pos = Vec2::new(base_x, base_y);
    let mut best_penalty = f32::INFINITY;

    for dy in y_shifts {
        let cand = Vec2::new(base_x, base_y + dy);

        // 1. Circuit track avoidance penalty
        let proj = track.spline.project_point(cand);
        let track_penalty = if proj.is_on_track {
            // Heavily penalize covering the main asphalt racing ribbon
            150.0
        } else if proj.is_on_curb {
            // Lightly penalize curbs
            35.0
        } else {
            // Free terrain (grass, gravel, runoff) - optimal!
            0.0
        };

        // 2. Opponent car avoidance penalty
        let mut car_penalty = 0.0f32;
        let min_car_clearance = (28.0 / zoom).clamp(2.2, 4.5);
        for other in all_cars.iter().skip(1) {
            let dist = other.state.position.distance(cand);
            if dist < min_car_clearance {
                car_penalty += 180.0 * (1.0 - dist / min_car_clearance);
            }
        }

        // 3. Displacement penalty (prefer staying closer to car level if clear)
        let dist_penalty = dy.abs() * 1.5;

        let total_penalty = track_penalty + car_penalty + dist_penalty;
        if total_penalty < best_penalty {
            best_penalty = total_penalty;
            best_pos = cand;
        }
    }

    best_pos
}

/// Renders the simplified curve alert chevrons in world space adjacent to the player car.
///
/// Features no background box and no text labels — only anti-aliased, glowing vector chevrons
/// positioned left or right of the car, with smart up/down repositioning to avoid obscuring
/// the circuit ribbon or other vehicles.
pub fn render_curve_indicator(
    track: &Track,
    all_cars: &[Car],
    player_car: &Car,
    status: &CurveApproachStatus,
    scheme: CurveColorScheme,
    current_zoom: f32,
    anim_time: f32,
) {
    let alpha = compute_indicator_alpha(
        status.distance_to_entry,
        status.distance_to_apex,
        status.is_inside_curve,
    );

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

    // Compute smart position to the left or right of the car
    let arrow_center = compute_smart_curve_arrow_position(
        track,
        all_cars,
        player_car,
        status.curve.direction,
        zoom,
    );

    // Vector chevron dimensions in world units (scaled by 1.0 / zoom for fixed screen size)
    let chevron_w = (14.0 * pulse_scale) / zoom;
    let chevron_h = (22.0 * pulse_scale) / zoom;
    let chevron_thickness = (3.5 * pulse_scale) / zoom;
    let spacing = (16.0 * pulse_scale) / zoom;
    let shadow_offset = 1.6 / zoom;

    let num_chevrons = degree as usize;
    let total_w = (num_chevrons as f32 - 1.0) * spacing + chevron_w;
    let start_x = arrow_center.x - total_w * 0.5;
    let cy = arrow_center.y;

    let shadow_col = Color::new(0.0, 0.0, 0.0, 0.70 * alpha);
    let glow_col = Color::new(color.r, color.g, color.b, 0.35 * alpha);

    for i in 0..num_chevrons {
        let cx = start_x + (i as f32) * spacing;

        match status.curve.direction {
            CurveDirection::Left => {
                // Points Left: <
                let apex = Vec2::new(cx, cy);
                let top = Vec2::new(cx + chevron_w, cy + chevron_h * 0.5);
                let btm = Vec2::new(cx + chevron_w, cy - chevron_h * 0.5);

                // 1. Dark outer drop shadow
                draw_line(
                    top.x + shadow_offset,
                    top.y - shadow_offset,
                    apex.x + shadow_offset,
                    apex.y - shadow_offset,
                    chevron_thickness + 1.2 / zoom,
                    shadow_col,
                );
                draw_line(
                    apex.x + shadow_offset,
                    apex.y - shadow_offset,
                    btm.x + shadow_offset,
                    btm.y - shadow_offset,
                    chevron_thickness + 1.2 / zoom,
                    shadow_col,
                );

                // 2. Glowing ambient stroke
                draw_line(top.x, top.y, apex.x, apex.y, chevron_thickness * 1.8, glow_col);
                draw_line(apex.x, apex.y, btm.x, btm.y, chevron_thickness * 1.8, glow_col);

                // 3. Crisp sharp primary stroke
                draw_line(top.x, top.y, apex.x, apex.y, chevron_thickness, color);
                draw_line(apex.x, apex.y, btm.x, btm.y, chevron_thickness, color);
            }
            CurveDirection::Right => {
                // Points Right: >
                let apex = Vec2::new(cx + chevron_w, cy);
                let top = Vec2::new(cx, cy + chevron_h * 0.5);
                let btm = Vec2::new(cx, cy - chevron_h * 0.5);

                // 1. Dark outer drop shadow
                draw_line(
                    top.x + shadow_offset,
                    top.y - shadow_offset,
                    apex.x + shadow_offset,
                    apex.y - shadow_offset,
                    chevron_thickness + 1.2 / zoom,
                    shadow_col,
                );
                draw_line(
                    apex.x + shadow_offset,
                    apex.y - shadow_offset,
                    btm.x + shadow_offset,
                    btm.y - shadow_offset,
                    chevron_thickness + 1.2 / zoom,
                    shadow_col,
                );

                // 2. Glowing ambient stroke
                draw_line(top.x, top.y, apex.x, apex.y, chevron_thickness * 1.8, glow_col);
                draw_line(apex.x, apex.y, btm.x, btm.y, chevron_thickness * 1.8, glow_col);

                // 3. Crisp sharp primary stroke
                draw_line(top.x, top.y, apex.x, apex.y, chevron_thickness, color);
                draw_line(apex.x, apex.y, btm.x, btm.y, chevron_thickness, color);
            }
        }
    }
}
