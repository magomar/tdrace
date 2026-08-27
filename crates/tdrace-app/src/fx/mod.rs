pub mod drift_popup;
pub mod particles;
pub mod skidmarks;

pub use drift_popup::{DriftPopup, DriftPopupManager};
pub use particles::{Particle, ParticleSystem};
pub use skidmarks::{SkidSegment, SkidmarkBuffer};

use tdrace_core::collision::car_collision::CarCarCollisionEvent;
use tdrace_core::collision::wall::WallCollisionEvent;
use tdrace_core::physics::car::Car;
use tdrace_core::physics::surface::SurfaceType;

/// Unified visual effects coordinator managing skidmarks, smoke, dirt, collision sparks, and drift popups.
#[derive(Debug, Clone)]
pub struct EffectsManager {
    pub skidmarks: SkidmarkBuffer,
    pub particles: ParticleSystem,
    pub drift_popups: DriftPopupManager,
    prev_drifting: Vec<bool>,
}

impl EffectsManager {
    pub fn new(max_skidmarks: usize, max_particles: usize) -> Self {
        Self {
            skidmarks: SkidmarkBuffer::new(max_skidmarks),
            particles: ParticleSystem::new(max_particles),
            drift_popups: DriftPopupManager::new(32),
            prev_drifting: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.skidmarks.clear();
        self.particles.clear();
        self.drift_popups.clear();
        self.prev_drifting.clear();
    }

    /// Updates all visual effects for the current physics simulation step.
    pub fn update(
        &mut self,
        cars: &[Car],
        surfaces: &[[SurfaceType; 4]],
        wall_collisions: &[WallCollisionEvent],
        car_collisions: &[CarCarCollisionEvent],
        dt: f32,
    ) {
        // 1. Skidmarks buffer update
        self.skidmarks.update_for_cars(cars, surfaces);

        // 2. Tire smoke and off-track dirt particle emission
        if self.prev_drifting.len() < cars.len() {
            self.prev_drifting.resize(cars.len(), false);
        }

        for (i, car) in cars.iter().enumerate() {
            let wheel_pos = car.wheel_positions_world();
            let car_surfaces = surfaces.get(i).copied().unwrap_or([SurfaceType::Asphalt; 4]);

            for w in 0..4 {
                let telemetry = &car.state.wheels[w];
                let pos = wheel_pos[w];
                let surf = car_surfaces[w];

                // Tire smoke on asphalt/curb
                if (surf == SurfaceType::Asphalt || surf == SurfaceType::Curb)
                    && telemetry.skid_intensity > 0.25
                    && car.state.speed > 3.0
                {
                    self.particles
                        .emit_tire_smoke(pos, car.state.velocity, telemetry.skid_intensity);
                }

                // Off-track & dirt track roost
                if (surf == SurfaceType::Grass || surf == SurfaceType::Sand || surf == SurfaceType::Dirt)
                    && (telemetry.skid_intensity > 0.10 || telemetry.slip_ratio.abs() > 0.15)
                    && car.state.speed > 2.0
                {
                    self.particles.emit_dirt_roost(pos, surf, car.state.velocity);
                }

                // Water splash on puddles / water hazard
                if surf == SurfaceType::Water && car.state.speed > 1.5 {
                    self.particles.emit_water_splash(pos, car.state.velocity, car.state.speed);
                }
            }

            // Drift score popups when ending a drift
            let was_drifting = self.prev_drifting[i];
            let is_drifting = car.state.is_drifting;
            if was_drifting && !is_drifting && car.state.drift_score > 50.0 {
                self.drift_popups.spawn_drift_score(
                    car.state.position,
                    car.state.drift_score,
                    1.0 + (car.state.drift_score / 500.0).min(2.0),
                );
            }
            self.prev_drifting[i] = is_drifting;
        }

        // 3. Collision sparks for wall impacts
        for ev in wall_collisions {
            if ev.impact_speed > 2.5 {
                self.particles
                    .emit_sparks(ev.contact_point, ev.normal, ev.impact_speed);
            }
        }

        // 4. Collision sparks for car-car collisions
        for ev in car_collisions {
            if ev.closing_speed > 2.5 {
                self.particles
                    .emit_sparks(ev.contact_point, ev.normal, ev.closing_speed);
            }
        }

        // 5. Particle system and drift popups physics step
        self.particles.update(dt);
        self.drift_popups.update(dt);
    }

    /// Renders skidmarks in the ground pass.
    pub fn render_ground_fx(&self) {
        self.skidmarks.render();
    }

    /// Renders airborne particles and drift popups.
    pub fn render_airborne_fx(&self) {
        self.particles.render();
        self.drift_popups.render_in_world();
    }
}
