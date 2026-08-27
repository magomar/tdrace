use glam::Vec2;
use macroquad::color::Color;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::surface::SurfaceType;

use crate::render::color::Palette;
use crate::render::track::draw_quad;

/// A single persistent 2D skid mark quad segment.
#[derive(Debug, Clone, Copy)]
pub struct SkidSegment {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
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
}

impl SkidmarkBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            segments: Vec::with_capacity(capacity),
            max_capacity: capacity,
            write_idx: 0,
            total_count: 0,
            prev_wheel_positions: Vec::new(),
        }
    }

    /// Clears all skidmarks from the buffer.
    pub fn clear(&mut self) {
        self.segments.clear();
        self.write_idx = 0;
        self.total_count = 0;
        self.prev_wheel_positions.clear();
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
            let half_tire_w = 0.12;

            for wheel_id in 0..4 {
                let curr_pos = wheel_positions[wheel_id];
                let telemetry = &car.state.wheels[wheel_id];
                let surface = surfaces.get(car_idx).map(|s| s[wheel_id]).unwrap_or(SurfaceType::Asphalt);

                // Skid intensity: based on telemetry skid_intensity and surface
                let is_skidding = telemetry.skid_intensity > 0.08
                    || (car.state.is_drifting && telemetry.skid_intensity > 0.04)
                    || (surface == SurfaceType::Grass && telemetry.slip_ratio.abs() > 0.25);

                if is_skidding {
                    if let Some(prev_pos) = self.prev_wheel_positions[car_idx][wheel_id] {
                        let travel = curr_pos - prev_pos;
                        let dist = travel.length();

                        // Only add segment if vehicle moved sufficiently (prevents static stacking)
                        if (0.12..3.0).contains(&dist) {
                            let (base_col, alpha_mult) = match surface {
                                SurfaceType::Grass => (Color::new(0.18, 0.38, 0.16, 1.0), 0.70),
                                SurfaceType::Sand => (Color::new(0.70, 0.60, 0.35, 1.0), 0.75),
                                SurfaceType::Dirt => (Palette::DIRT_DARK, 0.70),
                                SurfaceType::Water => (Color::new(0.30, 0.70, 0.95, 0.40), 0.30),
                                _ => (Palette::SKIDMARK, 0.85),
                            };

                            let alpha = (telemetry.skid_intensity * alpha_mult).clamp(0.08, 0.85);
                            let color = Color::new(base_col.r, base_col.g, base_col.b, alpha);

                            // Calculate quad perpendicular to travel direction or tire orientation
                            let seg_right = if dist > 1e-4 {
                                Vec2::new(-travel.y, travel.x) / dist
                            } else {
                                car_right
                            };

                            let p0 = prev_pos - seg_right * half_tire_w;
                            let p1 = prev_pos + seg_right * half_tire_w;
                            let p2 = curr_pos + seg_right * half_tire_w;
                            let p3 = curr_pos - seg_right * half_tire_w;

                            self.add_segment(SkidSegment {
                                p0,
                                p1,
                                p2,
                                p3,
                                color,
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

    /// Renders all active skid marks.
    pub fn render(&self) {
        for seg in &self.segments {
            draw_quad(seg.p0, seg.p1, seg.p2, seg.p3, seg.color);
        }
    }
}
