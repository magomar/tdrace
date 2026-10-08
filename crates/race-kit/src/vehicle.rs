//! The interface a vehicle model gives to [`crate::RaceWorld`].
//!
//! Spec 056 (`specs/056_racekit_headless_race_world.md`). `impl Vehicle for Car` moves the
//! code of `RaceSession::physics_step` without change, so today's cars keep bit-identical
//! results (`tests/golden_world.rs`).

use arcade_race_core::track::{JumpRamp, JumpRampCarExt, SplineProjection, Track, Tree, TreeType};
use arcade_race_core::Body2D;
use glam::Vec2;
use wheelbase::{Car, CarControls, SurfaceType};

use crate::events::DnfCause;

/// Controls for one step. The alias keeps the `.tdr` replay format unchanged.
pub type DriveControls = CarControls;

/// One tree canopy that a vehicle brushed through in a step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanopyBrush {
    pub tree: TreeType,
    /// Vehicle position before the canopy drag.
    pub position: Vec2,
    /// Vehicle velocity after the canopy drag.
    pub velocity: Vec2,
}

/// A vehicle that [`crate::RaceWorld`] can step, collide and track.
pub trait Vehicle: Body2D {
    /// Surface under each of the four contact points, with the tracker distance as a hint.
    fn sample_surfaces(&self, track: &Track, hint: f32) -> [SurfaceType; 4];
    /// Slipstream intensity from the other vehicles.
    fn draft_intensity(&self, others: &[&Self]) -> f32;
    fn set_draft(&mut self, intensity: f32);
    /// Road elevation, bank, grade and track frame under the vehicle.
    fn set_road(&mut self, proj: &SplineProjection);
    fn step(&mut self, controls: &DriveControls, surfaces: [SurfaceType; 4], dt: f32);
    /// Tree canopy drag, right after `step`. Pushes one entry per brushed tree that emits
    /// foliage. The default does nothing.
    fn brush_canopy(&mut self, _trees: &[Tree], _dt: f32, _out: &mut Vec<CanopyBrush>) {}
    /// Jump ramp take-off and roll-off, after all vehicles have stepped. The default does nothing.
    fn step_ramps(&mut self, _ramps: &[JumpRamp], _dt: f32) {}
    /// Air time of a landing in this step, if there was one. The default never lands.
    fn landed(&self) -> Option<f32> {
        None
    }
    /// Called for each wall or vehicle impact. `Some` wrecks the vehicle. The default never wrecks.
    fn on_impact(&mut self, _impact_speed: f32) -> Option<DnfCause> {
        None
    }
    /// Resets mechanical tire wear across all wheels to 0.0. The default does nothing.
    fn service_tires(&mut self) {}
    /// Restores chassis health by amount (clamped to 1.0). Returns health gained. The default returns 0.0.
    fn apply_field_repair(&mut self, _amount: f32) -> f32 { 0.0 }
    /// Current chassis health [0.0..1.0]. The default returns 1.0.
    fn health(&self) -> f32 { 1.0 }
    /// Applies collision impact damage partitioned by impact zone and engine placement (Spec 078).
    fn apply_collision_damage(&mut self, _contact_point: Vec2, _damage_energy: f32) {}
    /// Current engine health [0.0..1.0]. The default returns 1.0.
    fn engine_health(&self) -> f32 { 1.0 }
    /// 4-corner suspension health [FL, FR, RL, RR] [0.0..1.0]. The default returns [1.0; 4].
    fn suspension_health(&self) -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }
    /// Maximum tire wear across all wheels [0.0..1.0]. The default returns 0.0.
    fn max_tire_wear(&self) -> f32 { 0.0 }
}

impl Vehicle for Car {
    #[inline]
    fn sample_surfaces(&self, track: &Track, hint: f32) -> [SurfaceType; 4] {
        track.sample_car_surfaces_with_hint(self, hint)
    }

    #[inline]
    fn draft_intensity(&self, others: &[&Self]) -> f32 {
        self.compute_draft_intensity(others)
    }

    #[inline]
    fn set_draft(&mut self, intensity: f32) {
        self.state.draft_intensity = intensity;
    }

    fn set_road(&mut self, proj: &SplineProjection) {
        self.state.road_elevation = proj.elevation;
        self.state.road_bank_angle = proj.bank_angle;
        self.state.road_grade_slope = proj.grade_slope;
        self.state.road_vertical_curvature = proj.vertical_curvature;
        self.state.track_right = Vec2::new(proj.tangent.y, -proj.tangent.x);
        self.state.track_forward = proj.tangent;
    }

    #[inline]
    fn step(&mut self, controls: &DriveControls, surfaces: [SurfaceType; 4], dt: f32) {
        self.step_per_wheel(controls, surfaces, dt);
    }

    fn brush_canopy(&mut self, trees: &[Tree], dt: f32, out: &mut Vec<CanopyBrush>) {
        // Soft tree canopy brush interaction: viscous foliage drag & leaf roost particles
        if !self.state.is_airborne && self.state.elevation < 0.6 {
            for tree in trees {
                let car_pos = self.state.position;
                if tree.contains_canopy(car_pos) && !tree.contains_trunk(car_pos) {
                    // Only ground-level shrubs (TreeType::Bush, which have no solid trunk)
                    // apply physical drag deceleration and roost particles at ground level.
                    // Tall trees with trunks (Oak, Pine, Palm, etc.) have elevated canopies
                    // that vehicles pass under without ground-level resistance.
                    if !tree.has_trunk() {
                        let drag_rate = tree.tree_type.canopy_drag_deceleration();
                        self.state.velocity *= (1.0 - drag_rate * dt).max(0.0);
                        self.state.speed = self.state.velocity.length();

                        if self.state.speed > 3.0 {
                            out.push(CanopyBrush { tree: tree.tree_type, position: car_pos, velocity: self.state.velocity });
                        }
                    }
                }
            }
        }
    }

    fn step_ramps(&mut self, ramps: &[JumpRamp], dt: f32) {
        let was_airborne = self.state.is_airborne;
        let mut on_any_ramp = false;

        if !was_airborne {
            for ramp in ramps {
                if ramp.contains(self.state.position) {
                    on_any_ramp = true;
                    if self.step_ramp_interaction(ramp, dt) {
                        break;
                    }
                }
            }
            if !on_any_ramp {
                if self.state.ramp_elevation > 0.10 {
                    // Rolled off an elevated ramp edge without launching at speed
                    self.state.elevation = self.state.ramp_elevation;
                    self.state.is_airborne = true;
                    self.state.vertical_velocity = 0.0;
                }
                self.state.ramp_elevation = 0.0;
            }
        }
    }

    #[inline]
    fn landed(&self) -> Option<f32> {
        self.state.just_landed.then_some(self.state.last_air_time)
    }

    #[inline]
    fn service_tires(&mut self) {
        self.service_tires();
    }

    #[inline]
    fn apply_field_repair(&mut self, amount: f32) -> f32 {
        self.apply_field_repair(amount)
    }

    #[inline]
    fn health(&self) -> f32 {
        self.state.health
    }

    #[inline]
    fn apply_collision_damage(&mut self, contact_point: Vec2, damage_energy: f32) {
        self.apply_collision_damage(contact_point, damage_energy);
    }

    #[inline]
    fn engine_health(&self) -> f32 {
        self.state.engine_health
    }

    #[inline]
    fn suspension_health(&self) -> [f32; 4] {
        self.state.suspension_health
    }

    #[inline]
    fn max_tire_wear(&self) -> f32 {
        self.state.wheel_assemblies.iter().map(|w| w.wear).fold(0.0f32, f32::max)
    }
}
