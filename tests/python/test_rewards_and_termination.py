"""
Reward shaping, termination, and truncation tests.
"""

import gymnasium as gym
import numpy as np
import pytest
import tdrace


def test_progress_reward_forward_driving():
    """Tests that moving forward along the circuit generates positive progress reward."""
    env = gym.make("TDRace-v0")
    env.reset(seed=42)

    total_reward = 0.0
    for _ in range(50):
        obs, reward, term, trunc, info = env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))
        total_reward += reward

    assert total_reward > 0.0
    assert info["progress_distance"] > 10.0
    env.close()


def test_drift_score_reward():
    """Tests that controlled sliding in TDRace-Drift-v0 generates high drift rewards."""
    env = gym.make("TDRace-Drift-v0")
    env.reset(seed=42)

    # Accelerate up to speed
    for _ in range(60):
        env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))

    # Initiate drift with handbrake + hard steer
    drift_score_accum = 0.0
    for _ in range(30):
        obs, reward, term, trunc, info = env.step(np.array([1.0, 0.7, 0.0, 1.0], dtype=np.float32))
        drift_score_accum += info["step_drift_score"]

    assert info["drift_score"] > 0.0 or drift_score_accum > 0.0, "Drift score should accumulate"
    env.close()


def test_truncation_at_max_episode_steps():
    """Tests that episode truncates exactly at max_episode_steps."""
    max_steps = 40
    env = gym.make("TDRace-v0", max_episode_steps=max_steps)
    env.reset(seed=42)

    truncated_step = None
    for step in range(1, max_steps + 10):
        obs, rew, term, trunc, info = env.step(np.array([0.0, 0.5, 0.0], dtype=np.float32))
        if trunc:
            truncated_step = step
            break

    assert truncated_step == max_steps
    env.close()


def test_termination_wrong_way():
    """Tests that driving in the wrong direction for >3 seconds triggers episode termination."""
    env = gym.make("TDRace-v0")
    obs, info = env.reset(seed=42)

    # Turn around 180 degrees from starting pose and drive backwards
    curr_state = env.unwrapped.get_state()
    x, y, vx, vy, angle, ang_vel, steer = curr_state
    # Point exactly opposite to track forward direction
    env.unwrapped.set_state(x, y, 0.0, 0.0, angle + np.pi, 0.0)

    terminated = False
    for _ in range(250):  # > 3 seconds at 60 Hz
        obs, rew, term, trunc, info = env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))
        if term:
            terminated = True
            break

    assert terminated, "Should terminate after prolonged wrong-way driving"
    env.close()
