//! What happened in one [`crate::RaceWorld::step`], for sounds, effects and scoring.

use arcade_race_core::collision::{CarCarCollisionEvent, WallCollisionEvent};
use arcade_race_core::track::TreeType;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Why a vehicle did not finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DnfCause {
    /// An impact wrecked the vehicle.
    Impact,
}

/// One event of a step. [`crate::RaceWorld::step`] returns them in the order they happened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RaceEvent {
    /// A vehicle brushed through a tree canopy fast enough to throw foliage.
    CanopyBrush { car: usize, tree: TreeType, position: Vec2, velocity: Vec2 },
    /// A vehicle touched down after a jump. Position and speed are taken at landing.
    Landed { car: usize, air_time: f32, position: Vec2, speed: f32 },
    /// Two vehicles collided.
    VehicleImpact(CarCarCollisionEvent),
    /// A vehicle hit a wall or an obstacle.
    WallImpact { car: usize, event: WallCollisionEvent },
    /// A vehicle crossed the finish line for the last time.
    Finished { car: usize, position: usize, time: f32 },
    /// An impact wrecked a vehicle. It is out of the race from now on.
    Wrecked { car: usize, cause: DnfCause },
}
