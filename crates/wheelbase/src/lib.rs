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
pub use car::{normalize_angle, Car, CarControls, CarState, ImpactZone, JumpRampProperties};
pub use config::{
    default_wheel_assemblies, pacejka_peak_slip_angle_deg, CarConfig, ChassisSkeleton, DifferentialType,
    DriverAssistsConfig, EnginePlacement, PacejkaTireConfig, PlayerHandling, RearAxleTire,
    SuspensionArchetype, SuspensionConfig, SuspensionCornerConfig, TireConfig, WheelAssemblyConfig,
};
pub use surface::{CompoundId, SurfaceAffinityMap, SurfaceProperties, SurfaceSampler, SurfaceType, UniformSurface};
pub use tire::{
    combined_slip_forces, combined_slip_fx, compute_skid_telemetry, longitudinal_slip_stiffness,
    normalized_grip_curve, pacejka_lateral_force, solve_combined_slip_forces,
    tire_friction_envelope, TireCompoundConfig, WheelAssembly, WheelId, WheelTelemetry,
};

pub use glam::Vec2;

