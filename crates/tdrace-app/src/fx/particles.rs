use macroquad::color::Color;
use macroquad::shapes::{draw_circle, draw_line};
use glam::Vec2;
use tdrace_core::physics::surface::SurfaceType;

use crate::render::color::Palette;

/// Individual particle state.
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size_start: f32,
    pub size_end: f32,
    pub color_start: Color,
    pub color_end: Color,
    pub lifetime: f32,
    pub remaining_life: f32,
    pub drag: f32,
    pub is_spark: bool,
}

/// Particle manager handling tire smoke, off-track dirt/grass, and collision sparks.
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    max_particles: usize,
    rng_state: u32,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
            rng_state: 1337,
        }
    }

    /// Clears all active particles.
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// Count of active particles.
    pub fn count(&self) -> usize {
        self.particles.len()
    }

    /// Simple fast deterministic pseudo-random float in [0.0, 1.0).
    fn rand_f32(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.rng_state >> 8) as f32 / 16777216.0
    }

    /// Random float in [-1.0, 1.0].
    fn rand_signed(&mut self) -> f32 {
        self.rand_f32() * 2.0 - 1.0
    }

    /// Emits tire smoke puffs during heavy skids or power slides.
    pub fn emit_tire_smoke(&mut self, pos: Vec2, base_vel: Vec2, intensity: f32) {
        if self.particles.len() >= self.max_particles {
            return;
        }

        let count = ((intensity * 1.8).ceil() as usize).clamp(1, 2);
        for _ in 0..count {
            if self.particles.len() >= self.max_particles {
                break;
            }

            let spread = Vec2::new(self.rand_signed(), self.rand_signed()) * 0.25;
            let p_pos = pos + spread;
            let rand_vel = Vec2::new(self.rand_signed(), self.rand_signed()) * (0.8 * intensity);
            let p_vel = base_vel * 0.10 + rand_vel;

            let life = 0.25 + self.rand_f32() * 0.20;
            let start_size = 0.10 + self.rand_f32() * 0.08;
            let end_size = 0.38 + self.rand_f32() * 0.25;

            let alpha_start = (0.18 * intensity).clamp(0.05, 0.30);
            let col_start = Color::new(0.92, 0.92, 0.95, alpha_start);
            let col_end = Color::new(0.92, 0.92, 0.95, 0.0);

            self.particles.push(Particle {
                pos: p_pos,
                vel: p_vel,
                size_start: start_size,
                size_end: end_size,
                color_start: col_start,
                color_end: col_end,
                lifetime: life,
                remaining_life: life,
                drag: 3.5,
                is_spark: false,
            });
        }
    }

    /// Emits dirt/grass/sand roost particles when wheels slide or accelerate off-track.
    pub fn emit_dirt_roost(&mut self, pos: Vec2, surface: SurfaceType, wheel_vel: Vec2) {
        if self.particles.len() >= self.max_particles {
            return;
        }

        let (base_col, col_var) = match surface {
            SurfaceType::Dirt => (Palette::DIRT, Palette::DIRT_DARK),
            SurfaceType::Grass => (Palette::GRASS_DARK, Color::new(0.35, 0.25, 0.12, 0.9)),
            SurfaceType::Sand => (Palette::SAND, Palette::SAND_DARK),
            _ => return,
        };

        let count = 3;
        for _ in 0..count {
            if self.particles.len() >= self.max_particles {
                break;
            }

            let p_pos = pos + Vec2::new(self.rand_signed(), self.rand_signed()) * 0.2;
            // Roost flies backward relative to wheel velocity
            let roost_dir = -wheel_vel.normalize_or_zero();
            let spread = Vec2::new(-roost_dir.y, roost_dir.x) * self.rand_signed() * 0.6;
            let p_vel = (roost_dir + spread) * (3.0 + self.rand_f32() * 6.0);

            let life = 0.35 + self.rand_f32() * 0.30;
            let size = 0.08 + self.rand_f32() * 0.08;

            let use_secondary = self.rand_f32() > 0.5;
            let col = if use_secondary { col_var } else { base_col };
            let col_end = Color::new(col.r, col.g, col.b, 0.0);

            self.particles.push(Particle {
                pos: p_pos,
                vel: p_vel,
                size_start: size,
                size_end: size * 0.5,
                color_start: col,
                color_end: col_end,
                lifetime: life,
                remaining_life: life,
                drag: 2.8,
                is_spark: false,
            });
        }
    }

    /// Emits dynamic water splash droplet plumes and white foam when wheels drive through water patches.
    pub fn emit_water_splash(&mut self, pos: Vec2, wheel_vel: Vec2, speed: f32) {
        if self.particles.len() >= self.max_particles {
            return;
        }

        let count = ((speed * 0.45).ceil() as usize).clamp(2, 6);
        for _ in 0..count {
            if self.particles.len() >= self.max_particles {
                break;
            }

            let splash_dir = -wheel_vel.normalize_or_zero();
            let spread = Vec2::new(-splash_dir.y, splash_dir.x) * self.rand_signed() * 1.1;
            let p_vel = (splash_dir + spread).normalize_or_zero() * (2.5 + self.rand_f32() * 7.0 + speed * 0.2);

            let p_pos = pos + Vec2::new(self.rand_signed(), self.rand_signed()) * 0.25;
            let life = 0.25 + self.rand_f32() * 0.25;
            let size = 0.10 + self.rand_f32() * 0.10;

            let is_foam = self.rand_f32() > 0.4;
            let col = if is_foam {
                Color::new(0.92, 0.96, 1.0, 0.85) // Frosted white spray
            } else {
                Color::new(0.35, 0.78, 1.0, 0.70) // Aqua droplet
            };
            let col_end = Color::new(col.r, col.g, col.b, 0.0);

            self.particles.push(Particle {
                pos: p_pos,
                vel: p_vel,
                size_start: size,
                size_end: size * 1.6,
                color_start: col,
                color_end: col_end,
                lifetime: life,
                remaining_life: life,
                drag: 3.2,
                is_spark: false,
            });
        }
    }

    /// Emits brilliant orange/yellow sparks on wall collisions or heavy car impacts.
    pub fn emit_sparks(&mut self, contact_point: Vec2, normal: Vec2, impact_speed: f32) {
        if self.particles.len() >= self.max_particles {
            return;
        }

        let spark_count = ((impact_speed * 1.8).ceil() as usize).clamp(4, 24);
        let tangent = Vec2::new(-normal.y, normal.x);

        for _ in 0..spark_count {
            if self.particles.len() >= self.max_particles {
                break;
            }

            // Scatter along collision normal and tangent
            let norm_spread = normal * (self.rand_f32() * 0.8 + 0.2);
            let tang_spread = tangent * self.rand_signed() * 1.2;
            let spark_dir = (norm_spread + tang_spread).normalize_or_zero();
            let speed = (impact_speed * 0.8 + 4.0 + self.rand_f32() * 12.0).clamp(4.0, 30.0);
            let p_vel = spark_dir * speed;

            let life = 0.18 + self.rand_f32() * 0.25;
            let col_start = if self.rand_f32() > 0.4 {
                Palette::SPARK
            } else {
                Palette::SPARK_WHITE
            };
            let col_end = Color::new(1.0, 0.25, 0.05, 0.0);

            self.particles.push(Particle {
                pos: contact_point,
                vel: p_vel,
                size_start: 0.12,
                size_end: 0.04,
                color_start: col_start,
                color_end: col_end,
                lifetime: life,
                remaining_life: life,
                drag: 4.5,
                is_spark: true,
            });
        }
    }

    /// Emits 360-degree landing dust / smoke burst when car touches down from a jump.
    pub fn emit_landing_dust(&mut self, pos: Vec2, impact_speed: f32, surface: SurfaceType) {
        if self.particles.len() >= self.max_particles {
            return;
        }

        let (base_col, col_var) = match surface {
            SurfaceType::Sand => (Palette::SAND, Palette::SAND_DARK),
            SurfaceType::Dirt => (Palette::DIRT, Palette::DIRT_DARK),
            SurfaceType::Water => (Palette::WATER_BORDER, Palette::WATER),
            SurfaceType::Grass => (Palette::GRASS_DARK, Palette::CURB_RED),
            SurfaceType::Asphalt => (Palette::TIRE_SMOKE, Color::new(0.70, 0.70, 0.75, 0.35)),
            _ => (Palette::TIRE_SMOKE, Color::new(0.70, 0.70, 0.75, 0.35)),
        };

        let count = ((impact_speed * 1.5).ceil() as usize).clamp(8, 20);
        for _ in 0..count {
            if self.particles.len() >= self.max_particles {
                break;
            }

            let angle = self.rand_f32() * std::f32::consts::TAU;
            let dir = Vec2::new(angle.cos(), angle.sin());
            let speed = 2.0 + self.rand_f32() * 5.0 + impact_speed * 0.25;
            let p_vel = dir * speed;

            let life = 0.30 + self.rand_f32() * 0.35;
            let size = 0.14 + self.rand_f32() * 0.12;

            let use_secondary = self.rand_f32() > 0.5;
            let col = if use_secondary { col_var } else { base_col };
            let col_end = Color::new(col.r, col.g, col.b, 0.0);
            let p_pos = pos + dir * (self.rand_f32() * 0.4);

            self.particles.push(Particle {
                pos: p_pos,
                vel: p_vel,
                size_start: size,
                size_end: size * 1.8,
                color_start: col,
                color_end: col_end,
                lifetime: life,
                remaining_life: life,
                drag: 3.5,
                is_spark: false,
            });
        }
    }

    /// Updates particle physics, velocities, drag, and lifespans.
    pub fn update(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.particles.len() {
            let p = &mut self.particles[i];
            p.remaining_life -= dt;

            if p.remaining_life <= 0.0 {
                self.particles.swap_remove(i);
                continue;
            }

            // Apply velocity and aerodynamic drag
            p.pos += p.vel * dt;
            let drag_factor = (1.0 - p.drag * dt).max(0.0);
            p.vel *= drag_factor;

            i += 1;
        }
    }

    /// Renders all active particles.
    pub fn render(&self) {
        for p in &self.particles {
            let t = 1.0 - (p.remaining_life / p.lifetime).clamp(0.0, 1.0);
            let current_size = p.size_start + (p.size_end - p.size_start) * t;

            let r = p.color_start.r + (p.color_end.r - p.color_start.r) * t;
            let g = p.color_start.g + (p.color_end.g - p.color_start.g) * t;
            let b = p.color_start.b + (p.color_end.b - p.color_start.b) * t;
            let a = p.color_start.a + (p.color_end.a - p.color_start.a) * t;
            let col = Color::new(r, g, b, a);

            if p.is_spark {
                // Spark streak line in direction of velocity
                let tail = p.pos - p.vel * 0.025;
                draw_line(p.pos.x, p.pos.y, tail.x, tail.y, current_size * 1.5, col);
            } else {
                draw_circle(p.pos.x, p.pos.y, current_size, col);
            }
        }
    }
}
