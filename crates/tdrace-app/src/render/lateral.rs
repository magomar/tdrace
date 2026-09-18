use macroquad::color::Color;
use macroquad::math::Vec2;
use macroquad::shapes::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle,
};

use super::color::{CarColorScheme, Palette};
use crate::ui::menu::CarChoice;

use crate::module::VehicleVisualType;

/// Adapts the module roster visual type to `render_car_lateral`.
#[allow(clippy::too_many_arguments)]
pub fn render_lateral_car(
    center_x: f32,
    center_y: f32,
    scale: f32,
    visual_type: VehicleVisualType,
    color_scheme: &CarColorScheme,
    brake_heat: f32,
    revving: bool,
    draw_reflection: bool,
) {
    let car_choice = match visual_type {
        VehicleVisualType::OpenWheel { .. } => CarChoice::F1Car,
        VehicleVisualType::GoKart { .. } => CarChoice::Kart,
        VehicleVisualType::RallyHatch { .. } => CarChoice::RallyCar,
        VehicleVisualType::StockCar { .. } => CarChoice::StockCar,
        VehicleVisualType::SandRail { .. } => CarChoice::SandRail,
        VehicleVisualType::TouringGT { .. } => CarChoice::GT3Car,
    };
    render_car_lateral(
        car_choice,
        color_scheme,
        center_x,
        center_y,
        scale,
        if revving { 1.0 } else { brake_heat },
        draw_reflection,
    );
}

/// Renders a high-fidelity 2D lateral (side-profile) vector illustration of a vehicle.
///
/// Front of the car points to the right (+X).
/// `center_x`, `center_y` is the visual center of the vehicle stage.
/// `scale` is the UI scaling multiplier (1.0 = ~180px length).
/// `rev_intensity` [0.0..1.0] heats up the brake calipers and animates exhaust heat.
/// `draw_reflection` toggles the polished showroom floor mirror reflection.
pub fn render_car_lateral(
    car_choice: CarChoice,
    color_scheme: &CarColorScheme,
    center_x: f32,
    center_y: f32,
    scale: f32,
    rev_intensity: f32,
    draw_reflection: bool,
) {
    let s = scale.max(0.2);
    let primary = color_scheme.primary;
    let secondary = color_scheme.secondary;
    let helmet_col = color_scheme.helmet;

    // Base dimensions
    let half_len = 80.0 * s;
    let ground_y = center_y + 18.0 * s;
    let wheel_y = ground_y - 12.0 * s;

    // Module-specific wheelbase offsets
    let (wf_x, wr_x, r_wheel) = match car_choice {
        CarChoice::F1Car => (center_x + half_len * 0.58, center_x - half_len * 0.52, 13.5 * s),
        CarChoice::Kart => (center_x + half_len * 0.45, center_x - half_len * 0.45, 9.0 * s),
        CarChoice::SandRail => (center_x + half_len * 0.52, center_x - half_len * 0.48, 14.5 * s),
        CarChoice::StockCar => (center_x + half_len * 0.54, center_x - half_len * 0.50, 13.0 * s),
        CarChoice::RallyCar => (center_x + half_len * 0.50, center_x - half_len * 0.48, 13.0 * s),
        CarChoice::HypercarPrototype | CarChoice::GT1Legend => (center_x + half_len * 0.56, center_x - half_len * 0.52, 13.0 * s),
        _ => (center_x + half_len * 0.52, center_x - half_len * 0.50, 12.5 * s),
    };

    // Ground contact shadow
    draw_circle(center_x, ground_y + 1.0 * s, half_len * 0.85, Color::new(0.0, 0.0, 0.0, 0.42));
    draw_circle(center_x, ground_y + 1.5 * s, half_len * 0.65, Color::new(0.0, 0.0, 0.0, 0.28));

    // Optional Showroom Mirror Floor Reflection (drawn inverted below ground_y)
    if draw_reflection {
        let refl_fade = Color::new(primary.r * 0.6, primary.g * 0.6, primary.b * 0.6, 0.15);
        let refl_w_col = Color::new(0.08, 0.09, 0.11, 0.22);

        // Inverted chassis reflection
        draw_rectangle(
            center_x - half_len * 0.70,
            ground_y + 2.0 * s,
            half_len * 1.40,
            12.0 * s,
            refl_fade,
        );
        // Inverted wheel shadows
        draw_circle(wf_x, ground_y + 6.0 * s, r_wheel * 0.75, refl_w_col);
        draw_circle(wr_x, ground_y + 6.0 * s, r_wheel * 0.75, refl_w_col);
    }

    // Floor contact baseline
    draw_line(
        center_x - half_len * 1.05,
        ground_y,
        center_x + half_len * 1.05,
        ground_y,
        1.0 * s,
        Color::new(0.25, 0.35, 0.50, 0.35),
    );

    // Draw Vehicle Category Silhouette
    match car_choice {
        CarChoice::F1Car => {
            render_lateral_f1(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::Kart => {
            render_lateral_kart(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::RallyCar => {
            render_lateral_rally(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::StockCar => {
            render_lateral_stock_car(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::SandRail => {
            render_lateral_sand_rail(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::HypercarPrototype => {
            render_lateral_hypercar(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::GT1Legend => {
            render_lateral_gt1(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::GT3Car => {
            render_lateral_gt3(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::GT2Biturbo => {
            render_lateral_gt2(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col);
        }
        CarChoice::GT4Clubsport | CarChoice::SportsCar | CarChoice::DriftCar => {
            render_lateral_touring_gt(center_x, center_y, ground_y, half_len, s, primary, secondary, helmet_col, car_choice);
        }
    }

    // Render Wheels with Alloy Spokes and Heat-Glow Calipers
    render_lateral_wheel(wf_x, wheel_y, r_wheel, s, rev_intensity);
    render_lateral_wheel(wr_x, wheel_y, r_wheel, s, rev_intensity);

    // Dynamic Rev Exhaust Backfire Sparks
    if rev_intensity > 0.05 {
        let exh_x = center_x - half_len * 0.90;
        let exh_y = ground_y - 8.0 * s;
        let flame_len = (18.0 * rev_intensity * s).min(28.0 * s);
        draw_triangle(
            Vec2::new(exh_x, exh_y - 2.5 * s),
            Vec2::new(exh_x, exh_y + 2.5 * s),
            Vec2::new(exh_x - flame_len, exh_y),
            Color::new(1.0, 0.45, 0.10, (0.6 + 0.4 * rev_intensity).min(1.0)),
        );
        draw_triangle(
            Vec2::new(exh_x, exh_y - 1.2 * s),
            Vec2::new(exh_x, exh_y + 1.2 * s),
            Vec2::new(exh_x - flame_len * 0.6, exh_y),
            Color::new(1.0, 0.95, 0.60, 0.95),
        );
    }
}

/// Renders a side-profile wheel with rim, rotor, spoke alloys, and dynamic brake caliper heat glow.
fn render_lateral_wheel(x: f32, y: f32, r: f32, s: f32, rev_intensity: f32) {
    // 1. Black tire rubber
    draw_circle(x, y, r, Color::new(0.10, 0.11, 0.13, 1.0));
    draw_circle_lines(x, y, r, 1.2 * s, Color::new(0.20, 0.22, 0.25, 1.0));

    // 2. White sidewall lettering ring / lip
    draw_circle_lines(x, y, r * 0.88, 0.8 * s, Color::new(0.40, 0.42, 0.46, 0.60));

    // 3. Rim well
    let r_rim = r * 0.72;
    draw_circle(x, y, r_rim, Color::new(0.18, 0.20, 0.24, 1.0));

    // 4. Brake rotor disc inside rim
    let r_rotor = r_rim * 0.78;
    draw_circle(x, y, r_rotor, Color::new(0.35, 0.38, 0.42, 1.0));
    draw_circle_lines(x, y, r_rotor, 0.8 * s, Color::new(0.55, 0.58, 0.62, 0.80));

    // 5. Brake caliper with glowing heat
    let cal_w = 4.0 * s;
    let cal_h = 7.0 * s;
    let cal_col = if rev_intensity > 0.05 {
        Color::new(
            1.0,
            0.20 + 0.60 * rev_intensity,
            0.05,
            0.85 + 0.15 * rev_intensity,
        )
    } else {
        Palette::RED
    };
    draw_rectangle(x + r_rotor * 0.45, y - cal_h * 0.5, cal_w, cal_h, cal_col);

    // 6. Alloy rim spokes (6-spoke star)
    for i in 0..6 {
        let angle = (i as f32) * std::f32::consts::PI / 3.0;
        let sx = x + angle.cos() * (r_rim * 0.90);
        let sy = y + angle.sin() * (r_rim * 0.90);
        draw_line(x, y, sx, sy, 1.4 * s, Color::new(0.85, 0.88, 0.92, 0.90));
    }

    // 7. Center hub nut
    draw_circle(x, y, 3.2 * s, Color::new(0.95, 0.95, 0.98, 1.0));
    draw_circle(x, y, 1.6 * s, Color::new(0.15, 0.16, 0.18, 1.0));
}

/// Standard Touring GT / Sports Coupe side profile.
fn render_lateral_touring_gt(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
    choice: CarChoice,
) {
    let nose_x = cx + hl * 0.92;
    let tail_x = cx - hl * 0.88;
    let sill_y = gy - 7.0 * s;
    let belt_y = cy + 2.0 * s;
    let roof_y = cy - 17.0 * s;

    // Lower chassis sill & splitter
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_rectangle(tail_x - 4.0 * s, sill_y + 4.0 * s, (nose_x - tail_x) + 8.0 * s, 2.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));

    // Main body trapezoid
    draw_triangle(
        Vec2::new(nose_x, sill_y),
        Vec2::new(cx + hl * 0.50, belt_y),
        Vec2::new(cx - hl * 0.50, belt_y),
        primary,
    );
    draw_rectangle(cx - hl * 0.65, belt_y, hl * 1.15, sill_y - belt_y, primary);
    draw_triangle(
        Vec2::new(cx - hl * 0.65, belt_y),
        Vec2::new(tail_x, sill_y),
        Vec2::new(cx - hl * 0.65, sill_y),
        primary,
    );

    // Livery accent stripe along waistline
    draw_rectangle(tail_x + 10.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 15.0 * s, 2.5 * s, secondary);

    // Greenhouse / Cabin / Windows
    let a_pillar_x = cx + hl * 0.28;
    let c_pillar_x = cx - hl * 0.38;
    draw_triangle(
        Vec2::new(cx + hl * 0.40, belt_y),
        Vec2::new(a_pillar_x, roof_y),
        Vec2::new(cx - hl * 0.15, roof_y),
        Color::new(0.12, 0.15, 0.20, 0.85),
    );
    draw_rectangle(cx - hl * 0.15, roof_y, (a_pillar_x - (cx - hl * 0.15)).abs().max(10.0 * s), belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_triangle(
        Vec2::new(c_pillar_x, belt_y),
        Vec2::new(cx - hl * 0.15, roof_y),
        Vec2::new(cx - hl * 0.15, belt_y),
        Color::new(0.12, 0.15, 0.20, 0.85),
    );

    // Driver helmet in cockpit
    draw_circle(cx - hl * 0.05, cy - 6.0 * s, 4.0 * s, helmet);

    // Roll cage bar
    draw_line(cx + hl * 0.20, belt_y - 1.0 * s, cx - hl * 0.25, roof_y + 3.0 * s, 1.2 * s, Color::new(0.85, 0.88, 0.95, 0.70));

    // Roof panel
    draw_rectangle(cx - hl * 0.20, roof_y - 2.0 * s, hl * 0.42, 2.5 * s, primary);

    // Rear Spoiler
    let wing_x = tail_x + 4.0 * s;
    if choice == CarChoice::GT4Clubsport {
        draw_line(wing_x, belt_y, wing_x, belt_y - 10.0 * s, 1.6 * s, Color::new(0.15, 0.16, 0.18, 1.0));
        draw_rectangle(wing_x - 4.0 * s, belt_y - 11.0 * s, 16.0 * s, 2.5 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    } else if choice == CarChoice::DriftCar {
        draw_triangle(
            Vec2::new(tail_x + 2.0 * s, belt_y),
            Vec2::new(tail_x - 3.0 * s, belt_y - 8.0 * s),
            Vec2::new(tail_x + 12.0 * s, belt_y),
            secondary,
        );
    }

    // Headlight lens & Taillight
    draw_triangle(
        Vec2::new(nose_x, sill_y - 2.0 * s),
        Vec2::new(nose_x - 8.0 * s, belt_y + 1.0 * s),
        Vec2::new(nose_x - 2.0 * s, belt_y + 4.0 * s),
        Color::new(0.70, 0.95, 1.0, 0.90),
    );
    draw_rectangle(tail_x, belt_y + 2.0 * s, 4.0 * s, 3.5 * s, Palette::RED);
}

/// FIA GT3 Evo racer with aggressive aerodynamics, dive planes, wide rear GT wing, and deep rear diffuser.
fn render_lateral_gt3(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 0.96;
    let tail_x = cx - hl * 0.90;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.5 * s;
    let roof_y = cy - 18.0 * s;

    // Carbon front splitter extending forward
    draw_rectangle(nose_x - 12.0 * s, sill_y + 4.0 * s, 18.0 * s, 2.5 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    // Front dive plane canards
    draw_line(nose_x - 6.0 * s, belt_y + 4.0 * s, nose_x + 2.0 * s, belt_y + 1.0 * s, 1.5 * s, secondary);

    // Lower chassis & side skirt
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_rectangle(tail_x - 6.0 * s, sill_y + 4.5 * s, (nose_x - tail_x) + 12.0 * s, 2.0 * s, Color::new(0.12, 0.13, 0.16, 1.0));

    // Main wedge profile
    draw_triangle(
        Vec2::new(nose_x, sill_y),
        Vec2::new(cx + hl * 0.48, belt_y),
        Vec2::new(cx - hl * 0.55, belt_y),
        primary,
    );
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.18, sill_y - belt_y, primary);
    draw_triangle(
        Vec2::new(cx - hl * 0.70, belt_y),
        Vec2::new(tail_x, sill_y),
        Vec2::new(cx - hl * 0.70, sill_y),
        primary,
    );

    // Secondary race livery graphics & competition number plate
    draw_rectangle(tail_x + 15.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 25.0 * s, 3.5 * s, secondary);
    draw_rectangle(cx - hl * 0.10, belt_y + 2.0 * s, 16.0 * s, 12.0 * s, Palette::WHITE);
    draw_rectangle_lines(cx - hl * 0.10, belt_y + 2.0 * s, 16.0 * s, 12.0 * s, 1.0 * s, Palette::BLACK);

    // Greenhouse canopy
    let a_pillar_x = cx + hl * 0.25;
    draw_triangle(
        Vec2::new(cx + hl * 0.38, belt_y),
        Vec2::new(a_pillar_x, roof_y),
        Vec2::new(cx - hl * 0.20, roof_y),
        Color::new(0.12, 0.15, 0.20, 0.88),
    );
    draw_rectangle(cx - hl * 0.20, roof_y, (a_pillar_x - (cx - hl * 0.20)).abs().max(10.0 * s), belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.88));
    draw_triangle(
        Vec2::new(cx - hl * 0.45, belt_y),
        Vec2::new(cx - hl * 0.20, roof_y),
        Vec2::new(cx - hl * 0.20, belt_y),
        Color::new(0.12, 0.15, 0.20, 0.88),
    );

    // Roll cage crossbar & driver helmet
    draw_line(cx + hl * 0.18, belt_y - 1.0 * s, cx - hl * 0.30, roof_y + 3.0 * s, 1.4 * s, Color::new(0.85, 0.88, 0.95, 0.75));
    draw_circle(cx - hl * 0.08, cy - 6.5 * s, 4.2 * s, helmet);

    // Roof & air intake roof scoop
    draw_rectangle(cx - hl * 0.22, roof_y - 2.0 * s, hl * 0.45, 2.5 * s, primary);
    draw_triangle(
        Vec2::new(cx - hl * 0.05, roof_y - 5.0 * s),
        Vec2::new(cx + hl * 0.10, roof_y - 2.0 * s),
        Vec2::new(cx - hl * 0.15, roof_y - 2.0 * s),
        secondary,
    );

    // Swan-neck Rear GT Wing
    let wing_pylon_x = tail_x + 8.0 * s;
    draw_line(wing_pylon_x, belt_y, wing_pylon_x - 3.0 * s, roof_y + 2.0 * s, 2.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_pylon_x - 12.0 * s, roof_y, 22.0 * s, 3.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    draw_rectangle(wing_pylon_x - 12.0 * s, roof_y - 3.0 * s, 2.5 * s, 8.0 * s, secondary);

    // Rear diffuser fins
    draw_line(tail_x - 5.0 * s, sill_y + 5.0 * s, tail_x - 1.0 * s, sill_y + 1.0 * s, 1.8 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    // Headlight & Taillight
    draw_triangle(
        Vec2::new(nose_x, sill_y - 1.5 * s),
        Vec2::new(nose_x - 10.0 * s, belt_y + 2.0 * s),
        Vec2::new(nose_x - 2.0 * s, belt_y + 5.0 * s),
        Palette::NEON_CYAN,
    );
    draw_rectangle(tail_x, belt_y + 2.0 * s, 4.0 * s, 3.5 * s, Palette::RED);
}

/// SRO GT2 Biturbo racer: long-tail aerodynamic body, aggressive power vents.
fn render_lateral_gt2(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    render_lateral_gt3(cx, cy, gy, hl, s, primary, secondary, helmet);
    let vent_x = cx - hl * 0.35;
    let vent_y = cy + 6.0 * s;
    draw_rectangle(vent_x, vent_y, 8.0 * s, 4.0 * s, Color::new(0.05, 0.05, 0.07, 0.95));
    draw_line(vent_x, vent_y + 2.0 * s, vent_x + 8.0 * s, vent_y + 2.0 * s, 1.0 * s, secondary);
}

/// 90s Le Mans GT1 Legend: ultra-low elongated tail with massive rear wing and low nose.
fn render_lateral_gt1(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 1.02;
    let tail_x = cx - hl * 0.98;
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.5 * s;
    let roof_y = cy - 14.0 * s;

    // Ultra-low front nose & long overhang
    draw_rectangle(nose_x - 14.0 * s, sill_y + 3.0 * s, 16.0 * s, 2.0 * s, Color::new(0.08, 0.08, 0.10, 1.0));
    draw_triangle(
        Vec2::new(nose_x, sill_y + 2.0 * s),
        Vec2::new(cx + hl * 0.40, belt_y),
        Vec2::new(cx - hl * 0.60, belt_y),
        primary,
    );
    draw_rectangle(cx - hl * 0.75, belt_y, hl * 1.30, sill_y - belt_y, primary);
    draw_triangle(
        Vec2::new(cx - hl * 0.75, belt_y),
        Vec2::new(tail_x, sill_y + 1.0 * s),
        Vec2::new(cx - hl * 0.75, sill_y),
        primary,
    );

    // Low central cockpit bubble
    draw_triangle(
        Vec2::new(cx + hl * 0.32, belt_y),
        Vec2::new(cx + hl * 0.15, roof_y),
        Vec2::new(cx - hl * 0.25, roof_y),
        Color::new(0.12, 0.16, 0.22, 0.90),
    );
    draw_triangle(
        Vec2::new(cx - hl * 0.45, belt_y),
        Vec2::new(cx - hl * 0.25, roof_y),
        Vec2::new(cx - hl * 0.25, belt_y),
        Color::new(0.12, 0.16, 0.22, 0.90),
    );
    draw_circle(cx - hl * 0.05, cy - 4.0 * s, 4.0 * s, helmet);

    // Giant GT1 Le Mans rear wing
    let wing_x = tail_x + 4.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.2 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_rectangle(wing_x - 10.0 * s, roof_y - 4.0 * s, 24.0 * s, 3.0 * s, secondary);
    draw_rectangle(wing_x - 10.0 * s, roof_y - 7.0 * s, 2.0 * s, 8.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
}

/// Le Mans Hypercar (LMH/LMDh) Prototype: aerodynamic cockpit dome and dorsal shark fin.
fn render_lateral_hypercar(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 1.00;
    let tail_x = cx - hl * 0.96;
    let sill_y = gy - 5.5 * s;
    let belt_y = cy + 3.0 * s;
    let roof_y = cy - 16.0 * s;

    // Sculpted low prototype nose
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 5.5 * s, primary);
    draw_triangle(
        Vec2::new(nose_x, sill_y + 1.0 * s),
        Vec2::new(cx + hl * 0.42, belt_y),
        Vec2::new(cx - hl * 0.60, belt_y),
        primary,
    );
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.25, sill_y - belt_y, primary);

    // Rounded aerodynamic cockpit bubble
    draw_circle(cx + hl * 0.02, cy - 6.0 * s, 10.0 * s, Color::new(0.10, 0.14, 0.18, 0.92));
    draw_circle(cx - hl * 0.02, cy - 6.0 * s, 4.0 * s, helmet);

    // Dorsal Shark Fin along engine spine!
    draw_triangle(
        Vec2::new(cx + hl * 0.05, roof_y),
        Vec2::new(cx - hl * 0.55, roof_y - 2.0 * s),
        Vec2::new(cx - hl * 0.55, belt_y),
        secondary,
    );

    // High Mount Hypercar Rear Wing
    let wing_x = tail_x + 6.0 * s;
    draw_line(wing_x, belt_y, wing_x, roof_y - 2.0 * s, 2.2 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(wing_x - 12.0 * s, roof_y - 4.0 * s, 26.0 * s, 3.2 * s, primary);
    draw_rectangle(wing_x - 12.0 * s, roof_y - 7.0 * s, 2.5 * s, 9.0 * s, secondary);
}

/// Open-Wheel Formula 1 car: stepped needle nose, front wing, exposed halo, high airbox, rear wing.
fn render_lateral_f1(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 1.06;
    let tail_x = cx - hl * 0.95;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 4.0 * s;
    let cockpit_y = cy - 6.0 * s;
    let airbox_y = cy - 18.0 * s;

    // Slender low monocoque chassis
    draw_rectangle(cx - hl * 0.60, sill_y, hl * 1.30, 5.5 * s, primary);
    // Pointed needle nose cone
    draw_triangle(
        Vec2::new(nose_x, sill_y + 2.0 * s),
        Vec2::new(cx + hl * 0.50, belt_y),
        Vec2::new(cx + hl * 0.50, sill_y),
        primary,
    );
    // Front wing multi-tier elements
    draw_rectangle(nose_x - 16.0 * s, sill_y + 3.0 * s, 20.0 * s, 2.2 * s, secondary);
    draw_rectangle(nose_x + 2.0 * s, sill_y - 2.0 * s, 2.0 * s, 7.0 * s, Color::new(0.15, 0.16, 0.18, 1.0));

    // Sidepod radiator intake
    draw_rectangle(cx - hl * 0.15, belt_y - 2.0 * s, hl * 0.45, sill_y - belt_y + 2.0 * s, secondary);

    // Cockpit opening & Driver Helmet
    draw_circle(cx + hl * 0.05, cockpit_y + 1.0 * s, 4.4 * s, helmet);

    // Cockpit Halo curved titanium bar
    draw_line(cx + hl * 0.22, belt_y - 2.0 * s, cx + hl * 0.08, airbox_y + 5.0 * s, 2.0 * s, Color::new(0.15, 0.16, 0.18, 1.0));
    draw_line(cx + hl * 0.08, airbox_y + 5.0 * s, cx - hl * 0.10, belt_y - 2.0 * s, 2.0 * s, Color::new(0.15, 0.16, 0.18, 1.0));

    // Airbox engine intake scoop rising behind driver's helmet
    draw_triangle(
        Vec2::new(cx - hl * 0.02, airbox_y),
        Vec2::new(cx - hl * 0.25, airbox_y + 8.0 * s),
        Vec2::new(cx - hl * 0.25, belt_y),
        primary,
    );
    draw_rectangle(cx - hl * 0.25, airbox_y + 2.0 * s, hl * 0.35, belt_y - (airbox_y + 2.0 * s), primary);

    // Exposed suspension wishbones (front and rear)
    let wf_x = cx + hl * 0.58;
    let wr_x = cx - hl * 0.52;
    draw_line(cx + hl * 0.45, sill_y + 1.0 * s, wf_x, gy - 12.0 * s, 1.5 * s, Color::new(0.70, 0.72, 0.76, 0.90));
    draw_line(cx + hl * 0.40, belt_y, wf_x, gy - 12.0 * s, 1.5 * s, Color::new(0.70, 0.72, 0.76, 0.90));
    draw_line(cx - hl * 0.40, sill_y + 1.0 * s, wr_x, gy - 12.0 * s, 1.5 * s, Color::new(0.70, 0.72, 0.76, 0.90));

    // High Bi-Plane Rear Wing
    let wing_x = tail_x + 5.0 * s;
    draw_line(wing_x, belt_y, wing_x, airbox_y + 1.0 * s, 2.4 * s, Color::new(0.15, 0.16, 0.18, 1.0));
    draw_rectangle(wing_x - 14.0 * s, airbox_y, 24.0 * s, 3.0 * s, secondary);
    draw_rectangle(wing_x - 12.0 * s, airbox_y + 5.0 * s, 20.0 * s, 2.0 * s, Color::new(0.20, 0.22, 0.25, 1.0));
    draw_rectangle(wing_x - 14.0 * s, airbox_y - 2.0 * s, 2.5 * s, 12.0 * s, primary);
}

/// Sprint Go-Kart side profile: ultra-low tubular chassis, exposed driver torso, direct steering column.
fn render_lateral_kart(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 0.75;
    let tail_x = cx - hl * 0.75;
    let sill_y = gy - 4.5 * s;

    // Low tubular chassis rail
    draw_line(tail_x, sill_y, nose_x, sill_y, 3.0 * s, Color::new(0.25, 0.28, 0.32, 1.0));

    // Front spoiler pod & Nassau steering panel
    draw_triangle(
        Vec2::new(nose_x + 5.0 * s, sill_y + 1.0 * s),
        Vec2::new(nose_x - 12.0 * s, sill_y - 8.0 * s),
        Vec2::new(nose_x - 12.0 * s, sill_y + 1.0 * s),
        primary,
    );
    draw_rectangle(nose_x - 18.0 * s, sill_y - 12.0 * s, 5.0 * s, 12.0 * s, secondary);

    // Side crash bumper pod
    draw_rectangle(cx - hl * 0.28, sill_y - 5.0 * s, hl * 0.56, 6.0 * s, primary);
    draw_rectangle_lines(cx - hl * 0.28, sill_y - 5.0 * s, hl * 0.56, 6.0 * s, 1.0 * s, Palette::BLACK);

    // Direct steering column and wheel
    draw_line(cx + hl * 0.18, sill_y, cx + hl * 0.05, cy - 8.0 * s, 2.0 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_circle(cx + hl * 0.05, cy - 8.0 * s, 3.5 * s, Color::new(0.15, 0.16, 0.18, 1.0));

    // Exposed driver bucket seat, torso, and helmet
    draw_rectangle(cx - hl * 0.25, cy - 4.0 * s, 8.0 * s, 12.0 * s, Color::new(0.10, 0.10, 0.12, 1.0));
    draw_triangle(
        Vec2::new(cx - hl * 0.18, sill_y),
        Vec2::new(cx - hl * 0.05, cy - 10.0 * s),
        Vec2::new(cx + hl * 0.08, sill_y),
        secondary,
    );
    draw_circle(cx - hl * 0.04, cy - 16.0 * s, 5.5 * s, helmet);
}

/// Rally Hatchback side profile: compact body, roof scoop, large rally wing, mudflaps.
fn render_lateral_rally(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 0.88;
    let tail_x = cx - hl * 0.84;
    let sill_y = gy - 7.5 * s;
    let belt_y = cy + 1.0 * s;
    let roof_y = cy - 20.0 * s;

    // Hatchback body box
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.0 * s, primary);
    draw_triangle(
        Vec2::new(nose_x, sill_y),
        Vec2::new(cx + hl * 0.45, belt_y),
        Vec2::new(cx - hl * 0.50, belt_y),
        primary,
    );
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.15, sill_y - belt_y, primary);
    draw_rectangle(tail_x, belt_y, hl * 0.25, sill_y - belt_y, primary);

    // Side livery splash
    draw_rectangle(tail_x + 8.0 * s, belt_y - 2.0 * s, (nose_x - tail_x) - 16.0 * s, 3.5 * s, secondary);

    // Windows (tall greenhouse)
    draw_triangle(
        Vec2::new(cx + hl * 0.35, belt_y),
        Vec2::new(cx + hl * 0.18, roof_y),
        Vec2::new(cx - hl * 0.15, roof_y),
        Color::new(0.12, 0.15, 0.20, 0.85),
    );
    draw_rectangle(cx - hl * 0.15, roof_y, 22.0 * s, belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_rectangle(cx - hl * 0.45, roof_y + 2.0 * s, 18.0 * s, belt_y - roof_y - 2.0 * s, Color::new(0.12, 0.15, 0.20, 0.85));
    draw_circle(cx - hl * 0.05, cy - 8.0 * s, 4.2 * s, helmet);

    // Roof & prominent Rally Air Scoop
    draw_rectangle(cx - hl * 0.20, roof_y - 2.0 * s, hl * 0.45, 2.5 * s, primary);
    draw_triangle(
        Vec2::new(cx - hl * 0.02, roof_y - 6.0 * s),
        Vec2::new(cx + hl * 0.12, roof_y - 2.0 * s),
        Vec2::new(cx - hl * 0.12, roof_y - 2.0 * s),
        secondary,
    );

    // Large WRC / RX Roof Spoiler
    draw_line(tail_x, roof_y, tail_x - 6.0 * s, roof_y - 6.0 * s, 2.5 * s, Color::new(0.12, 0.12, 0.15, 1.0));
    draw_rectangle(tail_x - 12.0 * s, roof_y - 8.0 * s, 16.0 * s, 3.0 * s, secondary);

    // Mudflaps behind rear wheel
    draw_rectangle(tail_x - 2.0 * s, sill_y, 2.5 * s, 8.0 * s, Color::new(0.85, 0.15, 0.18, 0.90));
}

/// NASCAR Cup / Trans-Am TA1 Stock Car side profile.
fn render_lateral_stock_car(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 0.94;
    let tail_x = cx - hl * 0.92;
    let sill_y = gy - 6.5 * s;
    let belt_y = cy + 1.0 * s;
    let roof_y = cy - 18.5 * s;

    // Muscular high beltline body
    draw_rectangle(tail_x, sill_y, nose_x - tail_x, 6.5 * s, primary);
    draw_triangle(
        Vec2::new(nose_x, sill_y),
        Vec2::new(cx + hl * 0.48, belt_y),
        Vec2::new(cx - hl * 0.55, belt_y),
        primary,
    );
    draw_rectangle(cx - hl * 0.70, belt_y, hl * 1.18, sill_y - belt_y, primary);
    draw_triangle(
        Vec2::new(cx - hl * 0.70, belt_y),
        Vec2::new(tail_x, sill_y + 1.0 * s),
        Vec2::new(cx - hl * 0.70, sill_y),
        primary,
    );

    // Big NASCAR door number plate
    draw_rectangle(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, Palette::WHITE);
    draw_rectangle_lines(cx - hl * 0.12, belt_y + 1.0 * s, 18.0 * s, 14.0 * s, 1.2 * s, secondary);

    // Windows & Driver Window Net
    let a_pillar_x = cx + hl * 0.22;
    draw_triangle(
        Vec2::new(cx + hl * 0.35, belt_y),
        Vec2::new(a_pillar_x, roof_y),
        Vec2::new(cx - hl * 0.20, roof_y),
        Color::new(0.12, 0.15, 0.20, 0.85),
    );
    draw_rectangle(cx - hl * 0.20, roof_y, (a_pillar_x - (cx - hl * 0.20)).abs().max(10.0 * s), belt_y - roof_y, Color::new(0.12, 0.15, 0.20, 0.85));
    // Window net criss-cross pattern
    for i in 0..4 {
        let lx = cx - hl * 0.10 + (i as f32) * 4.0 * s;
        draw_line(lx, roof_y + 2.0 * s, lx, belt_y - 1.0 * s, 1.0 * s, Color::new(0.08, 0.08, 0.10, 0.90));
    }
    draw_circle(cx - hl * 0.05, cy - 7.0 * s, 4.2 * s, helmet);

    // Roof Shark Fins
    draw_line(cx - hl * 0.05, roof_y - 1.0 * s, cx - hl * 0.20, roof_y - 5.0 * s, 1.5 * s, secondary);

    // Ducktail spoiler on trunk
    draw_line(tail_x + 2.0 * s, belt_y, tail_x - 4.0 * s, belt_y - 8.0 * s, 2.5 * s, Color::new(0.10, 0.10, 0.12, 1.0));

    // Boom-tube side exhaust exit in front of rear wheel
    let exh_x = cx - hl * 0.30;
    draw_rectangle(exh_x, sill_y + 3.0 * s, 8.0 * s, 2.5 * s, Color::new(0.35, 0.36, 0.40, 1.0));
}

/// Extreme Off-Road Sand Rail Buggy side profile.
fn render_lateral_sand_rail(
    cx: f32,
    cy: f32,
    gy: f32,
    hl: f32,
    s: f32,
    primary: Color,
    secondary: Color,
    helmet: Color,
) {
    let nose_x = cx + hl * 0.86;
    let tail_x = cx - hl * 0.82;
    let sill_y = gy - 9.0 * s;
    let cage_roof_y = cy - 20.0 * s;

    // Exposed tubular chromoly roll cage frame
    draw_line(nose_x, sill_y, cx - hl * 0.40, sill_y, 2.5 * s, primary);
    draw_line(cx + hl * 0.35, sill_y, cx + hl * 0.10, cage_roof_y, 2.5 * s, primary);
    draw_line(cx + hl * 0.10, cage_roof_y, cx - hl * 0.35, cage_roof_y, 2.5 * s, primary);
    draw_line(cx - hl * 0.35, cage_roof_y, cx - hl * 0.50, sill_y, 2.5 * s, primary);
    draw_line(cx - hl * 0.35, cage_roof_y, tail_x, sill_y, 2.0 * s, primary);

    // Visible long-travel coilover shock absorbers
    let wf_x = cx + hl * 0.52;
    let wr_x = cx - hl * 0.48;
    draw_line(cx + hl * 0.25, sill_y - 4.0 * s, wf_x, gy - 14.5 * s, 2.0 * s, secondary);
    draw_line(cx - hl * 0.30, sill_y - 4.0 * s, wr_x, gy - 14.5 * s, 2.0 * s, secondary);

    // Exposed rear turbo boxer engine
    draw_rectangle(tail_x + 4.0 * s, sill_y - 8.0 * s, 14.0 * s, 8.0 * s, Color::new(0.30, 0.32, 0.36, 1.0));
    draw_circle(tail_x + 8.0 * s, sill_y - 4.0 * s, 3.5 * s, Color::new(0.65, 0.68, 0.72, 1.0));

    // Driver in bucket seat
    draw_circle(cx - hl * 0.05, cy - 8.0 * s, 4.4 * s, helmet);

    // Roof 4-pod LED Lightbar
    draw_rectangle(cx - hl * 0.05, cage_roof_y - 3.5 * s, 14.0 * s, 3.0 * s, Palette::NEON_GOLD);

    // Whip antenna with safety pennant flag
    draw_line(tail_x + 2.0 * s, sill_y - 8.0 * s, tail_x - 12.0 * s, cage_roof_y - 12.0 * s, 1.0 * s, Palette::WHITE);
    draw_triangle(
        Vec2::new(tail_x - 12.0 * s, cage_roof_y - 12.0 * s),
        Vec2::new(tail_x - 6.0 * s, cage_roof_y - 8.0 * s),
        Vec2::new(tail_x - 12.0 * s, cage_roof_y - 4.0 * s),
        Palette::NEON_ORANGE,
    );
}
