use glam::Vec2;
use macroquad::color::Color;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::surface::SurfaceType;

#[inline]
fn skid_noise(p: Vec2, seed: u32) -> f32 {
    let ix = (p.x * 100.0) as i32 as u32;
    let iy = (p.y * 100.0) as i32 as u32;
    let mut h = ix.wrapping_mul(374761393) ^ iy.wrapping_mul(668265263) ^ seed.wrapping_mul(1274126177);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xFFFF) as f32 / 65535.0
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

/// Fixed-capacity ring buffer for persistent skid marks on the racing circuit.
#[derive(Debug, Clone)]
pub struct SkidmarkBuffer {
    segments: Vec<SkidSegment>,
    max_capacity: usize,
    write_idx: usize,
    total_count: usize,
    prev_wheel_positions: Vec<[Option<Vec2>; 4]>,
    wheel_accum_v: Vec<[f32; 4]>,
}

impl SkidmarkBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            segments: if capacity > 0 {
                Vec::with_capacity(capacity.min(64000))
            } else {
                Vec::with_capacity(8192)
            },
            max_capacity: capacity,
            write_idx: 0,
            total_count: 0,
            prev_wheel_positions: Vec::new(),
            wheel_accum_v: Vec::new(),
        }
    }

    /// Creates a persistent skidmark buffer that retains all tire rubber traces
    /// across the whole race session, from first to last lap, without overwriting.
    pub fn new_persistent() -> Self {
        Self::new(0)
    }

    /// Returns true if the buffer operates in persistent mode (no mid-race overwrite).
    pub fn is_persistent(&self) -> bool {
        self.max_capacity == 0
    }

    /// Clears all skidmarks from the buffer.
    pub fn clear(&mut self) {
        self.segments.clear();
        self.write_idx = 0;
        self.total_count = 0;
        self.prev_wheel_positions.clear();
        self.wheel_accum_v.clear();
    }

    /// Number of active skid mark segments.
    pub fn count(&self) -> usize {
        self.segments.len()
    }

    /// Read-only slice of active skid mark segments.
    pub fn segments(&self) -> &[SkidSegment] {
        &self.segments
    }

    /// Updates the skid mark buffer for a set of active cars on the track.
    pub fn update_for_cars(&mut self, cars: &[Car], surfaces: &[[SurfaceType; 4]]) {
        // Ensure tracking storage matches cars length
        if self.prev_wheel_positions.len() < cars.len() {
            self.prev_wheel_positions.resize(cars.len(), [None; 4]);
            self.wheel_accum_v.resize(cars.len(), [0.0; 4]);
        }

        for (car_idx, car) in cars.iter().enumerate() {
            if car.state.is_airborne || car.state.elevation > 0.0 {
                for wheel_id in 0..4 {
                    self.prev_wheel_positions[car_idx][wheel_id] = None;
                }
                continue;
            }

            let wheel_positions = car.wheel_positions_world();
            let car_right = car.right_vector();
            let half_tire_w = 0.16;

            for wheel_id in 0..4 {
                let curr_pos = wheel_positions[wheel_id];
                let telemetry = &car.state.wheels[wheel_id];
                let surface = surfaces.get(car_idx).map(|s| s[wheel_id]).unwrap_or(SurfaceType::Asphalt);

                // Mark trigger: active tire slip OR rolling indentation on loose/deformable terrain OR dirt contamination transfer on pavement
                let has_slip = telemetry.skid_intensity > 0.025
                    || telemetry.is_skidding
                    || car.state.is_drifting
                    || telemetry.slip_ratio.abs() > 0.10
                    || telemetry.slip_angle.abs() > 0.07;

                let is_rolling_loose = surface.leaves_rolling_rut() && car.state.speed > 1.2;
                let is_transferring_dirt = surface.is_rigid_pavement() && telemetry.dirt_contamination > 0.03 && car.state.speed > 1.2;

                let leaves_mark = has_slip || is_rolling_loose || is_transferring_dirt;

                if leaves_mark {
                    if let Some(prev_pos) = self.prev_wheel_positions[car_idx][wheel_id] {
                        let travel = curr_pos - prev_pos;
                        let dist = travel.length();

                        // Only add segment if vehicle moved sufficiently (prevents static stacking)
                        if (0.20..=3.0).contains(&dist) {
                            let (base_col, alpha, width_mult, jitter_mult) = if is_transferring_dirt && !has_slip {
                                let dirt_col = match telemetry.dirt_surface {
                                    SurfaceType::Gravel => Color::new(0.38, 0.36, 0.34, 1.0),
                                    SurfaceType::Sand => Color::new(0.68, 0.58, 0.36, 1.0),
                                    SurfaceType::Dirt => Color::new(0.35, 0.22, 0.12, 1.0),
                                    SurfaceType::Mud => Color::new(0.24, 0.16, 0.08, 1.0),
                                    SurfaceType::Grass => Color::new(0.22, 0.30, 0.16, 1.0),
                                    _ => Color::new(0.45, 0.45, 0.45, 1.0),
                                };
                                let a = (telemetry.dirt_contamination * 0.55).clamp(0.18, 0.65);
                                (dirt_col, a, 0.92, 0.20)
                            } else {
                                match surface {
                                    SurfaceType::Asphalt => {
                                        let a = (0.50 + telemetry.skid_intensity * 0.45).clamp(0.45, 0.95);
                                        (Color::new(0.03, 0.03, 0.04, 1.0), a, 1.0, 0.20)
                                    }
                                    SurfaceType::Concrete => {
                                        let a = (0.45 + telemetry.skid_intensity * 0.45).clamp(0.40, 0.90);
                                        (Color::new(0.04, 0.04, 0.05, 1.0), a, 1.0, 0.20)
                                    }
                                    SurfaceType::Curb => {
                                        let a = (0.40 + telemetry.skid_intensity * 0.40).clamp(0.35, 0.80);
                                        (Color::new(0.05, 0.05, 0.06, 1.0), a, 0.95, 0.18)
                                    }
                                    SurfaceType::Gravel => {
                                        // Dark slate stone furrow bed with jagged edge jitter
                                        let a = if has_slip {
                                            (0.65 + telemetry.skid_intensity * 0.30).clamp(0.60, 0.92)
                                        } else {
                                            0.42
                                        };
                                        let w = if has_slip { 1.25 } else { 0.95 };
                                        (Color::new(0.28, 0.26, 0.24, 1.0), a, w, 0.45)
                                    }
                                    SurfaceType::Sand => {
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
                                    SurfaceType::Mud => {
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
                                    SurfaceType::Snow => {
                                        // Cool blue-shadowed powder rut
                                        let a = if has_slip {
                                            (0.55 + telemetry.skid_intensity * 0.32).clamp(0.50, 0.85)
                                        } else {
                                            0.40
                                        };
                                        let w = if has_slip { 1.20 } else { 0.95 };
                                        (Color::new(0.65, 0.72, 0.82, 1.0), a, w, 0.20)
                                    }
                                    SurfaceType::Ice => {
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

                            // Contact chatter modulation (pulsing alpha along stroke)
                            let chatter = 0.88 + skid_noise(curr_pos, 101) * 0.24;
                            let alpha_mod = (alpha * chatter).clamp(0.15, 0.96);
                            let col_outer = Color::new(base_col.r, base_col.g, base_col.b, alpha_mod);
                            let col_inner = Color::new(base_col.r, base_col.g, base_col.b, (alpha_mod * 0.94).clamp(0.14, 0.92));

                            // Multi-ribbon tread contact striations with ragged edge jitter
                            let jitter = (skid_noise(curr_pos, 202) - 0.5) * (effective_half_w * jitter_mult);
                            let sub_w = effective_half_w * 0.48;
                            let offset = effective_half_w * 0.50 + jitter;

                            // Texture UV longitudinal coordinate mapping (tread pattern repeats every 0.75m)
                            let v0 = self.wheel_accum_v[car_idx][wheel_id];
                            let v1 = v0 + dist / 0.75;
                            self.wheel_accum_v[car_idx][wheel_id] = v1 % 1000.0;

                            // Ribbon A: Outer shoulder tread contact track (UV u: 0.00 .. 0.48)
                            let p0_a = (prev_pos - seg_right * offset) - seg_right * sub_w;
                            let p1_a = (prev_pos - seg_right * offset) + seg_right * sub_w;
                            let p2_a = (curr_pos - seg_right * offset) + seg_right * sub_w;
                            let p3_a = (curr_pos - seg_right * offset) - seg_right * sub_w;

                            let uv0_a = macroquad::prelude::Vec2::new(0.0, v0);
                            let uv1_a = macroquad::prelude::Vec2::new(0.48, v0);
                            let uv2_a = macroquad::prelude::Vec2::new(0.48, v1);
                            let uv3_a = macroquad::prelude::Vec2::new(0.0, v1);

                            // Ribbon B: Inner shoulder tread contact track (UV u: 0.52 .. 1.00)
                            let p0_b = (prev_pos + seg_right * offset) - seg_right * sub_w;
                            let p1_b = (prev_pos + seg_right * offset) + seg_right * sub_w;
                            let p2_b = (curr_pos + seg_right * offset) + seg_right * sub_w;
                            let p3_b = (curr_pos + seg_right * offset) - seg_right * sub_w;

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

    /// Adds a skid segment to the buffer (persistent or ring-buffered if bounded).
    fn add_segment(&mut self, segment: SkidSegment) {
        if self.max_capacity == 0 || self.segments.len() < self.max_capacity {
            self.segments.push(segment);
        } else {
            self.segments[self.write_idx] = segment;
            self.write_idx = (self.write_idx + 1) % self.max_capacity;
        }
        self.total_count += 1;
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

    /// Renders active skid marks using a specific texture with optional camera viewport culling.
    pub fn render_textured_culled(
        &self,
        texture: Option<&macroquad::texture::Texture2D>,
        view_bounds: Option<(Vec2, Vec2)>,
    ) {
        if self.segments.is_empty() {
            return;
        }

        let mut builder = crate::render::track::BatchMeshBuilder::new(texture.cloned());
        if let Some((min, max)) = view_bounds {
            for seg in &self.segments {
                if is_quad_in_view(seg.p0, seg.p1, seg.p2, seg.p3, min, max) {
                    builder.push_quad(
                        seg.p0, seg.uv0, seg.color,
                        seg.p1, seg.uv1, seg.color,
                        seg.p2, seg.uv2, seg.color,
                        seg.p3, seg.uv3, seg.color,
                    );
                }
            }
        } else {
            for seg in &self.segments {
                builder.push_quad(
                    seg.p0, seg.uv0, seg.color,
                    seg.p1, seg.uv1, seg.color,
                    seg.p2, seg.uv2, seg.color,
                    seg.p3, seg.uv3, seg.color,
                );
            }
        }
        builder.flush();
    }

    /// Renders all active skid marks as a high-performance GPU batch mesh.
    pub fn render_textured(&self, texture: Option<&macroquad::texture::Texture2D>) {
        self.render_textured_culled(texture, None);
    }
}

#[inline]
fn is_quad_in_view(
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    min: Vec2, max: Vec2,
) -> bool {
    let s_min_x = p0.x.min(p1.x).min(p2.x).min(p3.x);
    let s_max_x = p0.x.max(p1.x).max(p2.x).max(p3.x);
    let s_min_y = p0.y.min(p1.y).min(p2.y).min(p3.y);
    let s_max_y = p0.y.max(p1.y).max(p2.y).max(p3.y);
    !(s_max_x < min.x || s_min_x > max.x || s_max_y < min.y || s_min_y > max.y)
}
