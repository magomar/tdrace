use glam::Vec2;
use macroquad::color::Color;
use macroquad::shapes::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle,
};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::SurfaceShape;
use tdrace_core::track::Track;

use super::font::Fonts;
use super::scaler::UiScaler;
use crate::render::color::Palette;

/// Maps a surface type to an attractive neon UI palette color for vector previews.
pub fn surface_preview_color(surface: SurfaceType) -> Color {
    match surface {
        SurfaceType::Asphalt => Color::new(0.35, 0.78, 0.98, 0.95), // Bright Electric Cyan
        SurfaceType::Dirt => Color::new(0.94, 0.62, 0.22, 0.95),    // Vibrant Ochre / Rally Dirt
        SurfaceType::Curb => Palette::CURB_RED,
        SurfaceType::Grass => Color::new(0.30, 0.82, 0.40, 0.95),   // Lush Turf Green
        SurfaceType::Sand => Color::new(0.96, 0.84, 0.42, 0.95),    // Desert Sand Gold
        SurfaceType::Water => Color::new(0.20, 0.60, 0.95, 0.95),   // Azure Water Blue
        SurfaceType::Oil => Color::new(0.40, 0.28, 0.50, 0.95),     // Deep Hazard Violet
        SurfaceType::Ice => Color::new(0.85, 0.95, 1.00, 0.95),     // Glacial White / Pale Blue
    }
}

/// Computes the 2D bounding box of a track spline.
pub fn compute_track_bounds(track: &Track) -> (Vec2, Vec2) {
    if track.spline.samples.is_empty() {
        if track.spline.waypoints.is_empty() {
            return (Vec2::ZERO, Vec2::new(100.0, 100.0));
        }
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for wp in &track.spline.waypoints {
            min = min.min(wp.point);
            max = max.max(wp.point);
        }
        return (min, max);
    }

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for s in &track.spline.samples {
        min = min.min(s.point);
        max = max.max(s.point);
    }
    (min, max)
}

/// Renders a small track thumbnail for track selection list items.
pub fn render_track_thumbnail(
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    track: &Track,
    is_selected: bool,
) {
    // Card backdrop
    let bg_col = if is_selected {
        Color::new(0.06, 0.10, 0.16, 0.95)
    } else {
        Color::new(0.03, 0.05, 0.08, 0.85)
    };
    let border_col = if is_selected {
        Palette::NEON_CYAN
    } else {
        Color::new(0.18, 0.22, 0.30, 0.80)
    };

    draw_rectangle(x, y, w, h, bg_col);
    draw_rectangle_lines(x, y, w, h, if is_selected { 1.5 } else { 1.0 }, border_col);

    let (min, max) = compute_track_bounds(track);
    let track_w = (max.x - min.x).max(10.0);
    let track_h = (max.y - min.y).max(10.0);

    let pad = scaler.s(5.0);
    let avail_w = (w - pad * 2.0).max(4.0);
    let avail_h = (h - pad * 2.0).max(4.0);
    let scale = (avail_w / track_w).min(avail_h / track_h);

    let center_x = x + w * 0.5;
    let center_y = y + h * 0.5;
    let track_cx = (min.x + max.x) * 0.5;
    let track_cy = (min.y + max.y) * 0.5;

    let to_screen = |pt: Vec2| -> Vec2 {
        let rx = (pt.x - track_cx) * scale;
        let ry = -(pt.y - track_cy) * scale; // Invert world Y to screen coords
        Vec2::new(center_x + rx, center_y + ry)
    };

    let samples = &track.spline.samples;
    if samples.len() >= 2 {
        let n = samples.len();
        let seg_count = if track.spline.closed { n } else { n - 1 };

        // Pass 1: Outer subtle glow
        for i in 0..seg_count {
            let p0 = to_screen(samples[i].point);
            let p1 = to_screen(samples[(i + 1) % n].point);
            draw_line(p0.x, p0.y, p1.x, p1.y, scaler.s(2.8), Color::new(0.15, 0.35, 0.50, 0.35));
        }

        // Pass 2: Core surface-colored track line
        for i in 0..seg_count {
            let p0 = to_screen(samples[i].point);
            let p1 = to_screen(samples[(i + 1) % n].point);
            let col = surface_preview_color(samples[i].surface);
            draw_line(p0.x, p0.y, p1.x, p1.y, scaler.s(1.6), col);
        }

        // Pass 3: Start/Finish line indicator dot
        let start_pt = to_screen(samples[0].point);
        draw_circle(start_pt.x, start_pt.y, scaler.s(2.6), Palette::NEON_GREEN);
        draw_circle_lines(start_pt.x, start_pt.y, scaler.s(2.6), 1.0, Palette::WHITE);
    } else if !track.spline.waypoints.is_empty() {
        // Fallback for waypoints-only
        let wps = &track.spline.waypoints;
        for i in 0..wps.len() {
            let next_i = (i + 1) % wps.len();
            let p0 = to_screen(wps[i].point);
            let p1 = to_screen(wps[next_i].point);
            let col = wps[i].surface.map(surface_preview_color).unwrap_or(Palette::NEON_CYAN);
            draw_line(p0.x, p0.y, p1.x, p1.y, scaler.s(1.6), col);
        }
        let start_pt = to_screen(wps[0].point);
        draw_circle(start_pt.x, start_pt.y, scaler.s(2.4), Palette::NEON_GREEN);
    }
}

/// Renders a detailed larger track preview widget for the Track Details dossier.
pub fn render_track_detailed_preview(
    fonts: &Fonts,
    scaler: &UiScaler,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    track: &Track,
) {
    // Glass card background
    scaler.draw_glass_card(x, y, w, h, Color::new(0.05, 0.07, 0.11, 0.92), Palette::UI_CARD_BORDER, 1.2);

    // Subtle background motorsport grid lines
    let grid_spacing = scaler.s(28.0);
    let mut gx = x + grid_spacing;
    while gx < x + w {
        draw_line(gx, y + 1.0, gx, y + h - 1.0, 0.6, Color::new(0.12, 0.16, 0.24, 0.25));
        gx += grid_spacing;
    }
    let mut gy = y + grid_spacing;
    while gy < y + h {
        draw_line(x + 1.0, gy, x + w - 1.0, gy, 0.6, Color::new(0.12, 0.16, 0.24, 0.25));
        gy += grid_spacing;
    }

    let (min, max) = compute_track_bounds(track);
    let track_w = (max.x - min.x).max(10.0);
    let track_h = (max.y - min.y).max(10.0);

    let pad_x = scaler.s(20.0);
    let pad_top = scaler.s(24.0);
    let pad_bottom = scaler.s(28.0);
    let avail_w = (w - pad_x * 2.0).max(10.0);
    let avail_h = (h - pad_top - pad_bottom).max(10.0);
    let scale = (avail_w / track_w).min(avail_h / track_h);

    let center_x = x + w * 0.5;
    let center_y = y + pad_top + avail_h * 0.5;
    let track_cx = (min.x + max.x) * 0.5;
    let track_cy = (min.y + max.y) * 0.5;

    let to_screen = |pt: Vec2| -> Vec2 {
        let rx = (pt.x - track_cx) * scale;
        let ry = -(pt.y - track_cy) * scale; // Invert world Y to screen coords
        Vec2::new(center_x + rx, center_y + ry)
    };

    // Header label overlay
    fonts.draw_ui_bold(
        "CIRCUIT GEOMETRY & SURFACE MAPPING",
        x + scaler.s(12.0),
        y + scaler.s(14.0),
        scaler.font_s(10.5),
        Palette::NEON_GOLD,
    );

    let len_badge = format!("{:.0}m Total Length", track.total_length_m());
    fonts.draw_ui_bold(
        &len_badge,
        x + w - scaler.s(120.0),
        y + scaler.s(14.0),
        scaler.font_s(10.5),
        Palette::NEON_CYAN,
    );

    let render_preview_zones = |layer: tdrace_core::track::geometry::SurfaceLayer| {
        for zone in &track.geometry.surface_zones {
            if zone.layer != layer {
                continue;
            }
            let col = match zone.surface {
                SurfaceType::Water => Color::new(0.18, 0.48, 0.85, 0.55),
                SurfaceType::Sand => Color::new(0.85, 0.72, 0.32, 0.50),
                SurfaceType::Dirt => Color::new(0.70, 0.45, 0.18, 0.50),
                SurfaceType::Ice => Color::new(0.85, 0.92, 0.98, 0.65),
                SurfaceType::Oil => Color::new(0.12, 0.12, 0.15, 0.75),
                _ => Color::new(0.30, 0.35, 0.45, 0.45),
            };
            match &zone.shape {
                SurfaceShape::Aabb { min: z_min, max: z_max } => {
                    let p0 = to_screen(Vec2::new(z_min.x, z_max.y));
                    let p1 = to_screen(Vec2::new(z_max.x, z_min.y));
                    let rect_x = p0.x.min(p1.x);
                    let rect_y = p0.y.min(p1.y);
                    let rect_w = (p0.x - p1.x).abs();
                    let rect_h = (p0.y - p1.y).abs();
                    draw_rectangle(rect_x, rect_y, rect_w, rect_h, col);
                    draw_rectangle_lines(rect_x, rect_y, rect_w, rect_h, 0.8, col);
                }
                SurfaceShape::Circle { center, radius } => {
                    let center_s = to_screen(*center);
                    let rad_s = (radius * scale).max(scaler.s(2.0));
                    draw_circle(center_s.x, center_s.y, rad_s, col);
                    draw_circle_lines(center_s.x, center_s.y, rad_s, 0.8, col);
                }
                SurfaceShape::OrientedBox { center, half_extents, angle } => {
                    let fwd = Vec2::new(angle.cos(), angle.sin()) * half_extents.x;
                    let right = Vec2::new(-angle.sin(), angle.cos()) * half_extents.y;
                    let p0 = to_screen(*center - fwd - right);
                    let p1 = to_screen(*center + fwd - right);
                    let p2 = to_screen(*center + fwd + right);
                    let p3 = to_screen(*center - fwd + right);
                    draw_triangle(
                        macroquad::prelude::Vec2::new(p0.x, p0.y),
                        macroquad::prelude::Vec2::new(p1.x, p1.y),
                        macroquad::prelude::Vec2::new(p2.x, p2.y),
                        col,
                    );
                    draw_triangle(
                        macroquad::prelude::Vec2::new(p0.x, p0.y),
                        macroquad::prelude::Vec2::new(p2.x, p2.y),
                        macroquad::prelude::Vec2::new(p3.x, p3.y),
                        col,
                    );
                    draw_line(p0.x, p0.y, p1.x, p1.y, 0.8, col);
                    draw_line(p1.x, p1.y, p2.x, p2.y, 0.8, col);
                    draw_line(p2.x, p2.y, p3.x, p3.y, 0.8, col);
                    draw_line(p3.x, p3.y, p0.x, p0.y, 0.8, col);
                }
                SurfaceShape::Polygon { vertices } => {
                    if vertices.len() >= 3 {
                        let pts: Vec<Vec2> = vertices.iter().map(|v| to_screen(*v)).collect();
                        let v0 = pts[0];
                        for i in 1..pts.len() - 1 {
                            let v1 = pts[i];
                            let v2 = pts[i + 1];
                            draw_triangle(
                                macroquad::prelude::Vec2::new(v0.x, v0.y),
                                macroquad::prelude::Vec2::new(v1.x, v1.y),
                                macroquad::prelude::Vec2::new(v2.x, v2.y),
                                col,
                            );
                        }
                        for i in 0..pts.len() {
                            let next = (i + 1) % pts.len();
                            draw_line(pts[i].x, pts[i].y, pts[next].x, pts[next].y, 0.8, col);
                        }
                    }
                }
            }
        }
    };

    // 1. Render BelowTrack Surface Zones (e.g. sand traps, lakes, off-track dirt)
    render_preview_zones(tdrace_core::track::geometry::SurfaceLayer::BelowTrack);

    let samples = &track.spline.samples;
    if samples.len() >= 2 {
        let n = samples.len();
        let seg_count = if track.spline.closed { n } else { n - 1 };

        // Pass 1: Curbs (rumble strips on apexes)
        for i in 0..seg_count {
            let s0 = &samples[i];
            let s1 = &samples[(i + 1) % n];
            if s0.left_curb || s0.right_curb || s1.left_curb || s1.right_curb {
                let p0 = to_screen(s0.point);
                let p1 = to_screen(s1.point);
                draw_line(p0.x, p0.y, p1.x, p1.y, scaler.s(5.5), Palette::CURB_RED);
            }
        }

        // Pass 2: Road ribbon with thickness
        for i in 0..seg_count {
            let s0 = &samples[i];
            let s1 = &samples[(i + 1) % n];
            let p0 = to_screen(s0.point);
            let p1 = to_screen(s1.point);

            let road_thickness = (s0.width * scale * 0.9).clamp(scaler.s(3.5), scaler.s(9.0));
            let col = surface_preview_color(s0.surface);

            // Outer subtle track border
            draw_line(p0.x, p0.y, p1.x, p1.y, road_thickness + scaler.s(1.5), Color::new(0.08, 0.12, 0.18, 0.90));
            // Core surface ribbon
            draw_line(p0.x, p0.y, p1.x, p1.y, road_thickness, col);
        }

        // Pass 2b: Render AboveTrack Surface Zones (e.g. oil slicks, water puddles, on-top hazards)
        render_preview_zones(tdrace_core::track::geometry::SurfaceLayer::AboveTrack);

        // Pass 3: Checkpoint timing gates (subtle translucent tick marks)
        for cp in &track.checkpoints {
            if !cp.is_finish_line {
                let g0 = to_screen(cp.gate.start);
                let g1 = to_screen(cp.gate.end);
                draw_line(g0.x, g0.y, g1.x, g1.y, 1.2, Color::new(1.0, 0.90, 0.30, 0.65));
            }
        }

        // Pass 4: Starting grid boxes & Finish Line
        let (f_start, f_end, f_dir) = if let Some(cp) = track.checkpoints.iter().find(|c| c.is_finish_line) {
            (to_screen(cp.gate.start), to_screen(cp.gate.end), cp.direction)
        } else {
            let p = to_screen(samples[0].point);
            let tan = samples[0].tangent;
            let norm = samples[0].normal;
            (p + Vec2::new(norm.x, -norm.y) * scaler.s(6.0), p - Vec2::new(norm.x, -norm.y) * scaler.s(6.0), tan)
        };

        // Finish line checkerboard bar
        draw_line(f_start.x, f_start.y, f_end.x, f_end.y, scaler.s(3.0), Palette::NEON_GREEN);
        draw_line(f_start.x, f_start.y, f_end.x, f_end.y, scaler.s(1.2), Palette::WHITE);

        // Direction arrow
        let f_mid = (f_start + f_end) * 0.5;
        let arrow_dir = Vec2::new(f_dir.x, -f_dir.y).normalize_or_zero();
        let arrow_tip = f_mid + arrow_dir * scaler.s(12.0);
        draw_line(f_mid.x, f_mid.y, arrow_tip.x, arrow_tip.y, scaler.s(1.8), Palette::WHITE);

        // Start point badge circle
        draw_circle(f_mid.x, f_mid.y, scaler.s(3.5), Palette::NEON_GREEN);
        draw_circle_lines(f_mid.x, f_mid.y, scaler.s(3.5), 1.2, Palette::BLACK);
    }

    // Bottom Surface Composition Mini-Legend Bar
    let legend_y = y + h - scaler.s(18.0);
    let breakdown = track.surface_breakdown();

    let bar_x = x + scaler.s(12.0);
    let bar_w = (w - scaler.s(24.0)).max(20.0);
    let bar_h = scaler.s(4.0);

    let mut curr_bx = bar_x;
    for (surf, pct) in &breakdown {
        let seg_w = bar_w * (pct / 100.0);
        let col = surface_preview_color(*surf);
        draw_rectangle(curr_bx, legend_y - scaler.s(6.0), seg_w, bar_h, col);
        curr_bx += seg_w;
    }

    // Text labels for surface breakdown
    let summary_text = track.surface_summary_string();
    let legend_str = format!("SURFACES: {}", summary_text);
    fonts.draw_ui_bold(
        &legend_str,
        bar_x,
        legend_y + scaler.s(10.0),
        scaler.font_s(10.0),
        Palette::WHITE,
    );
}
