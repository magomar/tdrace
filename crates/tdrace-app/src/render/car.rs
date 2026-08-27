use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line};
use tdrace_core::physics::car::Car;

use super::color::{CarColorScheme, Palette};
use super::track::draw_quad;

/// Renders a vehicle with modern 2.5D motorsport arcade aesthetics:
/// - Smooth 2.5D drop shadow beneath chassis (accounting for body roll & pitch)
/// - Steered front wheels with visible alloy rims & brake calipers
/// - Streamlined chassis body with metallic specular highlights & aero contours
/// - Windshield / cockpit canopy with dynamic specular glass reflection
/// - Driver helmet with tinted visor inside cockpit
/// - High-visibility glowing LED projector headlights and illuminated LED taillights/brake lights
pub fn render_car(car: &Car, color_scheme: &CarColorScheme, is_braking: bool) {
    let pos = car.state.position;
    let angle = car.state.angle;
    let fwd = car.forward_vector();
    let right = car.right_vector();
    let z = car.state.elevation.max(0.0);

    // Body roll and squat/dive offsets from local accelerations
    let roll_offset_lat = (-car.state.acceleration_local.y * 0.015).clamp(-0.18, 0.18);
    let pitch_offset_long = (car.state.acceleration_local.x * 0.012).clamp(-0.15, 0.15);

    // 2.5D Elevation: airborne car rises along -Y screen projection and scales up slightly
    let elevation_lift = Vec2::new(0.0, -z * 1.6);
    let air_scale = 1.0 + (z * 0.07).min(0.35);

    let chassis_center = pos + right * roll_offset_lat + fwd * pitch_offset_long + elevation_lift;

    // Dimensions from car config
    let half_w = (car.config.track_width * 0.5) * air_scale;
    let lf = car.config.cg_to_front * air_scale;
    let lr = car.config.cg_to_rear * air_scale;
    let total_len = lf + lr + 0.50 * air_scale; // include overhangs
    let body_half_len = total_len * 0.5;
    let body_half_w = half_w + 0.12 * air_scale;

    // 1. --- 2.5D Drop Shadow (anchored to ground at z=0, expanding and offsetting with elevation) ---
    let shadow_pos = pos + Vec2::new(0.30 + z * 0.35, 0.40 + z * 0.45);
    let shadow_scale = 1.0 + (z * 0.15).min(0.60);
    render_chassis_shadow(shadow_pos, fwd, right, body_half_len * shadow_scale, body_half_w * shadow_scale);

    // 2. --- 4 Wheels (Tires & Rims) with Steered Front Wheels ---
    let (steer_fl, steer_fr) = car.compute_ackermann_angles(car.state.steer_angle);
    let wheel_steers = [steer_fl, steer_fr, 0.0, 0.0];
    let wheel_positions = [
        chassis_center + fwd * lf - right * half_w,
        chassis_center + fwd * lf + right * half_w,
        chassis_center - fwd * lr - right * half_w,
        chassis_center - fwd * lr + right * half_w,
    ];

    for i in 0..4 {
        let w_pos = wheel_positions[i];
        let w_angle = angle + wheel_steers[i];
        render_wheel(w_pos, w_angle);
    }

    // 3. --- Chassis Main Body ---
    render_chassis_body(chassis_center, fwd, right, body_half_len, body_half_w, color_scheme);

    // 4. --- Cockpit & Driver Helmet ---
    render_cockpit(chassis_center, fwd, right, color_scheme);

    // 5. --- Front / Rear Lighting & Aero ---
    render_car_details(chassis_center, fwd, right, body_half_len, body_half_w, is_braking);
}

/// Draws ground shadow for the vehicle.
fn render_chassis_shadow(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
) {
    let p_fl = pos + fwd * half_len - right * half_w;
    let p_fr = pos + fwd * half_len + right * half_w;
    let p_rr = pos - fwd * half_len + right * half_w;
    let p_rl = pos - fwd * half_len - right * half_w;

    draw_quad(p_fl, p_fr, p_rr, p_rl, Palette::SHADOW);
}

/// Draws an individual wheel with rubber tread and center alloy hub.
fn render_wheel(pos: Vec2, angle: f32) {
    let tire_fwd = Vec2::new(angle.cos(), angle.sin());
    let tire_right = Vec2::new(angle.sin(), -angle.cos());

    let tire_half_len = 0.32;
    let tire_half_w = 0.12;

    let p0 = pos + tire_fwd * tire_half_len - tire_right * tire_half_w;
    let p1 = pos + tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p2 = pos - tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p3 = pos - tire_fwd * tire_half_len - tire_right * tire_half_w;

    // Tire shadow
    let s_off = Vec2::new(0.08, 0.10);
    draw_quad(p0 + s_off, p1 + s_off, p2 + s_off, p3 + s_off, Palette::SHADOW);

    // Tire rubber
    draw_quad(p0, p1, p2, p3, Color::new(0.10, 0.10, 0.12, 1.0));

    // Silver / Alloy rim center
    let hub_len = 0.16;
    let hub_w = 0.06;
    let h0 = pos + tire_fwd * hub_len - tire_right * hub_w;
    let h1 = pos + tire_fwd * hub_len + tire_right * hub_w;
    let h2 = pos - tire_fwd * hub_len + tire_right * hub_w;
    let h3 = pos - tire_fwd * hub_len - tire_right * hub_w;
    draw_quad(h0, h1, h2, h3, Palette::TIRE_RIM);
}

/// Draws the main aerodynamic car body.
fn render_chassis_body(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    color_scheme: &CarColorScheme,
) {
    // 8-point sleek sports car hull contour
    let nose_w = half_w * 0.70;
    let tail_w = half_w * 0.85;

    let p_nose_l = pos + fwd * half_len - right * nose_w;
    let p_nose_r = pos + fwd * half_len + right * nose_w;
    let p_side_fl = pos + fwd * (half_len * 0.5) - right * half_w;
    let p_side_fr = pos + fwd * (half_len * 0.5) + right * half_w;
    let p_side_rl = pos - fwd * (half_len * 0.5) - right * half_w;
    let p_side_rr = pos - fwd * (half_len * 0.5) + right * half_w;
    let p_tail_l = pos - fwd * half_len - right * tail_w;
    let p_tail_r = pos - fwd * half_len + right * tail_w;

    // Body segments (Quads)
    draw_quad(p_nose_l, p_nose_r, p_side_fr, p_side_fl, color_scheme.primary);
    draw_quad(p_side_fl, p_side_fr, p_side_rr, p_side_rl, color_scheme.primary);
    draw_quad(p_side_rl, p_side_rr, p_tail_r, p_tail_l, color_scheme.primary);

    // Modern metallic / specular top reflection
    let spec_l = pos + fwd * (half_len * 0.8) - right * (half_w * 0.3);
    let spec_r = pos + fwd * (half_len * 0.8) + right * (half_w * 0.1);
    let spec_rr = pos - fwd * (half_len * 0.4) + right * (half_w * 0.1);
    let spec_rl = pos - fwd * (half_len * 0.4) - right * (half_w * 0.3);
    draw_quad(spec_l, spec_r, spec_rr, spec_rl, Color::new(1.0, 1.0, 1.0, 0.18));

    // Racing stripe down the center (secondary color)
    let stripe_w = half_w * 0.26;
    let s_nose_l = pos + fwd * half_len - right * stripe_w;
    let s_nose_r = pos + fwd * half_len + right * stripe_w;
    let s_tail_r = pos - fwd * half_len + right * stripe_w;
    let s_tail_l = pos - fwd * half_len - right * stripe_w;
    draw_quad(s_nose_l, s_nose_r, s_tail_r, s_tail_l, color_scheme.secondary);

    // Chassis edge highlights / outlines
    let outline_col = Color::new(0.04, 0.04, 0.06, 0.50);
    let th = 0.08;
    draw_line(p_nose_l.x, p_nose_l.y, p_nose_r.x, p_nose_r.y, th, outline_col);
    draw_line(p_nose_l.x, p_nose_l.y, p_side_fl.x, p_side_fl.y, th, outline_col);
    draw_line(p_nose_r.x, p_nose_r.y, p_side_fr.x, p_side_fr.y, th, outline_col);
    draw_line(p_side_fl.x, p_side_fl.y, p_side_rl.x, p_side_rl.y, th, outline_col);
    draw_line(p_side_fr.x, p_side_fr.y, p_side_rr.x, p_side_rr.y, th, outline_col);
    draw_line(p_tail_l.x, p_tail_l.y, p_tail_r.x, p_tail_r.y, th, outline_col);
}

/// Draws cockpit windshield, roof, and driver helmet with visor.
fn render_cockpit(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    color_scheme: &CarColorScheme,
) {
    let cockpit_center = pos - fwd * 0.10;
    let glass_half_len = 0.65;
    let glass_front_w = 0.42;
    let glass_rear_w = 0.48;

    let g_fl = cockpit_center + fwd * glass_half_len - right * glass_front_w;
    let g_fr = cockpit_center + fwd * glass_half_len + right * glass_front_w;
    let g_rr = cockpit_center - fwd * glass_half_len + right * glass_rear_w;
    let g_rl = cockpit_center - fwd * glass_half_len - right * glass_rear_w;

    // Tinted cockpit glass
    let glass_color = Color::new(0.10, 0.14, 0.20, 0.90);
    draw_quad(g_fl, g_fr, g_rr, g_rl, glass_color);

    // Windshield specular highlight
    let spec_fl = g_fl + fwd * 0.05;
    let spec_fr = cockpit_center + fwd * (glass_half_len * 0.6) + right * 0.1;
    let spec_rr = cockpit_center + right * 0.1;
    let spec_rl = g_fl - fwd * 0.2;
    draw_quad(spec_fl, spec_fr, spec_rr, spec_rl, Color::new(1.0, 1.0, 1.0, 0.35));

    // Driver Helmet
    let helmet_pos = cockpit_center - fwd * 0.05;
    let helmet_radius = 0.24;

    // Helmet shadow
    draw_circle(helmet_pos.x + 0.06, helmet_pos.y + 0.06, helmet_radius, Color::new(0.0, 0.0, 0.0, 0.4));
    // Helmet base
    draw_circle(helmet_pos.x, helmet_pos.y, helmet_radius, color_scheme.helmet);
    draw_circle_lines(helmet_pos.x, helmet_pos.y, helmet_radius, 0.05, Color::new(0.1, 0.1, 0.1, 0.8));

    // Dark Visor facing forward
    let visor_pos = helmet_pos + fwd * (helmet_radius * 0.55);
    let visor_fwd = fwd * 0.06;
    let visor_right = right * (helmet_radius * 0.65);
    draw_quad(
        visor_pos + visor_fwd - visor_right,
        visor_pos + visor_fwd + visor_right,
        visor_pos - visor_fwd + visor_right,
        visor_pos - visor_fwd - visor_right,
        Color::new(0.08, 0.08, 0.10, 0.95),
    );
}

/// Renders modern LED headlights, glowing taillights / brake lights, and rear carbon aero wing.
fn render_car_details(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    is_braking: bool,
) {
    // Projector LED Headlights with soft glow
    let light_w = half_w * 0.55;
    let head_l = pos + fwd * (half_len - 0.05) - right * light_w;
    let head_r = pos + fwd * (half_len - 0.05) + right * light_w;

    // Headlight outer glow
    draw_circle(head_l.x, head_l.y, 0.20, Color::new(0.95, 0.98, 1.0, 0.35));
    draw_circle(head_r.x, head_r.y, 0.20, Color::new(0.95, 0.98, 1.0, 0.35));
    // Headlight core
    draw_circle(head_l.x, head_l.y, 0.12, Color::new(1.0, 1.0, 1.0, 0.95));
    draw_circle(head_r.x, head_r.y, 0.12, Color::new(1.0, 1.0, 1.0, 0.95));

    // Tail / LED Brake Lights with glow halo
    let tail_l = pos - fwd * (half_len - 0.05) - right * (half_w * 0.65);
    let tail_r = pos - fwd * (half_len - 0.05) + right * (half_w * 0.65);

    if is_braking {
        // High-intensity brake light bloom
        draw_circle(tail_l.x, tail_l.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
        draw_circle(tail_r.x, tail_r.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
        draw_circle(tail_l.x, tail_l.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
        draw_circle(tail_r.x, tail_r.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
    } else {
        // Subtle ambient running LEDs
        draw_circle(tail_l.x, tail_l.y, 0.11, Color::new(0.60, 0.08, 0.08, 0.85));
        draw_circle(tail_r.x, tail_r.y, 0.11, Color::new(0.60, 0.08, 0.08, 0.85));
    }

    // Modern Carbon Aero Wing
    let wing_pos = pos - fwd * (half_len + 0.05);
    let wing_half_w = half_w * 0.95;
    let wing_thick = 0.12;

    let w0 = wing_pos + fwd * wing_thick - right * wing_half_w;
    let w1 = wing_pos + fwd * wing_thick + right * wing_half_w;
    let w2 = wing_pos - fwd * wing_thick + right * wing_half_w;
    let w3 = wing_pos - fwd * wing_thick - right * wing_half_w;

    draw_quad(w0, w1, w2, w3, Color::new(0.12, 0.12, 0.15, 0.98));
}
