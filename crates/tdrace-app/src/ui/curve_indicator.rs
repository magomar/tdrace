use macroquad::color::Color;
use macroquad::shapes::{draw_line, draw_rectangle, draw_rectangle_lines};
use serde::{Deserialize, Serialize};
use tdrace_core::physics::car::Car;
use tdrace_core::track::curve::{CurveApproachStatus, CurveDirection};

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::render::color::Palette;

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

/// Renders the Curve Approaching and Dynamic Braking Helper HUD widget.
pub fn render_curve_indicator(
    fonts: &Fonts,
    scaler: &UiScaler,
    center_x: f32,
    center_y: f32,
    status: &CurveApproachStatus,
    player_car: &Car,
    scheme: CurveColorScheme,
    anim_time: f32,
) {
    let distance_ahead = status.distance_to_entry;

    // Smooth fade in / out based on proximity
    // Fade in between 140m and 110m, full alpha inside curve, fade out past exit
    let alpha = if status.is_inside_curve {
        1.0
    } else if distance_ahead > 130.0 {
        ((150.0 - distance_ahead) / 20.0).clamp(0.0, 1.0)
    } else {
        1.0
    };

    if alpha <= 0.02 {
        return;
    }

    let degree = status.curve.degree.clamp(1, 5);
    let (color, border_color) = compute_curve_colors(scheme, status.urgency, degree, alpha);

    // Pulse intensity when in critical braking zone
    let is_critical = status.urgency >= 0.85;
    let pulse_scale = if is_critical {
        1.0 + (anim_time * 9.0).sin().abs() * 0.06
    } else {
        1.0
    };

    let card_w = scaler.s(280.0) * pulse_scale;
    let card_h = scaler.s(74.0) * pulse_scale;
    let card_x = center_x - card_w * 0.5;
    let card_y = center_y - card_h * 0.5;

    // Outer card backdrop with semi-transparent dark glass
    let bg_color = Color::new(0.04, 0.06, 0.10, 0.48 * alpha);
    let border_stroke = if is_critical { 2.5 } else { 1.5 };
    scaler.draw_glass_card(card_x, card_y, card_w, card_h, bg_color, border_color, border_stroke);

    // 1. Top Sub-badge: Direction + Degree name + Apex speed
    let severity_name = match degree {
        1 => "GENTLE",
        2 => "MILD",
        3 => "MEDIUM",
        4 => "SHARP",
        _ => "HAIRPIN",
    };
    let dir_name = status.curve.direction.as_str();
    let apex_speed_kmh = (status.curve.safe_apex_speed_mps * 3.6).round() as u32;

    let sub_title = format!("{} {}  •  APEX {} KM/H", severity_name, dir_name, apex_speed_kmh);
    fonts.draw_display_centered_with_shadow(
        &sub_title,
        center_x,
        card_y + scaler.s(16.0),
        scaler.font_s(11.0),
        Color::new(0.70, 0.78, 0.88, 0.95 * alpha),
        Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
        scaler.s(1.0),
    );

    // 2. Center: Draw 1 to 5 Vector Chevrons
    let chevron_w = scaler.s(18.0);
    let chevron_h = scaler.s(28.0);
    let chevron_thickness = scaler.s(4.5);
    let spacing = scaler.s(21.0);

    let num_chevrons = degree as usize;
    let total_chevrons_w = (num_chevrons as f32 - 1.0) * spacing + chevron_w;
    let chevrons_start_x = center_x - total_chevrons_w * 0.5;
    let chevrons_y = card_y + scaler.s(39.0);

    for i in 0..num_chevrons {
        let cx = chevrons_start_x + (i as f32) * spacing;
        let shadow_offset = scaler.s(1.5);
        let shadow_col = Color::new(0.0, 0.0, 0.0, 0.65 * alpha);

        match status.curve.direction {
            CurveDirection::Left => {
                // Points Left: <
                // Apex at (cx, chevrons_y)
                let apex = (cx, chevrons_y);
                let top = (cx + chevron_w, chevrons_y - chevron_h * 0.5);
                let btm = (cx + chevron_w, chevrons_y + chevron_h * 0.5);

                // Drop shadow
                draw_line(top.0 + shadow_offset, top.1 + shadow_offset, apex.0 + shadow_offset, apex.1 + shadow_offset, chevron_thickness, shadow_col);
                draw_line(apex.0 + shadow_offset, apex.1 + shadow_offset, btm.0 + shadow_offset, btm.1 + shadow_offset, chevron_thickness, shadow_col);

                // Main strokes
                draw_line(top.0, top.1, apex.0, apex.1, chevron_thickness, color);
                draw_line(apex.0, apex.1, btm.0, btm.1, chevron_thickness, color);
            }
            CurveDirection::Right => {
                // Points Right: >
                // Apex at (cx + chevron_w, chevrons_y)
                let apex = (cx + chevron_w, chevrons_y);
                let top = (cx, chevrons_y - chevron_h * 0.5);
                let btm = (cx, chevrons_y + chevron_h * 0.5);

                // Drop shadow
                draw_line(top.0 + shadow_offset, top.1 + shadow_offset, apex.0 + shadow_offset, apex.1 + shadow_offset, chevron_thickness, shadow_col);
                draw_line(apex.0 + shadow_offset, apex.1 + shadow_offset, btm.0 + shadow_offset, btm.1 + shadow_offset, chevron_thickness, shadow_col);

                // Main strokes
                draw_line(top.0, top.1, apex.0, apex.1, chevron_thickness, color);
                draw_line(apex.0, apex.1, btm.0, btm.1, chevron_thickness, color);
            }
        }
    }

    // 3. Bottom Row: Distance Countdown + Dynamic Action Badge
    let dist_str = if status.is_inside_curve {
        "IN APEX".to_string()
    } else {
        format!("{}M", distance_ahead.max(0.0).round() as u32)
    };

    let badge_h = scaler.s(16.0);
    let badge_y = card_y + card_h - badge_h - scaler.s(6.0);

    // Draw distance on left side
    fonts.draw_display_with_shadow(
        &dist_str,
        card_x + scaler.s(18.0),
        badge_y + scaler.s(12.5),
        scaler.font_s(14.0),
        Color::new(1.0, 1.0, 1.0, 0.95 * alpha),
        Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
        scaler.s(1.0),
    );

    // Action status badge on right side
    let (badge_text, badge_bg, badge_fg) = if is_critical {
        if player_car.state.speed > status.curve.safe_apex_speed_mps {
            ("BRAKE HARD!", Color::new(0.85, 0.12, 0.18, 0.55 * alpha), Palette::WHITE)
        } else {
            ("SPEED SAFE", Color::new(0.12, 0.55, 0.28, 0.52 * alpha), Color::new(0.80, 1.0, 0.85, alpha))
        }
    } else if status.urgency >= 0.40 {
        ("PREPARE BRAKE", Color::new(0.65, 0.48, 0.08, 0.52 * alpha), Color::new(1.0, 0.95, 0.70, alpha))
    } else if player_car.state.speed <= status.curve.safe_apex_speed_mps + 1.5 {
        ("SPEED SAFE", Color::new(0.12, 0.55, 0.28, 0.52 * alpha), Color::new(0.80, 1.0, 0.85, alpha))
    } else {
        ("APPROACHING", Color::new(0.15, 0.20, 0.28, 0.52 * alpha), Color::new(0.70, 0.78, 0.88, alpha))
    };

    let badge_w = scaler.s(105.0);
    let badge_x = card_x + card_w - badge_w - scaler.s(16.0);

    draw_rectangle(badge_x, badge_y, badge_w, badge_h, badge_bg);
    draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.0, border_color);

    fonts.draw_display_centered_with_shadow(
        badge_text,
        badge_x + badge_w * 0.5,
        badge_y + scaler.s(11.5),
        scaler.font_s(10.5),
        badge_fg,
        Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
        scaler.s(1.0),
    );
}
