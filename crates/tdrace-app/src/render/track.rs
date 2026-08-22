use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_triangle};
use glam::Vec2;
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::SurfaceShape;
use tdrace_core::track::spline::TrackSpline;
use tdrace_core::track::Track;

use super::color::Palette;

/// Renders the complete track geometry including asphalt ribbon, rumble curbs,
/// sand traps, pit lanes, grid boxes, and start/finish line checkerboard.
pub fn render_track(track: &Track) {
    // 1. Render geometric surface zones (sand traps, asphalt runoff, hazard zones)
    render_surface_zones(track);

    // 2. Render pit box area if defined
    if let Some(pit_area) = &track.pit_box_area {
        render_surface_shape(pit_area, Palette::PIT_LANE, Some(Palette::WHITE_LINE));
    }

    // 3. Render the asphalt track ribbon and rumble curbs
    render_track_ribbon(&track.spline);

    // 4. Render starting grid slots
    render_starting_grid(track);

    // 5. Render start/finish line checkerboard
    render_finish_line(track);
}

/// Draws all custom surface zones (e.g. sand traps outside hairpins).
fn render_surface_zones(track: &Track) {
    for zone in &track.geometry.surface_zones {
        let (fill_col, border_col) = match zone.surface {
            SurfaceType::Sand => (Palette::SAND, Some(Palette::SAND_DARK)),
            SurfaceType::Asphalt => (Palette::RUNOFF_ASPHALT, Some(Palette::WHITE_LINE)),
            SurfaceType::Grass => (Palette::GRASS_DARK, None),
            SurfaceType::Curb => (Palette::CURB_RED, None),
            SurfaceType::Ice => (Color::new(0.85, 0.92, 0.98, 0.8), None),
            SurfaceType::Oil => (Color::new(0.12, 0.12, 0.15, 0.85), None),
        };
        render_surface_shape(&zone.shape, fill_col, border_col);
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

    // --- Pass 2: Asphalt Track Surface Quads ---
    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];

        let hw0 = s0.width * 0.5;
        let hw1 = s1.width * 0.5;

        let left0 = s0.point + s0.normal * hw0;
        let right0 = s0.point - s0.normal * hw0;
        let left1 = s1.point + s1.normal * hw1;
        let right1 = s1.point - s1.normal * hw1;

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
