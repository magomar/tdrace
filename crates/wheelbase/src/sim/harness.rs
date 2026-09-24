use glam::Vec2;

use crate::car::{Car, CarControls};
use crate::config::CarConfig;
use crate::surface::SurfaceType;
use super::telemetry::TelemetryPoint;

/// Default deterministic simulation timestep: 120 Hz (approx 8.33 ms).
pub const DEFAULT_SIMULATION_DT: f32 = 1.0 / 120.0;

/// Headless physics simulation runner for executing deterministic vehicle dynamics sweeps.
pub struct SimulationRunner {
    /// Simulated vehicle model.
    pub car: Car,
    /// Cumulative simulation time in seconds.
    pub time: f32,
    /// Fixed physics timestep in seconds.
    pub dt: f32,
    /// High-frequency telemetry log buffer.
    pub telemetry: Vec<TelemetryPoint>,
    /// Whether to record a `TelemetryPoint` on every step.
    pub capture_telemetry: bool,
}

impl SimulationRunner {
    /// Creates a new simulation runner initialized at origin with zero velocity.
    pub fn new(config: CarConfig, dt: f32) -> Self {
        Self {
            car: Car::new(config),
            time: 0.0,
            dt,
            telemetry: Vec::new(),
            capture_telemetry: true,
        }
    }

    /// Sets initial pose (position and heading angle) and linear velocity.
    pub fn with_state(mut self, position: Vec2, angle: f32, velocity: Vec2) -> Self {
        self.car.state_mut().position = position;
        self.car.state_mut().angle = angle;
        self.car.set_velocity(velocity);
        self
    }

    /// Enables or disables telemetry point capture during steps.
    pub fn with_telemetry_capture(mut self, enabled: bool) -> Self {
        self.capture_telemetry = enabled;
        self
    }

    /// Clears the telemetry log and resets elapsed time.
    pub fn reset_telemetry(&mut self) {
        self.telemetry.clear();
        self.time = 0.0;
    }

    /// Steps the physics simulation forward by one fixed timestep `dt`.
    #[inline]
    pub fn step(&mut self, controls: &CarControls, surface: SurfaceType) {
        self.car.step(controls, surface, self.dt);
        self.time += self.dt;
        if self.capture_telemetry {
            self.telemetry.push(TelemetryPoint::capture(&self.car, self.time));
        }
    }

    /// Runs the simulation for a fixed elapsed duration in seconds.
    pub fn run_for<F>(&mut self, duration_s: f32, surface: SurfaceType, mut controls_fn: F)
    where
        F: FnMut(f32, &Car) -> CarControls,
    {
        let start_time = self.time;
        while (self.time - start_time) < duration_s {
            let elapsed = self.time - start_time;
            let controls = controls_fn(elapsed, &self.car);
            self.step(&controls, surface);
        }
    }

    /// Runs the simulation until `should_terminate` returns true or `max_duration_s` is exceeded.
    /// Returns `true` if terminated via predicate, `false` if timed out.
    pub fn run_until<F, C>(
        &mut self,
        max_duration_s: f32,
        surface: SurfaceType,
        mut controls_fn: F,
        mut should_terminate: C,
    ) -> bool
    where
        F: FnMut(f32, &Car) -> CarControls,
        C: FnMut(f32, &Car) -> bool,
    {
        let start_time = self.time;
        while (self.time - start_time) < max_duration_s {
            let elapsed = self.time - start_time;
            let controls = controls_fn(elapsed, &self.car);
            self.step(&controls, surface);
            if should_terminate(self.time - start_time, &self.car) {
                return true;
            }
        }
        false
    }

    /// Steps the physics simulation forward by one fixed timestep `dt` with per-wheel surfaces.
    #[inline]
    pub fn step_per_wheel(&mut self, controls: &CarControls, surfaces: [SurfaceType; 4]) {
        self.car.step_per_wheel(controls, surfaces, self.dt);
        self.time += self.dt;
        if self.capture_telemetry {
            self.telemetry.push(TelemetryPoint::capture(&self.car, self.time));
        }
    }

    /// Runs the simulation until `should_terminate` returns true or `max_duration_s` is exceeded,
    /// using per-wheel surface assignments.
    pub fn run_per_wheel_until<F, C>(
        &mut self,
        max_duration_s: f32,
        surfaces: [SurfaceType; 4],
        mut controls_fn: F,
        mut should_terminate: C,
    ) -> bool
    where
        F: FnMut(f32, &Car) -> CarControls,
        C: FnMut(f32, &Car) -> bool,
    {
        let start_time = self.time;
        while (self.time - start_time) < max_duration_s {
            let elapsed = self.time - start_time;
            let controls = controls_fn(elapsed, &self.car);
            self.step_per_wheel(&controls, surfaces);
            if should_terminate(self.time - start_time, &self.car) {
                return true;
            }
        }
        false
    }
}
