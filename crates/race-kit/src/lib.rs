//! # race-kit
//!
//! Headless race world for top-down racing games: one race step, finish order and real
//! finish times, wrecks and DNF, and race events instead of side effects.
//! Spec 056 (`specs/056_racekit_headless_race_world.md`).
//!
//! No window, sound, database or clock: the crate builds for `wasm32-unknown-unknown`.

pub mod ai;
pub mod events;
pub mod vehicle;
pub mod world;

pub use events::{DnfCause, RaceEvent};
pub use vehicle::{CanopyBrush, DriveControls, Vehicle};
pub use world::{
    CollisionParams, FinishState, JokerRule, ParticipantResult, PitServiceState, RaceFormat, RaceRules, RaceWorld,
    StageFinish, StageOutcome,
};

/// Compiles the README code as a doc test.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
