//! Tactical Hologram Cockpit Telemetry HUD (Spec 079).
//!
//! Provides a pure graphical cockpit instrument depicting proportional chassis
//! geometry (`ChassisSkeleton`), authentic powertrain placement and damage (`EnginePlacement`),
//! decoupled outboard tires with thermal casing and vertical tread drain, dynamic
//! Ackermann front wheel steering articulation, thick perimeter impact indicators,
//! and dual switchable suspension modes:
//! - Variant B: Kinematic Linkages & Structural Damage (Ctrl + 1)
//! - Variant C: Dynamic Telemetry & Damper Travel (Ctrl + 2)
//!
//! Zero microscopic text labels or percentages inside the HUD glass bezel.

use macroquad::color::Color;
use macroquad::math::Vec2;
use macroquad::shapes::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle,
};
use serde::{Deserialize, Serialize};

use cabinet::ui::font::Fonts;
use cabinet::ui::scaler::UiScaler;
use wheelbase::car::Car;
use wheelbase::config::{EnginePlacement, SuspensionArchetype};

use crate::hud::widgets::render_compound_legend;
use crate::render::color::Palette;

/// Cockpit telemetry HUD display mode (Spec 079).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CockpitTelemetryMode {
    /// Variant B: Authentic mechanical linkages with buckled fracture paths & camber skew.
    #[default]
    KinematicDamage,
    /// Variant C: 4-corner damper stroke capsules with dynamic Fz load transfer glow.
    DynamicTelemetry,
}

impl CockpitTelemetryMode {
    /// Concise uppercase label for the HUD header pill.
    pub fn label(&self) -> &'static str {
        match self {
            Self::KinematicDamage => "KINEMATICS",
            Self::DynamicTelemetry => "DYNAMICS",
        }
    }
}

/// Normalized HUD layout coordinates computed from physical chassis geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChassisHudGeometry {
    pub scale: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub front_axle_y: f32,
    pub rear_axle_y: f32,
    pub nose_y: f32,
    pub tail_y: f32,
    pub half_body_w: f32,
    pub half_track_w: f32,
    pub wheel_w: f32,
    pub wheel_h: f32,
    pub outboard_left_x: f32,
    pub outboard_right_x: f32,
}

impl ChassisHudGeometry {
    /// Computes proportional 2D HUD dimensions from the vehicle's physical chassis dimensions.
    ///
    /// Scales wheelbase, front/rear overhangs, and body width proportionally to fit
    /// comfortably within the HUD glass card without clipping.
    pub fn compute(
        box_w: f32,
        box_h: f32,
        origin_x: f32,
        origin_y: f32,
        car: &Car,
        scale_factor: f32,
    ) -> Self {
        let cx = origin_x + box_w * 0.5;
        let cy = origin_y + box_h * 0.48;

        let total_len = (car.config.wheelbase
            + car.config.chassis.front_overhang
            + car.config.chassis.rear_overhang)
            .max(0.8);

        // Target occupying ~58% of the box height to accommodate bottom legend
        let scale = (box_h * 0.58) / total_len;

        let front_axle_y = cy - (car.config.wheelbase * 0.5 * scale);
        let rear_axle_y = cy + (car.config.wheelbase * 0.5 * scale);
        let nose_y = front_axle_y - (car.config.chassis.front_overhang * scale);
        let tail_y = rear_axle_y + (car.config.chassis.rear_overhang * scale);

        let half_body_w = (car.config.chassis.body_width * 0.5 * scale).max(8.0 * scale_factor);
        let half_track_w = (car.config.track_width * 0.5 * scale).max(10.0 * scale_factor);

        let is_kart = car.config.suspension.front.archetype == SuspensionArchetype::RigidKart
            || car.config.wheelbase < 1.35;

        let wheel_w = if is_kart { 11.5 * scale_factor } else { 13.5 * scale_factor };
        let wheel_h = if is_kart { 21.0 * scale_factor } else { 26.5 * scale_factor };
        let clearance = if is_kart { 3.5 * scale_factor } else { 5.0 * scale_factor };

        let outboard_left_x = cx - half_body_w - clearance - wheel_w;
        let outboard_right_x = cx + half_body_w + clearance;

        Self {
            scale,
            center_x: cx,
            center_y: cy,
            front_axle_y,
            rear_axle_y,
            nose_y,
            tail_y,
            half_body_w,
            half_track_w,
            wheel_w,
            wheel_h,
            outboard_left_x,
            outboard_right_x,
        }
    }
}

/// Computes inner and outer steered wheel rotation angles using authentic Ackermann differential geometry.
///
/// Under right turn (positive steer), the right wheel is inner (sharper 1.15x)
/// and the left wheel is outer (shallower 0.88x).
/// Under left turn (negative steer), the left wheel is inner (sharper 1.15x)
/// and the right wheel is outer (shallower 0.88x).
#[inline]
pub fn compute_ackermann_steer_angles(steer_angle_rad: f32) -> (f32, f32) {
    if steer_angle_rad > 0.001 {
        let outer = steer_angle_rad * 0.88;
        let inner = steer_angle_rad * 1.15;
        (outer, inner) // (FL, FR)
    } else if steer_angle_rad < -0.001 {
        let inner = steer_angle_rad * 1.15;
        let outer = steer_angle_rad * 0.88;
        (inner, outer) // (FL, FR)
    } else {
        (0.0, 0.0)
    }
}

/// Maps tire temperature in Celsius to high-visibility HUD thermal colors.
#[inline]
pub fn tire_temp_to_color(temp_celsius: f32) -> Color {
    if temp_celsius < 60.0 {
        Palette::NEON_CYAN // Cold sky blue
    } else if temp_celsius <= 100.0 {
        Palette::NEON_GREEN // Optimal emerald
    } else if temp_celsius <= 115.0 {
        Palette::NEON_GOLD // Warm amber
    } else {
        Palette::RED // Overheated red
    }
}

/// Helper: rotates a 2D point around an arbitrary pivot.
#[inline]
fn rotate_point(pt: Vec2, pivot: Vec2, cos_t: f32, sin_t: f32) -> Vec2 {
    let dx = pt.x - pivot.x;
    let dy = pt.y - pivot.y;
    Vec2::new(
        pivot.x + dx * cos_t - dy * sin_t,
        pivot.y + dx * sin_t + dy * cos_t,
    )
}

/// Helper: draws an oriented (rotated) rectangle with optional fill and outline.
fn draw_oriented_quad(
    center: Vec2,
    hw: f32,
    hh: f32,
    theta: f32,
    fill_col: Color,
    border_col: Option<(Color, f32)>,
) {
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    let p0 = rotate_point(Vec2::new(center.x - hw, center.y - hh), center, cos_t, sin_t);
    let p1 = rotate_point(Vec2::new(center.x + hw, center.y - hh), center, cos_t, sin_t);
    let p2 = rotate_point(Vec2::new(center.x + hw, center.y + hh), center, cos_t, sin_t);
    let p3 = rotate_point(Vec2::new(center.x - hw, center.y + hh), center, cos_t, sin_t);

    draw_triangle(p0, p1, p2, fill_col);
    draw_triangle(p0, p2, p3, fill_col);

    if let Some((b_col, thickness)) = border_col {
        draw_line(p0.x, p0.y, p1.x, p1.y, thickness, b_col);
        draw_line(p1.x, p1.y, p2.x, p2.y, thickness, b_col);
        draw_line(p2.x, p2.y, p3.x, p3.y, thickness, b_col);
        draw_line(p3.x, p3.y, p0.x, p0.y, thickness, b_col);
    }
}

/// Draws an integrated-drain tire wheel assembly.
///
/// Features:
/// - 100% visible thermal casing border colored by tire temperature.
/// - Vertical tread drain fill (remaining tread ratio [0.0..1.0]).
/// - Depleted void alert tint if tread is depleted.
/// - 50% midpoint graduation tick marks on tire side walls.
/// - Rotatable around center for dynamic Ackermann steering articulation and camber skew.
fn draw_integrated_tire(
    center: Vec2,
    wheel_w: f32,
    wheel_h: f32,
    temp_celsius: f32,
    wear: f32,
    rotation_rad: f32,
    border_col: Color,
    scaler: &UiScaler,
) {
    let hw = wheel_w * 0.5;
    let hh = wheel_h * 0.5;
    let pad = scaler.s(2.0);
    let inner_hw = hw - pad;
    let inner_hh = hh - pad;
    let inner_h = inner_hh * 2.0;

    let thermal_col = tire_temp_to_color(temp_celsius);
    let tread_ratio = (1.0 - wear).clamp(0.0, 1.0);
    let fill_h = inner_h * tread_ratio;
    let empty_h = inner_h - fill_h;

    // 1. Outer casing background & prominent compound-colored border (Spec 089)
    let bg_col = Color::new(0.04, 0.06, 0.10, 0.95);
    draw_oriented_quad(
        center,
        hw,
        hh,
        rotation_rad,
        bg_col,
        Some((border_col, scaler.s(2.2))),
    );

    let cos_t = rotation_rad.cos();
    let sin_t = rotation_rad.sin();

    // 2. Depleted void alert (subtle red cavity if worn)
    if empty_h > scaler.s(1.0) {
        let void_local_y = -inner_hh + empty_h * 0.5;
        let void_center = rotate_point(
            Vec2::new(center.x, center.y + void_local_y),
            center,
            cos_t,
            sin_t,
        );
        let void_col = Color::new(1.0, 0.24, 0.0, 0.22);
        draw_oriented_quad(
            void_center,
            inner_hw,
            empty_h * 0.5,
            rotation_rad,
            void_col,
            None,
        );
    }

    // 3. Inner tread fill (drains downward, remaining rubber anchored to bottom)
    if fill_h > scaler.s(0.5) {
        let fill_local_y = inner_hh - fill_h * 0.5;
        let fill_center = rotate_point(
            Vec2::new(center.x, center.y + fill_local_y),
            center,
            cos_t,
            sin_t,
        );
        let mut fill_col = thermal_col;
        fill_col.a = 0.85;
        draw_oriented_quad(
            fill_center,
            inner_hw,
            fill_h * 0.5,
            rotation_rad,
            fill_col,
            None,
        );
    }

    // 4. 50% midpoint graduation tick line (left and right edges, NO numeric labels)
    let tick_len = scaler.s(2.5);
    let left_tick_start = rotate_point(Vec2::new(center.x - hw, center.y), center, cos_t, sin_t);
    let left_tick_end = rotate_point(
        Vec2::new(center.x - hw + tick_len, center.y),
        center,
        cos_t,
        sin_t,
    );
    let right_tick_start = rotate_point(Vec2::new(center.x + hw, center.y), center, cos_t, sin_t);
    let right_tick_end = rotate_point(
        Vec2::new(center.x + hw - tick_len, center.y),
        center,
        cos_t,
        sin_t,
    );

    let tick_col = Color::new(1.0, 1.0, 1.0, 0.70);
    draw_line(
        left_tick_start.x,
        left_tick_start.y,
        left_tick_end.x,
        left_tick_end.y,
        scaler.s(1.0),
        tick_col,
    );
    draw_line(
        right_tick_start.x,
        right_tick_start.y,
        right_tick_end.x,
        right_tick_end.y,
        scaler.s(1.0),
        tick_col,
    );
}

/// Draws procedural chassis bodywork silhouette and cockpit cell.
fn draw_chassis_silhouette(geo: &ChassisHudGeometry, car: &Car, scaler: &UiScaler) {
    let cx = geo.center_x;
    let cy = geo.center_y;
    let body_col = Color::new(0.05, 0.16, 0.25, 0.42);
    let stroke_col = Color::new(0.0, 0.88, 1.0, 0.55);
    let stroke_w = scaler.s(2.2);

    let is_kart = car.config.suspension.front.archetype == SuspensionArchetype::RigidKart
        || car.config.wheelbase < 1.35;

    if is_kart {
        // Kart: narrow nose cone, wide side pods, square rear bumper
        let pts = [
            Vec2::new(cx - geo.half_body_w * 0.65, geo.nose_y),
            Vec2::new(cx + geo.half_body_w * 0.65, geo.nose_y),
            Vec2::new(cx + geo.half_body_w, geo.front_axle_y + scaler.s(4.0)),
            Vec2::new(cx + geo.half_body_w, geo.rear_axle_y - scaler.s(4.0)),
            Vec2::new(cx + geo.half_body_w * 0.70, geo.tail_y),
            Vec2::new(cx - geo.half_body_w * 0.70, geo.tail_y),
            Vec2::new(cx - geo.half_body_w, geo.rear_axle_y - scaler.s(4.0)),
            Vec2::new(cx - geo.half_body_w, geo.front_axle_y + scaler.s(4.0)),
        ];
        draw_polygon(&pts, body_col, stroke_col, stroke_w);

        // Steering column & wheel hub
        draw_circle(cx, cy, scaler.s(6.5), Color::new(0.96, 0.62, 0.04, 0.45));
        draw_circle_lines(cx, cy, scaler.s(6.5), scaler.s(1.6), Palette::NEON_GOLD);
        draw_line(
            cx - scaler.s(6.0),
            cy - scaler.s(4.0),
            cx + scaler.s(6.0),
            cy - scaler.s(4.0),
            scaler.s(2.0),
            Palette::WHITE,
        );
    } else {
        // Sports/GT/Stock/Rally/Prototype: contoured sports car bodywork
        let pts = [
            Vec2::new(cx - geo.half_body_w * 0.72, geo.nose_y),
            Vec2::new(cx + geo.half_body_w * 0.72, geo.nose_y),
            Vec2::new(cx + geo.half_body_w * 0.94, geo.front_axle_y + scaler.s(6.0)),
            Vec2::new(cx + geo.half_body_w * 0.86, cy),
            Vec2::new(cx + geo.half_body_w, geo.rear_axle_y + scaler.s(8.0)),
            Vec2::new(cx + geo.half_body_w * 0.78, geo.tail_y),
            Vec2::new(cx - geo.half_body_w * 0.78, geo.tail_y),
            Vec2::new(cx - geo.half_body_w, geo.rear_axle_y + scaler.s(8.0)),
            Vec2::new(cx - geo.half_body_w * 0.86, cy),
            Vec2::new(cx - geo.half_body_w * 0.94, geo.front_axle_y + scaler.s(6.0)),
        ];
        draw_polygon(&pts, body_col, stroke_col, stroke_w);

        // Center Cockpit Cell
        let cabin_w = geo.half_body_w * 0.80;
        let cabin_h = geo.scale * 0.48;
        let cabin_x = cx - cabin_w * 0.5;
        let cabin_y = cy - cabin_h * 0.5;
        draw_rectangle(cabin_x, cabin_y, cabin_w, cabin_h, Color::new(0.06, 0.09, 0.16, 0.78));
        draw_rectangle_lines(
            cabin_x,
            cabin_y,
            cabin_w,
            cabin_h,
            scaler.s(1.4),
            Color::new(0.0, 0.90, 1.0, 0.40),
        );

        // Rear Wing / Spoiler Line
        draw_line(
            cx - geo.half_body_w * 0.88,
            geo.tail_y - scaler.s(2.0),
            cx + geo.half_body_w * 0.88,
            geo.tail_y - scaler.s(2.0),
            scaler.s(2.5),
            Color::new(0.0, 0.90, 1.0, 0.65),
        );
    }
}

/// Helper: draws a convex polygon with fill and outline.
fn draw_polygon(pts: &[Vec2], fill_col: Color, stroke_col: Color, stroke_w: f32) {
    if pts.len() < 3 {
        return;
    }
    // Triangle fan from pts[0]
    for i in 1..pts.len() - 1 {
        draw_triangle(pts[0], pts[i], pts[i + 1], fill_col);
    }
    // Outlines
    for i in 0..pts.len() {
        let next = (i + 1) % pts.len();
        draw_line(pts[i].x, pts[i].y, pts[next].x, pts[next].y, stroke_w, stroke_col);
    }
}

/// Draws thick, high-visibility 3-layer perimeter impact fracture indicators.
fn draw_perimeter_impact_indicators(
    geo: &ChassisHudGeometry,
    chassis_health: f32,
    scaler: &UiScaler,
) {
    if chassis_health >= 0.98 {
        return;
    }

    let cx = geo.center_x;

    // Severity color
    let (col, alpha_pulse) = if chassis_health <= 0.45 {
        let pulse = 0.6 + 0.4 * (macroquad::time::get_time() as f32 * 7.0).sin().abs();
        (Palette::RED, pulse)
    } else {
        (Palette::NEON_GOLD, 1.0)
    };

    // 1. Front Nose Impact Bar (Active if health < 0.90)
    if chassis_health < 0.90 {
        let p_left = Vec2::new(cx - geo.half_body_w * 0.72, geo.nose_y);
        let p_right = Vec2::new(cx + geo.half_body_w * 0.72, geo.nose_y);

        // Layer 1: Aura
        let mut aura_col = col;
        aura_col.a = 0.32 * alpha_pulse;
        draw_line(p_left.x, p_left.y, p_right.x, p_right.y, scaler.s(12.0), aura_col);

        // Layer 2: Fracture Bar
        let mut bar_col = col;
        bar_col.a = 0.90 * alpha_pulse;
        draw_line(p_left.x, p_left.y, p_right.x, p_right.y, scaler.s(6.0), bar_col);

        // Layer 3: High-Visibility White Core
        let core_left = Vec2::new(cx - geo.half_body_w * 0.52, geo.nose_y);
        let core_right = Vec2::new(cx + geo.half_body_w * 0.52, geo.nose_y);
        let white_core = Color::new(1.0, 1.0, 1.0, 0.85 * alpha_pulse);
        draw_line(core_left.x, core_left.y, core_right.x, core_right.y, scaler.s(2.2), white_core);
    }

    // 2. Rear Tail Impact Bar (Active if health < 0.70)
    if chassis_health < 0.70 {
        let p_left = Vec2::new(cx - geo.half_body_w * 0.75, geo.tail_y);
        let p_right = Vec2::new(cx + geo.half_body_w * 0.75, geo.tail_y);

        let mut aura_col = col;
        aura_col.a = 0.30 * alpha_pulse;
        draw_line(p_left.x, p_left.y, p_right.x, p_right.y, scaler.s(11.0), aura_col);

        let mut bar_col = col;
        bar_col.a = 0.85 * alpha_pulse;
        draw_line(p_left.x, p_left.y, p_right.x, p_right.y, scaler.s(5.5), bar_col);

        let core_left = Vec2::new(cx - geo.half_body_w * 0.55, geo.tail_y);
        let core_right = Vec2::new(cx + geo.half_body_w * 0.55, geo.tail_y);
        let white_core = Color::new(1.0, 1.0, 1.0, 0.80 * alpha_pulse);
        draw_line(core_left.x, core_left.y, core_right.x, core_right.y, scaler.s(2.0), white_core);
    }

    // 3. Flank Impact Bars (Active if health < 0.55)
    if chassis_health < 0.55 {
        let fl_y1 = geo.front_axle_y + scaler.s(10.0);
        let fl_y2 = geo.rear_axle_y - scaler.s(8.0);
        let left_x = cx - geo.half_body_w * 0.88;

        let mut aura_col = col;
        aura_col.a = 0.30 * alpha_pulse;
        draw_line(left_x, fl_y1, left_x, fl_y2, scaler.s(10.0), aura_col);

        let mut bar_col = col;
        bar_col.a = 0.85 * alpha_pulse;
        draw_line(left_x, fl_y1, left_x, fl_y2, scaler.s(5.0), bar_col);

        let white_core = Color::new(1.0, 1.0, 1.0, 0.80 * alpha_pulse);
        draw_line(left_x, fl_y1 + scaler.s(6.0), left_x, fl_y2 - scaler.s(6.0), scaler.s(1.8), white_core);
    }
}

/// Draws authentic powertrain engine block based on physical placement and health status.
fn draw_powertrain_block(geo: &ChassisHudGeometry, car: &Car, scaler: &UiScaler) {
    let cx = geo.center_x;
    let cy = geo.center_y;

    let is_kart = car.config.suspension.front.archetype == SuspensionArchetype::RigidKart
        || car.config.wheelbase < 1.35;

    // Resolve spatial anchor based on engine_placement
    let (eng_x, eng_y) = match car.config.engine_placement {
        EnginePlacement::FrontEngine => (cx, geo.front_axle_y + (geo.scale * 0.30)),
        EnginePlacement::MidEngine => {
            if is_kart {
                (cx + geo.half_body_w * 0.40, cy + scaler.s(4.0))
            } else {
                (cx, cy + (geo.scale * 0.15))
            }
        }
        EnginePlacement::RearEngine => (cx, geo.rear_axle_y + (geo.scale * 0.35)),
    };

    let eng_w = scaler.s(36.0);
    let eng_h = scaler.s(26.0);
    let eng_rect_x = eng_x - eng_w * 0.5;
    let eng_rect_y = eng_y - eng_h * 0.5;

    let health = car.state.engine_health;
    let (eng_col, pulse_alpha) = if health > 0.75 {
        (Palette::NEON_CYAN, 1.0)
    } else if health > 0.40 {
        (Palette::NEON_GOLD, 1.0)
    } else {
        let pulse = 0.4 + 0.6 * (macroquad::time::get_time() as f32 * 6.0).sin().abs();
        (Palette::RED, pulse)
    };

    // Engine block outer shell
    let mut bg = Color::new(0.06, 0.09, 0.15, 0.95);
    bg.a *= pulse_alpha;
    draw_rectangle(eng_rect_x, eng_rect_y, eng_w, eng_h, bg);

    let mut border_col = eng_col;
    border_col.a *= pulse_alpha;
    draw_rectangle_lines(eng_rect_x, eng_rect_y, eng_w, eng_h, scaler.s(2.2), border_col);

    // Authentic internal mechanical layout
    match car.config.engine_placement {
        EnginePlacement::FrontEngine => {
            // Front V8 / Inline: V-chevron intake and center crank line
            let v_left_start = Vec2::new(eng_rect_x + scaler.s(6.0), eng_rect_y + scaler.s(6.0));
            let v_left_mid = Vec2::new(eng_rect_x + scaler.s(13.0), eng_rect_y + scaler.s(13.0));
            let v_left_end = Vec2::new(eng_rect_x + scaler.s(6.0), eng_rect_y + scaler.s(20.0));
            draw_line(v_left_start.x, v_left_start.y, v_left_mid.x, v_left_mid.y, scaler.s(1.6), border_col);
            draw_line(v_left_mid.x, v_left_mid.y, v_left_end.x, v_left_end.y, scaler.s(1.6), border_col);

            let v_right_start = Vec2::new(eng_rect_x + scaler.s(30.0), eng_rect_y + scaler.s(6.0));
            let v_right_mid = Vec2::new(eng_rect_x + scaler.s(23.0), eng_rect_y + scaler.s(13.0));
            let v_right_end = Vec2::new(eng_rect_x + scaler.s(30.0), eng_rect_y + scaler.s(20.0));
            draw_line(v_right_start.x, v_right_start.y, v_right_mid.x, v_right_mid.y, scaler.s(1.6), border_col);
            draw_line(v_right_mid.x, v_right_mid.y, v_right_end.x, v_right_end.y, scaler.s(1.6), border_col);

            draw_circle(eng_x, eng_y, scaler.s(3.5), border_col);
        }
        EnginePlacement::RearEngine => {
            // Rear Flat-6: opposed horizontal cylinder heads + central crankshaft
            let head_w = scaler.s(8.0);
            let head_h = scaler.s(14.0);
            draw_rectangle(
                eng_rect_x + scaler.s(4.0),
                eng_rect_y + scaler.s(6.0),
                head_w,
                head_h,
                Color::new(border_col.r, border_col.g, border_col.b, 0.45 * pulse_alpha),
            );
            draw_rectangle(
                eng_rect_x + eng_w - scaler.s(4.0) - head_w,
                eng_rect_y + scaler.s(6.0),
                head_w,
                head_h,
                Color::new(border_col.r, border_col.g, border_col.b, 0.45 * pulse_alpha),
            );
            draw_circle_lines(eng_x, eng_y, scaler.s(4.8), scaler.s(1.6), border_col);
            draw_circle(eng_x, eng_y, scaler.s(2.2), border_col);
        }
        EnginePlacement::MidEngine => {
            // Mid-Engine / Kart: cylinder array
            let cyl_r = scaler.s(2.4);
            let offsets_x = [scaler.s(10.0), scaler.s(18.0), scaler.s(26.0)];
            for &ox in &offsets_x {
                draw_circle(eng_rect_x + ox, eng_rect_y + scaler.s(10.0), cyl_r, border_col);
                draw_circle(eng_rect_x + ox, eng_rect_y + scaler.s(16.0), cyl_r, border_col);
            }
        }
    }
}

/// Draws mechanical steering rack and tie-rods articulating with steered front wheel hubs.
fn draw_steering_rack(
    geo: &ChassisHudGeometry,
    steer_fl: f32,
    steer_fr: f32,
    scaler: &UiScaler,
) {
    let cx = geo.center_x;
    let rack_y = geo.front_axle_y - scaler.s(9.0);
    let avg_steer = (steer_fl + steer_fr) * 0.5;
    let rack_offset_x = (avg_steer / 0.50).clamp(-1.0, 1.0) * scaler.s(4.5);

    let rack_left = cx - geo.half_body_w * 0.45 + rack_offset_x;
    let rack_right = cx + geo.half_body_w * 0.45 + rack_offset_x;

    let fl_hub_x = geo.outboard_left_x + geo.wheel_w;
    let fr_hub_x = geo.outboard_right_x;

    let fl_tie_y = geo.front_axle_y - (steer_fl * scaler.s(10.0));
    let fr_tie_y = geo.front_axle_y + (steer_fr * scaler.s(10.0));

    let rack_col = Color::new(0.0, 0.90, 1.0, 0.85);
    let tie_col = Color::new(0.0, 0.90, 1.0, 0.70);

    // Central rack bar
    draw_line(rack_left, rack_y, rack_right, rack_y, scaler.s(2.4), rack_col);

    // Left and right articulating tie-rods
    draw_line(rack_left, rack_y, fl_hub_x, fl_tie_y, scaler.s(1.8), tie_col);
    draw_line(rack_right, rack_y, fr_hub_x, fr_tie_y, scaler.s(1.8), tie_col);

    // Hub steering knuckle pivots
    draw_circle(fl_hub_x, fl_tie_y, scaler.s(2.2), rack_col);
    draw_circle(fr_hub_x, fr_tie_y, scaler.s(2.2), rack_col);
}

/// Draws Variant B: Authentic Kinematic Architecture & Structural Damage Linkages.
fn draw_variant_b_kinematics(geo: &ChassisHudGeometry, car: &Car, scaler: &UiScaler) {
    let cx = geo.center_x;
    let fl_hub_x = geo.outboard_left_x + geo.wheel_w;
    let fr_hub_x = geo.outboard_right_x;
    let rl_hub_x = geo.outboard_left_x + geo.wheel_w;
    let rr_hub_x = geo.outboard_right_x;

    let fl_dmg = car.state.suspension_health[0] < 0.80;
    let fr_dmg = car.state.suspension_health[1] < 0.80;
    let rl_dmg = car.state.suspension_health[2] < 0.80;
    let rr_dmg = car.state.suspension_health[3] < 0.80;

    let norm_col = Palette::NEON_CYAN;
    let dmg_col = Palette::RED;

    match car.config.suspension.front.archetype {
        SuspensionArchetype::PushrodInboard => {
            // Front Inboard Bell-Cranks & Diagonal Pushrods
            let left_rocker = Vec2::new(cx - scaler.s(12.0), geo.front_axle_y - scaler.s(8.0));
            let right_rocker = Vec2::new(cx + scaler.s(12.0), geo.front_axle_y - scaler.s(8.0));

            // FL Pushrod
            if fl_dmg {
                draw_buckled_path(
                    Vec2::new(fl_hub_x, geo.front_axle_y + scaler.s(3.0)),
                    left_rocker,
                    scaler.s(4.0),
                    dmg_col,
                );
            } else {
                draw_line(
                    fl_hub_x,
                    geo.front_axle_y + scaler.s(3.0),
                    left_rocker.x,
                    left_rocker.y,
                    scaler.s(3.2),
                    norm_col,
                );
            }

            // FR Pushrod
            if fr_dmg {
                draw_buckled_path(
                    Vec2::new(fr_hub_x, geo.front_axle_y + scaler.s(3.0)),
                    right_rocker,
                    scaler.s(4.0),
                    dmg_col,
                );
            } else {
                draw_line(
                    fr_hub_x,
                    geo.front_axle_y + scaler.s(3.0),
                    right_rocker.x,
                    right_rocker.y,
                    scaler.s(3.2),
                    norm_col,
                );
            }

            // Inboard dampers
            let damp_w = scaler.s(16.0);
            let damp_h = scaler.s(5.0);
            draw_rectangle(
                cx - scaler.s(20.0),
                geo.front_axle_y - scaler.s(13.0),
                damp_w,
                damp_h,
                Color::new(0.96, 0.62, 0.04, 0.85),
            );
            draw_rectangle(
                cx + scaler.s(4.0),
                geo.front_axle_y - scaler.s(13.0),
                damp_w,
                damp_h,
                Color::new(0.96, 0.62, 0.04, 0.85),
            );

            // Rear Pushrods
            let rear_left_rocker = Vec2::new(cx - scaler.s(12.0), geo.rear_axle_y + scaler.s(8.0));
            let rear_right_rocker = Vec2::new(cx + scaler.s(12.0), geo.rear_axle_y + scaler.s(8.0));

            if rl_dmg {
                draw_buckled_path(
                    Vec2::new(rl_hub_x, geo.rear_axle_y - scaler.s(3.0)),
                    rear_left_rocker,
                    scaler.s(4.0),
                    dmg_col,
                );
            } else {
                draw_line(
                    rl_hub_x,
                    geo.rear_axle_y - scaler.s(3.0),
                    rear_left_rocker.x,
                    rear_left_rocker.y,
                    scaler.s(3.0),
                    norm_col,
                );
            }

            if rr_dmg {
                draw_buckled_path(
                    Vec2::new(rr_hub_x, geo.rear_axle_y - scaler.s(3.0)),
                    rear_right_rocker,
                    scaler.s(4.0),
                    dmg_col,
                );
            } else {
                draw_line(
                    rr_hub_x,
                    geo.rear_axle_y - scaler.s(3.0),
                    rear_right_rocker.x,
                    rear_right_rocker.y,
                    scaler.s(3.0),
                    norm_col,
                );
            }
        }
        SuspensionArchetype::SolidLiveAxle | SuspensionArchetype::RigidKart => {
            // Front steering track rods
            if fl_dmg {
                draw_buckled_path(
                    Vec2::new(cx - scaler.s(5.0), geo.front_axle_y),
                    Vec2::new(fl_hub_x, geo.front_axle_y),
                    scaler.s(3.8),
                    dmg_col,
                );
            } else {
                draw_line(
                    cx - scaler.s(5.0),
                    geo.front_axle_y,
                    fl_hub_x,
                    geo.front_axle_y,
                    scaler.s(3.0),
                    norm_col,
                );
            }
            draw_line(
                cx + scaler.s(5.0),
                geo.front_axle_y,
                fr_hub_x,
                geo.front_axle_y,
                scaler.s(3.0),
                norm_col,
            );

            // Rear Continuous Solid Beam Axle (Thick 5.5px)
            let beam_w = scaler.s(5.5);
            if rl_dmg || rr_dmg {
                draw_line(rl_hub_x, geo.rear_axle_y, cx, geo.rear_axle_y, beam_w, if rl_dmg { dmg_col } else { norm_col });
                draw_line(cx, geo.rear_axle_y, rr_hub_x, geo.rear_axle_y, beam_w, if rr_dmg { dmg_col } else { norm_col });
            } else {
                draw_line(rl_hub_x, geo.rear_axle_y, rr_hub_x, geo.rear_axle_y, beam_w, norm_col);
            }

            // Differential Pumpkin / Bearing Housing
            let diff_r = scaler.s(7.0);
            draw_circle(cx, geo.rear_axle_y, diff_r, Color::new(0.06, 0.09, 0.16, 0.95));
            draw_circle_lines(cx, geo.rear_axle_y, diff_r, scaler.s(2.2), norm_col);
            draw_circle(cx, geo.rear_axle_y, scaler.s(3.5), norm_col);
        }
        SuspensionArchetype::MacPhersonStrut => {
            // Front Telescoping Struts to Top Mount Pivots
            let left_top = Vec2::new(cx - geo.half_body_w * 0.70, geo.front_axle_y - scaler.s(9.0));
            let right_top = Vec2::new(cx + geo.half_body_w * 0.70, geo.front_axle_y - scaler.s(9.0));

            if fl_dmg {
                draw_buckled_path(Vec2::new(fl_hub_x, geo.front_axle_y + scaler.s(2.0)), left_top, scaler.s(4.2), dmg_col);
            } else {
                draw_line(fl_hub_x, geo.front_axle_y + scaler.s(2.0), left_top.x, left_top.y, scaler.s(3.8), norm_col);
                draw_circle(left_top.x, left_top.y, scaler.s(3.5), norm_col);
            }

            if fr_dmg {
                draw_buckled_path(Vec2::new(fr_hub_x, geo.front_axle_y + scaler.s(2.0)), right_top, scaler.s(4.2), dmg_col);
            } else {
                draw_line(fr_hub_x, geo.front_axle_y + scaler.s(2.0), right_top.x, right_top.y, scaler.s(3.8), norm_col);
                draw_circle(right_top.x, right_top.y, scaler.s(3.5), norm_col);
            }

            // Lower Control Arms
            draw_line(cx - geo.half_body_w, geo.front_axle_y + scaler.s(6.0), fl_hub_x, geo.front_axle_y + scaler.s(2.0), scaler.s(2.6), norm_col);
            draw_line(cx + geo.half_body_w, geo.front_axle_y + scaler.s(6.0), fr_hub_x, geo.front_axle_y + scaler.s(2.0), scaler.s(2.6), norm_col);

            // Rear Dual Links
            draw_line(cx - geo.half_body_w, geo.rear_axle_y - scaler.s(5.0), rl_hub_x, geo.rear_axle_y, scaler.s(2.6), norm_col);
            draw_line(cx - geo.half_body_w, geo.rear_axle_y + scaler.s(5.0), rl_hub_x, geo.rear_axle_y, scaler.s(2.6), norm_col);
            draw_line(cx + geo.half_body_w, geo.rear_axle_y - scaler.s(5.0), rr_hub_x, geo.rear_axle_y, scaler.s(2.6), norm_col);
            draw_line(cx + geo.half_body_w, geo.rear_axle_y + scaler.s(5.0), rr_hub_x, geo.rear_axle_y, scaler.s(2.6), norm_col);
        }
        SuspensionArchetype::LongTravelOffRoad => {
            // Front High-Travel Dual A-Arms
            if fl_dmg {
                draw_buckled_path(Vec2::new(cx - geo.half_body_w, geo.front_axle_y - scaler.s(7.0)), Vec2::new(fl_hub_x, geo.front_axle_y), scaler.s(4.0), dmg_col);
            } else {
                draw_line(cx - geo.half_body_w, geo.front_axle_y - scaler.s(7.0), fl_hub_x, geo.front_axle_y, scaler.s(3.2), norm_col);
                draw_line(cx - geo.half_body_w, geo.front_axle_y + scaler.s(7.0), fl_hub_x, geo.front_axle_y, scaler.s(3.2), norm_col);
            }

            draw_line(cx + geo.half_body_w, geo.front_axle_y - scaler.s(7.0), fr_hub_x, geo.front_axle_y, scaler.s(3.2), norm_col);
            draw_line(cx + geo.half_body_w, geo.front_axle_y + scaler.s(7.0), fr_hub_x, geo.front_axle_y, scaler.s(3.2), norm_col);

            // Rear Massive Trailing Arms (Thick 4.5px)
            let cy = geo.center_y;
            draw_line(cx - geo.half_body_w * 0.5, cy, rl_hub_x, geo.rear_axle_y, scaler.s(4.5), norm_col);
            if rr_dmg {
                draw_buckled_path(Vec2::new(cx + geo.half_body_w * 0.5, cy), Vec2::new(rr_hub_x, geo.rear_axle_y), scaler.s(4.5), dmg_col);
            } else {
                draw_line(cx + geo.half_body_w * 0.5, cy, rr_hub_x, geo.rear_axle_y, scaler.s(4.5), norm_col);
            }

            // Shock bypass canisters
            let can_w = scaler.s(6.0);
            let can_h = scaler.s(12.0);
            draw_rectangle(rl_hub_x - scaler.s(7.0), geo.rear_axle_y - scaler.s(14.0), can_w, can_h, Color::new(0.96, 0.62, 0.04, 0.90));
            draw_rectangle(rr_hub_x + scaler.s(1.0), geo.rear_axle_y - scaler.s(14.0), can_w, can_h, Color::new(0.96, 0.62, 0.04, 0.90));
        }
        SuspensionArchetype::DoubleWishbone => {
            // Front Left
            if fl_dmg {
                draw_buckled_path(Vec2::new(cx - geo.half_body_w, geo.front_axle_y - scaler.s(6.0)), Vec2::new(fl_hub_x, geo.front_axle_y), scaler.s(4.0), dmg_col);
                draw_line(cx - geo.half_body_w, geo.front_axle_y + scaler.s(6.0), fl_hub_x, geo.front_axle_y, scaler.s(2.8), Palette::NEON_GOLD);
            } else {
                draw_line(cx - geo.half_body_w, geo.front_axle_y - scaler.s(6.0), fl_hub_x, geo.front_axle_y, scaler.s(2.8), norm_col);
                draw_line(cx - geo.half_body_w, geo.front_axle_y + scaler.s(6.0), fl_hub_x, geo.front_axle_y, scaler.s(2.8), norm_col);
            }

            // Front Right
            if fr_dmg {
                draw_buckled_path(Vec2::new(cx + geo.half_body_w, geo.front_axle_y - scaler.s(6.0)), Vec2::new(fr_hub_x, geo.front_axle_y), scaler.s(4.0), dmg_col);
            } else {
                draw_line(cx + geo.half_body_w, geo.front_axle_y - scaler.s(6.0), fr_hub_x, geo.front_axle_y, scaler.s(2.8), norm_col);
                draw_line(cx + geo.half_body_w, geo.front_axle_y + scaler.s(6.0), fr_hub_x, geo.front_axle_y, scaler.s(2.8), norm_col);
            }

            // Rear Left
            if rl_dmg {
                draw_buckled_path(Vec2::new(cx - geo.half_body_w, geo.rear_axle_y - scaler.s(6.0)), Vec2::new(rl_hub_x, geo.rear_axle_y), scaler.s(4.0), dmg_col);
            } else {
                draw_line(cx - geo.half_body_w, geo.rear_axle_y - scaler.s(6.0), rl_hub_x, geo.rear_axle_y, scaler.s(2.8), norm_col);
                draw_line(cx - geo.half_body_w, geo.rear_axle_y + scaler.s(6.0), rl_hub_x, geo.rear_axle_y, scaler.s(2.8), norm_col);
            }

            // Rear Right
            if rr_dmg {
                draw_buckled_path(Vec2::new(cx + geo.half_body_w, geo.rear_axle_y - scaler.s(6.0)), Vec2::new(rr_hub_x, geo.rear_axle_y), scaler.s(4.0), dmg_col);
            } else {
                draw_line(cx + geo.half_body_w, geo.rear_axle_y - scaler.s(6.0), rr_hub_x, geo.rear_axle_y, scaler.s(2.8), norm_col);
                draw_line(cx + geo.half_body_w, geo.rear_axle_y + scaler.s(6.0), rr_hub_x, geo.rear_axle_y, scaler.s(2.8), norm_col);
            }
        }
    }

    // Hub pivot dots
    draw_circle(fl_hub_x, geo.front_axle_y, scaler.s(3.0), if fl_dmg { dmg_col } else { norm_col });
    draw_circle(fr_hub_x, geo.front_axle_y, scaler.s(3.0), if fr_dmg { dmg_col } else { norm_col });
    draw_circle(rl_hub_x, geo.rear_axle_y, scaler.s(3.0), if rl_dmg { dmg_col } else { norm_col });
    draw_circle(rr_hub_x, geo.rear_axle_y, scaler.s(3.0), if rr_dmg { dmg_col } else { norm_col });
}

/// Helper: draws a buckled / fractured zigzag path between two points.
fn draw_buckled_path(p_start: Vec2, p_end: Vec2, thickness: f32, color: Color) {
    let mid1 = p_start + (p_end - p_start) * 0.35 + Vec2::new(-4.0, -6.0);
    let mid2 = p_start + (p_end - p_start) * 0.65 + Vec2::new(3.0, 4.0);

    draw_line(p_start.x, p_start.y, mid1.x, mid1.y, thickness, color);
    draw_line(mid1.x, mid1.y, mid2.x, mid2.y, thickness, color);
    draw_line(mid2.x, mid2.y, p_end.x, p_end.y, thickness, color);
}

/// Draws Variant C: Dynamic Telemetry Damper Travel Capsules and Fz Glow Heatmap.
fn draw_variant_c_telemetry(geo: &ChassisHudGeometry, car: &Car, scaler: &UiScaler) {
    let cx = geo.center_x;
    let fl_hub_x = geo.outboard_left_x + geo.wheel_w;
    let fr_hub_x = geo.outboard_right_x;
    let rl_hub_x = geo.outboard_left_x + geo.wheel_w;
    let rr_hub_x = geo.outboard_right_x;

    let nominal_corner_load = (car.config.mass * 9.81 * 0.25).max(100.0);

    // FL
    draw_single_damper_gauge(
        cx - geo.half_body_w,
        fl_hub_x,
        geo.front_axle_y,
        car.state.suspension[0].normal_force / nominal_corner_load,
        car.state.suspension[0].deflection,
        car.state.suspension[0].bottomed_out,
        scaler,
    );

    // FR
    draw_single_damper_gauge(
        cx + geo.half_body_w,
        fr_hub_x,
        geo.front_axle_y,
        car.state.suspension[1].normal_force / nominal_corner_load,
        car.state.suspension[1].deflection,
        car.state.suspension[1].bottomed_out,
        scaler,
    );

    // RL
    draw_single_damper_gauge(
        cx - geo.half_body_w,
        rl_hub_x,
        geo.rear_axle_y,
        car.state.suspension[2].normal_force / nominal_corner_load,
        car.state.suspension[2].deflection,
        car.state.suspension[2].bottomed_out,
        scaler,
    );

    // RR
    draw_single_damper_gauge(
        cx + geo.half_body_w,
        rr_hub_x,
        geo.rear_axle_y,
        car.state.suspension[3].normal_force / nominal_corner_load,
        car.state.suspension[3].deflection,
        car.state.suspension[3].bottomed_out,
        scaler,
    );
}

/// Draws an 8.5 x 28 px damper stroke capsule with normal-force load glow and bottom-out pulse.
fn draw_single_damper_gauge(
    chassis_x: f32,
    hub_x: f32,
    axle_y: f32,
    fz_ratio: f32,
    deflection: f32,
    bottomed_out: bool,
    scaler: &UiScaler,
) {
    // 1. Wishbone link line with dynamic load glow thickness & color
    let (load_col, load_w) = if fz_ratio >= 1.8 {
        (Palette::RED, scaler.s(5.5))
    } else if fz_ratio >= 1.3 {
        (Palette::NEON_GOLD, scaler.s(4.5))
    } else if fz_ratio >= 0.7 {
        (Palette::NEON_CYAN, scaler.s(3.0))
    } else {
        (Color::new(0.40, 0.55, 0.65, 0.45), scaler.s(2.2))
    };
    draw_line(chassis_x, axle_y, hub_x, axle_y, load_w, load_col);

    // 2. Damper capsule housing
    let gauge_w = scaler.s(8.5);
    let gauge_h = scaler.s(28.0);
    let gauge_x = (chassis_x + hub_x) * 0.5 - gauge_w * 0.5;
    let gauge_y = axle_y - gauge_h * 0.5;
    let mid_y = gauge_y + gauge_h * 0.5;

    // Housing background & border
    draw_rectangle(gauge_x, gauge_y, gauge_w, gauge_h, Color::new(0.04, 0.07, 0.12, 0.95));
    draw_rectangle_lines(
        gauge_x,
        gauge_y,
        gauge_w,
        gauge_h,
        scaler.s(1.4),
        Color::new(0.58, 0.64, 0.72, 0.55),
    );

    // Static 0mm ride-height center notch (bold white 1.8px)
    draw_line(
        gauge_x,
        mid_y,
        gauge_x + gauge_w,
        mid_y,
        scaler.s(1.8),
        Palette::WHITE,
    );

    // Bump stop limit warning tick
    draw_line(
        gauge_x + scaler.s(1.0),
        gauge_y + scaler.s(2.0),
        gauge_x + gauge_w - scaler.s(1.0),
        gauge_y + scaler.s(2.0),
        scaler.s(1.2),
        Color::new(1.0, 0.24, 0.0, 0.85),
    );

    // 3. Dynamic stroke fill
    let max_stroke = (gauge_h * 0.5) - scaler.s(2.0);
    if deflection > 0.001 {
        // Bump / compression (fills upward)
        let fill_ratio = (deflection / 0.045).clamp(0.0, 1.0);
        let fill_h = max_stroke * fill_ratio;
        let fill_y = mid_y - fill_h;
        let fill_col = if bottomed_out {
            Palette::RED
        } else if fill_ratio > 0.65 {
            Palette::NEON_GOLD
        } else {
            Palette::NEON_GREEN
        };
        draw_rectangle(
            gauge_x + scaler.s(1.5),
            fill_y,
            gauge_w - scaler.s(3.0),
            fill_h,
            fill_col,
        );
    } else if deflection < -0.001 {
        // Rebound / extension (fills downward)
        let fill_ratio = (-deflection / 0.035).clamp(0.0, 1.0);
        let fill_h = max_stroke * fill_ratio;
        draw_rectangle(
            gauge_x + scaler.s(1.5),
            mid_y,
            gauge_w - scaler.s(3.0),
            fill_h,
            Palette::NEON_CYAN,
        );
    }

    // 4. Bottom-Out Pulse Rings (Pure shockwave, no micro-text)
    if bottomed_out {
        let pulse_phase = (macroquad::time::get_time() as f32 * 5.0).fract();
        let ring_r = scaler.s(6.0) + pulse_phase * scaler.s(12.0);
        let ring_alpha = 1.0 - pulse_phase;
        let ring_col = Color::new(1.0, 0.24, 0.0, ring_alpha);

        draw_circle_lines(hub_x, axle_y, ring_r, scaler.s(2.2), ring_col);
        draw_circle(hub_x, axle_y, scaler.s(5.0), Palette::RED);
        draw_circle(hub_x, axle_y, scaler.s(2.5), Palette::WHITE);
    }
}

/// Renders the Tactical Hologram Cockpit Telemetry HUD (Spec 079).
///
/// Anchored to the lower viewport, displaying proportional chassis geometry,
/// powertrain damage, decoupled tires, Ackermann steered front wheels, and
/// dual switchable suspension modes. Zero microscopic text or percentage numbers.
pub fn render_cockpit_chassis_telemetry(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    car: &Car,
    mode: CockpitTelemetryMode,
) {
    let box_w = scaler.s(190.0);
    let box_h = scaler.s(220.0);

    // 1. Modern Glassmorphism Container
    scaler.draw_glass_card(x, y, box_w, box_h, Palette::UI_CARD_BG, Palette::UI_CARD_BORDER, 1.6);

    // 2. Header Bezel: Chassis Label (left) & Mode Pill (right)
    fonts.draw_ui_bold(
        "CHASSIS",
        x + scaler.s(10.0),
        y + scaler.s(17.0),
        scaler.font_s(10.5),
        Palette::UI_TEXT_MUTED,
    );

    let mode_label = mode.label();
    let mode_col = match mode {
        CockpitTelemetryMode::KinematicDamage => Palette::NEON_GOLD,
        CockpitTelemetryMode::DynamicTelemetry => Palette::NEON_GREEN,
    };
    let mode_text_w = mode_label.len() as f32 * scaler.s(6.5);
    fonts.draw_ui_bold(
        mode_label,
        x + box_w - mode_text_w - scaler.s(10.0),
        y + scaler.s(17.0),
        scaler.font_s(10.5),
        mode_col,
    );

    // 3. Compute Proportional Chassis HUD Geometry
    let geo = ChassisHudGeometry::compute(box_w, box_h, x, y, car, scaler.scale);

    // Axle reference guidelines
    let guide_col = Color::new(0.58, 0.64, 0.72, 0.25);
    draw_line(
        geo.outboard_left_x,
        geo.front_axle_y,
        geo.outboard_right_x + geo.wheel_w,
        geo.front_axle_y,
        scaler.s(1.0),
        guide_col,
    );
    draw_line(
        geo.outboard_left_x,
        geo.rear_axle_y,
        geo.outboard_right_x + geo.wheel_w,
        geo.rear_axle_y,
        scaler.s(1.0),
        guide_col,
    );

    // 4. Render Active Suspension Mode
    match mode {
        CockpitTelemetryMode::KinematicDamage => {
            draw_variant_b_kinematics(&geo, car, scaler);
        }
        CockpitTelemetryMode::DynamicTelemetry => {
            let (steer_fl, steer_fr) = compute_ackermann_steer_angles(car.state.steer_angle);
            draw_steering_rack(&geo, steer_fl, steer_fr, scaler);
            draw_variant_c_telemetry(&geo, car, scaler);
        }
    }

    // 5. Render Procedural Chassis Silhouette
    draw_chassis_silhouette(&geo, car, scaler);

    // 6. Render Thick Perimeter Impact Indicators
    draw_perimeter_impact_indicators(&geo, car.state.chassis_health, scaler);

    // 7. Render Authentic Powertrain Block
    draw_powertrain_block(&geo, car, scaler);

    // 8. Render Outboard Integrated-Drain Tires with Compound Accent Borders
    // Compute front wheel steering rotation and damage camber skew
    let (base_steer_fl, base_steer_fr) = compute_ackermann_steer_angles(car.state.steer_angle);

    let is_fl_damaged = car.state.suspension_health[0] < 0.80;
    let is_rr_damaged = car.state.suspension_health[3] < 0.80;

    // Structural damage camber skew tilts wheel in Variant B
    let fl_skew = if mode == CockpitTelemetryMode::KinematicDamage && is_fl_damaged {
        0.096 // +5.5 deg
    } else {
        0.0
    };
    let rr_skew = if mode == CockpitTelemetryMode::KinematicDamage && is_rr_damaged {
        -0.078 // -4.5 deg
    } else {
        0.0
    };

    let fl_rot = base_steer_fl + fl_skew;
    let fr_rot = base_steer_fr;
    let rl_rot = 0.0;
    let rr_rot = rr_skew;

    let fl_center = Vec2::new(geo.outboard_left_x + geo.wheel_w * 0.5, geo.front_axle_y);
    let fr_center = Vec2::new(geo.outboard_right_x + geo.wheel_w * 0.5, geo.front_axle_y);
    let rl_center = Vec2::new(geo.outboard_left_x + geo.wheel_w * 0.5, geo.rear_axle_y);
    let rr_center = Vec2::new(geo.outboard_right_x + geo.wheel_w * 0.5, geo.rear_axle_y);

    let wheel_border_col = |idx: usize| {
        let [r, g, b, a] = car.state.wheel_assemblies[idx].config.compound.id.accent_rgba();
        Color::new(r, g, b, a)
    };

    draw_integrated_tire(
        fl_center,
        geo.wheel_w,
        geo.wheel_h,
        car.state.wheel_assemblies[0].temperature,
        car.state.wheel_assemblies[0].wear,
        fl_rot,
        wheel_border_col(0),
        scaler,
    );
    draw_integrated_tire(
        fr_center,
        geo.wheel_w,
        geo.wheel_h,
        car.state.wheel_assemblies[1].temperature,
        car.state.wheel_assemblies[1].wear,
        fr_rot,
        wheel_border_col(1),
        scaler,
    );
    draw_integrated_tire(
        rl_center,
        geo.wheel_w,
        geo.wheel_h,
        car.state.wheel_assemblies[2].temperature,
        car.state.wheel_assemblies[2].wear,
        rl_rot,
        wheel_border_col(2),
        scaler,
    );
    draw_integrated_tire(
        rr_center,
        geo.wheel_w,
        geo.wheel_h,
        car.state.wheel_assemblies[3].temperature,
        car.state.wheel_assemblies[3].wear,
        rr_rot,
        wheel_border_col(3),
        scaler,
    );

    // 9. Dedicated Compound Legend at Card Bottom (Spec 089)
    let compound = car.state.wheel_assemblies[0].config.compound.id;
    let code = compound.badge_code();
    let name = compound.name();
    let code_w = code.len() as f32 * scaler.s(6.5);
    let pill_w = (code_w + scaler.s(10.0)).max(scaler.s(20.0));
    let gap = scaler.s(8.0);
    let name_w = name.len() as f32 * scaler.s(7.0);
    let total_w = pill_w + gap + name_w;
    let legend_x = x + (box_w - total_w) * 0.5;
    let legend_y = y + box_h - scaler.s(20.0);
    render_compound_legend(fonts, scaler, legend_x, legend_y, compound);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ackermann_steer_differential() {
        // Zero steer: both 0.0
        let (fl_zero, fr_zero) = compute_ackermann_steer_angles(0.0);
        assert_eq!(fl_zero, 0.0);
        assert_eq!(fr_zero, 0.0);

        // Right Turn (+0.30 rad): FR is inner (sharper), FL is outer (shallower)
        let (fl_right, fr_right) = compute_ackermann_steer_angles(0.30);
        assert!(fr_right > fl_right, "FR inner must be sharper than FL outer");
        assert!((fr_right - 0.30 * 1.15).abs() < 1e-4);
        assert!((fl_right - 0.30 * 0.88).abs() < 1e-4);

        // Left Turn (-0.30 rad): FL is inner (sharper magnitude), FR is outer (shallower)
        let (fl_left, fr_left) = compute_ackermann_steer_angles(-0.30);
        assert!(fl_left.abs() > fr_left.abs(), "FL inner magnitude must be greater than FR outer");
        assert!((fl_left - (-0.30 * 1.15)).abs() < 1e-4);
        assert!((fr_left - (-0.30 * 0.88)).abs() < 1e-4);
    }

    #[test]
    fn test_cockpit_telemetry_mode_toggle_and_label() {
        let mut mode = CockpitTelemetryMode::default();
        assert_eq!(mode, CockpitTelemetryMode::KinematicDamage);
        assert_eq!(mode.label(), "KINEMATICS");

        mode = CockpitTelemetryMode::DynamicTelemetry;
        assert_eq!(mode.label(), "DYNAMICS");
    }

    #[test]
    fn test_tire_temp_to_color_thresholds() {
        assert_eq!(tire_temp_to_color(55.0), Palette::NEON_CYAN);
        assert_eq!(tire_temp_to_color(85.0), Palette::NEON_GREEN);
        assert_eq!(tire_temp_to_color(108.0), Palette::NEON_GOLD);
        assert_eq!(tire_temp_to_color(125.0), Palette::RED);
    }

    #[test]
    fn test_proportional_hud_bounds_all_8_chassis() {
        use wheelbase::config::{CarConfig, ChassisSkeleton, EnginePlacement, SuspensionArchetype};

        let box_w = 190.0;
        let box_h = 220.0;

        let chassis_roster: [(&str, f32, f32, f32, f32, EnginePlacement, SuspensionArchetype); 8] = [
            // (name, wheelbase, body_width, front_overhang, rear_overhang, engine, archetype)
            ("125cc Shifter Kart", 1.05, 1.40, 0.32, 0.35, EnginePlacement::MidEngine, SuspensionArchetype::RigidKart),
            ("GT4 Sports Coupe", 2.40, 1.80, 0.80, 0.90, EnginePlacement::FrontEngine, SuspensionArchetype::MacPhersonStrut),
            ("GT3 Touring (RR)", 2.51, 2.04, 0.88, 1.15, EnginePlacement::RearEngine, SuspensionArchetype::DoubleWishbone),
            ("NASCAR Stock Car", 2.79, 1.95, 0.98, 1.12, EnginePlacement::FrontEngine, SuspensionArchetype::SolidLiveAxle),
            ("Rallycross Supercar", 2.48, 1.88, 0.75, 0.70, EnginePlacement::FrontEngine, SuspensionArchetype::MacPhersonStrut),
            ("Dakar Sand Rail", 2.40, 1.95, 0.15, 0.42, EnginePlacement::RearEngine, SuspensionArchetype::LongTravelOffRoad),
            ("Autocross CrossCar", 2.10, 1.62, 0.28, 0.38, EnginePlacement::MidEngine, SuspensionArchetype::DoubleWishbone),
            ("Le Mans Hypercar", 3.15, 2.05, 1.05, 0.95, EnginePlacement::MidEngine, SuspensionArchetype::PushrodInboard),
        ];

        for (name, wb, width, fo, ro, placement, arch) in chassis_roster {
            let mut cfg = CarConfig::sports_car();
            cfg.wheelbase = wb;
            cfg.track_width = width * 0.88;
            cfg.engine_placement = placement;
            cfg.suspension.front.archetype = arch;
            cfg.suspension.rear.archetype = arch;
            cfg.chassis = ChassisSkeleton {
                front_overhang: fo,
                rear_overhang: ro,
                body_width: width,
                cabin_start_offset: -0.40,
                cabin_end_offset: 0.30,
                headlight_spread: 0.65,
                taillight_spread: 0.70,
                light_inset: 0.05,
            };

            let car = Car::new(cfg);
            let geo = ChassisHudGeometry::compute(box_w, box_h, 0.0, 0.0, &car, 1.0);

            // Assert strictly positive scale and front axle ahead of rear axle
            assert!(geo.scale > 0.0, "{}: scale must be positive", name);
            assert!(geo.front_axle_y < geo.rear_axle_y, "{}: front axle must be above rear axle in HUD (lower y)", name);

            // Assert bounds fit strictly within the 190x220 px box without viewport clipping
            assert!(geo.nose_y >= 0.0, "{}: nose_y ({:.1}) clipped above box", name, geo.nose_y);
            assert!(geo.tail_y <= box_h, "{}: tail_y ({:.1}) clipped below box", name, geo.tail_y);
            assert!(geo.outboard_left_x >= 0.0, "{}: outboard_left_x ({:.1}) clipped left", name, geo.outboard_left_x);
            assert!(
                geo.outboard_right_x + geo.wheel_w <= box_w,
                "{}: outboard right tire ({:.1}) clipped right of box ({:.1})",
                name,
                geo.outboard_right_x + geo.wheel_w,
                box_w
            );

            // Spec 089: Assert that vehicle chassis tail leaves ample clearance before bottom legend (at y = 200)
            let legend_y = box_h - 20.0;
            assert!(
                geo.tail_y < legend_y - 10.0,
                "{}: tail_y ({:.1}) must be well above bottom legend y ({:.1})",
                name,
                geo.tail_y,
                legend_y
            );
        }
    }

    #[test]
    fn test_compound_legend_dimensions_and_properties() {
        use wheelbase::surface::CompoundId;

        let all_compounds = [
            CompoundId::SoftSlick,
            CompoundId::MediumSlick,
            CompoundId::HardSlick,
            CompoundId::AllTerrain,
            CompoundId::IntermediateWet,
            CompoundId::MonsoonWet,
            CompoundId::ExtremeMud,
            CompoundId::StuddedIce,
        ];

        let box_w = 190.0;

        for compound in all_compounds {
            let code = compound.badge_code();
            let name = compound.name();
            assert!(!code.is_empty(), "Compound code must not be empty");
            assert!(!name.is_empty(), "Compound name must not be empty");

            // Calculate total legend width at scale 1.0
            let code_w = code.len() as f32 * 6.5;
            let pill_w = (code_w + 10.0).max(20.0);
            let gap = 8.0;
            let name_w = name.len() as f32 * 7.0;
            let total_w = pill_w + gap + name_w;

            assert!(
                total_w < box_w - 20.0,
                "Compound legend for {:?} (w={:.1}) must fit inside HUD box (w={:.1}) with margins",
                compound,
                total_w,
                box_w
            );
        }
    }

    #[test]
    fn test_compound_accent_colors_for_wheel_borders() {
        use wheelbase::surface::CompoundId;

        let all_compounds = [
            CompoundId::SoftSlick,
            CompoundId::MediumSlick,
            CompoundId::HardSlick,
            CompoundId::AllTerrain,
            CompoundId::IntermediateWet,
            CompoundId::MonsoonWet,
            CompoundId::ExtremeMud,
            CompoundId::StuddedIce,
        ];

        for compound in all_compounds {
            let [r, g, b, a] = compound.accent_rgba();
            assert!(r >= 0.0 && r <= 1.0, "Red channel out of bounds for {:?}", compound);
            assert!(g >= 0.0 && g <= 1.0, "Green channel out of bounds for {:?}", compound);
            assert!(b >= 0.0 && b <= 1.0, "Blue channel out of bounds for {:?}", compound);
            assert_eq!(a, 1.0, "Alpha channel must be fully opaque for tire border on {:?}", compound);
        }
    }
}
