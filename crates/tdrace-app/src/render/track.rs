use std::collections::HashMap;
use std::sync::Mutex;
use glam::Vec2;
use macroquad::color::{Color, WHITE};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_triangle};
use tdrace_core::physics::surface::SurfaceType;
use tdrace_core::track::geometry::{SurfaceLayer, SurfaceShape};
use tdrace_core::track::spline::{SplineSample, TrackSpline};
use tdrace_core::track::Track;

use super::color::Palette;
use super::surface_material::{evaluate_macro_modulation, SurfaceMaterialRegistry, SurfaceTextureQuality};

/// Computes instantaneous track curvature (radians per meter) between two spline samples.
#[inline]
pub fn compute_segment_curvature(s0: &SplineSample, s1: &SplineSample) -> f32 {
    let ds = (s1.point - s0.point).length();
    if ds > 1e-4 {
        let cross = s0.tangent.x * s1.tangent.y - s0.tangent.y * s1.tangent.x;
        let dot = (s0.tangent.dot(s1.tangent)).clamp(-1.0, 1.0);
        let angle = cross.atan2(dot);
        angle / ds
    } else {
        0.0
    }
}

static SURFACE_REGISTRY: Mutex<Option<SurfaceMaterialRegistry>> = Mutex::new(None);

pub fn set_surface_texture_quality(quality: SurfaceTextureQuality) {
    if let Ok(mut lock) = SURFACE_REGISTRY.lock() {
        if let Some(reg) = lock.as_mut() {
            reg.set_quality(quality);
        } else {
            *lock = Some(SurfaceMaterialRegistry::new(quality));
        }
    }
}

pub fn get_surface_texture_quality() -> SurfaceTextureQuality {
    if let Ok(lock) = SURFACE_REGISTRY.lock() {
        if let Some(reg) = lock.as_ref() {
            return reg.quality();
        }
    }
    SurfaceTextureQuality::High
}

fn ensure_surface_registry() {
    if let Ok(mut lock) = SURFACE_REGISTRY.lock() {
        if lock.is_none() {
            *lock = Some(SurfaceMaterialRegistry::new(SurfaceTextureQuality::High));
        }
    }
}

pub fn with_surface_registry<R, F: FnOnce(&SurfaceMaterialRegistry) -> R>(f: F) -> Option<R> {
    ensure_surface_registry();
    if let Ok(lock) = SURFACE_REGISTRY.lock() {
        if let Some(reg) = lock.as_ref() {
            return Some(f(reg));
        }
    }
    None
}

pub fn get_surface_material_info(surface: SurfaceType) -> (Option<macroquad::texture::Texture2D>, f32, SurfaceTextureQuality) {
    ensure_surface_registry();
    if let Ok(lock) = SURFACE_REGISTRY.lock() {
        if let Some(reg) = lock.as_ref() {
            let q = reg.quality();
            if q == SurfaceTextureQuality::Off {
                return (None, 4.0, q);
            }
            let tex = reg.get_material(surface).and_then(|m| m.texture.clone());
            let scale = reg.tile_scale(surface);
            return (tex, scale, q);
        }
    }
    (None, 4.0, SurfaceTextureQuality::Off)
}

pub fn get_curb_material_info() -> (Option<macroquad::texture::Texture2D>, f32, SurfaceTextureQuality) {
    ensure_surface_registry();
    if let Ok(lock) = SURFACE_REGISTRY.lock() {
        if let Some(reg) = lock.as_ref() {
            let q = reg.quality();
            if q == SurfaceTextureQuality::Off {
                return (None, 3.0, q);
            }
            let tex = reg.get_curb_material().and_then(|m| m.texture.clone());
            let scale = reg.get_curb_material().map(|m| m.tile_scale_meters).unwrap_or(3.0);
            return (tex, scale, q);
        }
    }
    (None, 3.0, SurfaceTextureQuality::Off)
}

/// Helper builder batching textured quad vertices and indices for high-performance draw_mesh dispatch.
pub struct BatchMeshBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture: Option<macroquad::texture::Texture2D>,
}

impl BatchMeshBuilder {
    pub fn new(texture: Option<macroquad::texture::Texture2D>) -> Self {
        Self {
            vertices: Vec::with_capacity(512),
            indices: Vec::with_capacity(768),
            texture,
        }
    }

    pub fn push_quad(
        &mut self,
        p0: Vec2, uv0: macroquad::prelude::Vec2, c0: Color,
        p1: Vec2, uv1: macroquad::prelude::Vec2, c1: Color,
        p2: Vec2, uv2: macroquad::prelude::Vec2, c2: Color,
        p3: Vec2, uv3: macroquad::prelude::Vec2, c3: Color,
    ) {
        if self.vertices.len() + 4 > 65000 {
            self.flush();
        }
        let base = self.vertices.len() as u16;
        self.vertices.push(Vertex { position: macroquad::prelude::Vec3::new(p0.x, p0.y, 0.0), uv: uv0, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: c0.into() });
        self.vertices.push(Vertex { position: macroquad::prelude::Vec3::new(p1.x, p1.y, 0.0), uv: uv1, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: c1.into() });
        self.vertices.push(Vertex { position: macroquad::prelude::Vec3::new(p2.x, p2.y, 0.0), uv: uv2, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: c2.into() });
        self.vertices.push(Vertex { position: macroquad::prelude::Vec3::new(p3.x, p3.y, 0.0), uv: uv3, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: c3.into() });

        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
        self.indices.push(base);
        self.indices.push(base + 2);
        self.indices.push(base + 3);
    }

    pub fn push_triangle(
        &mut self,
        p0: Vec2, uv0: macroquad::prelude::Vec2, c0: Color,
        p1: Vec2, uv1: macroquad::prelude::Vec2, c1: Color,
        p2: Vec2, uv2: macroquad::prelude::Vec2, c2: Color,
    ) {
        if self.vertices.len() + 3 > 65000 {
            self.flush();
        }
        let base = self.vertices.len() as u16;
        self.vertices.push(Vertex { position: macroquad::prelude::Vec3::new(p0.x, p0.y, 0.0), uv: uv0, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: c0.into() });
        self.vertices.push(Vertex { position: macroquad::prelude::Vec3::new(p1.x, p1.y, 0.0), uv: uv1, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: c1.into() });
        self.vertices.push(Vertex { position: macroquad::prelude::Vec3::new(p2.x, p2.y, 0.0), uv: uv2, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: c2.into() });

        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
    }

    pub fn flush(&mut self) {
        if !self.indices.is_empty() {
            let mesh = Mesh {
                vertices: std::mem::take(&mut self.vertices),
                indices: std::mem::take(&mut self.indices),
                texture: self.texture.clone(),
            };
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                draw_mesh(&mesh);
            }));
        }
    }
}

#[inline]
pub fn draw_textured_quad(
    p0: Vec2, uv0: macroquad::prelude::Vec2,
    p1: Vec2, uv1: macroquad::prelude::Vec2,
    p2: Vec2, uv2: macroquad::prelude::Vec2,
    p3: Vec2, uv3: macroquad::prelude::Vec2,
    texture: Option<&macroquad::texture::Texture2D>,
    color: Color,
) {
    let vertices = vec![
        Vertex { position: macroquad::prelude::Vec3::new(p0.x, p0.y, 0.0), uv: uv0, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: color.into() },
        Vertex { position: macroquad::prelude::Vec3::new(p1.x, p1.y, 0.0), uv: uv1, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: color.into() },
        Vertex { position: macroquad::prelude::Vec3::new(p2.x, p2.y, 0.0), uv: uv2, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: color.into() },
        Vertex { position: macroquad::prelude::Vec3::new(p3.x, p3.y, 0.0), uv: uv3, normal: macroquad::prelude::Vec4::new(0.0, 0.0, 1.0, 0.0), color: color.into() },
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    let mesh = Mesh {
        vertices,
        indices,
        texture: texture.cloned(),
    };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        draw_mesh(&mesh);
    }));
}

#[inline]
fn is_segment_in_view(s0: &SplineSample, s1: &SplineSample, view_bounds: Option<(Vec2, Vec2)>) -> bool {
    if let Some((min, max)) = view_bounds {
        let max_w = s0.width.max(s1.width) * 0.5 + 4.5;
        let s_min_x = s0.point.x.min(s1.point.x) - max_w;
        let s_max_x = s0.point.x.max(s1.point.x) + max_w;
        let s_min_y = s0.point.y.min(s1.point.y) - max_w;
        let s_max_y = s0.point.y.max(s1.point.y) + max_w;
        !(s_max_x < min.x || s_min_x > max.x || s_max_y < min.y || s_min_y > max.y)
    } else {
        true
    }
}

/// Renders the complete track geometry (ground layer followed by elevated bridges).
pub fn render_track(track: &Track) {
    render_track_culled(track, None);
}

/// Renders the complete track geometry with camera viewport culling.
pub fn render_track_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    render_ground_track_culled(track, view_bounds);
    render_elevated_track_culled(track, view_bounds);
}

/// Renders base ground environment, surface zones, ground track ribbon, hazard zones, and timing lines.
pub fn render_ground_track(track: &Track) {
    render_ground_track_culled(track, None);
}

/// Renders a textured ground plane across the visible camera viewport for the track's default surface.
pub fn render_backdrop_ground_pass(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    let quality = get_surface_texture_quality();
    if quality == SurfaceTextureQuality::Off {
        return;
    }
    let default_surf = track.default_surface;
    let (tex, scale, _) = get_surface_material_info(default_surf);
    if tex.is_none() {
        return;
    }

    let (min, max) = match view_bounds {
        Some((b_min, b_max)) => (b_min, b_max),
        None => {
            if track.spline.samples.is_empty() {
                return;
            }
            let mut min = track.spline.samples[0].point;
            let mut max = track.spline.samples[0].point;
            for s in &track.spline.samples {
                min = min.min(s.point);
                max = max.max(s.point);
            }
            (min - Vec2::splat(60.0), max + Vec2::splat(60.0))
        }
    };

    let p0 = Vec2::new(min.x, min.y);
    let p1 = Vec2::new(max.x, min.y);
    let p2 = Vec2::new(max.x, max.y);
    let p3 = Vec2::new(min.x, max.y);

    let uv0 = macroquad::prelude::Vec2::new(p0.x / scale, p0.y / scale);
    let uv1 = macroquad::prelude::Vec2::new(p1.x / scale, p1.y / scale);
    let uv2 = macroquad::prelude::Vec2::new(p2.x / scale, p2.y / scale);
    let uv3 = macroquad::prelude::Vec2::new(p3.x / scale, p3.y / scale);

    draw_textured_quad(p0, uv0, p1, uv1, p2, uv2, p3, uv3, tex.as_ref(), WHITE);
}

/// Renders ground track ribbon and surface features with camera viewport culling.
pub fn render_ground_track_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    ensure_surface_registry();

    // 0. Render active camera viewport ground backdrop plane
    render_backdrop_ground_pass(track, view_bounds);

    // 1. Render base off-track surface zones (BelowTrack: sand traps, asphalt runoff, dirt areas beneath road)
    render_surface_zones_layer(track, SurfaceLayer::BelowTrack);

    // 2. Render pit box area if defined
    if let Some(pit_area) = &track.pit_box_area {
        render_surface_shape(pit_area, Palette::PIT_LANE, Some(Palette::WHITE_LINE));
    }

    // 3. Render segment runoff corridors, ground curbs and ground surface quads
    render_runoff_pass(&track.spline, false, view_bounds);
    render_curbs_pass(&track.spline, false, view_bounds);
    render_surface_pass(&track.spline, false, view_bounds);

    // 4. Render on-top surface zones (AboveTrack: water puddles, oil slicks, sand/grass/dirt overlays)
    render_surface_zones_layer(track, SurfaceLayer::AboveTrack);

    // 5. Render 2.5D jump ramps
    render_jump_ramps(track);

    // 6. Render starting grid slots
    render_starting_grid(track);

    // 7. Render start/finish line checkerboard
    render_finish_line(track);
}

/// Renders elevated overpass bridges: drop shadows, solid concrete deck slab, curbs, and asphalt ribbon.
pub fn render_elevated_track(track: &Track) {
    render_elevated_track_culled(track, None);
}

/// Renders elevated overpass bridges with camera viewport culling.
pub fn render_elevated_track_culled(track: &Track, view_bounds: Option<(Vec2, Vec2)>) {
    ensure_surface_registry();
    let has_elevated = track.spline.samples.iter().any(|s| s.is_bridge);
    if has_elevated {
        render_bridge_structure_pass(&track.spline, view_bounds);
        render_runoff_pass(&track.spline, true, view_bounds);
        render_curbs_pass(&track.spline, true, view_bounds);
        render_surface_pass(&track.spline, true, view_bounds);
    }
}

/// Helper returning fill and border colors for a given surface type.
pub fn get_surface_zone_colors(surface: SurfaceType) -> (Color, Option<Color>) {
    match surface {
        SurfaceType::Sand => (Palette::SAND, Some(Palette::SAND_DARK)),
        SurfaceType::Dirt => (Palette::DIRT, Some(Palette::DIRT_DARK)),
        SurfaceType::Water => (Palette::WATER, Some(Palette::WATER_BORDER)),
        SurfaceType::Asphalt => (Palette::RUNOFF_ASPHALT, Some(Palette::WHITE_LINE)),
        SurfaceType::Grass => (Palette::GRASS_DARK, None),
        SurfaceType::Curb => (Palette::CURB_RED, None),
        SurfaceType::Ice => (Color::new(0.85, 0.92, 0.98, 0.8), None),
        SurfaceType::Oil => (Color::new(0.12, 0.12, 0.15, 0.85), None),
        SurfaceType::Mud => (Palette::MUD, Some(Palette::MUD_DARK)),
        SurfaceType::Snow => (Palette::SNOW, Some(Palette::SNOW_EDGE)),
        SurfaceType::Gravel => (Palette::GRAVEL, Some(Palette::GRAVEL_DARK)),
        SurfaceType::Concrete => (Palette::CONCRETE, Some(Palette::CONCRETE_DARK)),
    }
}

/// Helper returning the background/backdrop clear color for a given track off-track default surface.
pub fn get_track_backdrop_color(surface: SurfaceType) -> Color {
    match surface {
        SurfaceType::Grass => Palette::BACKDROP_GRASS,
        SurfaceType::Dirt => Palette::BACKDROP_DIRT,
        SurfaceType::Sand => Palette::BACKDROP_SAND,
        SurfaceType::Asphalt => Palette::BACKDROP_ASPHALT,
        SurfaceType::Mud => Palette::BACKDROP_MUD,
        SurfaceType::Snow => Palette::BACKDROP_SNOW,
        SurfaceType::Gravel => Palette::BACKDROP_GRAVEL,
        SurfaceType::Concrete => Palette::BACKDROP_CONCRETE,
        _ => Palette::BACKDROP_GRASS,
    }
}

/// Draws custom surface zones for a specific layer pass (BelowTrack or AboveTrack).
pub fn render_surface_zones_layer(track: &Track, layer: SurfaceLayer) {
    for zone in &track.geometry.surface_zones {
        if zone.layer == layer {
            let (fill_col, border_col) = get_surface_zone_colors(zone.surface);
            render_surface_shape_textured(&zone.shape, zone.surface, fill_col, border_col);
        }
    }
}

/// Draws all custom surface zones.
pub fn render_surface_zones(track: &Track) {
    for zone in &track.geometry.surface_zones {
        let (fill_col, border_col) = get_surface_zone_colors(zone.surface);
        render_surface_shape_textured(&zone.shape, zone.surface, fill_col, border_col);
    }
}
/// Helper returning base color, border rail color, and natural contour ridge color for a jump ramp according to its surface type.
pub fn get_ramp_surface_colors(surface: SurfaceType) -> (Color, Option<Color>, Color) {
    match surface {
        SurfaceType::Asphalt => (
            Color::new(0.20, 0.22, 0.26, 1.0),
            Some(Color::new(0.70, 0.72, 0.78, 0.85)),
            Color::new(0.80, 0.82, 0.88, 0.75),
        ),
        SurfaceType::Dirt => (
            Palette::DIRT,
            Some(Palette::DIRT_EDGE),
            Color::new(0.68, 0.50, 0.32, 0.85),
        ),
        SurfaceType::Sand => (
            Palette::SAND,
            Some(Palette::SAND_DARK),
            Color::new(0.72, 0.58, 0.38, 0.85),
        ),
        SurfaceType::Grass => (
            Palette::GRASS_DARK,
            Some(Palette::GRASS),
            Color::new(0.38, 0.65, 0.32, 0.85),
        ),
        SurfaceType::Ice => (
            Color::new(0.78, 0.88, 0.96, 1.0),
            Some(Color::new(0.92, 0.96, 1.0, 1.0)),
            Color::new(0.45, 0.68, 0.88, 0.85),
        ),
        SurfaceType::Water => (
            Palette::WATER,
            Some(Palette::WATER_BORDER),
            Color::new(0.40, 0.75, 0.90, 0.85),
        ),
        SurfaceType::Oil => (
            Color::new(0.12, 0.12, 0.15, 1.0),
            Some(Color::new(0.28, 0.24, 0.35, 1.0)),
            Color::new(0.35, 0.30, 0.45, 0.85),
        ),
        SurfaceType::Curb => (
            Palette::CURB_RED,
            Some(Palette::CURB_WHITE),
            Palette::WHITE,
        ),
        SurfaceType::Mud => (
            Palette::MUD,
            Some(Palette::MUD_DARK),
            Color::new(0.42, 0.30, 0.18, 0.85),
        ),
        SurfaceType::Snow => (
            Palette::SNOW,
            Some(Palette::SNOW_EDGE),
            Color::new(0.80, 0.85, 0.92, 0.85),
        ),
        SurfaceType::Gravel => (
            Palette::GRAVEL,
            Some(Palette::GRAVEL_EDGE),
            Color::new(0.70, 0.68, 0.64, 0.85),
        ),
        SurfaceType::Concrete => (
            Color::new(0.70, 0.72, 0.74, 1.0),
            Some(Color::new(0.85, 0.87, 0.90, 0.85)),
            Color::new(0.82, 0.84, 0.88, 0.75),
        ),
    }
}

pub fn render_jump_ramps(track: &Track) {
    for ramp in &track.geometry.jump_ramps {
        let (ramp_base_col, border_opt, contour_col) = get_ramp_surface_colors(ramp.surface);
        match &ramp.shape {
            SurfaceShape::OrientedBox {
                center,
                half_extents,
                angle,
            } => {
                let fwd = Vec2::new(angle.cos(), angle.sin());
                let right = Vec2::new(-angle.sin(), angle.cos());
                let half_len = half_extents.x;
                let half_wid = half_extents.y;

                let p0 = *center - fwd * half_len - right * half_wid;
                let p1 = *center + fwd * half_len - right * half_wid;
                let p2 = *center + fwd * half_len + right * half_wid;
                let p3 = *center - fwd * half_len + right * half_wid;

                // 1. Drop shadow beneath ramp platform
                let s_off = Vec2::new(0.40, 0.55);
                draw_quad(p0 + s_off, p1 + s_off, p2 + s_off, p3 + s_off, Palette::SHADOW);

                // 2. Base metallic/surface ramp quad
                draw_quad(p0, p1, p2, p3, ramp_base_col);

                // 3. Natural curved elevation contour arcs across the ramp width (showing progressive mound incline)
                let num_contours = if half_len >= 8.0 {
                    4
                } else if half_len >= 4.0 {
                    3
                } else {
                    2
                };

                let arc_w = (half_wid * 0.82).max(0.6);
                let arc_bulge = (half_len * 0.22).min(arc_w * 0.40);

                for s in 0..num_contours {
                    let t = (s as f32 + 1.0) / (num_contours as f32 + 1.0);
                    let x_center = -half_len * 0.70 + t * (half_len * 1.35);

                    // Draw smooth curved contour arc across the ramp width (12 segments)
                    let num_segments = 12;
                    let mut prev_pt: Option<Vec2> = None;

                    for seg in 0..=num_segments {
                        let frac = (seg as f32 / num_segments as f32) * 2.0 - 1.0; // [-1.0, 1.0]
                        let y_offset = frac * arc_w;
                        let curve_offset = (1.0 - frac * frac) * arc_bulge;
                        let pt = *center + fwd * (x_center + curve_offset) + right * y_offset;

                        if let Some(prev) = prev_pt {
                            draw_line(prev.x, prev.y, pt.x, pt.y, 0.35, contour_col);
                        }
                        prev_pt = Some(pt);
                    }
                }

                // 4. Elevated natural curved takeoff lip at exit edge
                let lip_bulge = (half_len * 0.12).min(0.45);
                let mut prev_lip: Option<Vec2> = None;
                let num_lip_segs = 12;
                for seg in 0..=num_lip_segs {
                    let frac = (seg as f32 / num_lip_segs as f32) * 2.0 - 1.0;
                    let y_offset = frac * half_wid;
                    let curve_offset = (1.0 - frac * frac) * lip_bulge;
                    let pt = *center + fwd * (half_len + curve_offset) + right * y_offset;

                    if let Some(prev) = prev_lip {
                        draw_line(prev.x, prev.y, pt.x, pt.y, 0.40, Color::new(0.95, 0.95, 0.98, 0.95));
                    }
                    prev_lip = Some(pt);
                }

                // 5. Ramp side border rails & entrance edge
                let rail_col = border_opt.unwrap_or(Color::new(0.85, 0.85, 0.90, 1.0));
                draw_line(p0.x, p0.y, p1.x, p1.y, 0.30, rail_col);
                draw_line(p3.x, p3.y, p2.x, p2.y, 0.30, rail_col);
                draw_line(p0.x, p0.y, p3.x, p3.y, 0.25, Color::new(0.55, 0.55, 0.60, 0.60));
            }
            _ => {
                render_surface_shape(&ramp.shape, ramp_base_col, border_opt);
                let center = ramp.shape.center();
                let dir = ramp.direction;
                let right = Vec2::new(-dir.y, dir.x);
                let half_len = ramp.half_extents().x.max(2.0);
                let half_wid = ramp.half_extents().y.max(1.5);
                let arc_w = half_wid * 0.75;
                let arc_bulge = (half_len * 0.20).min(arc_w * 0.35);

                for s in 0..3 {
                    let t = (s as f32 + 1.0) / 4.0;
                    let x_c = -half_len * 0.5 + t * half_len;
                    let mut prev_pt: Option<Vec2> = None;
                    for seg in 0..=10 {
                        let frac = (seg as f32 / 10.0) * 2.0 - 1.0;
                        let y = frac * arc_w;
                        let bulge = (1.0 - frac * frac) * arc_bulge;
                        let pt = center + dir * (x_c + bulge) + right * y;
                        if let Some(prev) = prev_pt {
                            draw_line(prev.x, prev.y, pt.x, pt.y, 0.35, contour_col);
                        }
                        prev_pt = Some(pt);
                    }
                }
            }
        }
    }
}

/// Renders a single 2D surface shape (AABB, circle, oriented box, polygon).
pub fn render_surface_shape(shape: &SurfaceShape, fill_col: Color, border_col: Option<Color>) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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
    }));
}

/// Renders a single 2D surface shape with world-space planar texture UV mapping if textures are enabled.
pub fn render_surface_shape_textured(
    shape: &SurfaceShape,
    surface: SurfaceType,
    fill_col: Color,
    border_col: Option<Color>,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let (tex, scale, quality) = get_surface_material_info(surface);
        if quality != SurfaceTextureQuality::Off && tex.is_some() {
            let texture = tex.as_ref();
            match shape {
                SurfaceShape::Aabb { min, max } => {
                    let p0 = Vec2::new(min.x, min.y);
                    let p1 = Vec2::new(max.x, min.y);
                    let p2 = Vec2::new(max.x, max.y);
                    let p3 = Vec2::new(min.x, max.y);
                    let uv0 = macroquad::prelude::Vec2::new(p0.x / scale, p0.y / scale);
                    let uv1 = macroquad::prelude::Vec2::new(p1.x / scale, p1.y / scale);
                    let uv2 = macroquad::prelude::Vec2::new(p2.x / scale, p2.y / scale);
                    let uv3 = macroquad::prelude::Vec2::new(p3.x / scale, p3.y / scale);
                    draw_textured_quad(p0, uv0, p1, uv1, p2, uv2, p3, uv3, texture, WHITE);
                    if let Some(b_col) = border_col {
                        let thickness = 0.35;
                        draw_line(min.x, min.y, max.x, min.y, thickness, b_col);
                        draw_line(max.x, min.y, max.x, max.y, thickness, b_col);
                        draw_line(max.x, max.y, min.x, max.y, thickness, b_col);
                        draw_line(min.x, max.y, min.x, min.y, thickness, b_col);
                    }
                }
                SurfaceShape::Circle { center, radius } => {
                    let segments = 24;
                    let mut builder = BatchMeshBuilder::new(tex);
                    let c_uv = macroquad::prelude::Vec2::new(center.x / scale, center.y / scale);
                    for i in 0..segments {
                        let th0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                        let th1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                        let p0 = *center + Vec2::new(th0.cos(), th0.sin()) * *radius;
                        let p1 = *center + Vec2::new(th1.cos(), th1.sin()) * *radius;
                        let uv0 = macroquad::prelude::Vec2::new(p0.x / scale, p0.y / scale);
                        let uv1 = macroquad::prelude::Vec2::new(p1.x / scale, p1.y / scale);
                        builder.push_triangle(*center, c_uv, WHITE, p0, uv0, WHITE, p1, uv1, WHITE);
                    }
                    builder.flush();
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
                    let uv0 = macroquad::prelude::Vec2::new(p0.x / scale, p0.y / scale);
                    let uv1 = macroquad::prelude::Vec2::new(p1.x / scale, p1.y / scale);
                    let uv2 = macroquad::prelude::Vec2::new(p2.x / scale, p2.y / scale);
                    let uv3 = macroquad::prelude::Vec2::new(p3.x / scale, p3.y / scale);
                    draw_textured_quad(p0, uv0, p1, uv1, p2, uv2, p3, uv3, texture, WHITE);
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
                        let mut builder = BatchMeshBuilder::new(tex);
                        let v0 = vertices[0];
                        let uv0 = macroquad::prelude::Vec2::new(v0.x / scale, v0.y / scale);
                        for i in 1..vertices.len() - 1 {
                            let v1 = vertices[i];
                            let v2 = vertices[i + 1];
                            let uv1 = macroquad::prelude::Vec2::new(v1.x / scale, v1.y / scale);
                            let uv2 = macroquad::prelude::Vec2::new(v2.x / scale, v2.y / scale);
                            builder.push_triangle(v0, uv0, WHITE, v1, uv1, WHITE, v2, uv2, WHITE);
                        }
                        builder.flush();
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
        } else {
            render_surface_shape(shape, fill_col, border_col);
        }
    }));
}

/// Draws drop shadows and structural concrete slab deck for elevated bridge sections.
fn render_bridge_structure_pass(spline: &TrackSpline, view_bounds: Option<(Vec2, Vec2)>) {
    let samples = &spline.samples;
    let n = samples.len();
    if n < 2 {
        return;
    }
    let seg_count = if spline.closed { n } else { n.saturating_sub(1) };

    // Pass A: Bridge drop shadow on the ground / underpass beneath
    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];
        if !s0.is_bridge || !s1.is_bridge || !is_segment_in_view(s0, s1, view_bounds) {
            continue;
        }

        let avg_elev = (s0.elevation + s1.elevation) * 0.5;
        let s_off = Vec2::new(0.35, 0.55) * (avg_elev * 0.45 + 1.0);
        let hw0 = s0.width * 0.5 + 0.6;
        let hw1 = s1.width * 0.5 + 0.6;
        let left0 = s0.point + s0.normal * hw0 + s_off;
        let right0 = s0.point - s0.normal * hw0 + s_off;
        let left1 = s1.point + s1.normal * hw1 + s_off;
        let right1 = s1.point - s1.normal * hw1 + s_off;

        draw_quad(left0, left1, right1, right0, Palette::SHADOW);
    }

    // Pass B: Solid opaque concrete bridge deck undertray and side support beams
    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];
        if !s0.is_bridge || !s1.is_bridge || !is_segment_in_view(s0, s1, view_bounds) {
            continue;
        }

        // Bridge deck covers the entire track ribbon plus curbs and barrier mount width
        let deck_extra = 1.6;
        let hw0 = s0.width * 0.5 + deck_extra;
        let hw1 = s1.width * 0.5 + deck_extra;
        let left0 = s0.point + s0.normal * hw0;
        let right0 = s0.point - s0.normal * hw0;
        let left1 = s1.point + s1.normal * hw1;
        let right1 = s1.point - s1.normal * hw1;

        let deck_col = Color::new(0.13, 0.14, 0.17, 1.0);
        draw_quad(left0, left1, right1, right0, deck_col);

        // Deck outer border girder lines
        draw_line(left0.x, left0.y, left1.x, left1.y, 0.40, Color::new(0.32, 0.34, 0.38, 1.0));
        draw_line(right0.x, right0.y, right1.x, right1.y, 0.40, Color::new(0.32, 0.34, 0.38, 1.0));

        // Draw bridge expansion joint line at transition points
        if !s0.is_bridge && s1.is_bridge {
            draw_line(left0.x, left0.y, right0.x, right0.y, 0.50, Color::new(0.08, 0.08, 0.10, 1.0));
        } else if s0.is_bridge && !s1.is_bridge {
            draw_line(left1.x, left1.y, right1.x, right1.y, 0.50, Color::new(0.08, 0.08, 0.10, 1.0));
        }
    }
}

/// Draws track runoff ribbon quads between track/curb edge and wall boundary for ground or elevated segments.
fn render_runoff_pass(spline: &TrackSpline, elevated: bool, view_bounds: Option<(Vec2, Vec2)>) {
    let samples = &spline.samples;
    let n = samples.len();
    if n < 2 {
        return;
    }
    let curb_extra_width = 1.35;
    let seg_count = if spline.closed { n } else { n.saturating_sub(1) };

    let quality = get_surface_texture_quality();
    let mut runoff_builders: HashMap<SurfaceType, BatchMeshBuilder> = HashMap::new();
    let fringe_tex = if quality == SurfaceTextureQuality::High {
        with_surface_registry(|r| r.edge_fringe_texture().cloned()).flatten()
    } else {
        None
    };
    let mut fringe_builder = BatchMeshBuilder::new(fringe_tex);

    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];
        let is_seg_elevated = s0.is_bridge && s1.is_bridge;
        if is_seg_elevated != elevated || !is_segment_in_view(s0, s1, view_bounds) {
            continue;
        }

        let d0 = s0.distance;
        let d1 = if (i + 1) % n == 0 && spline.closed {
            s0.distance + (s1.point - s0.point).length()
        } else {
            s1.distance
        };
        let u0 = d0 / 2.0;
        let u1 = d1 / 2.0;

        // Left runoff corridor (constrained strictly to between track/curb edge and wall boundary)
        if s0.left_wall && s1.left_wall {
            if let Some(runoff_surf) = s0.left_runoff_surface {
                let hw0 = s0.width * 0.5;
                let hw1 = s1.width * 0.5;
                let curb_w0 = if s0.left_curb { curb_extra_width } else { 0.0 };
                let curb_w1 = if s1.left_curb { curb_extra_width } else { 0.0 };

                let wall_dist0 = s0.left_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);
                let wall_dist1 = s1.left_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);

                if wall_dist0 > curb_w0 && wall_dist1 > curb_w1 {
                    let p0_inner = s0.point + s0.normal * (hw0 + curb_w0);
                    let p1_inner = s1.point + s1.normal * (hw1 + curb_w1);
                    let p0_outer = s0.point + s0.normal * (hw0 + wall_dist0);
                    let p1_outer = s1.point + s1.normal * (hw1 + wall_dist1);

                    let builder = runoff_builders.entry(runoff_surf).or_insert_with(|| {
                        let (tex, _, _) = get_surface_material_info(runoff_surf);
                        BatchMeshBuilder::new(tex)
                    });

                    let scale = with_surface_registry(|r| r.tile_scale(runoff_surf)).unwrap_or(4.0);
                    let (fill_col, _) = get_surface_zone_colors(runoff_surf);

                    if quality != SurfaceTextureQuality::Off && builder.texture.is_some() {
                        let uv0 = macroquad::prelude::Vec2::new(p0_inner.x / scale, p0_inner.y / scale);
                        let uv1 = macroquad::prelude::Vec2::new(p1_inner.x / scale, p1_inner.y / scale);
                        let uv2 = macroquad::prelude::Vec2::new(p1_outer.x / scale, p1_outer.y / scale);
                        let uv3 = macroquad::prelude::Vec2::new(p0_outer.x / scale, p0_outer.y / scale);

                        let (c0, c1, c2, c3) = if quality == SurfaceTextureQuality::High {
                            let m0 = 1.0 + evaluate_macro_modulation(p0_inner.x, p0_inner.y) * 0.18;
                            let m1 = 1.0 + evaluate_macro_modulation(p1_inner.x, p1_inner.y) * 0.18;
                            let m2 = 1.0 + evaluate_macro_modulation(p1_outer.x, p1_outer.y) * 0.18;
                            let m3 = 1.0 + evaluate_macro_modulation(p0_outer.x, p0_outer.y) * 0.18;
                            (
                                Color::new(m0, m0, m0, 1.0),
                                Color::new(m1, m1, m1, 1.0),
                                Color::new(m2, m2, m2, 1.0),
                                Color::new(m3, m3, m3, 1.0),
                            )
                        } else {
                            (WHITE, WHITE, WHITE, WHITE)
                        };

                        builder.push_quad(p0_inner, uv0, c0, p1_inner, uv1, c1, p1_outer, uv2, c2, p0_outer, uv3, c3);

                        // Outer organic fringe feathering
                        if fringe_builder.texture.is_some() {
                            let p0_fringe = p0_outer + s0.normal * 0.6;
                            let p1_fringe = p1_outer + s1.normal * 0.6;
                            let fringe_c = Color::new(fill_col.r, fill_col.g, fill_col.b, 0.75);
                            fringe_builder.push_quad(
                                p0_outer, macroquad::prelude::Vec2::new(u0, 1.0), fringe_c,
                                p1_outer, macroquad::prelude::Vec2::new(u1, 1.0), fringe_c,
                                p1_fringe, macroquad::prelude::Vec2::new(u1, 0.0), fringe_c,
                                p0_fringe, macroquad::prelude::Vec2::new(u0, 0.0), fringe_c,
                            );
                        }
                    } else {
                        builder.push_quad(p0_inner, macroquad::prelude::Vec2::ZERO, fill_col, p1_inner, macroquad::prelude::Vec2::ZERO, fill_col, p1_outer, macroquad::prelude::Vec2::ZERO, fill_col, p0_outer, macroquad::prelude::Vec2::ZERO, fill_col);
                    }
                }
            }
        }

        // Right runoff corridor (constrained strictly to between track/curb edge and wall boundary)
        if s0.right_wall && s1.right_wall {
            if let Some(runoff_surf) = s0.right_runoff_surface {
                let hw0 = s0.width * 0.5;
                let hw1 = s1.width * 0.5;
                let curb_w0 = if s0.right_curb { curb_extra_width } else { 0.0 };
                let curb_w1 = if s1.right_curb { curb_extra_width } else { 0.0 };

                let wall_dist0 = s0.right_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);
                let wall_dist1 = s1.right_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);

                if wall_dist0 > curb_w0 && wall_dist1 > curb_w1 {
                    let p0_inner = s0.point - s0.normal * (hw0 + curb_w0);
                    let p1_inner = s1.point - s1.normal * (hw1 + curb_w1);
                    let p0_outer = s0.point - s0.normal * (hw0 + wall_dist0);
                    let p1_outer = s1.point - s1.normal * (hw1 + wall_dist1);

                    let builder = runoff_builders.entry(runoff_surf).or_insert_with(|| {
                        let (tex, _, _) = get_surface_material_info(runoff_surf);
                        BatchMeshBuilder::new(tex)
                    });

                    let scale = with_surface_registry(|r| r.tile_scale(runoff_surf)).unwrap_or(4.0);
                    let (fill_col, _) = get_surface_zone_colors(runoff_surf);

                    if quality != SurfaceTextureQuality::Off && builder.texture.is_some() {
                        let uv0 = macroquad::prelude::Vec2::new(p0_inner.x / scale, p0_inner.y / scale);
                        let uv1 = macroquad::prelude::Vec2::new(p1_inner.x / scale, p1_inner.y / scale);
                        let uv2 = macroquad::prelude::Vec2::new(p1_outer.x / scale, p1_outer.y / scale);
                        let uv3 = macroquad::prelude::Vec2::new(p0_outer.x / scale, p0_outer.y / scale);

                        let (c0, c1, c2, c3) = if quality == SurfaceTextureQuality::High {
                            let m0 = 1.0 + evaluate_macro_modulation(p0_inner.x, p0_inner.y) * 0.18;
                            let m1 = 1.0 + evaluate_macro_modulation(p1_inner.x, p1_inner.y) * 0.18;
                            let m2 = 1.0 + evaluate_macro_modulation(p1_outer.x, p1_outer.y) * 0.18;
                            let m3 = 1.0 + evaluate_macro_modulation(p0_outer.x, p0_outer.y) * 0.18;
                            (
                                Color::new(m0, m0, m0, 1.0),
                                Color::new(m1, m1, m1, 1.0),
                                Color::new(m2, m2, m2, 1.0),
                                Color::new(m3, m3, m3, 1.0),
                            )
                        } else {
                            (WHITE, WHITE, WHITE, WHITE)
                        };

                        builder.push_quad(p0_inner, uv0, c0, p1_inner, uv1, c1, p1_outer, uv2, c2, p0_outer, uv3, c3);

                        // Outer organic fringe feathering
                        if fringe_builder.texture.is_some() {
                            let p0_fringe = p0_outer - s0.normal * 0.6;
                            let p1_fringe = p1_outer - s1.normal * 0.6;
                            let fringe_c = Color::new(fill_col.r, fill_col.g, fill_col.b, 0.75);
                            fringe_builder.push_quad(
                                p0_outer, macroquad::prelude::Vec2::new(u0, 1.0), fringe_c,
                                p1_outer, macroquad::prelude::Vec2::new(u1, 1.0), fringe_c,
                                p1_fringe, macroquad::prelude::Vec2::new(u1, 0.0), fringe_c,
                                p0_fringe, macroquad::prelude::Vec2::new(u0, 0.0), fringe_c,
                            );
                        }
                    } else {
                        builder.push_quad(p0_inner, macroquad::prelude::Vec2::ZERO, fill_col, p1_inner, macroquad::prelude::Vec2::ZERO, fill_col, p1_outer, macroquad::prelude::Vec2::ZERO, fill_col, p0_outer, macroquad::prelude::Vec2::ZERO, fill_col);
                    }
                }
            }
        }
    }

    for (_, mut builder) in runoff_builders {
        builder.flush();
    }
    fringe_builder.flush();
}

/// Draws curb rumble strips for either ground or elevated bridge segments.
fn render_curbs_pass(spline: &TrackSpline, elevated: bool, view_bounds: Option<(Vec2, Vec2)>) {
    let samples = &spline.samples;
    let n = samples.len();
    if n < 2 {
        return;
    }
    let curb_extra_width = 1.35;
    let seg_count = if spline.closed { n } else { n.saturating_sub(1) };

    let (curb_tex, curb_tile_scale, quality) = get_curb_material_info();
    let mut curb_builder = BatchMeshBuilder::new(curb_tex);

    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];
        let is_seg_elevated = s0.is_bridge && s1.is_bridge;
        if is_seg_elevated != elevated || !is_segment_in_view(s0, s1, view_bounds) {
            continue;
        }

        let d0 = s0.distance;
        let d1 = if (i + 1) % n == 0 && spline.closed {
            s0.distance + (s1.point - s0.point).length()
        } else {
            s1.distance
        };

        let u0 = d0 / curb_tile_scale;
        let u1 = d1 / curb_tile_scale;

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

            if quality != SurfaceTextureQuality::Off && curb_builder.texture.is_some() {
                curb_builder.push_quad(
                    p0_inner, macroquad::prelude::Vec2::new(u0, 0.0), WHITE,
                    p1_inner, macroquad::prelude::Vec2::new(u1, 0.0), WHITE,
                    p1_outer, macroquad::prelude::Vec2::new(u1, 1.0), WHITE,
                    p0_outer, macroquad::prelude::Vec2::new(u0, 1.0), WHITE,
                );
            } else {
                curb_builder.push_quad(
                    p0_inner, macroquad::prelude::Vec2::ZERO, curb_color,
                    p1_inner, macroquad::prelude::Vec2::ZERO, curb_color,
                    p1_outer, macroquad::prelude::Vec2::ZERO, curb_color,
                    p0_outer, macroquad::prelude::Vec2::ZERO, curb_color,
                );
            }
        }

        // Right curb
        if s0.right_curb || s1.right_curb {
            let hw0 = s0.width * 0.5;
            let hw1 = s1.width * 0.5;
            let p0_inner = s0.point - s0.normal * hw0;
            let p1_inner = s1.point - s1.normal * hw1;
            let p0_outer = s0.point - s0.normal * (hw0 + curb_extra_width);
            let p1_outer = s1.point - s1.normal * (hw1 + curb_extra_width);

            if quality != SurfaceTextureQuality::Off && curb_builder.texture.is_some() {
                curb_builder.push_quad(
                    p0_inner, macroquad::prelude::Vec2::new(u0, 0.0), WHITE,
                    p1_inner, macroquad::prelude::Vec2::new(u1, 0.0), WHITE,
                    p1_outer, macroquad::prelude::Vec2::new(u1, 1.0), WHITE,
                    p0_outer, macroquad::prelude::Vec2::new(u0, 1.0), WHITE,
                );
            } else {
                curb_builder.push_quad(
                    p0_inner, macroquad::prelude::Vec2::ZERO, curb_color,
                    p1_inner, macroquad::prelude::Vec2::ZERO, curb_color,
                    p1_outer, macroquad::prelude::Vec2::ZERO, curb_color,
                    p0_outer, macroquad::prelude::Vec2::ZERO, curb_color,
                );
            }
        }
    }

    curb_builder.flush();
}

/// Draws track surface quads (asphalt/dirt) for either ground or elevated bridge segments.
fn render_surface_pass(spline: &TrackSpline, elevated: bool, view_bounds: Option<(Vec2, Vec2)>) {
    let samples = &spline.samples;
    let n = samples.len();
    if n < 2 {
        return;
    }
    let seg_count = if spline.closed { n } else { n.saturating_sub(1) };

    let quality = get_surface_texture_quality();
    let mut surface_builders: HashMap<SurfaceType, BatchMeshBuilder> = HashMap::new();
    let mut lines_to_draw: Vec<(Vec2, Vec2, f32, Color)> = Vec::with_capacity(seg_count * 2);

    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];
        let is_seg_elevated = s0.is_bridge && s1.is_bridge;
        if is_seg_elevated != elevated || !is_segment_in_view(s0, s1, view_bounds) {
            continue;
        }

        let hw0 = s0.width * 0.5;
        let hw1 = s1.width * 0.5;

        let left0 = s0.point + s0.normal * hw0;
        let right0 = s0.point - s0.normal * hw0;
        let left1 = s1.point + s1.normal * hw1;
        let right1 = s1.point - s1.normal * hw1;

        let avg_bank = (s0.bank_angle + s1.bank_angle) * 0.5;
        let is_banked = avg_bank.abs() > 0.8;

        // On banked curves, render subtle 2.5D outer rim embankment shadow under outer edge
        if is_banked {
            let rim_offset = if avg_bank > 0.0 {
                -s0.normal * (avg_bank.abs().min(25.0) * 0.03 + 0.30)
            } else {
                s0.normal * (avg_bank.abs().min(25.0) * 0.03 + 0.30)
            };
            if avg_bank > 0.0 {
                let r0_shad = right0 + rim_offset;
                let r1_shad = right1 + rim_offset;
                draw_quad(right0, right1, r1_shad, r0_shad, Palette::SHADOW);
            } else {
                let l0_shad = left0 + rim_offset;
                let l1_shad = left1 + rim_offset;
                draw_quad(left0, left1, l1_shad, l0_shad, Palette::SHADOW);
            }
        }

        let d0 = s0.distance;
        let d1 = if (i + 1) % n == 0 && spline.closed {
            s0.distance + (s1.point - s0.point).length()
        } else {
            s1.distance
        };

        let surf = s0.surface;
        let builder = surface_builders.entry(surf).or_insert_with(|| {
            let (tex, _, _) = get_surface_material_info(surf);
            BatchMeshBuilder::new(tex)
        });

        let tile_scale = with_surface_registry(|r| r.tile_scale(surf)).unwrap_or(4.0);
        let v0 = d0 / tile_scale;
        let v1 = d1 / tile_scale;
        let has_tex = quality != SurfaceTextureQuality::Off && builder.texture.is_some();

        match surf {
            SurfaceType::Dirt => {
                if is_banked {
                    let mid_l0 = s0.point + s0.normal * (hw0 * 0.30);
                    let mid_l1 = s1.point + s1.normal * (hw1 * 0.30);
                    let mid_r0 = s0.point - s0.normal * (hw0 * 0.30);
                    let mid_r1 = s1.point - s1.normal * (hw1 * 0.30);

                    let (c_low, c_mid, c_high) = if has_tex {
                        (
                            Color::new(0.68, 0.65, 0.60, 1.0),
                            Color::new(0.95, 0.95, 0.95, 1.0),
                            Color::new(1.15, 1.15, 1.10, 1.0),
                        )
                    } else {
                        (
                            Palette::DIRT_DARK,
                            Palette::DIRT,
                            Color::new(0.60, 0.44, 0.28, 1.0),
                        )
                    };

                    let (c_l, c_m, c_r) = if avg_bank > 0.0 {
                        (c_low, c_mid, c_high)
                    } else {
                        (c_high, c_mid, c_low)
                    };

                    let uv_l0 = macroquad::prelude::Vec2::new(0.0, v0);
                    let uv_l1 = macroquad::prelude::Vec2::new(0.0, v1);
                    let uv_ml0 = macroquad::prelude::Vec2::new(0.35, v0);
                    let uv_ml1 = macroquad::prelude::Vec2::new(0.35, v1);
                    let uv_mr0 = macroquad::prelude::Vec2::new(0.65, v0);
                    let uv_mr1 = macroquad::prelude::Vec2::new(0.65, v1);
                    let uv_r0 = macroquad::prelude::Vec2::new(1.0, v0);
                    let uv_r1 = macroquad::prelude::Vec2::new(1.0, v1);

                    let (c_l, c_m, c_r) = if has_tex && quality == SurfaceTextureQuality::High {
                        let m_l = 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18;
                        let m_m = 1.0 + evaluate_macro_modulation(mid_l0.x, mid_l0.y) * 0.18;
                        let m_r = 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18;
                        (
                            Color::new((c_l.r * m_l).clamp(0.1, 1.8), (c_l.g * m_l).clamp(0.1, 1.8), (c_l.b * m_l).clamp(0.1, 1.8), 1.0),
                            Color::new((c_m.r * m_m).clamp(0.1, 1.8), (c_m.g * m_m).clamp(0.1, 1.8), (c_m.b * m_m).clamp(0.1, 1.8), 1.0),
                            Color::new((c_r.r * m_r).clamp(0.1, 1.8), (c_r.g * m_r).clamp(0.1, 1.8), (c_r.b * m_r).clamp(0.1, 1.8), 1.0),
                        )
                    } else {
                        (c_l, c_m, c_r)
                    };

                    builder.push_quad(left0, uv_l0, c_l, left1, uv_l1, c_l, mid_l1, uv_ml1, c_l, mid_l0, uv_ml0, c_l);
                    builder.push_quad(mid_l0, uv_ml0, c_m, mid_l1, uv_ml1, c_m, mid_r1, uv_mr1, c_m, mid_r0, uv_mr0, c_m);
                    builder.push_quad(mid_r0, uv_mr0, c_r, mid_r1, uv_mr1, c_r, right1, uv_r1, c_r, right0, uv_r0, c_r);

                    lines_to_draw.push((left0, left1, 0.32, Palette::DIRT_EDGE));
                    lines_to_draw.push((right0, right1, 0.32, Palette::DIRT_EDGE));
                } else {
                    let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                    let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                    let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                    let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                    let (c0, c1, c2, c3) = if has_tex {
                        if quality == SurfaceTextureQuality::High {
                            let m0 = 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18;
                            let m1 = 1.0 + evaluate_macro_modulation(left1.x, left1.y) * 0.18;
                            let m2 = 1.0 + evaluate_macro_modulation(right1.x, right1.y) * 0.18;
                            let m3 = 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18;
                            (Color::new(m0, m0, m0, 1.0), Color::new(m1, m1, m1, 1.0), Color::new(m2, m2, m2, 1.0), Color::new(m3, m3, m3, 1.0))
                        } else {
                            (WHITE, WHITE, WHITE, WHITE)
                        }
                    } else {
                        (Palette::DIRT, Palette::DIRT, Palette::DIRT, Palette::DIRT)
                    };

                    builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                    lines_to_draw.push((left0, left1, 0.32, Palette::DIRT_EDGE));
                    lines_to_draw.push((right0, right1, 0.32, Palette::DIRT_EDGE));

                    let groove_l0 = s0.point + s0.normal * (hw0 * 0.45);
                    let groove_l1 = s1.point + s1.normal * (hw1 * 0.45);
                    let groove_r0 = s0.point - s0.normal * (hw0 * 0.45);
                    let groove_r1 = s1.point - s1.normal * (hw1 * 0.45);
                    lines_to_draw.push((groove_l0, groove_l1, 0.22, Palette::DIRT_DARK));
                    lines_to_draw.push((groove_r0, groove_r1, 0.22, Palette::DIRT_DARK));
                }
            }
            SurfaceType::Sand => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let (c0, c1, c2, c3) = if has_tex {
                    if quality == SurfaceTextureQuality::High {
                        let m0 = 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18;
                        let m1 = 1.0 + evaluate_macro_modulation(left1.x, left1.y) * 0.18;
                        let m2 = 1.0 + evaluate_macro_modulation(right1.x, right1.y) * 0.18;
                        let m3 = 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18;
                        (Color::new(m0, m0, m0, 1.0), Color::new(m1, m1, m1, 1.0), Color::new(m2, m2, m2, 1.0), Color::new(m3, m3, m3, 1.0))
                    } else {
                        (WHITE, WHITE, WHITE, WHITE)
                    }
                } else {
                    (Palette::SAND, Palette::SAND, Palette::SAND, Palette::SAND)
                };
                builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                lines_to_draw.push((left0, left1, 0.32, Palette::SAND_DARK));
                lines_to_draw.push((right0, right1, 0.32, Palette::SAND_DARK));
            }
            SurfaceType::Grass => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let (c0, c1, c2, c3) = if has_tex {
                    if quality == SurfaceTextureQuality::High {
                        let m0 = 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18;
                        let m1 = 1.0 + evaluate_macro_modulation(left1.x, left1.y) * 0.18;
                        let m2 = 1.0 + evaluate_macro_modulation(right1.x, right1.y) * 0.18;
                        let m3 = 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18;
                        (Color::new(m0, m0, m0, 1.0), Color::new(m1, m1, m1, 1.0), Color::new(m2, m2, m2, 1.0), Color::new(m3, m3, m3, 1.0))
                    } else {
                        (WHITE, WHITE, WHITE, WHITE)
                    }
                } else {
                    (Palette::GRASS_DARK, Palette::GRASS_DARK, Palette::GRASS_DARK, Palette::GRASS_DARK)
                };
                builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                lines_to_draw.push((left0, left1, 0.32, Palette::GRASS));
                lines_to_draw.push((right0, right1, 0.32, Palette::GRASS));
            }
            SurfaceType::Ice => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { Color::new(1.0, 1.0, 1.0, 0.95) } else { Color::new(0.85, 0.92, 0.98, 0.95) };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                lines_to_draw.push((left0, left1, 0.32, Color::new(0.65, 0.82, 0.95, 0.8)));
                lines_to_draw.push((right0, right1, 0.32, Color::new(0.65, 0.82, 0.95, 0.8)));
            }
            SurfaceType::Water => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { WHITE } else { Palette::WATER };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                lines_to_draw.push((left0, left1, 0.32, Palette::WATER_BORDER));
                lines_to_draw.push((right0, right1, 0.32, Palette::WATER_BORDER));
            }
            SurfaceType::Oil => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { WHITE } else { Color::new(0.12, 0.12, 0.15, 0.95) };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                lines_to_draw.push((left0, left1, 0.32, Color::new(0.35, 0.25, 0.40, 0.85)));
                lines_to_draw.push((right0, right1, 0.32, Color::new(0.35, 0.25, 0.40, 0.85)));
            }
            SurfaceType::Curb => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { WHITE } else { Palette::CURB_RED };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
            }
            SurfaceType::Mud => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let (c0, c1, c2, c3) = if has_tex {
                    if quality == SurfaceTextureQuality::High {
                        let m0 = 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18;
                        let m1 = 1.0 + evaluate_macro_modulation(left1.x, left1.y) * 0.18;
                        let m2 = 1.0 + evaluate_macro_modulation(right1.x, right1.y) * 0.18;
                        let m3 = 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18;
                        (Color::new(m0, m0, m0, 1.0), Color::new(m1, m1, m1, 1.0), Color::new(m2, m2, m2, 1.0), Color::new(m3, m3, m3, 1.0))
                    } else {
                        (WHITE, WHITE, WHITE, WHITE)
                    }
                } else {
                    (Palette::MUD, Palette::MUD, Palette::MUD, Palette::MUD)
                };
                builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                lines_to_draw.push((left0, left1, 0.34, Palette::MUD_DARK));
                lines_to_draw.push((right0, right1, 0.34, Palette::MUD_DARK));

                let rut_l0 = s0.point + s0.normal * (hw0 * 0.40);
                let rut_l1 = s1.point + s1.normal * (hw1 * 0.40);
                let rut_r0 = s0.point - s0.normal * (hw0 * 0.40);
                let rut_r1 = s1.point - s1.normal * (hw1 * 0.40);
                lines_to_draw.push((rut_l0, rut_l1, 0.26, Palette::MUD_DARK));
                lines_to_draw.push((rut_r0, rut_r1, 0.26, Palette::MUD_DARK));
            }
            SurfaceType::Snow => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let (c0, c1, c2, c3) = if has_tex {
                    if quality == SurfaceTextureQuality::High {
                        let m0 = 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18;
                        let m1 = 1.0 + evaluate_macro_modulation(left1.x, left1.y) * 0.18;
                        let m2 = 1.0 + evaluate_macro_modulation(right1.x, right1.y) * 0.18;
                        let m3 = 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18;
                        (Color::new(m0, m0, m0, 1.0), Color::new(m1, m1, m1, 1.0), Color::new(m2, m2, m2, 1.0), Color::new(m3, m3, m3, 1.0))
                    } else {
                        (WHITE, WHITE, WHITE, WHITE)
                    }
                } else {
                    (Palette::SNOW, Palette::SNOW, Palette::SNOW, Palette::SNOW)
                };
                builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                lines_to_draw.push((left0, left1, 0.32, Palette::SNOW_EDGE));
                lines_to_draw.push((right0, right1, 0.32, Palette::SNOW_EDGE));

                let track_l0 = s0.point + s0.normal * (hw0 * 0.42);
                let track_l1 = s1.point + s1.normal * (hw1 * 0.42);
                let track_r0 = s0.point - s0.normal * (hw0 * 0.42);
                let track_r1 = s1.point - s1.normal * (hw1 * 0.42);
                lines_to_draw.push((track_l0, track_l1, 0.20, Color::new(0.85, 0.90, 0.96, 0.90)));
                lines_to_draw.push((track_r0, track_r1, 0.20, Color::new(0.85, 0.90, 0.96, 0.90)));
            }
            SurfaceType::Gravel => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let (c0, c1, c2, c3) = if has_tex {
                    if quality == SurfaceTextureQuality::High {
                        let m0 = 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18;
                        let m1 = 1.0 + evaluate_macro_modulation(left1.x, left1.y) * 0.18;
                        let m2 = 1.0 + evaluate_macro_modulation(right1.x, right1.y) * 0.18;
                        let m3 = 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18;
                        (Color::new(m0, m0, m0, 1.0), Color::new(m1, m1, m1, 1.0), Color::new(m2, m2, m2, 1.0), Color::new(m3, m3, m3, 1.0))
                    } else {
                        (WHITE, WHITE, WHITE, WHITE)
                    }
                } else {
                    (Palette::GRAVEL, Palette::GRAVEL, Palette::GRAVEL, Palette::GRAVEL)
                };
                builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                lines_to_draw.push((left0, left1, 0.32, Palette::GRAVEL_EDGE));
                lines_to_draw.push((right0, right1, 0.32, Palette::GRAVEL_EDGE));

                let track_l0 = s0.point + s0.normal * (hw0 * 0.44);
                let track_l1 = s1.point + s1.normal * (hw1 * 0.44);
                let track_r0 = s0.point - s0.normal * (hw0 * 0.44);
                let track_r1 = s1.point - s1.normal * (hw1 * 0.44);
                lines_to_draw.push((track_l0, track_l1, 0.22, Palette::GRAVEL_DARK));
                lines_to_draw.push((track_r0, track_r1, 0.22, Palette::GRAVEL_DARK));
            }
            SurfaceType::Asphalt => {
                let mid_l0 = s0.point + s0.normal * (hw0 * 0.33);
                let mid_l1 = s1.point + s1.normal * (hw1 * 0.33);
                let mid_r0 = s0.point - s0.normal * (hw0 * 0.33);
                let mid_r1 = s1.point - s1.normal * (hw1 * 0.33);

                // Isotropic 1:1 metric UV mapping: scale lateral U by physical track width
                let u_span0 = s0.width / tile_scale;
                let u_span1 = s1.width / tile_scale;

                let uv_l0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv_l1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv_ml0 = macroquad::prelude::Vec2::new(u_span0 * 0.335, v0);
                let uv_ml1 = macroquad::prelude::Vec2::new(u_span1 * 0.335, v1);
                let uv_mr0 = macroquad::prelude::Vec2::new(u_span0 * 0.665, v0);
                let uv_mr1 = macroquad::prelude::Vec2::new(u_span1 * 0.665, v1);
                let uv_r0 = macroquad::prelude::Vec2::new(u_span0, v0);
                let uv_r1 = macroquad::prelude::Vec2::new(u_span1, v1);

                // Longitudinal slope lighting factor
                let avg_slope = (s0.grade_slope + s1.grade_slope) * 0.5;
                let slope_factor = if avg_slope.abs() > 0.02 {
                    let light_dir = Vec2::new(-0.6, -0.8).normalize();
                    let fwd_dot_light = s0.tangent.dot(light_dir);
                    (1.0 + avg_slope * fwd_dot_light * 0.8).clamp(0.70, 1.25)
                } else {
                    1.0
                };

                // Cross-slope banking gradient multipliers
                let (bank_l, bank_m, bank_r) = if is_banked {
                    if avg_bank > 0.0 {
                        (0.65, 0.95, 1.18)
                    } else {
                        (1.18, 0.95, 0.65)
                    }
                } else {
                    (1.0, 1.0, 1.0)
                };

                if has_tex {
                    let m_l0 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(left0.x, left0.y) * 0.18 } else { 1.0 };
                    let m_l1 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(left1.x, left1.y) * 0.18 } else { 1.0 };
                    let m_ml0 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(mid_l0.x, mid_l0.y) * 0.18 } else { 1.0 };
                    let m_ml1 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(mid_l1.x, mid_l1.y) * 0.18 } else { 1.0 };
                    let m_mr0 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(mid_r0.x, mid_r0.y) * 0.18 } else { 1.0 };
                    let m_mr1 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(mid_r1.x, mid_r1.y) * 0.18 } else { 1.0 };
                    let m_r0 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(right0.x, right0.y) * 0.18 } else { 1.0 };
                    let m_r1 = if quality == SurfaceTextureQuality::High { 1.0 + evaluate_macro_modulation(right1.x, right1.y) * 0.18 } else { 1.0 };

                    let f_l0 = (bank_l * slope_factor * m_l0).clamp(0.2, 1.8);
                    let f_l1 = (bank_l * slope_factor * m_l1).clamp(0.2, 1.8);
                    let f_ml0 = (bank_m * slope_factor * m_ml0).clamp(0.2, 1.8);
                    let f_ml1 = (bank_m * slope_factor * m_ml1).clamp(0.2, 1.8);
                    let f_mr0 = (bank_m * slope_factor * m_mr0).clamp(0.2, 1.8);
                    let f_mr1 = (bank_m * slope_factor * m_mr1).clamp(0.2, 1.8);
                    let f_r0 = (bank_r * slope_factor * m_r0).clamp(0.2, 1.8);
                    let f_r1 = (bank_r * slope_factor * m_r1).clamp(0.2, 1.8);

                    let c_l0 = Color::new(f_l0, f_l0, f_l0, 1.0);
                    let c_l1 = Color::new(f_l1, f_l1, f_l1, 1.0);
                    let c_ml0 = Color::new(f_ml0, f_ml0, f_ml0, 1.0);
                    let c_ml1 = Color::new(f_ml1, f_ml1, f_ml1, 1.0);
                    let c_mr0 = Color::new(f_mr0, f_mr0, f_mr0, 1.0);
                    let c_mr1 = Color::new(f_mr1, f_mr1, f_mr1, 1.0);
                    let c_r0 = Color::new(f_r0, f_r0, f_r0, 1.0);
                    let c_r1 = Color::new(f_r1, f_r1, f_r1, 1.0);

                    builder.push_quad(left0, uv_l0, c_l0, left1, uv_l1, c_l1, mid_l1, uv_ml1, c_ml1, mid_l0, uv_ml0, c_ml0);
                    builder.push_quad(mid_l0, uv_ml0, c_ml0, mid_l1, uv_ml1, c_ml1, mid_r1, uv_mr1, c_mr1, mid_r0, uv_mr0, c_mr0);
                    builder.push_quad(mid_r0, uv_mr0, c_mr0, mid_r1, uv_mr1, c_mr1, right1, uv_r1, c_r1, right0, uv_r0, c_r0);
                } else {
                    let (base_l, base_m, base_r) = if is_banked {
                        let (c_low, c_mid, c_high) = (
                            Color::new(0.12, 0.13, 0.16, 1.0),
                            Palette::ASPHALT,
                            Color::new(0.24, 0.25, 0.29, 1.0),
                        );
                        if avg_bank > 0.0 {
                            (c_low, c_mid, c_high)
                        } else {
                            (c_high, c_mid, c_low)
                        }
                    } else {
                        (Palette::ASPHALT, Palette::ASPHALT, Palette::ASPHALT)
                    };

                    let c_l = base_l;
                    let c_m = base_m;
                    let c_r = base_r;

                    builder.push_quad(left0, uv_l0, c_l, left1, uv_l1, c_l, mid_l1, uv_ml1, c_l, mid_l0, uv_ml0, c_l);
                    builder.push_quad(mid_l0, uv_ml0, c_m, mid_l1, uv_ml1, c_m, mid_r1, uv_mr1, c_m, mid_r0, uv_mr0, c_m);
                    builder.push_quad(mid_r0, uv_mr0, c_r, mid_r1, uv_mr1, c_r, right1, uv_r1, c_r, right0, uv_r0, c_r);
                }

                lines_to_draw.push((left0, left1, 0.28, Palette::WHITE_LINE));
                lines_to_draw.push((right0, right1, 0.28, Palette::WHITE_LINE));

                if is_banked {
                    lines_to_draw.push((mid_l0, mid_l1, 0.12, Color::new(0.40, 0.42, 0.46, 0.4)));
                    lines_to_draw.push((mid_r0, mid_r1, 0.12, Color::new(0.40, 0.42, 0.46, 0.4)));
                } else {
                    let center_stripe = ((s0.distance / 3.0).floor() as usize).is_multiple_of(2);
                    if center_stripe {
                        lines_to_draw.push((s0.point, s1.point, 0.16, Color::new(0.95, 0.95, 0.95, 0.35)));
                    }
                }
            }
            SurfaceType::Concrete => {
                if is_banked {
                    let mid_l0 = s0.point + s0.normal * (hw0 * 0.33);
                    let mid_l1 = s1.point + s1.normal * (hw1 * 0.33);
                    let mid_r0 = s0.point - s0.normal * (hw0 * 0.33);
                    let mid_r1 = s1.point - s1.normal * (hw1 * 0.33);

                    let (c_low, c_mid, c_high) = if has_tex {
                        (
                            Color::new(0.72, 0.74, 0.76, 1.0),
                            Color::new(0.96, 0.96, 0.98, 1.0),
                            Color::new(1.15, 1.16, 1.18, 1.0),
                        )
                    } else {
                        (
                            Color::new(0.62, 0.64, 0.66, 1.0),
                            Palette::CONCRETE,
                            Color::new(0.78, 0.80, 0.82, 1.0),
                        )
                    };

                    let (c_l, c_m, c_r) = if avg_bank > 0.0 {
                        (c_low, c_mid, c_high)
                    } else {
                        (c_high, c_mid, c_low)
                    };

                    let uv_l0 = macroquad::prelude::Vec2::new(0.0, v0);
                    let uv_l1 = macroquad::prelude::Vec2::new(0.0, v1);
                    let uv_ml0 = macroquad::prelude::Vec2::new(0.335, v0);
                    let uv_ml1 = macroquad::prelude::Vec2::new(0.335, v1);
                    let uv_mr0 = macroquad::prelude::Vec2::new(0.665, v0);
                    let uv_mr1 = macroquad::prelude::Vec2::new(0.665, v1);
                    let uv_r0 = macroquad::prelude::Vec2::new(1.0, v0);
                    let uv_r1 = macroquad::prelude::Vec2::new(1.0, v1);

                    builder.push_quad(left0, uv_l0, c_l, left1, uv_l1, c_l, mid_l1, uv_ml1, c_l, mid_l0, uv_ml0, c_l);
                    builder.push_quad(mid_l0, uv_ml0, c_m, mid_l1, uv_ml1, c_m, mid_r1, uv_mr1, c_m, mid_r0, uv_mr0, c_m);
                    builder.push_quad(mid_r0, uv_mr0, c_r, mid_r1, uv_mr1, c_r, right1, uv_r1, c_r, right0, uv_r0, c_r);

                    lines_to_draw.push((left0, left1, 0.28, Palette::WHITE_LINE));
                    lines_to_draw.push((right0, right1, 0.28, Palette::WHITE_LINE));
                    lines_to_draw.push((mid_l0, mid_l1, 0.12, Color::new(0.50, 0.52, 0.55, 0.45)));
                    lines_to_draw.push((mid_r0, mid_r1, 0.12, Color::new(0.50, 0.52, 0.55, 0.45)));
                } else {
                    let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                    let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                    let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                    let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                    let col = if has_tex { WHITE } else { Palette::CONCRETE };

                    builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                    lines_to_draw.push((left0, left1, 0.28, Palette::WHITE_LINE));
                    lines_to_draw.push((right0, right1, 0.28, Palette::WHITE_LINE));

                    let center_stripe = ((s0.distance / 3.0).floor() as usize).is_multiple_of(2);
                    if center_stripe {
                        lines_to_draw.push((s0.point, s1.point, 0.16, Color::new(0.95, 0.95, 0.95, 0.35)));
                    }
                }
            }
        }
    }

    // Flush all batched surface meshes
    for (_, mut builder) in surface_builders {
        builder.flush();
    }

    // Render markings and seams on top of the surface
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for (p0, p1, th, col) in lines_to_draw {
            draw_line(p0.x, p0.y, p1.x, p1.y, th, col);
        }
    }));
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
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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
    }));
}

/// Utility to draw a filled convex quad from 4 vertices in CCW/CW order.
#[inline]
pub fn draw_quad(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, color: Color) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_and_insufficient_spline_rendering_does_not_panic() {
        let empty_spline = TrackSpline::empty();
        render_curbs_pass(&empty_spline, false, None);
        render_curbs_pass(&empty_spline, true, None);
        render_surface_pass(&empty_spline, false, None);
        render_surface_pass(&empty_spline, true, None);
        render_runoff_pass(&empty_spline, false, None);
        render_runoff_pass(&empty_spline, true, None);
        render_bridge_structure_pass(&empty_spline, None);

        // Spline with 1 sample
        let mut single_spline = TrackSpline::empty();
        single_spline.samples.push(SplineSample {
            point: Vec2::ZERO,
            tangent: Vec2::X,
            normal: Vec2::Y,
            distance: 0.0,
            width: 10.0,
            left_curb: true,
            right_curb: true,
            surface: SurfaceType::Asphalt,
            elevation: 1.0,
            bank_angle: 0.0,
            is_bridge: false,
            grade_slope: 0.0,
            vertical_curvature: 0.0,
            left_wall: false,
            right_wall: false,
            left_wall_distance: None,
            right_wall_distance: None,
            wall_type: None,
            left_runoff_surface: Some(SurfaceType::Gravel),
            right_runoff_surface: Some(SurfaceType::Sand),
        });
        render_runoff_pass(&single_spline, false, None);
        render_runoff_pass(&single_spline, true, None);
        render_curbs_pass(&single_spline, false, None);
        render_curbs_pass(&single_spline, true, None);
        render_surface_pass(&single_spline, false, None);
        render_surface_pass(&single_spline, true, None);
        render_bridge_structure_pass(&single_spline, None);
    }

    #[test]
    fn test_batch_mesh_builder_quad_and_triangle_indices() {
        let mut builder = BatchMeshBuilder::new(None);
        assert_eq!(builder.vertices.len(), 0);
        assert_eq!(builder.indices.len(), 0);

        builder.push_quad(
            Vec2::new(0.0, 0.0), macroquad::prelude::Vec2::ZERO, WHITE,
            Vec2::new(1.0, 0.0), macroquad::prelude::Vec2::X, WHITE,
            Vec2::new(1.0, 1.0), macroquad::prelude::Vec2::ONE, WHITE,
            Vec2::new(0.0, 1.0), macroquad::prelude::Vec2::Y, WHITE,
        );
        assert_eq!(builder.vertices.len(), 4);
        assert_eq!(builder.indices.len(), 6);
        assert_eq!(builder.indices, vec![0, 1, 2, 0, 2, 3]);

        builder.push_triangle(
            Vec2::new(2.0, 0.0), macroquad::prelude::Vec2::ZERO, WHITE,
            Vec2::new(3.0, 0.0), macroquad::prelude::Vec2::X, WHITE,
            Vec2::new(2.5, 1.0), macroquad::prelude::Vec2::Y, WHITE,
        );
        assert_eq!(builder.vertices.len(), 7);
        assert_eq!(builder.indices.len(), 9);
        assert_eq!(&builder.indices[6..9], &[4, 5, 6]);
    }

    #[test]
    fn test_textured_rendering_across_quality_levels() {
        use tdrace_core::track::presets::classic_grand_prix;

        let track = classic_grand_prix();
        let qualities = [
            SurfaceTextureQuality::Off,
            SurfaceTextureQuality::Standard,
            SurfaceTextureQuality::High,
        ];

        for &q in &qualities {
            set_surface_texture_quality(q);
            assert_eq!(get_surface_texture_quality(), q);

            // Verify surface passes execute safely without panic
            render_runoff_pass(&track.spline, false, None);
            render_curbs_pass(&track.spline, false, None);
            render_surface_pass(&track.spline, false, None);
            render_surface_zones_layer(&track, SurfaceLayer::BelowTrack);
            render_surface_zones_layer(&track, SurfaceLayer::AboveTrack);
        }

        // Restore High default
        set_surface_texture_quality(SurfaceTextureQuality::High);
    }

    #[test]
    fn test_render_surface_shape_textured_all_geometries() {
        let aabb = SurfaceShape::Aabb {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(10.0, 10.0),
        };
        let circle = SurfaceShape::Circle {
            center: Vec2::new(5.0, 5.0),
            radius: 4.0,
        };
        let obox = SurfaceShape::OrientedBox {
            center: Vec2::new(15.0, 15.0),
            half_extents: Vec2::new(5.0, 2.0),
            angle: 0.785,
        };
        let poly = SurfaceShape::Polygon {
            vertices: vec![
                Vec2::new(20.0, 20.0),
                Vec2::new(25.0, 20.0),
                Vec2::new(25.0, 25.0),
                Vec2::new(20.0, 25.0),
            ],
        };

        let surfaces = [
            SurfaceType::Asphalt,
            SurfaceType::Dirt,
            SurfaceType::Grass,
            SurfaceType::Gravel,
            SurfaceType::Sand,
        ];

        for &surf in &surfaces {
            let (fill, border) = get_surface_zone_colors(surf);
            render_surface_shape_textured(&aabb, surf, fill, border);
            render_surface_shape_textured(&circle, surf, fill, border);
            render_surface_shape_textured(&obox, surf, fill, border);
            render_surface_shape_textured(&poly, surf, fill, border);
        }
    }
}
