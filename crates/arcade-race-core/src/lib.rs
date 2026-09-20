//! # Arcade Race Core
//!
//! Reusable 2D/2.5D course geometry, continuous SAT collision resolution,
//! directional checkpoints, sector timing, and racing LIDAR perception.

pub mod car_category;
pub mod collision;
pub mod lidar;
pub mod track;

pub use car_category::*;
pub use collision::*;
pub use lidar::*;
pub use track::*;

pub use glam::Vec2;
