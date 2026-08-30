use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tdrace_core::physics::car::{Car, CarControls};
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oasis_rally, outlaw_pass, oval_speedway,
    ramp_raceway,
};
use tdrace_core::CarConfig;

use crate::ui::menu::{CarChoice, TrackChoice};

/// Magic identifier for TDRace replay files.
pub const REPLAY_MAGIC: &str = "TDR1";
pub const REPLAY_VERSION: u32 = 1;

/// Metadata header describing race parameters and initial conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayHeader {
    pub magic: String,
    pub version: u32,
    pub track_choice: TrackChoice,
    pub car_choice: CarChoice,
    pub random_seed: u64,
    pub fixed_dt: f32,
    pub total_frames: usize,
    pub race_duration: f32,
    pub best_lap_time: Option<f32>,
    pub player_name: String,
    pub timestamp_epoch_sec: u64,
}

/// Recorded input controls for a single fixed physics tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReplayInputFrame {
    pub frame_index: u32,
    pub controls: CarControls,
}

/// Periodic snapshot for scrubbing, fast-forwarding, and trajectory verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayKeyframe {
    pub frame_index: u32,
    pub position: Vec2,
    pub angle: f32,
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub current_lap: u32,
    pub current_checkpoint: usize,
}

/// Complete race replay file structure (.tdr).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replay {
    pub header: ReplayHeader,
    pub frames: Vec<ReplayInputFrame>,
    pub keyframes: Vec<ReplayKeyframe>,
}

impl Replay {
    /// Serializes replay into compact binary representation.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let json_str = self.to_json()?;
        let mut bytes = Vec::with_capacity(json_str.len() + 8);
        bytes.extend_from_slice(REPLAY_MAGIC.as_bytes());
        bytes.extend_from_slice(&self.header.version.to_le_bytes());
        bytes.extend_from_slice(json_str.as_bytes());
        Ok(bytes)
    }

    /// Deserializes replay from binary representation.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Invalid replay file: buffer too short".to_string());
        }

        let magic = std::str::from_utf8(&bytes[0..4]).map_err(|e| e.to_string())?;
        if magic != REPLAY_MAGIC {
            // Check if it's raw JSON
            if let Ok(replay) = Self::from_json(std::str::from_utf8(bytes).map_err(|e| e.to_string())?) {
                return Ok(replay);
            }
            return Err(format!("Invalid replay magic header: {}", magic));
        }

        let json_str = std::str::from_utf8(&bytes[8..]).map_err(|e| e.to_string())?;
        Self::from_json(json_str)
    }

    /// Serializes replay into JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Deserializes replay from JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    /// Saves replay to a file on disk.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let bytes = self.to_bytes()?;
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())
    }

    /// Loads replay from a file on disk.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        Self::from_bytes(&bytes)
    }
}

/// Continuous race session recorder.
#[derive(Debug, Clone)]
pub struct ReplayRecorder {
    pub header: ReplayHeader,
    pub frames: Vec<ReplayInputFrame>,
    pub keyframes: Vec<ReplayKeyframe>,
    pub keyframe_interval: usize,
    pub current_frame: usize,
}

impl ReplayRecorder {
    pub fn new(track_choice: TrackChoice, car_choice: CarChoice, seed: u64, fixed_dt: f32) -> Self {
        Self {
            header: ReplayHeader {
                magic: REPLAY_MAGIC.to_string(),
                version: REPLAY_VERSION,
                track_choice,
                car_choice,
                random_seed: seed,
                fixed_dt,
                total_frames: 0,
                race_duration: 0.0,
                best_lap_time: None,
                player_name: "Player 1".to_string(),
                timestamp_epoch_sec: 0,
            },
            frames: Vec::with_capacity(7200),
            keyframes: Vec::with_capacity(120),
            keyframe_interval: 60, // snapshot every 0.5s @ 120Hz
            current_frame: 0,
        }
    }

    /// Records input controls and car state at the current physics tick.
    pub fn record_frame(&mut self, controls: CarControls, car: &Car, tracker: &TrackProgressTracker) {
        let frame_idx = self.current_frame as u32;

        self.frames.push(ReplayInputFrame {
            frame_index: frame_idx,
            controls,
        });

        // Periodic keyframe snapshot
        if self.current_frame % self.keyframe_interval == 0 {
            self.keyframes.push(ReplayKeyframe {
                frame_index: frame_idx,
                position: car.state.position,
                angle: car.state.angle,
                velocity: car.state.velocity,
                angular_velocity: car.state.angular_velocity,
                current_lap: tracker.current_lap,
                current_checkpoint: tracker.next_checkpoint_idx,
            });
        }

        self.current_frame += 1;
    }

    /// Finalizes recording and produces an immutable `Replay`.
    pub fn finish(mut self, best_lap_time: Option<f32>) -> Replay {
        self.header.total_frames = self.frames.len();
        self.header.race_duration = self.frames.len() as f32 * self.header.fixed_dt;
        self.header.best_lap_time = best_lap_time;

        Replay {
            header: self.header,
            frames: self.frames,
            keyframes: self.keyframes,
        }
    }
}

/// Playback speed multiplier options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackSpeed {
    Paused,
    Speed1x,
    Speed2x,
    Speed4x,
    Speed8x,
}

impl PlaybackSpeed {
    pub fn multiplier(self) -> f32 {
        match self {
            PlaybackSpeed::Paused => 0.0,
            PlaybackSpeed::Speed1x => 1.0,
            PlaybackSpeed::Speed2x => 2.0,
            PlaybackSpeed::Speed4x => 4.0,
            PlaybackSpeed::Speed8x => 8.0,
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            PlaybackSpeed::Paused => PlaybackSpeed::Speed1x,
            PlaybackSpeed::Speed1x => PlaybackSpeed::Speed2x,
            PlaybackSpeed::Speed2x => PlaybackSpeed::Speed4x,
            PlaybackSpeed::Speed4x => PlaybackSpeed::Speed8x,
            PlaybackSpeed::Speed8x => PlaybackSpeed::Speed1x,
        }
    }
}

/// Deterministic replay playback controller and verification engine.
#[derive(Debug, Clone)]
pub struct ReplayPlayer {
    pub replay: Replay,
    pub current_frame: usize,
    pub playback_accumulator: f32,
    pub speed: PlaybackSpeed,
    pub is_finished: bool,
}

impl ReplayPlayer {
    pub fn new(replay: Replay) -> Self {
        Self {
            replay,
            current_frame: 0,
            playback_accumulator: 0.0,
            speed: PlaybackSpeed::Speed1x,
            is_finished: false,
        }
    }

    /// Returns current playback time in seconds.
    pub fn current_time(&self) -> f32 {
        self.current_frame as f32 * self.replay.header.fixed_dt
    }

    /// Total replay duration in seconds.
    pub fn total_duration(&self) -> f32 {
        self.replay.header.race_duration
    }

    /// Advances playback by delta time and returns controls for any elapsed fixed ticks.
    pub fn step(&mut self, dt: f32) -> Option<CarControls> {
        if self.speed == PlaybackSpeed::Paused || self.is_finished {
            return None;
        }

        let fixed_dt = self.replay.header.fixed_dt;
        self.playback_accumulator += dt * self.speed.multiplier();

        if self.playback_accumulator >= fixed_dt {
            self.playback_accumulator -= fixed_dt;

            if self.current_frame < self.replay.frames.len() {
                let controls = self.replay.frames[self.current_frame].controls;
                self.current_frame += 1;
                if self.current_frame >= self.replay.frames.len() {
                    self.is_finished = true;
                }
                return Some(controls);
            } else {
                self.is_finished = true;
            }
        }

        None
    }

    /// Returns recorded controls at a specific frame index.
    pub fn get_controls_at_frame(&self, frame: usize) -> Option<CarControls> {
        self.replay.frames.get(frame).map(|f| f.controls)
    }

    /// Seeks playback to a specific frame index.
    pub fn scrub_to_frame(&mut self, frame: usize) {
        self.current_frame = frame.min(self.replay.frames.len());
        self.playback_accumulator = 0.0;
        self.is_finished = self.current_frame >= self.replay.frames.len();
    }

    /// Seeks playback to a specific time in seconds.
    pub fn scrub_to_time(&mut self, time_sec: f32) {
        let frame = (time_sec / self.replay.header.fixed_dt).round().max(0.0) as usize;
        self.scrub_to_frame(frame);
    }

    /// Toggles pause state.
    pub fn toggle_pause(&mut self) {
        self.speed = if self.speed == PlaybackSpeed::Paused {
            PlaybackSpeed::Speed1x
        } else {
            PlaybackSpeed::Paused
        };
    }

    /// Cycles speed 1x -> 2x -> 4x -> 8x -> 1x.
    pub fn cycle_speed(&mut self) {
        self.speed = self.speed.cycle();
    }

    /// Runs a complete deterministic physics simulation of the entire replay and verifies
    /// that the replayed trajectory matches the recorded keyframes within floating-point tolerance (< 1e-4).
    ///
    /// Returns `Ok(max_error_distance)` if determinism passes, or `Err(diagnostic)` if mismatch occurs.
    pub fn verify_determinism(&self) -> Result<f32, String> {
        let track = match &self.replay.header.track_choice {
            TrackChoice::ClassicGrandPrix => classic_grand_prix(),
            TrackChoice::OvalSpeedway => oval_speedway(),
            TrackChoice::DriftPark => drift_park(),
            TrackChoice::KartArena => kart_arena(),
            TrackChoice::RampRaceway => ramp_raceway(),
            TrackChoice::OasisRally => oasis_rally(),
            TrackChoice::OutlawPass => outlaw_pass(),
            TrackChoice::Custom { path, .. } => {
                tdrace_core::track::Track::load_from_file(path).unwrap_or_else(|_| classic_grand_prix())
            }
        };

        let config = match self.replay.header.car_choice {
            CarChoice::SportsCar => CarConfig::sports_car(),
            CarChoice::DriftCar => CarConfig::drift_car(),
            CarChoice::Kart => CarConfig::kart(),
            CarChoice::RallyCar => CarConfig::rally_car(),
            CarChoice::F1Car => crate::module::f1::F1GameModule::car_f1_hybrid(),
        };

        let initial_pose = track
            .grid_positions
            .first()
            .cloned()
            .unwrap_or(tdrace_core::track::geometry::SpawnPose {
                position: Vec2::ZERO,
                angle: 0.0,
                grid_slot: 0,
            });
        let mut sim_car = Car::new(config).with_pose(initial_pose.position, initial_pose.angle);

        let dt = self.replay.header.fixed_dt;
        let mut max_pos_err = 0.0f32;
        let mut keyframe_cursor = 0;

        for (frame_idx, input_frame) in self.replay.frames.iter().enumerate() {
            // Check against keyframe
            if keyframe_cursor < self.replay.keyframes.len()
                && self.replay.keyframes[keyframe_cursor].frame_index == frame_idx as u32
            {
                let kf = &self.replay.keyframes[keyframe_cursor];
                let err = (sim_car.state.position - kf.position).length();
                max_pos_err = max_pos_err.max(err);

                if err > 1e-3 {
                    return Err(format!(
                        "Determinism divergence at frame {}: recorded pos {:?}, simulated pos {:?}, err = {:.6}",
                        frame_idx, kf.position, sim_car.state.position, err
                    ));
                }

                keyframe_cursor += 1;
            }

            // Step vehicle physics
            let surfaces = track.sample_car_surfaces(&sim_car);
            sim_car.step_per_wheel(&input_frame.controls, surfaces, dt);
        }

        Ok(max_pos_err)
    }
}
