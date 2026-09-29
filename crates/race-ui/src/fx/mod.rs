pub mod drift_popup;
pub mod particles;
pub mod skidmarks;

pub use drift_popup::{DriftPopup, DriftPopupManager};
pub use particles::{Particle, ParticleSystem};
pub use skidmarks::{SkidSegment, SkidmarkBuffer};

use arcade_race_core::collision::car_collision::CarCarCollisionEvent;
use arcade_race_core::collision::wall::WallCollisionEvent;
use arcade_race_core::track::geometry::BarrierType;
use arcade_race_core::Body2D;
use glam::Vec2;
use wheelbase::car::Car;
use wheelbase::surface::SurfaceType;
use wheelbase::WheelTelemetry;

/// What the effects need from a vehicle besides its [`Body2D`] state. Spec 061.
pub trait FxVehicle: Body2D {
    /// Four ground contact points: the wheels of a car; for a chariot, e.g. its two wheels and
    /// two hoof groups.
    fn contact_points(&self) -> [Vec2; 4];
    /// Slip and skid data for each contact point, in the order of [`FxVehicle::contact_points`].
    fn contact_telemetry(&self) -> &[WheelTelemetry; 4];
    /// Unit vector to the vehicle's right.
    fn right_vector(&self) -> Vec2;
    fn is_airborne(&self) -> bool;
    fn is_drifting(&self) -> bool {
        false
    }
    fn drift_score(&self) -> f32 {
        0.0
    }
}

impl FxVehicle for Car {
    #[inline]
    fn contact_points(&self) -> [Vec2; 4] {
        self.wheel_positions_world()
    }
    #[inline]
    fn contact_telemetry(&self) -> &[WheelTelemetry; 4] {
        &self.state.wheels
    }
    #[inline]
    fn right_vector(&self) -> Vec2 {
        Car::right_vector(self)
    }
    #[inline]
    fn is_airborne(&self) -> bool {
        self.state.is_airborne
    }
    #[inline]
    fn is_drifting(&self) -> bool {
        self.state.is_drifting
    }
    #[inline]
    fn drift_score(&self) -> f32 {
        self.state.drift_score
    }
}

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

    /// Creates an effects manager with persistent skidmark retention across the entire race.
    pub fn new_persistent(max_particles: usize) -> Self {
        Self {
            skidmarks: SkidmarkBuffer::new_persistent(),
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
    pub fn update<V: FxVehicle>(
        &mut self,
        cars: &[V],
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
            // Suppress ground wheel particles while car is airborne / jumping
            if !car.is_airborne() && car.jump_height() <= 0.0 {
                let wheel_pos = car.contact_points();
                let car_surfaces = surfaces.get(i).copied().unwrap_or([SurfaceType::Asphalt; 4]);

                for w in 0..4 {
                    let telemetry = &car.contact_telemetry()[w];
                    let pos = wheel_pos[w];
                    let surf = car_surfaces[w];

                    // Tire smoke on asphalt/curb/concrete
                    if (surf == SurfaceType::Asphalt || surf == SurfaceType::Curb || surf == SurfaceType::Concrete)
                        && telemetry.skid_intensity > 0.25
                        && car.speed() > 3.0
                    {
                        self.particles
                            .emit_tire_smoke(pos, car.velocity(), telemetry.skid_intensity);
                    }

                    // Debris particle roost on loose / deformable terrain (Gravel, Mud, Snow, Dirt, Sand, Grass)
                    if surf.produces_debris_particles()
                        && (telemetry.skid_intensity > 0.08 || telemetry.slip_ratio.abs() > 0.12 || telemetry.slip_angle.abs() > 0.08)
                        && car.speed() > 1.5
                    {
                        let roost_intensity = telemetry
                            .skid_intensity
                            .max(telemetry.slip_ratio.abs())
                            .max(telemetry.slip_angle.abs());
                        self.particles
                            .emit_dirt_roost(pos, surf, car.velocity(), roost_intensity);
                    }

                    // Water splash on puddles / water hazard
                    if surf == SurfaceType::Water && car.speed() > 1.5 {
                        self.particles.emit_water_splash(pos, car.velocity(), car.speed());
                    }
                }
            }

            // Drift score popups when ending a drift
            let was_drifting = self.prev_drifting[i];
            let is_drifting = car.is_drifting();
            if was_drifting && !is_drifting && car.drift_score() > 50.0 {
                self.drift_popups.spawn_drift_score(
                    car.position(),
                    car.drift_score(),
                    1.0 + (car.drift_score() / 500.0).min(2.0),
                );
            }
            self.prev_drifting[i] = is_drifting;
        }

        // 3. Collision sparks and particles for wall impacts
        for ev in wall_collisions {
            if ev.impact_speed > 2.5 {
                match ev.barrier_type {
                    BarrierType::Steel => {
                        self.particles
                            .emit_sparks(ev.contact_point, ev.normal, ev.impact_speed * 1.3);
                    }
                    BarrierType::TireWall => {
                        self.particles.emit_tire_smoke(
                            ev.contact_point,
                            ev.normal * -2.0,
                            (ev.impact_speed / 10.0).clamp(0.4, 1.2),
                        );
                    }
                    BarrierType::Concrete | BarrierType::CurbWall => {
                        self.particles
                            .emit_sparks(ev.contact_point, ev.normal, ev.impact_speed);
                    }
                    BarrierType::Virtual => {}
                }
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

    /// Renders skidmarks in the ground pass with camera viewport culling.
    pub fn render_ground_fx_culled(&self, view_bounds: Option<(glam::Vec2, glam::Vec2)>) {
        self.skidmarks.render_culled(view_bounds);
    }

    /// Renders skidmarks in the ground pass.
    pub fn render_ground_fx(&self) {
        self.render_ground_fx_culled(None);
    }

    /// Renders off-track roost particles under the cars.
    pub fn render_ground_debris(&self) {
        self.particles.render_ground();
    }

    /// Renders airborne particles and drift popups.
    pub fn render_airborne_fx(&self) {
        self.particles.render();
        self.drift_popups.render_in_world();
    }
}
