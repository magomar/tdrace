//! TDRace Core: High-performance deterministic top-down arcade vehicle physics,
//! track geometry, collision resolution, and LIDAR simulation engine.

pub mod collision;
pub mod lidar;
pub mod physics;
pub mod track;

pub use collision::*;
pub use glam::Vec2;
pub use lidar::*;
pub use physics::*;
pub use track::*;
