use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_triangle};
use macroquad::texture::{draw_texture_ex, DrawTextureParams, Texture2D};
use tdrace_core::physics::car::Car;

use super::color::{CarColorScheme, Palette};
use super::lighting::{resolve_vehicle_lighting, VehicleLightingConfig};
use super::track::draw_quad;
use crate::module::VehicleVisualType;

static VANDORN_ARROWHEAD_PNG: &[u8] = include_bytes!("../../../../assets/textures/vehicles/topdown/gt/gt_vandorn_arrowhead_t2.png");
static VANDORN_ARROWHEAD_TOPDOWN_CACHE: std::sync::Mutex<Option<std::collections::HashMap<(u32, u32), Texture2D>>> = std::sync::Mutex::new(None);

#[inline]
fn color_to_u32(c: Color) -> u32 {
    let r = (c.r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (c.g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (c.b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

/// Retrieves or dynamically generates a colorway-tinted top-down texture for the Vandorn Arrowhead R T2.
pub fn get_tinted_vandorn_arrowhead_topdown(primary: Color, secondary: Color) -> Texture2D {
    let k1 = color_to_u32(primary);
    let k2 = color_to_u32(secondary);

    let mut guard = VANDORN_ARROWHEAD_TOPDOWN_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(std::collections::HashMap::new);
    if let Some(tex) = map.get(&(k1, k2)) {
        return tex.clone();
    }

    let base_img = macroquad::texture::Image::from_file_with_format(VANDORN_ARROWHEAD_PNG, None)
        .expect("failed to load vandorn arrowhead topdown PNG");
    let mut tinted = base_img.clone();
    for pixel in tinted.bytes.chunks_exact_mut(4) {
        let a = pixel[3];
        if a < 15 {
            continue;
        }
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let sat = if max_c > 0.001 { (max_c - min_c) / max_c } else { 0.0 };
        let lum = (r + g + b) / 3.0;

        if lum > 0.65 && sat < 0.22 {
            pixel[0] = ((primary.r * lum * 1.05).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = ((primary.g * lum * 1.05).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = ((primary.b * lum * 1.05).clamp(0.0, 1.0) * 255.0) as u8;
        } else if g > 0.58 && r > 0.45 && b < 0.45 && sat > 0.30 {
            pixel[0] = ((secondary.r * lum * 1.15).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = ((secondary.g * lum * 1.15).clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = ((secondary.b * lum * 1.15).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }

    let texture = Texture2D::from_image(&tinted);
    map.insert((k1, k2), texture.clone());
    texture
}

pub fn vandorn_arrowhead_texture() -> Texture2D {
    get_tinted_vandorn_arrowhead_topdown(Color::new(0.92, 0.92, 0.94, 1.0), Color::new(0.48, 0.85, 0.12, 1.0))
}

/// Renders vehicle lighting based on the resolved modality lighting profile.
pub fn render_car_lights(
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    is_braking: bool,
    cfg: &VehicleLightingConfig,
) {
    // 1. Projector LED Headlights with soft glow
    if cfg.has_headlights && cfg.lights_on {
        let light_w = half_w * 0.55;
        let head_l = pos + fwd * (half_len - 0.05) - right * light_w;
        let head_r = pos + fwd * (half_len - 0.05) + right * light_w;

        // Headlight outer glow
        let glow_col = Color::new(cfg.headlight_color.r, cfg.headlight_color.g, cfg.headlight_color.b, 0.35);
        draw_circle(head_l.x, head_l.y, 0.20, glow_col);
        draw_circle(head_r.x, head_r.y, 0.20, glow_col);
        // Headlight core
        draw_circle(head_l.x, head_l.y, 0.12, Color::new(1.0, 1.0, 1.0, 0.95));
        draw_circle(head_r.x, head_r.y, 0.12, Color::new(1.0, 1.0, 1.0, 0.95));
    }

    // 2. Tail / LED Brake Lights with glow halo
    if cfg.has_brake_lights {
        let tail_l = pos - fwd * (half_len - 0.05) - right * (half_w * 0.65);
        let tail_r = pos - fwd * (half_len - 0.05) + right * (half_w * 0.65);

        if is_braking {
            draw_circle(tail_l.x, tail_l.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
            draw_circle(tail_r.x, tail_r.y, 0.28, Color::new(1.0, 0.15, 0.15, 0.45));
            draw_circle(tail_l.x, tail_l.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
            draw_circle(tail_r.x, tail_r.y, 0.18, Color::new(1.0, 0.20, 0.20, 1.0));
        } else if cfg.lights_on {
            draw_circle(tail_l.x, tail_l.y, 0.11, Color::new(0.60, 0.08, 0.08, 0.85));
            draw_circle(tail_r.x, tail_r.y, 0.11, Color::new(0.60, 0.08, 0.08, 0.85));
        }
    }

    // 3. High-Mount Amber Dust Chase Strobe (SCORE / Baja Off-Road)
    if cfg.has_dust_chase_light && cfg.lights_on {
        crate::render::lighting::render_dust_chase_strobe(pos, fwd, half_len);
    }
}

/// On-screen size and rotation of a top-down chassis sprite: `(dest_w, dest_h, draw_angle)`.
///
/// The sprite spans 1.06x the body length. Portrait textures (nose up) are rotated a quarter turn.
pub fn topdown_sprite_dest(tex_w: f32, tex_h: f32, body_half_len: f32, angle: f32) -> (f32, f32, f32) {
    let w = body_half_len * 2.0 * 1.06;
    if tex_h > tex_w {
        let h = w * (tex_w / tex_h);
        (h, w, angle + std::f32::consts::FRAC_PI_2)
    } else {
        let h = w * (tex_h / tex_w);
        (w, h, angle)
    }
}

/// Renders a vehicle topdown textured sprite with modality-governed lighting.
pub fn render_vehicle_topdown_sprite(
    texture: &Texture2D,
    visual_center: Vec2,
    angle: f32,
    fwd: Vec2,
    right: Vec2,
    body_half_len: f32,
    body_half_w: f32,
    is_braking: bool,
    lighting_cfg: &VehicleLightingConfig,
) {
    let (dest_w, dest_h, draw_angle) = topdown_sprite_dest(texture.width(), texture.height(), body_half_len, angle);

    draw_texture_ex(
        texture,
        visual_center.x - dest_w * 0.5,
        visual_center.y - dest_h * 0.5,
        Color::new(1.0, 1.0, 1.0, 1.0),
        DrawTextureParams {
            dest_size: Some(macroquad::math::Vec2::new(dest_w, dest_h)),
            rotation: draw_angle,
            pivot: Some(macroquad::math::Vec2::new(visual_center.x, visual_center.y)),
            ..Default::default()
        },
    );

    render_car_lights(visual_center, fwd, right, body_half_len, body_half_w, is_braking, lighting_cfg);
}

/// Context passed to top-down sprite rendering when steered wheel animation is enabled.
pub struct SteeredWheelRenderContext<'a> {
    pub config: crate::render::vehicle_assets::SteeredWheelConfig,
    pub wheel_texture: &'a Texture2D,
    pub steer_fl: f32,
    pub steer_fr: f32,
}

/// Renders soft ground shadow for an individual steered wheel.
pub fn render_wheel_shadow(pos: Vec2, angle: f32, size: Vec2, alpha: f32) {
    let half_len = size.y * 0.5;
    let half_w = size.x * 0.5;
    let fwd = Vec2::new(angle.cos(), angle.sin());
    let right = Vec2::new(angle.sin(), -angle.cos());

    let p0 = pos + fwd * half_len - right * half_w;
    let p1 = pos + fwd * half_len + right * half_w;
    let p2 = pos - fwd * half_len + right * half_w;
    let p3 = pos - fwd * half_len - right * half_w;

    let shadow_color = Color::new(0.0, 0.0, 0.0, 0.38 * alpha);
    draw_quad(p0, p1, p2, p3, shadow_color);
}

/// Renders an individual standalone steered wheel sprite rotated to its absolute world angle.
pub fn draw_steered_wheel(
    texture: &Texture2D,
    pos: Vec2,
    angle: f32,
    size: Vec2,
) {
    let dest_w = size.x;
    let dest_h = size.y;
    draw_texture_ex(
        texture,
        pos.x - dest_w * 0.5,
        pos.y - dest_h * 0.5,
        Color::new(1.0, 1.0, 1.0, 1.0),
        DrawTextureParams {
            dest_size: Some(macroquad::math::Vec2::new(dest_w, dest_h)),
            rotation: angle + std::f32::consts::FRAC_PI_2,
            pivot: Some(macroquad::math::Vec2::new(pos.x, pos.y)),
            ..Default::default()
        },
    );
}

/// Renders an individual standalone steered wheel sprite rotated to its absolute world angle.
/// Deprecated in Spec 089: procedural compound accent stripes have been removed from world view.
pub fn draw_steered_wheel_with_accent(
    texture: &Texture2D,
    pos: Vec2,
    angle: f32,
    size: Vec2,
    _accent: Option<Color>,
    _is_left: bool,
) {
    draw_steered_wheel(texture, pos, angle, size);
}

/// Renders steered front wheels with ground shadows and exact Ackermann geometry.
pub fn render_steered_wheels(
    chassis_center: Vec2,
    angle: f32,
    fwd: Vec2,
    right: Vec2,
    ctx: &SteeredWheelRenderContext,
) {
    let p_fl = chassis_center + fwd * ctx.config.front_axle_offset - right * ctx.config.half_track_width;
    let p_fr = chassis_center + fwd * ctx.config.front_axle_offset + right * ctx.config.half_track_width;

    let ang_fl = angle + ctx.steer_fl;
    let ang_fr = angle + ctx.steer_fr;

    // 1. Render soft ground shadow beneath each steered wheel
    render_wheel_shadow(p_fl, ang_fl, ctx.config.wheel_size, 1.0);
    render_wheel_shadow(p_fr, ang_fr, ctx.config.wheel_size, 1.0);

    // 2. Render Left and Right Front Wheels
    draw_steered_wheel(ctx.wheel_texture, p_fl, ang_fl, ctx.config.wheel_size);
    draw_steered_wheel(ctx.wheel_texture, p_fr, ang_fr, ctx.config.wheel_size);
}

/// Renders a high-detail top-down sprite for the Vandorn Arrowhead R T2.
pub fn render_porsche_gt3r_sprite(
    chassis_center: Vec2,
    angle: f32,
    fwd: Vec2,
    right: Vec2,
    body_half_len: f32,
    body_half_w: f32,
    primary: Color,
    secondary: Color,
    is_braking: bool,
) {
    if let Some(texture) = crate::render::vehicle_assets::get_vehicle_topdown_texture("gt_vandorn_arrowhead_t2", primary, secondary) {
        let lighting_cfg = VehicleLightingConfig::gt_touring();
        render_vehicle_topdown_sprite(
            &texture,
            chassis_center,
            angle,
            fwd,
            right,
            body_half_len,
            body_half_w,
            is_braking,
            &lighting_cfg,
        );
    }
}

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
    render_car_with_visual_type_and_model(car, color_scheme, is_braking, visual_type, None);
}

/// Renders a vehicle with model-specific top-down sprite support when available,
/// falling back to procedural visual archetype rendering.
pub fn render_car_with_visual_type_and_model(
    car: &Car,
    color_scheme: &CarColorScheme,
    is_braking: bool,
    visual_type: VehicleVisualType,
    model_id: Option<&str>,
) {
    render_car_with_visual_type_model_and_shadows(
        car,
        color_scheme,
        is_braking,
        visual_type,
        model_id,
        true,
        true,
    );
}

/// Renders a vehicle with model-specific top-down sprite support and explicit shadow toggle.
pub fn render_car_with_visual_type_model_and_shadows(
    car: &Car,
    color_scheme: &CarColorScheme,
    is_braking: bool,
    visual_type: VehicleVisualType,
    model_id: Option<&str>,
    shadows_enabled: bool,
    lights_on: bool,
) {
    let pos = car.state.position;
    let angle = car.state.angle;
    let fwd = car.forward_vector();
    let right = car.right_vector();
    let z_jump = car.state.elevation.max(0.0);
    let z_ramp = car.state.ramp_elevation.max(0.0);
    let z_lift = z_jump + z_ramp;

    // Body roll and squat/dive offsets from local accelerations and road grade slope
    let roll_offset_lat = (-car.state.acceleration_local.y * 0.015).clamp(-0.18, 0.18);
    let grade_pitch_vis = (-car.state.road_grade_slope * 0.20).clamp(-0.12, 0.12);
    let pitch_offset_long = (car.state.acceleration_local.x * 0.012 + grade_pitch_vis).clamp(-0.18, 0.18);

    // 2.5D Elevation: airborne or on-ramp car rises along -Y screen projection and scales up slightly
    let elevation_lift = Vec2::new(0.0, -z_lift * 1.6);
    let air_scale = 1.0 + (z_lift * 0.07).min(0.35);

    let hub_center = pos + elevation_lift;
    let chassis_center = pos + right * roll_offset_lat + fwd * pitch_offset_long + elevation_lift;

    // Dimensions from car config & ChassisSkeleton
    let half_w = (car.config.track_width * 0.5) * air_scale;
    let lf = car.config.cg_to_front * air_scale;
    let lr = car.config.cg_to_rear * air_scale;
    let geom_offset = car.config.chassis.geometric_center_offset_from_cg(car.config.cg_to_front, car.config.cg_to_rear) * air_scale;
    let visual_center = chassis_center + fwd * geom_offset;
    let body_half_len = car.config.chassis.half_length(car.config.wheelbase) * air_scale;
    let body_half_w = car.config.chassis.half_width() * air_scale;

    let sprite = model_id.and_then(|m_id| {
        crate::render::vehicle_assets::get_vehicle_topdown_chassis_texture(m_id, color_scheme.primary, color_scheme.secondary)
            .map(|texture| (m_id, texture))
    });

    // 1. --- 2.5D Chassis-Conforming Ground Shadow (spec 096) ---
    let lift = shadow_lift(z_lift);
    let shadow_pos = pos + lift.offset + fwd * geom_offset;
    match select_shadow_path(shadows_enabled, sprite.is_some()) {
        ShadowPath::Off => {}
        ShadowPath::Sprite => {
            if let Some((_, texture)) = &sprite {
                render_sprite_chassis_shadow(texture, shadow_pos, angle, body_half_len * lift.scale, lift.alpha);
            }
        }
        ShadowPath::Archetype => {
            render_archetype_chassis_shadow(
                visual_type,
                shadow_pos,
                fwd,
                right,
                body_half_len * lift.scale,
                body_half_w * lift.scale,
                lift.alpha,
            );
        }
    }

    // Resolve modality-governed vehicle lighting profile
    let lighting_cfg = resolve_vehicle_lighting(model_id, visual_type).with_lights_on(lights_on);

    // Forward track illumination cone projected onto track surface ahead of vehicle
    if lighting_cfg.project_track_beams {
        crate::render::lighting::render_headlight_track_beams(
            visual_center,
            fwd,
            right,
            body_half_len,
            body_half_w,
            &lighting_cfg,
        );
    }

    // 2. If model-specific high-detail top-down sprite is available, render sprite directly
    if let Some((m_id, texture)) = sprite {
        let steered_cfg = crate::render::vehicle_assets::derive_steered_wheel_config(m_id, &car.config);
        let wheel_tex = steered_cfg.and_then(|cfg| crate::render::vehicle_assets::get_wheel_texture(cfg.wheel_texture_id));

        if let (Some(cfg), Some(ref w_tex)) = (steered_cfg, wheel_tex) {
            let (steer_fl, steer_fr) = car.compute_ackermann_angles(car.state.steer_angle);
            let lf = cfg.front_axle_offset * air_scale;
            let half_w = cfg.half_track_width * air_scale;
            let p_fl = visual_center + fwd * lf - right * half_w;
            let p_fr = visual_center + fwd * lf + right * half_w;
            let ang_fl = angle + steer_fl;
            let ang_fr = angle + steer_fr;
            let wheel_size = cfg.wheel_size * air_scale;
            let quad_size = glam::Vec2::new(
                wheel_size.x * cfg.texture_padding_factor.x,
                wheel_size.y * cfg.texture_padding_factor.y,
            );

            // 1. Wheel ground shadows
            if shadows_enabled {
                render_wheel_shadow(p_fl + lift.offset, ang_fl, wheel_size * lift.scale, lift.alpha);
                render_wheel_shadow(p_fr + lift.offset, ang_fr, wheel_size * lift.scale, lift.alpha);
            }

            // 2. UnderChassis wheels
            if cfg.layering == crate::render::vehicle_assets::WheelLayerMode::UnderChassis {
                draw_steered_wheel(w_tex, p_fl, ang_fl, quad_size);
                draw_steered_wheel(w_tex, p_fr, ang_fr, quad_size);
            }

            // 3. Chassis bodywork
            render_vehicle_topdown_sprite(
                &texture,
                visual_center,
                angle,
                fwd,
                right,
                body_half_len,
                body_half_w,
                is_braking,
                &lighting_cfg,
            );

            // 4. OverChassis wheels
            if cfg.layering == crate::render::vehicle_assets::WheelLayerMode::OverChassis {
                draw_steered_wheel(w_tex, p_fl, ang_fl, quad_size);
                draw_steered_wheel(w_tex, p_fr, ang_fr, quad_size);
            }

            return;
        }

        render_vehicle_topdown_sprite(
            &texture,
            visual_center,
            angle,
            fwd,
            right,
            body_half_len,
            body_half_w,
            is_braking,
            &lighting_cfg,
        );
        return;
    }

    // 3. Procedural Archetype Rendering
    let (steer_fl, steer_fr) = car.compute_ackermann_angles(car.state.steer_angle);
    let wheel_steers = [steer_fl, steer_fr, 0.0, 0.0];
    let wheel_positions = [
        hub_center + fwd * lf - right * half_w,
        hub_center + fwd * lf + right * half_w,
        hub_center - fwd * lr - right * half_w,
        hub_center - fwd * lr + right * half_w,
    ];


    match visual_type {
        VehicleVisualType::OpenWheel { front_wing_span, rear_wing_height, halo } => {
            // Render exposed suspension wishbones
            render_open_wheel_suspension(chassis_center, &wheel_positions, fwd, right);

            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], true);
            }

            // Monocoque, nosecone, sidepods
            render_open_wheel_body(visual_center, fwd, right, body_half_len, body_half_w, color_scheme, front_wing_span, rear_wing_height, halo, is_braking);
        }
        VehicleVisualType::GoKart { exposed_driver, side_bumpers } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], false);
            }
            render_kart_body(visual_center, fwd, right, body_half_len, body_half_w, color_scheme, exposed_driver, side_bumpers, is_braking);
        }
        VehicleVisualType::RallyHatch { roof_scoop, mudflaps, large_wing } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], false);
            }
            if mudflaps {
                render_mudflaps(visual_center, &wheel_positions, fwd, right);
            }
            render_rally_body(visual_center, fwd, right, body_half_len, body_half_w, color_scheme, roof_scoop, large_wing, is_braking, &lighting_cfg);
        }
        VehicleVisualType::TouringGT { .. } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], false);
            }
            render_chassis_body(visual_center, fwd, right, body_half_len, body_half_w, color_scheme);
            render_cockpit(visual_center, fwd, right, color_scheme);
            render_car_details(visual_center, fwd, right, body_half_len, body_half_w, is_braking, &lighting_cfg);
        }
        VehicleVisualType::StockCar { tall_wing, roof_fins, window_net } => {
            for i in 0..4 {
                render_wheel(wheel_positions[i], angle + wheel_steers[i], true);
            }
            render_stock_car_body(
                car,
                visual_center,
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
        VehicleVisualType::SandRail {
            lightbar,
            whip_antenna,
            paddle_tires,
        } => {
            let rear_half_w = half_w * 1.08;
            let sand_rail_wheels = [
                hub_center + fwd * lf - right * half_w,
                hub_center + fwd * lf + right * half_w,
                hub_center - fwd * lr - right * rear_half_w,
                hub_center - fwd * lr + right * rear_half_w,
            ];

            render_sand_rail_suspension(chassis_center, &sand_rail_wheels, fwd, right);

            // Front wheels (knobby / ribbed)
            render_sand_rail_wheel(sand_rail_wheels[0], angle + wheel_steers[0], false);
            render_sand_rail_wheel(sand_rail_wheels[1], angle + wheel_steers[1], false);
            // Rear wheels (paddles or knobbies)
            render_sand_rail_wheel(sand_rail_wheels[2], angle, paddle_tires);
            render_sand_rail_wheel(sand_rail_wheels[3], angle, paddle_tires);

            render_sand_rail_body(
                car,
                visual_center,
                fwd,
                right,
                body_half_len,
                body_half_w,
                color_scheme,
                lightbar,
                whip_antenna,
                paddle_tires,
                is_braking,
                &lighting_cfg,
            );
        }
    }
}

/// Ground-shadow placement for a vehicle lifted `z_lift` metres off the track (spec 096).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowLift {
    /// Ground-plane offset along the ambient light vector (+X, +Y).
    pub offset: Vec2,
    /// Outward scale of the shadow footprint.
    pub scale: f32,
    /// Opacity multiplier, floored at 0.25.
    pub alpha: f32,
}

/// Shadow offset, scale and opacity for a vehicle at elevation `z_lift` (jump + ramp, metres).
pub fn shadow_lift(z_lift: f32) -> ShadowLift {
    ShadowLift {
        offset: Vec2::new(0.06 + z_lift * 0.30, 0.08 + z_lift * 0.40),
        scale: 1.0 + (z_lift * 0.08).min(0.40),
        alpha: (1.0 / (1.0 + z_lift * 0.55)).clamp(0.25, 1.0),
    }
}

/// Which chassis shadow pass a vehicle gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowPath {
    /// Shadows disabled: no shadow draw calls at all.
    Off,
    /// Top-down sprite: the sprite's alpha silhouette, tinted black.
    Sprite,
    /// Procedural archetype: the archetype's body polygons.
    Archetype,
}

/// Picks the chassis shadow pass from the `vehicle_shadows` setting and sprite availability.
pub fn select_shadow_path(shadows_enabled: bool, has_topdown_sprite: bool) -> ShadowPath {
    match (shadows_enabled, has_topdown_sprite) {
        (false, _) => ShadowPath::Off,
        (true, true) => ShadowPath::Sprite,
        (true, false) => ShadowPath::Archetype,
    }
}

/// Opacity of the outer feathered penumbra layer (before the lift alpha).
pub const SHADOW_PENUMBRA_ALPHA: f32 = 0.12;
/// Opacity of the inner umbra core layer (before the lift alpha).
pub const SHADOW_CORE_ALPHA: f32 = 0.28;
/// Size of the sprite penumbra layer relative to the sprite.
pub const SPRITE_SHADOW_PENUMBRA_SCALE: f32 = 1.04;
/// Outward growth of each archetype polygon for the penumbra layer, in metres.
const ARCHETYPE_SHADOW_PENUMBRA_MARGIN: f32 = 0.06;

/// Sprite shadow layers, outer penumbra first: `(size scale, tint)`.
///
/// The tint is pure black, so only the sprite's alpha channel survives: the shadow is the
/// exact silhouette of the bodywork, wings and splitters included.
pub fn sprite_shadow_layers(alpha_scale: f32) -> [(f32, Color); 2] {
    [
        (SPRITE_SHADOW_PENUMBRA_SCALE, Color::new(0.0, 0.0, 0.0, SHADOW_PENUMBRA_ALPHA * alpha_scale)),
        (1.0, Color::new(0.0, 0.0, 0.0, SHADOW_CORE_ALPHA * alpha_scale)),
    ]
}

/// Draws the ground shadow of a top-down sprite vehicle from the sprite texture itself.
pub fn render_sprite_chassis_shadow(texture: &Texture2D, center: Vec2, angle: f32, body_half_len: f32, alpha_scale: f32) {
    let (dest_w, dest_h, draw_angle) = topdown_sprite_dest(texture.width(), texture.height(), body_half_len, angle);
    for (scale, tint) in sprite_shadow_layers(alpha_scale) {
        let (w, h) = (dest_w * scale, dest_h * scale);
        draw_texture_ex(
            texture,
            center.x - w * 0.5,
            center.y - h * 0.5,
            tint,
            DrawTextureParams {
                dest_size: Some(macroquad::math::Vec2::new(w, h)),
                rotation: draw_angle,
                pivot: Some(macroquad::math::Vec2::new(center.x, center.y)),
                ..Default::default()
            },
        );
    }
}

/// Local trapezoid from `x_front` (half width `hw_front`) back to `x_rear` (half width `hw_rear`).
fn shadow_trapezoid(x_front: f32, hw_front: f32, x_rear: f32, hw_rear: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(x_front, -hw_front),
        Vec2::new(x_front, hw_front),
        Vec2::new(x_rear, hw_rear),
        Vec2::new(x_rear, -hw_rear),
    ]
}

/// Local box centred at (`cx`, `cy`) with half length `hl` and half width `hw`.
fn shadow_box(cx: f32, cy: f32, hl: f32, hw: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(cx + hl, cy - hw),
        Vec2::new(cx + hl, cy + hw),
        Vec2::new(cx - hl, cy + hw),
        Vec2::new(cx - hl, cy - hw),
    ]
}

/// Convex polygons that make up the ground silhouette of a procedural archetype.
///
/// Coordinates are local: x along forward, y along right, in metres from the body centre.
/// The pieces follow the shapes the archetype body renderers draw, so the space around
/// exposed wheels stays open.
pub fn archetype_shadow_polygons(visual_type: VehicleVisualType, half_len: f32, half_w: f32) -> Vec<Vec<Vec2>> {
    let (l, w) = (half_len, half_w);
    match visual_type {
        VehicleVisualType::OpenWheel { front_wing_span, .. } => vec![
            shadow_box(l + 0.35, 0.0, 0.22, front_wing_span * 0.5),     // front wing
            shadow_trapezoid(l + 0.13, 0.13, l * 0.3, w * 0.35),        // nosecone and cockpit cell
            shadow_trapezoid(l * 0.3, w * 0.75, -l * 0.6, w * 0.65),    // sidepods
            shadow_trapezoid(-l * 0.6, 0.12, -(l - 0.05), 0.12),        // engine cover and gearbox
            shadow_box(-(l + 0.15), 0.0, 0.20, w * 0.85),               // rear wing
        ],
        VehicleVisualType::GoKart { side_bumpers, .. } => {
            let mut polys = vec![
                shadow_box(l * 0.65, 0.0, 0.25, 0.35),                  // nose fairing
                shadow_trapezoid(l * 0.65 - 0.25, 0.12, -l * 0.8, 0.12), // floorpan rail
                shadow_box(0.0, -w * 0.70, l * 0.40, 0.16),             // left side pod
                shadow_box(0.0, w * 0.70, l * 0.40, 0.16),              // right side pod
            ];
            if side_bumpers {
                // Nerf bars between the axles and a rear bar at the tail, clear of the tyres.
                polys.push(shadow_box(0.0, -w * 0.95, l * 0.40, 0.04)); // left nerf bar
                polys.push(shadow_box(0.0, w * 0.95, l * 0.40, 0.04));  // right nerf bar
                polys.push(shadow_box(-(l - 0.04), 0.0, 0.04, w * 0.95)); // rear bumper bar
            }
            polys
        }
        VehicleVisualType::TouringGT { .. } | VehicleVisualType::RallyHatch { .. } => {
            let mut polys = vec![
                vec![
                    Vec2::new(l, -w * 0.70),
                    Vec2::new(l, w * 0.70),
                    Vec2::new(l * 0.5, w),
                    Vec2::new(-l * 0.5, w),
                    Vec2::new(-l, w * 0.85),
                    Vec2::new(-l, -w * 0.85),
                    Vec2::new(-l * 0.5, -w),
                    Vec2::new(l * 0.5, -w),
                ],
                shadow_trapezoid(-l, w * 0.95, -(l + 0.17), w * 0.95), // carbon aero wing
            ];
            if let VehicleVisualType::RallyHatch { large_wing: true, .. } = visual_type {
                polys.push(shadow_trapezoid(-(l + 0.17), w * 0.90, -(l + 0.28), w * 0.90)); // rally spoiler
            }
            polys
        }
        VehicleVisualType::StockCar { .. } => vec![
            shadow_trapezoid(l + 0.18, w * 0.88 + 0.08, l, w * 0.88 + 0.08), // chin splitter
            vec![
                Vec2::new(l, -w * 0.88),
                Vec2::new(l, w * 0.88),
                Vec2::new(l * 0.55, w * 1.02),
                Vec2::new(0.0, w * 0.96),
                Vec2::new(0.0, -w * 0.96),
                Vec2::new(l * 0.55, -w * 1.02),
            ],
            vec![
                Vec2::new(0.0, -w * 0.96),
                Vec2::new(0.0, w * 0.96),
                Vec2::new(-l * 0.55, w * 1.02),
                Vec2::new(-l, w * 0.90),
                Vec2::new(-l, -w * 0.90),
                Vec2::new(-l * 0.55, -w * 1.02),
            ],
        ],
        VehicleVisualType::SandRail { .. } => vec![
            shadow_trapezoid(l * 0.95, w * 0.19, l * 0.40, w * 0.49),   // skid plate and prerunner bumper
            shadow_trapezoid(l * 0.40, w * 0.49, -l * 0.45, w * 0.58),  // belly pan and roll cage
            shadow_trapezoid(-l * 0.45, w * 0.50, -l * 0.88, w * 0.42), // rear engine cradle
        ],
    }
}

/// Centroid (vertex average) of a convex polygon.
fn polygon_centroid(poly: &[Vec2]) -> Vec2 {
    poly.iter().copied().sum::<Vec2>() / poly.len() as f32
}

/// Fills a convex local-space polygon placed at `center` in the (`fwd`, `right`) frame.
fn fill_local_polygon(poly: &[Vec2], center: Vec2, fwd: Vec2, right: Vec2, color: Color) {
    let to_world = |p: Vec2| {
        let q = center + fwd * p.x + right * p.y;
        macroquad::math::Vec2::new(q.x, q.y)
    };
    let c = to_world(polygon_centroid(poly));
    for i in 0..poly.len() {
        draw_triangle(c, to_world(poly[i]), to_world(poly[(i + 1) % poly.len()]), color);
    }
}

/// Draws the ground shadow of a procedural archetype from its body polygons:
/// a dilated faint penumbra under a full-opacity core.
pub fn render_archetype_chassis_shadow(
    visual_type: VehicleVisualType,
    center: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    alpha_scale: f32,
) {
    let polys = archetype_shadow_polygons(visual_type, half_len, half_w);
    let penumbra = Color::new(0.0, 0.0, 0.0, SHADOW_PENUMBRA_ALPHA * alpha_scale);
    let core = Color::new(0.0, 0.0, 0.0, SHADOW_CORE_ALPHA * alpha_scale);

    for poly in &polys {
        let c = polygon_centroid(poly);
        let dilated: Vec<Vec2> = poly
            .iter()
            .map(|&p| p + (p - c).normalize_or_zero() * ARCHETYPE_SHADOW_PENUMBRA_MARGIN)
            .collect();
        fill_local_polygon(&dilated, center, fwd, right, penumbra);
    }
    for poly in &polys {
        fill_local_polygon(poly, center, fwd, right, core);
    }
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
    let s_off = Vec2::new(0.03, 0.04);
    draw_quad(p0 + s_off, p1 + s_off, p2 + s_off, p3 + s_off, Palette::SOFT_SHADOW);

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

/// Renders exposed double-wishbone suspension arms for open-wheel vehicles.
fn render_open_wheel_suspension(_chassis: Vec2, wheels: &[Vec2; 4], _fwd: Vec2, _right: Vec2) {
    let arm_col = Color::new(0.18, 0.18, 0.22, 0.95);
    let th = 0.06;

    let front_hub = (wheels[0] + wheels[1]) * 0.5;
    let rear_hub = (wheels[2] + wheels[3]) * 0.5;

    // Front left & right wishbones
    draw_line(front_hub.x, front_hub.y, wheels[0].x, wheels[0].y, th, arm_col);
    draw_line(front_hub.x, front_hub.y, wheels[1].x, wheels[1].y, th, arm_col);
    // Rear left & right wishbones
    draw_line(rear_hub.x, rear_hub.y, wheels[2].x, wheels[2].y, th, arm_col);
    draw_line(rear_hub.x, rear_hub.y, wheels[3].x, wheels[3].y, th, arm_col);
}

/// Renders open-wheel aerodynamic body, wings, sidepods, and halo.
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
    cfg: &VehicleLightingConfig,
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

    render_car_details(pos, fwd, right, half_len, half_w, is_braking, cfg);
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
    cfg: &VehicleLightingConfig,
) {
    // 1. Render headlights and brake lights based on modality lighting profile
    render_car_lights(pos, fwd, right, half_len, half_w, is_braking, cfg);

    // 2. Modern Carbon Aero Wing
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

/// Draws an individual wheel for a sand rail buggy (paddle tire rear or knobby/ribbed front).
fn render_sand_rail_wheel(pos: Vec2, angle: f32, is_paddle: bool) {
    let tire_fwd = Vec2::new(angle.cos(), angle.sin());
    let tire_right = Vec2::new(angle.sin(), -angle.cos());

    let (tire_half_len, tire_half_w) = if is_paddle {
        (0.40, 0.20)
    } else {
        (0.34, 0.13)
    };

    let p0 = pos + tire_fwd * tire_half_len - tire_right * tire_half_w;
    let p1 = pos + tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p2 = pos - tire_fwd * tire_half_len + tire_right * tire_half_w;
    let p3 = pos - tire_fwd * tire_half_len - tire_right * tire_half_w;

    // Tire shadow
    let s_off = Vec2::new(0.03, 0.04);
    draw_quad(p0 + s_off, p1 + s_off, p2 + s_off, p3 + s_off, Palette::SOFT_SHADOW);

    // Tire rubber
    draw_quad(p0, p1, p2, p3, Color::new(0.12, 0.12, 0.14, 1.0));

    if is_paddle {
        // Sand paddle ribs across the tire tread
        for step in [-0.28, -0.14, 0.0, 0.14, 0.28] {
            let rib_c = pos + tire_fwd * step;
            let rib_l = rib_c - tire_right * (tire_half_w * 0.95);
            let rib_r = rib_c + tire_right * (tire_half_w * 0.95);
            draw_line(rib_l.x, rib_l.y, rib_r.x, rib_r.y, 0.045, Color::new(0.30, 0.30, 0.35, 1.0));
        }
    } else {
        // Front knobby / ribbed center line
        let rib_f = pos + tire_fwd * (tire_half_len * 0.9);
        let rib_b = pos - tire_fwd * (tire_half_len * 0.9);
        draw_line(rib_f.x, rib_f.y, rib_b.x, rib_b.y, 0.035, Color::new(0.28, 0.28, 0.32, 1.0));
    }

    // Beadlock rim center
    let hub_len = tire_half_len * 0.45;
    let hub_w = tire_half_w * 0.50;
    let h0 = pos + tire_fwd * hub_len - tire_right * hub_w;
    let h1 = pos + tire_fwd * hub_len + tire_right * hub_w;
    let h2 = pos - tire_fwd * hub_len + tire_right * hub_w;
    let h3 = pos - tire_fwd * hub_len - tire_right * hub_w;
    draw_quad(h0, h1, h2, h3, Palette::TIRE_RIM);
    // Beadlock ring bolts
    draw_circle_lines(pos.x, pos.y, hub_len.min(hub_w) * 0.85, 0.02, Palette::NEON_GOLD);
}

/// Renders exposed long-travel suspension A-arms and coilover shocks for sand rail buggy.
fn render_sand_rail_suspension(chassis: Vec2, wheels: &[Vec2; 4], fwd: Vec2, right: Vec2) {
    let arm_col = Color::new(0.22, 0.24, 0.28, 1.0);
    let shock_body = Color::new(0.85, 0.18, 0.15, 1.0); // Bright red coilover spring
    let shock_res = Color::new(0.95, 0.80, 0.10, 1.0); // Anodized gold piggyback reservoir

    // Front long-travel A-arms (dual wishbones)
    let front_bulkhead = (wheels[0] + wheels[1]) * 0.5;
    for (side, idx) in [(-1.0f32, 0), (1.0f32, 1)] {
        let wh = wheels[idx];
        let inner_upper = front_bulkhead + right * (side * 0.20) + fwd * 0.10;
        let inner_lower = front_bulkhead + right * (side * 0.18) - fwd * 0.10;
        draw_line(inner_upper.x, inner_upper.y, wh.x, wh.y, 0.05, arm_col);
        draw_line(inner_lower.x, inner_lower.y, wh.x, wh.y, 0.05, arm_col);

        // Coilover shock from shock tower to lower arm
        let shock_mount = front_bulkhead + right * (side * 0.12) + fwd * 0.05;
        let shock_end = wh * 0.65 + inner_lower * 0.35;
        draw_line(shock_mount.x, shock_mount.y, shock_end.x, shock_end.y, 0.07, shock_body);
        let res_pos = shock_mount * 0.6 + shock_end * 0.4 + right * (side * 0.05);
        draw_circle(res_pos.x, res_pos.y, 0.04, shock_res);
    }

    // Rear trailing arms
    let mid_chassis = chassis - fwd * 0.10;
    for (side, idx) in [(-1.0f32, 2), (1.0f32, 3)] {
        let wh = wheels[idx];
        let pivot = mid_chassis + right * (side * 0.35);
        draw_line(pivot.x, pivot.y, wh.x, wh.y, 0.065, arm_col);

        // Giant rear bypass / coilover shock
        let rear_shock_tower = chassis - fwd * 0.45 + right * (side * 0.25);
        draw_line(rear_shock_tower.x, rear_shock_tower.y, wh.x, wh.y, 0.08, shock_body);
        let res_pos = rear_shock_tower * 0.5 + wh * 0.5 + right * (side * 0.06);
        draw_circle(res_pos.x, res_pos.y, 0.05, shock_res);
    }
}

/// Renders an Extreme Off-Road Sand Rail Buggy with tubular chromoly spaceframe,
/// exposed rear flat-4 turbo engine, safety pennant whip antenna, and roof lightbar.
fn render_sand_rail_body(
    car: &Car,
    pos: Vec2,
    fwd: Vec2,
    right: Vec2,
    half_len: f32,
    half_w: f32,
    color_scheme: &CarColorScheme,
    lightbar: bool,
    whip_antenna: bool,
    _paddle_tires: bool,
    is_braking: bool,
    cfg: &VehicleLightingConfig,
) {
    let tube_shadow = Color::new(0.08, 0.08, 0.10, 1.0);
    let tube_color = color_scheme.primary;
    let tube_highlight = Color::new(
        (color_scheme.primary.r * 1.3).min(1.0),
        (color_scheme.primary.g * 1.3).min(1.0),
        (color_scheme.primary.b * 1.3).min(1.0),
        1.0,
    );

    let draw_tube = |p1: Vec2, p2: Vec2, thickness: f32| {
        draw_line(p1.x, p1.y, p2.x, p2.y, thickness + 0.03, tube_shadow);
        draw_line(p1.x, p1.y, p2.x, p2.y, thickness, tube_color);
        draw_line(p1.x, p1.y, p2.x, p2.y, thickness * 0.35, tube_highlight);
    };

    // 1. --- Aluminum Belly Pan & Floor Tub ---
    let tub_f = pos + fwd * (half_len * 0.40);
    let tub_r = pos - fwd * (half_len * 0.45);
    let tub_hw = half_w * 0.58;
    draw_quad(
        tub_f - right * (tub_hw * 0.85),
        tub_f + right * (tub_hw * 0.85),
        tub_r + right * tub_hw,
        tub_r - right * tub_hw,
        Color::new(0.18, 0.20, 0.24, 0.98),
    );
    // Aluminum side panels with secondary color accent
    let side_w = tub_hw * 0.16;
    draw_quad(
        tub_f - right * tub_hw,
        tub_f - right * (tub_hw - side_w),
        tub_r - right * (tub_hw - side_w),
        tub_r - right * tub_hw,
        color_scheme.secondary,
    );
    draw_quad(
        tub_f + right * (tub_hw - side_w),
        tub_f + right * tub_hw,
        tub_r + right * tub_hw,
        tub_r + right * (tub_hw - side_w),
        color_scheme.secondary,
    );

    // 2. --- Front Prerunner Bumper & Skid Plate ---
    let nose_tip = pos + fwd * (half_len * 0.95);
    let skid_base = pos + fwd * (half_len * 0.45);
    let skid_hw = half_w * 0.32;
    // Angled brushed aluminum skid plate
    draw_quad(
        nose_tip - right * (skid_hw * 0.6),
        nose_tip + right * (skid_hw * 0.6),
        skid_base + right * skid_hw,
        skid_base - right * skid_hw,
        Color::new(0.65, 0.68, 0.74, 0.95),
    );
    // Front prerunner bumper tubular loop
    let bmp_l = nose_tip - right * (skid_hw * 0.75) - fwd * 0.05;
    let bmp_r = nose_tip + right * (skid_hw * 0.75) - fwd * 0.05;
    let bmp_c = nose_tip + fwd * 0.12;
    draw_tube(bmp_l, bmp_c, 0.06);
    draw_tube(bmp_c, bmp_r, 0.06);
    draw_tube(bmp_l, skid_base - right * (skid_hw * 0.8), 0.05);
    draw_tube(bmp_r, skid_base + right * (skid_hw * 0.8), 0.05);
    // Dual front bumper amber fog lamps
    draw_circle((nose_tip - right * 0.10).x, (nose_tip - right * 0.10).y, 0.06, Color::new(1.0, 0.75, 0.10, 0.95));
    draw_circle((nose_tip + right * 0.10).x, (nose_tip + right * 0.10).y, 0.06, Color::new(1.0, 0.75, 0.10, 0.95));

    // 3. --- Tubular Chromoly Roll Cage Geometry ---
    let brow_l = pos + fwd * (half_len * 0.22) - right * (half_w * 0.45);
    let brow_r = pos + fwd * (half_len * 0.22) + right * (half_w * 0.45);
    let hoop_l = pos - fwd * (half_len * 0.28) - right * (half_w * 0.52);
    let hoop_r = pos - fwd * (half_len * 0.28) + right * (half_w * 0.52);
    let rear_cradle_l = pos - fwd * (half_len * 0.88) - right * (half_w * 0.42);
    let rear_cradle_r = pos - fwd * (half_len * 0.88) + right * (half_w * 0.42);

    // Front cage down-tubes to nose
    draw_tube(brow_l, skid_base - right * (skid_hw * 0.8), 0.06);
    draw_tube(brow_r, skid_base + right * (skid_hw * 0.8), 0.06);

    // Side intrusion bars
    draw_tube(skid_base - right * (skid_hw * 0.8), hoop_l, 0.06);
    draw_tube(skid_base + right * (skid_hw * 0.8), hoop_r, 0.06);

    // Roof lateral hoops
    draw_tube(brow_l, brow_r, 0.07);
    draw_tube(hoop_l, hoop_r, 0.07);
    draw_tube(brow_l, hoop_l, 0.07);
    draw_tube(brow_r, hoop_r, 0.07);

    // Roof 'X' cross-brace
    draw_tube(brow_l, hoop_r, 0.045);
    draw_tube(brow_r, hoop_l, 0.045);

    // Rear engine cradle down-tubes
    draw_tube(hoop_l, rear_cradle_l, 0.065);
    draw_tube(hoop_r, rear_cradle_r, 0.065);
    draw_tube(rear_cradle_l, rear_cradle_r, 0.065);

    // 4. --- Open Cockpit: Driver & Harness ---
    let cockpit_pos = pos - fwd * (half_len * 0.05);
    let seat_f = cockpit_pos + fwd * 0.20;
    let seat_r = cockpit_pos - fwd * 0.22;
    let seat_hw = half_w * 0.30;
    draw_quad(
        seat_f - right * seat_hw,
        seat_f + right * seat_hw,
        seat_r + right * seat_hw,
        seat_r - right * seat_hw,
        Color::new(0.12, 0.12, 0.14, 1.0),
    );
    // Steering wheel
    let wheel_pos = cockpit_pos + fwd * 0.26;
    draw_circle_lines(wheel_pos.x, wheel_pos.y, 0.11, 0.03, Color::new(0.25, 0.25, 0.30, 1.0));
    draw_circle(wheel_pos.x, wheel_pos.y, 0.03, Palette::RED);

    // Driver helmet with dark visor
    draw_circle(cockpit_pos.x, cockpit_pos.y, 0.18, color_scheme.helmet);
    let visor_pos = cockpit_pos + fwd * 0.07;
    let v_l = visor_pos - right * 0.12;
    let v_r = visor_pos + right * 0.12;
    draw_line(v_l.x, v_l.y, v_r.x, v_r.y, 0.07, Color::new(0.08, 0.08, 0.10, 0.95));

    // 5-point safety harness straps across shoulders
    let strap_col = Palette::NEON_ORANGE;
    draw_line((cockpit_pos - fwd * 0.15 - right * 0.08).x, (cockpit_pos - fwd * 0.15 - right * 0.08).y,
              (cockpit_pos + fwd * 0.06 - right * 0.05).x, (cockpit_pos + fwd * 0.06 - right * 0.05).y, 0.04, strap_col);
    draw_line((cockpit_pos - fwd * 0.15 + right * 0.08).x, (cockpit_pos - fwd * 0.15 + right * 0.08).y,
              (cockpit_pos + fwd * 0.06 + right * 0.05).x, (cockpit_pos + fwd * 0.06 + right * 0.05).y, 0.04, strap_col);

    // 5. --- Exposed Rear Flat-4 Turbocharged Boxer Engine ---
    let engine_pos = pos - fwd * (half_len * 0.58);
    let eng_hw = half_w * 0.32;
    let eng_len = 0.24;
    draw_quad(
        engine_pos + fwd * eng_len - right * eng_hw,
        engine_pos + fwd * eng_len + right * eng_hw,
        engine_pos - fwd * eng_len + right * eng_hw,
        engine_pos - fwd * eng_len - right * eng_hw,
        Color::new(0.35, 0.38, 0.42, 1.0),
    );
    // Finned boxer cylinder heads (horizontally opposed)
    let fin_col = Color::new(0.50, 0.54, 0.60, 1.0);
    draw_line((engine_pos - right * (eng_hw + 0.10)).x, (engine_pos - right * (eng_hw + 0.10)).y,
              (engine_pos - right * eng_hw).x, (engine_pos - right * eng_hw).y, 0.12, fin_col);
    draw_line((engine_pos + right * eng_hw).x, (engine_pos + right * eng_hw).y,
              (engine_pos + right * (eng_hw + 0.10)).x, (engine_pos + right * (eng_hw + 0.10)).y, 0.12, fin_col);

    // Turbocharger turbine housing & intercooler
    let turbo_pos = engine_pos + fwd * 0.10;
    draw_circle(turbo_pos.x, turbo_pos.y, 0.09, Color::new(0.60, 0.62, 0.68, 1.0));
    draw_circle(turbo_pos.x, turbo_pos.y, 0.05, Color::new(0.20, 0.20, 0.25, 1.0));

    // High-mount upward stinger exhaust pipes
    let stinger_tip_l = rear_cradle_l - fwd * 0.22 + right * 0.10;
    let stinger_tip_r = rear_cradle_r - fwd * 0.22 - right * 0.10;

    // Chrome header tubes merging into stingers
    draw_line(engine_pos.x - (eng_hw * 0.8), engine_pos.y, stinger_tip_l.x, stinger_tip_l.y, 0.05, Color::new(0.80, 0.82, 0.88, 1.0));
    draw_line(engine_pos.x + (eng_hw * 0.8), engine_pos.y, stinger_tip_r.x, stinger_tip_r.y, 0.05, Color::new(0.80, 0.82, 0.88, 1.0));

    // Hot glowing exhaust tips
    let tip_glow = if is_braking || car.state.speed > 25.0 {
        Color::new(1.0, 0.35, 0.10, 0.95)
    } else {
        Color::new(0.40, 0.42, 0.48, 1.0)
    };
    draw_circle(stinger_tip_l.x, stinger_tip_l.y, 0.045, tip_glow);
    draw_circle(stinger_tip_r.x, stinger_tip_r.y, 0.045, tip_glow);

    // Stinger exhaust backfire flames on deceleration / braking
    if is_braking && car.state.speed > 8.0 {
        for tip in [stinger_tip_l, stinger_tip_r] {
            let flame_len = 0.38 + (pos.x * 15.0).sin().abs() * 0.20;
            let flame_end = tip - fwd * flame_len;
            draw_line(tip.x, tip.y, flame_end.x, flame_end.y, 0.08, Color::new(1.0, 0.40, 0.05, 0.90));
            draw_line(tip.x, tip.y, (tip - fwd * (flame_len * 0.6)).x, (tip - fwd * (flame_len * 0.6)).y, 0.05, Color::new(1.0, 0.95, 0.30, 0.98));
        }
    }

    // 6. --- High-Intensity 4-Pod Roof Lightbar ---
    if lightbar || cfg.has_roof_lightbar {
        let bar_center = (brow_l + brow_r) * 0.5 + fwd * 0.04;
        let bar_hw = half_w * 0.40;
        draw_line((bar_center - right * bar_hw).x, (bar_center - right * bar_hw).y,
                  (bar_center + right * bar_hw).x, (bar_center + right * bar_hw).y, 0.05, Color::new(0.12, 0.12, 0.15, 1.0));

        for step in [-0.75, -0.25, 0.25, 0.75] {
            let pod_pos = bar_center + right * (bar_hw * step);
            draw_circle(pod_pos.x, pod_pos.y, 0.055, Color::new(0.18, 0.18, 0.22, 1.0));
            if cfg.lights_on {
                draw_circle(pod_pos.x, pod_pos.y, 0.038, Color::new(1.0, 0.98, 0.80, 0.98));
                let bloom_center = pod_pos + fwd * 0.08;
                draw_circle(bloom_center.x, bloom_center.y, 0.09, Color::new(1.0, 0.95, 0.70, 0.28));
            }
        }
    }

    // Rear high-mount dust chase strobe (amber safety strobe for extreme off-road / Baja)
    if cfg.has_dust_chase_light && cfg.lights_on {
        crate::render::lighting::render_dust_chase_strobe(pos, fwd, half_len);
    }

    // 7. --- Dynamic Whip Antenna with Neon Safety Pennant ---
    if whip_antenna {
        let whip_mount = hoop_l + right * 0.06 - fwd * 0.05;
        let speed_drag = (car.state.speed * 0.022).clamp(0.0, 0.55);
        let lat_deflection = (-car.state.angular_velocity * 0.12).clamp(-0.40, 0.40);
        let whip_len = 0.85;
        let whip_tip = whip_mount - fwd * (whip_len + speed_drag) + right * lat_deflection;

        draw_circle(whip_mount.x, whip_mount.y, 0.035, Color::new(0.25, 0.25, 0.28, 1.0));

        let mid_point = (whip_mount + whip_tip) * 0.5 - fwd * (speed_drag * 0.4);
        draw_line(whip_mount.x, whip_mount.y, mid_point.x, mid_point.y, 0.025, Color::new(0.85, 0.88, 0.92, 0.95));
        draw_line(mid_point.x, mid_point.y, whip_tip.x, whip_tip.y, 0.02, Color::new(0.85, 0.88, 0.92, 0.95));

        let flag_len = 0.26;
        let flag_w = 0.14;
        let flag_tip = whip_tip - fwd * flag_len;
        let flag_p1 = whip_tip + right * (flag_w * 0.5);
        let flag_p2 = whip_tip - right * (flag_w * 0.5);
        draw_triangle(
            macroquad::prelude::Vec2::new(flag_p1.x, flag_p1.y),
            macroquad::prelude::Vec2::new(flag_p2.x, flag_p2.y),
            macroquad::prelude::Vec2::new(flag_tip.x, flag_tip.y),
            Palette::NEON_ORANGE,
        );
    }
}

// ---------------------------------------------------------------------------
// Racing number plate (spec 092; placement and size by spec 111)
// ---------------------------------------------------------------------------

/// Advance width of one Rajdhani-Bold digit, in em.
pub const NUMBER_DIGIT_ADVANCE_EM: f32 = 0.56;
/// Height of a Rajdhani-Bold digit, in em.
pub const NUMBER_DIGIT_HEIGHT_EM: f32 = 0.70;
/// The number never fills more than this fraction of the plate, so the digits keep a margin to the body edge.
const NUMBER_PLATE_FILL: f32 = 0.9;
/// The digits are at most this fraction of the body width high (spec 111).
pub const NUMBER_MAX_HEIGHT_FRACTION: f32 = 0.35;
/// The roof number sits this fraction of the cabin length behind the cabin centre: the skeleton cabin includes
/// the windscreen, so its centre is ahead of the middle of the roof on the sprites (spec 111 contact sheets).
pub const NUMBER_ROOF_REARWARD: f32 = 0.15;
/// The front number sits this fraction of the front zone ahead of the front edge of the cabin: on the bonnet,
/// clear of the rounded nose.
pub const NUMBER_FRONT_POSITION: f32 = 0.35;
/// Smallest digit height, in screen pixels, that is still drawn.
const NUMBER_MIN_PIXEL_HEIGHT: f32 = 5.0;

/// Where a kind of car carries its racing number in the top-down view (spec 111).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlateZone {
    /// Between the front edge of the cabin and the front end of the body, nearer the cabin: the bonnet.
    Front,
    /// The middle of the roof, a little behind the cabin centre.
    Roof,
}

/// The zone for a kind of car, `None` when it shows no number on its body: karts (`GoKart`, and `OpenWheel`, which
/// only the superkarts use) carry none; buggies and GT cars carry it at the front; stock cars and rally cars on
/// the roof.
pub fn number_plate_zone(visual_type: VehicleVisualType) -> Option<PlateZone> {
    match visual_type {
        VehicleVisualType::GoKart { .. } | VehicleVisualType::OpenWheel { .. } => None,
        VehicleVisualType::SandRail { .. } | VehicleVisualType::TouringGT { .. } => Some(PlateZone::Front),
        VehicleVisualType::StockCar { .. } | VehicleVisualType::RallyHatch { .. } => Some(PlateZone::Roof),
    }
}

/// Where the racing number sits on a car and how much room it has, in the car's own frame (metres).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberPlate {
    /// Centre of the plate, measured forward from the geometric centre of the body.
    pub forward_offset: f32,
    /// Half of the plate along the heading: bounded by the length of its zone.
    pub half_len: f32,
    /// Half of the plate across the car: sized so the digits are at most `NUMBER_MAX_HEIGHT_FRACTION` of the body
    /// width high.
    pub half_width: f32,
}

/// The number plate for a chassis in `zone`. Roof: `NUMBER_ROOF_REARWARD` of the cabin length behind the cabin
/// centre. Front: `NUMBER_FRONT_POSITION` of the way from the front edge of the cabin to the front end of the body.
/// The plate never reaches past the ends of its zone. Pure: only the `ChassisSkeleton` and the axle positions are
/// read.
pub fn number_plate(
    chassis: &tdrace_core::physics::config::ChassisSkeleton,
    wheelbase: f32,
    cg_to_front: f32,
    cg_to_rear: f32,
    zone: PlateZone,
) -> NumberPlate {
    let geom_offset = chassis.geometric_center_offset_from_cg(cg_to_front, cg_to_rear);
    let cabin_front = cg_to_front + chassis.cabin_start_offset;
    let cabin_rear = -cg_to_rear + chassis.cabin_end_offset;
    let body_front = cg_to_front + chassis.front_overhang;
    let body_half_len = chassis.half_length(wheelbase);
    let has_cabin = cabin_front - cabin_rear > 0.1;
    // The zone as (rear end, front end, where the plate centre sits in it from the rear end).
    let (zone_rear, zone_front, position) = match zone {
        PlateZone::Roof if has_cabin => (cabin_rear, cabin_front, 0.5 - NUMBER_ROOF_REARWARD),
        // A skeleton whose cabin edges cross has no cabin; fall back to the middle third of the body.
        PlateZone::Roof => (geom_offset - body_half_len / 3.0, geom_offset + body_half_len / 3.0, 0.5),
        PlateZone::Front if has_cabin && body_front - cabin_front > 0.1 => (cabin_front, body_front, NUMBER_FRONT_POSITION),
        // No cabin, or a cabin that reaches the nose: the front third of the body.
        PlateZone::Front => (geom_offset + body_half_len / 3.0, body_front, NUMBER_FRONT_POSITION),
    };
    let centre = zone_rear + (zone_front - zone_rear) * position;
    let forward_offset = centre - geom_offset;
    // The plate stays inside its zone and on the body.
    let room_in_zone = (centre - zone_rear).min(zone_front - centre);
    let room_to_end = (body_half_len - forward_offset.abs()).max(0.0);
    NumberPlate {
        forward_offset,
        half_len: room_in_zone.min(room_to_end).max(0.0),
        half_width: chassis.half_width() * NUMBER_MAX_HEIGHT_FRACTION / NUMBER_PLATE_FILL,
    }
}

/// Size of the digits, in em (metres of font size), so `digits` characters fit inside `plate` with a margin.
pub fn number_em_size(plate: &NumberPlate, digits: usize) -> f32 {
    let digits = digits.max(1) as f32;
    let by_length = 2.0 * plate.half_len * NUMBER_PLATE_FILL / (digits * NUMBER_DIGIT_ADVANCE_EM);
    let by_width = 2.0 * plate.half_width * NUMBER_PLATE_FILL / NUMBER_DIGIT_HEIGHT_EM;
    by_length.min(by_width).max(0.0)
}

/// Brightness of a colour, 0.0 (black) to 1.0 (white): Rec. 709 weights on the colour values.
pub fn color_luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// Glyph and outline colours for a number drawn on a body of colour `body`: white on a dark body with a black
/// outline, black on a light body with a white outline (luminance 0.5 counts as light).
pub fn number_glyph_colors(body: Color) -> (Color, Color) {
    if color_luminance(body) < 0.5 {
        (Palette::WHITE, Color::new(0.0, 0.0, 0.0, 1.0))
    } else {
        (Color::new(0.0, 0.0, 0.0, 1.0), Palette::WHITE)
    }
}

/// A racing number to draw on one car: where, how big and in what colours, in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarNumberItem {
    pub number: u8,
    /// World position of the plate centre.
    pub center: Vec2,
    /// World heading of the car, radians.
    pub angle: f32,
    /// Size of the digits, in em (world metres).
    pub em: f32,
    /// Effective primary colour of the body under the number.
    pub body_color: Color,
}

impl CarNumberItem {
    /// The number plate of `car`, placed on the body as `render_car_with_visual_type_model_and_shadows` places it
    /// (roll, pitch and jump lift included), in the zone of its `visual_type`. `None` when `number` is not a valid
    /// racing number or the kind of car shows no number on its body.
    pub fn for_car(car: &Car, number: u8, body_color: Color, visual_type: VehicleVisualType) -> Option<Self> {
        if !crate::ai::is_valid_race_number(number) {
            return None;
        }
        let zone = number_plate_zone(visual_type)?;
        let fwd = car.forward_vector();
        let right = car.right_vector();
        let z_lift = car.state.elevation.max(0.0) + car.state.ramp_elevation.max(0.0);
        let roll_offset_lat = (-car.state.acceleration_local.y * 0.015).clamp(-0.18, 0.18);
        let grade_pitch_vis = (-car.state.road_grade_slope * 0.20).clamp(-0.12, 0.12);
        let pitch_offset_long = (car.state.acceleration_local.x * 0.012 + grade_pitch_vis).clamp(-0.18, 0.18);
        let air_scale = 1.0 + (z_lift * 0.07).min(0.35);
        let plate = number_plate(&car.config.chassis, car.config.wheelbase, car.config.cg_to_front, car.config.cg_to_rear, zone);
        let geom_offset = car.config.chassis.geometric_center_offset_from_cg(car.config.cg_to_front, car.config.cg_to_rear);
        let center = car.state.position
            + right * roll_offset_lat
            + fwd * pitch_offset_long
            + Vec2::new(0.0, -z_lift * 1.6)
            + fwd * ((geom_offset + plate.forward_offset) * air_scale);
        let digits = if number >= 10 { 2 } else { 1 };
        Some(Self { number, center, angle: car.state.angle, em: number_em_size(&plate, digits) * air_scale, body_color })
    }
}

/// Draws racing numbers on cars in screen space: projected through `camera`, rotated with the car, glyphs by
/// the contrast rule. Numbers whose digits would be under 5 px high are skipped, as are cars outside the viewport.
pub fn render_car_numbers(
    fonts: &crate::ui::font::Fonts,
    camera: &crate::camera::RaceCamera,
    viewport_rect: Option<(f32, f32, f32, f32)>,
    items: &[CarNumberItem],
) {
    let Some(font) = fonts.display.as_ref() else {
        return;
    };
    let (sw, sh) = crate::camera::RaceCamera::get_screen_dimensions_safe();
    let (vp_x, vp_y, vp_w, vp_h) = viewport_rect.unwrap_or((0.0, 0.0, sw, sh));
    for item in items {
        let em_px = item.em * camera.current_zoom;
        if em_px * NUMBER_DIGIT_HEIGHT_EM < NUMBER_MIN_PIXEL_HEIGHT {
            continue;
        }
        let center = camera.world_to_screen_with_viewport(item.center, sw, sh);
        let reach = em_px * 1.5;
        if center.x - reach < vp_x || center.x + reach > vp_x + vp_w || center.y - reach < vp_y || center.y + reach > vp_y + vp_h {
            continue;
        }
        let (glyph, outline) = number_glyph_colors(item.body_color);
        let text = item.number.to_string();
        let _ = std::panic::catch_unwind(|| draw_rotated_number(font, &text, center, -item.angle, em_px, glyph, outline));
    }
}

/// Draws `text` centred on `center`, its baseline running along the screen angle `rotation` (clockwise, radians).
fn draw_rotated_number(font: &macroquad::text::Font, text: &str, center: Vec2, rotation: f32, em_px: f32, glyph: Color, outline: Color) {
    let raster = em_px.round().clamp(8.0, 512.0);
    let scale = em_px / raster;
    let params = |color: Color| macroquad::text::TextParams {
        font: Some(font),
        font_size: raster as u16,
        font_scale: scale,
        color,
        rotation,
        ..Default::default()
    };
    let size = macroquad::text::measure_text(text, Some(font), raster as u16, scale);
    // The text box is `size.width` along the baseline and `size.offset_y` above it; centre it on `center`.
    let (sin, cos) = rotation.sin_cos();
    let local = Vec2::new(-size.width * 0.5, size.offset_y * 0.5);
    let origin = center + Vec2::new(local.x * cos - local.y * sin, local.x * sin + local.y * cos);
    let reach = (em_px * 0.06).max(1.0);
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (-0.7, -0.7), (0.7, -0.7), (-0.7, 0.7), (0.7, 0.7)] {
        macroquad::text::draw_text_ex(text, origin.x + dx * reach, origin.y + dy * reach, params(outline));
    }
    macroquad::text::draw_text_ex(text, origin.x, origin.y, params(glyph));
}
