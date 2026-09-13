//! TDRace Core: High-performance deterministic top-down arcade vehicle physics,
//! track geometry, collision resolution, and LIDAR simulation engine.
//!
//! This crate is a unified facade combining:
//! - [`wheelbase`]: Dedicated 2D/2.5D wheeled vehicle dynamics and Pacejka tire solver.
//! - [`arcade_race_core`]: 2D course geometry, continuous SAT collisions, sector gates, and racing LIDAR.

pub use arcade_race_core::collision;
pub use arcade_race_core::lidar;
pub use arcade_race_core::track;
pub use wheelbase as physics;

pub use arcade_race_core::*;
pub use wheelbase::*;
