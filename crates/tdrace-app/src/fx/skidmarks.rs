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
#[derive(Debug, Clone, Copy)]
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
            segments: Vec::with_capacity(capacity),
            max_capacity: capacity,
            write_idx: 0,
            total_count: 0,
            prev_wheel_positions: Vec::new(),
            wheel_accum_v: Vec::new(),
        }
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

                // Skid intensity: based on telemetry skid_intensity, slip angle, slip ratio, and drift
                let is_skidding = telemetry.skid_intensity > 0.025
                    || telemetry.is_skidding
                    || car.state.is_drifting
                    || telemetry.slip_ratio.abs() > 0.10
                    || telemetry.slip_angle.abs() > 0.07
                    || (surface == SurfaceType::Grass && telemetry.slip_ratio.abs() > 0.18);

                if is_skidding {
                    if let Some(prev_pos) = self.prev_wheel_positions[car_idx][wheel_id] {
                        let travel = curr_pos - prev_pos;
                        let dist = travel.length();

                        // Only add segment if vehicle moved sufficiently (prevents static stacking)
                        if (0.10..3.0).contains(&dist) {
                            let (base_col, alpha_mult) = match surface {
                                SurfaceType::Grass => (Color::new(0.12, 0.28, 0.10, 1.0), 0.85),
                                SurfaceType::Sand => (Color::new(0.55, 0.45, 0.25, 1.0), 0.85),
                                SurfaceType::Dirt => (Color::new(0.25, 0.15, 0.08, 1.0), 0.85),
                                SurfaceType::Water => (Color::new(0.40, 0.70, 0.90, 0.50), 0.50),
                                _ => (Color::new(0.03, 0.03, 0.04, 1.0), 1.0), // Deep carbon black rubber
                            };

                            let alpha = (0.50 + telemetry.skid_intensity * 0.45 * alpha_mult).clamp(0.45, 0.95);

                            // Calculate quad perpendicular to travel direction or tire orientation
                            let seg_right = if dist > 1e-4 {
                                Vec2::new(-travel.y, travel.x) / dist
                            } else {
                                car_right
                            };

                            // Contact chatter modulation (pulsing alpha along stroke)
                            let chatter = 0.88 + skid_noise(curr_pos, 101) * 0.24;
                            let alpha_mod = (alpha * chatter).clamp(0.40, 0.95);
                            let col_outer = Color::new(base_col.r, base_col.g, base_col.b, alpha_mod);
                            let col_inner = Color::new(base_col.r, base_col.g, base_col.b, (alpha_mod * 0.94).clamp(0.38, 0.92));

                            // Multi-ribbon tread contact striations with ragged edge jitter
                            let jitter = (skid_noise(curr_pos, 202) - 0.5) * (half_tire_w * 0.20);
                            let sub_w = half_tire_w * 0.48;
                            let offset = half_tire_w * 0.50 + jitter;

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
                        }
                    }
                    self.prev_wheel_positions[car_idx][wheel_id] = Some(curr_pos);
                } else {
                    // Break the contiguous skid line
                    self.prev_wheel_positions[car_idx][wheel_id] = None;
                }
            }
        }
    }

    /// Adds a skid segment to the ring buffer.
    fn add_segment(&mut self, segment: SkidSegment) {
        if self.segments.len() < self.max_capacity {
            self.segments.push(segment);
        } else {
            self.segments[self.write_idx] = segment;
            self.write_idx = (self.write_idx + 1) % self.max_capacity;
        }
        self.total_count += 1;
    }

    /// Renders all active skid marks using the global surface material registry's tire rubber texture.
    pub fn render(&self) {
        let tex = crate::render::track::with_surface_registry(|reg| {
            reg.tire_rubber_texture().cloned()
        }).flatten();
        self.render_textured(tex.as_ref());
    }

    /// Renders all active skid marks as a high-performance GPU batch mesh.
    pub fn render_textured(&self, texture: Option<&macroquad::texture::Texture2D>) {
        if self.segments.is_empty() {
            return;
        }

        let mut builder = crate::render::track::BatchMeshBuilder::new(texture.cloned());
        for seg in &self.segments {
            builder.push_quad(
                seg.p0, seg.uv0, seg.color,
                seg.p1, seg.uv1, seg.color,
                seg.p2, seg.uv2, seg.color,
                seg.p3, seg.uv3, seg.color,
            );
        }
        builder.flush();
    }
}
