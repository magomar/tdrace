//! Spatial proximity audio calculations, distance attenuation, stereo panning,
//! and Doppler shift frequency modulation for nearby on-track vehicles.

use glam::Vec2;
use crate::audio::manager::EngineSoundType;
use crate::audio::engine_mixer::EngineAudioMixer;
use crate::audio::samples::ArchetypeSampleBank;

pub const DEFAULT_MIN_DISTANCE: f32 = 2.5;
pub const DEFAULT_MAX_DISTANCE: f32 = 75.0;
pub const DEFAULT_PAN_RADIUS: f32 = 35.0;
pub const DEFAULT_OPPONENT_GAIN_CEILING: f32 = 0.65;
pub const DEFAULT_SPEED_OF_SOUND: f32 = 160.0;
pub const DEFAULT_MIN_DOPPLER: f32 = 0.65;
pub const DEFAULT_MAX_DOPPLER: f32 = 1.45;
pub const MAX_PROXIMITY_VOICES: usize = 3;
pub const PROXIMITY_HYSTERESIS: f32 = 3.0;

/// Pure engine RPM and transmission model calculating target RPM, gear transitions, and shift cooldowns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineRpmModel {
    pub current_rpm: f32,
    pub current_gear: usize,
    pub shift_cooldown: f32,
}

impl Default for EngineRpmModel {
    fn default() -> Self {
        Self {
            current_rpm: 1100.0,
            current_gear: 1,
            shift_cooldown: 0.0,
        }
    }
}

impl EngineRpmModel {
    pub fn update(&mut self, forward_speed: f32, throttle: f32, max_slip: f32, dt: f32) -> (f32, bool) {
        self.shift_cooldown = (self.shift_cooldown - dt).max(0.0);
        let speed_abs = forward_speed.abs();
        let is_reverse = forward_speed < -0.5 && throttle < 0.0;

        let (new_gear, target_rpm) = if is_reverse {
            let rpm = (1100.0 + (speed_abs / 12.0) * 5500.0).clamp(1100.0, 7200.0);
            (0, rpm)
        } else if speed_abs < 1.0 {
            // Stationary launch revs / idle
            let throttle_revs = if throttle > 0.05 {
                1100.0 + throttle * 5500.0
            } else {
                1100.0
            };
            (1, throttle_revs)
        } else {
            // 5-speed forward sequential transmission
            let (gear, base_rpm) = if speed_abs < 12.5 {
                (1, 1200.0 + (speed_abs / 12.5) * 5800.0)
            } else if speed_abs < 23.5 {
                (2, 3800.0 + ((speed_abs - 12.5) / 11.0) * 3400.0)
            } else if speed_abs < 35.5 {
                (3, 4200.0 + ((speed_abs - 23.5) / 12.0) * 3000.0)
            } else if speed_abs < 47.5 {
                (4, 4600.0 + ((speed_abs - 35.5) / 12.0) * 2600.0)
            } else {
                (5, 5000.0 + ((speed_abs - 47.5) / 16.0) * 2500.0)
            };

            // Wheelspin rev-flare (power drift / burnout)
            let slip_flare = if max_slip > 0.3 { (max_slip - 0.3) * 2500.0 } else { 0.0 };
            (gear, (base_rpm + slip_flare).clamp(1100.0, 7800.0))
        };

        let is_upshift = new_gear > self.current_gear && self.current_gear > 0 && self.shift_cooldown <= 0.0;
        if is_upshift {
            self.shift_cooldown = 0.22;
        }
        self.current_gear = new_gear;

        // Smooth RPM interpolation with realistic engine inertia
        let responsiveness = if target_rpm > self.current_rpm { 16.0 } else { 10.0 };
        self.current_rpm += (target_rpm - self.current_rpm) * (dt * responsiveness).min(1.0);

        (self.current_rpm, is_upshift)
    }
}

/// An allocated audio voice channel dedicated to a nearby opponent vehicle.
pub struct ProximityVoiceSlot {
    pub vehicle_id: Option<usize>,
    pub engine_type: EngineSoundType,
    pub mixer: EngineAudioMixer,
    pub rpm_model: EngineRpmModel,
    pub current_gain: f32,
    pub current_pan: f32,
    pub current_doppler: f32,
}

impl ProximityVoiceSlot {
    pub fn new(bank: ArchetypeSampleBank) -> Self {
        Self {
            vehicle_id: None,
            engine_type: EngineSoundType::Generic,
            mixer: EngineAudioMixer::new(bank),
            rpm_model: EngineRpmModel::default(),
            current_gain: 0.0,
            current_pan: 0.0,
            current_doppler: 1.0,
        }
    }
}

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

/// Evaluates dynamic engine warmup throttle for a vehicle during the race countdown.
///
/// Produces non-uniform, rhythmic accelerations (throttle blips) tailored per vehicle ID
/// to warm up engines and keep RPM revolutionized on the starting grid, building
/// into an electrifying pre-launch crescendo as the green light approaches.
pub fn calculate_countdown_warmup_throttle(vehicle_id: usize, remaining_sec: f32) -> f32 {
    let t = (10.0 - remaining_sec).max(0.0);

    // Deterministic per-vehicle rhythm and cadence parameters
    let freq = 1.10 + ((vehicle_id * 7 + 3) % 8) as f32 * 0.09;
    let phase = ((vehicle_id * 13 + 5) % 11) as f32 / 11.0;
    let peak_throttle = 0.78 + ((vehicle_id * 17 + 2) % 7) as f32 * 0.03;
    let base_floor = 0.22 + ((vehicle_id * 11 + 1) % 5) as f32 * 0.02;

    let cycle = (t * freq + phase).rem_euclid(1.0);

    // Asymmetric throttle blip pulse: rapid attack surge, natural decay, and warm idle dwell
    let mut pulse = if cycle < 0.28 {
        let k = cycle / 0.28;
        base_floor + (peak_throttle - base_floor) * k.powf(1.4)
    } else if cycle < 0.65 {
        let k = (cycle - 0.28) / (0.65 - 0.28);
        base_floor + (peak_throttle - base_floor) * (1.0 - k).powi(2)
    } else {
        base_floor
    };

    // Pre-launch staging crescendo: in the final 0.7s before green light, rev engines up to launch RPM
    if remaining_sec <= 0.70 {
        let progress = ((0.70 - remaining_sec) / 0.70).clamp(0.0, 1.0);
        let flutter = (t * 40.0 + vehicle_id as f32).sin() * 0.04;
        let launch_thr = 0.72 + 0.25 * progress + flutter;
        pulse = pulse.max(launch_thr);
    }

    pulse.clamp(0.0, 1.0)
}
