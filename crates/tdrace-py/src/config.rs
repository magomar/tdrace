use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use tdrace_core::lidar::LidarConfig;
use tdrace_core::physics::config::CarConfig;

/// Configurable reward shaping weights for RL training.
#[pyclass(get_all, set_all)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RewardConfig {
    /// Reward multiplier for forward distance traveled along track centerline spline (meters).
    pub progress_weight: f32,
    /// Reward multiplier for forward velocity (m/s * dt).
    pub speed_weight: f32,
    /// Reward multiplier for controlled drifting (sideslip_angle * speed * dt).
    pub drift_weight: f32,
    /// One-time reward bonus for completing a full lap.
    pub lap_completion_reward: f32,
    /// Penalty applied on colliding with track walls and barriers.
    pub wall_collision_penalty: f32,
    /// Penalty per second (scaled by dt) spent driving on grass / off-track surfaces.
    pub grass_penalty: f32,
    /// Penalty per second (scaled by dt) spent driving the wrong direction.
    pub wrong_way_penalty: f32,
    /// Constant per-step penalty to encourage fast completion.
    pub idle_penalty: f32,
    /// Penalty applied when episode terminates due to going completely out of bounds.
    pub out_of_bounds_penalty: f32,
    /// Penalty for high steering rate / jerky inputs.
    pub action_smoothness_penalty: f32,
    /// Penalty applied on vehicle-to-vehicle collision.
    pub opponent_collision_penalty: f32,
}

#[pymethods]
impl RewardConfig {
    #[new]
    #[pyo3(signature = (
        progress_weight=1.0,
        speed_weight=0.05,
        drift_weight=0.0,
        lap_completion_reward=100.0,
        wall_collision_penalty=2.0,
        grass_penalty=0.2,
        wrong_way_penalty=1.0,
        idle_penalty=0.01,
        out_of_bounds_penalty=10.0,
        action_smoothness_penalty=0.0,
        opponent_collision_penalty=1.0
    ))]
    pub fn new(
        progress_weight: f32,
        speed_weight: f32,
        drift_weight: f32,
        lap_completion_reward: f32,
        wall_collision_penalty: f32,
        grass_penalty: f32,
        wrong_way_penalty: f32,
        idle_penalty: f32,
        out_of_bounds_penalty: f32,
        action_smoothness_penalty: f32,
        opponent_collision_penalty: f32,
    ) -> Self {
        Self {
            progress_weight,
            speed_weight,
            drift_weight,
            lap_completion_reward,
            wall_collision_penalty,
            grass_penalty,
            wrong_way_penalty,
            idle_penalty,
            out_of_bounds_penalty,
            action_smoothness_penalty,
            opponent_collision_penalty,
        }
    }

    /// Standard racing reward preset (speed and forward track progress).
    #[staticmethod]
    pub fn standard_racing() -> Self {
        Self::default()
    }

    /// Drift scoring reward preset.
    #[staticmethod]
    pub fn drift_challenge() -> Self {
        Self {
            progress_weight: 0.5,
            speed_weight: 0.1,
            drift_weight: 2.5,
            lap_completion_reward: 100.0,
            wall_collision_penalty: 5.0,
            grass_penalty: 0.5,
            wrong_way_penalty: 2.0,
            idle_penalty: 0.01,
            out_of_bounds_penalty: 15.0,
            action_smoothness_penalty: 0.0,
            opponent_collision_penalty: 2.0,
        }
    }

    /// Aggressive time attack preset.
    #[staticmethod]
    pub fn time_attack() -> Self {
        Self {
            progress_weight: 2.0,
            speed_weight: 0.1,
            drift_weight: 0.0,
            lap_completion_reward: 200.0,
            wall_collision_penalty: 10.0,
            grass_penalty: 1.0,
            wrong_way_penalty: 5.0,
            idle_penalty: 0.05,
            out_of_bounds_penalty: 20.0,
            action_smoothness_penalty: 0.01,
            opponent_collision_penalty: 5.0,
        }
    }
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            progress_weight: 1.0,
            speed_weight: 0.05,
            drift_weight: 0.0,
            lap_completion_reward: 100.0,
            wall_collision_penalty: 2.0,
            grass_penalty: 0.2,
            wrong_way_penalty: 1.0,
            idle_penalty: 0.01,
            out_of_bounds_penalty: 10.0,
            action_smoothness_penalty: 0.0,
            opponent_collision_penalty: 1.0,
        }
    }
}

/// Helper to parse CarConfig from string name.
pub fn parse_car_config(name: &str) -> CarConfig {
    match name.to_lowercase().as_str() {
        "drift" | "drift_car" => CarConfig::drift_car(),
        "kart" | "go_kart" => CarConfig::kart(),
        "rally" | "rally_car" => CarConfig::rally_car(),
        _ => CarConfig::sports_car(),
    }
}

/// Helper to parse LidarConfig from string name or ray count.
pub fn parse_lidar_config(name: &str, num_rays: Option<usize>) -> LidarConfig {
    if let Some(rays) = num_rays {
        if rays == 0 {
            return LidarConfig {
                num_rays: 0,
                fov_radians: 0.0,
                max_range: 0.0,
                offset_forward: 0.0,
                angle_offset: 0.0,
            };
        } else if rays == 19 {
            return LidarConfig::gym_carracing_19();
        } else if rays == 16 {
            return LidarConfig::forward_cone_16();
        } else if rays == 64 {
            return LidarConfig::surround_64();
        } else {
            return LidarConfig {
                num_rays: rays,
                fov_radians: std::f32::consts::PI,
                max_range: 50.0,
                offset_forward: 1.2,
                angle_offset: 0.0,
            };
        }
    }

    match name.to_lowercase().as_str() {
        "cone16" | "forward_16" => LidarConfig::forward_cone_16(),
        "surround64" | "360" => LidarConfig::surround_64(),
        "none" | "disabled" => LidarConfig {
            num_rays: 0,
            fov_radians: 0.0,
            max_range: 0.0,
            offset_forward: 0.0,
            angle_offset: 0.0,
        },
        _ => LidarConfig::gym_carracing_19(),
    }
}
