//! Spatial proximity audio calculations, distance attenuation, stereo panning,
//! and Doppler shift frequency modulation for nearby on-track vehicles.

use glam::Vec2;
use crate::audio::manager::EngineSoundType;

pub const DEFAULT_MIN_DISTANCE: f32 = 2.5;
pub const DEFAULT_MAX_DISTANCE: f32 = 75.0;
pub const DEFAULT_PAN_RADIUS: f32 = 35.0;
pub const DEFAULT_OPPONENT_GAIN_CEILING: f32 = 0.65;
pub const DEFAULT_SPEED_OF_SOUND: f32 = 160.0;
pub const DEFAULT_MIN_DOPPLER: f32 = 0.65;
pub const DEFAULT_MAX_DOPPLER: f32 = 1.45;
pub const MAX_PROXIMITY_VOICES: usize = 3;
pub const PROXIMITY_HYSTERESIS: f32 = 3.0;

/// Snapshot of vehicle physics and powertrain state for spatial audio processing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleAudioSource {
    pub vehicle_id: usize,
    pub engine_type: EngineSoundType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub forward_speed: f32,
    pub throttle: f32,
    pub slip_ratio: f32,
}

/// Dynamic Doppler calculation state and configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DopplerConfig {
    pub speed_of_sound: f32,
    pub min_multiplier: f32,
    pub max_multiplier: f32,
    pub enabled: bool,
}

impl Default for DopplerConfig {
    fn default() -> Self {
        Self {
            speed_of_sound: DEFAULT_SPEED_OF_SOUND,
            min_multiplier: DEFAULT_MIN_DOPPLER,
            max_multiplier: DEFAULT_MAX_DOPPLER,
            enabled: true,
        }
    }
}

/// Evaluated spatial audio results for an emitter source relative to the listener.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialAudioResult {
    pub distance: f32,
    pub gain: f32,
    pub pan: f32,
    pub doppler_factor: f32,
}

/// Evaluates distance attenuation using a smooth quadratic roll-off envelope.
///
/// If distance <= `min_dist`, gain is 1.0 (saturation boundary).
/// If distance >= `max_dist`, gain is 0.0 (inaudible cutoff).
/// In between, gain smoothly diminishes quadratically: `((max_dist - d) / (max_dist - min_dist))^2`.
pub fn calculate_distance_attenuation(distance: f32, min_dist: f32, max_dist: f32) -> f32 {
    if distance <= min_dist {
        1.0
    } else if distance >= max_dist {
        0.0
    } else {
        let span = (max_dist - min_dist).max(0.001);
        let normalized = (max_dist - distance) / span;
        normalized.clamp(0.0, 1.0).powi(2)
    }
}

/// Calculates horizontal stereo pan in [-1.0, 1.0] from relative X coordinate.
///
/// -1.0 = hard left, 0.0 = center, +1.0 = hard right.
pub fn calculate_stereo_pan(rel_pos_x: f32, pan_radius: f32) -> f32 {
    let r = pan_radius.abs().max(0.1);
    (rel_pos_x / r).clamp(-1.0, 1.0)
}

/// Calculates physical Doppler shift multiplier based on relative line-of-sight velocity.
///
/// Emitter at `source_pos` with velocity `source_vel`.
/// Listener at `listener_pos` with velocity `listener_vel`.
///
/// Uses arcade-tuned speed of sound with safety clamping against singularities.
pub fn calculate_doppler_factor(
    source_pos: Vec2,
    source_vel: Vec2,
    listener_pos: Vec2,
    listener_vel: Vec2,
    config: &DopplerConfig,
) -> f32 {
    if !config.enabled {
        return 1.0;
    }

    let delta = listener_pos - source_pos;
    let dist = delta.length();
    if dist < 0.01 {
        return 1.0;
    }

    // Unit line of sight vector pointing from source towards listener
    let los = delta / dist;

    // Relative velocity of source relative to listener
    let v_rel = source_vel - listener_vel;

    // Radial approach velocity along line of sight: > 0 means approaching, < 0 means receding
    let v_approach = v_rel.dot(los);

    // Clamp approach velocity to avoid division by zero or negative denominator
    let v_clamped = v_approach.min(0.60 * config.speed_of_sound);

    let raw_multiplier = config.speed_of_sound / (config.speed_of_sound - v_clamped);
    raw_multiplier.clamp(config.min_multiplier, config.max_multiplier)
}

/// Evaluates spatial proximity and Doppler parameters for a source relative to a listener.
pub fn calculate_spatial_audio(
    source: &VehicleAudioSource,
    listener_pos: Vec2,
    listener_vel: Vec2,
    doppler_cfg: &DopplerConfig,
) -> SpatialAudioResult {
    let delta = source.position - listener_pos;
    let dist = delta.length();
    let gain = calculate_distance_attenuation(dist, DEFAULT_MIN_DISTANCE, DEFAULT_MAX_DISTANCE);
    let pan = calculate_stereo_pan(delta.x, DEFAULT_PAN_RADIUS);
    let doppler_factor = calculate_doppler_factor(
        source.position,
        source.velocity,
        listener_pos,
        listener_vel,
        doppler_cfg,
    );

    SpatialAudioResult {
        distance: dist,
        gain,
        pan,
        doppler_factor,
    }
}

/// Identifies and sorts candidate audible sources by ascending distance within `max_dist`.
/// Returns the indices in `sources` of the top $K$ closest vehicles.
pub fn select_top_k_audible_sources(
    sources: &[VehicleAudioSource],
    listener_pos: Vec2,
    max_dist: f32,
    max_k: usize,
) -> Vec<usize> {
    let mut candidates: Vec<(usize, f32)> = sources
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let d = (s.position - listener_pos).length();
            (idx, d)
        })
        .filter(|(_, d)| *d < max_dist)
        .collect();

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(max_k);
    candidates.into_iter().map(|(idx, _)| idx).collect()
}
