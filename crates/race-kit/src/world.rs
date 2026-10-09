//! The race in progress: one step order for every game.
//!
//! Spec 056 (`specs/056_racekit_headless_race_world.md`). The step order is the one of
//! `RaceSession::physics_step` in `tdrace-app`, without sounds, camera or particles.

use std::cmp::Ordering;

use arcade_race_core::collision::{resolve_all_wall_collisions, resolve_car_car_collision};
use arcade_race_core::track::{LineSegment, Track, TrackProgressTracker};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use wheelbase::{Car, SurfaceType};

use crate::events::{DnfCause, RaceEvent};
use crate::vehicle::{CanopyBrush, DriveControls, Vehicle};

/// Health a pit stop restores to the chassis, the engine and each suspension corner, up to the
/// field repair caps in `wheelbase::car` (Spec 062, Spec 078).
pub const PIT_STOP_REPAIR_AMOUNT: f32 = 0.25;

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

/// Rallycross joker rule: each vehicle must drive the joker route `mandatory` times.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct JokerRule {
    /// Joker laps each vehicle must complete. 0 turns the rule off.
    pub mandatory: u32,
    /// Seconds added to the finish time of a vehicle with fewer jokers than `mandatory`.
    pub penalty_s: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RaceRules {
    pub format: RaceFormat,
    pub collision: CollisionParams,
    #[serde(default)]
    pub damage_enabled: bool,
    #[serde(default)]
    pub joker: JokerRule,
}

impl Default for RaceRules {
    fn default() -> Self {
        Self {
            format: RaceFormat::Laps(3),
            collision: CollisionParams::default(),
            damage_enabled: false,
            joker: JokerRule::default(),
        }
    }
}

/// Runtime state of a vehicle navigating the pit lane.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PitServiceState {
    NotPitting,
    InTransit { distance: f32 },
    StationaryInBox { timer: f32, target_duration: f32 },
    ServiceComplete { release_time: f32 },
}

impl PitServiceState {
    #[inline]
    pub fn is_limiter_active(&self) -> bool {
        !matches!(self, Self::NotPitting)
    }

    #[inline]
    pub fn are_controls_locked(&self) -> bool {
        matches!(self, Self::StationaryInBox { .. })
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
    /// Seconds added for missing joker laps (see [`JokerRule`]). Not included in `time`; rows are
    /// ordered by `time + penalty`.
    pub penalty: f32,
    /// Joker laps the vehicle completed.
    pub jokers: u32,
}

/// How a vehicle left a sprint stage (see [`RaceWorld::stage_classification`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageOutcome {
    /// The vehicle crossed the line: its time is real.
    Finished,
    /// The vehicle was wrecked. It is classified after every other vehicle.
    Dnf,
    /// The vehicle was still racing when the stage was read: its time is a projection.
    Unfinished,
}

/// One row of [`RaceWorld::stage_classification`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageFinish {
    /// Vehicle index in [`RaceWorld::vehicles`].
    pub car: usize,
    /// Position in the stage, starting at 1. Wrecked vehicles come last.
    pub position: usize,
    pub outcome: StageOutcome,
    /// Race time in seconds including the joker penalty. `None` unless the vehicle finished, so a
    /// wreck time or a projection never seeds a later stage.
    pub time: Option<f32>,
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
    /// Runtime pit service state for each vehicle.
    pub pit_states: Vec<PitServiceState>,
    /// Highest speed of each vehicle, in m/s, read after the step and canopy drag.
    pub top_speed: Vec<f32>,
    /// Surfaces under each vehicle in the last step, for effects and sounds.
    pub last_surfaces: Vec<[SurfaceType; 4]>,
    /// Joker penalty of each vehicle in seconds, set when it finishes (see [`JokerRule`]).
    pub penalty: Vec<f32>,
    /// Race time in seconds: the sum of every step's `dt`.
    pub time: f32,
    events: Vec<RaceEvent>,
    brushes: Vec<CanopyBrush>,
    prev_positions: Vec<Vec2>,
    drafts: Vec<f32>,
}

impl<V: Vehicle> RaceWorld<V> {
    pub fn new(rules: RaceRules) -> Self {
        Self {
            rules,
            vehicles: Vec::new(),
            trackers: Vec::new(),
            finish: Vec::new(),
            pit_states: Vec::new(),
            top_speed: Vec::new(),
            last_surfaces: Vec::new(),
            penalty: Vec::new(),
            time: 0.0,
            events: Vec::new(),
            brushes: Vec::new(),
            prev_positions: Vec::new(),
            drafts: Vec::new(),
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
        self.pit_states.clear();
        self.top_speed.clear();
        self.last_surfaces.clear();
        self.penalty.clear();
        self.time = 0.0;
        self.events.clear();
        self.prev_positions.clear();
        self.drafts.clear();
    }

    /// Keeps the per-vehicle state as long as `vehicles`, for callers that push vehicles directly.
    fn fit(&mut self) {
        let n = self.vehicles.len();
        self.finish.resize(n, FinishState::Racing);
        self.pit_states.resize(n, PitServiceState::NotPitting);
        self.top_speed.resize(n, 0.0);
        self.last_surfaces.resize(n, [SurfaceType::Asphalt; 4]);
        self.penalty.resize(n, 0.0);
        self.prev_positions.resize(n, Vec2::ZERO);
        self.drafts.resize(n, 0.0);
    }

    /// The events of the last step, in the order they happened.
    pub fn events(&self) -> &[RaceEvent] {
        &self.events
    }

    pub fn is_finished(&self, car: usize) -> bool {
        matches!(self.finish.get(car), Some(FinishState::Finished { .. }))
    }

    /// Joker laps vehicle `car` has completed. 0 on a track without a network.
    pub fn jokers_taken(&self, car: usize) -> u32 {
        self.trackers.get(car).and_then(|t| t.multi_route.as_ref()).map_or(0, |m| m.joker_laps_completed)
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

        // Aerodynamic slipstream wake drafting between cars (zero-allocation stack buffer for <= 32 cars)
        for i in 0..n_cars {
            let intensity = if n_cars <= 32 {
                let mut buf = [&self.vehicles[0]; 32];
                let mut count = 0;
                for (idx, v) in self.vehicles.iter().enumerate() {
                    if idx != i {
                        buf[count] = v;
                        count += 1;
                    }
                }
                self.vehicles[i].draft_intensity(&buf[..count])
            } else {
                let other_refs: Vec<&V> =
                    self.vehicles.iter().enumerate().filter(|(idx, _)| *idx != i).map(|(_, c)| c).collect();
                self.vehicles[i].draft_intensity(&other_refs)
            };
            self.drafts[i] = intensity;
        }
        for i in 0..n_cars {
            self.vehicles[i].set_draft(self.drafts[i]);
        }

        // Track vehicle positions before step for crossing checks (zero-allocation scratch buffer)
        for i in 0..n_cars {
            self.prev_positions[i] = self.vehicles[i].position();
        }

        // Step individual vehicle dynamics and update road elevation & cross-slope banking
        for i in 0..n_cars {
            let prev_prog = self.trackers.get(i).map(|tp| tp.progress_distance).unwrap_or(0.0);
            // Off the main road the joker or chute road gives the height: the walls there stand at its height, and
            // a car at the main road's height drove through them (tdrace-joker-wall-ghost-ebrgn).
            let proj = track.project_point_continuity(self.vehicles[i].position(), prev_prog, 50.0);
            self.vehicles[i].set_road(&proj);

            let mut ctrl = match self.finish[i] {
                FinishState::Dnf { .. } => DriveControls::default(),
                _ => controls.get(i).copied().unwrap_or_default(),
            };
            if self.pit_states[i].are_controls_locked() {
                ctrl = DriveControls {
                    throttle: 0.0,
                    brake: 1.0,
                    handbrake: true,
                    steer: 0.0,
                    ..Default::default()
                };
            }
            self.vehicles[i].step(&ctrl, self.last_surfaces[i], dt);

            // Controls lock or pit speed limiter enforcement
            if self.pit_states[i].are_controls_locked() {
                let v = self.vehicles[i].velocity();
                self.vehicles[i].add_velocity(-v);
            } else if self.pit_states[i].is_limiter_active() {
                let pit_speed_limit = track.pit_lane.as_ref().map_or(16.67, |lane| lane.speed_limit);
                let spd = self.vehicles[i].speed();
                if spd > pit_speed_limit {
                    let v = self.vehicles[i].velocity();
                    let excess = spd - pit_speed_limit;
                    let dir = v.normalize_or_zero();
                    self.vehicles[i].add_velocity(-dir * excess);
                }
            }

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
        // Ghost collision safety: cars stationary in pit box are neutralized
        if n_cars > 1 && self.rules.collision.iterations > 0 {
            let c = self.rules.collision;
            for iter in 0..c.iterations {
                for i in 0..n_cars {
                    if self.pit_states.get(i).map_or(false, |s| s.are_controls_locked()) {
                        continue;
                    }
                    for j in (i + 1)..n_cars {
                        if self.pit_states.get(j).map_or(false, |s| s.are_controls_locked()) {
                            continue;
                        }
                        let (left, right) = self.vehicles.split_at_mut(j);
                        let car_i = &mut left[i];
                        let car_j = &mut right[0];
                        if let Some(mut ev) = resolve_car_car_collision(car_i, car_j, c.restitution, c.friction) {
                            if iter == 0 {
                                ev.car_a_idx = i;
                                ev.car_b_idx = j;
                                let damage_energy = ev.estimated_damage_energy();
                                if self.rules.damage_enabled {
                                    car_i.apply_collision_damage(ev.contact_point, damage_energy);
                                    car_j.apply_collision_damage(ev.contact_point, damage_energy);
                                }
                                self.events.push(RaceEvent::VehicleImpact(ev));
                                self.impact(i, ev.closing_speed);
                                self.impact(j, ev.closing_speed);
                            }
                        }
                    }
                }
            }
        }

        // Wall and obstacle boundary collisions for each car (including grandstands & tree trunks)
        let scenery_obstacles = track.geometry.all_obstacles_with_scenery();
        for i in 0..n_cars {
            let wall_events = {
                let car = &mut self.vehicles[i];
                let mut events = resolve_all_wall_collisions(car, &track.geometry.inner_walls, scenery_obstacles);
                events.extend(resolve_all_wall_collisions(car, &track.geometry.outer_walls, &[]));
                events.extend(resolve_all_wall_collisions(car, &track.geometry.network_walls, &[]));
                for ev in &events {
                    let damage_energy = ev.estimated_damage_energy();
                    if self.rules.damage_enabled {
                        car.apply_collision_damage(ev.contact_point, damage_energy);
                    }
                }
                events
            };
            for ev in wall_events {
                self.events.push(RaceEvent::WallImpact { car: i, event: ev });
                self.impact(i, ev.impact_speed);
            }
        }

        // Race progression, lap tracking, sector splits, pit service state machine
        for i in 0..n_cars {
            let was_in_pit = self.trackers[i].in_pit_lane;
            if let Some(net) = &track.network {
                self.trackers[i].update_network(&self.vehicles[i], net, &track.checkpoints, dt);
            } else {
                self.trackers[i].update(&self.vehicles[i], &track.spline, &track.checkpoints, dt);
            }

            // Also check track.pit_lane entry and exit line segments if defined
            if let Some(lane) = &track.pit_lane {
                let car_seg = LineSegment::new(self.prev_positions[i], self.vehicles[i].position());
                if lane.entry_gate.intersect_segment(&car_seg).is_some() {
                    self.trackers[i].in_pit_lane = true;
                    self.trackers[i].has_stopped_in_pit_box = false;
                }
                if lane.exit_gate.intersect_segment(&car_seg).is_some() {
                    if self.trackers[i].in_pit_lane && self.trackers[i].has_stopped_in_pit_box {
                        self.trackers[i].pit_stops += 1;
                    }
                    self.trackers[i].in_pit_lane = false;
                    self.trackers[i].has_stopped_in_pit_box = false;
                }
            }

            let now_in_pit = self.trackers[i].in_pit_lane;

            if !was_in_pit && now_in_pit {
                if self.pit_states[i] == PitServiceState::NotPitting {
                    self.pit_states[i] = PitServiceState::InTransit { distance: 0.0 };
                    self.events.push(RaceEvent::PitEntry { car: i });
                    let pit_speed_limit = track.pit_lane.as_ref().map_or(16.67, |lane| lane.speed_limit);
                    let spd = self.vehicles[i].speed();
                    if spd > pit_speed_limit {
                        let v = self.vehicles[i].velocity();
                        let excess = spd - pit_speed_limit;
                        let dir = v.normalize_or_zero();
                        self.vehicles[i].add_velocity(-dir * excess);
                    }
                }
            } else if was_in_pit && !now_in_pit {
                self.pit_states[i] = PitServiceState::NotPitting;
                self.events.push(RaceEvent::PitExit { car: i });
            }

            match self.pit_states[i] {
                PitServiceState::InTransit { distance } => {
                    let new_dist = distance + self.vehicles[i].speed() * dt;
                    if track.is_in_pit_box(&self.vehicles[i]) && self.vehicles[i].speed() < 1.5 {
                        let v = self.vehicles[i].velocity();
                        self.vehicles[i].add_velocity(-v);
                        self.pit_states[i] = PitServiceState::StationaryInBox {
                            timer: 0.0,
                            target_duration: 2.5,
                        };
                        self.events.push(RaceEvent::PitServiceStart { car: i });
                    } else {
                        self.pit_states[i] = PitServiceState::InTransit { distance: new_dist };
                    }
                }
                PitServiceState::StationaryInBox { timer, target_duration } => {
                    let new_timer = timer + dt;
                    if new_timer >= target_duration {
                        self.vehicles[i].service_tires();
                        self.vehicles[i].apply_field_repair(PIT_STOP_REPAIR_AMOUNT);
                        self.trackers[i].has_stopped_in_pit_box = true;
                        self.pit_states[i] = PitServiceState::ServiceComplete {
                            release_time: self.time,
                        };
                        self.events.push(RaceEvent::PitServiceComplete { car: i });
                    } else {
                        self.pit_states[i] = PitServiceState::StationaryInBox {
                            timer: new_timer,
                            target_duration,
                        };
                    }
                }
                _ => {}
            }
        }

        self.time += dt;
        if let RaceFormat::Laps(laps) = self.rules.format {
            for i in 0..n_cars {
                if self.finish[i] == FinishState::Racing && self.trackers[i].current_lap > laps {
                    let position = 1 + self.finish.iter().filter(|f| matches!(f, FinishState::Finished { .. })).count();
                    self.finish[i] = FinishState::Finished { time: self.time, position };
                    if self.jokers_taken(i) < self.rules.joker.mandatory {
                        self.penalty[i] = self.rules.joker.penalty_s;
                    }
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

    /// Vehicle indices from first to last: finished vehicles by finish time plus joker penalty (finish
    /// order breaks ties), then racing vehicles by lap and progress, then wrecked vehicles, the latest
    /// wreck first.
    pub fn standings(&self) -> Vec<usize> {
        let group = |i: usize| match self.finish.get(i).copied().unwrap_or(FinishState::Racing) {
            FinishState::Finished { time, .. } => (0, time + self.penalty.get(i).copied().unwrap_or(0.0)),
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
            if ga == 0 {
                let pos = |i: usize| match self.finish[i] {
                    FinishState::Finished { position, .. } => position,
                    _ => 0,
                };
                return ka.partial_cmp(&kb).unwrap_or(Ordering::Equal).then(pos(a).cmp(&pos(b)));
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
    /// Projected times never come before the time of a vehicle that finished ahead. Rows are ordered by
    /// `time + penalty`, with wrecked vehicles last, so a penalised finisher can drop behind a vehicle
    /// that is still racing.
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
            rows.push(ParticipantResult {
                car,
                position: rank + 1,
                state,
                time,
                projected,
                best_lap: tracker.best_lap_time,
                penalty: self.penalty[car],
                jokers: self.jokers_taken(car),
            });
        }
        // Without penalties the rows are already in this order and the sort changes nothing.
        rows.sort_by(|a, b| {
            let dnf = |r: &ParticipantResult| matches!(r.state, FinishState::Dnf { .. });
            if dnf(a) || dnf(b) {
                return dnf(a).cmp(&dnf(b));
            }
            (a.time + a.penalty).partial_cmp(&(b.time + b.penalty)).unwrap_or(Ordering::Equal)
        });
        for (rank, row) in rows.iter_mut().enumerate() {
            row.position = rank + 1;
        }
        rows
    }

    /// True when every vehicle has finished or been wrecked: a sprint stage (a heat, a semifinal or a
    /// final) has nothing left to decide. An empty world is not complete.
    pub fn is_stage_complete(&self) -> bool {
        !self.finish.is_empty() && self.finish.iter().all(|f| !matches!(f, FinishState::Racing))
    }

    /// The classification of a sprint stage, in order: finished vehicles by time plus joker penalty, then
    /// vehicles still racing, then wrecked vehicles last (Spec 104). Only a finished vehicle has a time.
    pub fn stage_classification(&self, track: &Track) -> Vec<StageFinish> {
        self.results(track)
            .into_iter()
            .map(|row| {
                let (outcome, time) = match row.state {
                    FinishState::Finished { .. } => (StageOutcome::Finished, Some(row.time + row.penalty)),
                    FinishState::Dnf { .. } => (StageOutcome::Dnf, None),
                    FinishState::Racing => (StageOutcome::Unfinished, None),
                };
                StageFinish { car: row.car, position: row.position, outcome, time, best_lap: row.best_lap }
            })
            .collect()
    }
}
