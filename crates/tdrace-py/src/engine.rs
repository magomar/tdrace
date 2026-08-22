use glam::Vec2;
use numpy::ndarray::Array3;
use numpy::{PyArray1, PyArray3};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::f32::consts::PI;

use tdrace_core::collision::car_collision::resolve_multi_car_collisions;
use tdrace_core::collision::wall::resolve_all_wall_collisions;
use tdrace_core::lidar::{LidarHit, LidarScanner};
use tdrace_core::physics::car::{normalize_angle, Car, CarControls};
use tdrace_core::physics::config::CarConfig;
use tdrace_core::track::checkpoint::TrackProgressTracker;
use tdrace_core::track::geometry::WallBarrier;
use tdrace_core::track::presets::{
    classic_grand_prix, drift_park, kart_arena, oval_speedway,
};
use tdrace_core::track::Track;

use crate::config::{parse_car_config, parse_lidar_config, RewardConfig};
use crate::rasterizer::{FastRasterizer, SkidMark};

/// Deterministic Xorshift64 PRNG.
#[inline]
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    if x == 0 {
        x = 0x9E3779B97F4A7C15;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

#[inline]
fn rand_uniform_f32(state: &mut u64, min_val: f32, max_val: f32) -> f32 {
    let r = (xorshift64(state) >> 40) as f32 / 16777216.0; // 24-bit float in [0, 1)
    min_val + r * (max_val - min_val)
}

/// High-performance headless multi-agent racing simulation engine.
#[pyclass(name = "Engine")]
pub struct PyEngine {
    track: Track,
    walls: Vec<WallBarrier>,
    cars: Vec<Car>,
    car_configs: Vec<CarConfig>,
    trackers: Vec<TrackProgressTracker>,
    scanners: Vec<LidarScanner>,
    rasterizer: FastRasterizer,
    reward_config: RewardConfig,
    skid_marks: Vec<SkidMark>,
    prev_steer: Vec<f32>,
    step_count: usize,
    max_episode_steps: usize,
    dt: f32,
    num_agents: usize,
    seed: u64,
    rng_state: u64,
    collision_restitution: f32,
    collision_friction: f32,
    terminate_on_off_track: bool,
    terminate_on_wall_crash: bool,
    lap_limit: u32,
    obs_buffer: Vec<f32>,
    ray_hits_buffer: Vec<LidarHit>,
}

#[pymethods]
impl PyEngine {
    #[new]
    #[pyo3(signature = (
        track_name="classic_grand_prix",
        num_agents=1,
        car_type="sports_car",
        num_lidar_rays=19,
        max_episode_steps=1000,
        dt=0.016666667,
        reward_config=None,
        lap_limit=1,
        terminate_on_off_track=false,
        terminate_on_wall_crash=false
    ))]
    pub fn new(
        track_name: &str,
        num_agents: usize,
        car_type: &str,
        num_lidar_rays: usize,
        max_episode_steps: usize,
        dt: f32,
        reward_config: Option<RewardConfig>,
        lap_limit: u32,
        terminate_on_off_track: bool,
        terminate_on_wall_crash: bool,
    ) -> Self {
        let num_agents = num_agents.max(1);
        let track = match track_name.to_lowercase().as_str() {
            "drift" | "drift_park" => drift_park(),
            "oval" | "oval_speedway" => oval_speedway(),
            "kart" | "kart_arena" => kart_arena(),
            _ => classic_grand_prix(),
        };

        let walls: Vec<WallBarrier> = track.geometry.all_walls().cloned().collect();
        let base_car_cfg = parse_car_config(car_type);
        let lidar_cfg = parse_lidar_config("", Some(num_lidar_rays));
        let reward_cfg = reward_config.unwrap_or_default();

        let rasterizer = FastRasterizer::new(&track);

        let mut cars = Vec::with_capacity(num_agents);
        let mut car_configs = Vec::with_capacity(num_agents);
        let mut trackers = Vec::with_capacity(num_agents);
        let mut scanners = Vec::with_capacity(num_agents);
        let mut prev_steer = Vec::with_capacity(num_agents);

        let num_checkpoints = track.checkpoints.len();
        let num_sectors = track.checkpoints.iter().map(|c| c.sector).max().unwrap_or(0) + 1;

        for i in 0..num_agents {
            let spawn = if i < track.grid_positions.len() {
                track.grid_positions[i]
            } else {
                track.grid_positions[0]
            };

            let car = Car::new(base_car_cfg).with_pose(spawn.position, spawn.angle);
            cars.push(car);
            car_configs.push(base_car_cfg);
            trackers.push(TrackProgressTracker::new(num_checkpoints, num_sectors));
            scanners.push(LidarScanner::new(lidar_cfg));
            prev_steer.push(0.0);
        }

        let obs_dim = Self::compute_obs_dim(num_agents, lidar_cfg.num_rays);

        Self {
            track,
            walls,
            cars,
            car_configs,
            trackers,
            scanners,
            rasterizer,
            reward_config: reward_cfg,
            skid_marks: Vec::with_capacity(256),
            prev_steer,
            step_count: 0,
            max_episode_steps,
            dt,
            num_agents,
            seed: 0,
            rng_state: 0x123456789ABCDEF0,
            collision_restitution: 0.8,
            collision_friction: 0.3,
            terminate_on_off_track,
            terminate_on_wall_crash,
            lap_limit,
            obs_buffer: vec![0.0; obs_dim],
            ray_hits_buffer: vec![LidarHit::default(); lidar_cfg.num_rays],
        }
    }

    /// Dimension of the flattened float32 observation vector for a single car.
    #[getter]
    pub fn obs_dim(&self) -> usize {
        Self::compute_obs_dim(self.num_agents, self.scanners[0].config.num_rays)
    }

    /// Number of active agents in the environment.
    #[getter]
    pub fn num_agents(&self) -> usize {
        self.num_agents
    }

    /// Current track name.
    #[getter]
    pub fn track_name(&self) -> String {
        self.track.name.clone()
    }

    /// Total track length in meters.
    #[getter]
    pub fn track_length(&self) -> f32 {
        self.track.spline.total_length()
    }

    /// Total number of checkpoints.
    #[getter]
    pub fn checkpoints_count(&self) -> usize {
        self.track.checkpoints.len()
    }

    /// Angles (radians relative to car heading) of all LIDAR beams.
    pub fn get_ray_angles(&self) -> Vec<f32> {
        self.scanners[0].compute_ray_angles()
    }

    /// Resets the engine state, repositioning all cars on the starting grid.
    #[pyo3(signature = (seed=None, randomize_pose=false))]
    pub fn reset(
        &mut self,
        py: Python<'_>,
        seed: Option<u64>,
        randomize_pose: bool,
    ) -> PyResult<(Py<PyAny>, Py<PyAny>)> {
        if let Some(s) = seed {
            self.seed = s;
            self.rng_state = if s == 0 { 0x9E3779B97F4A7C15 } else { s };
        }

        self.step_count = 0;
        self.skid_marks.clear();

        for i in 0..self.num_agents {
            let spawn = if i < self.track.grid_positions.len() {
                self.track.grid_positions[i]
            } else {
                self.track.grid_positions[0]
            };

            let mut pos = spawn.position;
            let mut heading = spawn.angle;

            if randomize_pose {
                let jitter_x = rand_uniform_f32(&mut self.rng_state, -0.5, 0.5);
                let jitter_y = rand_uniform_f32(&mut self.rng_state, -0.5, 0.5);
                let jitter_h = rand_uniform_f32(&mut self.rng_state, -0.05, 0.05);
                pos += Vec2::new(jitter_x, jitter_y);
                heading += jitter_h;
            }

            self.cars[i] = Car::new(self.car_configs[i]).with_pose(pos, heading);
            self.trackers[i].reset();
            self.trackers[i].update(&self.cars[i], &self.track.spline, &self.track.checkpoints, 0.0);
            self.prev_steer[i] = 0.0;
        }

        if self.num_agents == 1 {
            let obs = self.build_agent_obs(py, 0)?;
            let info = self.build_info_dict(py, 0, 0.0, 0.0, 0.0, 0.0, false, false)?;
            Ok((obs.into_any().unbind(), info.into_any().unbind()))
        } else {
            let obs_list = PyList::empty(py);
            let info_list = PyList::empty(py);
            for i in 0..self.num_agents {
                obs_list.append(self.build_agent_obs(py, i)?)?;
                info_list.append(self.build_info_dict(py, i, 0.0, 0.0, 0.0, 0.0, false, false)?)?;
            }
            Ok((obs_list.into_any().unbind(), info_list.into_any().unbind()))
        }
    }

    /// Fast single-agent step path avoiding vector allocations.
    #[pyo3(signature = (throttle=0.0, steer=0.0, brake=0.0, handbrake=false, reverse=false))]
    pub fn step_single(
        &mut self,
        py: Python<'_>,
        throttle: f32,
        steer: f32,
        brake: f32,
        handbrake: bool,
        reverse: bool,
    ) -> PyResult<(Py<PyAny>, f32, bool, bool, Py<PyAny>)> {
        let safe_throttle = if throttle.is_finite() { throttle.clamp(-1.0, 1.0) } else { 0.0 };
        let safe_steer = if steer.is_finite() { steer.clamp(-1.0, 1.0) } else { 0.0 };
        let safe_brake = if brake.is_finite() { brake.clamp(0.0, 1.0) } else { 0.0 };

        let ctrl = CarControls {
            throttle: safe_throttle,
            steer: safe_steer,
            brake: safe_brake,
            handbrake,
            reverse,
        };

        // 1. Record pre-step progress
        let prev_progress = self.trackers[0].progress_distance;
        let prev_laps = self.trackers[0].current_lap;

        // 2. Sample surface & step car physics
        let surfaces = self.track.sample_car_surfaces(&self.cars[0]);
        self.cars[0].step_per_wheel(&ctrl, surfaces, self.dt);

        // 3. Collision resolution against walls & obstacles (broadphase filtered)
        let mut wall_impulse = 0.0;
        let mut wall_hit = false;
        let car_pos = self.cars[0].state.position;
        let candidate_walls: Vec<WallBarrier> = self
            .walls
            .iter()
            .filter(|w| {
                let mid = (w.segment.start + w.segment.end) * 0.5;
                mid.distance_squared(car_pos) < 64.0 // 8m radius
            })
            .cloned()
            .collect();

        let wall_events = resolve_all_wall_collisions(
            &mut self.cars[0],
            &candidate_walls,
            &self.track.geometry.obstacles,
        );
        for ev in wall_events {
            wall_impulse += ev.normal_impulse;
            wall_hit = true;
        }

        // 4. Update track progress tracker
        self.trackers[0].update(&self.cars[0], &self.track.spline, &self.track.checkpoints, self.dt);

        // 5. Update skid marks
        self.record_skid_marks(0);

        // 6. Reward shaping calculation with closed-loop wrapping
        let track_len = self.track.spline.total_length();
        let mut progress_delta = self.trackers[0].progress_distance - prev_progress;
        if track_len > 0.0 {
            if progress_delta > track_len * 0.5 {
                progress_delta -= track_len;
            } else if progress_delta < -track_len * 0.5 {
                progress_delta += track_len;
            }
        }

        let lap_completed = self.trackers[0].current_lap > prev_laps || self.trackers[0].lap_completed;
        let is_off_track = self.trackers[0].is_off_track;
        let is_wrong_way = self.trackers[0].is_wrong_way;
        let speed = if self.cars[0].state.speed.is_finite() { self.cars[0].state.speed } else { 0.0 };
        let drift_score = if self.cars[0].state.is_drifting && self.cars[0].state.sideslip_angle.is_finite() {
            self.cars[0].state.sideslip_angle.abs() * speed * self.dt
        } else {
            0.0
        };

        let steer_delta = if (ctrl.steer - self.prev_steer[0]).is_finite() {
            (ctrl.steer - self.prev_steer[0]).abs()
        } else {
            0.0
        };
        self.prev_steer[0] = ctrl.steer;

        let mut reward = 0.0f32;
        reward += progress_delta * self.reward_config.progress_weight;
        reward += speed * self.dt * self.reward_config.speed_weight;
        reward += drift_score * self.reward_config.drift_weight;
        if lap_completed {
            reward += self.reward_config.lap_completion_reward;
        }
        if wall_hit {
            reward -= self.reward_config.wall_collision_penalty * (wall_impulse / 1000.0f32).clamp(0.5f32, 3.0f32);
        }
        if is_off_track {
            reward -= self.reward_config.grass_penalty * self.dt;
        }
        if is_wrong_way {
            reward -= self.reward_config.wrong_way_penalty * self.dt;
        }
        reward -= self.reward_config.idle_penalty;
        reward -= steer_delta * self.reward_config.action_smoothness_penalty;

        if !reward.is_finite() {
            reward = 0.0;
        }

        // 7. Termination & Truncation checks
        self.step_count += 1;
        let mut terminated = false;
        let mut truncated = false;

        if self.lap_limit > 0 && self.trackers[0].current_lap > self.lap_limit {
            terminated = true;
        }

        if self.trackers[0].wrong_way_timer > 2.0 {
            terminated = true;
            reward -= self.reward_config.wrong_way_penalty * 5.0;
        }

        if self.terminate_on_off_track && self.trackers[0].off_track_timer > 2.0 {
            terminated = true;
            reward -= self.reward_config.out_of_bounds_penalty;
        }

        if self.terminate_on_wall_crash && wall_impulse > 15000.0 {
            terminated = true;
        }

        if self.max_episode_steps > 0 && self.step_count >= self.max_episode_steps {
            truncated = true;
        }

        let obs = self.build_agent_obs(py, 0)?;
        let info = self.build_info_dict(
            py,
            0,
            progress_delta,
            wall_impulse,
            drift_score,
            reward,
            lap_completed,
            wall_hit,
        )?;

        Ok((obs.into_any().unbind(), reward, terminated, truncated, info.into_any().unbind()))
    }

    /// Step all agents simultaneously with multi-car collisions and independent rewards.
    pub fn step_multi(
        &mut self,
        py: Python<'_>,
        actions: Vec<(f32, f32, f32, bool, bool)>,
    ) -> PyResult<(Py<PyAny>, Py<PyAny>, Py<PyAny>, Py<PyAny>, Py<PyAny>)> {
        let n = self.num_agents.min(actions.len());

        let mut prev_progress = Vec::with_capacity(n);
        let mut prev_laps = Vec::with_capacity(n);

        // 1. Pre-step progress
        for i in 0..n {
            prev_progress.push(self.trackers[i].progress_distance);
            prev_laps.push(self.trackers[i].current_lap);
        }

        // 2. Step each car physics
        for i in 0..n {
            let (throttle, steer, brake, handbrake, reverse) = actions[i];
            let safe_throttle = if throttle.is_finite() { throttle.clamp(-1.0, 1.0) } else { 0.0 };
            let safe_steer = if steer.is_finite() { steer.clamp(-1.0, 1.0) } else { 0.0 };
            let safe_brake = if brake.is_finite() { brake.clamp(0.0, 1.0) } else { 0.0 };

            let ctrl = CarControls {
                throttle: safe_throttle,
                steer: safe_steer,
                brake: safe_brake,
                handbrake,
                reverse,
            };
            let surfaces = self.track.sample_car_surfaces(&self.cars[i]);
            self.cars[i].step_per_wheel(&ctrl, surfaces, self.dt);
        }

        // 3. Resolve multi-car collisions
        let mut car_collisions = vec![false; n];
        if n > 1 {
            let events = resolve_multi_car_collisions(
                &mut self.cars,
                self.collision_restitution,
                self.collision_friction,
                2,
            );
            for ev in events {
                car_collisions[ev.car_a_idx] = true;
                car_collisions[ev.car_b_idx] = true;
            }
        }

        // 4. Resolve wall & obstacle collisions for all cars (broadphase filtered)
        let mut wall_impulses = vec![0.0f32; n];
        let mut wall_hits = vec![false; n];
        for i in 0..n {
            let car_pos = self.cars[i].state.position;
            let candidate_walls: Vec<WallBarrier> = self
                .walls
                .iter()
                .filter(|w| {
                    let mid = (w.segment.start + w.segment.end) * 0.5;
                    mid.distance_squared(car_pos) < 64.0
                })
                .cloned()
                .collect();

            let wall_events = resolve_all_wall_collisions(
                &mut self.cars[i],
                &candidate_walls,
                &self.track.geometry.obstacles,
            );
            for ev in wall_events {
                wall_impulses[i] += ev.normal_impulse;
                wall_hits[i] = true;
            }

            self.trackers[i].update(&self.cars[i], &self.track.spline, &self.track.checkpoints, self.dt);
            self.record_skid_marks(i);
        }

        self.step_count += 1;
        let is_truncated = self.max_episode_steps > 0 && self.step_count >= self.max_episode_steps;

        let track_len = self.track.spline.total_length();
        let obs_list = PyList::empty(py);
        let rewards_list = PyList::empty(py);
        let terms_list = PyList::empty(py);
        let truncs_list = PyList::empty(py);
        let infos_list = PyList::empty(py);

        for i in 0..n {
            let mut progress_delta = self.trackers[i].progress_distance - prev_progress[i];
            if track_len > 0.0 {
                if progress_delta > track_len * 0.5 {
                    progress_delta -= track_len;
                } else if progress_delta < -track_len * 0.5 {
                    progress_delta += track_len;
                }
            }

            let lap_completed = self.trackers[i].current_lap > prev_laps[i] || self.trackers[i].lap_completed;
            let is_off_track = self.trackers[i].is_off_track;
            let is_wrong_way = self.trackers[i].is_wrong_way;
            let speed = if self.cars[i].state.speed.is_finite() { self.cars[i].state.speed } else { 0.0 };
            let drift_score = if self.cars[i].state.is_drifting && self.cars[i].state.sideslip_angle.is_finite() {
                self.cars[i].state.sideslip_angle.abs() * speed * self.dt
            } else {
                0.0
            };

            let steer_input = if actions[i].1.is_finite() { actions[i].1 } else { 0.0 };
            let steer_delta = if (steer_input - self.prev_steer[i]).is_finite() {
                (steer_input - self.prev_steer[i]).abs()
            } else {
                0.0
            };
            self.prev_steer[i] = steer_input;

            let mut reward = 0.0f32;
            reward += progress_delta * self.reward_config.progress_weight;
            reward += speed * self.dt * self.reward_config.speed_weight;
            reward += drift_score * self.reward_config.drift_weight;
            if lap_completed {
                reward += self.reward_config.lap_completion_reward;
            }
            if wall_hits[i] {
                reward -= self.reward_config.wall_collision_penalty * (wall_impulses[i] / 1000.0f32).clamp(0.5f32, 3.0f32);
            }
            if car_collisions[i] {
                reward -= self.reward_config.opponent_collision_penalty;
            }
            if is_off_track {
                reward -= self.reward_config.grass_penalty * self.dt;
            }
            if is_wrong_way {
                reward -= self.reward_config.wrong_way_penalty * self.dt;
            }
            reward -= self.reward_config.idle_penalty;
            reward -= steer_delta * self.reward_config.action_smoothness_penalty;

            if !reward.is_finite() {
                reward = 0.0;
            }

            let mut terminated = false;
            if self.lap_limit > 0 && self.trackers[i].current_lap > self.lap_limit {
                terminated = true;
            }
            if self.trackers[i].wrong_way_timer > 2.0 {
                terminated = true;
                reward -= self.reward_config.wrong_way_penalty * 5.0;
            }
            if self.terminate_on_off_track && self.trackers[i].off_track_timer > 2.0 {
                terminated = true;
                reward -= self.reward_config.out_of_bounds_penalty;
            }

            obs_list.append(self.build_agent_obs(py, i)?)?;
            rewards_list.append(reward)?;
            terms_list.append(terminated)?;
            truncs_list.append(is_truncated)?;
            infos_list.append(self.build_info_dict(
                py,
                i,
                progress_delta,
                wall_impulses[i],
                drift_score,
                reward,
                lap_completed,
                wall_hits[i],
            )?)?;
        }

        Ok((
            obs_list.into_any().unbind(),
            rewards_list.into_any().unbind(),
            terms_list.into_any().unbind(),
            truncs_list.into_any().unbind(),
            infos_list.into_any().unbind(),
        ))
    }

    /// Renders the top-down RGB software pixel observation of shape (height, width, 3).
    #[pyo3(signature = (agent_idx=0, width=96, height=96, follow_car=true, scale=None))]
    pub fn render_rgb<'py>(
        &self,
        py: Python<'py>,
        agent_idx: usize,
        width: usize,
        height: usize,
        follow_car: bool,
        scale: Option<f32>,
    ) -> PyResult<Bound<'py, PyArray3<u8>>> {
        let idx = agent_idx.min(self.cars.len() - 1);
        let target_car = &self.cars[idx];

        // Opponents list excluding target car
        let opponents: Vec<Car> = self
            .cars
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, c)| c.clone())
            .collect();

        let mut buffer = vec![0u8; width * height * 3];
        self.rasterizer.render(
            target_car,
            &opponents,
            &self.track.geometry.obstacles,
            &self.skid_marks,
            width,
            height,
            follow_car,
            scale,
            &mut buffer,
        );

        let array = Array3::from_shape_vec((height, width, 3), buffer)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Array error: {e}")))?;

        // Return PyArray3
        Ok(PyArray3::from_owned_array(py, array))
    }

    /// Returns detailed physics and telemetry dictionary for an agent.
    pub fn get_telemetry<'py>(&self, py: Python<'py>, agent_idx: usize) -> PyResult<Bound<'py, PyDict>> {
        let idx = agent_idx.min(self.cars.len() - 1);
        let car = &self.cars[idx];
        let tracker = &self.trackers[idx];

        let dict = PyDict::new(py);
        dict.set_item(pyo3::intern!(py, "speed_mps"), car.state.speed)?;
        dict.set_item(pyo3::intern!(py, "speed_kmh"), car.speed_kmh())?;
        dict.set_item(pyo3::intern!(py, "pos_x"), car.state.position.x)?;
        dict.set_item(pyo3::intern!(py, "pos_y"), car.state.position.y)?;
        dict.set_item(pyo3::intern!(py, "vel_x"), car.state.velocity.x)?;
        dict.set_item(pyo3::intern!(py, "vel_y"), car.state.velocity.y)?;
        dict.set_item(pyo3::intern!(py, "local_vel_x"), car.state.local_velocity.x)?;
        dict.set_item(pyo3::intern!(py, "local_vel_y"), car.state.local_velocity.y)?;
        dict.set_item(pyo3::intern!(py, "angle"), car.state.angle)?;
        dict.set_item(pyo3::intern!(py, "angular_velocity"), car.state.angular_velocity)?;
        dict.set_item(pyo3::intern!(py, "steer_angle"), car.state.steer_angle)?;
        dict.set_item(pyo3::intern!(py, "sideslip_angle"), car.state.sideslip_angle)?;
        dict.set_item(pyo3::intern!(py, "is_drifting"), car.state.is_drifting)?;
        dict.set_item(pyo3::intern!(py, "drift_score"), car.state.drift_score)?;
        dict.set_item(pyo3::intern!(py, "current_lap"), tracker.current_lap)?;
        dict.set_item(pyo3::intern!(py, "lap_time"), tracker.lap_time)?;
        dict.set_item(pyo3::intern!(py, "best_lap_time"), tracker.best_lap_time)?;
        dict.set_item(pyo3::intern!(py, "last_lap_time"), tracker.last_lap_time)?;
        dict.set_item(pyo3::intern!(py, "progress_distance"), tracker.progress_distance)?;
        dict.set_item(pyo3::intern!(py, "normalized_progress"), tracker.normalized_progress)?;
        dict.set_item(pyo3::intern!(py, "is_off_track"), tracker.is_off_track)?;
        dict.set_item(pyo3::intern!(py, "is_wrong_way"), tracker.is_wrong_way)?;

        // Per-wheel skid & loads
        let wheel_skids = PyList::new(py, car.state.wheels.iter().map(|w| w.skid_intensity))?;
        let wheel_loads = PyList::new(py, car.state.wheels.iter().map(|w| w.normal_load))?;
        dict.set_item(pyo3::intern!(py, "wheel_skid_intensities"), wheel_skids)?;
        dict.set_item(pyo3::intern!(py, "wheel_normal_loads"), wheel_loads)?;

        Ok(dict)
    }

    /// Sets explicit vehicle pose and velocity (useful for testing, checkpoints, and save/restore).
    pub fn set_state(
        &mut self,
        agent_idx: usize,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        angle: f32,
        angular_velocity: f32,
    ) {
        let idx = agent_idx.min(self.cars.len() - 1);
        self.cars[idx].state.position = Vec2::new(x, y);
        self.cars[idx].state.velocity = Vec2::new(vx, vy);
        self.cars[idx].state.angle = normalize_angle(angle);
        self.cars[idx].state.angular_velocity = angular_velocity;
        self.cars[idx].state.speed = Vec2::new(vx, vy).length();
    }

    /// Gets current car state tuple: (x, y, vx, vy, angle, angular_vel, steer_angle).
    pub fn get_state(&self, agent_idx: usize) -> (f32, f32, f32, f32, f32, f32, f32) {
        let idx = agent_idx.min(self.cars.len() - 1);
        let s = &self.cars[idx].state;
        (s.position.x, s.position.y, s.velocity.x, s.velocity.y, s.angle, s.angular_velocity, s.steer_angle)
    }
}

impl PyEngine {
    #[inline]
    fn compute_obs_dim(num_agents: usize, num_lidar_rays: usize) -> usize {
        let base_features = 26;
        let lidar_features = num_lidar_rays;
        let multi_agent_features = if num_agents > 1 { 4 } else { 0 };
        base_features + lidar_features + multi_agent_features
    }

    fn record_skid_marks(&mut self, agent_idx: usize) {
        let car = &self.cars[agent_idx];
        let wheel_positions = car.wheel_positions_world();
        for (i, &w_pos) in wheel_positions.iter().enumerate() {
            if car.state.wheels[i].is_skidding {
                let fwd = car.forward_vector();
                self.skid_marks.push(SkidMark {
                    start: w_pos - fwd * 0.3,
                    end: w_pos,
                    width: 0.22,
                    alpha: car.state.wheels[i].skid_intensity,
                });
            }
        }

        // Limit skid marks capacity to avoid memory accumulation
        if self.skid_marks.len() > 300 {
            self.skid_marks.drain(0..100);
        }
    }

    fn build_agent_obs<'py>(&mut self, py: Python<'py>, agent_idx: usize) -> PyResult<Bound<'py, PyArray1<f32>>> {
        let car = &self.cars[agent_idx];
        let tracker = &self.trackers[agent_idx];
        let scanner = &self.scanners[agent_idx];

        // 1. Vehicle dynamics features
        let max_steer = car.config.max_steer_angle.max(1e-3);
        let v_long = car.state.local_velocity.x;
        let v_lat = car.state.local_velocity.y;

        self.obs_buffer[0] = v_long / 50.0;
        self.obs_buffer[1] = v_lat / 25.0;
        self.obs_buffer[2] = car.state.angular_velocity / 10.0;
        self.obs_buffer[3] = (car.state.steer_angle / max_steer).clamp(-1.0, 1.0);
        self.obs_buffer[4] = car.state.speed / 50.0;
        self.obs_buffer[5] = (car.state.sideslip_angle / PI).clamp(-1.0, 1.0);
        self.obs_buffer[6] = if car.state.is_drifting { 1.0 } else { 0.0 };
        self.obs_buffer[7] = car.state.acceleration_local.x / 20.0;
        self.obs_buffer[8] = car.state.acceleration_local.y / 20.0;

        // 2. Track spline relative state
        let proj = self.track.spline.project_point(car.state.position);
        let half_w = (proj.track_width * 0.5).max(1.0);
        self.obs_buffer[9] = (proj.lateral_offset / half_w).clamp(-3.0, 3.0);

        let heading_diff = normalize_angle(car.state.angle - proj.tangent.y.atan2(proj.tangent.x));
        self.obs_buffer[10] = heading_diff / PI;
        self.obs_buffer[11] = tracker.normalized_progress;

        // Curvature ahead at +5m, +15m, +30m
        let s_curr = proj.progress_distance;
        let compute_curvature = |s: f32| -> f32 {
            let t0 = self.track.spline.sample_at_distance(s).tangent;
            let t1 = self.track.spline.sample_at_distance(s + 2.0).tangent;
            (t0.x * t1.y - t0.y * t1.x) / 2.0
        };
        let c5 = compute_curvature(s_curr + 5.0);
        let c15 = compute_curvature(s_curr + 15.0);
        let c30 = compute_curvature(s_curr + 30.0);
        self.obs_buffer[12] = (c5 * 50.0).clamp(-1.0, 1.0);
        self.obs_buffer[13] = (c15 * 50.0).clamp(-1.0, 1.0);
        self.obs_buffer[14] = (c30 * 50.0).clamp(-1.0, 1.0);

        // 3. Wheel surface friction & skid intensity
        let wheel_surfs = self.track.sample_car_surfaces(car);
        for i in 0..4 {
            self.obs_buffer[15 + i] = wheel_surfs[i].friction_coefficient();
            self.obs_buffer[19 + i] = car.state.wheels[i].skid_intensity;
        }

        // 4. Next checkpoint vector in car local frame
        let next_cp_idx = tracker.next_checkpoint_idx.min(self.track.checkpoints.len() - 1);
        let cp = &self.track.checkpoints[next_cp_idx];
        let cp_center = (cp.gate.start + cp.gate.end) * 0.5;
        let delta_cp_world = cp_center - car.state.position;
        let fwd = car.forward_vector();
        let right = car.right_vector();
        let cp_dist = delta_cp_world.length();
        let cp_dir = if cp_dist > 1e-3 { delta_cp_world / cp_dist } else { Vec2::X };
        self.obs_buffer[23] = cp_dir.dot(fwd);
        self.obs_buffer[24] = cp_dir.dot(right);
        self.obs_buffer[25] = (cp_dist / 100.0).clamp(0.0, 2.0);

        // 5. LIDAR raycast hits
        let num_rays = scanner.config.num_rays;
        if num_rays > 0 {
            let opponents: Vec<Car> = self
                .cars
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != agent_idx)
                .map(|(_, c)| c.clone())
                .collect();

            if self.ray_hits_buffer.len() < num_rays {
                self.ray_hits_buffer.resize(num_rays, LidarHit::default());
            }

            scanner.scan_into(car, &self.track, &opponents, &mut self.ray_hits_buffer[..num_rays]);

            for i in 0..num_rays {
                self.obs_buffer[26 + i] = self.ray_hits_buffer[i].normalized_distance;
            }
        }

        // 6. Multi-agent nearest opponent features
        if self.num_agents > 1 {
            let mut min_dist_sq = f32::MAX;
            let mut nearest_idx = None;

            for i in 0..self.num_agents {
                if i == agent_idx {
                    continue;
                }
                let d_sq = self.cars[i].state.position.distance_squared(car.state.position);
                if d_sq < min_dist_sq {
                    min_dist_sq = d_sq;
                    nearest_idx = Some(i);
                }
            }

            let start_idx = 26 + num_rays;
            if let Some(opp_i) = nearest_idx {
                let opp = &self.cars[opp_i];
                let delta_pos = opp.state.position - car.state.position;
                let delta_vel = opp.state.velocity - car.state.velocity;

                self.obs_buffer[start_idx] = (delta_pos.dot(fwd) / 50.0).clamp(-1.0, 1.0);
                self.obs_buffer[start_idx + 1] = (delta_pos.dot(right) / 50.0).clamp(-1.0, 1.0);
                self.obs_buffer[start_idx + 2] = (delta_vel.dot(fwd) / 50.0).clamp(-1.0, 1.0);
                self.obs_buffer[start_idx + 3] = (delta_vel.dot(right) / 50.0).clamp(-1.0, 1.0);
            } else {
                self.obs_buffer[start_idx..start_idx + 4].fill(0.0);
            }
        }

        Ok(PyArray1::from_slice(py, &self.obs_buffer))
    }

    fn build_info_dict<'py>(
        &self,
        py: Python<'py>,
        agent_idx: usize,
        progress_delta: f32,
        wall_impulse: f32,
        drift_score: f32,
        step_reward: f32,
        lap_completed: bool,
        wall_hit: bool,
    ) -> PyResult<Bound<'py, PyDict>> {
        let car = &self.cars[agent_idx];
        let tracker = &self.trackers[agent_idx];

        let dict = PyDict::new(py);
        dict.set_item(pyo3::intern!(py, "speed_mps"), car.state.speed)?;
        dict.set_item(pyo3::intern!(py, "speed_kmh"), car.speed_kmh())?;
        dict.set_item(pyo3::intern!(py, "progress_distance"), tracker.progress_distance)?;
        dict.set_item(pyo3::intern!(py, "normalized_progress"), tracker.normalized_progress)?;
        dict.set_item(pyo3::intern!(py, "progress_delta"), progress_delta)?;
        dict.set_item(pyo3::intern!(py, "current_lap"), tracker.current_lap)?;
        dict.set_item(pyo3::intern!(py, "lap_time"), tracker.lap_time)?;
        dict.set_item(pyo3::intern!(py, "lap_completed"), lap_completed)?;
        dict.set_item(pyo3::intern!(py, "best_lap_time"), tracker.best_lap_time)?;
        dict.set_item(pyo3::intern!(py, "is_off_track"), tracker.is_off_track)?;
        dict.set_item(pyo3::intern!(py, "is_wrong_way"), tracker.is_wrong_way)?;
        dict.set_item(pyo3::intern!(py, "is_drifting"), car.state.is_drifting)?;
        dict.set_item(pyo3::intern!(py, "drift_score"), car.state.drift_score)?;
        dict.set_item(pyo3::intern!(py, "step_drift_score"), drift_score)?;
        dict.set_item(pyo3::intern!(py, "wall_hit"), wall_hit)?;
        dict.set_item(pyo3::intern!(py, "wall_impulse"), wall_impulse)?;
        dict.set_item(pyo3::intern!(py, "step_reward"), step_reward)?;
        dict.set_item(pyo3::intern!(py, "step_count"), self.step_count)?;

        Ok(dict)
    }
}
