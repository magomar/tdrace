use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line};
use tdrace_core::physics::car::Car;

use super::color::{CarColorScheme, Palette};
use super::track::draw_quad;
use crate::module::VehicleVisualType;

/// Renders a vehicle with modern 2.5D motorsport arcade aesthetics based on its visual archetype.
pub fn render_car(car: &Car, color_scheme: &CarColorScheme, is_braking: bool) {
    render_car_with_visual_type(
        car,
        color_scheme,
        is_braking,
        VehicleVisualType::TouringGT {
            widebody: true,
            gt_wing: true,
            diffuser: true,
        },
    );
}

/// Renders a vehicle with a specific visual archetype (OpenWheel, TouringGT, RallyHatch, GoKart).
pub fn render_car_with_visual_type(
    car: &Car,
    color_scheme: &CarColorScheme,
    is_braking: bool,
    visual_type: VehicleVisualType,
) {
    let pos = car.state.position;
    let angle = car.state.angle;
    let fwd = car.forward_vector();
    let right = car.right_vector();
    let z_jump = car.state.elevation.max(0.0);
    let z_road = car.state.road_elevation.max(0.0);
    let z_total = z_jump + z_road;

    // Body roll and squat/dive offsets from local accelerations
    let roll_offset_lat = (-car.state.acceleration_local.y * 0.015).clamp(-0.18, 0.18);
    let pitch_offset_long = (car.state.acceleration_local.x * 0.012).clamp(-0.15, 0.15);

    // 2.5D Elevation: airborne car rises along -Y screen projection and scales up slightly
    let elevation_lift = Vec2::new(0.0, -z_jump * 1.6);
    let air_scale = 1.0 + (z_jump * 0.07).min(0.35);

    let chassis_center = pos + right * roll_offset_lat + fwd * pitch_offset_long + elevation_lift;

    // Dimensions from car config
    let half_w = (car.config.track_width * 0.5) * air_scale;
    let lf = car.config.cg_to_front * air_scale;
    let lr = car.config.cg_to_rear * air_scale;
    let total_len = lf + lr + 0.50 * air_scale; // include overhangs
    let body_half_len = total_len * 0.5;
    let body_half_w = half_w + 0.12 * air_scale;

    // 1. --- 2.5D Drop Shadow ---
    let shadow_pos = pos + Vec2::new(0.30 + z_total * 0.25, 0.40 + z_total * 0.35);
    let shadow_scale = 1.0 + (z_total * 0.08).min(0.60);
    render_chassis_shadow(shadow_pos, fwd, right, body_half_len * shadow_scale, body_half_w * shadow_scale);

    // 2. --- 4 Wheels & Steering ---
    let (steer_fl, steer_fr) = car.compute_ackermann_angles(car.state.steer_angle);
    let wheel_steers = [steer_fl, steer_fr, 0.0, 0.0];
    let wheel_positions = [
        chassis_center + fwd * lf - right * half_w,
        chassis_center + fwd * lf + right * half_w,
        chassis_center - fwd * lr - right * half_w,
        chassis_center - fwd * lr + right * half_w,
    ];

    match visual_type {
        VehicleVisualType::OpenWheel { front_wing_span, rear_wing_height, halo } => {
            // Render exposed suspension wishbones
            render_open_wheel_suspension(chassis_center, &wheel_positions, fwd, right);

            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], true);
            }

            // Monocoque, nosecone, sidepods
            render_open_wheel_body(chassis_center, fwd, right, body_half_len, body_half_w, color_scheme, front_wing_span, rear_wing_height, halo, is_braking);
        }
        VehicleVisualType::GoKart { exposed_driver, side_bumpers } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], false);
            }
            render_kart_body(chassis_center, fwd, right, body_half_len, body_half_w, color_scheme, exposed_driver, side_bumpers, is_braking);
        }
        VehicleVisualType::RallyHatch { roof_scoop, mudflaps, large_wing } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], false);
            }
            if mudflaps {
                render_mudflaps(chassis_center, &wheel_positions, fwd, right);
            }
            render_rally_body(chassis_center, fwd, right, body_half_len, body_half_w, color_scheme, roof_scoop, large_wing, is_braking);
        }
        VehicleVisualType::TouringGT { .. } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], false);
            }
            render_chassis_body(chassis_center, fwd, right, body_half_len, body_half_w, color_scheme);
            render_cockpit(chassis_center, fwd, right, color_scheme);
            render_car_details(chassis_center, fwd, right, body_half_len, body_half_w, is_braking);
        }
        VehicleVisualType::StockCar { tall_wing, roof_fins, window_net } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], true);
            }
            render_stock_car_body(
                car,
                chassis_center,
                fwd,
                right,
                body_half_len,
                body_half_w,
                color_scheme,
                tall_wing,
                roof_fins,
                window_net,
                is_braking,
            );
        }
    }
}

/// Draws ground shadow for the vehicle.
fn render_chassis_shadow(pos: Vec2, fwd: Vec2, right: Vec2, half_len: f32, half_w: f32) {
    let p_fl = pos + fwd * half_len - right * half_w;
    let p_fr = pos + fwd * half_len + right * half_w;
    let p_rr = pos - fwd * half_len + right * half_w;
    let p_rl = pos - fwd * half_len - right * half_w;

    draw_quad(p_fl, p_fr, p_rr, p_rl, Palette::SHADOW);
}

/// Draws an individual wheel with rubber tread and center alloy hub.
fn render_wheel(pos: Vec2, angle: f32, is_wide_slick: bool) {
    let tire_fwd = Vec2::new(angle.cos(), angle.sin());
    let tire_right = Vec2::new(angle.sin(), -angle.cos());

    let tire_half_len = if is_wide_slick { 0.36 } else { 0.32 };
    let tire_half_w = if is_wide_slick { 0.18 } else { 0.12 };

    let p0 = pos + tire_fwd * tire_half_len - tire_right * tire_half_w;
    let p1 = pos + tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p2 = pos - tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p3 = pos - tire_fwd * tire_half_len - tire_right * tire_half_w;

    // Tire shadow
    let s_off = Vec2::new(0.08, 0.10);
    draw_quad(p0 + s_off, p1 + s_off, p2 + s_off, p3 + s_off, Palette::SHADOW);

    // Tire rubber
    draw_quad(p0, p1, p2, p3, Color::new(0.10, 0.10, 0.12, 1.0));

    // Alloy rim center
    let hub_len = tire_half_len * 0.55;
    let hub_w = tire_half_w * 0.55;
    let h0 = pos + tire_fwd * hub_len - tire_right * hub_w;
    let h1 = pos + tire_fwd * hub_len + tire_right * hub_w;
    let h2 = pos - tire_fwd * hub_len + tire_right * hub_w;
    let h3 = pos - tire_fwd * hub_len - tire_right * hub_w;
    draw_quad(h0, h1, h2, h3, Palette::TIRE_RIM);
}

/// Renders exposed double-wishbone suspension arms for open-wheel formula cars.
fn render_open_wheel_suspension(chassis: Vec2, wheels: &[Vec2; 4], _fwd: Vec2, _right: Vec2) {
    let arm_col = Color::new(0.18, 0.18, 0.22, 0.95);
    let th = 0.06;

    // Front left & right wishbones
    draw_line(chassis.x, chassis.y, wheels[0].x, wheels[0].y, th, arm_col);
    draw_line(chassis.x, chassis.y, wheels[1].x, wheels[1].y, th, arm_col);
    // Rear left & right wishbones
    draw_line(chassis.x, chassis.y, wheels[2].x, wheels[2].y, th, arm_col);
    draw_line(chassis.x, chassis.y, wheels[3].x, wheels[3].y, th, arm_col);
}

/// Renders Formula 1 Open-Wheel monocoque, wings, sidepods, and halo.
#[allow(clippy::too_many_arguments)]
fn render_open_wheel_body(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    color_scheme: &CarColorScheme,
    front_wing_span: f32,
    _rear_wing_height: f32,
    halo: bool,
    is_braking: bool,
) {
    // 1. Front Wing
    let fw_pos = pos + fwd * (half_len + 0.35);
    let fw_half_w = front_wing_span * 0.5;
    let fw_depth = 0.22;
    draw_quad(
        fw_pos + fwd * fw_depth - right * fw_half_w,
        fw_pos + fwd * fw_depth + right * fw_half_w,
        fw_pos - fwd * fw_depth + right * fw_half_w,
        fw_pos - fwd * fw_depth - right * fw_half_w,
        color_scheme.primary,
    );
    // Front wing endplates
    let ep_th = 0.06;
    let ep_len = 0.28;
    draw_line(
        (fw_pos - right * fw_half_w - fwd * ep_len).x,
        (fw_pos - right * fw_half_w - fwd * ep_len).y,
        (fw_pos - right * fw_half_w + fwd * ep_len).x,
        (fw_pos - right * fw_half_w + fwd * ep_len).y,
        ep_th,
        color_scheme.secondary,
    );
    draw_line(
        (fw_pos + right * fw_half_w - fwd * ep_len).x,
        (fw_pos + right * fw_half_w - fwd * ep_len).y,
        (fw_pos + right * fw_half_w + fwd * ep_len).x,
        (fw_pos + right * fw_half_w + fwd * ep_len).y,
        ep_th,
        color_scheme.secondary,
    );

    // 2. Needle Nosecone & Cockpit Cell
    let nose_tip = pos + fwd * (half_len + 0.25);
    let nose_base_l = pos + fwd * (half_len * 0.4) - right * (half_w * 0.35);
    let nose_base_r = pos + fwd * (half_len * 0.4) + right * (half_w * 0.35);
    draw_quad(nose_tip - right * 0.12, nose_tip + right * 0.12, nose_base_r, nose_base_l, color_scheme.primary);

    // 3. Sidepods
    let sp_front_l = pos + fwd * (half_len * 0.3) - right * (half_w * 0.75);
    let sp_front_r = pos + fwd * (half_len * 0.3) + right * (half_w * 0.75);
    let sp_rear_l = pos - fwd * (half_len * 0.6) - right * (half_w * 0.65);
    let sp_rear_r = pos - fwd * (half_len * 0.6) + right * (half_w * 0.65);
    draw_quad(sp_front_l, sp_front_r, sp_rear_r, sp_rear_l, color_scheme.secondary);

    // Engine Cover & Shark Fin Spine
    let fin_start = pos + fwd * 0.20;
    let fin_end = pos - fwd * (half_len * 0.85);
    draw_line(fin_start.x, fin_start.y, fin_end.x, fin_end.y, 0.08, color_scheme.primary);

    // 4. Driver Helmet & Cockpit
    let helmet_pos = pos + fwd * 0.15;
    draw_circle(helmet_pos.x, helmet_pos.y, 0.22, color_scheme.helmet);
    draw_circle_lines(helmet_pos.x, helmet_pos.y, 0.22, 0.04, Color::new(0.1, 0.1, 0.1, 0.9));

    // Visor
    let visor_pos = helmet_pos + fwd * 0.12;
    draw_line((visor_pos - right * 0.12).x, (visor_pos - right * 0.12).y, (visor_pos + right * 0.12).x, (visor_pos + right * 0.12).y, 0.08, Color::new(0.05, 0.05, 0.08, 0.95));

    // Halo safety ring
    if halo {
        let halo_center = helmet_pos + fwd * 0.10;
        draw_circle_lines(halo_center.x, halo_center.y, 0.28, 0.06, Color::new(0.12, 0.12, 0.15, 0.90));
        draw_line(helmet_pos.x, (helmet_pos + fwd * 0.35).y, helmet_pos.x, (helmet_pos + fwd * 0.10).y, 0.06, Color::new(0.12, 0.12, 0.15, 0.90));
    }

    // 5. Rear Wing & Rain Light
    let rw_pos = pos - fwd * (half_len + 0.15);
    let rw_half_w = half_w * 0.85;
    let rw_depth = 0.20;
    draw_quad(
        rw_pos + fwd * rw_depth - right * rw_half_w,
        rw_pos + fwd * rw_depth + right * rw_half_w,
        rw_pos - fwd * rw_depth + right * rw_half_w,
        rw_pos - fwd * rw_depth - right * rw_half_w,
        color_scheme.primary,
    );

    // Rear LED safety rain light / brake flasher
    let led_pos = rw_pos - fwd * rw_depth;
    if is_braking {
        draw_circle(led_pos.x, led_pos.y, 0.18, Color::new(1.0, 0.1, 0.1, 0.95));
    } else {
        draw_circle(led_pos.x, led_pos.y, 0.08, Color::new(0.7, 0.1, 0.1, 0.70));
    }
}

/// Renders Go-Kart tubular chassis, side bumpers, exposed engine and driver.
#[allow(clippy::too_many_arguments)]
fn render_kart_body(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    color_scheme: &CarColorScheme,
    exposed_driver: bool,
    side_bumpers: bool,
    _is_braking: bool,
) {
    // 1. Tubular perimeter chassis
    if side_bumpers {
        let bumper_col = Color::new(0.20, 0.20, 0.25, 0.95);
        let b_fl = pos + fwd * (half_len * 0.8) - right * (half_w * 0.95);
        let b_fr = pos + fwd * (half_len * 0.8) + right * (half_w * 0.95);
        let b_rr = pos - fwd * (half_len * 0.8) + right * (half_w * 0.95);
        let b_rl = pos - fwd * (half_len * 0.8) - right * (half_w * 0.95);
        draw_line(b_fl.x, b_fl.y, b_fr.x, b_fr.y, 0.08, bumper_col);
        draw_line(b_fl.x, b_fl.y, b_rl.x, b_rl.y, 0.08, bumper_col);
        draw_line(b_fr.x, b_fr.y, b_rr.x, b_rr.y, 0.08, bumper_col);
        draw_line(b_rl.x, b_rl.y, b_rr.x, b_rr.y, 0.08, bumper_col);
    }

    // 2. Front Nosecone / Number Plate Fairing
    let nose_pos = pos + fwd * (half_len * 0.65);
    draw_quad(
        nose_pos + fwd * 0.25 - right * 0.35,
        nose_pos + fwd * 0.25 + right * 0.35,
        nose_pos - fwd * 0.25 + right * 0.35,
        nose_pos - fwd * 0.25 - right * 0.35,
        color_scheme.primary,
    );

    // 3. Side Pods
    let sp_l = pos - right * (half_w * 0.70);
    let sp_r = pos + right * (half_w * 0.70);
    let sp_hl = half_len * 0.40;
    let sp_hw = 0.16;
    draw_quad(sp_l + fwd * sp_hl - right * sp_hw, sp_l + fwd * sp_hl + right * sp_hw, sp_l - fwd * sp_hl + right * sp_hw, sp_l - fwd * sp_hl - right * sp_hw, color_scheme.secondary);
    draw_quad(sp_r + fwd * sp_hl - right * sp_hw, sp_r + fwd * sp_hl + right * sp_hw, sp_r - fwd * sp_hl + right * sp_hw, sp_r - fwd * sp_hl - right * sp_hw, color_scheme.secondary);

    // 4. Exposed Rear Engine & Exhaust
    let eng_pos = pos - fwd * (half_len * 0.5) + right * 0.30;
    draw_circle(eng_pos.x, eng_pos.y, 0.16, Color::new(0.35, 0.35, 0.40, 1.0)); // Cylinder head
    draw_line(eng_pos.x, eng_pos.y, (eng_pos - fwd * 0.30).x, (eng_pos - fwd * 0.30).y, 0.08, Color::new(0.6, 0.6, 0.65, 1.0)); // Chrome pipe

    // 5. Driver Body, Arms, and Helmet
    if exposed_driver {
        let driver_pos = pos - fwd * 0.10;
        // Driver Torso (Racing suit)
        draw_circle(driver_pos.x, driver_pos.y, 0.28, color_scheme.primary);
        // Steering wheel
        let wheel_pos = pos + fwd * 0.25;
        draw_circle_lines(wheel_pos.x, wheel_pos.y, 0.14, 0.05, Color::new(0.1, 0.1, 0.1, 0.9));
        // Driver Helmet
        draw_circle(driver_pos.x, driver_pos.y, 0.20, color_scheme.helmet);
        let visor = driver_pos + fwd * 0.10;
        draw_line((visor - right * 0.10).x, (visor - right * 0.10).y, (visor + right * 0.10).x, (visor + right * 0.10).y, 0.06, Color::new(0.05, 0.05, 0.08, 0.95));
    }
}

/// Renders Rally mudflaps behind all 4 wheels.
fn render_mudflaps(_chassis: Vec2, wheels: &[Vec2; 4], fwd: Vec2, right: Vec2) {
    let flap_col = Color::new(0.85, 0.10, 0.10, 0.95); // Bright Red Rally Mudflaps
    let flap_hw = 0.16;
    let flap_len = 0.10;

    for w in wheels {
        let flap_pos = *w - fwd * 0.40;
        draw_quad(
            flap_pos + fwd * flap_len - right * flap_hw,
            flap_pos + fwd * flap_len + right * flap_hw,
            flap_pos - fwd * flap_len + right * flap_hw,
            flap_pos - fwd * flap_len - right * flap_hw,
            flap_col,
        );
    }
}

/// Renders Rally Hatchback body with roof scoop and large rally wing.
fn render_rally_body(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    color_scheme: &CarColorScheme,
    roof_scoop: bool,
    large_wing: bool,
    is_braking: bool,
) {
    render_chassis_body(pos, fwd, right, half_len, half_w, color_scheme);
    render_cockpit(pos, fwd, right, color_scheme);

    // Roof air intake scoop
    if roof_scoop {
        let scoop_pos = pos + fwd * 0.15;
        let scoop_hl = 0.20;
        let scoop_hw = 0.15;
        draw_quad(
            scoop_pos + fwd * scoop_hl - right * scoop_hw,
            scoop_pos + fwd * scoop_hl + right * scoop_hw,
            scoop_pos - fwd * scoop_hl + right * scoop_hw,
            scoop_pos - fwd * scoop_hl - right * scoop_hw,
            color_scheme.secondary,
        );
    }

    // Prominent Rally Spoiler
    if large_wing {
        let wing_pos = pos - fwd * (half_len + 0.10);
        let wing_hw = half_w * 0.90;
        draw_quad(
            wing_pos + fwd * 0.18 - right * wing_hw,
            wing_pos + fwd * 0.18 + right * wing_hw,
            wing_pos - fwd * 0.18 + right * wing_hw,
            wing_pos - fwd * 0.18 - right * wing_hw,
            color_scheme.primary,
        );
    }

    render_car_details(pos, fwd, right, half_len, half_w, is_braking);
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
        draw_circle(tail_l.x, tail_l.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
        draw_circle(tail_r.x, tail_r.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
        draw_circle(tail_l.x, tail_l.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
        draw_circle(tail_r.x, tail_r.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
    } else {
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

/// Renders American Stock Car / Trans-Am TA1 silhouette body with muscular proportions,
/// side exhaust heat shields, overrun flames, door number plates, window netting, and aero wings.
#[allow(clippy::too_many_arguments)]
fn render_stock_car_body(
    car: &Car,
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    color_scheme: &CarColorScheme,
    tall_wing: bool,
    roof_fins: bool,
    window_net: bool,
    is_braking: bool,
) {
    // 1. --- Muscular Stock Car Body Profile ---
    let nose_w = half_w * 0.88;
    let tail_w = half_w * 0.90;

    let p_nose_l = pos + fwd * half_len - right * nose_w;
    let p_nose_r = pos + fwd * half_len + right * nose_w;
    let p_fender_fl = pos + fwd * (half_len * 0.55) - right * (half_w * 1.02);
    let p_fender_fr = pos + fwd * (half_len * 0.55) + right * (half_w * 1.02);
    let p_waist_l = pos - right * (half_w * 0.96);
    let p_waist_r = pos + right * (half_w * 0.96);
    let p_fender_rl = pos - fwd * (half_len * 0.55) - right * (half_w * 1.02);
    let p_fender_rr = pos - fwd * (half_len * 0.55) + right * (half_w * 1.02);
    let p_tail_l = pos - fwd * half_len - right * tail_w;
    let p_tail_r = pos - fwd * half_len + right * tail_w;

    // Body segments
    draw_quad(p_nose_l, p_nose_r, p_fender_fr, p_fender_fl, color_scheme.primary);
    draw_quad(p_fender_fl, p_fender_fr, p_waist_r, p_waist_l, color_scheme.primary);
    draw_quad(p_waist_l, p_waist_r, p_fender_rr, p_fender_rl, color_scheme.primary);
    draw_quad(p_fender_rl, p_fender_rr, p_tail_r, p_tail_l, color_scheme.primary);

    // Front chin splitter / air dam
    let split_len = half_len + 0.18;
    let split_w = nose_w + 0.08;
    let sp_fl = pos + fwd * split_len - right * split_w;
    let sp_fr = pos + fwd * split_len + right * split_w;
    let sp_bl = pos + fwd * half_len - right * split_w;
    let sp_br = pos + fwd * half_len + right * split_w;
    draw_quad(sp_fl, sp_fr, sp_br, sp_bl, Color::new(0.10, 0.10, 0.12, 0.98));

    // Splitter tie rods (struts)
    let rod_y = half_w * 0.45;
    let rod_l_start = pos + fwd * split_len - right * rod_y;
    let rod_l_end = pos + fwd * half_len - right * (rod_y * 0.85);
    let rod_r_start = pos + fwd * split_len + right * rod_y;
    let rod_r_end = pos + fwd * half_len + right * (rod_y * 0.85);
    draw_line(rod_l_start.x, rod_l_start.y, rod_l_end.x, rod_l_end.y, 0.05, Color::new(0.75, 0.75, 0.80, 0.95));
    draw_line(rod_r_start.x, rod_r_start.y, rod_r_end.x, rod_r_end.y, 0.05, Color::new(0.75, 0.75, 0.80, 0.95));

    // Specular top reflection along body centerline
    let spec_l = pos + fwd * (half_len * 0.85) - right * (half_w * 0.28);
    let spec_r = pos + fwd * (half_len * 0.85) + right * (half_w * 0.08);
    let spec_rr = pos - fwd * (half_len * 0.50) + right * (half_w * 0.08);
    let spec_rl = pos - fwd * (half_len * 0.50) - right * (half_w * 0.28);
    draw_quad(spec_l, spec_r, spec_rr, spec_rl, Color::new(1.0, 1.0, 1.0, 0.16));

    // Classic twin racing stripes or hood scallop
    let stripe_hw = half_w * 0.24;
    let s_nl = pos + fwd * half_len - right * stripe_hw;
    let s_nr = pos + fwd * half_len + right * stripe_hw;
    let s_tr = pos - fwd * half_len + right * stripe_hw;
    let s_tl = pos - fwd * half_len - right * stripe_hw;
    draw_quad(s_nl, s_nr, s_tr, s_tl, color_scheme.secondary);

    // Hood pins
    let pin_fwd = pos + fwd * (half_len * 0.72);
    let pin_offset = right * (half_w * 0.55);
    draw_circle((pin_fwd - pin_offset).x, (pin_fwd - pin_offset).y, 0.06, Color::new(0.85, 0.85, 0.90, 0.95));
    draw_circle((pin_fwd + pin_offset).x, (pin_fwd + pin_offset).y, 0.06, Color::new(0.85, 0.85, 0.90, 0.95));

    // 2. --- Greenhouse / Cockpit ---
    let cockpit_center = pos - fwd * 0.12;
    let glass_flen = 0.55;
    let glass_rlen = 0.65;
    let glass_fw = half_w * 0.65;
    let glass_rw = half_w * 0.70;

    let g_fl = cockpit_center + fwd * glass_flen - right * glass_fw;
    let g_fr = cockpit_center + fwd * glass_flen + right * glass_fw;
    let g_rr = cockpit_center - fwd * glass_rlen + right * glass_rw;
    let g_rl = cockpit_center - fwd * glass_rlen - right * glass_rw;

    // Dark safety glass
    draw_quad(g_fl, g_fr, g_rr, g_rl, Color::new(0.08, 0.10, 0.14, 0.92));

    // Windshield specular angle highlight
    let w_spec_l = g_fl + fwd * 0.04;
    let w_spec_r = cockpit_center + fwd * (glass_flen * 0.6) + right * 0.08;
    let w_spec_rr = cockpit_center + right * 0.08;
    let w_spec_rl = g_fl - fwd * 0.16;
    draw_quad(w_spec_l, w_spec_r, w_spec_rr, w_spec_rl, Color::new(1.0, 1.0, 1.0, 0.30));

    // Roof metal panel
    let roof_hl = 0.38;
    let roof_hw = half_w * 0.58;
    let r_fl = cockpit_center + fwd * roof_hl - right * roof_hw;
    let r_fr = cockpit_center + fwd * roof_hl + right * roof_hw;
    let r_rr = cockpit_center - fwd * roof_hl + right * roof_hw;
    let r_rl = cockpit_center - fwd * roof_hl - right * roof_hw;
    draw_quad(r_fl, r_fr, r_rr, r_rl, color_scheme.primary);

    // Driver window safety netting (Left side of cockpit)
    if window_net {
        let net_start = cockpit_center + fwd * (roof_hl * 0.8) - right * (glass_fw * 0.95);
        let net_end = cockpit_center - fwd * (roof_hl * 0.8) - right * (glass_rw * 0.95);
        let net_col = Color::new(0.06, 0.06, 0.08, 0.95);

        // Outer border
        draw_line(net_start.x, net_start.y, net_end.x, net_end.y, 0.06, net_col);

        // Webbing grid (vertical & horizontal net ribbons)
        for step in 1..4 {
            let t = step as f32 / 4.0;
            let p = net_start.lerp(net_end, t);
            let inner_p = p + right * 0.16;
            draw_line(p.x, p.y, inner_p.x, inner_p.y, 0.035, net_col);
        }
        draw_line(
            (net_start + right * 0.08).x,
            (net_start + right * 0.08).y,
            (net_end + right * 0.08).x,
            (net_end + right * 0.08).y,
            0.035,
            net_col,
        );
    }

    // Aerodynamic roof fins & roof safety flaps
    if roof_fins {
        let fin_col = Color::new(0.12, 0.12, 0.16, 0.90);
        // Roof longitudinal roof rails
        let fin_offset = roof_hw * 0.70;
        let l_start = cockpit_center + fwd * roof_hl - right * fin_offset;
        let l_end = cockpit_center - fwd * roof_hl - right * fin_offset;
        let r_start = cockpit_center + fwd * roof_hl + right * fin_offset;
        let r_end = cockpit_center - fwd * roof_hl + right * fin_offset;
        draw_line(l_start.x, l_start.y, l_end.x, l_end.y, 0.04, fin_col);
        draw_line(r_start.x, r_start.y, r_end.x, r_end.y, 0.04, fin_col);

        // Rear window shark-fin stabilizer (NASCAR superspeedway shark-fin on greenhouse)
        let fin_win_start = cockpit_center - fwd * roof_hl - right * (roof_hw * 0.35);
        let fin_win_end = cockpit_center - fwd * glass_rlen - right * (roof_hw * 0.35);
        draw_line(fin_win_start.x, fin_win_start.y, fin_win_end.x, fin_win_end.y, 0.05, fin_col);
    }

    // Driver Helmet inside cockpit
    let helmet_pos = cockpit_center - fwd * 0.05 - right * 0.14;
    let helmet_radius = 0.22;
    draw_circle(helmet_pos.x + 0.04, helmet_pos.y + 0.04, helmet_radius, Color::new(0.0, 0.0, 0.0, 0.4));
    draw_circle(helmet_pos.x, helmet_pos.y, helmet_radius, color_scheme.helmet);
    draw_circle_lines(helmet_pos.x, helmet_pos.y, helmet_radius, 0.04, Color::new(0.1, 0.1, 0.1, 0.8));
    // Visor
    let visor_pos = helmet_pos + fwd * (helmet_radius * 0.55);
    draw_line(
        (visor_pos - right * 0.11).x,
        (visor_pos - right * 0.11).y,
        (visor_pos + right * 0.11).x,
        (visor_pos + right * 0.11).y,
        0.06,
        Color::new(0.08, 0.08, 0.10, 0.95),
    );

    // 3. --- Sponsor Door Number Plates ---
    let plate_hl = 0.28;
    let plate_hw = 0.10;
    // Right side door plate
    let plate_r = pos - fwd * 0.08 + right * (half_w * 0.98);
    draw_quad(
        plate_r + fwd * plate_hl - right * plate_hw,
        plate_r + fwd * plate_hl + right * plate_hw,
        plate_r - fwd * plate_hl + right * plate_hw,
        plate_r - fwd * plate_hl - right * plate_hw,
        Palette::WHITE,
    );
    // Dark door race number badge inside plate
    draw_line(
        (plate_r + fwd * 0.14).x,
        (plate_r + fwd * 0.14).y,
        (plate_r - fwd * 0.14).x,
        (plate_r - fwd * 0.14).y,
        0.08,
        Color::new(0.10, 0.10, 0.12, 0.95),
    );

    // Left side door plate
    let plate_l = pos - fwd * 0.08 - right * (half_w * 0.98);
    draw_quad(
        plate_l + fwd * plate_hl - right * plate_hw,
        plate_l + fwd * plate_hl + right * plate_hw,
        plate_l - fwd * plate_hl + right * plate_hw,
        plate_l - fwd * plate_hl - right * plate_hw,
        Palette::WHITE,
    );
    draw_line(
        (plate_l + fwd * 0.14).x,
        (plate_l + fwd * 0.14).y,
        (plate_l - fwd * 0.14).x,
        (plate_l - fwd * 0.14).y,
        0.08,
        Color::new(0.10, 0.10, 0.12, 0.95),
    );

    // 4. --- Side Exhaust Heat Shields & Outlets ---
    // Right side boom tube exhaust (classic NASCAR side exit)
    let exh_pos = pos - fwd * (half_len * 0.22) + right * (half_w * 0.98);
    let shield_hl = 0.28;
    let shield_hw = 0.08;
    // Aluminum heat shield plate
    draw_quad(
        exh_pos + fwd * shield_hl - right * shield_hw,
        exh_pos + fwd * shield_hl + right * shield_hw,
        exh_pos - fwd * shield_hl + right * shield_hw,
        exh_pos - fwd * shield_hl - right * shield_hw,
        Color::new(0.74, 0.76, 0.80, 0.95),
    );
    // Fasteners / rivets on heat shield
    draw_circle((exh_pos + fwd * 0.22).x, (exh_pos + fwd * 0.22).y, 0.025, Color::new(0.2, 0.2, 0.25, 0.9));
    draw_circle((exh_pos - fwd * 0.22).x, (exh_pos - fwd * 0.22).y, 0.025, Color::new(0.2, 0.2, 0.25, 0.9));

    // Dual exhaust pipe tips
    let tip1 = exh_pos + fwd * 0.08;
    let tip2 = exh_pos - fwd * 0.08;
    draw_circle(tip1.x, tip1.y, 0.055, Color::new(0.15, 0.15, 0.18, 1.0));
    draw_circle(tip2.x, tip2.y, 0.055, Color::new(0.15, 0.15, 0.18, 1.0));
    draw_circle(tip1.x, tip1.y, 0.035, Color::new(0.04, 0.04, 0.05, 1.0));
    draw_circle(tip2.x, tip2.y, 0.035, Color::new(0.04, 0.04, 0.05, 1.0));

    // 5. --- Exhaust Flame Visual Effect on Overrun ---
    // Overrun occurs during deceleration or braking when lifting off throttle at speed
    let is_overrun = (is_braking || car.state.acceleration_local.x < -0.6) && car.state.speed > 5.0;
    if is_overrun {
        // High-compression 850 BHP V8 combustion overrun flames licking out from the boom tubes
        let flame_dir = (-fwd * 0.65 + right * 0.75).normalize();
        let flame_len = 0.52 + ((pos.x * 12.0 + pos.y * 7.0).sin().abs() * 0.22);
        let flame_tip = exh_pos + flame_dir * flame_len;
        let flame_perp = Vec2::new(-flame_dir.y, flame_dir.x);

        // Outer orange/red flame plume
        let plume_w = 0.12;
        draw_quad(
            exh_pos - flame_perp * (plume_w * 0.6),
            exh_pos + flame_perp * (plume_w * 0.6),
            flame_tip + flame_perp * (plume_w * 0.2),
            flame_tip - flame_perp * (plume_w * 0.2),
            Color::new(1.0, 0.30, 0.05, 0.88),
        );

        // Hot inner yellow/white flame core
        let core_len = flame_len * 0.65;
        let core_tip = exh_pos + flame_dir * core_len;
        let core_w = plume_w * 0.5;
        draw_quad(
            exh_pos - flame_perp * core_w,
            exh_pos + flame_perp * core_w,
            core_tip + flame_perp * (core_w * 0.2),
            core_tip - flame_perp * (core_w * 0.2),
            Color::new(1.0, 0.95, 0.40, 0.98),
        );

        // Sparks ejecting backwards
        let spark1 = flame_tip + flame_dir * 0.12 + flame_perp * 0.05;
        let spark2 = flame_tip + flame_dir * 0.22 - flame_perp * 0.08;
        draw_circle(spark1.x, spark1.y, 0.035, Color::new(1.0, 0.85, 0.20, 0.95));
        draw_circle(spark2.x, spark2.y, 0.028, Color::new(1.0, 0.50, 0.10, 0.85));
    }

    // 6. --- Rear Wing / Ducktail Spoiler ---
    if tall_wing {
        // Trans-Am TA1 High-Mount Carbon GT Wing
        let wing_pos = pos - fwd * (half_len + 0.16);
        let wing_hw = half_w * 0.98;
        let wing_th = 0.16;

        // Uprights / stanchions connecting to rear deck
        let stanch_offset = half_w * 0.40;
        let s_l_start = pos - fwd * (half_len * 0.75) - right * stanch_offset;
        let s_l_end = wing_pos - right * stanch_offset;
        let s_r_start = pos - fwd * (half_len * 0.75) + right * stanch_offset;
        let s_r_end = wing_pos + right * stanch_offset;
        draw_line(s_l_start.x, s_l_start.y, s_l_end.x, s_l_end.y, 0.06, Color::new(0.20, 0.20, 0.24, 1.0));
        draw_line(s_r_start.x, s_r_start.y, s_r_end.x, s_r_end.y, 0.06, Color::new(0.20, 0.20, 0.24, 1.0));

        // Main wing blade
        let w0 = wing_pos + fwd * wing_th - right * wing_hw;
        let w1 = wing_pos + fwd * wing_th + right * wing_hw;
        let w2 = wing_pos - fwd * wing_th + right * wing_hw;
        let w3 = wing_pos - fwd * wing_th - right * wing_hw;
        draw_quad(w0, w1, w2, w3, Color::new(0.12, 0.12, 0.15, 0.98));

        // Aerodynamic endplates
        let ep_len = 0.24;
        let ep_col = color_scheme.primary;
        draw_line(
            (wing_pos - right * wing_hw + fwd * ep_len).x,
            (wing_pos - right * wing_hw + fwd * ep_len).y,
            (wing_pos - right * wing_hw - fwd * ep_len).x,
            (wing_pos - right * wing_hw - fwd * ep_len).y,
            0.08,
            ep_col,
        );
        draw_line(
            (wing_pos + right * wing_hw + fwd * ep_len).x,
            (wing_pos + right * wing_hw + fwd * ep_len).y,
            (wing_pos + right * wing_hw - fwd * ep_len).x,
            (wing_pos + right * wing_hw - fwd * ep_len).y,
            0.08,
            ep_col,
        );
    } else {
        // NASCAR Cup Ducktail / Blade Spoiler
        let spoiler_pos = pos - fwd * (half_len - 0.02);
        let spoiler_hw = half_w * 0.94;
        let spoiler_th = 0.14;

        // Polycarbonate blade / aluminum decklid spoiler
        let sp0 = spoiler_pos + fwd * spoiler_th - right * spoiler_hw;
        let sp1 = spoiler_pos + fwd * spoiler_th + right * spoiler_hw;
        let sp2 = spoiler_pos - fwd * spoiler_th + right * spoiler_hw;
        let sp3 = spoiler_pos - fwd * spoiler_th - right * spoiler_hw;
        draw_quad(sp0, sp1, sp2, sp3, Color::new(0.18, 0.20, 0.25, 0.95));

        // Top wickerbill lip / Gurney flap
        let gurney_pos = spoiler_pos - fwd * spoiler_th;
        draw_line(
            (gurney_pos - right * spoiler_hw).x,
            (gurney_pos - right * spoiler_hw).y,
            (gurney_pos + right * spoiler_hw).x,
            (gurney_pos + right * spoiler_hw).y,
            0.05,
            Color::new(0.85, 0.88, 0.95, 0.95),
        );

        // Turnbuckle adjustment braces supporting spoiler from rear decklid
        let tb1 = spoiler_pos - right * (spoiler_hw * 0.45);
        let tb2 = spoiler_pos + right * (spoiler_hw * 0.45);
        let tb1_base = tb1 + fwd * 0.22;
        let tb2_base = tb2 + fwd * 0.22;
        draw_line(tb1.x, tb1.y, tb1_base.x, tb1_base.y, 0.04, Color::new(0.65, 0.68, 0.72, 0.95));
        draw_line(tb2.x, tb2.y, tb2_base.x, tb2_base.y, 0.04, Color::new(0.65, 0.68, 0.72, 0.95));
    }

    // 7. --- Front Decal Headlights & Rear Taillights ---
    // NASCAR Printed Decal Headlights (authentic sticker look)
    let decal_w = half_w * 0.62;
    let head_l = pos + fwd * (half_len - 0.02) - right * decal_w;
    let head_r = pos + fwd * (half_len - 0.02) + right * decal_w;
    draw_circle(head_l.x, head_l.y, 0.13, Color::new(0.90, 0.92, 0.96, 0.85));
    draw_circle(head_r.x, head_r.y, 0.13, Color::new(0.90, 0.92, 0.96, 0.85));
    draw_circle(head_l.x, head_l.y, 0.08, Color::new(0.40, 0.45, 0.55, 0.90));
    draw_circle(head_r.x, head_r.y, 0.08, Color::new(0.40, 0.45, 0.55, 0.90));

    // Rear Taillights & Brake Lights
    let tail_w = half_w * 0.72;
    let tail_l = pos - fwd * (half_len - 0.04) - right * tail_w;
    let tail_r = pos - fwd * (half_len - 0.04) + right * tail_w;

    if is_braking {
        draw_circle(tail_l.x, tail_l.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
        draw_circle(tail_r.x, tail_r.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
        draw_circle(tail_l.x, tail_l.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
        draw_circle(tail_r.x, tail_r.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
    } else {
        draw_circle(tail_l.x, tail_l.y, 0.11, Color::new(0.60, 0.08, 0.08, 0.85));
        draw_circle(tail_r.x, tail_r.y, 0.11, Color::new(0.60, 0.08, 0.08, 0.85));
    }

    // Rear fuel cell filler cap (NASCAR red quick-fill cap on left rear quarter panel)
    let fuel_cap_pos = pos - fwd * (half_len * 0.65) - right * (half_w * 0.86);
    draw_circle(fuel_cap_pos.x, fuel_cap_pos.y, 0.07, Color::new(0.85, 0.12, 0.12, 0.95));
    draw_circle_lines(fuel_cap_pos.x, fuel_cap_pos.y, 0.07, 0.02, Color::new(0.2, 0.2, 0.2, 0.9));
}
