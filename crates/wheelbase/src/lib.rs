//! # Wheelbase
//!
//! High-performance deterministic 2D/2.5D motorsport vehicle physics,
//! Pacejka Magic Formula tire dynamics, dynamic weight transfer, chassis electronic assists,
//! and terrain surface sampling for arcade and simulation racing games.

pub mod bike;
pub mod car;
pub mod config;
pub mod sim;
pub mod surface;
pub mod tire;

pub use bike::{Motorbike, MotorbikeConfig, MotorbikeControls, MotorbikeState};
pub use car::{normalize_angle, Car, CarControls, CarState, JumpRampProperties};
pub use config::{CarConfig, DriverAssistsConfig, TireConfig};
pub use surface::{SurfaceProperties, SurfaceSampler, SurfaceType, UniformSurface};
pub use tire::{
    compute_skid_telemetry, pacejka_lateral_force, solve_combined_slip_forces, WheelId,
    WheelTelemetry,
};

pub use glam::Vec2;

