//! # Arcade Race Core
//!
//! Reusable 2D/2.5D course geometry, continuous SAT collision resolution,
//! directional checkpoints, sector timing, and racing LIDAR perception.

pub mod body;
pub mod car_category;
pub mod collision;
pub mod lidar;
pub mod profile;
pub mod track;

pub use body::{Body2D, BodyHull};
pub use car_category::*;
pub use collision::*;
pub use lidar::*;
pub use profile::*;
pub use track::*;

pub use glam::Vec2;
