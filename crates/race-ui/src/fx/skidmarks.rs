use std::cell::RefCell;
use std::collections::HashMap;

use glam::Vec2;
use macroquad::color::Color;
use macroquad::models::{draw_mesh, Mesh, Vertex};
use wheelbase::surface::SurfaceType;

use crate::render::track::{is_render_context_active, CachedMesh};

use super::FxVehicle;

/// Minimum configurable skidmark ring buffer capacity (`32K` segments).
pub const MIN_SKIDMARK_CAPACITY: usize = 32_768;
/// Default skidmark ring buffer capacity (`256K` segments).
pub const DEFAULT_SKIDMARK_CAPACITY: usize = 262_144;
/// Maximum configurable skidmark ring buffer capacity (`1M` segments).
pub const MAX_SKIDMARK_CAPACITY: usize = 1_048_576;

/// World-space side length (meters) of each spatial skidmark mesh chunk.
pub const SKID_CHUNK_SIZE: f32 = 32.0;

/// World-space side length (meters) of each overdraw saturation grid cell.
const CELL_SIZE: f32 = 0.35;
/// Maximum stacked skidmark segments per `0.35m x 0.35m` world cell before in-place overwrite.
const MAX_SEGMENTS_PER_CELL: usize = 8;
/// Maximum vertices per `Mesh` inside a `SkidChunk` (500 quads = 2000 vertices < Macroquad's quad_gl limit).
const MAX_VERTICES_PER_CHUNK_MESH: usize = 2000;

#[inline]
fn skid_noise(p: Vec2, seed: u32) -> f32 {
    let ix = (p.x * 100.0) as i32 as u32;
    let iy = (p.y * 100.0) as i32 as u32;
    let mut h = ix.wrapping_mul(374761393) ^ iy.wrapping_mul(668265263) ^ seed.wrapping_mul(1274126177);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
}

#[inline]
fn cell_key_for_point(p: Vec2) -> (i32, i32) {
    (
        (p.x / CELL_SIZE).floor() as i32,
        (p.y / CELL_SIZE).floor() as i32,
    )
}

#[inline]
fn chunk_key_for_point(p: Vec2) -> (i32, i32) {
    (
        (p.x / SKID_CHUNK_SIZE).floor() as i32,
        (p.y / SKID_CHUNK_SIZE).floor() as i32,
    )
}

#[inline]
fn segment_midpoint(seg: &SkidSegment) -> Vec2 {
    (seg.p0 + seg.p1 + seg.p2 + seg.p3) * 0.25
}

#[inline]
fn segment_vertices(seg: &SkidSegment) -> [Vertex; 4] {
    [
        Vertex::new(seg.p0.x, seg.p0.y, 0.0, seg.uv0.x, seg.uv0.y, seg.color),
        Vertex::new(seg.p1.x, seg.p1.y, 0.0, seg.uv1.x, seg.uv1.y, seg.color),
        Vertex::new(seg.p2.x, seg.p2.y, 0.0, seg.uv2.x, seg.uv2.y, seg.color),
        Vertex::new(seg.p3.x, seg.p3.y, 0.0, seg.uv3.x, seg.uv3.y, seg.color),
    ]
}

#[inline]
fn degenerate_vertices() -> [Vertex; 4] {
    let zero_col = Color::new(0.0, 0.0, 0.0, 0.0);
    [
        Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, zero_col),
        Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, zero_col),
        Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, zero_col),
        Vertex::new(0.0, 0.0, 0.0, 0.0, 0.0, zero_col),
    ]
}

/// Fixed-capacity per-cell ring slot tracking up to `MAX_SEGMENTS_PER_CELL` segment indices.
#[derive(Debug, Clone, Copy, Default)]
struct CellSlot {
    indices: [u32; MAX_SEGMENTS_PER_CELL],
    len: u8,
    write_pos: u8,
}

/// Direct handle to a segment's 4 vertices inside a spatial `SkidChunk` mesh batch.
#[derive(Debug, Clone, Copy)]
struct ChunkQuadRef {
    chunk_key: (i32, i32),
    mesh_idx: u16,
    vertex_offset: u16,
}

/// Spatial `32m x 32m` chunk holding pre-built GPU `Mesh` batches for O(1) vertex updates and O(chunks) culling.
#[derive(Debug, Clone)]
struct SkidChunk {
    min_bound: Vec2,
    max_bound: Vec2,
    meshes: RefCell<Vec<CachedMesh>>,
    free_slots: Vec<(u16, u16)>,
}

impl SkidChunk {
    fn new(chunk_key: (i32, i32)) -> Self {
        let min_x = chunk_key.0 as f32 * SKID_CHUNK_SIZE;
        let min_y = chunk_key.1 as f32 * SKID_CHUNK_SIZE;
        Self {
            // Margin of 2.0m accounts for quads whose endpoints straddle chunk borders
            min_bound: Vec2::new(min_x - 2.0, min_y - 2.0),
            max_bound: Vec2::new(min_x + SKID_CHUNK_SIZE + 2.0, min_y + SKID_CHUNK_SIZE + 2.0),
            meshes: RefCell::new(Vec::new()),
            free_slots: Vec::new(),
        }
    }

    #[inline]
    fn intersects_view(&self, min: Vec2, max: Vec2) -> bool {
        !(self.max_bound.x < min.x
            || self.min_bound.x > max.x
            || self.max_bound.y < min.y
            || self.min_bound.y > max.y)
    }

    fn alloc_quad(&mut self, seg: &SkidSegment) -> (u16, u16) {
        let verts = segment_vertices(seg);
        let mut meshes = self.meshes.borrow_mut();

        if let Some((mesh_idx, vertex_offset)) = self.free_slots.pop() {
            let m = &mut meshes[mesh_idx as usize].0;
            let base = vertex_offset as usize;
            m.vertices[base..base + 4].copy_from_slice(&verts);
            return (mesh_idx, vertex_offset);
        }

        let need_new_mesh = meshes
            .last()
            .map(|m| m.0.vertices.len() + 4 > MAX_VERTICES_PER_CHUNK_MESH)
            .unwrap_or(true);

        if need_new_mesh {
            meshes.push(CachedMesh(Mesh {
                vertices: Vec::with_capacity(256),
                indices: Vec::with_capacity(384),
                texture: None,
            }));
        }

        let mesh_idx = (meshes.len() - 1) as u16;
        let mesh = &mut meshes[mesh_idx as usize].0;
        let vertex_offset = mesh.vertices.len() as u16;
        mesh.vertices.extend_from_slice(&verts);
        mesh.indices.extend_from_slice(&[
            vertex_offset,
            vertex_offset + 1,
            vertex_offset + 2,
            vertex_offset,
            vertex_offset + 2,
            vertex_offset + 3,
        ]);
        (mesh_idx, vertex_offset)
    }

    #[inline]
    fn overwrite_quad(&mut self, mesh_idx: u16, vertex_offset: u16, seg: &SkidSegment) {
        let verts = segment_vertices(seg);
        let mut meshes = self.meshes.borrow_mut();
        if let Some(cached) = meshes.get_mut(mesh_idx as usize) {
            let base = vertex_offset as usize;
            if base + 4 <= cached.0.vertices.len() {
                cached.0.vertices[base..base + 4].copy_from_slice(&verts);
            }
        }
    }

    #[inline]
    fn free_quad(&mut self, mesh_idx: u16, vertex_offset: u16) {
        let verts = degenerate_vertices();
        let mut meshes = self.meshes.borrow_mut();
        if let Some(cached) = meshes.get_mut(mesh_idx as usize) {
            let base = vertex_offset as usize;
            if base + 4 <= cached.0.vertices.len() {
                cached.0.vertices[base..base + 4].copy_from_slice(&verts);
                self.free_slots.push((mesh_idx, vertex_offset));
            }
        }
    }
}

/// A single persistent 2D skid mark quad segment with UV coordinates for realistic tire tread rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkidSegment {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
    pub uv0: macroquad::prelude::Vec2,
    pub uv1: macroquad::prelude::Vec2,
    pub uv2: macroquad::prelude::Vec2,
    pub uv3: macroquad::prelude::Vec2,
    pub color: Color,
}

/// Bounded ring buffer with spatial cell overdraw saturation and `32m x 32m` pre-built GPU chunk meshes.
#[derive(Debug, Clone)]
pub struct SkidmarkBuffer {
    segments: Vec<SkidSegment>,
    segment_cells: Vec<(i32, i32)>,
    quad_refs: Vec<ChunkQuadRef>,
    chunks: HashMap<(i32, i32), SkidChunk>,
    cell_map: HashMap<(i32, i32), CellSlot>,
    max_capacity: usize,
    write_idx: usize,
    total_count: usize,
    prev_wheel_positions: Vec<[Option<Vec2>; 4]>,
    wheel_accum_v: Vec<[f32; 4]>,
}

impl SkidmarkBuffer {
    pub fn new(capacity: usize) -> Self {
        let prealloc = if capacity > 0 {
            capacity.min(65_536)
        } else {
            8192
        };
        Self {
            segments: Vec::with_capacity(prealloc),
            segment_cells: Vec::with_capacity(prealloc),
            quad_refs: Vec::with_capacity(prealloc),
            chunks: HashMap::new(),
            cell_map: HashMap::new(),
            max_capacity: capacity,
            write_idx: 0,
            total_count: 0,
            prev_wheel_positions: Vec::new(),
            wheel_accum_v: Vec::new(),
        }
    }

    /// Creates a persistent skidmark buffer using the generous default ring capacity (`256K` segments).
    pub fn new_persistent() -> Self {
        Self::new(DEFAULT_SKIDMARK_CAPACITY)
    }

    /// Creates a persistent skidmark buffer with a user-configured capacity clamped to `[32K, 1M]`.
    pub fn new_persistent_with_capacity(capacity: usize) -> Self {
        Self::new(capacity.clamp(MIN_SKIDMARK_CAPACITY, MAX_SKIDMARK_CAPACITY))
    }

    /// Returns the configured maximum segment capacity (`0` if unbounded).
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Updates the ring buffer capacity at runtime, clamping to `[MIN_SKIDMARK_CAPACITY, MAX_SKIDMARK_CAPACITY]`.
    pub fn set_capacity(&mut self, capacity: usize) {
        let clamped = capacity.clamp(MIN_SKIDMARK_CAPACITY, MAX_SKIDMARK_CAPACITY);
        if self.max_capacity == clamped {
            return;
        }
        self.max_capacity = clamped;
        if self.segments.len() > clamped {
            let excess = self.segments.len() - clamped;
            self.segments.drain(0..excess);
            self.write_idx = 0;
            self.rebuild_spatial_index();
        } else if self.write_idx >= clamped {
            self.write_idx %= clamped;
        }
    }

    /// Returns true if the buffer operates in race-persistent mode (`>= MIN_SKIDMARK_CAPACITY` or unbounded).
    pub fn is_persistent(&self) -> bool {
        self.max_capacity == 0 || self.max_capacity >= MIN_SKIDMARK_CAPACITY
    }

    /// Clears all skidmarks and spatial chunk caches from the buffer.
    pub fn clear(&mut self) {
        self.segments.clear();
        self.segment_cells.clear();
        self.quad_refs.clear();
        self.chunks.clear();
        self.cell_map.clear();
        self.write_idx = 0;
        self.total_count = 0;
        self.prev_wheel_positions.clear();
        self.wheel_accum_v.clear();
    }

    /// Number of active skid mark segments.
    pub fn count(&self) -> usize {
        self.segments.len()
    }

    /// Number of allocated `32m x 32m` spatial mesh chunks.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Number of `32m x 32m` spatial mesh chunks intersecting `view_bounds`.
    pub fn visible_chunk_count(&self, view_bounds: Option<(Vec2, Vec2)>) -> usize {
        if let Some((min, max)) = view_bounds {
            self.chunks
                .values()
                .filter(|c| c.intersects_view(min, max))
                .count()
        } else {
            self.chunks.len()
        }
    }

    /// Read-only slice of active skid mark segments.
    pub fn segments(&self) -> &[SkidSegment] {
        &self.segments
    }

    /// Updates the skid mark buffer for a set of active cars on the track.
    pub fn update_for_cars<V: FxVehicle>(&mut self, cars: &[V], surfaces: &[[SurfaceType; 4]]) {
        // Ensure tracking storage matches cars length
        if self.prev_wheel_positions.len() < cars.len() {
            self.prev_wheel_positions.resize(cars.len(), [None; 4]);
            self.wheel_accum_v.resize(cars.len(), [0.0; 4]);
        }

        for (car_idx, car) in cars.iter().enumerate() {
            if car.is_airborne() || car.jump_height() > 0.0 {
                for wheel_id in 0..4 {
                    self.prev_wheel_positions[car_idx][wheel_id] = None;
                }
                continue;
            }

            let wheel_positions = car.contact_points();
            let car_right = car.right_vector();
            let half_tire_w = 0.16;

            for wheel_id in 0..4 {
                let curr_pos = wheel_positions[wheel_id];
                let telemetry = &car.contact_telemetry()[wheel_id];
                let surface = surfaces.get(car_idx).map(|s| s[wheel_id]).unwrap_or(SurfaceType::Asphalt);

                // Mark trigger: active tire slip OR rolling indentation on loose/deformable terrain OR dirt contamination transfer on pavement
                let has_slip = telemetry.skid_intensity > 0.025
                    || telemetry.is_skidding
                    || car.is_drifting()
                    || telemetry.slip_ratio.abs() > 0.10
                    || telemetry.slip_angle.abs() > 0.07;

                let is_rolling_loose = surface.leaves_rolling_rut() && car.speed() > 1.2;
                let is_transferring_dirt = surface.is_rigid_pavement() && telemetry.dirt_contamination > 0.035 && car.speed() > 1.2;

                let leaves_mark = has_slip || is_rolling_loose || is_transferring_dirt;

                if leaves_mark {
                    if let Some(prev_pos) = self.prev_wheel_positions[car_idx][wheel_id] {
                        let travel = curr_pos - prev_pos;
                        let dist = travel.length();

                        // Pure rolling ruts on loose terrain use a coarser 0.45m step and a single full-width
                        // rut quad; active tire slip and pavement dirt transfer keep 0.20m dual-tread detail.
                        let is_pure_rolling_rut = !has_slip && !is_transferring_dirt && is_rolling_loose;
                        let min_step_dist = if is_pure_rolling_rut { 0.45 } else { 0.20 };

                        // Only add segment if vehicle moved sufficiently (prevents static stacking)
                        if (min_step_dist..=3.0).contains(&dist) {
                            let (base_col, alpha, width_mult, jitter_mult) = if is_transferring_dirt {
                                let dirt_col = match telemetry.dirt_surface {
                                    SurfaceType::PackedGravel => Color::new(0.38, 0.36, 0.34, 1.0),
                                    SurfaceType::DeepGravel => Color::new(0.58, 0.53, 0.44, 1.0),
                                    SurfaceType::PackedSand | SurfaceType::DeepSand => Color::new(0.68, 0.58, 0.36, 1.0),
                                    SurfaceType::Dirt => {
                                        let mix = skid_noise(curr_pos, 808);
                                        Color::new(
                                            0.35 + (mix - 0.5) * 0.03,
                                            0.22 + (mix - 0.5) * 0.02,
                                            0.12 + (mix - 0.5) * 0.02,
                                            1.0,
                                        )
                                    }
                                    SurfaceType::MudTrack | SurfaceType::DeepMud => Color::new(0.24, 0.16, 0.08, 1.0),
                                    SurfaceType::Grass => {
                                        let mix = skid_noise(curr_pos, 808);
                                        Color::new(
                                            0.19 + mix * 0.06,
                                            0.28 + (1.0 - mix) * 0.05,
                                            0.14 + mix * 0.04,
                                            1.0,
                                        )
                                    }
                                    _ => Color::new(0.45, 0.45, 0.45, 1.0),
                                };

                                if has_slip && telemetry.dirt_contamination < 0.35 {
                                    // High slip on light dirt: transition toward tire rubber compound
                                    let blend = (telemetry.dirt_contamination / 0.35).clamp(0.0, 1.0);
                                    let col = Color::new(
                                        dirt_col.r * blend + 0.03 * (1.0 - blend),
                                        dirt_col.g * blend + 0.03 * (1.0 - blend),
                                        dirt_col.b * blend + 0.04 * (1.0 - blend),
                                        1.0,
                                    );
                                    let a = ((telemetry.dirt_contamination * 0.38) + telemetry.skid_intensity * 0.28).clamp(0.18, 0.70);
                                    (col, a, 0.86, 0.42)
                                } else {
                                    let a = if has_slip {
                                        ((telemetry.dirt_contamination * 0.42) + telemetry.skid_intensity * 0.20).clamp(0.16, 0.65)
                                    } else {
                                        (telemetry.dirt_contamination * 0.44).clamp(0.14, 0.50)
                                    };
                                    (dirt_col, a, 0.84, 0.48)
                                }
                            } else {
                                match surface {
                                    SurfaceType::Asphalt => {
                                        let a = (0.38 + telemetry.skid_intensity * 0.38).clamp(0.34, 0.78);
                                        (Color::new(0.03, 0.03, 0.04, 1.0), a, 0.90, 0.35)
                                    }
                                    SurfaceType::Concrete => {
                                        let a = (0.34 + telemetry.skid_intensity * 0.36).clamp(0.30, 0.74);
                                        (Color::new(0.04, 0.04, 0.05, 1.0), a, 0.90, 0.32)
                                    }
                                    SurfaceType::Curb => {
                                        let a = (0.30 + telemetry.skid_intensity * 0.34).clamp(0.26, 0.68);
                                        (Color::new(0.05, 0.05, 0.06, 1.0), a, 0.88, 0.30)
                                    }
                                    SurfaceType::PackedGravel | SurfaceType::DeepGravel => {
                                        // Dark slate stone furrow bed with jagged edge jitter
                                        let a = if has_slip {
                                            (0.65 + telemetry.skid_intensity * 0.30).clamp(0.60, 0.92)
                                        } else {
                                            0.42
                                        };
                                        let w = if has_slip { 1.25 } else { 0.95 };
                                        (Color::new(0.28, 0.26, 0.24, 1.0), a, w, 0.45)
                                    }
                                    SurfaceType::PackedSand | SurfaceType::DeepSand => {
                                        // Warm shadowed dune furrow
                                        let a = if has_slip {
                                            (0.55 + telemetry.skid_intensity * 0.35).clamp(0.50, 0.88)
                                        } else {
                                            0.38
                                        };
                                        let w = if has_slip { 1.20 } else { 0.90 };
                                        (Color::new(0.65, 0.52, 0.28, 1.0), a, w, 0.22)
                                    }
                                    SurfaceType::Dirt => {
                                        // Compacted moist loam ruts
                                        let a = if has_slip {
                                            (0.60 + telemetry.skid_intensity * 0.35).clamp(0.55, 0.92)
                                        } else {
                                            0.44
                                        };
                                        let w = if has_slip { 1.15 } else { 0.92 };
                                        (Color::new(0.24, 0.14, 0.07, 1.0), a, w, 0.25)
                                    }
                                    SurfaceType::MudTrack | SurfaceType::DeepMud => {
                                        // Deep viscous muck furrow
                                        let a = if has_slip {
                                            (0.70 + telemetry.skid_intensity * 0.28).clamp(0.65, 0.96)
                                        } else {
                                            0.52
                                        };
                                        let w = if has_slip { 1.35 } else { 1.05 };
                                        (Color::new(0.18, 0.12, 0.06, 1.0), a, w, 0.30)
                                    }
                                    SurfaceType::Grass => {
                                        // Bruised turf & exposed topsoil furrow on slip
                                        let a = if has_slip {
                                            (0.55 + telemetry.skid_intensity * 0.35).clamp(0.50, 0.88)
                                        } else {
                                            0.36
                                        };
                                        let w = if has_slip { 1.10 } else { 0.85 };
                                        let col = if has_slip {
                                            Color::new(0.16, 0.22, 0.10, 1.0)
                                        } else {
                                            Color::new(0.12, 0.28, 0.10, 1.0)
                                        };
                                        (col, a, w, 0.25)
                                    }
                                    SurfaceType::PackedSnow | SurfaceType::DeepSnow => {
                                        // Cool blue-shadowed powder rut
                                        let a = if has_slip {
                                            (0.55 + telemetry.skid_intensity * 0.32).clamp(0.50, 0.85)
                                        } else {
                                            0.40
                                        };
                                        let w = if has_slip { 1.20 } else { 0.95 };
                                        (Color::new(0.65, 0.72, 0.82, 1.0), a, w, 0.20)
                                    }
                                    SurfaceType::SheetIce => {
                                        // Frosted white claw scratch
                                        let a = (0.20 + telemetry.skid_intensity * 0.35).clamp(0.20, 0.55);
                                        (Color::new(0.92, 0.96, 1.0, 1.0), a, 0.75, 0.12)
                                    }
                                    SurfaceType::Water => {
                                        // Translucent parted wake
                                        let a = (0.25 + telemetry.skid_intensity * 0.25).clamp(0.20, 0.50);
                                        (Color::new(0.40, 0.70, 0.90, 0.50), a, 1.30, 0.15)
                                    }
                                    SurfaceType::Oil => {
                                        // Sheared rainbow interference dark film
                                        (Color::new(0.14, 0.11, 0.18, 1.0), 0.60, 1.10, 0.15)
                                    }
                                }
                            };

                            let effective_half_w = half_tire_w * width_mult;

                            // Calculate quad perpendicular to travel direction or tire orientation
                            let seg_right = if dist > 1e-4 {
                                Vec2::new(-travel.y, travel.x) / dist
                            } else {
                                car_right
                            };

                            // Contact chatter and clumping modulation (pulsing alpha along stroke)
                            let chatter_a = 0.72 + skid_noise(curr_pos, 101) * 0.52;
                            let chatter_b = 0.70 + skid_noise(curr_pos, 102) * 0.52;

                            let (mod_a, mod_b) = if is_transferring_dirt {
                                // Irregular clumping and particulate deposition for dirt and grass
                                let clump_a = 0.45 + skid_noise(curr_pos, 606) * 0.85;
                                let clump_b = 0.45 + skid_noise(curr_pos, 707) * 0.85;
                                (chatter_a * clump_a, chatter_b * clump_b)
                            } else {
                                (chatter_a, chatter_b)
                            };

                            let alpha_a = (alpha * mod_a).clamp(0.08, 0.90);
                            let alpha_b = (alpha * mod_b * 0.93).clamp(0.07, 0.88);
                            let col_outer = Color::new(base_col.r, base_col.g, base_col.b, alpha_a);
                            let col_inner = Color::new(base_col.r, base_col.g, base_col.b, alpha_b);

                            // Texture UV longitudinal coordinate mapping (tread pattern repeats every 0.75m)
                            let v0 = self.wheel_accum_v[car_idx][wheel_id];
                            let v1 = v0 + dist / 0.75;
                            self.wheel_accum_v[car_idx][wheel_id] = v1 % 1000.0;

                            if is_pure_rolling_rut {
                                // Single full-width rut ribbon for passive rolling on loose terrain
                                let jitter = (skid_noise(curr_pos, 202) - 0.5) * (effective_half_w * jitter_mult * 0.5);
                                let rut_half_w = effective_half_w * (0.78 + skid_noise(curr_pos, 404) * 0.20);
                                let center_prev = prev_pos + seg_right * jitter;
                                let center_curr = curr_pos + seg_right * jitter;

                                self.add_segment(SkidSegment {
                                    p0: center_prev - seg_right * rut_half_w,
                                    p1: center_prev + seg_right * rut_half_w,
                                    p2: center_curr + seg_right * rut_half_w,
                                    p3: center_curr - seg_right * rut_half_w,
                                    uv0: macroquad::prelude::Vec2::new(0.0, v0),
                                    uv1: macroquad::prelude::Vec2::new(1.0, v0),
                                    uv2: macroquad::prelude::Vec2::new(1.0, v1),
                                    uv3: macroquad::prelude::Vec2::new(0.0, v1),
                                    color: col_outer,
                                });
                            } else {
                                // Multi-ribbon tread contact striations with decoupled ragged edge jitter
                                let jitter_a = (skid_noise(curr_pos, 202) - 0.5) * (effective_half_w * jitter_mult);
                                let jitter_b = (skid_noise(curr_pos, 303) - 0.5) * (effective_half_w * jitter_mult);

                                // Dynamic ribbon width variation across contact patch
                                let width_noise_a = skid_noise(curr_pos, 404);
                                let width_noise_b = skid_noise(curr_pos, 505);
                                let sub_w_a = effective_half_w * (0.36 + width_noise_a * 0.22);
                                let sub_w_b = effective_half_w * (0.36 + width_noise_b * 0.22);

                                let offset_a = effective_half_w * 0.50 + jitter_a;
                                let offset_b = effective_half_w * 0.50 + jitter_b;

                                // Ribbon A: Outer shoulder tread contact track (UV u: 0.00 .. 0.48)
                                let p0_a = (prev_pos - seg_right * offset_a) - seg_right * sub_w_a;
                                let p1_a = (prev_pos - seg_right * offset_a) + seg_right * sub_w_a;
                                let p2_a = (curr_pos - seg_right * offset_a) + seg_right * sub_w_a;
                                let p3_a = (curr_pos - seg_right * offset_a) - seg_right * sub_w_a;

                                let uv0_a = macroquad::prelude::Vec2::new(0.0, v0);
                                let uv1_a = macroquad::prelude::Vec2::new(0.48, v0);
                                let uv2_a = macroquad::prelude::Vec2::new(0.48, v1);
                                let uv3_a = macroquad::prelude::Vec2::new(0.0, v1);

                                // Ribbon B: Inner shoulder tread contact track (UV u: 0.52 .. 1.00)
                                let p0_b = (prev_pos + seg_right * offset_b) - seg_right * sub_w_b;
                                let p1_b = (prev_pos + seg_right * offset_b) + seg_right * sub_w_b;
                                let p2_b = (curr_pos + seg_right * offset_b) + seg_right * sub_w_b;
                                let p3_b = (curr_pos + seg_right * offset_b) - seg_right * sub_w_b;

                                let uv0_b = macroquad::prelude::Vec2::new(0.52, v0);
                                let uv1_b = macroquad::prelude::Vec2::new(1.00, v0);
                                let uv2_b = macroquad::prelude::Vec2::new(1.00, v1);
                                let uv3_b = macroquad::prelude::Vec2::new(0.52, v1);

                                self.add_segment(SkidSegment {
                                    p0: p0_a,
                                    p1: p1_a,
                                    p2: p2_a,
                                    p3: p3_a,
                                    uv0: uv0_a,
                                    uv1: uv1_a,
                                    uv2: uv2_a,
                                    uv3: uv3_a,
                                    color: col_outer,
                                });
                                self.add_segment(SkidSegment {
                                    p0: p0_b,
                                    p1: p1_b,
                                    p2: p2_b,
                                    p3: p3_b,
                                    uv0: uv0_b,
                                    uv1: uv1_b,
                                    uv2: uv2_b,
                                    uv3: uv3_b,
                                    color: col_inner,
                                });
                            }

                            self.prev_wheel_positions[car_idx][wheel_id] = Some(curr_pos);
                        } else if dist > 3.0 {
                            // Car jumped or teleported across track: reset anchor without stretching quad
                            self.prev_wheel_positions[car_idx][wheel_id] = Some(curr_pos);
                        }
                    } else {
                        self.prev_wheel_positions[car_idx][wheel_id] = Some(curr_pos);
                    }
                } else {
                    // Break the contiguous skid line
                    self.prev_wheel_positions[car_idx][wheel_id] = None;
                }
            }
        }
    }

    /// Adds a skid segment to the buffer, applying per-cell overdraw saturation and O(1) spatial chunk mesh updates.
    fn add_segment(&mut self, segment: SkidSegment) {
        let mid = segment_midpoint(&segment);
        let cell_key = cell_key_for_point(mid);
        let chunk_key = chunk_key_for_point(mid);

        // 1. Check spatial cell overdraw saturation (max 8 segments per 0.35m x 0.35m cell)
        if let Some(cell) = self.cell_map.get_mut(&cell_key) {
            if (cell.len as usize) >= MAX_SEGMENTS_PER_CELL {
                let overwrite_idx = cell.indices[cell.write_pos as usize] as usize;
                cell.write_pos = (cell.write_pos + 1) % (MAX_SEGMENTS_PER_CELL as u8);
                if overwrite_idx < self.segments.len() {
                    self.segments[overwrite_idx] = segment;
                    self.segment_cells[overwrite_idx] = cell_key;
                    let old_qref = self.quad_refs[overwrite_idx];
                    if old_qref.chunk_key == chunk_key {
                        if let Some(chunk) = self.chunks.get_mut(&chunk_key) {
                            chunk.overwrite_quad(old_qref.mesh_idx, old_qref.vertex_offset, &segment);
                        }
                    } else {
                        if let Some(old_chunk) = self.chunks.get_mut(&old_qref.chunk_key) {
                            old_chunk.free_quad(old_qref.mesh_idx, old_qref.vertex_offset);
                        }
                        let new_qref = self.alloc_chunk_quad(chunk_key, &segment);
                        self.quad_refs[overwrite_idx] = new_qref;
                    }
                    self.total_count += 1;
                    return;
                }
            }
        }

        // 2. Append to buffer or wrap around ring capacity
        let seg_idx = if self.max_capacity == 0 || self.segments.len() < self.max_capacity {
            let idx = self.segments.len();
            let qref = self.alloc_chunk_quad(chunk_key, &segment);
            self.segments.push(segment);
            self.segment_cells.push(cell_key);
            self.quad_refs.push(qref);
            idx
        } else {
            let idx = self.write_idx;
            self.write_idx = (self.write_idx + 1) % self.max_capacity;

            let old_cell = self.segment_cells[idx];
            self.remove_from_cell(idx, old_cell);

            let old_qref = self.quad_refs[idx];
            self.segments[idx] = segment;
            self.segment_cells[idx] = cell_key;

            if old_qref.chunk_key == chunk_key {
                if let Some(chunk) = self.chunks.get_mut(&chunk_key) {
                    chunk.overwrite_quad(old_qref.mesh_idx, old_qref.vertex_offset, &segment);
                }
            } else {
                if let Some(old_chunk) = self.chunks.get_mut(&old_qref.chunk_key) {
                    old_chunk.free_quad(old_qref.mesh_idx, old_qref.vertex_offset);
                }
                self.quad_refs[idx] = self.alloc_chunk_quad(chunk_key, &segment);
            }
            idx
        };

        let cell = self.cell_map.entry(cell_key).or_default();
        if (cell.len as usize) < MAX_SEGMENTS_PER_CELL {
            cell.indices[cell.len as usize] = seg_idx as u32;
            cell.len += 1;
        }

        self.total_count += 1;
    }

    #[inline]
    fn alloc_chunk_quad(&mut self, chunk_key: (i32, i32), segment: &SkidSegment) -> ChunkQuadRef {
        let chunk = self
            .chunks
            .entry(chunk_key)
            .or_insert_with(|| SkidChunk::new(chunk_key));
        let (mesh_idx, vertex_offset) = chunk.alloc_quad(segment);
        ChunkQuadRef {
            chunk_key,
            mesh_idx,
            vertex_offset,
        }
    }

    #[inline]
    fn remove_from_cell(&mut self, seg_idx: usize, cell_key: (i32, i32)) {
        if let Some(cell) = self.cell_map.get_mut(&cell_key) {
            let target = seg_idx as u32;
            for i in 0..(cell.len as usize) {
                if cell.indices[i] == target {
                    for j in i..((cell.len as usize) - 1) {
                        cell.indices[j] = cell.indices[j + 1];
                    }
                    cell.len -= 1;
                    if cell.len == 0 {
                        cell.write_pos = 0;
                    } else if cell.write_pos >= cell.len {
                        cell.write_pos %= cell.len;
                    }
                    break;
                }
            }
        }
    }

    fn rebuild_spatial_index(&mut self) {
        self.segment_cells.clear();
        self.quad_refs.clear();
        self.chunks.clear();
        self.cell_map.clear();

        for idx in 0..self.segments.len() {
            let seg = self.segments[idx];
            let mid = segment_midpoint(&seg);
            let cell_key = cell_key_for_point(mid);
            let chunk_key = chunk_key_for_point(mid);

            let qref = self.alloc_chunk_quad(chunk_key, &seg);
            self.segment_cells.push(cell_key);
            self.quad_refs.push(qref);

            let cell = self.cell_map.entry(cell_key).or_default();
            if (cell.len as usize) < MAX_SEGMENTS_PER_CELL {
                cell.indices[cell.len as usize] = idx as u32;
                cell.len += 1;
            }
        }
    }

    /// Renders active skid marks with optional camera viewport culling using the global surface material registry's tire rubber texture.
    pub fn render_culled(&self, view_bounds: Option<(Vec2, Vec2)>) {
        let tex = crate::render::track::with_surface_registry(|reg| {
            reg.tire_rubber_texture().cloned()
        }).flatten();
        self.render_textured_culled(tex.as_ref(), view_bounds);
    }

    /// Renders all active skid marks using the global surface material registry's tire rubber texture.
    pub fn render(&self) {
        self.render_culled(None);
    }

    /// Renders active skid marks using pre-built `32m x 32m` spatial chunk meshes and O(chunks) viewport AABB culling.
    pub fn render_textured_culled(
        &self,
        texture: Option<&macroquad::texture::Texture2D>,
        view_bounds: Option<(Vec2, Vec2)>,
    ) {
        if self.segments.is_empty() || !is_render_context_active() {
            return;
        }

        for chunk in self.chunks.values() {
            if let Some((min, max)) = view_bounds {
                if !chunk.intersects_view(min, max) {
                    continue;
                }
            }
            let mut meshes = chunk.meshes.borrow_mut();
            for cached in meshes.iter_mut() {
                if !cached.0.vertices.is_empty() {
                    cached.0.texture = texture.cloned();
                    draw_mesh(&cached.0);
                }
            }
        }
    }

    /// Renders all active skid marks as a high-performance GPU batch mesh.
    pub fn render_textured(&self, texture: Option<&macroquad::texture::Texture2D>) {
        self.render_textured_culled(texture, None);
    }
}

