use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_triangle};
use glam::Vec2;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::SurfaceShape;
use tdrace_core::track::spline::TrackSpline;
use tdrace_core::track::Track;

use super::color::Palette;

/// Renders the complete track geometry including asphalt ribbon, rumble curbs,
/// sand traps, jump ramps, pit lanes, grid boxes, and start/finish line checkerboard.
pub fn render_track(track: &Track) {
    // 1. Render base off-track surface zones (sand traps, asphalt runoff, dirt areas)
    render_surface_zones(track);

    // 2. Render pit box area if defined
    if let Some(pit_area) = &track.pit_box_area {
        render_surface_shape(pit_area, Palette::PIT_LANE, Some(Palette::WHITE_LINE));
    }

    // 3. Render the asphalt/dirt track ribbon and rumble curbs
    render_track_ribbon(&track.spline);

    // 4. Render on-top surface hazard patches (water puddles, oil slicks) so they visibly overlay the track
    render_on_track_hazard_zones(track);

    // 5. Render 2.5D jump ramps
    render_jump_ramps(track);

    // 6. Render starting grid slots
    render_starting_grid(track);

    // 7. Render start/finish line checkerboard
    render_finish_line(track);
}

/// Draws dynamic on-track hazard overlays like shimmering water puddles or oil slicks.
fn render_on_track_hazard_zones(track: &Track) {
    for zone in &track.geometry.surface_zones {
        if matches!(zone.surface, SurfaceType::Water | SurfaceType::Oil | SurfaceType::Ice) {
            let (fill_col, border_col) = match zone.surface {
                SurfaceType::Water => (Palette::WATER, Some(Palette::WATER_BORDER)),
                SurfaceType::Ice => (Color::new(0.85, 0.92, 0.98, 0.8), None),
                SurfaceType::Oil => (Color::new(0.12, 0.12, 0.15, 0.85), None),
                _ => continue,
            };
            render_surface_shape(&zone.shape, fill_col, border_col);
        }
    }
}

/// Draws all custom surface zones (e.g. sand traps, dirt areas, water hazards).
fn render_surface_zones(track: &Track) {
    for zone in &track.geometry.surface_zones {
        let (fill_col, border_col) = match zone.surface {
            SurfaceType::Sand => (Palette::SAND, Some(Palette::SAND_DARK)),
            SurfaceType::Dirt => (Palette::DIRT, Some(Palette::DIRT_DARK)),
            SurfaceType::Water => (Palette::WATER, Some(Palette::WATER_BORDER)),
            SurfaceType::Asphalt => (Palette::RUNOFF_ASPHALT, Some(Palette::WHITE_LINE)),
            SurfaceType::Grass => (Palette::GRASS_DARK, None),
            SurfaceType::Curb => (Palette::CURB_RED, None),
            SurfaceType::Ice => (Color::new(0.85, 0.92, 0.98, 0.8), None),
            SurfaceType::Oil => (Color::new(0.12, 0.12, 0.15, 0.85), None),
        };
        render_surface_shape(&zone.shape, fill_col, border_col);
    }
}
pub fn render_jump_ramps(track: &Track) {
    for ramp in &track.geometry.jump_ramps {
        match &ramp.shape {
            SurfaceShape::OrientedBox {
                center,
                half_extents,
                angle,
            } => {
                let fwd = Vec2::new(angle.cos(), angle.sin()) * half_extents.x;
                let right = Vec2::new(-angle.sin(), angle.cos()) * half_extents.y;
                let p0 = *center - fwd - right;
                let p1 = *center + fwd - right;
                let p2 = *center + fwd + right;
                let p3 = *center - fwd + right;

                // 1. Drop shadow beneath ramp platform
                let s_off = Vec2::new(0.40, 0.55);
                draw_quad(p0 + s_off, p1 + s_off, p2 + s_off, p3 + s_off, Palette::SHADOW);

                // 2. Base metallic ramp quad
                let ramp_base_col = Color::new(0.24, 0.26, 0.30, 1.0);
                draw_quad(p0, p1, p2, p3, ramp_base_col);

                // 3. Directional hazard chevron stripes (yellow / black)
                let num_stripes = 4;
                for s in 0..num_stripes {
                    let t0 = s as f32 / num_stripes as f32;
                    let t1 = (s as f32 + 0.5) / num_stripes as f32;
                    let s_p0 = p0.lerp(p1, t0);
                    let s_p1 = p0.lerp(p1, t1);
                    let s_p2 = p3.lerp(p2, t1);
                    let s_p3 = p3.lerp(p2, t0);

                    let stripe_col = if s % 2 == 0 {
                        Color::new(0.98, 0.82, 0.12, 0.95) // Neon caution yellow
                    } else {
                        Color::new(0.12, 0.12, 0.15, 0.95) // Dark charcoal
                    };
                    draw_quad(s_p0, s_p1, s_p2, s_p3, stripe_col);
                }

                // 4. Elevated launch lip line at exit edge (bright cyan glow)
                let launch_edge_col = Color::new(0.30, 0.95, 1.0, 1.0);
                draw_line(p1.x, p1.y, p2.x, p2.y, 0.45, launch_edge_col);

                // 5. Ramp side border rails
                draw_line(p0.x, p0.y, p1.x, p1.y, 0.30, Color::new(0.85, 0.85, 0.90, 1.0));
                draw_line(p3.x, p3.y, p2.x, p2.y, 0.30, Color::new(0.85, 0.85, 0.90, 1.0));
                draw_line(p0.x, p0.y, p3.x, p3.y, 0.30, Color::new(0.60, 0.60, 0.65, 1.0));
            }
            _ => {
                render_surface_shape(&ramp.shape, Color::new(0.95, 0.80, 0.10, 0.85), Some(Palette::WHITE_LINE));
            }
        }
    }
}

/// Renders a single 2D surface shape (AABB, circle, oriented box, polygon).
pub fn render_surface_shape(shape: &SurfaceShape, fill_col: Color, border_col: Option<Color>) {
    match shape {
        SurfaceShape::Aabb { min, max } => {
            let width = max.x - min.x;
            let height = max.y - min.y;
            draw_rectangle(min.x, min.y, width, height, fill_col);
            if let Some(b_col) = border_col {
                let thickness = 0.35;
                draw_line(min.x, min.y, max.x, min.y, thickness, b_col);
                draw_line(max.x, min.y, max.x, max.y, thickness, b_col);
                draw_line(max.x, max.y, min.x, max.y, thickness, b_col);
                draw_line(min.x, max.y, min.x, min.y, thickness, b_col);
            }
        }
        SurfaceShape::Circle { center, radius } => {
            draw_circle(center.x, center.y, *radius, fill_col);
            if let Some(b_col) = border_col {
                draw_circle_lines(center.x, center.y, *radius, 0.35, b_col);
            }
        }
        SurfaceShape::OrientedBox {
            center,
            half_extents,
            angle,
        } => {
            let fwd = Vec2::new(angle.cos(), angle.sin()) * half_extents.x;
            let right = Vec2::new(-angle.sin(), angle.cos()) * half_extents.y;
            let p0 = *center - fwd - right;
            let p1 = *center + fwd - right;
            let p2 = *center + fwd + right;
            let p3 = *center - fwd + right;
            draw_quad(p0, p1, p2, p3, fill_col);
            if let Some(b_col) = border_col {
                let thickness = 0.35;
                draw_line(p0.x, p0.y, p1.x, p1.y, thickness, b_col);
                draw_line(p1.x, p1.y, p2.x, p2.y, thickness, b_col);
                draw_line(p2.x, p2.y, p3.x, p3.y, thickness, b_col);
                draw_line(p3.x, p3.y, p0.x, p0.y, thickness, b_col);
            }
        }
        SurfaceShape::Polygon { vertices } => {
            if vertices.len() >= 3 {
                // Fan triangulation from first vertex
                let v0 = vertices[0];
                for i in 1..vertices.len() - 1 {
                    let v1 = vertices[i];
                    let v2 = vertices[i + 1];
                    draw_triangle(
                        macroquad::prelude::Vec2::new(v0.x, v0.y),
                        macroquad::prelude::Vec2::new(v1.x, v1.y),
                        macroquad::prelude::Vec2::new(v2.x, v2.y),
                        fill_col,
                    );
                }
                if let Some(b_col) = border_col {
                    for i in 0..vertices.len() {
                        let next = (i + 1) % vertices.len();
                        draw_line(
                            vertices[i].x,
                            vertices[i].y,
                            vertices[next].x,
                            vertices[next].y,
                            0.35,
                            b_col,
                        );
                    }
                }
            }
        }
    }
}

/// Renders the continuous asphalt spline ribbon with boundary lines and curbs.
fn render_track_ribbon(spline: &TrackSpline) {
    let samples = &spline.samples;
    let n = samples.len();
    if n < 2 {
        return;
    }

    let curb_extra_width = 1.35;
    let seg_count = if spline.closed { n } else { n - 1 };

    // --- Pass 1: Curbs (rendered slightly underneath/adjacent to track edge) ---
    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];

        let stripe_idx = (s0.distance / 1.5).floor() as usize;
        let curb_color = if stripe_idx.is_multiple_of(2) {
            Palette::CURB_RED
        } else {
            Palette::CURB_WHITE
        };

        // Left curb
        if s0.left_curb || s1.left_curb {
            let hw0 = s0.width * 0.5;
            let hw1 = s1.width * 0.5;
            let p0_inner = s0.point + s0.normal * hw0;
            let p1_inner = s1.point + s1.normal * hw1;
            let p0_outer = s0.point + s0.normal * (hw0 + curb_extra_width);
            let p1_outer = s1.point + s1.normal * (hw1 + curb_extra_width);

            draw_quad(p0_inner, p1_inner, p1_outer, p0_outer, curb_color);
        }

        // Right curb
        if s0.right_curb || s1.right_curb {
            let hw0 = s0.width * 0.5;
            let hw1 = s1.width * 0.5;
            let p0_inner = s0.point - s0.normal * hw0;
            let p1_inner = s1.point - s1.normal * hw1;
            let p0_outer = s0.point - s0.normal * (hw0 + curb_extra_width);
            let p1_outer = s1.point - s1.normal * (hw1 + curb_extra_width);

            draw_quad(p0_inner, p1_inner, p1_outer, p0_outer, curb_color);
        }
    }

    // --- Pass 2: Track Surface Quads (Asphalt or Dirt) ---
    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];

        let hw0 = s0.width * 0.5;
        let hw1 = s1.width * 0.5;

        let left0 = s0.point + s0.normal * hw0;
        let right0 = s0.point - s0.normal * hw0;
        let left1 = s1.point + s1.normal * hw1;
        let right1 = s1.point - s1.normal * hw1;

        match s0.surface {
            SurfaceType::Dirt => {
                // Draw playable dirt track segment
                draw_quad(left0, left1, right1, right0, Palette::DIRT);

                // Earthen/dusty track edge boundary lines
                draw_line(left0.x, left0.y, left1.x, left1.y, 0.32, Palette::DIRT_EDGE);
                draw_line(right0.x, right0.y, right1.x, right1.y, 0.32, Palette::DIRT_EDGE);

                // Subtle packed dirt tire groove lines along left & right wheel paths
                let groove_l0 = s0.point + s0.normal * (hw0 * 0.45);
                let groove_l1 = s1.point + s1.normal * (hw1 * 0.45);
                let groove_r0 = s0.point - s0.normal * (hw0 * 0.45);
                let groove_r1 = s1.point - s1.normal * (hw1 * 0.45);

                draw_line(groove_l0.x, groove_l0.y, groove_l1.x, groove_l1.y, 0.22, Palette::DIRT_DARK);
                draw_line(groove_r0.x, groove_r0.y, groove_r1.x, groove_r1.y, 0.22, Palette::DIRT_DARK);
            }
            SurfaceType::Sand => {
                draw_quad(left0, left1, right1, right0, Palette::SAND);
                draw_line(left0.x, left0.y, left1.x, left1.y, 0.32, Palette::SAND_DARK);
                draw_line(right0.x, right0.y, right1.x, right1.y, 0.32, Palette::SAND_DARK);
            }
            SurfaceType::Grass => {
                draw_quad(left0, left1, right1, right0, Palette::GRASS_DARK);
                draw_line(left0.x, left0.y, left1.x, left1.y, 0.32, Palette::GRASS);
                draw_line(right0.x, right0.y, right1.x, right1.y, 0.32, Palette::GRASS);
            }
            SurfaceType::Ice => {
                draw_quad(left0, left1, right1, right0, Color::new(0.85, 0.92, 0.98, 0.95));
                draw_line(left0.x, left0.y, left1.x, left1.y, 0.32, Color::new(0.65, 0.82, 0.95, 0.8));
                draw_line(right0.x, right0.y, right1.x, right1.y, 0.32, Color::new(0.65, 0.82, 0.95, 0.8));
            }
            SurfaceType::Water => {
                draw_quad(left0, left1, right1, right0, Palette::WATER);
                draw_line(left0.x, left0.y, left1.x, left1.y, 0.32, Palette::WATER_BORDER);
                draw_line(right0.x, right0.y, right1.x, right1.y, 0.32, Palette::WATER_BORDER);
            }
            SurfaceType::Oil => {
                draw_quad(left0, left1, right1, right0, Color::new(0.12, 0.12, 0.15, 0.95));
                draw_line(left0.x, left0.y, left1.x, left1.y, 0.32, Color::new(0.35, 0.25, 0.40, 0.85));
                draw_line(right0.x, right0.y, right1.x, right1.y, 0.32, Color::new(0.35, 0.25, 0.40, 0.85));
            }
            SurfaceType::Curb => {
                draw_quad(left0, left1, right1, right0, Palette::CURB_RED);
            }
            SurfaceType::Asphalt => {
                // Draw asphalt segment
                draw_quad(left0, left1, right1, right0, Palette::ASPHALT);

                // White track edge boundary lines
                draw_line(left0.x, left0.y, left1.x, left1.y, 0.28, Palette::WHITE_LINE);
                draw_line(right0.x, right0.y, right1.x, right1.y, 0.28, Palette::WHITE_LINE);

                // Subtle center dashed line (every 4m)
                let center_stripe = ((s0.distance / 3.0).floor() as usize).is_multiple_of(2);
                if center_stripe {
                    draw_line(
                        s0.point.x,
                        s0.point.y,
                        s1.point.x,
                        s1.point.y,
                        0.16,
                        Color::new(0.95, 0.95, 0.95, 0.35),
                    );
                }
            }
        }
    }
}

/// Renders the start/finish timing line with a classic black/white checkered pattern.
fn render_finish_line(track: &Track) {
    if track.checkpoints.is_empty() {
        return;
    }

    // Use the first checkpoint or sample at dist 0
    let finish_cp = track.checkpoints.iter().find(|cp| cp.is_finish_line).unwrap_or(&track.checkpoints[0]);
    let start_pt = finish_cp.gate.start;
    let end_pt = finish_cp.gate.end;
    let dir = end_pt - start_pt;
    let total_w = dir.length();
    if total_w < 1.0 {
        return;
    }
    let norm = dir / total_w;
    let fwd = finish_cp.direction * 0.85; // depth of finish line checkering

    let num_checks = 10;
    let check_w = total_w / num_checks as f32;

    for row in 0..2 {
        let row_offset = fwd * (row as f32 - 0.5);
        for col in 0..num_checks {
            let p0 = start_pt + norm * (col as f32 * check_w) + row_offset;
            let p1 = start_pt + norm * ((col + 1) as f32 * check_w) + row_offset;
            let p2 = p1 + fwd * 0.5;
            let p3 = p0 + fwd * 0.5;

            let color = if (col + row) % 2 == 0 {
                Palette::WHITE_LINE
            } else {
                Palette::ASPHALT
            };
            draw_quad(p0, p1, p2, p3, color);
        }
    }
}

/// Renders starting grid boxes for all spawn positions.
fn render_starting_grid(track: &Track) {
    for pose in &track.grid_positions {
        let pos = pose.position;
        let angle = pose.angle;
        let fwd = Vec2::new(angle.cos(), angle.sin());
        let right = Vec2::new(-angle.sin(), angle.cos());

        let box_len = 3.6;
        let box_w = 1.9;

        let p_fl = pos + fwd * (box_len * 0.5) - right * (box_w * 0.5);
        let p_fr = pos + fwd * (box_len * 0.5) + right * (box_w * 0.5);
        let p_rl = pos - fwd * (box_len * 0.5) - right * (box_w * 0.5);
        let p_rr = pos - fwd * (box_len * 0.5) + right * (box_w * 0.5);

        // Front line & side brackets
        let line_thickness = 0.22;
        draw_line(p_fl.x, p_fl.y, p_fr.x, p_fr.y, line_thickness, Palette::GRID_LINE);
        draw_line(p_fl.x, p_fl.y, p_fl.x - fwd.x * 0.8, p_fl.y - fwd.y * 0.8, line_thickness, Palette::GRID_LINE);
        draw_line(p_fr.x, p_fr.y, p_fr.x - fwd.x * 0.8, p_fr.y - fwd.y * 0.8, line_thickness, Palette::GRID_LINE);
        draw_line(p_rl.x, p_rl.y, p_rr.x, p_rr.y, line_thickness * 0.7, Color::new(0.9, 0.9, 0.9, 0.4));
    }
}

/// Utility to draw a filled convex quad from 4 vertices in CCW/CW order.
#[inline]
pub fn draw_quad(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, color: Color) {
    draw_triangle(
        macroquad::prelude::Vec2::new(p0.x, p0.y),
        macroquad::prelude::Vec2::new(p1.x, p1.y),
        macroquad::prelude::Vec2::new(p2.x, p2.y),
        color,
    );
    draw_triangle(
        macroquad::prelude::Vec2::new(p0.x, p0.y),
        macroquad::prelude::Vec2::new(p2.x, p2.y),
        macroquad::prelude::Vec2::new(p3.x, p3.y),
        color,
    );
}
