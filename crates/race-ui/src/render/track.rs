use std::collections::HashMap;
use std::sync::Mutex;
use glam::Vec2;
use macroquad::color::{Color, WHITE};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_triangle};
use wheelbase::surface::SurfaceType;
use arcade_race_core::track::geometry::{LineSegment, PitLane, SurfaceLayer, SurfaceShape};
use arcade_race_core::track::network::{JunctionKind, RoadSegment};
use arcade_race_core::track::spline::{SplineSample, TrackSpline};
use arcade_race_core::track::Track;

use super::barrier;
use super::color::Palette;
use super::surface_material::{evaluate_macro_modulation, SurfaceMaterialRegistry, SurfaceTextureQuality};

/// Validates whether a quad (p0_in -> p1_in -> p1_out -> p0_out) is non-degenerate
/// and non-self-intersecting (preventing bowtie/hourglass artifacts).
#[inline]
fn is_quad_valid(p0_in: Vec2, p1_in: Vec2, p1_out: Vec2, p0_out: Vec2) -> bool {
    let in_len_sq = (p1_in - p0_in).length_squared();
    let out_len_sq = (p1_out - p0_out).length_squared();
    if in_len_sq < 1e-4 && out_len_sq < 1e-4 {
        return false;
    }
    if in_len_sq > 1e-4 && out_len_sq > 1e-4 {
        let seg_radial_0 = LineSegment::new(p0_in, p0_out);
        let seg_radial_1 = LineSegment::new(p1_in, p1_out);
        if seg_radial_0.intersect_segment(&seg_radial_1).is_some() {
            return false;
        }
    }
    true
}

/// Edge suppression flags indicating whether the left or right road edge intersects another ribbon.
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSuppression {
    pub suppress_left: bool,
    pub suppress_right: bool,
}

/// Computes edge suppression masks preventing outer white lines and curbs from drawing across open junction throats.
pub fn compute_segment_edge_suppressions(
    samples: &[SplineSample],
    closed: bool,
    other_segments: &[&RoadSegment],
    other_spline: Option<&TrackSpline>,
) -> Vec<EdgeSuppression> {
    let n = samples.len();
    if n < 2 {
        return Vec::new();
    }
    let seg_count = if closed { n } else { n - 1 };
    let mut suppressions = Vec::with_capacity(seg_count);

    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];

        let hw0 = s0.width * 0.5;
        let hw1 = s1.width * 0.5;

        let left0 = s0.point + s0.normal * hw0;
        let left1 = s1.point + s1.normal * hw1;
        let right0 = s0.point - s0.normal * hw0;
        let right1 = s1.point - s1.normal * hw1;

        let left_mid = (left0 + left1) * 0.5;
        let right_mid = (right0 + right1) * 0.5;
        let elev = (s0.elevation + s1.elevation) * 0.5;

        let is_in_ribbon = |pt: Vec2| -> bool {
            if let Some(sp) = other_spline {
                if sp.samples.len() >= 2 {
                    let proj = sp.project_point(pt);
                    if (elev - proj.elevation).abs() < 1.5 {
                        let half_w = proj.track_width * 0.5;
                        if proj.lateral_offset.abs() < (half_w - 0.25) {
                            return true;
                        }
                    }
                }
            }
            for o_seg in other_segments {
                if o_seg.samples.len() >= 2 {
                    let proj = o_seg.project_point(pt);
                    if (elev - proj.elevation).abs() < 1.5 {
                        let half_w = proj.track_width * 0.5;
                        if proj.lateral_offset.abs() < (half_w - 0.25) {
                            return true;
                        }
                    }
                }
            }
            false
        };

        suppressions.push(EdgeSuppression {
            suppress_left: is_in_ribbon(left_mid),
            suppress_right: is_in_ribbon(right_mid),
        });
    }

    suppressions
}

/// Synchronized untangled boundaries tuple: (left_road, right_road, left_curb, right_curb).
pub type UntangledBoundaryTuple = (Vec<Vec2>, Vec<Vec2>, Vec<Vec2>, Vec<Vec2>);

/// Borrowed slices of untangled boundaries.
pub type UntangledBoundarySlices<'a> = (&'a [Vec2], &'a [Vec2], &'a [Vec2], &'a [Vec2]);

/// Cached render data for a single network branch segment (Spec 084).
#[derive(Debug, Clone)]
pub struct CachedSegmentRenderData {
    pub id: arcade_race_core::track::network::SegmentId,
    pub spline: TrackSpline,
    pub boundaries: UntangledBoundaryTuple,
    pub suppressions: Vec<EdgeSuppression>,
}

/// Precomputed track rendering cache retaining untangled boundaries, branch splines,
/// and junction edge suppressions across frames (Spec 084).
#[derive(Debug, Clone)]
pub struct TrackRenderCache {
    pub track_ptr: usize,
    pub sample_count: usize,
    pub total_length_bits: u32,
    pub segments_len: usize,
    pub junctions_len: usize,
    pub has_pit_lane: bool,
    pub main_boundaries: UntangledBoundaryTuple,
    pub main_suppressions: Vec<EdgeSuppression>,
    pub pit_lane_boundaries: Option<UntangledBoundaryTuple>,
    pub branch_segments: Vec<CachedSegmentRenderData>,
}

impl TrackRenderCache {
    pub fn new(track: &Track) -> Self {
        let main_boundaries = track.spline.untangled_boundaries(1.35);
        let mut main_suppressions = Vec::new();
        let mut branch_segments = Vec::new();

        if let Some(ref net) = track.network {
            let branch_segs: Vec<&RoadSegment> = net.segments.iter().filter(|s| s.id.0 != 0).collect();
            main_suppressions = compute_segment_edge_suppressions(
                &track.spline.samples,
                track.spline.closed,
                &branch_segs,
                None,
            );

            for seg in &net.segments {
                if seg.id.0 != 0 && seg.samples.len() >= 2 {
                    let other_branches: Vec<&RoadSegment> =
                        net.segments.iter().filter(|s| s.id != seg.id).collect();
                    let branch_supp = compute_segment_edge_suppressions(
                        &seg.samples,
                        false,
                        &other_branches,
                        Some(&track.spline),
                    );
                    let seg_spline = seg.to_spline();
                    let branch_bnd = seg_spline.untangled_boundaries(1.35);
                    branch_segments.push(CachedSegmentRenderData {
                        id: seg.id,
                        spline: seg_spline,
                        boundaries: branch_bnd,
                        suppressions: branch_supp,
                    });
                }
            }
        }

        let pit_lane_boundaries = track.pit_lane.as_ref().map(|lane| {
            lane.spline.untangled_boundaries(1.35)
        });

        Self {
            track_ptr: track as *const Track as usize,
            sample_count: track.spline.samples.len(),
            total_length_bits: track.spline.total_length.to_bits(),
            segments_len: track.network.as_ref().map_or(0, |n| n.segments.len()),
            junctions_len: track.network.as_ref().map_or(0, |n| n.junctions.len()),
            has_pit_lane: track.pit_lane.is_some(),
            main_boundaries,
            main_suppressions,
            pit_lane_boundaries,
            branch_segments,
        }
    }

    pub fn matches(&self, track: &Track) -> bool {
        self.track_ptr == (track as *const Track as usize)
            && self.sample_count == track.spline.samples.len()
            && self.total_length_bits == track.spline.total_length.to_bits()
            && self.segments_len == track.network.as_ref().map_or(0, |n| n.segments.len())
            && self.junctions_len == track.network.as_ref().map_or(0, |n| n.junctions.len())
            && self.has_pit_lane == track.pit_lane.is_some()
    }
}

static TRACK_RENDER_CACHE: Mutex<Option<TrackRenderCache>> = Mutex::new(None);

/// Clears the global track render cache, forcing boundary and junction suppression recomputation on next render.
pub fn clear_track_render_cache() {
    if let Ok(mut lock) = TRACK_RENDER_CACHE.lock() {
        *lock = None;
    }
}

/// Accesses the global track render cache with automatic creation and validation.
pub fn with_track_render_cache<R>(track: &Track, f: impl FnOnce(&TrackRenderCache) -> R) -> R {
    let mut lock = TRACK_RENDER_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let needs_update = match lock.as_ref() {
        Some(c) => !c.matches(track),
        None => true,
    };
    if needs_update {
        *lock = Some(TrackRenderCache::new(track));
    }
    f(lock.as_ref().unwrap())
}

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
        if self.vertices.len() + 4 > 2000 {
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
        if self.vertices.len() + 3 > 2000 {
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

    // 2b. & 3. Render pit ribbon, segment runoff corridors, ground curbs and ground surface quads
    with_track_render_cache(track, |cache| {
        if let Some(pit_lane) = &track.pit_lane {
            let pit_bnd = cache.pit_lane_boundaries.as_ref().map(|b| (&b.0[..], &b.1[..], &b.2[..], &b.3[..]));
            render_surface_pass_filtered(&pit_lane.spline, false, view_bounds, None, pit_bnd);
        }

        let main_bnd = (&cache.main_boundaries.0[..], &cache.main_boundaries.1[..], &cache.main_boundaries.2[..], &cache.main_boundaries.3[..]);
        let main_supp = if track.network.is_some() { Some(&cache.main_suppressions[..]) } else { None };

        render_runoff_pass_filtered(&track.spline, false, view_bounds, Some(main_bnd));
        render_curbs_pass_filtered(&track.spline, false, view_bounds, main_supp, Some(main_bnd));
        render_surface_pass_filtered(&track.spline, false, view_bounds, main_supp, Some(main_bnd));

        if track.network.is_some() {
            for branch in &cache.branch_segments {
                let b_bnd = (&branch.boundaries.0[..], &branch.boundaries.1[..], &branch.boundaries.2[..], &branch.boundaries.3[..]);
                render_runoff_pass_filtered(&branch.spline, false, view_bounds, Some(b_bnd));
                render_curbs_pass_filtered(&branch.spline, false, view_bounds, Some(&branch.suppressions), Some(b_bnd));
                render_surface_pass_filtered(&branch.spline, false, view_bounds, Some(&branch.suppressions), Some(b_bnd));
            }
        }
    });

    // 3b. Render network junctions (paved throat wedges, gore triangles, chevrons, nose attenuators)
    render_network_junctions_pass(track, false, view_bounds);

    // 3c. Render pit lane junctions (throat wedge, gore triangle, chevrons, attenuators, merge patch)
    if let Some(pit_lane) = &track.pit_lane {
        render_pit_lane_junctions_pass(track, pit_lane, view_bounds);
    }

    // 4. Render on-top surface zones (AboveTrack: water puddles, oil slicks, sand/grass/dirt overlays)
    render_surface_zones_layer(track, SurfaceLayer::AboveTrack);

    // 5. Render 2.5D jump ramps
    render_jump_ramps(track);

    // 6. Render starting grid slots
    render_starting_grid(track);

    // 6b. Render procedural pit lane stalls & speed limit gates (Spec 062/077)
    if let Some(pit_lane) = &track.pit_lane {
        render_pit_boxes(pit_lane, view_bounds);
    }

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
    let has_elevated = track.spline.samples.iter().any(|s| s.is_bridge)
        || track.network.as_ref().map_or(false, |net| {
            net.segments.iter().any(|seg| seg.id.0 != 0 && seg.samples.iter().any(|s| s.is_bridge || s.elevation >= 0.6))
        });
    if has_elevated {
        render_bridge_structure_pass(&track.spline, view_bounds);

        with_track_render_cache(track, |cache| {
            let main_bnd = (&cache.main_boundaries.0[..], &cache.main_boundaries.1[..], &cache.main_boundaries.2[..], &cache.main_boundaries.3[..]);
            let main_supp = if track.network.is_some() { Some(&cache.main_suppressions[..]) } else { None };

            render_runoff_pass_filtered(&track.spline, true, view_bounds, Some(main_bnd));
            render_curbs_pass_filtered(&track.spline, true, view_bounds, main_supp, Some(main_bnd));
            render_surface_pass_filtered(&track.spline, true, view_bounds, main_supp, Some(main_bnd));

            if track.network.is_some() {
                for branch in &cache.branch_segments {
                    let b_bnd = (&branch.boundaries.0[..], &branch.boundaries.1[..], &branch.boundaries.2[..], &branch.boundaries.3[..]);
                    render_runoff_pass_filtered(&branch.spline, true, view_bounds, Some(b_bnd));
                    render_curbs_pass_filtered(&branch.spline, true, view_bounds, Some(&branch.suppressions), Some(b_bnd));
                    render_surface_pass_filtered(&branch.spline, true, view_bounds, Some(&branch.suppressions), Some(b_bnd));
                }
            }
        });

        render_network_junctions_pass(track, true, view_bounds);
    }
}

/// Helper returning fill and border colors for a given surface type.
pub fn get_surface_zone_colors(surface: SurfaceType) -> (Color, Option<Color>) {
    match surface {
        SurfaceType::PackedSand => (Palette::SAND, Some(Palette::SAND_DARK)),
        SurfaceType::DeepSand => (Palette::SAND_DARK, Some(Color::new(0.68, 0.54, 0.32, 1.0))),
        SurfaceType::Dirt => (Palette::DIRT, Some(Palette::DIRT_DARK)),
        SurfaceType::Water => (Palette::WATER, Some(Palette::WATER_BORDER)),
        SurfaceType::Asphalt => (Palette::RUNOFF_ASPHALT, Some(Palette::WHITE_LINE)),
        SurfaceType::Grass => (Palette::GRASS_DARK, None),
        SurfaceType::Curb => (Palette::CURB_RED, None),
        SurfaceType::SheetIce => (Color::new(0.85, 0.92, 0.98, 0.8), None),
        SurfaceType::Oil => (Color::new(0.12, 0.12, 0.15, 0.85), None),
        SurfaceType::MudTrack => (Palette::MUD, Some(Palette::MUD_DARK)),
        SurfaceType::DeepMud => (Palette::MUD_DARK, Some(Color::new(0.25, 0.16, 0.08, 1.0))),
        SurfaceType::PackedSnow => (Palette::SNOW, Some(Palette::SNOW_EDGE)),
        SurfaceType::DeepSnow => (Color::new(0.92, 0.94, 0.98, 1.0), Some(Palette::SNOW_EDGE)),
        SurfaceType::Gravel => (Palette::GRAVEL, Some(Palette::GRAVEL_DARK)),
        SurfaceType::Concrete => (Palette::CONCRETE, Some(Palette::CONCRETE_DARK)),
    }
}

/// Helper returning the background/backdrop clear color for a given track off-track default surface.
pub fn get_track_backdrop_color(surface: SurfaceType) -> Color {
    match surface {
        SurfaceType::Grass => Palette::BACKDROP_GRASS,
        SurfaceType::Dirt => Palette::BACKDROP_DIRT,
        SurfaceType::PackedSand | SurfaceType::DeepSand => Palette::BACKDROP_SAND,
        SurfaceType::Asphalt => Palette::BACKDROP_ASPHALT,
        SurfaceType::MudTrack | SurfaceType::DeepMud => Palette::BACKDROP_MUD,
        SurfaceType::PackedSnow | SurfaceType::DeepSnow => Palette::BACKDROP_SNOW,
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
        SurfaceType::PackedSand => (
            Palette::SAND,
            Some(Palette::SAND_DARK),
            Color::new(0.72, 0.58, 0.38, 0.85),
        ),
        SurfaceType::DeepSand => (
            Palette::SAND_DARK,
            Some(Color::new(0.68, 0.54, 0.32, 1.0)),
            Color::new(0.65, 0.52, 0.34, 0.85),
        ),
        SurfaceType::Grass => (
            Palette::GRASS_DARK,
            Some(Palette::GRASS),
            Color::new(0.38, 0.65, 0.32, 0.85),
        ),
        SurfaceType::SheetIce => (
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
        SurfaceType::MudTrack => (
            Palette::MUD,
            Some(Palette::MUD_DARK),
            Color::new(0.42, 0.30, 0.18, 0.85),
        ),
        SurfaceType::DeepMud => (
            Palette::MUD_DARK,
            Some(Color::new(0.25, 0.16, 0.08, 1.0)),
            Color::new(0.35, 0.22, 0.12, 0.85),
        ),
        SurfaceType::PackedSnow => (
            Palette::SNOW,
            Some(Palette::SNOW_EDGE),
            Color::new(0.80, 0.85, 0.92, 0.85),
        ),
        SurfaceType::DeepSnow => (
            Color::new(0.92, 0.94, 0.98, 1.0),
            Some(Palette::SNOW_EDGE),
            Color::new(0.75, 0.80, 0.88, 0.85),
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
#[allow(dead_code)]
fn render_runoff_pass(spline: &TrackSpline, elevated: bool, view_bounds: Option<(Vec2, Vec2)>) {
    render_runoff_pass_filtered(spline, elevated, view_bounds, None);
}

/// Draws track runoff ribbon quads with optional precomputed untangled boundaries (Spec 084).
fn render_runoff_pass_filtered(
    spline: &TrackSpline,
    elevated: bool,
    view_bounds: Option<(Vec2, Vec2)>,
    cached_boundaries: Option<UntangledBoundarySlices>,
) {
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

    let fallback;
    let (untangled_road_left, untangled_road_right, untangled_curb_left, untangled_curb_right) =
        if let Some(b) = cached_boundaries {
            b
        } else {
            fallback = spline.untangled_boundaries(curb_extra_width);
            (&fallback.0[..], &fallback.1[..], &fallback.2[..], &fallback.3[..])
        };

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
        if let Some(runoff_surf) = s0.left_runoff_surface {
            let hw0 = s0.width * 0.5;
            let hw1 = s1.width * 0.5;
            let curb_w0 = if s0.left_curb { curb_extra_width } else { 0.0 };
            let curb_w1 = if s1.left_curb { curb_extra_width } else { 0.0 };

            let wall_dist0 = s0.left_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);
            let wall_dist1 = s1.left_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);

            if wall_dist0 > curb_w0 && wall_dist1 > curb_w1 {
                let p0_inner = if s0.left_curb { untangled_curb_left[i] } else { untangled_road_left[i] };
                let p1_inner = if s1.left_curb { untangled_curb_left[(i + 1) % n] } else { untangled_road_left[(i + 1) % n] };
                let p0_outer = s0.point + s0.normal * (hw0 + wall_dist0);
                let p1_outer = s1.point + s1.normal * (hw1 + wall_dist1);

                if is_quad_valid(p0_inner, p1_inner, p1_outer, p0_outer) {
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
        if let Some(runoff_surf) = s0.right_runoff_surface {
            let hw0 = s0.width * 0.5;
            let hw1 = s1.width * 0.5;
            let curb_w0 = if s0.right_curb { curb_extra_width } else { 0.0 };
            let curb_w1 = if s1.right_curb { curb_extra_width } else { 0.0 };

            let wall_dist0 = s0.right_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);
            let wall_dist1 = s1.right_wall_distance.unwrap_or(TrackSpline::DEFAULT_WALL_DISTANCE);

            if wall_dist0 > curb_w0 && wall_dist1 > curb_w1 {
                let p0_inner = if s0.right_curb { untangled_curb_right[i] } else { untangled_road_right[i] };
                let p1_inner = if s1.right_curb { untangled_curb_right[(i + 1) % n] } else { untangled_road_right[(i + 1) % n] };
                let p0_outer = s0.point - s0.normal * (hw0 + wall_dist0);
                let p1_outer = s1.point - s1.normal * (hw1 + wall_dist1);

                if is_quad_valid(p0_inner, p1_inner, p1_outer, p0_outer) {

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

/// Draws curb rumble strips for either ground or elevated bridge segments with optional edge suppression and precomputed boundaries (Spec 084).
fn render_curbs_pass_filtered(
    spline: &TrackSpline,
    elevated: bool,
    view_bounds: Option<(Vec2, Vec2)>,
    suppressions: Option<&[EdgeSuppression]>,
    cached_boundaries: Option<UntangledBoundarySlices>,
) {
    let samples = &spline.samples;
    let n = samples.len();
    if n < 2 {
        return;
    }
    let curb_extra_width = 1.35;
    let seg_count = if spline.closed { n } else { n.saturating_sub(1) };

    let (curb_tex, curb_tile_scale, quality) = get_curb_material_info();
    let mut curb_builder = BatchMeshBuilder::new(curb_tex);

    let fallback;
    let (untangled_road_left, untangled_road_right, untangled_curb_left, untangled_curb_right) =
        if let Some(b) = cached_boundaries {
            b
        } else {
            fallback = spline.untangled_boundaries(curb_extra_width);
            (&fallback.0[..], &fallback.1[..], &fallback.2[..], &fallback.3[..])
        };

    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];
        let is_seg_elevated = s0.is_bridge && s1.is_bridge;
        if is_seg_elevated != elevated || !is_segment_in_view(s0, s1, view_bounds) {
            continue;
        }

        let supp_l = suppressions.map_or(false, |s| s.get(i).map_or(false, |m| m.suppress_left));
        let supp_r = suppressions.map_or(false, |s| s.get(i).map_or(false, |m| m.suppress_right));

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
        if !supp_l && (s0.left_curb || s1.left_curb) {
            let p0_inner = untangled_road_left[i];
            let p1_inner = untangled_road_left[(i + 1) % n];
            let p0_outer = untangled_curb_left[i];
            let p1_outer = untangled_curb_left[(i + 1) % n];

            if is_quad_valid(p0_inner, p1_inner, p1_outer, p0_outer) {
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

        // Right curb
        if !supp_r && (s0.right_curb || s1.right_curb) {
            let p0_inner = untangled_road_right[i];
            let p1_inner = untangled_road_right[(i + 1) % n];
            let p0_outer = untangled_curb_right[i];
            let p1_outer = untangled_curb_right[(i + 1) % n];

            if is_quad_valid(p0_inner, p1_inner, p1_outer, p0_outer) {
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
    }

    curb_builder.flush();
}

#[allow(dead_code)]
fn render_curbs_pass(spline: &TrackSpline, elevated: bool, view_bounds: Option<(Vec2, Vec2)>) {
    render_curbs_pass_filtered(spline, elevated, view_bounds, None, None);
}

/// Draws track surface quads (asphalt/dirt) for either ground or elevated bridge segments with optional edge suppression and precomputed boundaries (Spec 084).
fn render_surface_pass_filtered(
    spline: &TrackSpline,
    elevated: bool,
    view_bounds: Option<(Vec2, Vec2)>,
    suppressions: Option<&[EdgeSuppression]>,
    cached_boundaries: Option<UntangledBoundarySlices>,
) {
    let samples = &spline.samples;
    let n = samples.len();
    if n < 2 {
        return;
    }
    let seg_count = if spline.closed { n } else { n.saturating_sub(1) };

    let quality = get_surface_texture_quality();
    let mut surface_builders: HashMap<SurfaceType, BatchMeshBuilder> = HashMap::new();
    let mut lines_to_draw: Vec<(Vec2, Vec2, f32, Color)> = Vec::with_capacity(seg_count * 2);

    let fallback;
    let (untangled_left, untangled_right, untangled_curb_left, untangled_curb_right) =
        if let Some(b) = cached_boundaries {
            b
        } else {
            fallback = spline.untangled_boundaries(1.35);
            (&fallback.0[..], &fallback.1[..], &fallback.2[..], &fallback.3[..])
        };

    for i in 0..seg_count {
        let s0 = &samples[i];
        let s1 = &samples[(i + 1) % n];
        let is_seg_elevated = s0.is_bridge && s1.is_bridge;
        if is_seg_elevated != elevated || !is_segment_in_view(s0, s1, view_bounds) {
            continue;
        }

        let supp_l = suppressions.map_or(false, |s| s.get(i).map_or(false, |m| m.suppress_left));
        let supp_r = suppressions.map_or(false, |s| s.get(i).map_or(false, |m| m.suppress_right));

        let hw0 = s0.width * 0.5;
        let hw1 = s1.width * 0.5;

        let left0 = untangled_left[i];
        let right0 = untangled_right[i];
        let left1 = untangled_left[(i + 1) % n];
        let right1 = untangled_right[(i + 1) % n];

        let avg_bank = (s0.bank_angle + s1.bank_angle) * 0.5;
        let is_banked = avg_bank.abs() > 0.8;

        // Ground shading: banked curve rim shadows and raised-ground embankment shade (Spec 055 §5).
        if !elevated && !s0.is_bridge && !s1.is_bridge {
            let max_embankment_width = 3.0;
            let mut l_shade_w0 = if s0.elevation > 0.8 { ((s0.elevation - 0.8) * 0.40).min(max_embankment_width) } else { 0.0 };
            let mut l_shade_w1 = if s1.elevation > 0.8 { ((s1.elevation - 0.8) * 0.40).min(max_embankment_width) } else { 0.0 };
            let mut r_shade_w0 = if s0.elevation > 0.8 { ((s0.elevation - 0.8) * 0.40).min(max_embankment_width) } else { 0.0 };
            let mut r_shade_w1 = if s1.elevation > 0.8 { ((s1.elevation - 0.8) * 0.40).min(max_embankment_width) } else { 0.0 };

            if is_banked {
                let rim_w = avg_bank.abs().min(25.0) * 0.03 + 0.30;
                if avg_bank > 0.0 {
                    r_shade_w0 = r_shade_w0.max(rim_w);
                    r_shade_w1 = r_shade_w1.max(rim_w);
                } else {
                    l_shade_w0 = l_shade_w0.max(rim_w);
                    l_shade_w1 = l_shade_w1.max(rim_w);
                }
            }

            let l0_base = if s0.left_curb { untangled_curb_left[i] } else { left0 };
            let l1_base = if s1.left_curb { untangled_curb_left[(i + 1) % n] } else { left1 };
            let r0_base = if s0.right_curb { untangled_curb_right[i] } else { right0 };
            let r1_base = if s1.right_curb { untangled_curb_right[(i + 1) % n] } else { right1 };

            if l_shade_w0 > 0.001 || l_shade_w1 > 0.001 {
                let l0_outer = l0_base + s0.normal * l_shade_w0;
                let l1_outer = l1_base + s1.normal * l_shade_w1;
                draw_quad(l0_base, l1_base, l1_outer, l0_outer, Palette::SHADOW);
            }
            if r_shade_w0 > 0.001 || r_shade_w1 > 0.001 {
                let r0_outer = r0_base - s0.normal * r_shade_w0;
                let r1_outer = r1_base - s1.normal * r_shade_w1;
                draw_quad(r0_base, r1_base, r1_outer, r0_outer, Palette::SHADOW);
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

                    if !supp_l {
                        lines_to_draw.push((left0, left1, 0.32, Palette::DIRT_EDGE));
                    }
                    if !supp_r {
                        lines_to_draw.push((right0, right1, 0.32, Palette::DIRT_EDGE));
                    }
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
                    if !supp_l {
                        lines_to_draw.push((left0, left1, 0.32, Palette::DIRT_EDGE));
                    }
                    if !supp_r {
                        lines_to_draw.push((right0, right1, 0.32, Palette::DIRT_EDGE));
                    }

                    let groove_l0 = s0.point + s0.normal * (hw0 * 0.45);
                    let groove_l1 = s1.point + s1.normal * (hw1 * 0.45);
                    let groove_r0 = s0.point - s0.normal * (hw0 * 0.45);
                    let groove_r1 = s1.point - s1.normal * (hw1 * 0.45);
                    lines_to_draw.push((groove_l0, groove_l1, 0.22, Palette::DIRT_DARK));
                    lines_to_draw.push((groove_r0, groove_r1, 0.22, Palette::DIRT_DARK));
                }
            }
            SurfaceType::PackedSand | SurfaceType::DeepSand => {
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
                } else if surf == SurfaceType::DeepSand {
                    (Palette::SAND_DARK, Palette::SAND_DARK, Palette::SAND_DARK, Palette::SAND_DARK)
                } else {
                    (Palette::SAND, Palette::SAND, Palette::SAND, Palette::SAND)
                };
                builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.32, Palette::SAND_DARK));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.32, Palette::SAND_DARK));
                }
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
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.32, Palette::GRASS));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.32, Palette::GRASS));
                }
            }
            SurfaceType::SheetIce => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { Color::new(1.0, 1.0, 1.0, 0.95) } else { Color::new(0.85, 0.92, 0.98, 0.95) };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.32, Color::new(0.65, 0.82, 0.95, 0.8)));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.32, Color::new(0.65, 0.82, 0.95, 0.8)));
                }
            }
            SurfaceType::Water => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { WHITE } else { Palette::WATER };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.32, Palette::WATER_BORDER));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.32, Palette::WATER_BORDER));
                }
            }
            SurfaceType::Oil => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { WHITE } else { Color::new(0.12, 0.12, 0.15, 0.95) };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.32, Color::new(0.35, 0.25, 0.40, 0.85)));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.32, Color::new(0.35, 0.25, 0.40, 0.85)));
                }
            }
            SurfaceType::Curb => {
                let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                let col = if has_tex { WHITE } else { Palette::CURB_RED };
                builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
            }
            SurfaceType::MudTrack | SurfaceType::DeepMud => {
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
                } else if surf == SurfaceType::DeepMud {
                    (Palette::MUD_DARK, Palette::MUD_DARK, Palette::MUD_DARK, Palette::MUD_DARK)
                } else {
                    (Palette::MUD, Palette::MUD, Palette::MUD, Palette::MUD)
                };
                builder.push_quad(left0, uv0, c0, left1, uv1, c1, right1, uv2, c2, right0, uv3, c3);
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.34, Palette::MUD_DARK));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.34, Palette::MUD_DARK));
                }

                let rut_l0 = s0.point + s0.normal * (hw0 * 0.40);
                let rut_l1 = s1.point + s1.normal * (hw1 * 0.40);
                let rut_r0 = s0.point - s0.normal * (hw0 * 0.40);
                let rut_r1 = s1.point - s1.normal * (hw1 * 0.40);
                lines_to_draw.push((rut_l0, rut_l1, 0.26, Palette::MUD_DARK));
                lines_to_draw.push((rut_r0, rut_r1, 0.26, Palette::MUD_DARK));
            }
            SurfaceType::PackedSnow | SurfaceType::DeepSnow => {
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
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.32, Palette::SNOW_EDGE));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.32, Palette::SNOW_EDGE));
                }

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
                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.32, Palette::GRAVEL_EDGE));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.32, Palette::GRAVEL_EDGE));
                }

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

                if !supp_l {
                    lines_to_draw.push((left0, left1, 0.28, Palette::WHITE_LINE));
                }
                if !supp_r {
                    lines_to_draw.push((right0, right1, 0.28, Palette::WHITE_LINE));
                }

                if is_banked {
                    lines_to_draw.push((mid_l0, mid_l1, 0.12, Color::new(0.40, 0.42, 0.46, 0.4)));
                    lines_to_draw.push((mid_r0, mid_r1, 0.12, Color::new(0.40, 0.42, 0.46, 0.4)));
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

                    if !supp_l {
                        lines_to_draw.push((left0, left1, 0.28, Palette::WHITE_LINE));
                    }
                    if !supp_r {
                        lines_to_draw.push((right0, right1, 0.28, Palette::WHITE_LINE));
                    }
                    lines_to_draw.push((mid_l0, mid_l1, 0.12, Color::new(0.50, 0.52, 0.55, 0.45)));
                    lines_to_draw.push((mid_r0, mid_r1, 0.12, Color::new(0.50, 0.52, 0.55, 0.45)));
                } else {
                    let uv0 = macroquad::prelude::Vec2::new(0.0, v0);
                    let uv1 = macroquad::prelude::Vec2::new(0.0, v1);
                    let uv2 = macroquad::prelude::Vec2::new(1.0, v1);
                    let uv3 = macroquad::prelude::Vec2::new(1.0, v0);
                    let col = if has_tex { WHITE } else { Palette::CONCRETE };

                    builder.push_quad(left0, uv0, col, left1, uv1, col, right1, uv2, col, right0, uv3, col);
                    if !supp_l {
                        lines_to_draw.push((left0, left1, 0.28, Palette::WHITE_LINE));
                    }
                    if !supp_r {
                        lines_to_draw.push((right0, right1, 0.28, Palette::WHITE_LINE));
                    }

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
            if (p1 - p0).length_squared() > 1e-4 {
                draw_line(p0.x, p0.y, p1.x, p1.y, th, col);
            }
        }
    }));
}

/// Draws track surface quads for either ground or elevated bridge segments.
#[allow(dead_code)]
fn render_surface_pass(spline: &TrackSpline, elevated: bool, view_bounds: Option<(Vec2, Vec2)>) {
    render_surface_pass_filtered(spline, elevated, view_bounds, None, None);
}

/// Renders network junction geometry: paved throat wedges, gore triangles, painted chevrons, and nose crash cushions.
pub fn render_network_junctions_pass(
    track: &Track,
    elevated: bool,
    _view_bounds: Option<(Vec2, Vec2)>,
) {
    let Some(ref net) = track.network else { return; };

    for junction in &net.junctions {
        match &junction.kind {
            JunctionKind::Split {
                ingress_socket,
                egress_sockets,
                gore_config,
            } => {
                let is_junc_elev = ingress_socket.elevation >= 0.6;
                if is_junc_elev != elevated {
                    continue;
                }

                // 1. Paved Bifurcation Throat Wedge
                if egress_sockets.len() >= 2 {
                    let in_l = ingress_socket.left_edge();
                    let in_r = ingress_socket.right_edge();
                    let e0_l = egress_sockets[0].left_edge();
                    let e0_r = egress_sockets[0].right_edge();
                    let e1_l = egress_sockets[1].left_edge();
                    let e1_r = egress_sockets[1].right_edge();

                    draw_triangle(
                        macroquad::prelude::Vec2::new(in_l.x, in_l.y),
                        macroquad::prelude::Vec2::new(e0_l.x, e0_l.y),
                        macroquad::prelude::Vec2::new(e1_l.x, e1_l.y),
                        Palette::ASPHALT,
                    );
                    draw_triangle(
                        macroquad::prelude::Vec2::new(in_r.x, in_r.y),
                        macroquad::prelude::Vec2::new(e0_r.x, e0_r.y),
                        macroquad::prelude::Vec2::new(e1_r.x, e1_r.y),
                        Palette::ASPHALT,
                    );
                    draw_quad(in_l, in_r, e0_r, e0_l, Palette::ASPHALT);
                    draw_quad(in_l, in_r, e1_r, e1_l, Palette::ASPHALT);
                }

                // 2. Gore Triangle & Markings
                if let Some(gore) = gore_config {
                    let p_apex = gore.apex_point;
                    let v0 = egress_sockets.get(0).map_or(ingress_socket.tangent, |s| s.tangent);
                    let v1 = egress_sockets.get(1).map_or(ingress_socket.tangent, |s| s.tangent);

                    let gore_len = gore.gore_length.max(6.0);
                    let p0 = p_apex + v0 * gore_len;
                    let p1 = p_apex + v1 * gore_len;

                    // A. Paved asphalt gore triangle
                    draw_triangle(
                        macroquad::prelude::Vec2::new(p_apex.x, p_apex.y),
                        macroquad::prelude::Vec2::new(p0.x, p0.y),
                        macroquad::prelude::Vec2::new(p1.x, p1.y),
                        Palette::RUNOFF_ASPHALT,
                    );

                    // B. White gore perimeter lines
                    draw_line(p_apex.x, p_apex.y, p0.x, p0.y, 0.35, Palette::WHITE_LINE);
                    draw_line(p_apex.x, p_apex.y, p1.x, p1.y, 0.35, Palette::WHITE_LINE);
                    draw_line(p0.x, p0.y, p1.x, p1.y, 0.35, Palette::WHITE_LINE);

                    // C. Painted Chevrons (V-stripes) pointing toward incoming traffic
                    if gore.has_chevrons {
                        let num_chevrons = (gore_len / 2.8).floor() as usize;
                        let bisect = (v0 + v1).normalize_or_zero();
                        for step in 1..=num_chevrons {
                            let d = step as f32 * 2.8;
                            if d >= gore_len - 0.5 { break; }
                            let a = p_apex + v0 * d;
                            let b = p_apex + v1 * d;
                            let apex_chevron = p_apex + bisect * (d - 1.2).max(0.2);
                            draw_line(apex_chevron.x, apex_chevron.y, a.x, a.y, 0.32, Palette::WHITE_LINE);
                            draw_line(apex_chevron.x, apex_chevron.y, b.x, b.y, 0.32, Palette::WHITE_LINE);
                        }
                    }

                    // D. Attenuator Nose Barrier & Hazard Cap
                    let w_len = (gore.nose_barrier.segment.end - gore.nose_barrier.segment.start).length();
                    if w_len > 0.05 {
                        barrier::render_wall_shadow(&gore.nose_barrier);
                        barrier::render_wall_body(&gore.nose_barrier);
                    }
                    // High-visibility impact attenuator nose cap at apex
                    draw_circle(p_apex.x, p_apex.y, 0.9, Palette::CURB_RED);
                    draw_circle(p_apex.x, p_apex.y, 0.6, Palette::CURB_WHITE);
                    draw_circle(p_apex.x, p_apex.y, 0.3, Palette::CURB_RED);
                }
            }
            JunctionKind::Merge {
                ingress_sockets,
                egress_socket,
                merge_config: _,
            } => {
                let is_junc_elev = egress_socket.elevation >= 0.6;
                if is_junc_elev != elevated {
                    continue;
                }

                // Smooth asphalt merge taper patch
                if !ingress_sockets.is_empty() {
                    let eg_l = egress_socket.left_edge();
                    let eg_r = egress_socket.right_edge();
                    for in_sock in ingress_sockets {
                        let in_l = in_sock.left_edge();
                        let in_r = in_sock.right_edge();
                        draw_quad(in_l, in_r, eg_r, eg_l, Palette::ASPHALT);
                    }
                }
            }
            JunctionKind::Terminal { .. } => {}
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
    let gate_dir = finish_cp.gate.end - finish_cp.gate.start;
    let gate_w = gate_dir.length();
    if gate_w < 1.0 {
        return;
    }
    let norm = gate_dir / gate_w;

    // The timing gate extends past the track edges; paint the checkers across the track width only,
    // centered where the gate crosses the centerline.
    let gate_mid = (finish_cp.gate.start + finish_cp.gate.end) * 0.5;
    let (center, total_w) = if track.spline.samples.len() >= 2 {
        let proj = track.spline.project_point(gate_mid);
        let along = (proj.closest_point - finish_cp.gate.start).dot(norm).clamp(0.0, gate_w);
        (finish_cp.gate.start + norm * along, proj.track_width.min(gate_w))
    } else {
        (gate_mid, gate_w)
    };
    let start_pt = center - norm * (total_w * 0.5);
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

/// Renders pit lane junctions: paved entrance wedge, gore triangle, chevrons, impact attenuator, and exit merge taper.
pub fn render_pit_lane_junctions_pass(
    track: &Track,
    pit_lane: &PitLane,
    view_bounds: Option<(Vec2, Vec2)>,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if pit_lane.spline.samples.len() < 4 || track.spline.samples.len() < 4 {
            return;
        }

        let is_in_view = |pos: Vec2, radius: f32| -> bool {
            if let Some((min, max)) = view_bounds {
                !(pos.x + radius < min.x || pos.x - radius > max.x || pos.y + radius < min.y || pos.y - radius > max.y)
            } else {
                true
            }
        };

        let n_pit = pit_lane.spline.samples.len();
        let pit_w = pit_lane.road_width;
        let pit_hw = pit_w * 0.5;

        // 1. Analyze entrance split geometry
        // Find where the pit lane branches from the main track
        let mut apex_sample_idx = None;
        let mut pit_side = -1.0f32;

        for (i, s) in pit_lane.spline.samples.iter().enumerate().take(n_pit / 2) {
            let proj = track.spline.project_point(s.point);
            let track_hw = proj.track_width * 0.5;
            let to_pit = s.point - proj.closest_point;
            let side = if to_pit.dot(proj.normal) >= 0.0 { 1.0f32 } else { -1.0f32 };
            pit_side = side;

            let track_edge = proj.closest_point + proj.normal * (side * track_hw);
            let pit_inner = s.point - proj.normal * (side * pit_hw);
            let gap = (pit_inner - track_edge).dot(proj.normal * side);

            if gap >= 1.2 {
                apex_sample_idx = Some(i);
                break;
            }
        }

        // 2. Render Entrance Throat Wedge & Gore Triangle
        if let Some(apex_idx) = apex_sample_idx {
            let s_apex = &pit_lane.spline.samples[apex_idx];
            let proj_apex = track.spline.project_point(s_apex.point);
            let track_hw_apex = proj_apex.track_width * 0.5;
            let track_edge_apex = proj_apex.closest_point + proj_apex.normal * (pit_side * track_hw_apex);
            let pit_inner_apex = s_apex.point - proj_apex.normal * (pit_side * pit_hw);

            let p_apex = (track_edge_apex + pit_inner_apex) * 0.5;

            if is_in_view(p_apex, 35.0) {
                // A. Paved entrance throat wedge between sample 0 and apex
                for i in 0..apex_idx {
                    let s0 = &pit_lane.spline.samples[i];
                    let s1 = &pit_lane.spline.samples[i + 1];
                    let p0 = track.spline.project_point(s0.point);
                    let p1 = track.spline.project_point(s1.point);

                    let te0 = p0.closest_point + p0.normal * (pit_side * p0.track_width * 0.5);
                    let te1 = p1.closest_point + p1.normal * (pit_side * p1.track_width * 0.5);
                    let pe0 = s0.point - p0.normal * (pit_side * pit_hw);
                    let pe1 = s1.point - p1.normal * (pit_side * pit_hw);

                    draw_quad(te0, te1, pe1, pe0, Palette::ASPHALT);
                }

                // B. Gore Triangle
                let s_start = &pit_lane.spline.samples[0];
                let p_start = track.spline.project_point(s_start.point);
                let te_start = p_start.closest_point + p_start.normal * (pit_side * p_start.track_width * 0.5);
                let pe_start = s_start.point - p_start.normal * (pit_side * pit_hw);

                let v_track = (track_edge_apex - te_start).normalize_or_zero();
                let v_pit = (pit_inner_apex - pe_start).normalize_or_zero();

                // Paved asphalt gore triangle
                draw_triangle(
                    macroquad::prelude::Vec2::new(p_apex.x, p_apex.y),
                    macroquad::prelude::Vec2::new(track_edge_apex.x, track_edge_apex.y),
                    macroquad::prelude::Vec2::new(pit_inner_apex.x, pit_inner_apex.y),
                    Palette::RUNOFF_ASPHALT,
                );

                // White perimeter border lines
                draw_line(p_apex.x, p_apex.y, te_start.x, te_start.y, 0.35, Palette::WHITE_LINE);
                draw_line(p_apex.x, p_apex.y, pe_start.x, pe_start.y, 0.35, Palette::WHITE_LINE);

                // Directional Chevron Markings (V-stripes pointing upstream toward traffic)
                let gore_len = (p_apex - te_start).length();
                if gore_len > 8.0 {
                    let num_chevrons = ((gore_len - 4.0) / 3.0).floor() as usize;
                    let bisect = -(v_track + v_pit).normalize_or_zero();
                    for step in 1..=num_chevrons {
                        let d = step as f32 * 3.0;
                        if d >= gore_len - 2.0 { break; }
                        let t_frac = d / gore_len;
                        let pt_t = te_start.lerp(track_edge_apex, t_frac);
                        let pt_p = pe_start.lerp(pit_inner_apex, t_frac);
                        let chevron_apex = (pt_t + pt_p) * 0.5 + bisect * 1.2;

                        draw_line(chevron_apex.x, chevron_apex.y, pt_t.x, pt_t.y, 0.30, Palette::WHITE_LINE);
                        draw_line(chevron_apex.x, chevron_apex.y, pt_p.x, pt_p.y, 0.30, Palette::WHITE_LINE);
                    }
                }

                // High-visibility impact attenuator nose cap at apex
                draw_circle(p_apex.x, p_apex.y, 1.1, Palette::CURB_RED);
                draw_circle(p_apex.x, p_apex.y, 0.75, Palette::CURB_WHITE);
                draw_circle(p_apex.x, p_apex.y, 0.4, Palette::CURB_RED);
            }
        }

        // 3. Render Exit Merge Taper
        // Find where the pit lane rejoins the main track (in the second half of samples)
        let mut merge_start_idx = None;
        for i in (n_pit / 2..n_pit).rev() {
            let s = &pit_lane.spline.samples[i];
            let proj = track.spline.project_point(s.point);
            let track_hw = proj.track_width * 0.5;
            let track_edge = proj.closest_point + proj.normal * (pit_side * track_hw);
            let pit_inner = s.point - proj.normal * (pit_side * pit_hw);
            let gap = (pit_inner - track_edge).dot(proj.normal * pit_side);

            if gap >= 1.2 {
                merge_start_idx = Some(i);
                break;
            }
        }

        if let Some(merge_idx) = merge_start_idx {
            for i in merge_idx..n_pit - 1 {
                let s0 = &pit_lane.spline.samples[i];
                let s1 = &pit_lane.spline.samples[i + 1];
                let p0 = track.spline.project_point(s0.point);
                let p1 = track.spline.project_point(s1.point);

                let te0 = p0.closest_point + p0.normal * (pit_side * p0.track_width * 0.5);
                let te1 = p1.closest_point + p1.normal * (pit_side * p1.track_width * 0.5);
                let pe0 = s0.point - p0.normal * (pit_side * pit_hw);
                let pe1 = s1.point - p1.normal * (pit_side * pit_hw);

                if is_in_view(pe0, 20.0) {
                    draw_quad(te0, te1, pe1, pe0, Palette::ASPHALT);
                    // Dashed merge guidance line
                    if i % 2 == 0 {
                        draw_line(pe0.x, pe0.y, pe1.x, pe1.y, 0.28, Palette::WHITE_LINE);
                    }
                }
            }
        }
    }));
}

/// Renders pit box team stalls and speed limit gates along the pit lane.
fn render_pit_boxes(pit_lane: &PitLane, view_bounds: Option<(Vec2, Vec2)>) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let is_in_view = |pos: Vec2, radius: f32| -> bool {
            if let Some((min, max)) = view_bounds {
                !(pos.x + radius < min.x || pos.x - radius > max.x || pos.y + radius < min.y || pos.y - radius > max.y)
            } else {
                true
            }
        };

        // Draw yellow speed limiter entry and exit lines
        let entry = &pit_lane.entry_gate;
        let entry_mid = (entry.start + entry.end) * 0.5;
        if is_in_view(entry_mid, 8.0) {
            draw_line(entry.start.x, entry.start.y, entry.end.x, entry.end.y, 0.30, Color::new(1.0, 0.82, 0.15, 0.85));
        }
        let exit = &pit_lane.exit_gate;
        let exit_mid = (exit.start + exit.end) * 0.5;
        if is_in_view(exit_mid, 8.0) {
            draw_line(exit.start.x, exit.start.y, exit.end.x, exit.end.y, 0.30, Color::new(1.0, 0.82, 0.15, 0.85));
        }

        // Draw team pit stalls
        for pit_box in &pit_lane.pit_boxes {
            if !is_in_view(pit_box.position, 6.0) {
                continue;
            }
            let pos = pit_box.position;
            let fwd = if pit_box.direction.length_squared() > 1e-4 {
                pit_box.direction.normalize()
            } else {
                Vec2::new(1.0, 0.0)
            };
            let right = Vec2::new(-fwd.y, fwd.x);

            let box_len = 4.8;
            let box_w = 2.4;

            let p_fl = pos + fwd * (box_len * 0.5) - right * (box_w * 0.5);
            let p_fr = pos + fwd * (box_len * 0.5) + right * (box_w * 0.5);
            let p_rl = pos - fwd * (box_len * 0.5) - right * (box_w * 0.5);
            let p_rr = pos - fwd * (box_len * 0.5) + right * (box_w * 0.5);

            // Team stall box outline and center stop line
            let line_thickness = 0.22;
            let box_col = Color::new(0.95, 0.95, 0.95, 0.75);
            let stop_col = Color::new(0.95, 0.25, 0.25, 0.70);
            draw_line(p_fl.x, p_fl.y, p_fr.x, p_fr.y, line_thickness, box_col);
            draw_line(p_fr.x, p_fr.y, p_rr.x, p_rr.y, line_thickness, box_col);
            draw_line(p_rr.x, p_rr.y, p_rl.x, p_rl.y, line_thickness, box_col);
            draw_line(p_rl.x, p_rl.y, p_fl.x, p_fl.y, line_thickness, box_col);

            // Center stop line
            let stop_l = pos - right * (box_w * 0.45);
            let stop_r = pos + right * (box_w * 0.45);
            draw_line(stop_l.x, stop_l.y, stop_r.x, stop_r.y, line_thickness * 1.5, stop_col);
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
            right_runoff_surface: Some(SurfaceType::DeepSand),
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
        
        let track = arcade_race_core::track::create_prototypical_track("gt", arcade_race_core::track::TrackShape::Oval, arcade_race_core::track::RaceDirection::Right);
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
            SurfaceType::PackedSand,
            SurfaceType::DeepSand,
        ];

        for &surf in &surfaces {
            let (fill, border) = get_surface_zone_colors(surf);
            render_surface_shape_textured(&aabb, surf, fill, border);
            render_surface_shape_textured(&circle, surf, fill, border);
            render_surface_shape_textured(&obox, surf, fill, border);
            render_surface_shape_textured(&poly, surf, fill, border);
        }
    }

    #[test]
    fn test_raised_ground_embankment_shade_pass() {
        let mut spline = TrackSpline::empty();
        spline.closed = true;

        // Create samples: flat, raised without curbs, raised with curbs, banked raised, and bridge
        let elevations = [0.0, 0.5, 1.2, 3.5, 6.0, 9.5, 0.0];
        let banks = [0.0, 0.0, 0.0, 15.0, -10.0, 0.0, 0.0];
        for (i, (&elev, &bank)) in elevations.iter().zip(banks.iter()).enumerate() {
            spline.samples.push(SplineSample {
                point: Vec2::new(i as f32 * 20.0, 0.0),
                tangent: Vec2::X,
                normal: Vec2::Y,
                distance: i as f32 * 20.0,
                width: 10.0,
                left_curb: i % 2 == 1,
                right_curb: i % 2 == 0,
                surface: SurfaceType::Asphalt,
                elevation: elev,
                bank_angle: bank,
                is_bridge: false,
                grade_slope: 0.0,
                vertical_curvature: 0.0,
                left_wall: false,
                right_wall: false,
                left_wall_distance: None,
                right_wall_distance: None,
                wall_type: None,
                left_runoff_surface: None,
                right_runoff_surface: None,
            });
        }

        // Must execute cleanly without panics for both ground and elevated passes
        render_surface_pass(&spline, false, None);
        render_surface_pass(&spline, true, None);
    }

    #[test]
    fn test_is_quad_valid_rejects_bowtie_and_accepts_apex_fan() {
        // Normal convex quad: inner from (0, 0) to (5, 0), outer from (0, 2) to (5, 2)
        assert!(is_quad_valid(
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(5.0, 2.0),
            Vec2::new(0.0, 2.0),
        ));

        // Triangle fan quad with collapsed inner edge: inner is single apex point (2, 0)
        assert!(is_quad_valid(
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(5.0, 2.0),
            Vec2::new(0.0, 2.0),
        ));

        // Degenerate zero-area quad: both inner and outer collapsed to single points
        assert!(!is_quad_valid(
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(2.0, 2.0),
        ));

        // Bowtie quad: radial lines cross each other
        // Radial 0 from (0, 0) to (5, 2), Radial 1 from (5, 0) to (0, 2) -> lines cross at (2.5, 1.0)
        assert!(!is_quad_valid(
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(0.0, 2.0),
            Vec2::new(5.0, 2.0),
        ));
    }

    #[test]
    fn test_pit_lane_and_pit_boxes_ground_rendering() {
        use arcade_race_core::track::geometry::PitBox;
        let mut track = arcade_race_core::track::create_prototypical_track("gt", arcade_race_core::track::TrackShape::Oval, arcade_race_core::track::RaceDirection::Right);
        track.spline = TrackSpline::empty();
        for i in 0..10 {
            track.spline.samples.push(SplineSample {
                point: Vec2::new(i as f32 * 10.0, 0.0),
                tangent: Vec2::X,
                normal: Vec2::Y,
                distance: i as f32 * 10.0,
                width: 10.0,
                left_curb: false,
                right_curb: false,
                surface: SurfaceType::Asphalt,
                elevation: 0.0,
                bank_angle: 0.0,
                is_bridge: false,
                grade_slope: 0.0,
                vertical_curvature: 0.0,
                left_wall: false,
                right_wall: false,
                left_wall_distance: None,
                right_wall_distance: None,
                wall_type: None,
                left_runoff_surface: None,
                right_runoff_surface: None,
            });
        }
        let mut pit_spline = TrackSpline::empty();
        for i in 0..8 {
            pit_spline.samples.push(SplineSample {
                point: Vec2::new(i as f32 * 10.0 + 5.0, 8.0),
                tangent: Vec2::X,
                normal: Vec2::Y,
                distance: i as f32 * 10.0,
                width: 6.0,
                left_curb: false,
                right_curb: false,
                surface: SurfaceType::Asphalt,
                elevation: 0.0,
                bank_angle: 0.0,
                is_bridge: false,
                grade_slope: 0.0,
                vertical_curvature: 0.0,
                left_wall: false,
                right_wall: false,
                left_wall_distance: None,
                right_wall_distance: None,
                wall_type: None,
                left_runoff_surface: None,
                right_runoff_surface: None,
            });
        }
        track.pit_lane = Some(PitLane {
            spline: pit_spline,
            road_width: 6.0,
            entry_gate: LineSegment::new(Vec2::new(5.0, 0.0), Vec2::new(5.0, 8.0)),
            exit_gate: LineSegment::new(Vec2::new(75.0, 8.0), Vec2::new(75.0, 0.0)),
            speed_limit: 16.67,
            pit_boxes: vec![
                PitBox {
                    position: Vec2::new(30.0, 8.0),
                    direction: Vec2::X,
                    stop_radius: 3.0,
                    elevation: 0.0,
                },
                PitBox {
                    position: Vec2::new(50.0, 8.0),
                    direction: Vec2::X,
                    stop_radius: 3.0,
                    elevation: 0.0,
                },
            ],
        });

        // Must execute cleanly without panicking
        render_ground_track_culled(&track, None);
        render_ground_track_culled(&track, Some((Vec2::new(0.0, -10.0), Vec2::new(100.0, 20.0))));
    }

    #[test]
    fn test_track_render_cache_hit_and_performance() {
        clear_track_render_cache();
        let track = arcade_race_core::track::create_prototypical_track(
            "gt",
            arcade_race_core::track::TrackShape::Oval,
            arcade_race_core::track::RaceDirection::Right,
        );

        // First render initializes the cache
        with_track_render_cache(&track, |cache| {
            assert!(cache.matches(&track));
            assert_eq!(cache.main_boundaries.0.len(), track.spline.samples.len());
        });

        // Verify cache hit and consistency across frames
        with_track_render_cache(&track, |cache| {
            assert!(cache.matches(&track));
        });

        // Run render passes to ensure zero panics and valid execution
        render_ground_track_culled(&track, None);
        render_elevated_track_culled(&track, None);
    }
}

