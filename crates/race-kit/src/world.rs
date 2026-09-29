//! The race in progress: one step order for every game.
//!
//! Spec 056 (`specs/056_racekit_headless_race_world.md`). The step order is the one of
//! `RaceSession::physics_step` in `tdrace-app`, without sounds, camera or particles.

use std::cmp::Ordering;

use arcade_race_core::collision::{resolve_all_wall_collisions, resolve_multi_car_collisions};
use arcade_race_core::track::{Track, TrackProgressTracker};
use serde::{Deserialize, Serialize};
use wheelbase::{Car, SurfaceType};

use crate::events::{DnfCause, RaceEvent};
use crate::vehicle::{CanopyBrush, DriveControls, Vehicle};

/// How a race ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaceFormat {
    /// A vehicle finishes when it completes this many laps.
    Laps(u32),
    /// Nobody finishes. The player drives laps against the clock.
    TimeAttack,
}

/// Vehicle-vehicle collision solver settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CollisionParams {
    pub restitution: f32,
    pub friction: f32,
    pub iterations: usize,
}

impl Default for CollisionParams {
    fn default() -> Self {
        Self { restitution: 0.45, friction: 0.35, iterations: 3 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RaceRules {
    pub format: RaceFormat,
    pub collision: CollisionParams,
}

impl Default for RaceRules {
    fn default() -> Self {
        Self { format: RaceFormat::Laps(3), collision: CollisionParams::default() }
    }
}

/// Where a vehicle is in the race.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FinishState {
    Racing,
    /// `time` is the race time at the step the vehicle crossed the line. `position` starts at 1.
    Finished { time: f32, position: usize },
    /// `time` is the race time at the step the vehicle was wrecked.
    Dnf { time: f32, cause: DnfCause },
}

/// One row of [`RaceWorld::results`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticipantResult {
    /// Vehicle index in [`RaceWorld::vehicles`].
    pub car: usize,
    /// Position in the standings, starting at 1.
    pub position: usize,
    pub state: FinishState,
    /// Race time in seconds. See `projected`.
    pub time: f32,
    /// True when the vehicle was still racing: `time` is then an estimate from the distance it
    /// still has to drive at its average speed so far.
    pub projected: bool,
    pub best_lap: Option<f32>,
}

/// A race: vehicles, their trackers, their finish states and the race clock.
///
/// The world does not own the track. The caller passes the same `&Track` to each step.
/// `vehicles` and `trackers` must have the same length; [`RaceWorld::spawn`] keeps them so.
pub struct RaceWorld<V: Vehicle = Car> {
    pub rules: RaceRules,
    pub vehicles: Vec<V>,
    pub trackers: Vec<TrackProgressTracker>,
    pub finish: Vec<FinishState>,
    /// Highest speed of each vehicle, in m/s, read after the step and canopy drag.
    pub top_speed: Vec<f32>,
    /// Surfaces under each vehicle in the last step, for effects and sounds.
    pub last_surfaces: Vec<[SurfaceType; 4]>,
    /// Race time in seconds: the sum of every step's `dt`.
    pub time: f32,
    events: Vec<RaceEvent>,
    brushes: Vec<CanopyBrush>,
}

impl<V: Vehicle> RaceWorld<V> {
    pub fn new(rules: RaceRules) -> Self {
        Self {
            rules,
            vehicles: Vec::new(),
            trackers: Vec::new(),
            finish: Vec::new(),
            top_speed: Vec::new(),
            last_surfaces: Vec::new(),
            time: 0.0,
            events: Vec::new(),
            brushes: Vec::new(),
        }
    }

    /// Adds a vehicle and its tracker. Returns the vehicle index.
    pub fn spawn(&mut self, vehicle: V, tracker: TrackProgressTracker) -> usize {
        self.vehicles.push(vehicle);
        self.trackers.push(tracker);
        self.fit();
        self.vehicles.len() - 1
    }

    /// Removes every vehicle and resets the clock. The rules stay.
    pub fn clear(&mut self) {
        self.vehicles.clear();
        self.trackers.clear();
        self.finish.clear();
        self.top_speed.clear();
        self.last_surfaces.clear();
        self.time = 0.0;
        self.events.clear();
    }

    /// Keeps the per-vehicle state as long as `vehicles`, for callers that push vehicles directly.
    fn fit(&mut self) {
        let n = self.vehicles.len();
        self.finish.resize(n, FinishState::Racing);
        self.top_speed.resize(n, 0.0);
        self.last_surfaces.resize(n, [SurfaceType::Asphalt; 4]);
    }

    /// The events of the last step, in the order they happened.
    pub fn events(&self) -> &[RaceEvent] {
        &self.events
    }

    pub fn is_finished(&self, car: usize) -> bool {
        matches!(self.finish.get(car), Some(FinishState::Finished { .. }))
    }

    /// Advances the race by `dt` seconds. `controls[i]` drives vehicle `i`; a missing entry or a
    /// wrecked vehicle gets default (released) controls.
    pub fn step(&mut self, track: &Track, controls: &[DriveControls], dt: f32) -> &[RaceEvent] {
        self.events.clear();
        let n_cars = self.vehicles.len();
        if n_cars == 0 {
            return &self.events;
        }
        self.fit();

        // Sample surfaces under all wheels of all cars
        for (i, car) in self.vehicles.iter().enumerate() {
            let prog = self.trackers.get(i).map(|tp| tp.progress_distance).unwrap_or(0.0);
            self.last_surfaces[i] = car.sample_surfaces(track, prog);
        }

        // Aerodynamic slipstream wake drafting between cars
        let mut drafts = Vec::with_capacity(n_cars);
        for i in 0..n_cars {
            let other_refs: Vec<&V> =
                self.vehicles.iter().enumerate().filter(|(idx, _)| *idx != i).map(|(_, c)| c).collect();
            drafts.push(self.vehicles[i].draft_intensity(&other_refs));
        }
        for i in 0..n_cars {
            self.vehicles[i].set_draft(drafts[i]);
        }

        // Step individual vehicle dynamics and update road elevation & cross-slope banking
        for i in 0..n_cars {
            let prev_prog = self.trackers.get(i).map(|tp| tp.progress_distance).unwrap_or(0.0);
            let proj = track.spline.project_point_continuity(self.vehicles[i].position(), prev_prog, 50.0);
            self.vehicles[i].set_road(&proj);

            let ctrl = match self.finish[i] {
                FinishState::Dnf { .. } => DriveControls::default(),
                _ => controls.get(i).copied().unwrap_or_default(),
            };
            self.vehicles[i].step(&ctrl, self.last_surfaces[i], dt);

            self.brushes.clear();
            self.vehicles[i].brush_canopy(&track.geometry.trees, dt, &mut self.brushes);
            for b in &self.brushes {
                self.events.push(RaceEvent::CanopyBrush { car: i, tree: b.tree, position: b.position, velocity: b.velocity });
            }
        }

        for (i, car) in self.vehicles.iter().enumerate() {
            self.top_speed[i] = self.top_speed[i].max(car.speed());
        }

        // Continuous jump ramp traversal, lip takeoff & landing
        for (i, car) in self.vehicles.iter_mut().enumerate() {
            car.step_ramps(&track.geometry.jump_ramps, dt);
            if let Some(air_time) = car.landed() {
                self.events.push(RaceEvent::Landed { car: i, air_time, position: car.position(), speed: car.speed() });
            }
        }

        // Car-to-car collisions with momentum exchange and penetration pushback
        if n_cars > 1 && self.rules.collision.iterations > 0 {
            let c = self.rules.collision;
            for ev in resolve_multi_car_collisions(&mut self.vehicles, c.restitution, c.friction, c.iterations) {
                self.events.push(RaceEvent::VehicleImpact(ev));
                for car in [ev.car_a_idx, ev.car_b_idx] {
                    self.impact(car, ev.closing_speed);
                }
            }
        }

        // Wall and obstacle boundary collisions for each car (including grandstands & tree trunks)
        let scenery_obstacles = track.geometry.all_obstacles_with_scenery();
        for i in 0..n_cars {
            let car = &mut self.vehicles[i];
            let mut wall_events = resolve_all_wall_collisions(car, &track.geometry.inner_walls, &scenery_obstacles);
            wall_events.extend(resolve_all_wall_collisions(car, &track.geometry.outer_walls, &[]));
            for ev in wall_events {
                self.events.push(RaceEvent::WallImpact { car: i, event: ev });
                self.impact(i, ev.impact_speed);
            }
        }

        // Race progression, lap tracking, sector splits
        for i in 0..n_cars {
            self.trackers[i].update(&self.vehicles[i], &track.spline, &track.checkpoints, dt);
        }

        self.time += dt;
        if let RaceFormat::Laps(laps) = self.rules.format {
            for i in 0..n_cars {
                if self.finish[i] == FinishState::Racing && self.trackers[i].current_lap > laps {
                    let position = 1 + self.finish.iter().filter(|f| matches!(f, FinishState::Finished { .. })).count();
                    self.finish[i] = FinishState::Finished { time: self.time, position };
                    self.events.push(RaceEvent::Finished { car: i, position, time: self.time });
                }
            }
        }
        &self.events
    }

    fn impact(&mut self, car: usize, speed: f32) {
        if car >= self.vehicles.len() || self.finish[car] != FinishState::Racing {
            return;
        }
        if let Some(cause) = self.vehicles[car].on_impact(speed) {
            self.finish[car] = FinishState::Dnf { time: self.time, cause };
            self.events.push(RaceEvent::Wrecked { car, cause });
        }
    }

    /// Vehicle indices from first to last: finished vehicles in finish order, then racing
    /// vehicles by lap and progress, then wrecked vehicles, the latest wreck first.
    pub fn standings(&self) -> Vec<usize> {
        let group = |i: usize| match self.finish.get(i).copied().unwrap_or(FinishState::Racing) {
            FinishState::Finished { position, .. } => (0, position as f32),
            FinishState::Racing => (1, 0.0),
            FinishState::Dnf { time, .. } => (2, -time),
        };
        let mut indices: Vec<usize> = (0..self.vehicles.len()).collect();
        indices.sort_by(|&a, &b| {
            let (ga, ka) = group(a);
            let (gb, kb) = group(b);
            if ga != gb {
                return ga.cmp(&gb);
            }
            if ga != 1 {
                return ka.partial_cmp(&kb).unwrap_or(Ordering::Equal);
            }
            let tr_a = &self.trackers[a];
            let tr_b = &self.trackers[b];
            // Primary: Lap number descending
            if tr_a.current_lap != tr_b.current_lap {
                return tr_b.current_lap.cmp(&tr_a.current_lap);
            }
            // Secondary: Normalized track progress descending
            tr_b.normalized_progress.partial_cmp(&tr_a.normalized_progress).unwrap_or(Ordering::Equal)
        });
        indices
    }

    /// One row per vehicle, in standings order.
    ///
    /// In a `Laps` race, a vehicle still racing gets a projected time: the race time plus the
    /// distance it still has to drive, divided by its average speed so far (at least 1 m/s).
    /// Projected times never come before the time of the row above.
    pub fn results(&self, track: &Track) -> Vec<ParticipantResult> {
        let lap_len = track.spline.total_length();
        let mut rows = Vec::with_capacity(self.vehicles.len());
        let mut prev_time = 0.0f32;
        for (rank, car) in self.standings().into_iter().enumerate() {
            let state = self.finish.get(car).copied().unwrap_or(FinishState::Racing);
            let tracker = &self.trackers[car];
            let (time, projected) = match (state, self.rules.format) {
                (FinishState::Finished { time, .. }, _) | (FinishState::Dnf { time, .. }, _) => (time, false),
                (FinishState::Racing, RaceFormat::TimeAttack) => (self.time, false),
                (FinishState::Racing, RaceFormat::Laps(laps)) => {
                    let done = tracker.current_lap.saturating_sub(1) as f32 + tracker.normalized_progress;
                    let remaining = (laps as f32 - done).max(0.0) * lap_len;
                    let avg_speed = if self.time > 0.0 { tracker.total_distance_travelled / self.time } else { 0.0 };
                    ((self.time + remaining / avg_speed.max(1.0)).max(prev_time), true)
                }
            };
            if !matches!(state, FinishState::Dnf { .. }) {
                prev_time = time;
            }
            rows.push(ParticipantResult { car, position: rank + 1, state, time, projected, best_lap: tracker.best_lap_time });
        }
        rows
    }
}
