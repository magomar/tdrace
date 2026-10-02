//! Rigid-body interface for collision, LIDAR, progress tracking and pit checks.
//!
//! Spec 054 (`specs/054_body2d_trait_for_vehiclegeneric_collision_and_progress.md`). The
//! engine functions in this crate read only a body's pose, motion, mass, height and hull,
//! and write only three raw adders. Any vehicle model (a car, a chariot team) can plug in
//! by implementing [`Body2D`].

use glam::Vec2;
use wheelbase::Car;

/// Collision hull measured from the centre of gravity, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyHull {
    /// Distance from the centre of gravity to the front edge.
    pub front: f32,
    /// Distance from the centre of gravity to the rear edge.
    pub rear: f32,
    /// Half of the hull width.
    pub half_width: f32,
}

impl BodyHull {
    /// Margin that [`crate::collision::OrientedBox::from_body`] adds on every side.
    pub const MARGIN: f32 = 0.15;

    /// Distance from the centre of gravity to the farthest hull corner, margin included.
    pub fn reach(&self) -> f32 {
        let long = self.front.max(self.rear) + Self::MARGIN;
        let wide = self.half_width + Self::MARGIN;
        (long * long + wide * wide).sqrt()
    }
}

/// A rigid body the engine can collide, scan and track.
///
/// Implementations must return stored state, not recomputed values: results are pinned
/// bit-for-bit by the golden tests (`tests/golden_sim.rs`).
pub trait Body2D {
    fn position(&self) -> Vec2;
    /// Heading in radians.
    fn angle(&self) -> f32;
    fn velocity(&self) -> Vec2;
    fn angular_velocity(&self) -> f32;
    /// Cached scalar speed in m/s.
    fn speed(&self) -> f32;
    fn mass(&self) -> f32;
    fn inertia(&self) -> f32;
    /// Height of a jump above the surface, in metres.
    fn jump_height(&self) -> f32;
    /// Road, ramp and jump height combined, in metres.
    fn total_elevation(&self) -> f32;
    fn forward_vector(&self) -> Vec2;
    fn hull(&self) -> BodyHull;
    fn translate(&mut self, d: Vec2);
    fn add_velocity(&mut self, d: Vec2);
    fn add_angular_velocity(&mut self, d: f32);
}

impl Body2D for Car {
    #[inline]
    fn position(&self) -> Vec2 {
        self.state.position
    }
    #[inline]
    fn angle(&self) -> f32 {
        self.state.angle
    }
    #[inline]
    fn velocity(&self) -> Vec2 {
        self.state.velocity
    }
    #[inline]
    fn angular_velocity(&self) -> f32 {
        self.state.angular_velocity
    }
    #[inline]
    fn speed(&self) -> f32 {
        self.state.speed
    }
    #[inline]
    fn mass(&self) -> f32 {
        self.config.mass
    }
    #[inline]
    fn inertia(&self) -> f32 {
        self.config.inertia
    }
    #[inline]
    fn jump_height(&self) -> f32 {
        self.state.elevation
    }
    #[inline]
    fn total_elevation(&self) -> f32 {
        Car::total_elevation(self)
    }
    #[inline]
    fn forward_vector(&self) -> Vec2 {
        Car::forward_vector(self)
    }
    #[inline]
    fn hull(&self) -> BodyHull {
        let (front, rear, half_width) = self
            .config
            .chassis
            .to_body_hull(self.config.cg_to_front, self.config.cg_to_rear);
        BodyHull {
            front,
            rear,
            half_width,
        }
    }
    #[inline]
    fn translate(&mut self, d: Vec2) {
        self.state.position += d;
    }
    #[inline]
    fn add_velocity(&mut self, d: Vec2) {
        self.state.velocity += d;
        self.state.speed = self.state.velocity.length();
    }
    #[inline]
    fn add_angular_velocity(&mut self, d: f32) {
        self.state.angular_velocity += d;
    }
}
